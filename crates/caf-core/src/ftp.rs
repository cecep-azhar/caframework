//! FTP and FTPS backend (`RemoteFileSystem`).
//!
//! Provides file transfer and directory operations over standard FTP
//! and explicit TLS (FTPS) using `suppaftp`.

use crate::error::{CatermError, FtpError};
use crate::sftp::{SftpFileEntry, SftpProgressPayload};
use crate::vfs::{ProgressCallback, RemoteFileSystem};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use suppaftp::native_tls::TlsConnector;
use suppaftp::{NativeTlsConnector, NativeTlsFtpStream};

fn ftp_err(msg: impl Into<String>) -> CatermError {
    CatermError::Ftp(FtpError::Generic(msg.into()))
}

/// FTP / FTPS backend implementing `RemoteFileSystem`.
pub struct FtpFileSystem {
    pub host_id: String,
    pub secure: bool,
}

impl FtpFileSystem {
    /// # Infallible: creates an FtpFileSystem instance holding host_id and TLS mode.
    pub fn new(host_id: impl Into<String>, secure: bool) -> Self {
        Self {
            host_id: host_id.into(),
            secure,
        }
    }

    fn connect(&self) -> Result<NativeTlsFtpStream, CatermError> {
        let (host, secret) = crate::store::load_host_for_connect(&self.host_id)?;
        let port = if host.port == 0 { 21 } else { host.port };
        let addr = format!("{}:{}", host.address, port);

        let mut stream = NativeTlsFtpStream::connect(&addr)
            .map_err(|e| ftp_err(format!("FTP connection to {addr} failed: {e}")))?;

        if self.secure {
            let tls_connector = TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(|e| ftp_err(format!("TLS config error: {e}")))?;
            stream = stream
                .into_secure(NativeTlsConnector::from(tls_connector), &host.address)
                .map_err(|e| ftp_err(format!("FTPS handshake failed: {e}")))?;
        }

        let password = secret.unwrap_or_default();
        stream
            .login(&host.username, &password)
            .map_err(|e| ftp_err(format!("FTP login failed for {}: {e}", host.username)))?;

        Ok(stream)
    }

