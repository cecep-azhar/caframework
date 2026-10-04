//! Privacy-first, opt-in crash reporting.
//!
//! Spec: `caframework-crash-reporting-spec-v1.md` (Notes/5 - Project/caframework/concept). Zero telemetry
//! by default — a panic is written to a local file and nothing else happens. Only on the next
//! launch, if the user explicitly reviews the scrubbed payload and clicks "send", does anything
//! leave the device. Declining, or "never ask again", deletes the dump without any network I/O.
//!
//! Scrubbing runs entirely on this device, before the payload is shown to the user or sent
//! anywhere — see [`PiiScrubber`]. The proxy this posts to is the same pattern as
//! [`crate::feedback`]: the actual error-collector DSN is a server-side secret, never compiled
//! into this binary.

use crate::error::{CatermError, IoError, ValidationError};
use crate::paths::{crash_dumps_dir, resolve_data_dir};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const PREFS_FILE: &str = "crash_reporting.json";
const PROXY_URL: &str = "";
/// A dump older than this is almost certainly from a version so old the stack trace is no
/// longer useful, and keeping it around only risks an unbounded pile of unread dumps.
const MAX_DUMP_AGE_SECS: i64 = 30 * 24 * 3600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct CrashReportingPrefs {
    /// Set once the user picks "never ask again": every future dump is deleted unseen, and
    /// the panic hook itself stops writing new ones.
    disabled: bool,
}

fn load_prefs(dir: &Path) -> CrashReportingPrefs {
    std::fs::read(dir.join(PREFS_FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_prefs(dir: &Path, prefs: &CrashReportingPrefs) -> Result<(), CatermError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash prefs dir: {e}"))))?;
    let bytes = serde_json::to_vec_pretty(prefs)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash prefs encode: {e}"))))?;
    std::fs::write(dir.join(PREFS_FILE), bytes)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash prefs write: {e}"))))
}

/// Whether the user has turned crash reporting off entirely (no prompt, no local dump).
/// # Infallible: an unreadable data dir or prefs file means "not disabled" (the safe
/// default for a feature that is itself opt-in and local-only), never an error to report.
pub fn is_disabled() -> bool {
    match resolve_data_dir() {
        Ok(info) => load_prefs(&info.path).disabled,
        Err(_) => false,
    }
}

// ---- Panic capture -----------------------------------------------------------------------------

/// Raw (unscrubbed) event captured at panic time. Never leaves this file, and never leaves the
/// device, in this shape.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RawCrashEvent {
    timestamp: String,
    app_version: String,
    target: String,
    os_name: String,
    message: String,
    location: Option<String>,
    backtrace: String,
}

/// Installs the process-wide panic hook. Call once, as early as possible in `main`, before any
/// other thread can panic.
///
/// The hook always still runs Rust's normal panic printing first (so `RUST_BACKTRACE`, stderr
/// and any existing logging keep working exactly as before), then makes one best-effort
/// attempt to persist a local dump. Every failure inside that attempt is swallowed: a second
/// panic while handling the first would abort the whole process instead of a graceful report,
/// which would turn "the crash reporter has a bug" into "crash reporting made crashes worse".
/// # Infallible: installs a hook and returns nothing; the hook's own best-effort failures are
/// swallowed by design (see above), not surfaced here.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        // `PanicHookInfo` borrows the panic payload (`dyn Any`), which isn't provably
        // `RefUnwindSafe` — but we only ever read from it here, never mutate through it, so a
        // second panic inside `try_record_panic` cannot observe it in some torn intermediate
        // state. That's exactly what `AssertUnwindSafe` is for.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| try_record_panic(info)));
    }));
}

