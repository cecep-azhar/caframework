//! SFTP Remote File Manager (`T2-SFTP-01`).
//! Provides remote directory listing, reading, writing, chmod, recursive deletion,
//! and high-performance chunked file transfer with progress callbacks.
//!
//! Every operation runs on a pooled non-interactive session obtained from
//! [`crate::ssh::with_exec_session`], never on the `ssh2::Session` behind a terminal pane.

use crate::error::{CatermError, SftpError};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

static CANCEL_TOKENS: once_cell::sync::Lazy<Mutex<HashSet<String>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashSet::new()));

/// # Infallible: inserts transfer ID to atomic cancel set.
pub fn cancel_transfer(transfer_id: &str) -> Result<(), CatermError> {
    let mut set = CANCEL_TOKENS.lock();
    set.insert(transfer_id.to_string());
    Ok(())
}

pub(crate) fn is_cancelled(transfer_id: &str) -> bool {
    let set = CANCEL_TOKENS.lock();
    set.contains(transfer_id)
}

pub(crate) fn clear_cancel_token(transfer_id: &str) {
    let mut set = CANCEL_TOKENS.lock();
    set.remove(transfer_id);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpProgressPayload {
    pub transfer_id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
}

fn sftp_err(message: String) -> CatermError {
    CatermError::Sftp(SftpError::Generic(message))
}

fn open_sftp(sess: &ssh2::Session) -> Result<ssh2::Sftp, CatermError> {
    sess.sftp()
        .map_err(|e| sftp_err(format!("Failed to initialize SFTP subsystem: {e}")))
}

/// # Infallible: pure string manipulation joining paths with one separator.
pub fn join_remote(dir: &str, name: &str) -> Result<String, CatermError> {
    if dir == "/" || dir == "." || dir.is_empty() {
        Ok(format!("/{name}"))
    } else {
        Ok(format!("{}/{name}", dir.trim_end_matches('/')))
    }
}

pub fn list_remote_dir(
    host_id: &str,
    remote_path: &str,
) -> Result<Vec<SftpFileEntry>, CatermError> {
    let path = if remote_path.trim().is_empty() {
        ".".to_string()
    } else {
        remote_path.to_string()
    };

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut dir = sftp
            .opendir(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to open remote dir {path}: {e}")))?;

        let mut entries = Vec::new();
        while let Ok((p, stat)) = dir.readdir() {
            let file_name = match p.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            if file_name == "." || file_name == ".." {
                continue;
            }

            let mode = stat.perm.unwrap_or(0);
            let is_symlink = (mode & 0o120000) == 0o120000;

            let entry_p =
                join_remote(&path, &file_name).unwrap_or_else(|_| format!("{path}/{file_name}"));
            entries.push(SftpFileEntry {
                path: entry_p,
                name: file_name,
                is_dir: stat.is_dir(),
                is_symlink,
                size: stat.size.unwrap_or(0),
                mtime: stat.mtime.unwrap_or(0),
                mode,
            });
        }

        entries.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.cmp(&b.name)
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(entries)
    })
}

pub fn stat_remote(host_id: &str, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let stat = sftp
            .stat(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {path}: {e}")))?;

        let name = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        let mode = stat.perm.unwrap_or(0);
        let is_symlink = (mode & 0o120000) == 0o120000;

        Ok(SftpFileEntry {
            name,
            path: path.clone(),
            is_dir: stat.is_dir(),
            is_symlink,
            size: stat.size.unwrap_or(0),
            mtime: stat.mtime.unwrap_or(0),
            mode,
        })
    })
}

pub fn read_remote_file(host_id: &str, remote_path: &str) -> Result<Vec<u8>, CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut file = sftp
            .open(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to open remote file {path}: {e}")))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| sftp_err(format!("Failed to read remote file: {e}")))?;
        Ok(buf)
    })
}

