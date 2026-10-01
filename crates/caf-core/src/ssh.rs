//! Real SSH command surface implemented with `ssh2` and Tokio async channel routing.
//! Active sessions are maintained in an in-memory session registry.
//!
//! `connect` takes only a saved host id — never raw connection details from the frontend.
//! It resolves the host record and its decrypted credential via
//! `crate::store::load_host_for_connect` internally, so a plaintext password/passphrase
//! never has to cross the Tauri IPC boundary on every connect (it was already saved,
//! encrypted, once via `store::save_host`).
//!
//! Two kinds of session live here and they are deliberately kept apart:
//!
//! * **PTY sessions** (`SESSIONS`) back an interactive terminal pane. Their `ssh2::Session`
//!   is switched to non-blocking and is owned by a reader thread that polls it continuously.
//! * **Exec sessions** (`EXEC_SESSIONS`, via [`with_exec_session`]) back every *non*-interactive
//!   consumer — SFTP, monitoring, AI step execution. They are blocking and pooled per host.
//!
//! Sharing one `ssh2::Session` between the two is not safe: a consumer that flips the shared
//! session to blocking mode leaves the PTY reader thread parked inside `read()` while it still
//! holds the channel lock, which freezes the terminal permanently. Every non-interactive caller
//! must therefore go through [`with_exec_session`] and must never touch a PTY session.

use crate::error::{CatermError, ValidationError};
use crate::store::{AuthMethod, HostRecord};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshSession {
    pub session_id: String,
    pub host_id: String,
}

/// Something that happened on a live PTY session, pushed out as it happens instead of
/// waiting for the frontend to ask. `caf-app` turns these into Tauri events (K2-1: the
/// core stays GUI-free and only knows about this callback).
#[derive(Debug, Clone)]
pub enum SshEvent {
    Output { session_id: String, data: String },
    Closed { session_id: String },
    OsDetected { host_id: String, os: String },
}

type EventSink = Box<dyn Fn(SshEvent) + Send + Sync + 'static>;

static EVENT_SINK: Lazy<Mutex<Option<EventSink>>> = Lazy::new(|| Mutex::new(None));

/// Registers the process-wide sink that receives PTY output as it arrives. Installing a sink
/// switches sessions from "buffer and wait for [`read`]" to push delivery — the buffer is then
/// left empty on purpose so the same bytes are never delivered twice.
///
/// # Infallible
/// Stores a callback in a process-wide slot; a poisoned lock leaves the previous sink in place
/// rather than failing, because there is no caller-recoverable outcome to report at startup.
pub fn set_event_sink(sink: impl Fn(SshEvent) + Send + Sync + 'static) {
    if let Ok(mut slot) = EVENT_SINK.lock() {
        *slot = Some(Box::new(sink));
    }
}

fn has_event_sink() -> bool {
    EVENT_SINK.lock().map(|s| s.is_some()).unwrap_or(false)
}

fn emit(event: SshEvent) {
    if let Ok(sink) = EVENT_SINK.lock()
        && let Some(sink) = sink.as_ref()
    {
        sink(event);
    }
}

pub(crate) struct SessionHandle {
    pub(crate) tx: std::sync::mpsc::Sender<Vec<u8>>,
    pub(crate) output_buffer: Arc<Mutex<Vec<u8>>>,
    pub(crate) input_buffer: Arc<Mutex<String>>,
    /// The last ~200 bytes of remote/local output actually emitted to the frontend, kept so
    /// `write()` can tell whether the terminal is mid password prompt before audit-logging a
    /// submitted line (C-09). Not a substitute for real echo-state tracking (ssh2 exposes none),
    /// just a heuristic against common prompt text.
    pub(crate) last_output_tail: Arc<Mutex<String>>,
    pub(crate) channel: Option<Arc<Mutex<ssh2::Channel>>>,
    /// The local-shell child process — a real PTY-backed one since the fix for the "Local
    /// Terminal" garbled-output bug (no job control, no `TERM`, misaligned prompts because the
    /// shell had no real terminal device at all). `portable_pty::Child` instead of
    /// `std::process::Child` because it was spawned via `PtySystem::openpty` +
    /// `PtyPair::slave.spawn_command`, not `std::process::Command` directly.
    pub(crate) child: Option<Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>>,
    /// The local PTY's master side, kept only so [`resize`] can propagate a frontend resize to
    /// it — `ssh2::Channel` already carries its own size via `request_pty_size`, this is the
    /// local-session equivalent.
    pub(crate) pty_master: Option<Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>>,
    pub(crate) host_id: String,
}

/// How much trailing output text `last_output_tail` keeps. Only needs to cover the tail end of
/// one prompt line ("[sudo] password for alice: "), not a whole screen.
const OUTPUT_TAIL_CAPACITY: usize = 200;

/// Caps how long a burst of PTY output is allowed to accumulate before being flushed as one
/// `SshEvent::Output`, when the remote keeps handing over more bytes read after read (`cat` on a
/// big file, `find /`, a noisy build). Below this, a single keystroke's echo still flushes
/// immediately via the "stream went quiet" branch, so typed latency is unaffected — this only
/// coalesces sustained floods that would otherwise emit (and cross the Tauri IPC boundary) once
/// per `read()` syscall, which measured over 17,000 events/sec on a `yes` -style flood.
const OUTPUT_BATCH_MAX_DELAY: Duration = Duration::from_millis(8);
/// Also flush early if a single burst already built up this much text, so one pathologically long
/// quiet-free stream doesn't grow `pending` unboundedly between time-based flushes.
const OUTPUT_BATCH_MAX_BYTES: usize = 64 * 1024;

/// Whether a currently-accumulating output batch should be flushed now. Pulled out as a pure
/// function (the read loop itself needs a live `ssh2::Channel` and can't run in a unit test) so
/// the coalescing thresholds above have a regression test.
fn should_flush_batch(pending_len: usize, batch_age: Duration) -> bool {
    pending_len >= OUTPUT_BATCH_MAX_BYTES || batch_age >= OUTPUT_BATCH_MAX_DELAY
}

/// Floor and ceiling for [`next_poll_delay`]. The reader loop below is non-blocking (`ch.read()`
/// returns `WouldBlock`/`Ok(0)` immediately rather than parking the thread), so without a sleep
/// between empty reads it would spin the CPU at 100% per open session. A fixed 1ms sleep fixed
/// that but replaced it with a *steady* per-session wakeup 1000 times a second for the entire
/// life of an idle pane — measurable with several panes left open overnight. Backing off toward
/// `POLL_MAX_DELAY` while the session stays quiet, and resetting to `POLL_MIN_DELAY` the instant
/// data shows up again, keeps interactive latency (a keystroke's echo) unaffected while cutting
/// the idle wakeup rate by more than an order of magnitude.
const POLL_MIN_DELAY: Duration = Duration::from_millis(1);
const POLL_MAX_DELAY: Duration = Duration::from_millis(40);

/// How long to sleep before the next poll, given how many consecutive empty reads have happened
/// since data last arrived. Pulled out as a pure function for the same reason as
/// [`should_flush_batch`] — the read loop needs a live channel and can't run in a unit test.
fn next_poll_delay(consecutive_idle_reads: u32) -> Duration {
    let shift = consecutive_idle_reads.min(6); // 1,2,4,8,16,32,64ms, clamped to POLL_MAX_DELAY
    let ms = (POLL_MIN_DELAY.as_millis() as u64).saturating_mul(1u64 << shift);
    Duration::from_millis(ms).min(POLL_MAX_DELAY)
}