    fn with_stream<T, F>(&self, mut f: F) -> Result<T, CatermError>
    where
        F: FnMut(&mut NativeTlsFtpStream) -> Result<T, CatermError>,
    {
        let mut attempts = 0;
        loop {
            match self.connect() {
                Ok(mut stream) => match f(&mut stream) {
                    Ok(val) => {
                        let _ = stream.quit();
                        return Ok(val);
                    }
                    Err(_e) if attempts < 2 => {
                        let _ = stream.quit();
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(300));
                    }
                    Err(e) => {
                        let _ = stream.quit();
                        return Err(e);
                    }
                },
                Err(_) if attempts < 2 => {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl RemoteFileSystem for FtpFileSystem {
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError> {
        let path = if remote_path.trim().is_empty() || remote_path == "." {
            None
        } else {
            Some(remote_path)
        };

        self.with_stream(|stream| {
            let lines = stream
                .list(path)
                .map_err(|e| ftp_err(format!("FTP LIST failed: {e}")))?;

            let mut entries = Vec::new();
            let base_p = path.unwrap_or("/");

            for line in lines {
                if let Ok(file) = suppaftp::list::File::from_str(&line) {
                    let name = file.name().to_string();
                    if name == "." || name == ".." {
                        continue;
                    }
                    let full_path = crate::sftp::join_remote(base_p, &name)
                        .unwrap_or_else(|_| format!("{base_p}/{name}"));
                    let mtime = file
                        .modified()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    entries.push(SftpFileEntry {
                        name,
                        path: full_path,
                        is_dir: file.is_directory(),
                        is_symlink: file.is_symlink(),
                        size: file.size() as u64,
                        mtime,
                        mode: if file.is_directory() { 0o755 } else { 0o644 },
                    });
                }
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
        let path_str = remote_path.to_string();
        let name = Path::new(&path_str)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        self.with_stream(|stream| {
            let size = stream.size(&path_str).unwrap_or(0) as u64;
            Ok(SftpFileEntry {
                name: name.clone(),
                path: path_str.clone(),
                is_dir: false,
                is_symlink: false,
                size,
                mtime: 0,
                mode: 0o644,
            })
        })
    }

    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError> {
        let p = remote_path.to_string();
        self.with_stream(|stream| {
            let cursor = stream
                .retr_as_buffer(&p)
                .map_err(|e| ftp_err(format!("FTP RETR failed for {p}: {e}")))?;
            Ok(cursor.into_inner())
        })
    }

    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
        let p = remote_path.to_string();
        let mut cursor = Cursor::new(data.to_vec());
        self.with_stream(|stream| {
            stream
                .put_file(&p, &mut cursor)
                .map_err(|e| ftp_err(format!("FTP STOR failed for {p}: {e}")))?;
            Ok(())
        })
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError> {
        let p = remote_path.to_string();
        self.with_stream(|stream| {
            stream
                .mkdir(&p)
                .map_err(|e| ftp_err(format!("FTP MKD failed for {p}: {e}")))?;
            Ok(())
        })
    }

    fn delete(&self, remote_path: &str, is_dir: bool, _recursive: bool) -> Result<(), CatermError> {
        let p = remote_path.to_string();
        self.with_stream(|stream| {
            if is_dir {
                stream
                    .rmdir(&p)
                    .map_err(|e| ftp_err(format!("FTP RMD failed for {p}: {e}")))?;
            } else {
                stream
                    .rm(&p)
                    .map_err(|e| ftp_err(format!("FTP DELE failed for {p}: {e}")))?;
            }
            Ok(())
        })
    }

    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError> {
        let o = old_path.to_string();
        let n = new_path.to_string();
        self.with_stream(|stream| {
            stream
                .rename(&o, &n)
                .map_err(|e| ftp_err(format!("FTP RNFR/RNTO failed: {e}")))?;
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
        let mut local_file = File::open(local_path)
            .map_err(|e| ftp_err(format!("Failed to open local file {local_path}: {e}")))?;
        let total_bytes = local_file.metadata().map(|m| m.len()).unwrap_or(0);

        let t_id = transfer_id.to_string();
        let r_p = remote_path.to_string();

        let mut stream = self.connect()?;
        let mut data_stream = stream
            .put_with_stream(&r_p)
            .map_err(|e| ftp_err(format!("FTP STOR init failed: {e}")))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if crate::sftp::is_cancelled(&t_id) {
                crate::sftp::clear_cancel_token(&t_id);
                let _ = stream.finalize_put_stream(data_stream);
                let _ = stream.quit();
                return Err(ftp_err("Transfer cancelled by user"));
            }

            let n = local_file
                .read(&mut buffer)
                .map_err(|e| ftp_err(format!("Failed to read local file: {e}")))?;
            if n == 0 {
                break;
            }

            if let Some(chunk) = buffer.get(..n) {
                data_stream
                    .write_all(chunk)
                    .map_err(|e| ftp_err(format!("FTP write failed: {e}")))?;
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

        stream
            .finalize_put_stream(data_stream)
            .map_err(|e| ftp_err(format!("FTP finalize PUT failed: {e}")))?;
        let _ = stream.quit();
        crate::sftp::clear_cancel_token(&t_id);
        Ok(())
    }

    fn download(
        &self,
        remote_path: &str,
        local_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError> {
        crate::sftp::clear_cancel_token(transfer_id);
        let mut local_file = File::create(local_path)
            .map_err(|e| ftp_err(format!("Failed to create local file {local_path}: {e}")))?;

        let t_id = transfer_id.to_string();
        let r_p = remote_path.to_string();

        let mut stream = self.connect()?;
        let total_bytes = stream.size(&r_p).unwrap_or(0) as u64;

        let mut data_stream = stream
            .retr_as_stream(&r_p)
            .map_err(|e| ftp_err(format!("FTP RETR init failed: {e}")))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if crate::sftp::is_cancelled(&t_id) {
                crate::sftp::clear_cancel_token(&t_id);
                let _ = stream.finalize_retr_stream(data_stream);
                let _ = stream.quit();
                return Err(ftp_err("Transfer cancelled by user"));
            }

            let n = data_stream
                .read(&mut buffer)
                .map_err(|e| ftp_err(format!("FTP read failed: {e}")))?;
            if n == 0 {
                break;
            }

            if let Some(chunk) = buffer.get(..n) {
                local_file
                    .write_all(chunk)
                    .map_err(|e| ftp_err(format!("Failed to write local file: {e}")))?;
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

        stream
            .finalize_retr_stream(data_stream)
            .map_err(|e| ftp_err(format!("FTP finalize RETR failed: {e}")))?;
        let _ = stream.quit();
        crate::sftp::clear_cancel_token(&t_id);
        Ok(())
    }
}