pub fn write_remote_file(host_id: &str, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut file = sftp
            .create(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to create remote file {path}: {e}")))?;

        file.write_all(data)
            .map_err(|e| sftp_err(format!("Failed to write to remote file: {e}")))?;
        Ok(())
    })
}

pub fn mkdir_remote_dir(host_id: &str, remote_path: &str) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        sftp.mkdir(Path::new(&path), 0o755)
            .map_err(|e| sftp_err(format!("Failed to create remote directory {path}: {e}")))
    })
}

fn remove_remote_dir_recursive(sftp: &ssh2::Sftp, path: &Path) -> Result<(), CatermError> {
    let mut dir = match sftp.opendir(path) {
        Ok(d) => d,
        Err(_) => {
            let _ = sftp.unlink(path);
            let _ = sftp.rmdir(path);
            return Ok(());
        }
    };

    let mut children = Vec::new();
    while let Ok((child_path, stat)) = dir.readdir() {
        let name = match child_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name == "." || name == ".." {
            continue;
        }
        children.push((child_path, stat.is_dir()));
    }

    for (child, is_dir) in children {
        if is_dir {
            remove_remote_dir_recursive(sftp, &child)?;
        } else {
            let _ = sftp.unlink(&child);
        }
    }

    sftp.rmdir(path).map_err(|e| {
        sftp_err(format!(
            "Failed to remove remote dir {}: {e}",
            path.display()
        ))
    })?;
    Ok(())
}

pub fn delete_remote_file(
    host_id: &str,
    remote_path: &str,
    is_dir: bool,
    recursive: bool,
) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let p = Path::new(&path);
        if is_dir {
            if recursive {
                remove_remote_dir_recursive(&sftp, p)?;
            } else {
                sftp.rmdir(p)
                    .map_err(|e| sftp_err(format!("Failed to rmdir {path}: {e}")))?;
            }
        } else {
            sftp.unlink(p)
                .map_err(|e| sftp_err(format!("Failed to unlink {path}: {e}")))?;
        }
        Ok(())
    })
}

pub fn rename_remote_file(
    host_id: &str,
    old_path: &str,
    new_path: &str,
) -> Result<(), CatermError> {
    let old_p = old_path.to_string();
    let new_p = new_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        sftp.rename(Path::new(&old_p), Path::new(&new_p), None)
            .map_err(|e| {
                sftp_err(format!(
                    "Failed to rename remote file {old_p} -> {new_p}: {e}"
                ))
            })
    })
}

pub fn chmod_remote_file(host_id: &str, remote_path: &str, mode: u32) -> Result<(), CatermError> {
    let path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut stat = sftp
            .stat(Path::new(&path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {path}: {e}")))?;
        stat.perm = Some(mode);
        sftp.setstat(Path::new(&path), stat).map_err(|e| {
            sftp_err(format!(
                "Failed to chmod remote file {path} to {mode:o}: {e}"
            ))
        })
    })
}

pub fn copy_remote_file(host_id: &str, src_path: &str, dst_path: &str) -> Result<(), CatermError> {
    let data = read_remote_file(host_id, src_path)?;
    write_remote_file(host_id, dst_path, &data)
}

/// Builds the remote `cd ... && tar -czf ...` command for [`compress_remote`]. Pulled out as a
/// pure function so the `--` fix (see comment below) has a regression test that doesn't need a
/// live SSH session.
fn build_compress_cmd(parent_dir: &str, items: &[String], archive_name: &str) -> String {
    // ponytail: zip support, use archive_name suffix to pick tool; add when needed
    let items_shell: Vec<String> = items.iter().map(|s| shell_quote(s)).collect();
    let items_str = items_shell.join(" ");
    let q_dir = shell_quote(parent_dir);
    let q_archive = shell_quote(archive_name);
    // `--` marks the end of tar's own option parsing: without it, an item whose name starts
    // with `-` (a file literally named `--checkpoint=1000` or `--checkpoint-action=exec=sh
    // evil.sh`, which a remote directory listing can legitimately contain) is parsed by tar as
    // an *option*, not a filename — shell-quoting each item (via `shell_quote` above) only
    // controls how the shell tokenizes the command line, not how tar itself interprets an
    // argument that happens to start with a dash once it receives it. This is the standard
    // GNU-tar-and-friends mitigation for that class of argument injection.
    // Works on all POSIX targets; Windows SSH servers typically ship tar.exe (Win10+).
    format!("cd {q_dir} && tar -czf {q_archive} -- {items_str}")
}