/// Appends `text` to `tail`, keeping only the last [`OUTPUT_TAIL_CAPACITY`] bytes (cut at a
/// char boundary, since this holds UTF-8 text).
fn record_output_tail(tail: &Arc<Mutex<String>>, text: &str) {
    let Ok(mut t) = tail.lock() else { return };
    t.push_str(text);
    if t.len() > OUTPUT_TAIL_CAPACITY {
        let mut cut = t.len() - OUTPUT_TAIL_CAPACITY;
        while cut < t.len() && !t.is_char_boundary(cut) {
            cut += 1;
        }
        t.drain(..cut);
    }
}

/// Best-effort check for whether `tail` (recent terminal output) looks like it ends in a
/// password/passphrase prompt with nothing typed back yet — `sudo`, `su`, `ssh-add`, `passwd`,
/// login prompts, and similar all print one of these forms and then stop echoing input.
fn looks_like_password_prompt(tail: &str) -> bool {
    let lower = tail.to_ascii_lowercase();
    let trimmed = lower.trim_end().trim_end_matches(':').trim_end();
    trimmed.ends_with("password")
        || trimmed.ends_with("passphrase")
        || trimmed.contains("password for")
        || trimmed.contains("passphrase for")
}

pub(crate) static SESSIONS: Lazy<Arc<Mutex<HashMap<String, SessionHandle>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Disambiguates local-terminal session ids that would otherwise collide: the id is seeded from
/// a millisecond timestamp, which two `connect("local")` calls in the same millisecond (two tabs
/// restored from a saved layout at startup, or two tests running in parallel) can share. Mixing
/// two shells under one session id means one pane's keystrokes and output leak into the other's.
static LOCAL_SESSION_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Pooled blocking sessions for non-interactive work, one per host id.
static EXEC_SESSIONS: Lazy<Mutex<HashMap<String, Arc<Mutex<ssh2::Session>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn require_non_empty(field: &str, value: &str) -> Result<(), CatermError> {
    if value.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(format!(
            "{field} cannot be empty"
        ))));
    }
    Ok(())
}

fn invalid(message: String) -> CatermError {
    CatermError::Validation(ValidationError::Generic(message))
}

fn generate_session_id(host_id: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ssh-{host_id}-{nanos:x}")
}

/// libssh2 records a non-default port as `[host]:port` but only when it is told to. Its
/// `checkp` lookup applies that transformation internally while `add` does not, so the name
/// has to be normalised here — otherwise every connect to a non-22 port reports `NotFound`
/// and silently re-adds a duplicate TOFU entry, which defeats the whole check.
fn known_hosts_name(address: &str, port: u16) -> String {
    if port == 22 {
        address.to_string()
    } else {
        format!("[{address}]:{port}")
    }
}

/// Resolves `addr` (host:port, with the host DNS-resolvable or a bracketed IPv6 literal) and
/// tries each returned address in turn, returning the first successful TCP connection. Unlike
/// `SocketAddr::parse`, this accepts hostnames (`vps.example.com:22`, `localhost:22`) as well as
/// IP literals, and — when a name resolves to several addresses — falls back to the next one
/// instead of failing on the first that refuses the connection.
fn connect_resolved(addr: &str, timeout: Duration) -> std::io::Result<std::net::TcpStream> {
    use std::net::ToSocketAddrs;
    let mut last_err: Option<std::io::Error> = None;
    for candidate in addr.to_socket_addrs()? {
        match std::net::TcpStream::connect_timeout(&candidate, timeout) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "address resolved to no candidates",
        )
    }))
}

/// TCP connect + handshake + TOFU host key verification + user authentication for a saved
/// host. Returns a **blocking** authenticated session; interactive callers switch it to
/// non-blocking themselves once their channel is up.
///
/// This is the single place where a CAFramework SSH connection is established. Every consumer
/// (PTY, SFTP, monitoring, AI) goes through it so host-key verification can never be
/// accidentally skipped by one of them.
pub(crate) fn open_authenticated_session(
    host_id: &str,
) -> Result<(ssh2::Session, HostRecord), CatermError> {
    require_non_empty("host_id", host_id)?;

    let (host, secret) = crate::store::load_host_for_connect(host_id)?;

    let port = if host.port == 0 { 22 } else { host.port };
    let addr = format!("{}:{port}", host.address);
    // Bracket a bare IPv6 literal (e.g. "::1") so it parses as one host, not host:port:port.
    let lookup_addr = if host.address.contains(':') && !host.address.starts_with('[') {
        format!("[{}]:{port}", host.address)
    } else {
        addr.clone()
    };

    let tcp = connect_resolved(&lookup_addr, Duration::from_secs(10))
        .map_err(|e| invalid(format!("Connection failed to {addr}: {e}")))?;

    tcp.set_nonblocking(false)
        .map_err(|e| invalid(format!("Failed to set blocking TCP stream: {e}")))?;

    let mut sess =
        ssh2::Session::new().map_err(|e| invalid(format!("SSH session creation failed: {e}")))?;

    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| invalid(format!("SSH handshake failed: {e}")))?;

    verify_host_key(&sess, &host, port)?;
    authenticate(&sess, &host, secret)?;

    if !sess.authenticated() {
        return Err(invalid(format!(
            "Authentication failed for user {}",
            host.username
        )));
    }

    Ok((sess, host))
}

/// TOFU Host Key Verification (REQ-18, T2-SSH-04).
fn verify_host_key(sess: &ssh2::Session, host: &HostRecord, port: u16) -> Result<(), CatermError> {
    let data_info = crate::paths::resolve_data_dir()?;
    let kh_file = data_info.path.join("known_hosts");
    if let Some(parent) = kh_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut known_hosts = sess
        .known_hosts()
        .map_err(|e| invalid(format!("Failed to initialize known_hosts: {e}")))?;

    if kh_file.exists() {
        // A missing file is the normal first-connection-ever case and is handled below via
        // `CheckResult::NotFound`. An *existing* file that fails to read (corrupted, permission
        // denied, unexpectedly a directory) is different: silently treating that the same as "no
        // known hosts recorded" would make every previously-pinned host look brand new, TOFU-add
        // whatever key the server happens to present, and paper over both storage corruption and
        // a real MITM that happens to coincide with it. Fail closed instead.
        known_hosts
            .read_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH)
            .map_err(|e| {
                invalid(format!(
                    "Could not read the known_hosts file at {}: {e}. Host-key verification \
                     cannot proceed safely until this is fixed — check the file's permissions \
                     and that it isn't corrupted.",
                    kh_file.display()
                ))
            })?;
    }

    let Some((key, key_type)) = sess.host_key() else {
        return Err(invalid(
            "Remote server did not present a host key.".to_string(),
        ));
    };

    match known_hosts.check_port(&host.address, port, key) {
        ssh2::CheckResult::Match => Ok(()),
        ssh2::CheckResult::NotFound => {
            // Trust On First Use: record the new key.
            known_hosts
                .add(
                    &known_hosts_name(&host.address, port),
                    key,
                    &format!("Added by CAFramework for {}", host.label),
                    key_type.into(),
                )
                .map_err(|e| invalid(format!("Failed to record TOFU host key: {e}")))?;
            // If this write fails, the trust decision we just made lives only in this
            // in-process `known_hosts` handle: it vanishes the moment the session ends, and
            // every future connection silently re-runs TOFU from scratch with no memory of this
            // one — the exact "always trust whatever key shows up" behavior TOFU exists to avoid
            // after the first connection. Previously this was `let _ = ...`, discarded. Failing
            // the connection here is louder than real OpenSSH (which warns and continues), but
            // matches how the rest of this function already treats verification failures, and a
            // write failure here means the security property this function exists to provide
            // (remembering hosts across connections) silently isn't holding.
            known_hosts
                .write_file(&kh_file, ssh2::KnownHostFileKind::OpenSSH)
                .map_err(|e| {
                    invalid(format!(
                        "Verified this host's key but could not save it to {}: {e}. Future \
                         connections would silently trust whatever key is presented instead of \
                         checking against this one — check the file's permissions.",
                        kh_file.display()
                    ))
                })?;
            Ok(())
        }
        ssh2::CheckResult::Mismatch => Err(invalid(format!(
            "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED! Host key for {} ({}) does not match known_hosts record.",
            host.label, host.address
        ))),
        ssh2::CheckResult::Failure => Err(invalid(
            "Host key verification check failed unexpectedly.".to_string(),
        )),
    }
}

