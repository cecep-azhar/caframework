//! SCP (Secure Copy Protocol) backend (`RemoteFileSystem`).
//!
//! Reuses the SSH exec session pool to perform file transfers via SCP
//! and directory management via SSH exec commands / SFTP fallback.

use crate::error::{CatermError, SftpError};
use crate::sftp::{SftpFileEntry, SftpProgressPayload};
use crate::vfs::{ProgressCallback, RemoteFileSystem};
use parking_lot::Mutex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

fn scp_err(msg: impl Into<String>) -> CatermError {
    CatermError::Sftp(SftpError::Generic(msg.into()))
}

/// SCP backend for remote file system operations.
pub struct ScpFileSystem {
    pub host_id: String,
}

impl ScpFileSystem {
    /// # Infallible: creates an ScpFileSystem instance holding host_id.
    pub fn new(host_id: impl Into<String>) -> Self {
        Self {
            host_id: host_id.into(),
        }
    }
}

/// Helper: shell-escape a string using POSIX single-quote rules.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

impl RemoteFileSystem for ScpFileSystem {
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError> {
        // First try SFTP listing via the shared session pool.
        if let Ok(entries) = crate::sftp::list_remote_dir(&self.host_id, remote_path) {
            return Ok(entries);
        }

        // Fallback: parse `ls -la` output via SSH exec channel.
        let path = if remote_path.trim().is_empty() {
            ".".to_string()
        } else {
            remote_path.to_string()
        };

        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .channel_session()
                .map_err(|e| scp_err(format!("Failed to open SSH channel: {e}")))?;

            let q_path = shell_quote(&path);
            channel
                .exec(&format!("ls -la {q_path}"))
                .map_err(|e| scp_err(format!("Failed to exec ls: {e}")))?;

            let mut output = String::new();
            let _ = channel.read_to_string(&mut output);
            channel.wait_close().unwrap_or_default();

            let mut entries = Vec::new();
            for line in output.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 9 {
                    continue;
                }
                let name = parts.get(8..).map(|p| p.join(" ")).unwrap_or_default();
                if name == "." || name == ".." {
                    continue;
                }
                let is_dir = parts.first().map(|p| p.starts_with('d')).unwrap_or(false);
                let is_symlink = parts.first().map(|p| p.starts_with('l')).unwrap_or(false);
                let size = parts
                    .get(4)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                let full_path = crate::sftp::join_remote(&path, &name)
                    .unwrap_or_else(|_| format!("{path}/{name}"));

                entries.push(SftpFileEntry {
                    name,
                    path: full_path,
                    is_dir,
                    is_symlink,
                    size,
                    mtime: 0,
                    mode: if is_dir { 0o755 } else { 0o644 },
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

    fn stat(&self, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
        // Try SFTP stat first.
        if let Ok(stat) = crate::sftp::stat_remote(&self.host_id, remote_path) {
            return Ok(stat);
        }

        // Fallback: scp_recv metadata or `stat -c` command.
        let path = remote_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let (mut ch, file_stat) = sess
                .scp_recv(Path::new(&path))
                .map_err(|e| scp_err(format!("Failed to stat remote file via SCP: {e}")))?;
            let _ = ch.close();

            let name = Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());

            Ok(SftpFileEntry {
                name,
                path: path.clone(),
                is_dir: file_stat.is_dir(),
                is_symlink: false,
                size: file_stat.size(),
                mtime: 0,
                mode: file_stat.mode() as u32,
            })
        })
    }

    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError> {
        let path = remote_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let (mut channel, file_stat) = sess
                .scp_recv(Path::new(&path))
                .map_err(|e| scp_err(format!("Failed to open remote file via SCP {path}: {e}")))?;

            let mut buf = Vec::with_capacity(file_stat.size() as usize);
            channel
                .read_to_end(&mut buf)
                .map_err(|e| scp_err(format!("Failed to read SCP channel: {e}")))?;
            channel.close().unwrap_or_default();
            Ok(buf)
        })
    }

    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
        let path = remote_path.to_string();
        let total_bytes = data.len() as u64;
        let data_owned = data.to_vec();

        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .scp_send(Path::new(&path), 0o644, total_bytes, None)
                .map_err(|e| scp_err(format!("Failed to initiate SCP upload for {path}: {e}")))?;

            channel
                .write_all(&data_owned)
                .map_err(|e| scp_err(format!("Failed to write SCP payload: {e}")))?;
            channel.send_eof().unwrap_or_default();
            channel.wait_eof().unwrap_or_default();
            channel.close().unwrap_or_default();
            channel.wait_close().unwrap_or_default();
            Ok(())
        })
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError> {
        // Try SFTP first.
        if crate::sftp::mkdir_remote_dir(&self.host_id, remote_path).is_ok() {
            return Ok(());
        }

        // Fallback: exec mkdir -p
        let path = remote_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .channel_session()
                .map_err(|e| scp_err(format!("Failed to open SSH channel: {e}")))?;
            let cmd = format!("mkdir -p {}", shell_quote(&path));
            channel
                .exec(&cmd)
                .map_err(|e| scp_err(format!("Failed to exec mkdir: {e}")))?;
            channel.wait_close().unwrap_or_default();
            let exit = channel.exit_status().unwrap_or(0);
            if exit != 0 {
                return Err(scp_err(format!("mkdir exited with status {exit}")));
            }
            Ok(())
        })
    }

    fn delete(&self, remote_path: &str, is_dir: bool, recursive: bool) -> Result<(), CatermError> {
        // Try SFTP first.
        if crate::sftp::delete_remote_file(&self.host_id, remote_path, is_dir, recursive).is_ok() {
            return Ok(());
        }

        // Fallback: exec rm -f / rm -rf
        let path = remote_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .channel_session()
                .map_err(|e| scp_err(format!("Failed to open SSH channel: {e}")))?;
            let cmd = if is_dir && recursive {
                format!("rm -rf {}", shell_quote(&path))
            } else if is_dir {
                format!("rmdir {}", shell_quote(&path))
            } else {
                format!("rm -f {}", shell_quote(&path))
            };
            channel
                .exec(&cmd)
                .map_err(|e| scp_err(format!("Failed to exec rm: {e}")))?;
            channel.wait_close().unwrap_or_default();
            let exit = channel.exit_status().unwrap_or(0);
            if exit != 0 {
                return Err(scp_err(format!("delete exited with status {exit}")));
            }
            Ok(())
        })
    }

    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError> {
        // Try SFTP first.
        if crate::sftp::rename_remote_file(&self.host_id, old_path, new_path).is_ok() {
            return Ok(());
        }

        // Fallback: exec mv
        let old_p = old_path.to_string();
        let new_p = new_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .channel_session()
                .map_err(|e| scp_err(format!("Failed to open SSH channel: {e}")))?;
            let cmd = format!("mv {} {}", shell_quote(&old_p), shell_quote(&new_p));
            channel
                .exec(&cmd)
                .map_err(|e| scp_err(format!("Failed to exec mv: {e}")))?;
            channel.wait_close().unwrap_or_default();
            let exit = channel.exit_status().unwrap_or(0);
            if exit != 0 {
                return Err(scp_err(format!("mv exited with status {exit}")));
            }
            Ok(())
        })
    }

    fn chmod(&self, remote_path: &str, mode: u32) -> Result<(), CatermError> {
        // Try SFTP first.
        if crate::sftp::chmod_remote_file(&self.host_id, remote_path, mode).is_ok() {
            return Ok(());
        }

        // Fallback: exec chmod
        let path = remote_path.to_string();
        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .channel_session()
                .map_err(|e| scp_err(format!("Failed to open SSH channel: {e}")))?;
            let cmd = format!("chmod {:o} {}", mode, shell_quote(&path));
            channel
                .exec(&cmd)
                .map_err(|e| scp_err(format!("Failed to exec chmod: {e}")))?;
            channel.wait_close().unwrap_or_default();
            let exit = channel.exit_status().unwrap_or(0);
            if exit != 0 {
                return Err(scp_err(format!("chmod exited with status {exit}")));
            }
            Ok(())
        })
    }

    fn upload(
        &self,
        local_path: &str,
        remote_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError> {
        crate::sftp::clear_cancel_token(transfer_id);
        let local_file = File::open(local_path)
            .map_err(|e| scp_err(format!("Failed to open local file {local_path}: {e}")))?;
        let total_bytes = local_file.metadata().map(|m| m.len()).unwrap_or(0);

        let local_file_arc = Arc::new(Mutex::new(local_file));
        let t_id = transfer_id.to_string();
        let r_path = remote_path.to_string();

        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let mut channel = sess
                .scp_send(Path::new(&r_path), 0o644, total_bytes, None)
                .map_err(|e| scp_err(format!("SCP send init failed for {r_path}: {e}")))?;

            let mut buffer = vec![0u8; 64 * 1024];
            let mut bytes_transferred = 0u64;
            let start_time = Instant::now();
            let mut last_emit = Instant::now();

            loop {
                if crate::sftp::is_cancelled(&t_id) {
                    crate::sftp::clear_cancel_token(&t_id);
                    return Err(scp_err("Transfer cancelled by user".to_string()));
                }

                let n = {
                    let mut guard = local_file_arc.lock();
                    guard
                        .read(&mut buffer)
                        .map_err(|e| scp_err(format!("Failed to read local file: {e}")))?
                };
                if n == 0 {
                    break;
                }

                if let Some(chunk) = buffer.get(..n) {
                    channel
                        .write_all(chunk)
                        .map_err(|e| scp_err(format!("Failed to write SCP chunk: {e}")))?;
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

            channel.send_eof().unwrap_or_default();
            channel.wait_eof().unwrap_or_default();
            channel.close().unwrap_or_default();
            channel.wait_close().unwrap_or_default();
            crate::sftp::clear_cancel_token(&t_id);
            Ok(())
        })
    }

    fn download(
        &self,
        remote_path: &str,
        local_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError> {
        crate::sftp::clear_cancel_token(transfer_id);
        let t_id = transfer_id.to_string();
        let r_path = remote_path.to_string();
        let l_path = local_path.to_string();

        let local_file = File::create(&l_path)
            .map_err(|e| scp_err(format!("Failed to create local file {l_path}: {e}")))?;
        let local_file_arc = Arc::new(Mutex::new(local_file));

        crate::ssh::with_exec_session(&self.host_id, move |sess| {
            let (mut channel, file_stat) = sess
                .scp_recv(Path::new(&r_path))
                .map_err(|e| scp_err(format!("SCP recv init failed for {r_path}: {e}")))?;
            let total_bytes = file_stat.size();

            let mut buffer = vec![0u8; 64 * 1024];
            let mut bytes_transferred = 0u64;
            let start_time = Instant::now();
            let mut last_emit = Instant::now();

            loop {
                if crate::sftp::is_cancelled(&t_id) {
                    crate::sftp::clear_cancel_token(&t_id);
                    return Err(scp_err("Transfer cancelled by user".to_string()));
                }

                let n = channel
                    .read(&mut buffer)
                    .map_err(|e| scp_err(format!("Failed to read SCP chunk: {e}")))?;
                if n == 0 {
                    break;
                }

                {
                    let mut guard = local_file_arc.lock();
                    if let Some(chunk) = buffer.get(..n) {
                        guard.write_all(chunk).map_err(|e| {
                            scp_err(format!("Failed to write local file chunk: {e}"))
                        })?;
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

            channel.close().unwrap_or_default();
            channel.wait_close().unwrap_or_default();
            crate::sftp::clear_cancel_token(&t_id);
            Ok(())
        })
    }
}