/// Compress remote items via `tar -czf` (unix) or `tar.exe -czf` (windows).
/// `archive_name` must be a simple filename (e.g. `archive.tar.gz`); it is
/// created in `parent_dir`. Items are relative names inside `parent_dir`.
pub fn compress_remote(
    host_id: &str,
    parent_dir: &str,
    items: Vec<String>,
    archive_name: &str,
) -> Result<(), CatermError> {
    if items.is_empty() {
        return Err(sftp_err("No items specified for compression".into()));
    }
    let cmd = build_compress_cmd(parent_dir, &items, archive_name);

    crate::ssh::with_exec_session(host_id, move |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| sftp_err(format!("Failed to open SSH channel for compress: {e}")))?;
        channel
            .exec(&cmd)
            .map_err(|e| sftp_err(format!("Failed to exec compress command: {e}")))?;
        let mut stdout = String::new();
        let mut stderr = String::new();
        use std::io::Read;
        let _ = channel.read_to_string(&mut stdout);
        let _ = channel.stderr().read_to_string(&mut stderr);
        channel.wait_close().unwrap_or_default();
        let exit = channel.exit_status().unwrap_or(1);
        if exit != 0 {
            return Err(sftp_err(format!("tar exited with status {exit}: {stderr}")));
        }
        Ok(())
    })
}

/// Extract a remote archive into `dest_dir`.
/// Supports `.tar.gz`, `.tgz`, `.tar.bz2`, `.tar.xz`, `.tar`, `.zip`.
pub fn extract_remote(
    host_id: &str,
    archive_path: &str,
    dest_dir: &str,
) -> Result<(), CatermError> {
    let archive = archive_path.to_string();
    let dest = dest_dir.to_string();
    let q_archive = shell_quote(&archive);
    let q_dest = shell_quote(&dest);

    let lower = archive.to_lowercase();
    // ponytail: PowerShell Expand-Archive fallback for Windows-only SSH servers; add when needed
    let cmd = if lower.ends_with(".zip") {
        format!("mkdir -p {q_dest} && unzip -o {q_archive} -d {q_dest}")
    } else {
        // tar auto-detects compression from the flag set; -xf handles .gz/.bz2/.xz
        format!("mkdir -p {q_dest} && tar -xf {q_archive} -C {q_dest}")
    };

    crate::ssh::with_exec_session(host_id, move |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| sftp_err(format!("Failed to open SSH channel for extract: {e}")))?;
        channel
            .exec(&cmd)
            .map_err(|e| sftp_err(format!("Failed to exec extract command: {e}")))?;
        let mut stdout = String::new();
        let mut stderr = String::new();
        use std::io::Read;
        let _ = channel.read_to_string(&mut stdout);
        let _ = channel.stderr().read_to_string(&mut stderr);
        channel.wait_close().unwrap_or_default();
        let exit = channel.exit_status().unwrap_or(1);
        if exit != 0 {
            return Err(sftp_err(format!(
                "extract exited with status {exit}: {stderr}"
            )));
        }
        Ok(())
    })
}