fn authenticate(
    sess: &ssh2::Session,
    host: &HostRecord,
    secret: Option<String>,
) -> Result<(), CatermError> {
    match &host.auth_method {
        AuthMethod::Password => {
            let password = secret.ok_or_else(|| {
                invalid(format!(
                    "Password belum diset untuk host '{}' — edit host dan isi password terlebih dahulu.",
                    host.label
                ))
            })?;
            sess.userauth_password(&host.username, &password)
                .map_err(|e| invalid(format!("Auth failed for user {}: {e}", host.username)))
        }
        AuthMethod::Key { path } => {
            let expanded = crate::paths::expand_tilde(path);
            sess.userauth_pubkey_file(
                &host.username,
                None,
                Path::new(&expanded),
                secret.as_deref(),
            )
            .map_err(|e| {
                invalid(format!(
                    "SSH key authentication failed for {} ({expanded}): {e}",
                    host.username
                ))
            })
        }
        AuthMethod::KeyId { id } => {
            // C-10: this used to write the decrypted private key to a temp file (0644, the
            // std::fs::write default) so libssh2 could read it, then best-effort-shred and
            // delete it. Any other local user could read the key while it existed, and a crash
            // mid-auth left it on disk. userauth_pubkey_memory() hands libssh2 the PEM directly
            // — the key never touches disk at all.
            let priv_pem = crate::keys::get_private_key(id)?;
            sess.userauth_pubkey_memory(&host.username, None, &priv_pem, None)
                .map_err(|e| {
                    invalid(format!(
                        "Vault Key ID authentication failed for {}: {e}",
                        host.username
                    ))
                })
        }
    }
}

fn evict_exec_session(host_id: &str) {
    if let Ok(mut pool) = EXEC_SESSIONS.lock() {
        pool.remove(host_id);
    }
}

/// `keepalive_send` is a cheap round trip that fails as soon as the transport is gone — the one
/// thing a cached handle cannot tell on its own.
fn session_is_alive(session: &Arc<Mutex<ssh2::Session>>) -> bool {
    match session.lock() {
        Ok(sess) => sess.authenticated() && sess.keepalive_send().is_ok(),
        Err(_) => false,
    }
}

/// Returns a pooled session only if it is still alive.
fn live_exec_session(host_id: &str) -> Option<Arc<Mutex<ssh2::Session>>> {
    let cached = EXEC_SESSIONS.lock().ok()?.get(host_id).map(Arc::clone)?;
    if session_is_alive(&cached) {
        Some(cached)
    } else {
        evict_exec_session(host_id);
        None
    }
}

fn open_exec_session(host_id: &str) -> Result<Arc<Mutex<ssh2::Session>>, CatermError> {
    let (sess, _host) = open_authenticated_session(host_id)?;
    sess.set_blocking(true);
    sess.set_timeout(20_000);
    sess.set_keepalive(true, 30);

    let arc = Arc::new(Mutex::new(sess));
    if let Ok(mut pool) = EXEC_SESSIONS.lock() {
        pool.insert(host_id.to_string(), Arc::clone(&arc));
    }
    Ok(arc)
}

/// Runs `f` against a blocking SSH session dedicated to non-interactive work on `host_id`.
///
/// The session is pooled and reused across calls, and is completely independent of any
/// terminal pane: SFTP and monitoring therefore work whether or not a PTY tab for that host
/// happens to be open, and can never stall the terminal. If a reused session turns out to be
/// dead, the operation is retried once on a freshly opened one, so a connection that died while
/// idle surfaces as a reconnect rather than an error. Errors raised by a still-live session are
/// the operation's own and are returned as-is.
pub fn with_exec_session<T, F>(host_id: &str, f: F) -> Result<T, CatermError>
where
    F: Fn(&ssh2::Session) -> Result<T, CatermError>,
{
    require_non_empty("host_id", host_id)?;

    if let Some(cached) = live_exec_session(host_id) {
        let outcome = match cached.lock() {
            Ok(sess) => f(&sess),
            Err(_) => Err(invalid("Exec session mutex poisoned".to_string())),
        };
        match outcome {
            Ok(value) => return Ok(value),
            Err(err) => {
                // Distinguish "the link died" from "the operation legitimately failed". Only the
                // former is worth a reconnect; retrying a genuine `No such file` would throw away
                // a perfectly good connection on every mistyped path and report it twice as slowly.
                if session_is_alive(&cached) {
                    return Err(err);
                }
                evict_exec_session(host_id);
            }
        }
    }

    let fresh = open_exec_session(host_id)?;
    let sess = fresh
        .lock()
        .map_err(|_| invalid("Exec session mutex poisoned".to_string()))?;
    f(&sess)
}

/// Splits off every complete UTF-8 character currently in `buf`, leaving a partial trailing
/// sequence behind for the next chunk. Without this a multi-byte character straddling a 4 KiB
/// read boundary would be delivered as two replacement characters.
fn drain_utf8(buf: &mut Vec<u8>) -> String {
    match std::str::from_utf8(buf) {
        Ok(text) => {
            let out = text.to_string();
            buf.clear();
            out
        }
        Err(e) => {
            let valid_len = e.valid_up_to();
            if e.error_len().is_none() {
                // Truncated trailing sequence — keep it and wait for the rest.
                let rest = buf.split_off(valid_len);
                let out = String::from_utf8_lossy(buf).into_owned();
                *buf = rest;
                out
            } else {
                // Genuinely invalid byte (binary output): degrade lossily rather than stall.
                let out = String::from_utf8_lossy(buf).into_owned();
                buf.clear();
                out
            }
        }
    }
}

/// Upper bound on how much decoded output is held before being pushed out mid-burst. Output is
/// otherwise flushed the moment the stream goes quiet, so interactive echo is never delayed;
/// this only bounds a continuous firehose (`cat` on a large file, `tail -f` on a busy log),
/// which would otherwise emit one IPC event per 4 KiB read and flood the webview.
/// Takes bytes off the wire: decoded into `pending` for push delivery, or appended to the
/// session's buffer when no sink is installed.
fn absorb_pty_bytes(
    chunk: &[u8],
    push: bool,
    carry: &mut Vec<u8>,
    pending: &mut String,
    buffer: &Arc<Mutex<Vec<u8>>>,
) {
    if push {
        carry.extend_from_slice(chunk);
        pending.push_str(&drain_utf8(carry));
    } else if let Ok(mut out) = buffer.lock() {
        out.extend_from_slice(chunk);
    }
}

/// No-op when there is nothing pending, which is also the case whenever no sink is installed.
fn flush_pty_output(session_id: &str, pending: &mut String, tail: &Arc<Mutex<String>>) {
    if pending.is_empty() {
        return;
    }
    record_output_tail(tail, pending);
    emit(SshEvent::Output {
        session_id: session_id.to_string(),
        data: std::mem::take(pending),
    });
}

