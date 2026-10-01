//! Virtual Remote File System abstraction (`RemoteFileSystem`).
//!
//! Provides a unified interface for remote file operations across
//! different protocols: SFTP, SCP, FTP, FTPS, WebDAV, and S3.

use crate::error::CatermError;
use crate::sftp::{SftpFileEntry, SftpProgressPayload};
use parking_lot::Mutex;
use std::sync::Arc;

pub type ProgressCallback = Arc<Mutex<dyn FnMut(SftpProgressPayload) + Send + 'static>>;

/// Trait abstracting remote file systems across various protocols.
pub trait RemoteFileSystem: Send + Sync {
    /// List entries in the directory at `remote_path`.
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError>;

    /// Retrieve metadata for a file or directory at `remote_path`.
    fn stat(&self, remote_path: &str) -> Result<SftpFileEntry, CatermError>;

    /// Read the full contents of a file at `remote_path`.
    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError>;

    /// Write `data` to a file at `remote_path`.
    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError>;

    /// Create a directory at `remote_path`.
    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError>;

    /// Delete a file or directory at `remote_path`.
    fn delete(&self, remote_path: &str, is_dir: bool, recursive: bool) -> Result<(), CatermError>;

    /// Rename or move a remote item from `old_path` to `new_path`.
    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError>;

    /// Change permissions of `remote_path`. Default returns `NotImplemented`.
    fn chmod(&self, remote_path: &str, mode: u32) -> Result<(), CatermError> {
        let _ = (remote_path, mode);
        Err(CatermError::NotImplemented(
            "chmod is not supported by this protocol".into(),
        ))
    }

    /// Upload a local file to `remote_path` with progress reporting.
    fn upload(
        &self,
        local_path: &str,
        remote_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError>;

    /// Download a remote file from `remote_path` to `local_path` with progress reporting.
    fn download(
        &self,
        remote_path: &str,
        local_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError>;
}

/// Sftp implementation of `RemoteFileSystem`.
pub struct SftpFileSystem {
    pub host_id: String,
}

impl SftpFileSystem {
    /// # Infallible: creates an SftpFileSystem instance holding host_id.
    pub fn new(host_id: impl Into<String>) -> Self {
        Self {
            host_id: host_id.into(),
        }
    }
}

impl RemoteFileSystem for SftpFileSystem {
    fn list_dir(&self, remote_path: &str) -> Result<Vec<SftpFileEntry>, CatermError> {
        crate::sftp::list_remote_dir(&self.host_id, remote_path)
    }

    fn stat(&self, remote_path: &str) -> Result<SftpFileEntry, CatermError> {
        crate::sftp::stat_remote(&self.host_id, remote_path)
    }

    fn read_file(&self, remote_path: &str) -> Result<Vec<u8>, CatermError> {
        crate::sftp::read_remote_file(&self.host_id, remote_path)
    }

    fn write_file(&self, remote_path: &str, data: &[u8]) -> Result<(), CatermError> {
        crate::sftp::write_remote_file(&self.host_id, remote_path, data)
    }

    fn mkdir(&self, remote_path: &str) -> Result<(), CatermError> {
        crate::sftp::mkdir_remote_dir(&self.host_id, remote_path)
    }

    fn delete(&self, remote_path: &str, is_dir: bool, recursive: bool) -> Result<(), CatermError> {
        crate::sftp::delete_remote_file(&self.host_id, remote_path, is_dir, recursive)
    }

    fn rename(&self, old_path: &str, new_path: &str) -> Result<(), CatermError> {
        crate::sftp::rename_remote_file(&self.host_id, old_path, new_path)
    }

    fn chmod(&self, remote_path: &str, mode: u32) -> Result<(), CatermError> {
        crate::sftp::chmod_remote_file(&self.host_id, remote_path, mode)
    }

    fn upload(
        &self,
        local_path: &str,
        remote_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError> {
        crate::sftp::upload_file_with_progress(
            &self.host_id,
            local_path,
            remote_path,
            transfer_id,
            on_progress,
        )
    }

    fn download(
        &self,
        remote_path: &str,
        local_path: &str,
        transfer_id: &str,
        on_progress: ProgressCallback,
    ) -> Result<(), CatermError> {
        crate::sftp::download_file_with_progress(
            &self.host_id,
            remote_path,
            local_path,
            transfer_id,
            on_progress,
        )
    }
}

/// Factory function returning the appropriate `RemoteFileSystem` implementation
/// based on the host's configured connection protocol.
pub fn get_remote_fs(host_id: &str) -> Result<Box<dyn RemoteFileSystem>, CatermError> {
    let host = crate::store::get_host(host_id)?;
    match host.protocol {
        crate::store::ConnectionProtocol::Ssh => Ok(Box::new(SftpFileSystem::new(host_id))),
        crate::store::ConnectionProtocol::Scp => {
            Ok(Box::new(crate::scp::ScpFileSystem::new(host_id)))
        }
        crate::store::ConnectionProtocol::Ftp => {
            Ok(Box::new(crate::ftp::FtpFileSystem::new(host_id, false)))
        }
        crate::store::ConnectionProtocol::Ftps => {
            Ok(Box::new(crate::ftp::FtpFileSystem::new(host_id, true)))
        }
        crate::store::ConnectionProtocol::WebDav => {
            Ok(Box::new(crate::webdav::WebDavFileSystem::new(host_id)))
        }
        crate::store::ConnectionProtocol::S3 => Ok(Box::new(crate::s3::S3FileSystem::new(host_id))),
    }
}