/// POSIX single-quote escaping: wrap in `'`, replace every `'` inside with `'\''`.
pub(crate) fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// Uploads a local file to remote SFTP with 64KB chunking and progress callbacks.
pub fn upload_file_with_progress<F>(
    host_id: &str,
    local_path: &str,
    remote_path: &str,
    transfer_id: &str,
    on_progress: Arc<Mutex<F>>,
) -> Result<(), CatermError>
where
    F: FnMut(SftpProgressPayload) + Send + 'static + ?Sized,
{
    clear_cancel_token(transfer_id);
    let local_file = File::open(local_path)
        .map_err(|e| sftp_err(format!("Failed to open local file {local_path}: {e}")))?;
    let total_bytes = local_file.metadata().map(|m| m.len()).unwrap_or(0);

    let local_file_arc = Arc::new(Mutex::new(local_file));
    let t_id = transfer_id.to_string();
    let r_path = remote_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut remote_file = sftp
            .create(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to create remote file {r_path}: {e}")))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if is_cancelled(&t_id) {
                clear_cancel_token(&t_id);
                return Err(sftp_err("Transfer cancelled by user".to_string()));
            }

            let n = {
                let mut guard = local_file_arc.lock();
                guard
                    .read(&mut buffer)
                    .map_err(|e| sftp_err(format!("Failed to read local file: {e}")))?
            };
            if n == 0 {
                break;
            }

            if let Some(chunk) = buffer.get(..n) {
                remote_file
                    .write_all(chunk)
                    .map_err(|e| sftp_err(format!("Failed to write remote chunk: {e}")))?;
            }

            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed_secs > 0.0 {
                    (bytes_transferred as f64 / elapsed_secs) as u64
                } else {
                    0
                };

                {
                    let mut cb = on_progress.lock();
                    cb(SftpProgressPayload {
                        transfer_id: t_id.clone(),
                        bytes_transferred,
                        total_bytes,
                        speed_bps,
                    });
                }
                last_emit = Instant::now();
            }
        }

        clear_cancel_token(&t_id);
        Ok(())
    })
}

/// Downloads a remote SFTP file to local path with 64KB chunking and progress callbacks.
pub fn download_file_with_progress<F>(
    host_id: &str,
    remote_path: &str,
    local_path: &str,
    transfer_id: &str,
    on_progress: Arc<Mutex<F>>,
) -> Result<(), CatermError>
where
    F: FnMut(SftpProgressPayload) + Send + 'static + ?Sized,
{
    clear_cancel_token(transfer_id);
    let t_id = transfer_id.to_string();
    let r_path = remote_path.to_string();
    let l_path = local_path.to_string();

    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let stat = sftp
            .stat(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to stat remote file {r_path}: {e}")))?;
        let total_bytes = stat.size.unwrap_or(0);

        let mut remote_file = sftp
            .open(Path::new(&r_path))
            .map_err(|e| sftp_err(format!("Failed to open remote file {r_path}: {e}")))?;

        if let Some(parent) = Path::new(&l_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let local_file = File::create(&l_path)
            .map_err(|e| sftp_err(format!("Failed to create local destination {l_path}: {e}")))?;
        let local_file_arc = Arc::new(Mutex::new(local_file));

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if is_cancelled(&t_id) {
                clear_cancel_token(&t_id);
                return Err(sftp_err("Transfer cancelled by user".to_string()));
            }

            let n = remote_file
                .read(&mut buffer)
                .map_err(|e| sftp_err(format!("Failed to read remote chunk: {e}")))?;
            if n == 0 {
                break;
            }

            {
                let mut guard = local_file_arc.lock();
                if let Some(chunk) = buffer.get(..n) {
                    guard
                        .write_all(chunk)
                        .map_err(|e| sftp_err(format!("Failed to write to local file: {e}")))?;
                }
            }

            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed_secs > 0.0 {
                    (bytes_transferred as f64 / elapsed_secs) as u64
                } else {
                    0
                };

                {
                    let mut cb = on_progress.lock();
                    cb(SftpProgressPayload {
                        transfer_id: t_id.clone(),
                        bytes_transferred,
                        total_bytes,
                        speed_bps,
                    });
                }
                last_emit = Instant::now();
            }
        }

        clear_cancel_token(&t_id);
        Ok(())
    })
}

/// A file match returned by [`search_remote_files`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSearchItem {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub mtime: u64,
    pub is_dir: bool,
}