/// Spawns a local shell (PowerShell on Windows, the user's `$SHELL` on Unix) as a local
/// session, attached to a real pseudo-terminal (`openpty` on Unix, ConPTY on Windows via
/// `portable-pty`). Before this, the shell's stdio were plain OS pipes with no controlling
/// terminal at all — visible as "cannot set terminal process group", "no job control in this
/// shell", "TERM environment variable not set", and a prompt that renders in the wrong place
/// because the shell has no real notion of the pane's width. A real PTY fixes all of that at
/// once, the same way the remote-SSH path already gets it from `ssh2`'s `request_pty`.
fn connect_local() -> Result<SshSession, CatermError> {
    let seq = LOCAL_SESSION_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let session_id = format!(
        "local-{}-{seq}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    let pty_system = portable_pty::native_pty_system();
    // 80x24 is only the starting size: the frontend calls `fit.fit()` + `sshResize` right after
    // connecting, same as it does for a remote SSH pane, and `resize()` below propagates that to
    // `pty_master`.
    let pty_pair = pty_system
        .openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| invalid(format!("Failed to allocate a local PTY: {e}")))?;

    #[cfg(windows)]
    let mut cmd = portable_pty::CommandBuilder::new("powershell.exe");
    // Unix has no single canonical shell the way Windows has powershell.exe: macOS has
    // defaulted to zsh since Catalina, many Linux desktops to bash, some users to fish. `$SHELL`
    // is how every terminal emulator picks this, so CAFramework matches that instead of forcing bash
    // on people who never asked for it.
    #[cfg(not(windows))]
    let mut cmd = portable_pty::CommandBuilder::new(
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
    );

    #[cfg(windows)]
    cmd.args(["-NoLogo"]);
    // `-i`: forces interactive mode (loads .bashrc/.zshrc, shows a prompt) regardless of how the
    // shell's own isatty() heuristics read the pty. Job control and readline's own line-width
    // handling now work unaided, since this is a real terminal device — this used to be the one
    // thing `-i` alone could not fake.
    #[cfg(not(windows))]
    cmd.arg("-i");
    // The actual "TERM environment variable not set" fix: a PTY only supplies a terminal
    // *device*, not this variable — the process spawning the shell still has to set it, exactly
    // as a real terminal emulator would.
    cmd.env("TERM", "xterm-256color");

    let child = pty_pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| invalid(format!("Failed to spawn local shell: {e}")))?;
    // The child now owns the slave side of the pty; our handle to it has nothing left to do, and
    // ConPTY on Windows specifically expects it dropped once the child is spawned.
    drop(pty_pair.slave);

    let mut pty_writer = pty_pair
        .master
        .take_writer()
        .map_err(|e| invalid(format!("Failed to open local PTY writer: {e}")))?;
    let mut pty_reader = pty_pair
        .master
        .try_clone_reader()
        .map_err(|e| invalid(format!("Failed to open local PTY reader: {e}")))?;

    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let output_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let last_output_tail = Arc::new(Mutex::new(String::new()));

    // Writer thread: pipe user keystrokes into the PTY master — the shell reads them off its
    // slave side exactly as if they had been typed at a real terminal.
    thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            if pty_writer.write_all(&data).is_err() {
                break;
            }
            let _ = pty_writer.flush();
        }
    });

    // Reader thread: PTY output -> xterm events. A real PTY interleaves stdout and stderr the
    // same way an actual terminal does (there is no separate stderr stream once a program is
    // attached to one), so this single thread replaces the old stdout *and* stderr readers.
    let buffer_read = Arc::clone(&output_buffer);
    let tail_read = Arc::clone(&last_output_tail);
    let reader_session_id = session_id.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let push = has_event_sink();
        // Carries a UTF-8 sequence truncated at the 4 KiB read boundary over to the next read,
        // instead of decoding each chunk in isolation — otherwise a multi-byte character (an
        // emoji in `git log`, a non-ASCII filename in `ls`) split across two reads would surface
        // as a pair of replacement characters. Mirrors the remote-SSH reader path below.
        let mut carry = Vec::<u8>::new();
        loop {
            match pty_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Some(chunk) = buf.get(..n) {
                        if push {
                            carry.extend_from_slice(chunk);
                            let text = drain_utf8(&mut carry);
                            if !text.is_empty() {
                                record_output_tail(&tail_read, &text);
                                emit(SshEvent::Output {
                                    session_id: reader_session_id.clone(),
                                    data: text,
                                });
                            }
                        } else if let Ok(mut ob) = buffer_read.lock() {
                            ob.extend_from_slice(chunk);
                        }
                    }
                }
                // The shell exiting can surface as a read error rather than a clean Ok(0)
                // depending on platform — either way, the session is over.
                Err(_) => break,
            }
        }
        emit(SshEvent::Closed {
            session_id: reader_session_id,
        });
    });

    let handle = SessionHandle {
        tx,
        output_buffer,
        input_buffer: Arc::new(Mutex::new(String::new())),
        last_output_tail,
        channel: None,
        child: Some(Arc::new(Mutex::new(child))),
        pty_master: Some(Arc::new(Mutex::new(pty_pair.master))),
        host_id: "local".to_string(),
    };

    let mut sessions = SESSIONS
        .lock()
        .map_err(|_| invalid("Lock failure".to_string()))?;
    sessions.insert(session_id.clone(), handle);

    Ok(SshSession {
        session_id,
        host_id: "local".to_string(),
    })
}

