//! S3 backend (`RemoteFileSystem`).
//!
//! Provides file transfer and directory operations over Amazon S3 / S3-compatible APIs
//! (MinIO, Wasabi, Cloudflare R2, Ceph) using `ureq` and AWS Signature Version 4.

use crate::error::{CatermError, IoError};
use crate::sftp::{SftpFileEntry, SftpProgressPayload};
use crate::vfs::{ProgressCallback, RemoteFileSystem};
use chrono::Utc;
use hmac::{Hmac, Mac};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;
use ureq::http::{Method, Request};

fn io_err(msg: impl Into<String>) -> CatermError {
    CatermError::Io(IoError::Generic(msg.into()))
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, CatermError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| io_err(format!("HMAC init failed: {e}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn uri_encode(s: &str, encode_slash: bool) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'~' | b'.' => {
                out.push(b as char);
            }
            b'/' if !encode_slash => {
                out.push('/');
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn parse_date(s: &str) -> u64 {
    let trimmed = s.trim();
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        dt.timestamp().max(0) as u64
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(trimmed) {
        dt.timestamp().max(0) as u64
    } else {
        0
    }
}

/// S3 backend implementing `RemoteFileSystem`.
pub struct S3FileSystem {
    pub host_id: String,
}

impl S3FileSystem {
    /// # Infallible: creates an instance holding host_id.
    pub fn new(host_id: impl Into<String>) -> Self {
        Self {
            host_id: host_id.into(),
        }
    }

    fn get_conn(&self) -> Result<S3ConnInfo, CatermError> {
        let (host, secret) = crate::store::load_host_for_connect(&self.host_id)?;
        let raw = host.address.trim();

        let (_scheme, host_header, base_url) =
            if raw.starts_with("http://") || raw.starts_with("https://") {
                let s = if raw.starts_with("https://") {
                    "https"
                } else {
                    "http"
                };
                let stripped = raw
                    .trim_start_matches("https://")
                    .trim_start_matches("http://");
                let host_part = stripped.trim_end_matches('/');
                (
                    s.to_string(),
                    host_part.to_string(),
                    format!("{s}://{host_part}"),
                )
            } else {
                let scheme = if host.port == 80 { "http" } else { "https" };
                let host_header = if host.port == 0 || host.port == 80 || host.port == 443 {
                    raw.trim_end_matches('/').to_string()
                } else {
                    format!("{}:{}", raw.trim_end_matches('/'), host.port)
                };
                let base_url = format!("{scheme}://{host_header}");
                (scheme.to_string(), host_header, base_url)
            };

        // Determine region: default to us-east-1 unless present in address
        let region = if let Some(idx) = host_header.find(".s3.") {
            let rest = host_header.get(idx + 4..).unwrap_or("");
            let end_idx = rest.find('.').unwrap_or(rest.len());
            let r = rest.get(..end_idx).unwrap_or("us-east-1");
            if r.is_empty() || r == "amazonaws" {
                "us-east-1".to_string()
            } else {
                r.to_string()
            }
        } else {
            "us-east-1".to_string()
        };

        let access_key = host.username.trim().to_string();
        let secret_key = secret.unwrap_or_default();

        Ok(S3ConnInfo {
            base_url,
            host_header,
            region,
            access_key,
            secret_key,
        })
    }

    fn parse_path(remote_path: &str) -> (Option<String>, Option<String>) {
        let clean = remote_path.trim().trim_matches('/');
        if clean.is_empty() || clean == "." {
            (None, None)
        } else if let Some(idx) = clean.find('/') {
            let bucket = clean.get(..idx).map(|s| s.to_string());
            let key = clean.get(idx + 1..).map(|s| s.to_string());
            (bucket, key)
        } else {
            (Some(clean.to_string()), None)
        }
    }

    fn sign_and_execute(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: &[u8],
        extra_headers: &[(&str, &str)],
    ) -> Result<ureq::http::Response<ureq::Body>, CatermError> {
        let conn = self.get_conn()?;
        let canonical_uri = if path.starts_with('/') {
            uri_encode(path, false)
        } else {
            format!("/{}", uri_encode(path, false))
        };

        let mut sorted_query = query.to_vec();
        sorted_query.sort_by_key(|(k, _)| *k);
        let canonical_query = sorted_query
            .iter()
            .map(|(k, v)| format!("{}={}", uri_encode(k, true), uri_encode(v, true)))
            .collect::<Vec<_>>()
            .join("&");

        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let payload_hash = sha256_hex(body);

        let mut headers_to_sign = vec![
            ("host", conn.host_header.clone()),
            ("x-amz-content-sha256", payload_hash.clone()),
            ("x-amz-date", amz_date.clone()),
        ];

        for (k, v) in extra_headers {
            headers_to_sign.push((k, v.to_string()));
        }
        headers_to_sign.sort_by_key(|(k, _)| *k);

        let canonical_headers = headers_to_sign
            .iter()
            .map(|(k, v)| format!("{k}:{v}\n"))
            .collect::<String>();

        let signed_headers = headers_to_sign
            .iter()
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
            .join(";");

        let canonical_request = format!(
            "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );

        let credential_scope = format!("{date_stamp}/{}/s3/aws4_request", conn.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );

        let auth_header = if !conn.access_key.is_empty() && !conn.secret_key.is_empty() {
            let k_secret = format!("AWS4{}", conn.secret_key);
            let k_date = hmac_sha256(k_secret.as_bytes(), date_stamp.as_bytes())?;
            let k_region = hmac_sha256(&k_date, conn.region.as_bytes())?;
            let k_service = hmac_sha256(&k_region, b"s3")?;
            let k_signing = hmac_sha256(&k_service, b"aws4_request")?;
            let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes())?);
            Some(format!(
                "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
                conn.access_key
            ))
        } else {
            None
        };

        let full_url = if canonical_query.is_empty() {
            format!("{}{}", conn.base_url, canonical_uri)
        } else {
            format!("{}{}?{}", conn.base_url, canonical_uri, canonical_query)
        };

        let agent = ureq::Agent::new_with_defaults();
        let http_method =
            Method::from_bytes(method.as_bytes()).map_err(|e| io_err(e.to_string()))?;
        let mut builder = Request::builder()
            .method(http_method)
            .uri(&full_url)
            .header("Host", &conn.host_header)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", &payload_hash);

        if let Some(auth) = auth_header {
            builder = builder.header("Authorization", auth);
        }

        for (k, v) in extra_headers {
            builder = builder.header(*k, *v);
        }

        let req = builder
            .body(body.to_vec())
            .map_err(|e| io_err(format!("Build S3 request failed: {e}")))?;

        agent
            .run(req)
            .map_err(|e| io_err(format!("S3 HTTP request failed: {e}")))
    }
}

struct S3ConnInfo {
    base_url: String,
    host_header: String,
    region: String,
    access_key: String,
    secret_key: String,
}

impl RemoteFileSystem for S3FileSystem {
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError> {
        let (bucket_opt, key_opt) = Self::parse_path(remote_path);

        match bucket_opt {
            // Root level: list all buckets
            None => {
                let mut resp = self.sign_and_execute("GET", "/", &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    return Err(io_err(format!(
                        "S3 ListAllMyBuckets returned HTTP {status}"
                    )));
                }

                let xml = resp
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| io_err(format!("Failed reading S3 XML: {e}")))?;

                let bucket_re = Regex::new(r"(?is)<Bucket\b[^>]*>(.*?)</Bucket>")
                    .map_err(|e| io_err(e.to_string()))?;
                let name_re = Regex::new(r"(?is)<Name\b[^>]*>(.*?)</Name>")
                    .map_err(|e| io_err(e.to_string()))?;
                let date_re = Regex::new(r"(?is)<CreationDate\b[^>]*>(.*?)</CreationDate>")
                    .map_err(|e| io_err(e.to_string()))?;

                let mut entries = Vec::new();
                for cap in bucket_re.captures_iter(&xml) {
                    let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let name = match name_re.captures(block).and_then(|c| c.get(1)) {
                        Some(n) => n.as_str().trim().to_string(),
                        None => continue,
                    };
                    let mtime = date_re
                        .captures(block)
                        .and_then(|c| c.get(1))
                        .map(|m| parse_date(m.as_str()))
                        .unwrap_or(0);

                    entries.push(SftpFileEntry {
                        name: name.clone(),
                        path: format!("/{name}"),
                        is_dir: true,
                        is_symlink: false,
                        size: 0,
                        mtime,
                        mode: 0o755,
                    });
                }

                entries.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(entries)
            }

            // Bucket level or subfolder level: list objects via ListObjectsV2
            Some(bucket) => {
                let prefix = match key_opt {
                    Some(k) if !k.is_empty() => {
                        if k.ends_with('/') {
                            k
                        } else {
                            format!("{k}/")
                        }
                    }
                    _ => String::new(),
                };

                let query_params: Vec<(&str, &str)> = if prefix.is_empty() {
                    vec![("delimiter", "/"), ("list-type", "2")]
                } else {
                    vec![("delimiter", "/"), ("list-type", "2"), ("prefix", &prefix)]
                };

                let path = format!("/{bucket}");
                let mut resp = self.sign_and_execute("GET", &path, &query_params, &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    return Err(io_err(format!("S3 ListObjectsV2 returned HTTP {status}")));
                }

                let xml = resp
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| io_err(format!("Failed reading S3 XML: {e}")))?;

                let prefix_re = Regex::new(r"(?is)<CommonPrefixes\b[^>]*>.*?<Prefix\b[^>]*>(.*?)</Prefix>.*?</CommonPrefixes>")
                    .map_err(|e| io_err(e.to_string()))?;
                let contents_re = Regex::new(r"(?is)<Contents\b[^>]*>(.*?)</Contents>")
                    .map_err(|e| io_err(e.to_string()))?;
                let key_re = Regex::new(r"(?is)<Key\b[^>]*>(.*?)</Key>")
                    .map_err(|e| io_err(e.to_string()))?;
                let size_re = Regex::new(r"(?is)<Size\b[^>]*>(\d+)</Size>")
                    .map_err(|e| io_err(e.to_string()))?;
                let mod_re = Regex::new(r"(?is)<LastModified\b[^>]*>(.*?)</LastModified>")
                    .map_err(|e| io_err(e.to_string()))?;

                let mut entries = Vec::new();

                // 1. Folders (CommonPrefixes)
                for cap in prefix_re.captures_iter(&xml) {
                    let full_prefix = match cap.get(1) {
                        Some(m) => m.as_str().trim(),
                        None => continue,
                    };
                    let trimmed = full_prefix.trim_end_matches('/');
                    let name = trimmed.rsplit('/').next().unwrap_or(trimmed).to_string();
                    if name.is_empty() {
                        continue;
                    }

                    entries.push(SftpFileEntry {
                        name,
                        path: format!("/{bucket}/{trimmed}"),
                        is_dir: true,
                        is_symlink: false,
                        size: 0,
                        mtime: 0,
                        mode: 0o755,
                    });
                }

                // 2. Objects (Contents)
                for cap in contents_re.captures_iter(&xml) {
                    let block = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let key = match key_re.captures(block).and_then(|c| c.get(1)) {
                        Some(k) => k.as_str().trim().to_string(),
                        None => continue,
                    };

                    // Skip the prefix folder marker itself
                    if key == prefix || key == format!("{prefix}/") || key.ends_with('/') {
                        continue;
                    }

                    let name = key.rsplit('/').next().unwrap_or(&key).to_string();
                    let size = size_re
                        .captures(block)
                        .and_then(|c| c.get(1))
                        .and_then(|m| m.as_str().parse::<u64>().ok())
                        .unwrap_or(0);
                    let mtime = mod_re
                        .captures(block)
                        .and_then(|c| c.get(1))
                        .map(|m| parse_date(m.as_str()))
                        .unwrap_or(0);

                    entries.push(SftpFileEntry {
                        name,
                        path: format!("/{bucket}/{key}"),
                        is_dir: false,
                        is_symlink: false,
                        size,
                        mtime,
                        mode: 0o644,
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
        }
    }

    fn stat(&self, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
        let (bucket_opt, key_opt) = Self::parse_path(remote_path);

        match (bucket_opt, key_opt) {
            (None, None) => Ok(SftpFileEntry {
                name: "/".to_string(),
                path: "/".to_string(),
                is_dir: true,
                is_symlink: false,
                size: 0,
                mtime: 0,
                mode: 0o755,
            }),
            (Some(bucket), None) => Ok(SftpFileEntry {
                name: bucket.clone(),
                path: format!("/{bucket}"),
                is_dir: true,
                is_symlink: false,
                size: 0,
                mtime: 0,
                mode: 0o755,
            }),
            (Some(bucket), Some(key)) => {
                let path = format!("/{bucket}/{key}");
                let resp = self.sign_and_execute("HEAD", &path, &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    // Check if it's a virtual folder
                    let entries = self.list_dir(remote_path)?;
                    if !entries.is_empty() {
                        let name = key.rsplit('/').next().unwrap_or(&key).to_string();
                        return Ok(SftpFileEntry {
                            name,
                            path: format!("/{bucket}/{key}"),
                            is_dir: true,
                            is_symlink: false,
                            size: 0,
                            mtime: 0,
                            mode: 0o755,
                        });
                    }
                    return Err(io_err(format!("S3 object not found: HTTP {status}")));
                }

                let size = resp
                    .headers()
                    .get("Content-Length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);

                let mtime = resp
                    .headers()
                    .get("Last-Modified")
                    .and_then(|v| v.to_str().ok())
                    .map(parse_date)
                    .unwrap_or(0);

                let name = key.rsplit('/').next().unwrap_or(&key).to_string();
                Ok(SftpFileEntry {
                    name,
                    path: format!("/{bucket}/{key}"),
                    is_dir: false,
                    is_symlink: false,
                    size,
                    mtime,
                    mode: 0o644,
                })
            }
            (None, Some(_)) => Err(io_err("Invalid S3 path")),
        }
    }

    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError> {
        let (bucket, key) = Self::parse_path(remote_path);
        let b = bucket.ok_or_else(|| io_err("Cannot read root directory as file"))?;
        let k = key.ok_or_else(|| io_err("Cannot read bucket root as file"))?;

        let path = format!("/{b}/{k}");
        let mut resp = self.sign_and_execute("GET", &path, &[], &[], &[])?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("S3 GetObject returned HTTP {status}")));
        }

        resp.body_mut()
            .read_to_vec()
            .map_err(|e| io_err(format!("Failed reading S3 body: {e}")))
    }

    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
        let (bucket, key) = Self::parse_path(remote_path);
        let b = bucket.ok_or_else(|| io_err("Cannot write to root"))?;
        let k = key.ok_or_else(|| io_err("Cannot write directly to bucket root"))?;

        let path = format!("/{b}/{k}");
        let resp = self.sign_and_execute("PUT", &path, &[], data, &[])?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("S3 PutObject returned HTTP {status}")));
        }
        Ok(())
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError> {
        let (bucket, key) = Self::parse_path(remote_path);
        match (bucket, key) {
            (None, _) => Err(io_err("Cannot mkdir at root")),
            (Some(b), None) => {
                // Create bucket
                let path = format!("/{b}");
                let resp = self.sign_and_execute("PUT", &path, &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    return Err(io_err(format!("S3 CreateBucket returned HTTP {status}")));
                }
                Ok(())
            }
            (Some(b), Some(k)) => {
                // Create directory marker object (ending in /)
                let clean_key = if k.ends_with('/') { k } else { format!("{k}/") };
                let path = format!("/{b}/{clean_key}");
                let resp = self.sign_and_execute("PUT", &path, &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    return Err(io_err(format!(
                        "S3 create folder marker returned HTTP {status}"
                    )));
                }
                Ok(())
            }
        }
    }

    fn delete(
        &self,
        remote_path: &str,
        _is_dir: bool,
        _recursive: bool,
    ) -> Result<(), CatermError> {
        let (bucket, key) = Self::parse_path(remote_path);
        match (bucket, key) {
            (None, _) => Err(io_err("Cannot delete root")),
            (Some(b), None) => {
                // Delete bucket
                let path = format!("/{b}");
                let resp = self.sign_and_execute("DELETE", &path, &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    return Err(io_err(format!("S3 DeleteBucket returned HTTP {status}")));
                }
                Ok(())
            }
            (Some(b), Some(k)) => {
                let path = format!("/{b}/{k}");
                let resp = self.sign_and_execute("DELETE", &path, &[], &[], &[])?;
                let status = resp.status().as_u16();
                if !(200..300).contains(&status) {
                    // Also try deleting folder marker if failed
                    let folder_path = format!("/{b}/{k}/");
                    let _ = self.sign_and_execute("DELETE", &folder_path, &[], &[], &[]);
                }
                Ok(())
            }
        }
    }

    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError> {
        let (old_bucket, old_key) = Self::parse_path(old_path);
        let (new_bucket, new_key) = Self::parse_path(new_path);

        let ob = old_bucket.ok_or_else(|| io_err("Cannot rename root"))?;
        let ok = old_key.ok_or_else(|| io_err("Cannot rename bucket root"))?;
        let nb = new_bucket.ok_or_else(|| io_err("Cannot rename to root"))?;
        let nk = new_key.ok_or_else(|| io_err("Cannot rename to bucket root"))?;

        // 1. Copy Object: PUT /new_bucket/new_key with x-amz-copy-source: /old_bucket/old_key
        let copy_source = format!("/{ob}/{}", uri_encode(&ok, false));
        let dst_path = format!("/{nb}/{nk}");
        let resp = self.sign_and_execute(
            "PUT",
            &dst_path,
            &[],
            &[],
            &[("x-amz-copy-source", &copy_source)],
        )?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("S3 CopyObject failed: HTTP {status}")));
        }

        // 2. Delete old object
        let src_path = format!("/{ob}/{ok}");
        let _ = self.sign_and_execute("DELETE", &src_path, &[], &[], &[]);
        Ok(())
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
        let (bucket, key) = Self::parse_path(remote_path);
        let b = bucket.ok_or_else(|| io_err("Cannot upload to root"))?;
        let k = key.ok_or_else(|| io_err("Cannot upload directly to bucket root"))?;

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
                .map_err(|e| io_err(format!("Failed reading {local_path}: {e}")))?;
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

        let path = format!("/{b}/{k}");
        let resp = self.sign_and_execute("PUT", &path, &[], &data, &[])?;
        let status = resp.status().as_u16();
        crate::sftp::clear_cancel_token(transfer_id);

        if !(200..300).contains(&status) {
            return Err(io_err(format!("S3 upload PUT returned HTTP {status}")));
        }
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
            .map_err(|e| io_err(format!("Failed to create local file {local_path}: {e}")))?;

        let (bucket, key) = Self::parse_path(remote_path);
        let b = bucket.ok_or_else(|| io_err("Cannot download root directory"))?;
        let k = key.ok_or_else(|| io_err("Cannot download bucket root"))?;

        let path = format!("/{b}/{k}");
        let resp = self.sign_and_execute("GET", &path, &[], &[], &[])?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(io_err(format!("S3 GetObject returned HTTP {status}")));
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
                .map_err(|e| io_err(format!("Failed reading S3 download stream: {e}")))?;
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