/// Shell-quote a single token for POSIX `sh -c`: wrap in single quotes, escaping embedded `'`.
/// Prevents path / pattern injection into the remote `find` invocation.
fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            // end quote, escaped quote, re-open quote
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Search remote files on the host connected by `host_id`.
///
/// * On Unix-like remotes: runs `find <base_path> -maxdepth 8 \( -type f -o -type d \) -name <pattern>`
///   via [`crate::ssh::with_exec_session`], with shell-safe single-quoting on both arguments.
/// * Fallback (Windows or exec failure): walks `base_path` recursively via SFTP `readdir`.
///
/// Results are capped at `max_results`. Optional `min_size`/`max_size` filters apply to files
/// (directories always pass the size gate).
pub fn search_remote_files(
    host_id: &str,
    base_path: &str,
    pattern: &str,
    max_results: usize,
    min_size: Option<u64>,
    max_size: Option<u64>,
) -> Result<Vec<RemoteSearchItem>, CatermError> {
    if host_id.is_empty() {
        return Err(sftp_err("host_id must not be empty".to_string()));
    }
    let base = if base_path.trim().is_empty() {
        "/"
    } else {
        base_path
    };

    // Detect OS; default to Unix if detection fails.
    let is_unix = crate::ssh::detect_host_os(host_id)
        .map(|os| !os.to_lowercase().contains("windows"))
        .unwrap_or(true);

    if is_unix {
        search_via_find(host_id, base, pattern, max_results, min_size, max_size).or_else(|_| {
            search_via_sftp_walk(host_id, base, pattern, max_results, min_size, max_size)
        })
    } else {
        search_via_sftp_walk(host_id, base, pattern, max_results, min_size, max_size)
    }
}

fn search_via_find(
    host_id: &str,
    base: &str,
    pattern: &str,
    max_results: usize,
    min_size: Option<u64>,
    max_size: Option<u64>,
) -> Result<Vec<RemoteSearchItem>, CatermError> {
    let qbase = shell_single_quote(base);
    let qpat = shell_single_quote(pattern);
    // ponytail: -maxdepth 8 guards runaway deep trees; make configurable when needed.
    let cmd = format!(
        "find {qbase} -maxdepth 8 \\( -type f -o -type d \\) -name {qpat} -printf '%p\\t%s\\t%T@\\t%y\\n' 2>/dev/null"
    );

    let output = crate::ssh::with_exec_session(host_id, move |sess| {
        let mut channel = sess
            .channel_session()
            .map_err(|e| sftp_err(format!("Failed to open exec channel: {e}")))?;
        channel
            .exec(&cmd)
            .map_err(|e| sftp_err(format!("Failed to exec find: {e}")))?;
        let mut buf = String::new();
        use std::io::Read;
        channel
            .read_to_string(&mut buf)
            .map_err(|e| sftp_err(format!("Failed to read find output: {e}")))?;
        let _ = channel.wait_close();
        Ok(buf)
    })?;

    let mut results = Vec::new();
    for line in output.lines() {
        if results.len() >= max_results {
            break;
        }
        let parts: Vec<&str> = line.splitn(4, '\t').collect();
        if parts.len() < 4 {
            continue;
        }
        let path = parts.first().copied().unwrap_or("").to_string();
        let size: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let mtime: u64 = parts
            .get(2)
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let is_dir = parts.get(3).map(|s| s.trim() == "d").unwrap_or(false);

        if !is_dir {
            if min_size.is_some_and(|min| size < min) {
                continue;
            }
            if max_size.is_some_and(|max| size > max) {
                continue;
            }
        }

        let name = path.split('/').next_back().unwrap_or(&path).to_string();
        results.push(RemoteSearchItem {
            path,
            name,
            size,
            mtime,
            is_dir,
        });
    }
    Ok(results)
}