/// Opens a real SSH connection & PTY channel via `ssh2` for a saved host. Looks the host
/// (and its decrypted credential, if any) up via `crate::store::load_host_for_connect` —
/// the frontend only ever supplies a `host_id`.
pub fn connect(host_id: &str) -> Result<SshSession, CatermError> {
    if host_id == "local" || host_id == "__local__" {
        return connect_local();
    }
    let (sess, host) = open_authenticated_session(host_id)?;

    let session_id = generate_session_id(&host.id);

    let mut channel = sess
        .channel_session()
        .map_err(|e| invalid(format!("Channel creation failed: {e}")))?;

    channel
        .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| invalid(format!("PTY request failed: {e}")))?;

    channel
        .shell()
        .map_err(|e| invalid(format!("Shell request failed: {e}")))?;

    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let output_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let last_output_tail = Arc::new(Mutex::new(String::new()));

    // Set session to non-blocking so our reader loop doesn't hold the lock forever. The
    // session is moved into the reader thread afterwards and is never shared with SFTP,
    // monitoring or AI execution — those use `with_exec_session` instead.
    sess.set_blocking(false);

    let channel_arc = Arc::new(Mutex::new(channel));

    // Spawn reader background thread
    let channel_read = Arc::clone(&channel_arc);
    let buffer_read = Arc::clone(&output_buffer);
    let tail_read = Arc::clone(&last_output_tail);
    let reader_session_id = session_id.clone();
    thread::spawn(move || {
        // Owns the session for the lifetime of the PTY: dropping it here is what tears the
        // transport down once the channel is closed. (`ssh2::Channel` holds its own Arc on the
        // session internals, so the writer thread stays valid regardless of this handle.)
        let _owned_session = sess;
        let push = has_event_sink();
        let mut buf = [0u8; 4096];
        let mut err_buf = [0u8; 4096];
        let mut carry = Vec::<u8>::new();
        let mut pending = String::new();
        // Set the moment the current (still-unflushed) batch started accumulating; `None` means
        // `pending` is empty. Used to cap how long a continuous flood can be held before flushing.
        let mut batch_started: Option<Instant> = None;
        let mut idle_streak: u32 = 0;

        loop {
            let (read_res, is_eof) = {
                if let Ok(mut ch) = channel_read.lock() {
                    if let Ok(n_err) = ch.stderr().read(&mut err_buf)
                        && n_err > 0
                        && let Some(chunk) = err_buf.get(..n_err)
                    {
                        absorb_pty_bytes(chunk, push, &mut carry, &mut pending, &buffer_read);
                    }
                    let res = ch.read(&mut buf);
                    let eof = ch.eof();
                    (res, eof)
                } else {
                    break;
                }
            };

            match read_res {
                Ok(0) => {
                    // Stream went quiet: hand over whatever we have before idling, so a single
                    // keystroke echo is never held back waiting for more bytes.
                    flush_pty_output(&reader_session_id, &mut pending, &tail_read);
                    batch_started = None;
                    if is_eof {
                        break;
                    }
                    thread::sleep(next_poll_delay(idle_streak));
                    idle_streak = idle_streak.saturating_add(1);
                }
                Ok(n) => {
                    idle_streak = 0;
                    if let Some(chunk) = buf.get(..n) {
                        absorb_pty_bytes(chunk, push, &mut carry, &mut pending, &buffer_read);
                    }
                    // Coalesce a sustained flood (the remote keeps handing over more bytes with
                    // no gap) into batches instead of one IPC event per read(): only flush once
                    // this batch has been building for OUTPUT_BATCH_MAX_DELAY, or has grown past
                    // OUTPUT_BATCH_MAX_BYTES. A single keystroke's echo is unaffected — it still
                    // goes out immediately via the "stream went quiet" branch above, since the
                    // very next read is a WouldBlock/EOF, not another `Ok(n)`.
                    let started = batch_started.get_or_insert_with(Instant::now);
                    if should_flush_batch(pending.len(), started.elapsed()) {
                        flush_pty_output(&reader_session_id, &mut pending, &tail_read);
                        batch_started = None;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::Interrupted
                    {
                        flush_pty_output(&reader_session_id, &mut pending, &tail_read);
                        batch_started = None;
                        thread::sleep(next_poll_delay(idle_streak));
                        idle_streak = idle_streak.saturating_add(1);
                    } else {
                        break;
                    }
                }
            }
        }

        flush_pty_output(&reader_session_id, &mut pending, &tail_read);
        if push {
            emit(SshEvent::Closed {
                session_id: reader_session_id.clone(),
            });
        }
    });

    // Spawn writer background thread
    let channel_write = Arc::clone(&channel_arc);
    thread::spawn(move || {
        while let Ok(bytes) = rx.recv() {
            let mut written = 0;
            while written < bytes.len() {
                let (res, is_eof) = {
                    if let Ok(mut ch) = channel_write.lock() {
                        let pending = bytes.get(written..).unwrap_or(&[]);
                        let r = ch.write(pending);
                        let eof = ch.eof();
                        (r, eof)
                    } else {
                        return;
                    }
                };

                match res {
                    Ok(0) => {
                        if is_eof {
                            break;
                        }
                        thread::sleep(Duration::from_millis(1));
                    }
                    Ok(n) => {
                        written += n;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::Interrupted
                        {
                            thread::sleep(Duration::from_millis(1));
                        } else {
                            break;
                        }
                    }
                }
            }
            if let Ok(mut ch) = channel_write.lock() {
                let _ = ch.flush();
            }
        }
    });

    let handle = SessionHandle {
        tx,
        output_buffer,
        input_buffer: Arc::new(Mutex::new(String::new())),
        last_output_tail,
        channel: Some(channel_arc),
        child: None,
        pty_master: None,
        host_id: host.id.clone(),
    };

    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.insert(session_id.clone(), handle);
    }

    // Automatically detect remote OS in the background without blocking terminal startup
    let hid_detect = host.id.clone();
    thread::spawn(move || {
        let _ = detect_host_os(&hid_detect);
    });

    Ok(SshSession {
        session_id,
        host_id: host.id,
    })
}

/// Write data to active SSH channel.
pub fn write(session_id: &str, data: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;

    let (tx, input_buffer, last_output_tail, host_id) = {
        let sessions = SESSIONS
            .lock()
            .map_err(|_| invalid("Lock failure".to_string()))?;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| invalid(format!("Session {session_id} not found")))?;
        (
            handle.tx.clone(),
            Arc::clone(&handle.input_buffer),
            Arc::clone(&handle.last_output_tail),
            handle.host_id.clone(),
        )
    };

    if !data.is_empty() {
        if let Ok(mut buf) = input_buffer.lock() {
            if data == "\r" || data == "\n" {
                if !buf.is_empty() {
                    // C-09: don't audit-log what was typed at a silent prompt (sudo, su, ssh-add,
                    // passwd, ...) — the terminal never echoes it back, but this buffer captures
                    // every keystroke regardless, so without this check the password itself was
                    // stored (encrypted at rest, but plainly readable in Command Logs and its
                    // CSV export, and offered back as an autocomplete suggestion).
                    let tail = last_output_tail
                        .lock()
                        .map(|t| t.clone())
                        .unwrap_or_default();
                    if looks_like_password_prompt(&tail) {
                        let _ = crate::audit::log_event(
                            "PTY_COMMAND",
                            Some(&host_id),
                            "***redacted: typed at a password/passphrase prompt, not logged***",
                        );
                    } else {
                        let _ = crate::audit::log_event("PTY_COMMAND", Some(&host_id), &buf);
                    }
                    buf.clear();
                }
            } else if data == "\x7F" || data == "\x08" {
                // Backspace
                buf.pop();
            } else if !data.contains('\x1b') {
                // Ignore escape sequences
                buf.push_str(data);
            }
        }
        let _ = tx.send(data.as_bytes().to_vec());
    }

    Ok(String::new())
}

/// Read available output from an active SSH channel without writing.
///
/// Only meaningful when no event sink is installed (CLI, tests): with push delivery active
/// the reader thread never fills the buffer, so this returns an empty string rather than
/// handing the same bytes out twice.
pub fn read(session_id: &str) -> Result<String, CatermError> {
    require_non_empty("session_id", session_id)?;
    let output_buffer = {
        let sessions = SESSIONS
            .lock()
            .map_err(|_| invalid("Lock failure".to_string()))?;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| invalid(format!("Session {session_id} not found")))?;
        Arc::clone(&handle.output_buffer)
    };

    let mut out_bytes = Vec::new();
    if let Ok(mut buf) = output_buffer.lock()
        && !buf.is_empty()
    {
        match std::str::from_utf8(&buf) {
            Ok(_) => {
                out_bytes = std::mem::take(&mut *buf);
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                if e.error_len().is_none() {
                    out_bytes = buf.drain(..valid_len).collect();
                } else {
                    out_bytes = std::mem::take(&mut *buf);
                }
            }
        }
    }
    Ok(String::from_utf8_lossy(&out_bytes).to_string())
}

/// Resize active terminal PTY window.
pub fn resize(session_id: &str, cols: u16, rows: u16) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    let sessions = SESSIONS
        .lock()
        .map_err(|_| invalid("Lock failure".to_string()))?;
    if let Some(handle) = sessions.get(session_id) {
        if let Some(channel) = &handle.channel
            && let Ok(mut ch) = channel.lock()
        {
            let _ = ch.request_pty_size(cols as u32, rows as u32, None, None);
        }
        // Local sessions have no ssh2::Channel — resize their real PTY instead (see the
        // `pty_master` field doc comment). Without this the pane could be fit()'d to the
        // frontend's width but the shell itself would still think it was 80x24 forever.
        if let Some(master) = &handle.pty_master
            && let Ok(m) = master.lock()
        {
            let _ = m.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }
    Ok(())
}

/// Disconnect and remove active SSH session.
pub fn disconnect(session_id: &str) -> Result<(), CatermError> {
    require_non_empty("session_id", session_id)?;

    if let Ok(mut sessions) = SESSIONS.lock()
        && let Some(handle) = sessions.remove(session_id)
    {
        if let Some(channel) = &handle.channel
            && let Ok(mut ch) = channel.lock()
        {
            let _ = ch.close();
        }
        if let Some(child) = &handle.child
            && let Ok(mut ch) = child.lock()
        {
            let _ = ch.kill();
            emit(SshEvent::Closed {
                session_id: session_id.to_string(),
            });
        }
    }
    Ok(())
}