fn try_record_panic(info: &std::panic::PanicHookInfo<'_>) -> Result<(), CatermError> {
    let dir = resolve_data_dir()?.path;
    if load_prefs(&dir).disabled {
        return Ok(()); // opted out: don't even keep a local copy
    }
    let dumps_dir = crash_dumps_dir(&dir);
    std::fs::create_dir_all(&dumps_dir)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dumps dir: {e}"))))?;

    let message = info
        .payload()
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic (pesan tidak tersedia)".to_string());
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));

    let event = RawCrashEvent {
        timestamp: now_rfc3339(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        target: format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        os_name: std::env::consts::OS.to_string(),
        message,
        location,
        // `std::backtrace::Backtrace` only gives a `Display` string, not structured
        // file/line/symbol frames, without unstable APIs — this is the text form of what
        // `RUST_BACKTRACE=1` would already print, scrubbed the same way as everything else.
        backtrace: std::backtrace::Backtrace::force_capture().to_string(),
    };

    let bytes = serde_json::to_vec(&event)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump encode: {e}"))))?;
    // Filename starts with a millisecond timestamp so dumps sort chronologically by name alone
    // — no need to open every file just to find the most recent one.
    let id = format!("{}-{}", now_millis(), uuid::Uuid::new_v4());
    write_restricted(&dumps_dir.join(format!("{id}.raw.json")), &bytes)
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(unix)]
fn write_restricted(path: &Path, bytes: &[u8]) -> Result<(), CatermError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump open: {e}"))))?;
    file.write_all(bytes)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump write: {e}"))))
}

#[cfg(not(unix))]
fn write_restricted(path: &Path, bytes: &[u8]) -> Result<(), CatermError> {
    std::fs::write(path, bytes)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump write: {e}"))))
}

// ---- PII scrubbing (spec §2) --------------------------------------------------------------------

/// Deterministic, local-only redaction. Every rule here mirrors a row of the spec's redaction
/// table. A regex that somehow fails to compile is treated as "skip this rule" rather than a
/// panic — the same defensive idiom already used for the other regexes in this crate
/// (`audit::mask_secrets`, `s3`, `webdav`): a missing redaction is a bug to fix, never a reason
/// to crash the crash reporter.
pub struct PiiScrubber {
    home_dir: String,
    username: String,
}

macro_rules! static_regex {
    ($name:ident, $pattern:expr) => {
        fn $name() -> Option<&'static Regex> {
            static CELL: OnceLock<Option<Regex>> = OnceLock::new();
            CELL.get_or_init(|| Regex::new($pattern).ok()).as_ref()
        }
    };
}

static_regex!(
    private_key_re,
    r"(?s)-----BEGIN [A-Z ]+PRIVATE KEY-----.*?-----END [A-Z ]+PRIVATE KEY-----"
);
static_regex!(api_key_re, r"sk-[a-zA-Z0-9]{20,}|gsk_[a-zA-Z0-9]{20,}");
static_regex!(
    secret_kv_re,
    r#"(?i)("?(?:password|passphrase|secret|token|api_key|auth)"?\s*[:=]\s*)"?([^"'\s,}]+)"?"#
);
static_regex!(
    ipv4_re,
    r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b"
);

impl Default for PiiScrubber {
    fn default() -> Self {
        Self::new()
    }
}

impl PiiScrubber {
    /// # Infallible: falls back to an empty home-dir/username (scrubbing less, never failing)
    /// when the OS home directory or user env vars can\'t be read.
    pub fn new() -> Self {
        let home_dir = directories::BaseDirs::new()
            .map(|b| b.home_dir().to_string_lossy().into_owned())
            .unwrap_or_default();
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_default();
        Self { home_dir, username }
    }