fn search_via_sftp_walk(
    host_id: &str,
    base: &str,
    pattern: &str,
    max_results: usize,
    min_size: Option<u64>,
    max_size: Option<u64>,
) -> Result<Vec<RemoteSearchItem>, CatermError> {
    let base = base.to_string();
    let pattern = pattern.to_string();
    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut results = Vec::new();
        sftp_walk(
            &sftp,
            &base,
            &pattern,
            max_results,
            min_size,
            max_size,
            &mut results,
        )?;
        Ok(results)
    })
}

fn sftp_walk(
    sftp: &ssh2::Sftp,
    dir: &str,
    pattern: &str,
    max_results: usize,
    min_size: Option<u64>,
    max_size: Option<u64>,
    acc: &mut Vec<RemoteSearchItem>,
) -> Result<(), CatermError> {
    if acc.len() >= max_results {
        return Ok(());
    }
    let mut handle = match sftp.opendir(std::path::Path::new(dir)) {
        Ok(h) => h,
        Err(_) => return Ok(()), // inaccessible dir — skip silently
    };

    // ponytail: no depth cap here; add a depth counter when needed.
    while let Ok((entry_path, stat)) = handle.readdir() {
        if acc.len() >= max_results {
            break;
        }
        let name = match entry_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        if name == "." || name == ".." {
            continue;
        }
        let full_path = if dir == "/" {
            format!("/{name}")
        } else {
            format!("{}/{name}", dir.trim_end_matches('/'))
        };
        let is_dir = stat.is_dir();
        let size = stat.size.unwrap_or(0);
        let mtime = stat.mtime.unwrap_or(0);

        if glob_match(pattern, &name) {
            if !is_dir {
                let mut pass = true;
                if min_size.is_some_and(|min| size < min) {
                    pass = false;
                }
                if max_size.is_some_and(|max| size > max) {
                    pass = false;
                }
                if pass {
                    acc.push(RemoteSearchItem {
                        path: full_path.clone(),
                        name,
                        size,
                        mtime,
                        is_dir,
                    });
                }
            } else {
                acc.push(RemoteSearchItem {
                    path: full_path.clone(),
                    name,
                    size,
                    mtime,
                    is_dir,
                });
            }
        }

        if is_dir {
            sftp_walk(
                sftp,
                &full_path,
                pattern,
                max_results,
                min_size,
                max_size,
                acc,
            )?;
        }
    }
    Ok(())
}

/// Minimal glob: supports `*` (any chars) and `?` (single char). Case-insensitive on match.
/// ponytail: no `**` support; upgrade to the `glob` crate when needed.
fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = name.chars().collect();
    glob_match_inner(&p, &s)
}

fn glob_match_inner(p: &[char], s: &[char]) -> bool {
    let p_rest = p.get(1..).unwrap_or(&[]);
    let s_rest = s.get(1..).unwrap_or(&[]);
    match (p.first(), s.first()) {
        (None, None) => true,
        (Some(&'*'), _) => {
            // star can match zero or more chars
            glob_match_inner(p_rest, s) || (!s.is_empty() && glob_match_inner(p, s_rest))
        }
        (Some(&'?'), Some(_)) => glob_match_inner(p_rest, s_rest),
        (Some(pc), Some(sc)) => {
            pc.to_lowercase().eq(sc.to_lowercase()) && glob_match_inner(p_rest, s_rest)
        }
        _ => false,
    }
}