/// # Infallible: pure string parsing of os-release or uname output into a canonical OS identifier.
pub fn parse_os_key(raw: &str) -> String {
    let lower = raw.to_lowercase();

    // 1. Look for ID= line in os-release
    for line in lower.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("id=") {
            let id = rest.trim_matches('"').trim_matches('\'').trim();
            if !id.is_empty() {
                if id.contains("ubuntu") {
                    return "ubuntu".to_string();
                }
                if id.contains("debian") {
                    return "debian".to_string();
                }
                if id.contains("alpine") {
                    return "alpine".to_string();
                }
                if id.contains("arch") {
                    return "arch".to_string();
                }
                if id.contains("fedora") {
                    return "fedora".to_string();
                }
                if id.contains("centos") {
                    return "centos".to_string();
                }
                if id.contains("rocky") {
                    return "rocky".to_string();
                }
                if id.contains("alma") {
                    return "almalinux".to_string();
                }
                if id.contains("rhel") || id.contains("redhat") {
                    return "redhat".to_string();
                }
                if id.contains("manjaro") {
                    return "manjaro".to_string();
                }
                if id.contains("opensuse") || id.contains("suse") {
                    return "opensuse".to_string();
                }
                if id.contains("mint") {
                    return "mint".to_string();
                }
                if id.contains("kali") {
                    return "kali".to_string();
                }
                if id.contains("pop") {
                    return "popos".to_string();
                }
                if id.contains("gentoo") {
                    return "gentoo".to_string();
                }
                if id.contains("nixos") {
                    return "nixos".to_string();
                }
                if id.contains("raspbian") || id.contains("raspberry") {
                    return "raspberry".to_string();
                }
                if id.contains("amzn") || id.contains("amazon") {
                    return "amazon".to_string();
                }
                if id.contains("oracle") {
                    return "oracle".to_string();
                }
                if id.contains("void") {
                    return "void".to_string();
                }
                if id.contains("endeavour") {
                    return "endeavour".to_string();
                }
                if id.contains("elementary") {
                    return "elementary".to_string();
                }
                if id.contains("zorin") {
                    return "zorin".to_string();
                }
                if id.contains("freebsd") {
                    return "freebsd".to_string();
                }
                if id.contains("openbsd") {
                    return "openbsd".to_string();
                }
                if id.contains("netbsd") {
                    return "netbsd".to_string();
                }
                return id.to_string();
            }
        }
    }

    // 2. Check ID_LIKE= line
    for line in lower.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("id_like=") {
            let like = rest.trim_matches('"').trim_matches('\'').trim();
            if like.contains("ubuntu") {
                return "ubuntu".to_string();
            }
            if like.contains("debian") {
                return "debian".to_string();
            }
            if like.contains("rhel") || like.contains("centos") || like.contains("fedora") {
                return "redhat".to_string();
            }
            if like.contains("arch") {
                return "arch".to_string();
            }
            if like.contains("suse") {
                return "opensuse".to_string();
            }
        }
    }

    // 3. Fallback checks across full output
    if lower.contains("ubuntu") {
        return "ubuntu".to_string();
    }
    if lower.contains("debian") {
        return "debian".to_string();
    }
    if lower.contains("alpine") {
        return "alpine".to_string();
    }
    if lower.contains("arch") {
        return "arch".to_string();
    }
    if lower.contains("fedora") {
        return "fedora".to_string();
    }
    if lower.contains("centos") {
        return "centos".to_string();
    }
    if lower.contains("rocky") {
        return "rocky".to_string();
    }
    if lower.contains("alma") {
        return "almalinux".to_string();
    }
    if lower.contains("red hat") || lower.contains("rhel") {
        return "redhat".to_string();
    }
    if lower.contains("manjaro") {
        return "manjaro".to_string();
    }
    if lower.contains("opensuse") || lower.contains("suse") {
        return "opensuse".to_string();
    }
    if lower.contains("mint") {
        return "mint".to_string();
    }
    if lower.contains("kali") {
        return "kali".to_string();
    }
    if lower.contains("pop!_os") || lower.contains("popos") {
        return "popos".to_string();
    }
    if lower.contains("gentoo") {
        return "gentoo".to_string();
    }
    if lower.contains("nixos") {
        return "nixos".to_string();
    }
    if lower.contains("raspbian") || lower.contains("raspberry") {
        return "raspberry".to_string();
    }
    if lower.contains("amazon") || lower.contains("amzn") {
        return "amazon".to_string();
    }
    if lower.contains("oracle") {
        return "oracle".to_string();
    }
    if lower.contains("void") {
        return "void".to_string();
    }
    if lower.contains("darwin") || lower.contains("macos") || lower.contains("apple") {
        return "macos".to_string();
    }
    if lower.contains("windows")
        || lower.contains("microsoft")
        || lower.contains("cygwin")
        || lower.contains("mingw")
    {
        return "windows".to_string();
    }
    if lower.contains("freebsd") {
        return "freebsd".to_string();
    }
    if lower.contains("openbsd") {
        return "openbsd".to_string();
    }
    if lower.contains("netbsd") {
        return "netbsd".to_string();
    }
    if lower.contains("linux") {
        return "linux".to_string();
    }

    "linux".to_string()
}