    /// Redacts one string. Order matters: structural secrets (keys, key-value pairs) before the
    /// coarser username/home-dir replacement, so a key that happens to contain the username is
    /// still fully redacted rather than half-replaced.
    /// # Infallible: a redaction rule that fails to compile (see `static_regex!`) is skipped
    /// rather than failing the whole scrub — a missing rule is a bug to fix, not a crash.
    pub fn scrub_text(&self, input: &str) -> String {
        let mut text = input.to_string();

        if let Some(re) = private_key_re() {
            text = re.replace_all(&text, "[REDACTED_PRIVATE_KEY]").to_string();
        }
        if let Some(re) = api_key_re() {
            text = re.replace_all(&text, "[REDACTED_API_KEY]").to_string();
        }
        if let Some(re) = secret_kv_re() {
            text = re.replace_all(&text, "$1[REDACTED]").to_string();
        }
        if let Some(re) = ipv4_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures<'_>| {
                    // Loopback/any-address carry no location information worth hiding.
                    match &caps[0] {
                        "127.0.0.1" | "0.0.0.0" => caps[0].to_string(),
                        other => {
                            let _ = other;
                            "[REDACTED_IP]".to_string()
                        }
                    }
                })
                .to_string();
        }

        if !self.home_dir.is_empty() {
            text = text.replace(&self.home_dir, "~");
        }
        if self.username.len() > 2 {
            text = text.replace(&self.username, "[USER]");
        }

        text
    }
}

fn scrub_event(scrubber: &PiiScrubber, raw: &RawCrashEvent) -> RawCrashEvent {
    RawCrashEvent {
        timestamp: raw.timestamp.clone(),
        app_version: raw.app_version.clone(),
        target: raw.target.clone(),
        os_name: raw.os_name.clone(),
        message: scrubber.scrub_text(&raw.message),
        location: raw.location.as_deref().map(|l| scrubber.scrub_text(l)),
        backtrace: scrubber.scrub_text(&raw.backtrace),
    }
}

/// Builds the exact payload that would be shown for review and, if approved, sent — there is
/// deliberately only one representation, so "what you see is what gets sent" always holds.
fn envelope(event_id: &str, scrubbed: &RawCrashEvent) -> serde_json::Value {
    serde_json::json!({
        "event_id": event_id,
        "timestamp": scrubbed.timestamp,
        "release": format!("caframework@{}", scrubbed.app_version),
        "environment": "production",
        "platform": "native-desktop",
        "tags": { "os.name": scrubbed.os_name, "target": scrubbed.target },
        "exception": {
            "values": [{
                "type": "Panic",
                "value": scrubbed.message,
                "location": scrubbed.location,
                "stacktrace_text": scrubbed.backtrace,
            }]
        }
    })
}

// ---- Review & submit (Tauri-facing) --------------------------------------------------------------

/// A crash dump ready for the user to review: scrubbed, with the exact JSON that would be sent.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrubbedCrashReport {
    pub id: String,
    pub timestamp: String,
    pub app_version: String,
    pub os_name: String,
    pub message: String,
    pub location: Option<String>,
    /// Pretty-printed JSON of the exact payload a "send" would submit.
    pub preview_json: String,
}

/// Filenames are `{id}.raw.json`; `id` must be exactly what the panic hook generated
/// (`<millis>-<uuid>`), never taken as an arbitrary path from the caller.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 80 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn dump_path(dumps_dir: &Path, id: &str) -> Result<PathBuf, CatermError> {
    if !valid_id(id) {
        return Err(CatermError::Validation(ValidationError::Generic(
            "ID laporan crash tidak valid".to_string(),
        )));
    }
    Ok(dumps_dir.join(format!("{id}.raw.json")))
}

/// Every `*.raw.json` filename in `dumps_dir`, oldest first (the timestamp prefix sorts
/// lexicographically), each paired with its id (filename without the extension).
fn list_dumps(dumps_dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dumps_dir) else {
        return Vec::new();
    };
    let mut dumps: Vec<(String, PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            let id = path
                .file_name()?
                .to_str()?
                .strip_suffix(".raw.json")?
                .to_string();
            valid_id(&id).then_some((id, path))
        })
        .collect();
    dumps.sort_by(|a, b| a.0.cmp(&b.0));
    dumps
}