/// Compute SHA-256 or MD5 checksum of a remote file.
///
/// Tries `sha256sum`/`md5sum` command first (fast, no data transfer).
/// Falls back to streaming the file over SFTP and hashing locally if the command fails.
pub fn calculate_remote_checksum(
    host_id: &str,
    path: &str,
    algorithm: &str,
) -> Result<String, CatermError> {
    use md5::Md5;
    use sha2::{Digest, Sha256};

    if host_id.is_empty() {
        return Err(sftp_err("host_id must not be empty".to_string()));
    }
    let algo = algorithm.to_lowercase();
    if algo != "sha256" && algo != "md5" {
        return Err(sftp_err(format!(
            "unsupported algorithm '{algorithm}'; use sha256 or md5"
        )));
    }

    let remote_path = path.to_string();
    let algo_clone = algo.clone();

    // Try remote command first (zero-copy, fast).
    let cmd_result: Result<String, CatermError> =
        crate::ssh::with_exec_session(host_id, move |sess| {
            let escaped = remote_path.replace('\'', "'\\''");
            let cmd = if algo_clone == "sha256" {
                format!("sha256sum '{escaped}'")
            } else {
                format!("md5sum '{escaped}'")
            };

            let mut channel = sess
                .channel_session()
                .map_err(|e| sftp_err(format!("Failed to open channel for checksum: {e}")))?;
            channel
                .exec(&cmd)
                .map_err(|e| sftp_err(format!("Failed to exec checksum command: {e}")))?;
            let mut out = String::new();
            channel.read_to_string(&mut out).unwrap_or_default();
            channel.wait_close().unwrap_or_default();
            let exit_status = channel.exit_status().unwrap_or(1);
            if exit_status != 0 || out.trim().is_empty() {
                return Err(sftp_err("remote checksum command failed".to_string()));
            }
            // Output format: "<hash>  <filename>"
            let hash = out.split_whitespace().next().unwrap_or("").to_string();
            if hash.is_empty() {
                return Err(sftp_err("empty checksum output from remote".to_string()));
            }
            Ok(hash)
        });

    if let Ok(hash) = cmd_result {
        return Ok(hash);
    }

    // Fallback: stream file over SFTP and hash locally.
    let remote_path2 = path.to_string();
    crate::ssh::with_exec_session(host_id, move |sess| {
        let sftp = open_sftp(sess)?;
        let mut remote_file = sftp
            .open(Path::new(&remote_path2))
            .map_err(|e| sftp_err(format!("Failed to open remote file for checksum: {e}")))?;

        let mut buf = vec![0u8; 64 * 1024];
        if algo == "sha256" {
            let mut hasher = Sha256::new();
            loop {
                let n = remote_file
                    .read(&mut buf)
                    .map_err(|e| sftp_err(format!("Failed to read remote chunk: {e}")))?;
                if n == 0 {
                    break;
                }
                if let Some(chunk) = buf.get(..n) {
                    hasher.update(chunk);
                }
            }
            Ok(hex::encode(hasher.finalize()))
        } else {
            let mut hasher = Md5::new();
            loop {
                let n = remote_file
                    .read(&mut buf)
                    .map_err(|e| sftp_err(format!("Failed to read remote chunk: {e}")))?;
                if n == 0 {
                    break;
                }
                if let Some(chunk) = buf.get(..n) {
                    hasher.update(chunk);
                }
            }
            Ok(hex::encode(hasher.finalize()))
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumComparison {
    pub local_checksum: String,
    pub remote_checksum: String,
    pub matches: bool,
}

/// Compare checksum between a local file and a remote file.
pub fn compare_file_checksums(
    host_id: &str,
    remote_path: &str,
    local_path: &str,
    algorithm: &str,
) -> Result<ChecksumComparison, CatermError> {
    let local_hash = crate::local_fs::calculate_local_checksum(Path::new(local_path), algorithm)?;
    let remote_hash = calculate_remote_checksum(host_id, remote_path, algorithm)?;
    let matches = local_hash.eq_ignore_ascii_case(&remote_hash);
    Ok(ChecksumComparison {
        local_checksum: local_hash,
        remote_checksum: remote_hash,
        matches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operations_reject_empty_host_id() {
        assert!(list_remote_dir("", "/").is_err());
        assert!(read_remote_file("", "/etc/hostname").is_err());
        assert!(write_remote_file("", "/tmp/x", b"x").is_err());
        assert!(delete_remote_file("", "/tmp/x", false, false).is_err());
        assert!(chmod_remote_file("", "/tmp/x", 0o755).is_err());
        assert!(stat_remote("", "/tmp/x").is_err());
        assert!(compress_remote("", "/tmp", vec!["a".into()], "out.tar.gz").is_err());
        assert!(extract_remote("", "/tmp/out.tar.gz", "/tmp/dest").is_err());
    }

    #[test]
    fn compress_rejects_empty_items() {
        assert!(compress_remote("host1", "/tmp", vec![], "out.tar.gz").is_err());
    }

    #[test]
    fn shell_quote_escapes_properly() {
        assert_eq!(shell_quote("hello"), "'hello'");
        assert_eq!(shell_quote("hello world"), "'hello world'");
        assert_eq!(shell_quote("foo'bar"), r"'foo'\''bar'");
    }

    #[test]
    fn build_compress_cmd_does_not_let_an_item_name_run_as_a_tar_option() {
        // C-19: this is the classic GNU-tar argument-injection PoC. `--checkpoint=1` and
        // `--checkpoint-action=exec=<cmd>` are each individually valid *filenames* (a remote
        // directory can legitimately contain files with these names), so a caller could pass
        // them as two entries in `items`. Each is shell-quoted correctly (shell_quote handles
        // that), but without a `--` separator tar itself still parses each dash-prefixed
        // argument as an option once it receives them — quoting only affects shell tokenizing,
        // not tar's own argv parsing — so tar would run the "exec" action as a checkpoint
        // callback. This runs the actual built command through a real shell and `tar` binary
        // and proves the exec payload never runs.
        let tmp =
            std::env::temp_dir().join(format!("caframework_c19_tar_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).expect("create scratch dir");
        let marker = tmp.join("pwned");
        // tar's checkpoint only fires once it has actually processed a file, so the PoC needs a
        // real file to archive alongside the two option-shaped "filenames" below.
        std::fs::write(tmp.join("real_file.txt"), b"hello").expect("create a real file to archive");

        let items = vec![
            "real_file.txt".to_string(),
            "--checkpoint=1".to_string(),
            format!("--checkpoint-action=exec=touch {}", marker.display()),
        ];
        let cmd = build_compress_cmd(&tmp.to_string_lossy(), &items, "out.tar.gz");

        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .status();

        assert!(
            !marker.exists(),
            "tar must never treat an item name as a --checkpoint-action exec payload: {cmd}"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn join_remote_keeps_exactly_one_separator() {
        assert_eq!(join_remote("/", "etc").unwrap(), "/etc");
        assert_eq!(join_remote(".", "etc").unwrap(), "/etc");
        assert_eq!(
            join_remote("/var/log", "syslog").unwrap(),
            "/var/log/syslog"
        );
        assert_eq!(
            join_remote("/var/log/", "syslog").unwrap(),
            "/var/log/syslog"
        );
    }

    #[test]
    fn cancel_token_lifecycle() {
        let id = "test-transfer-cancel";
        assert!(!is_cancelled(id));
        cancel_transfer(id).unwrap();
        assert!(is_cancelled(id));
        clear_cancel_token(id);
        assert!(!is_cancelled(id));
    }

    #[test]
    fn shell_single_quote_plain() {
        assert_eq!(shell_single_quote("/home/user"), "'/home/user'");
    }

    #[test]
    fn shell_single_quote_with_embedded_single_quote() {
        // O'Brien  ->  'O'\''Brien'
        assert_eq!(shell_single_quote("O'Brien"), "'O'\\''Brien'");
    }

    #[test]
    fn glob_match_star() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "main.py"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn glob_match_question_mark() {
        assert!(glob_match("?.rs", "a.rs"));
        assert!(!glob_match("?.rs", "ab.rs"));
    }

    #[test]
    fn glob_match_case_insensitive() {
        assert!(glob_match("*.TXT", "readme.txt"));
    }

    #[test]
    fn search_remote_files_rejects_empty_host_id() {
        assert!(search_remote_files("", "/", "*", 10, None, None).is_err());
    }

    #[test]
    fn calculate_remote_checksum_rejects_empty_host_id() {
        assert!(calculate_remote_checksum("", "/tmp/foo", "sha256").is_err());
    }
}