/// Detects the remote host's operating system via SSH non-interactive exec session,
/// saves it to the local encrypted SQLite store, and emits an `OsDetected` event.
pub fn detect_host_os(host_id: &str) -> Result<String, CatermError> {
    require_non_empty("host_id", host_id)?;

    if host_id == "local" || host_id == "__local__" {
        let os = if cfg!(windows) { "windows" } else { "linux" }.to_string();
        emit(SshEvent::OsDetected {
            host_id: host_id.to_string(),
            os: os.clone(),
        });
        return Ok(os);
    }

    const OS_DETECT_SCRIPT: &str = r#"
if [ -f /etc/os-release ]; then
    cat /etc/os-release
elif [ -f /usr/lib/os-release ]; then
    cat /usr/lib/os-release
elif type uname >/dev/null 2>&1; then
    uname -s
elif [ -n "$COMSPEC" ] || [ -n "$OS" ]; then
    echo "windows"
else
    echo "linux"
fi
"#;

    let raw_output = with_exec_session(host_id, |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| invalid(format!("Failed to open SSH channel for OS detection: {e}")))?;
        channel
            .exec(OS_DETECT_SCRIPT)
            .map_err(|e| invalid(format!("Failed to execute OS detection command: {e}")))?;
        let mut out = String::new();
        channel.read_to_string(&mut out).unwrap_or_default();
        channel.wait_close().unwrap_or_default();
        Ok(out)
    })?;

    let detected = parse_os_key(&raw_output);
    if !detected.is_empty() && detected != "unknown" {
        let _ = crate::store::update_host_os(host_id, &detected);
        emit(SshEvent::OsDetected {
            host_id: host_id.to_string(),
            os: detected.clone(),
        });
    }

    Ok(detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_host_id() {
        // Fails at the `require_non_empty` guard, before ever touching the real on-disk
        // store — unlike a made-up-but-non-empty id, which would hit `store::load_host_for_connect`
        // and thus the real per-OS data dir (not something a unit test should touch; there is
        // no dependency-injection seam here yet to point it at a temp dir instead).
        assert!(connect("").is_err());
    }

    #[test]
    fn write_rejects_empty_session_id() {
        assert!(write("", "ls").is_err());
    }

    #[test]
    fn should_flush_batch_waits_for_size_or_age_threshold() {
        // Small batch, no age yet: hold it — this is the case that used to flush on every single
        // `read()` and produced 17,518 IPC events/sec on a sustained flood (C-12).
        assert!(!should_flush_batch(10, Duration::from_millis(0)));
        assert!(!should_flush_batch(
            OUTPUT_BATCH_MAX_BYTES - 1,
            OUTPUT_BATCH_MAX_DELAY - Duration::from_millis(1)
        ));

        // Either threshold on its own is enough to force a flush.
        assert!(should_flush_batch(
            OUTPUT_BATCH_MAX_BYTES,
            Duration::from_millis(0)
        ));
        assert!(should_flush_batch(1, OUTPUT_BATCH_MAX_DELAY));
    }

    #[test]
    fn next_poll_delay_backs_off_then_caps_and_resets_are_the_callers_job() {
        // C-16: the idle-read loop used to sleep a flat 1ms forever, waking an open-but-silent
        // session's reader thread 1000 times a second for as long as the pane stayed open. The
        // first idle read still polls almost immediately (interactive latency unaffected)...
        assert_eq!(next_poll_delay(0), POLL_MIN_DELAY);
        // ...but consecutive idle reads back off geometrically...
        assert_eq!(next_poll_delay(1), Duration::from_millis(2));
        assert_eq!(next_poll_delay(2), Duration::from_millis(4));
        assert_eq!(next_poll_delay(3), Duration::from_millis(8));
        // ...capping at POLL_MAX_DELAY rather than growing unboundedly.
        assert_eq!(next_poll_delay(6), POLL_MAX_DELAY);
        assert_eq!(next_poll_delay(1000), POLL_MAX_DELAY);
    }

    #[test]
    fn looks_like_password_prompt_matches_common_forms_only() {
        for tail in [
            "Password: ",
            "password:",
            "[sudo] password for alice: ",
            "Enter passphrase for key '/home/alice/.ssh/id_ed25519': ",
            "some earlier output\nPassword: ",
        ] {
            assert!(looks_like_password_prompt(tail), "should match: {tail:?}");
        }
        for tail in ["$ ", "user@host:~$ ", "Permission denied", ""] {
            assert!(
                !looks_like_password_prompt(tail),
                "should not match: {tail:?}"
            );
        }
    }

    #[test]
    fn key_id_auth_never_writes_the_private_key_to_disk() {
        // Guard for C-10. authenticate() with AuthMethod::KeyId used to write the decrypted
        // private key to std::env::temp_dir() (mode 0644, the std::fs::write default), then
        // best-effort-shred and delete it once auth finished -- readable by any other local
        // user for as long as it existed, and left behind entirely if the process crashed in
        // between. userauth_pubkey_memory() (the fix) hands libssh2 the PEM directly and never
        // touches disk, so there is no window and nothing to clean up either way.
        //
        // Note on what this test can and can't prove: because the old code's own cleanup ran
        // on every non-crash path, a before/after directory snapshot can't distinguish old vs.
        // fixed code here -- both leave temp_dir unchanged when nothing crashes. What this test
        // does verify, and keep verifying: no file is left behind, including if a future change
        // reintroduces a file-based approach with a cleanup bug (an early return before the
        // remove_file call, for instance).
        let _data = crate::test_support::isolated_data_dir("key_id_auth_no_tmp_file");
        crate::vault::unlock_vault("12345678").expect("unlock vault");

        let key = crate::keys::generate_key(crate::keys::KeyInput {
            name: "test-key".into(),
            algorithm: "Ed25519".into(),
        })
        .expect("generate_key");

        let before: std::collections::HashSet<_> = std::fs::read_dir(std::env::temp_dir())
            .expect("read temp_dir")
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect();

        let host = HostRecord {
            id: "host-under-test".into(),
            label: "Test".into(),
            address: "127.0.0.1".into(),
            port: 22,
            username: "root".into(),
            auth_method: AuthMethod::KeyId { id: key.id.clone() },
            tags: vec![],
            os: None,
            protocol: crate::store::ConnectionProtocol::default(),
            created_at: 0,
            updated_at: 0,
            has_secret: false,
        };
        // No real transport, so this is expected to fail -- what matters is that it never
        // touches disk on the way there.
        let sess = ssh2::Session::new().expect("create ssh2 session");
        let _ = authenticate(&sess, &host, None);

        let after: std::collections::HashSet<_> = std::fs::read_dir(std::env::temp_dir())
            .expect("read temp_dir")
            .filter_map(|e| e.ok().map(|e| e.file_name()))
            .collect();
        let new_files: Vec<_> = after.difference(&before).collect();
        assert!(
            new_files.is_empty(),
            "authenticate() left new file(s) in temp_dir: {new_files:?}"
        );
    }

    #[test]
    fn write_redacts_a_line_typed_at_a_password_prompt_instead_of_logging_it() {
        // Regression for C-09. Reproduces the audit's finding: text typed at a silent prompt
        // (sudo's included) used to be stored verbatim in Command Logs because audit::log_event
        // only masks lines matching "key=value"-style secrets, not a bare password with no
        // label. Simulates a session whose last output was a sudo password prompt, then checks
        // what write() actually persisted.
        let _data = crate::test_support::isolated_data_dir("ssh_password_prompt_redaction");
        crate::vault::unlock_vault("12345678").expect("unlock vault");

        let (tx, _rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let session_id = "test-session-password-prompt".to_string();
        let handle = SessionHandle {
            tx,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
            input_buffer: Arc::new(Mutex::new(String::new())),
            last_output_tail: Arc::new(Mutex::new("[sudo] password for alice: ".to_string())),
            channel: None,
            child: None,
            pty_master: None,
            host_id: "host-under-test".to_string(),
        };
        SESSIONS
            .lock()
            .expect("lock sessions")
            .insert(session_id.clone(), handle);

        write(&session_id, "hunter2").expect("type password");
        write(&session_id, "\r").expect("submit password");

        let logs = crate::audit::get_logs(Some("host-under-test"), None, None)
            .expect("get_logs should succeed");
        assert_eq!(logs.len(), 1);
        assert!(
            !logs[0].details.contains("hunter2"),
            "the typed password must never reach Command Logs: {:?}",
            logs[0].details
        );
        assert!(logs[0].details.contains("not logged"));

        SESSIONS.lock().expect("lock sessions").remove(&session_id);
    }

    #[test]
    fn read_rejects_empty_session_id() {
        assert!(read("").is_err());
    }

    #[test]
    fn resize_rejects_empty_session_id() {
        assert!(resize("", 80, 24).is_err());
    }

    #[test]
    fn disconnect_rejects_empty_session_id() {
        assert!(disconnect("").is_err());
    }

    #[test]
    fn with_exec_session_rejects_empty_host_id() {
        let result = with_exec_session("", |_| Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn connect_resolved_accepts_hostnames_not_just_ip_literals() {
        // Regression for the C-02 bug: `"localhost:0".parse::<SocketAddr>()` fails outright
        // (it is not an IP literal), while `to_socket_addrs()` resolves it via the system
        // resolver. Port 0 is deliberate: nothing listens there, so this proves resolution
        // succeeded and only the connection itself was refused, not that parsing failed.
        let err = connect_resolved("localhost:0", Duration::from_millis(200))
            .expect_err("nothing listens on port 0");
        assert_ne!(
            err.kind(),
            std::io::ErrorKind::InvalidInput,
            "hostname should resolve, not fail to parse: {err}"
        );
    }

    fn dummy_host(address: &str, port: u16) -> HostRecord {
        HostRecord {
            id: "host-under-test".into(),
            label: "Test".into(),
            address: address.into(),
            port,
            username: "root".into(),
            auth_method: AuthMethod::Password,
            tags: vec![],
            os: None,
            protocol: crate::store::ConnectionProtocol::default(),
            created_at: 0,
            updated_at: 0,
            has_secret: false,
        }
    }

    #[test]
    fn verify_host_key_fails_closed_when_known_hosts_is_unreadable() {
        // C-21: a `known_hosts` file that exists but can't be parsed (disk corruption, a stray
        // binary write, half a file left by a crash) used to be silently swallowed
        // (`let _ = known_hosts.read_file(...)`) and treated exactly like "no known_hosts
        // recorded yet". That doesn't just risk re-TOFUing a host that was already pinned; it
        // masks the read failure itself, which is often a sign of a real problem the user needs
        // to know about. Verifying still needs `sess.host_key()` to return something, which
        // needs a completed handshake — but the read happens *before* that check, so a session
        // that never handshook is enough to reach and exercise this branch without a live server.
        let data = crate::test_support::isolated_data_dir("verify_host_key_unreadable");
        let kh_path = data.path.join("known_hosts");
        // Not valid OpenSSH known_hosts syntax in any encoding: libssh2 opens this fine and then
        // fails to parse it, which is exactly the "exists but corrupted" case this guards.
        std::fs::write(
            &kh_path,
            [
                0xff, 0xfe, 0x00, 0x01, b'g', b'a', b'r', b'b', b'a', b'g', b'e',
            ],
        )
        .expect("write corrupted known_hosts");

        let sess = ssh2::Session::new().expect("create ssh2 session");
        let host = dummy_host("198.51.100.10", 22);
        let err =
            verify_host_key(&sess, &host, 22).expect_err("unreadable known_hosts must not pass");
        let msg = err.to_string();

        assert!(
            msg.contains("known_hosts"),
            "error should name the known_hosts file as the problem: {msg}"
        );
    }

    #[test]
    fn connect_resolved_accepts_bracketed_ipv6_literal() {
        // "::1:0" (unbracketed) is ambiguous; the caller in open_authenticated_session brackets
        // bare IPv6 addresses before calling this, so verify the bracketed form resolves.
        let err = connect_resolved("[::1]:0", Duration::from_millis(200))
            .expect_err("nothing listens on port 0");
        assert_ne!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn known_hosts_name_only_brackets_non_default_ports() {
        // libssh2's `checkp` derives this form itself; `add` does not, so the two only agree
        // when the name is normalised here.
        assert_eq!(known_hosts_name("10.0.0.1", 22), "10.0.0.1");
        assert_eq!(known_hosts_name("10.0.0.1", 2222), "[10.0.0.1]:2222");
    }

    #[test]
    fn drain_utf8_holds_back_a_split_multibyte_char() {
        // "é" is 0xC3 0xA9 — arriving one byte at a time across two reads.
        let mut buf = vec![b'a', 0xC3];
        assert_eq!(drain_utf8(&mut buf), "a");
        assert_eq!(buf, vec![0xC3]);

        buf.push(0xA9);
        assert_eq!(drain_utf8(&mut buf), "é");
        assert!(buf.is_empty());
    }

    #[test]
    fn drain_utf8_does_not_stall_on_invalid_bytes() {
        let mut buf = vec![b'o', b'k', 0xFF];
        let out = drain_utf8(&mut buf);
        assert!(out.starts_with("ok"));
        assert!(buf.is_empty());
    }

    #[test]
    fn parse_os_key_debian() {
        let os_release = r#"
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
VERSION="12 (bookworm)"
ID=debian
HOME_URL="https://www.debian.org/"
"#;
        assert_eq!(parse_os_key(os_release), "debian");
    }

    #[test]
    fn parse_os_key_ubuntu() {
        let os_release = r#"
NAME="Ubuntu"
VERSION="24.04 LTS (Noble Numbat)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 24.04 LTS"
"#;
        assert_eq!(parse_os_key(os_release), "ubuntu");
    }

    #[test]
    fn parse_os_key_alpine_and_arch() {
        assert_eq!(parse_os_key("ID=alpine\nNAME=\"Alpine Linux\""), "alpine");
        assert_eq!(parse_os_key("ID=arch\nNAME=\"Arch Linux\""), "arch");
        assert_eq!(parse_os_key("Linux"), "linux");
        assert_eq!(parse_os_key("Darwin"), "macos");
    }

    #[test]
    fn local_terminal_session_lifecycle() {
        let session = connect("local").expect("failed to connect local terminal");
        assert_eq!(session.host_id, "local");
        assert!(session.session_id.starts_with("local-"));

        let _ = write(&session.session_id, "echo hello\r\n");
        std::thread::sleep(Duration::from_millis(300));
        let _ = read(&session.session_id);

        disconnect(&session.session_id).expect("failed to disconnect local terminal");
        assert!(write(&session.session_id, "test").is_err());
    }

    // $TERM / job-control messages are bash/zsh-specific, so this is Unix-only; the Windows
    // path (ConPTY + powershell.exe) is exercised by the cross-platform test above instead.
    #[cfg(not(windows))]
    #[test]
    fn local_terminal_gets_a_real_pty_with_term_set() {
        // Regression test: the local shell used to run on plain `Stdio::piped()` with no
        // controlling terminal at all, which surfaced to the user as "cannot set terminal
        // process group", "no job control in this shell", and any program that queried the
        // terminal (even `clear`) failing with "TERM environment variable not set". A real PTY
        // (portable-pty's `openpty`) fixes all three at once, the same way the SSH-to-remote
        // path already gets them from ssh2's `request_pty`.
        let session = connect("local").expect("failed to connect local terminal");

        let _ = write(&session.session_id, "echo TERM_IS:$TERM\r\n");
        std::thread::sleep(Duration::from_millis(300));
        let out = read(&session.session_id).expect("read local terminal output");

        assert!(
            out.contains("TERM_IS:xterm-256color"),
            "shell should see TERM=xterm-256color from a real PTY, got: {out:?}"
        );
        assert!(
            !out.contains("cannot set terminal process group") && !out.contains("no job control"),
            "a real PTY should give the shell job control, not the old pipe-based errors: {out:?}"
        );

        disconnect(&session.session_id).expect("failed to disconnect local terminal");
    }

    #[test]
    #[cfg(not(windows))]
    fn local_terminal_does_not_mangle_utf8_split_across_a_read_boundary() {
        // Regression test for the other half of C-25: the local-PTY reader thread used to decode
        // each 4 KiB `read()` chunk independently with `String::from_utf8_lossy`, so a multi-byte
        // character landing right on that boundary came out as one or two U+FFFD replacement
        // characters instead of itself. A short command (well under the terminal's own
        // line-length limit) that emits a 3-byte UTF-8 character thousands of times in a tight
        // loop produces a bulk write the kernel hands back to us in ~4 KiB reads; since 3 does
        // not divide 4096, most of those reads land mid-character, reliably exercising the split.
        let session = connect("local").expect("failed to connect local terminal");

        let cmd =
            "for i in $(seq 1 4000); do printf '\\xE2\\x9C\\x93'; done; printf DONE_MARKER\r\n";
        let _ = write(&session.session_id, cmd);
        std::thread::sleep(Duration::from_millis(800));
        let out = read(&session.session_id).expect("read local terminal output");

        assert!(
            !out.contains('\u{FFFD}'),
            "a multi-byte char split across a PTY read boundary should be reassembled, not \
             replaced with U+FFFD: {} bytes, contains FFFD",
            out.len()
        );
        assert!(
            out.contains("DONE_MARKER"),
            "loop should have finished and printed the marker: {} bytes captured",
            out.len()
        );
        assert!(
            out.matches('\u{2713}').count() > 1000,
            "most of the 4000 checkmarks should survive intact in the output, got {}",
            out.matches('\u{2713}').count()
        );

        disconnect(&session.session_id).expect("failed to disconnect local terminal");
    }
}