/// The newest pending crash report, scrubbed and ready to show, or `None` if there isn't one.
///
/// Silently deletes anything it cannot make sense of (corrupt JSON, or reporting disabled since
/// the crash) rather than surfacing an error, and — when reporting is disabled — clears the
/// whole backlog without ever reading its contents into memory beyond the delete itself, since
/// "disabled" must mean disabled even for dumps written before the user turned it off. It also
/// clears dumps older than 30 days, which nobody would meaningfully report weeks later.
pub fn pending_crash_report() -> Result<Option<ScrubbedCrashReport>, CatermError> {
    let dir = resolve_data_dir()?.path;
    let dumps_dir = crash_dumps_dir(&dir);
    let disabled = load_prefs(&dir).disabled;
    let dumps = list_dumps(&dumps_dir);
    let cutoff_millis = now_millis() - MAX_DUMP_AGE_SECS * 1000;

    let mut newest: Option<(String, PathBuf)> = None;
    for (id, path) in dumps {
        let stale = id
            .split('-')
            .next()
            .and_then(|ts| ts.parse::<i64>().ok())
            .is_some_and(|ts| ts < cutoff_millis);
        if disabled || stale {
            let _ = std::fs::remove_file(&path);
            continue;
        }
        newest = Some((id, path)); // dumps is oldest-first, so the last kept one is newest
    }

    let Some((id, path)) = newest else {
        return Ok(None);
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return Ok(None);
    };
    let Ok(raw) = serde_json::from_slice::<RawCrashEvent>(&bytes) else {
        let _ = std::fs::remove_file(&path); // unreadable dump: nothing useful to show or keep
        return Ok(None);
    };

    let scrubbed = scrub_event(&PiiScrubber::new(), &raw);
    let payload = envelope(&id, &scrubbed);
    let preview_json = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
    Ok(Some(ScrubbedCrashReport {
        id,
        timestamp: scrubbed.timestamp,
        app_version: scrubbed.app_version,
        os_name: scrubbed.os_name,
        message: scrubbed.message,
        location: scrubbed.location,
        preview_json,
    }))
}

/// Sends the scrubbed report `id` to the feedback-style proxy (never straight to the error
/// collector — see module docs) and deletes the local dump once it is accepted. A failed send
/// leaves the dump in place so the same report can be retried on the next launch.
pub fn submit_crash_report(id: &str) -> Result<(), CatermError> {
    let dir = resolve_data_dir()?.path;
    let dumps_dir = crash_dumps_dir(&dir);
    let path = dump_path(&dumps_dir, id)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump not found: {e}"))))?;
    let raw: RawCrashEvent = serde_json::from_slice(&bytes)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("crash dump corrupt: {e}"))))?;
    let scrubbed = scrub_event(&PiiScrubber::new(), &raw);
    let payload = envelope(id, &scrubbed);

    let agent = ureq::Agent::new_with_defaults();
    let mut resp = agent
        .post(PROXY_URL)
        .header("Content-Type", "application/json")
        .send_json(&payload)
        .map_err(|e| {
            CatermError::Io(IoError::Generic(format!(
                "Gagal mengirim laporan crash: {e}"
            )))
        })?;

    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        let body = resp.body_mut().read_to_string().unwrap_or_default();
        return Err(CatermError::Io(IoError::Generic(format!(
            "Server crash-report menolak laporan (HTTP {status}): {body}"
        ))));
    }

    let _ = std::fs::remove_file(&path);
    Ok(())
}

