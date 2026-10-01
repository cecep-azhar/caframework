//! WebDAV backend (`RemoteFileSystem`).
//!
//! Provides file transfer and directory operations over WebDAV
//! using `ureq` HTTP client.

use crate::error::{CatermError, IoError};
use crate::sftp::{SftpFileEntry, SftpProgressPayload};
use crate::vfs::{ProgressCallback, RemoteFileSystem};
use base64::Engine;
use regex::Regex;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use ureq::http::{Method, Request};

fn io_err(msg: impl Into<String>) -> CatermError {
    CatermError::Io(IoError::Generic(msg.into()))
}

fn percent_decode(s: &str) -> String {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if let Some(&b'%') = bytes.get(i)
            && let (Some(&h1), Some(&h2)) = (bytes.get(i + 1), bytes.get(i + 2))
        {
            let hex_bytes = [h1, h2];
            if let Ok(hex_str) = std::str::from_utf8(&hex_bytes)
                && let Ok(byte) = u8::from_str_radix(hex_str, 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        if let Some(&b) = bytes.get(i) {
            out.push(b);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_date(s: &str) -> u64 {
    let trimmed = s.trim();
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(trimmed) {
        dt.timestamp().max(0) as u64
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        dt.timestamp().max(0) as u64
    } else {
        0
    }
}

/// WebDAV backend implementing `RemoteFileSystem`.
pub struct WebDavFileSystem {
    pub host_id: String,
}

impl WebDavFileSystem {
    /// # Infallible: creates an instance holding host_id.
    pub fn new(host_id: impl Into<String>) -> Self {
        Self {
            host_id: host_id.into(),
        }
    }

    fn get_conn(&self) -> Result<(String, Option<String>), CatermError> {
        let (host, secret) = crate::store::load_host_for_connect(&self.host_id)?;
        let raw = host.address.trim();
        let base = if raw.starts_with("http://") || raw.starts_with("https://") {
            raw.trim_end_matches('/').to_string()
        } else {
            let scheme = if host.port == 80 { "http" } else { "https" };
            let port_part = if host.port == 0 || host.port == 80 || host.port == 443 {
                String::new()
            } else {
                format!(":{}", host.port)
            };
            format!("{scheme}://{}{port_part}", raw.trim_end_matches('/'))
        };

        let auth = if !host.username.is_empty() {
            let pass = secret.unwrap_or_default();
            let cred = format!("{}:{}", host.username, pass);
            let enc = base64::engine::general_purpose::STANDARD.encode(cred.as_bytes());
            Some(format!("Basic {enc}"))
        } else {
            None
        };

        Ok((base, auth))
    }

    fn url_for(&self, base: &str, remote_path: &str) -> String {
        let clean = remote_path.trim().trim_matches('/');
        if clean.is_empty() || clean == "." {
            format!("{base}/")
        } else {
            format!("{base}/{clean}")
        }
    }
}

impl RemoteFileSystem for WebDavFileSystem {
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let method = Method::from_bytes(b"PROPFIND").map_err(|e| io_err(e.to_string()))?;
        let mut req_builder = Request::builder()
            .method(method)
            .uri(&target_url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8");

        if let Some(auth_val) = &auth {
            req_builder = req_builder.header("Authorization", auth_val);
        }

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:getcontentlength/>
    <D:getlastmodified/>
    <D:resourcetype/>
  </D:prop>
</D:propfind>"#;

        let req = req_builder
            .body(body)
            .map_err(|e| io_err(format!("Build request error: {e}")))?;

        let mut resp = agent
            .run(req)
            .map_err(|e| io_err(format!("WebDAV PROPFIND failed: {e}")))?;

        let status = resp.status().as_u16();
        if status != 207 && status != 200 {
            return Err(io_err(format!("WebDAV PROPFIND returned HTTP {status}")));
        }

        let xml = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| io_err(format!("Failed to read PROPFIND response: {e}")))?;

        let response_re = Regex::new(
            r"(?is)<([a-zA-Z0-9_-]+:)?response\b[^>]*>(.*?)</([a-zA-Z0-9_-]+:)?response>",
        )
        .map_err(|e| io_err(e.to_string()))?;
        let href_re =
            Regex::new(r"(?is)<([a-zA-Z0-9_-]+:)?href\b[^>]*>(.*?)</([a-zA-Z0-9_-]+:)?href>")
                .map_err(|e| io_err(e.to_string()))?;
        let len_re = Regex::new(r"(?is)<([a-zA-Z0-9_-]+:)?getcontentlength\b[^>]*>(\d+)</([a-zA-Z0-9_-]+:)?getcontentlength>")
            .map_err(|e| io_err(e.to_string()))?;
        let mod_re = Regex::new(r"(?is)<([a-zA-Z0-9_-]+:)?getlastmodified\b[^>]*>(.*?)</([a-zA-Z0-9_-]+:)?getlastmodified>")
            .map_err(|e| io_err(e.to_string()))?;

        let mut entries = Vec::new();
        let target_clean = target_url.trim_end_matches('/');

        for cap in response_re.captures_iter(&xml) {
            let block = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let href_raw = match href_re.captures(block).and_then(|c| c.get(2)) {
                Some(h) => h.as_str().trim(),
                None => continue,
            };

            let href_decoded = percent_decode(href_raw);
            let href_clean = href_decoded.trim_end_matches('/');

            // Skip the current directory itself
            if href_clean == target_clean
                || (href_clean.ends_with(remote_path.trim_matches('/')) && href_raw.ends_with('/'))
            {
                let last_segment = href_clean.rsplit('/').next().unwrap_or("");
                let req_segment = remote_path
                    .trim_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("");
                if last_segment == req_segment {
                    continue;
                }
            }

            let name = href_clean.rsplit('/').next().unwrap_or("").to_string();
            if name.is_empty() || name == "." || name == ".." {
                continue;
            }

            let is_dir = block.to_lowercase().contains(":collection")
                || block.to_lowercase().contains("<collection");

            let size = len_re
                .captures(block)
                .and_then(|c| c.get(2))
                .and_then(|m| m.as_str().parse::<u64>().ok())
                .unwrap_or(0);

            let mtime = mod_re
                .captures(block)
                .and_then(|c| c.get(2))
                .map(|m| parse_date(m.as_str()))
                .unwrap_or(0);

            let entry_path = crate::sftp::join_remote(remote_path, &name)
                .unwrap_or_else(|_| format!("{}/{}", remote_path.trim_end_matches('/'), name));

            entries.push(SftpFileEntry {
                name,
                path: entry_path,
                is_dir,
                is_symlink: false,
                size,
                mtime,
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
    }

    fn stat(&self, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let method = Method::from_bytes(b"PROPFIND").map_err(|e| io_err(e.to_string()))?;
        let mut req_builder = Request::builder()
            .method(method)
            .uri(&target_url)
            .header("Depth", "0")
            .header("Content-Type", "application/xml; charset=utf-8");

        if let Some(auth_val) = &auth {
            req_builder = req_builder.header("Authorization", auth_val);
        }

        let body = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:getcontentlength/>
    <D:getlastmodified/>
    <D:resourcetype/>
  </D:prop>
</D:propfind>"#;

        let req = req_builder
            .body(body)
            .map_err(|e| io_err(format!("Build request error: {e}")))?;

        let mut resp = agent
            .run(req)
            .map_err(|e| io_err(format!("WebDAV PROPFIND failed: {e}")))?;

        let status = resp.status().as_u16();
        if status != 207 && status != 200 {
            return Err(io_err(format!("WebDAV item not found: HTTP {status}")));
        }

        let xml = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| io_err(format!("Failed to read PROPFIND response: {e}")))?;

        let len_re = Regex::new(r"(?is)<([a-zA-Z0-9_-]+:)?getcontentlength\b[^>]*>(\d+)</([a-zA-Z0-9_-]+:)?getcontentlength>")
            .map_err(|e| io_err(e.to_string()))?;
        let mod_re = Regex::new(r"(?is)<([a-zA-Z0-9_-]+:)?getlastmodified\b[^>]*>(.*?)</([a-zA-Z0-9_-]+:)?getlastmodified>")
            .map_err(|e| io_err(e.to_string()))?;

        let is_dir = xml.to_lowercase().contains(":collection")
            || xml.to_lowercase().contains("<collection");

        let size = len_re
            .captures(&xml)
            .and_then(|c| c.get(2))
            .and_then(|m| m.as_str().parse::<u64>().ok())
            .unwrap_or(0);

        let mtime = mod_re
            .captures(&xml)
            .and_then(|c| c.get(2))
            .map(|m| parse_date(m.as_str()))
            .unwrap_or(0);

        let name = Path::new(remote_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| remote_path.to_string());

        Ok(SftpFileEntry {
            name,
            path: remote_path.to_string(),
            is_dir,
            is_symlink: false,
            size,
            mtime,
            mode: if is_dir { 0o755 } else { 0o644 },
        })
    }

    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let mut req = agent.get(&target_url);
        if let Some(auth_val) = &auth {
            req = req.header("Authorization", auth_val);
        }

        let mut resp = req
            .call()
            .map_err(|e| io_err(format!("WebDAV GET failed: {e}")))?;

        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("WebDAV GET returned HTTP {status}")));
        }

        resp.body_mut()
            .read_to_vec()
            .map_err(|e| io_err(format!("Failed to read response body: {e}")))
    }

    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let mut req = agent
            .put(&target_url)
            .header("Content-Type", "application/octet-stream");

        if let Some(auth_val) = &auth {
            req = req.header("Authorization", auth_val);
        }

        let resp = req
            .send(data)
            .map_err(|e| io_err(format!("WebDAV PUT failed: {e}")))?;

        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("WebDAV PUT returned HTTP {status}")));
        }

        Ok(())
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let method = Method::from_bytes(b"MKCOL").map_err(|e| io_err(e.to_string()))?;
        let mut req_builder = Request::builder().method(method).uri(&target_url);

        if let Some(auth_val) = &auth {
            req_builder = req_builder.header("Authorization", auth_val);
        }

        let req = req_builder
            .body("")
            .map_err(|e| io_err(format!("Build MKCOL request error: {e}")))?;

        let resp = agent
            .run(req)
            .map_err(|e| io_err(format!("WebDAV MKCOL failed: {e}")))?;

        let status = resp.status().as_u16();
        if (200..=299).contains(&status) || status == 405 {
            Ok(())
        } else {
            Err(io_err(format!("WebDAV MKCOL returned HTTP {status}")))
        }
    }

    fn delete(
        &self,
        remote_path: &str,
        _is_dir: bool,
        _recursive: bool,
    ) -> Result<(), CatermError> {
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let mut req = agent.delete(&target_url);
        if let Some(auth_val) = &auth {
            req = req.header("Authorization", auth_val);
        }

        let resp = req
            .call()
            .map_err(|e| io_err(format!("WebDAV DELETE failed: {e}")))?;

        let status = resp.status().as_u16();
        if (200..=299).contains(&status) || status == 404 {
            Ok(())
        } else {
            Err(io_err(format!("WebDAV DELETE returned HTTP {status}")))
        }
    }

    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError> {
        let (base, auth) = self.get_conn()?;
        let old_url = self.url_for(&base, old_path);
        let new_url = self.url_for(&base, new_path);

        let agent = ureq::Agent::new_with_defaults();
        let method = Method::from_bytes(b"MOVE").map_err(|e| io_err(e.to_string()))?;
        let mut req_builder = Request::builder()
            .method(method)
            .uri(&old_url)
            .header("Destination", &new_url)
            .header("Overwrite", "T");

        if let Some(auth_val) = &auth {
            req_builder = req_builder.header("Authorization", auth_val);
        }

        let req = req_builder
            .body("")
            .map_err(|e| io_err(format!("Build MOVE request error: {e}")))?;

        let resp = agent
            .run(req)
            .map_err(|e| io_err(format!("WebDAV MOVE failed: {e}")))?;

        let status = resp.status().as_u16();
        if (200..=299).contains(&status) {
            Ok(())
        } else {
            Err(io_err(format!("WebDAV MOVE returned HTTP {status}")))
        }
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
            .map_err(|e| io_err(format!("Failed to open local file {local_path}: {e}")))?;

        let total_bytes = local_file.metadata().map(|m| m.len()).unwrap_or(0);
        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let mut data = Vec::with_capacity(total_bytes as usize);
        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if crate::sftp::is_cancelled(transfer_id) {
                crate::sftp::clear_cancel_token(transfer_id);
                return Err(io_err("Transfer cancelled by user"));
            }

            let n = local_file
                .read(&mut buffer)
                .map_err(|e| io_err(format!("Failed to read local file: {e}")))?;
            if n == 0 {
                break;
            }

            if let Some(chunk) = buffer.get(..n) {
                data.extend_from_slice(chunk);
            }
            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed > 0.0 {
                    (bytes_transferred as f64 / elapsed) as u64
                } else {
                    0
                };
                let mut cb = on_progress.lock();
                cb(SftpProgressPayload {
                    transfer_id: transfer_id.to_string(),
                    bytes_transferred,
                    total_bytes,
                    speed_bps,
                });
                last_emit = Instant::now();
            }
        }

        let agent = ureq::Agent::new_with_defaults();
        let mut req = agent
            .put(&target_url)
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", total_bytes.to_string());

        if let Some(auth_val) = &auth {
            req = req.header("Authorization", auth_val);
        }

        let resp = req
            .send(&data)
            .map_err(|e| io_err(format!("WebDAV upload PUT failed: {e}")))?;

        let status = resp.status().as_u16();
        crate::sftp::clear_cancel_token(transfer_id);

        if (200..=299).contains(&status) {
            Ok(())
        } else {
            Err(io_err(format!("WebDAV upload returned HTTP {status}")))
        }
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
            .map_err(|e| io_err(format!("Failed to create local file {local_path}: {e}")))?;

        let (base, auth) = self.get_conn()?;
        let target_url = self.url_for(&base, remote_path);

        let agent = ureq::Agent::new_with_defaults();
        let mut req = agent.get(&target_url);
        if let Some(auth_val) = &auth {
            req = req.header("Authorization", auth_val);
        }

        let resp = req
            .call()
            .map_err(|e| io_err(format!("WebDAV GET failed: {e}")))?;

        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("WebDAV GET returned HTTP {status}")));
        }

        let total_bytes = resp
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let mut reader = resp.into_body().into_reader();
        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let start_time = Instant::now();
        let mut last_emit = Instant::now();

        loop {
            if crate::sftp::is_cancelled(transfer_id) {
                crate::sftp::clear_cancel_token(transfer_id);
                let _ = std::fs::remove_file(local_path);
                return Err(io_err("Transfer cancelled by user"));
            }

            let n = reader
                .read(&mut buffer)
                .map_err(|e| io_err(format!("Failed reading WebDAV download stream: {e}")))?;
            if n == 0 {
                break;
            }

            if let Some(chunk) = buffer.get(..n) {
                local_file
                    .write_all(chunk)
                    .map_err(|e| io_err(format!("Failed writing to {local_path}: {e}")))?;
            }

            bytes_transferred += n as u64;

            if last_emit.elapsed().as_millis() >= 100 || bytes_transferred == total_bytes {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed_bps = if elapsed > 0.0 {
                    (bytes_transferred as f64 / elapsed) as u64
                } else {
                    0
                };
                let mut cb = on_progress.lock();
                cb(SftpProgressPayload {
                    transfer_id: transfer_id.to_string(),
                    bytes_transferred,
                    total_bytes,
                    speed_bps,
                });
                last_emit = Instant::now();
            }
        }

        crate::sftp::clear_cancel_token(transfer_id);
        Ok(())
    }
}
