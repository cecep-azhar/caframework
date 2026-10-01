//! Local file system operations for file manager dual-pane.
use crate::error::{CatermError, IoError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalFileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
}

fn io_err(msg: impl Into<String>) -> CatermError {
    CatermError::Io(IoError::Generic(msg.into()))
}

fn resolve_path(p: &str) -> PathBuf {
    if p.is_empty() || p == "~" {
        dirs_home().unwrap_or_else(|| PathBuf::from("."))
    } else if let Some(stripped) = p.strip_prefix("~/") {
        if let Some(home) = dirs_home() {
            home.join(stripped)
        } else {
            PathBuf::from(p)
        }
    } else {
        PathBuf::from(p)
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn local_list_dir(path: &str) -> Result<Vec<LocalFileEntry>, CatermError> {
    let resolved = resolve_path(path);
    let canonical = resolved
        .canonicalize()
        .map_err(|e| io_err(format!("Cannot resolve path '{}': {e}", resolved.display())))?;

    let read_dir = fs::read_dir(&canonical).map_err(|e| {
        io_err(format!(
            "Failed to read directory '{}': {e}",
            canonical.display()
        ))
    })?;

    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        let symlink_meta = entry.metadata().ok();
        let is_symlink = entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false);

        let is_dir = symlink_meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = symlink_meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = symlink_meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            symlink_meta
                .as_ref()
                .map(|m| m.permissions().mode())
                .unwrap_or(0)
        };
        #[cfg(not(unix))]
        let mode = 0o644;

        entries.push(LocalFileEntry {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
            is_symlink,
            size,
            mtime,
            mode,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

pub fn local_stat(path: &str) -> Result<LocalFileEntry, CatermError> {
    let resolved = resolve_path(path);
    let canonical = resolved
        .canonicalize()
        .map_err(|e| io_err(format!("Path not found '{}': {e}", resolved.display())))?;

    let meta = fs::metadata(&canonical).map_err(|e| {
        io_err(format!(
            "Cannot get metadata for '{}': {e}",
            canonical.display()
        ))
    })?;

    let symlink_meta = fs::symlink_metadata(&canonical).ok();
    let is_symlink = symlink_meta
        .as_ref()
        .map(|m| m.is_symlink())
        .unwrap_or(false);
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "/".to_string());

    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);

    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode()
    };
    #[cfg(not(unix))]
    let mode = 0o644;

    Ok(LocalFileEntry {
        name,
        path: canonical.to_string_lossy().to_string(),
        is_dir: meta.is_dir(),
        is_symlink,
        size,
        mtime,
        mode,
    })
}

pub fn local_mkdir(path: &str) -> Result<(), CatermError> {
    let resolved = resolve_path(path);
    fs::create_dir_all(&resolved).map_err(|e| {
        io_err(format!(
            "Failed to create directory '{}': {e}",
            resolved.display()
        ))
    })
}

pub fn local_delete(path: &str, is_dir: bool, recursive: bool) -> Result<(), CatermError> {
    let target = Path::new(path);
    if !target.exists() {
        return Ok(());
    }

    if is_dir {
        if recursive {
            fs::remove_dir_all(target)
                .map_err(|e| io_err(format!("Failed to remove directory '{}': {e}", path)))?;
        } else {
            fs::remove_dir(target)
                .map_err(|e| io_err(format!("Failed to remove directory '{}': {e}", path)))?;
        }
    } else {
        fs::remove_file(target)
            .map_err(|e| io_err(format!("Failed to remove file '{}': {e}", path)))?;
    }
    Ok(())
}

pub fn local_rename(old_path: &str, new_path: &str) -> Result<(), CatermError> {
    let old_p = Path::new(old_path);
    let new_p = Path::new(new_path);
    fs::rename(old_p, new_p).map_err(|e| {
        io_err(format!(
            "Failed to rename '{}' -> '{}': {e}",
            old_path, new_path
        ))
    })
}

pub fn local_read_file(path: &str) -> Result<Vec<u8>, CatermError> {
    let resolved = resolve_path(path);
    fs::read(&resolved)
        .map_err(|e| io_err(format!("Failed to read file '{}': {e}", resolved.display())))
}

pub fn local_write_file(path: &str, data: &[u8]) -> Result<(), CatermError> {
    let resolved = resolve_path(path);
    if let Some(parent) = resolved.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&resolved, data).map_err(|e| {
        io_err(format!(
            "Failed to write file '{}': {e}",
            resolved.display()
        ))
    })
}

/// Compute SHA-256 or MD5 checksum of a local file.
pub fn calculate_local_checksum(path: &Path, algorithm: &str) -> Result<String, CatermError> {
    use md5::Md5;
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let algo = algorithm.to_lowercase();
    if algo != "sha256" && algo != "md5" {
        return Err(io_err(format!(
            "unsupported algorithm '{algorithm}'; use sha256 or md5"
        )));
    }

    let mut file = fs::File::open(path)
        .map_err(|e| io_err(format!("Cannot open local file '{}': {e}", path.display())))?;

    let mut buf = vec![0u8; 64 * 1024];

    if algo == "sha256" {
        let mut hasher = Sha256::new();
        loop {
            let n = file.read(&mut buf).map_err(|e| {
                io_err(format!(
                    "Failed reading local file '{}': {e}",
                    path.display()
                ))
            })?;
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
            let n = file.read(&mut buf).map_err(|e| {
                io_err(format!(
                    "Failed reading local file '{}': {e}",
                    path.display()
                ))
            })?;
            if n == 0 {
                break;
            }
            if let Some(chunk) = buf.get(..n) {
                hasher.update(chunk);
            }
        }
        Ok(hex::encode(hasher.finalize()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_list_and_stat() {
        let temp_dir = std::env::temp_dir().join(format!("caframework_test_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("hello.txt");
        fs::write(&file_path, b"world").unwrap();

        let entries = local_list_dir(temp_dir.to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "hello.txt");
        assert_eq!(entries[0].size, 5);
        assert!(!entries[0].is_dir);

        let stat = local_stat(file_path.to_str().unwrap()).unwrap();
        assert_eq!(stat.name, "hello.txt");
        assert_eq!(stat.size, 5);

        let sha = calculate_local_checksum(&file_path, "sha256").unwrap();
        // sha256 of "world" is 486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7
        assert_eq!(
            sha,
            "486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7"
        );

        let md5 = calculate_local_checksum(&file_path, "md5").unwrap();
        // md5 of "world" is 7d793037a0760186574b0282f2f435e7
        assert_eq!(md5, "7d793037a0760186574b0282f2f435e7");

        local_delete(file_path.to_str().unwrap(), false, false).unwrap();
        assert!(!file_path.exists());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
