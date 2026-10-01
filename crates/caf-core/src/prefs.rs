//! Device-level preferences that must be readable *before* the vault is unlocked, so they
//! cannot live in the encrypted database. Today that is only how the WebView is created
//! (GPU on/off), which the shell has to know before the first window exists.
//!
//! Plain JSON next to the database in the data directory. Nothing sensitive may go here.

use crate::error::{CatermError, IoError};
use crate::paths::resolve_data_dir;
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE_NAME: &str = "performance.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformancePrefs {
    /// GPU-accelerated rendering for the WebView. Off by default to conserve memory.
    pub gpu_acceleration: bool,
    /// Automatic trimming of memory working set when app is idle or minimized.
    pub background_memory_saving: bool,
    /// Max scrollback history lines stored in memory per terminal session.
    pub scrollback_lines: u32,
    /// Sleep/throttle background inactive terminal sessions to reduce CPU usage.
    pub inactive_session_sleep: bool,
    /// Low power mode (30 FPS limiter) for battery saving and low-end hardware.
    pub low_power_mode: bool,
}

impl Default for PerformancePrefs {
    fn default() -> Self {
        Self {
            gpu_acceleration: false,
            background_memory_saving: true,
            scrollback_lines: 5000,
            inactive_session_sleep: true,
            low_power_mode: false,
        }
    }
}

/// If anything fails: missing, unreadable or corrupt file falls back to defaults, because a bad
/// preferences file must not be able to stop the app from opening its window.
pub fn load_performance_prefs() -> Result<PerformancePrefs, CatermError> {
    let info = resolve_data_dir()?;
    Ok(load_from(&info.path))
}

pub fn save_performance_prefs(prefs: &PerformancePrefs) -> Result<(), CatermError> {
    save_to(&resolve_data_dir()?.path, prefs)
}

fn load_from(dir: &Path) -> PerformancePrefs {
    std::fs::read(dir.join(FILE_NAME))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_to(dir: &Path, prefs: &PerformancePrefs) -> Result<(), CatermError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("create data dir: {e}"))))?;
    let json = serde_json::to_vec_pretty(prefs)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("serialize prefs: {e}"))))?;
    std::fs::write(dir.join(FILE_NAME), json)
        .map_err(|e| CatermError::Io(IoError::Generic(format!("write {FILE_NAME}: {e}"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("caframework-prefs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_file_yields_defaults() {
        let dir = scratch_dir("missing");
        assert_eq!(load_from(&dir), PerformancePrefs::default());
    }

    #[test]
    fn corrupt_file_yields_defaults() {
        let dir = scratch_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), b"{not json").unwrap();
        assert_eq!(load_from(&dir), PerformancePrefs::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn round_trips() {
        let dir = scratch_dir("roundtrip");
        let prefs = PerformancePrefs {
            gpu_acceleration: true,
            background_memory_saving: false,
            scrollback_lines: 10000,
            inactive_session_sleep: false,
            low_power_mode: true,
        };
        save_to(&dir, &prefs).unwrap();
        assert_eq!(load_from(&dir), prefs);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_or_missing_fields_use_defaults() {
        let dir = scratch_dir("partial");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), br#"{"somethingElse": 1}"#).unwrap();
        assert_eq!(load_from(&dir), PerformancePrefs::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