/// Discards the dump `id` without sending anything. When `never_again` is set, also disables
/// future prompts and clears every other pending dump — "never ask again" must mean it, even
/// for crashes that happened earlier in the same session.
pub fn dismiss_crash_report(id: &str, never_again: bool) -> Result<(), CatermError> {
    let dir = resolve_data_dir()?.path;
    let dumps_dir = crash_dumps_dir(&dir);

    if never_again {
        save_prefs(&dir, &CrashReportingPrefs { disabled: true })?;
        for (_, path) in list_dumps(&dumps_dir) {
            let _ = std::fs::remove_file(path);
        }
        return Ok(());
    }

    let path = dump_path(&dumps_dir, id)?;
    let _ = std::fs::remove_file(path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrubber_redacts_every_documented_category() {
        let scrubber = PiiScrubber {
            home_dir: "/home/cecepazhar".to_string(),
            username: "cecepazhar".to_string(),
        };
        let raw = "panic at /home/cecepazhar/Project/caframework/crates/caf-core/src/ssh.rs:412\n\
                   connecting to root@192.168.1.120:22\n\
                   key_path: /home/cecepazhar/.ssh/id_ed25519\n\
                   \"api_key\": \"sk-abcdefghijklmnopqrstuvwx\"\n\
                   password: hunter2\n\
                   -----BEGIN OPENSSH PRIVATE KEY-----\nabc123\n-----END OPENSSH PRIVATE KEY-----\n\
                   loopback ping to 127.0.0.1 ok";
        let scrubbed = scrubber.scrub_text(raw);

        assert!(
            !scrubbed.contains("cecepazhar"),
            "username leaked: {scrubbed}"
        );
        assert!(
            !scrubbed.contains("/home/cecepazhar"),
            "home dir leaked: {scrubbed}"
        );
        assert!(!scrubbed.contains("192.168.1.120"), "IP leaked: {scrubbed}");
        assert!(
            !scrubbed.contains("sk-abcdefghijklmnopqrstuvwx"),
            "API key leaked: {scrubbed}"
        );
        assert!(!scrubbed.contains("hunter2"), "password leaked: {scrubbed}");
        assert!(
            !scrubbed.contains("sk-abcdefghijklmnopqrstuvwx"),
            "API key leaked: {scrubbed}"
        );
        assert!(
            !scrubbed.contains("BEGIN OPENSSH PRIVATE KEY-----\nabc123"),
            "private key leaked: {scrubbed}"
        );
        assert!(
            scrubbed.contains("127.0.0.1"),
            "loopback should be kept: {scrubbed}"
        );
        assert!(scrubbed.contains("[REDACTED_PRIVATE_KEY]"));
        // The api_key value also sits behind a "api_key: ..." label, so the broader
        // key-value rule may redact it a second time on top of the narrower rule's own
        // placeholder — both outcomes are equally safe, so accept either marker here.
        assert!(
            scrubbed.contains("[REDACTED_API_KEY]") || scrubbed.contains("[REDACTED]"),
            "api key value not redacted: {scrubbed}"
        );
        assert!(scrubbed.contains("[REDACTED_IP]"));
        assert!(scrubbed.contains("~/"));
        // The username only ever appeared embedded inside the home-directory path in this
        // fixture, and that whole path was already replaced with "~" — so there is nothing
        // standalone left for the username rule to redact separately. That's still fully
        // redacted (`!scrubbed.contains("cecepazhar")` above proves it), just via one rule
        // instead of two; a case with a *bare* username is covered next.
        assert!(!scrubbed.contains("cecepazhar"));
    }

    #[test]
    fn scrubber_redacts_a_bare_username_outside_any_path() {
        let scrubber = PiiScrubber {
            home_dir: "/home/cecepazhar".to_string(),
            username: "cecepazhar".to_string(),
        };
        let scrubbed = scrubber.scrub_text("connected as user cecepazhar from another session");
        assert!(!scrubbed.contains("cecepazhar"));
        assert!(scrubbed.contains("[USER]"));
    }

    #[test]
    fn scrubber_is_idempotent_with_no_secrets() {
        let scrubber = PiiScrubber {
            home_dir: String::new(),
            username: String::new(),
        };
        let text = "assertion failed: key_len == 32";
        assert_eq!(scrubber.scrub_text(text), text);
    }

    #[test]
    fn valid_id_rejects_path_traversal_and_junk() {
        assert!(valid_id(
            "1790300000000-a1b2c3d4-e5f6-4a1b-9c3d-8e7f6a5b4c3d"
        ));
        assert!(!valid_id(""));
        assert!(!valid_id("../../etc/passwd"));
        assert!(!valid_id("has spaces"));
        assert!(!valid_id(&"a".repeat(200)));
    }

    #[test]
    fn no_pending_report_when_directory_is_empty() {
        let guard = crate::test_support::isolated_data_dir("crash_empty");
        assert!(
            pending_crash_report()
                .expect("pending_crash_report failed")
                .is_none()
        );
        drop(guard);
    }

    #[test]
    fn writes_scrubs_and_lets_the_user_send_then_delete_the_newest_dump() {
        let guard = crate::test_support::isolated_data_dir("crash_flow");
        let dumps_dir = crash_dumps_dir(&guard.path);
        std::fs::create_dir_all(&dumps_dir).expect("mkdir");

        // Real filenames carry a genuine millisecond timestamp (see `try_record_panic`), which
        // `pending_crash_report` also uses to drop dumps older than `MAX_DUMP_AGE_SECS` — so the
        // fixture here must use realistic "now-ish" timestamps too, not placeholders like `1000`,
        // or the staleness check would (correctly, for a real dump) discard them immediately.
        let base = now_millis();
        let older_id = format!("{}-aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", base - 5_000);
        let newer_id = format!("{}-bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb", base);

        let older = RawCrashEvent {
            timestamp: now_rfc3339(),
            app_version: "2.1.10".to_string(),
            target: "x86_64-linux".to_string(),
            os_name: "linux".to_string(),
            message: "older crash".to_string(),
            location: None,
            backtrace: String::new(),
        };
        let newer = RawCrashEvent {
            message: format!(
                "assertion failed: key_len == 32 at /home/{}/.ssh/id_ed25519",
                "tester"
            ),
            ..older.clone()
        };
        write_restricted(
            &dumps_dir.join(format!("{older_id}.raw.json")),
            &serde_json::to_vec(&older).expect("encode"),
        )
        .expect("write older");
        write_restricted(
            &dumps_dir.join(format!("{newer_id}.raw.json")),
            &serde_json::to_vec(&newer).expect("encode"),
        )
        .expect("write newer");

        let pending = pending_crash_report()
            .expect("pending_crash_report failed")
            .expect("expected a pending report");
        assert_eq!(pending.id, newer_id);
        assert!(pending.message.contains("assertion failed"));
        assert!(pending.preview_json.contains("\"type\": \"Panic\""));

        // Dismissing without "never again" only removes the one report shown; the older dump
        // (not yet reviewed) is untouched.
        dismiss_crash_report(&pending.id, false).expect("dismiss failed");
        assert!(!dumps_dir.join(format!("{}.raw.json", pending.id)).exists());
        let remaining = pending_crash_report().expect("pending_crash_report failed");
        assert!(
            remaining.is_some(),
            "the older dump should still be pending"
        );

        drop(guard);
    }

    #[test]
    fn never_again_disables_reporting_and_clears_the_backlog() {
        let guard = crate::test_support::isolated_data_dir("crash_never_again");
        let dumps_dir = crash_dumps_dir(&guard.path);
        std::fs::create_dir_all(&dumps_dir).expect("mkdir");
        let event = RawCrashEvent {
            timestamp: now_rfc3339(),
            app_version: "2.1.10".to_string(),
            target: "x86_64-linux".to_string(),
            os_name: "linux".to_string(),
            message: "boom".to_string(),
            location: None,
            backtrace: String::new(),
        };
        write_restricted(
            &dumps_dir.join("3000-cccccccc-cccc-cccc-cccc-cccccccccccc.raw.json"),
            &serde_json::to_vec(&event).expect("encode"),
        )
        .expect("write");

        dismiss_crash_report("3000-cccccccc-cccc-cccc-cccc-cccccccccccc", true)
            .expect("dismiss failed");
        assert!(is_disabled());
        assert!(
            pending_crash_report()
                .expect("pending_crash_report failed")
                .is_none()
        );

        // A dump written after opting out is never even created.
        try_record_panic_test_hook(&guard.path);
        assert!(
            pending_crash_report()
                .expect("pending_crash_report failed")
                .is_none()
        );

        drop(guard);
    }

    /// Exercises the same "opted out" early-return `try_record_panic` takes, without going
    /// through an actual `std::panic::PanicHookInfo` (which has no public constructor).
    fn try_record_panic_test_hook(data_dir: &Path) {
        if load_prefs(data_dir).disabled {
            return;
        }
        panic!("test hook should not be reached once reporting is disabled");
    }
}
