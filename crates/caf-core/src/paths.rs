//! Runtime data directory resolution (REQ-30). One function, called identically by
//! `caframeworkctl` and by the Tauri app (K2-2) — there is exactly one place this decision
//! is made.
//!
//! Precedence: `CATERM_DATA_DIR` env override > per-OS default data dir > `./caframework-data`
//! next to the running binary when a `portable.txt` marker sits beside it.

use crate::error::{CatermError, IoError};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataDirSource {
    EnvOverride,
    PerOsDefault,
    Portable,
}

impl DataDirSource {
    /// # Infallible: exhaustive match over a fieldless enum, no I/O, cannot fail.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnvOverride => "env_override",
            Self::PerOsDefault => "per_os_default",
            Self::Portable => "portable",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataDirInfo {
    pub path: PathBuf,
    pub source: DataDirSource,
}

fn portable_marker_present() -> Result<bool, CatermError> {
    let exe = std::env::current_exe()
        .map_err(|e| CatermError::Io(IoError::Generic(format!("current_exe gagal: {e}"))))?;
    Ok(exe
        .parent()
        .map(|dir| dir.join("portable.txt").is_file())
        .unwrap_or(false))
}

/// Resolve the data directory following the precedence documented on this module.
pub fn resolve_data_dir() -> Result<DataDirInfo, CatermError> {
    let env_override = std::env::var("CATERM_DATA_DIR").ok();
    resolve_data_dir_with(env_override.as_deref())
}

/// Pure decision logic, separated from `resolve_data_dir()` so it is testable without
/// mutating the real process environment (`std::env::set_var` in tests is a data-race
/// hazard across parallel test threads — the same reason it's banned in `clippy.toml`).
fn resolve_data_dir_with(env_override: Option<&str>) -> Result<DataDirInfo, CatermError> {
    if let Some(dir) = env_override {
        return Ok(DataDirInfo {
            path: PathBuf::from(dir),
            source: DataDirSource::EnvOverride,
        });
    }

    if portable_marker_present()? {
        return Ok(DataDirInfo {
            path: PathBuf::from("./caframework-data"),
            source: DataDirSource::Portable,
        });
    }

    let base = directories::BaseDirs::new().ok_or_else(|| {
        CatermError::Io(IoError::Generic(
            "unable to determine OS data directory (HOME/APPDATA unreadable)".into(),
        ))
    })?;
    Ok(DataDirInfo {
        path: base.data_dir().join("caframework"),
        source: DataDirSource::PerOsDefault,
    })
}

/// Conventional path to the SQLite store inside the resolved data directory. The real
/// store module (T2-CORE-01, Fase 1) is the actual owner of this filename — this
/// function only needs to agree with it, not implement it.
/// # Infallible: pure path join, no I/O, cannot fail.
pub fn db_path(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("caframework.db")
}

/// Where local crash dumps live before they are reviewed and either sent or discarded
/// (crash-reporting spec §6). Never inside the encrypted vault: a dump must be readable and
/// scrubbable before the vault is unlocked, since a crash can happen on the lock screen.
/// # Infallible: pure path join, no I/O, cannot fail.
pub fn crash_dumps_dir(data_dir: &std::path::Path) -> PathBuf {
    data_dir.join("crash-dumps")
}

/// Expands a leading `~` or `~/...` in an SSH key path to the user's home directory.
/// Paths without a leading `~` are returned unchanged.
/// # Infallible: falls back to the original string when the home directory can't be
/// resolved rather than failing the whole connect flow — there is no error to report.
pub fn expand_tilde(path: &str) -> String {
    let Some(rest) = path.strip_prefix('~') else {
        return path.to_string();
    };
    let Some(base) = directories::BaseDirs::new() else {
        return path.to_string();
    };
    let home = base.home_dir();
    if let Some(rest) = rest.strip_prefix('/') {
        home.join(rest).to_string_lossy().into_owned()
    } else if rest.is_empty() {
        home.to_string_lossy().into_owned()
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod expand_tilde_tests {
    use super::expand_tilde;

    #[test]
    fn leaves_absolute_paths_unchanged() {
        assert_eq!(
            expand_tilde("/home/user/.ssh/id_ed25519"),
            "/home/user/.ssh/id_ed25519"
        );
    }

    #[test]
    fn expands_leading_tilde_slash() {
        println!("EXPANDED: {}", expand_tilde("~/.ssh/id_ed25519"));
        let expanded = expand_tilde("~/.ssh/id_ed25519");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.ends_with("/.ssh/id_ed25519") || expanded.ends_with(".ssh/id_ed25519"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_override_takes_precedence() {
        let info = resolve_data_dir_with(Some("/tmp/caframework_test_probe")).expect("resolve gagal");
        assert_eq!(info.source, DataDirSource::EnvOverride);
        assert_eq!(info.path, PathBuf::from("/tmp/caframework_test_probe"));
    }

    #[test]
    fn db_path_is_data_dir_slash_caframework_db() {
        let dir = PathBuf::from("/tmp/x");
        assert_eq!(db_path(&dir), PathBuf::from("/tmp/x/caframework.db"));
    }
}
