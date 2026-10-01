//! Directory Sync & Live Watch Engine (`WinSCP Parity Feature #2`).
//! Provides directory comparison, sync plan generation, plan execution,
//! and recursive live filesystem watching with debounced SFTP upload.

use crate::error::{CatermError, SyncError};
use crate::sftp;
use notify::Watcher;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

fn sync_err(msg: impl Into<String>) -> CatermError {
    CatermError::Sync(SyncError::Generic(msg.into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    LocalToRemote,
    RemoteToLocal,
    TwoWay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPlan {
    pub to_upload: Vec<SyncEntry>,
    pub to_download: Vec<SyncEntry>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntry {
    pub local_path: String,
    pub remote_path: String,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub local_path: String,
    pub remote_path: String,
    pub local_mtime: u64,
    pub remote_mtime: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    pub files_uploaded: usize,
    pub files_downloaded: usize,
    pub bytes_transferred: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchHandle {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchInfo {
    pub id: String,
    pub host_id: String,
    pub local_dir: String,
    pub remote_dir: String,
    pub is_active: bool,
}

struct ActiveWatcher {
    info: WatchInfo,
    stop_tx: std::sync::mpsc::Sender<()>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

static WATCHERS: Lazy<Mutex<HashMap<String, ActiveWatcher>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
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

fn collect_local_files(
    base_dir: &Path,
) -> Result<HashMap<String, (PathBuf, u64, u64)>, CatermError> {
    let mut result = HashMap::new();
    if !base_dir.exists() {
        return Ok(result);
    }
    let mut queue = Vec::new();
    queue.push(base_dir.to_path_buf());

    while let Some(current_dir) = queue.pop() {
        let read_dir = match std::fs::read_dir(&current_dir) {
            Ok(rd) => rd,
            Err(e) => {
                return Err(sync_err(format!(
                    "Failed to read local dir '{}': {e}",
                    current_dir.display()
                )));
            }
        };

        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                if !file_type.is_symlink() {
                    queue.push(path);
                }
            } else if file_type.is_file() {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let size = meta.len();
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                if let Ok(rel) = path.strip_prefix(base_dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if !rel_str.is_empty() {
                        result.insert(rel_str, (path, size, mtime));
                    }
                }
            }
        }
    }

    Ok(result)
}

fn collect_remote_files(
    host_id: &str,
    base_dir: &str,
) -> Result<HashMap<String, sftp::SftpFileEntry>, CatermError> {
    let mut result = HashMap::new();
    let mut queue = Vec::new();
    queue.push(base_dir.to_string());

    let normalized_base = base_dir.trim_end_matches('/');

    while let Some(current_dir) = queue.pop() {
        let entries = match sftp::list_remote_dir(host_id, &current_dir) {
            Ok(e) => e,
            Err(_) => {
                if current_dir == base_dir {
                    return Ok(result);
                }
                continue;
            }
        };

        for entry in entries {
            if entry.is_dir {
                if !entry.is_symlink {
                    queue.push(entry.path.clone());
                }
            } else {
                let rel = if normalized_base.is_empty() {
                    entry.path.trim_start_matches('/').to_string()
                } else if let Some(stripped) = entry.path.strip_prefix(normalized_base) {
                    stripped.trim_start_matches('/').to_string()
                } else {
                    entry.name.clone()
                };

                if !rel.is_empty() {
                    result.insert(rel, entry);
                }
            }
        }
    }

    Ok(result)
}

fn ensure_remote_parent_dir(host_id: &str, remote_file_path: &str) -> Result<(), CatermError> {
    let path = Path::new(remote_file_path);
    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy().replace('\\', "/");
        if !parent_str.is_empty() && parent_str != "/" && parent_str != "." {
            let segments: Vec<&str> = parent_str.split('/').filter(|s| !s.is_empty()).collect();
            let is_absolute = parent_str.starts_with('/');
            let mut accumulated = if is_absolute {
                String::new()
            } else {
                ".".to_string()
            };

            for seg in segments {
                if is_absolute || (!accumulated.is_empty() && accumulated != ".") {
                    accumulated.push('/');
                }
                accumulated.push_str(seg);
                let _ = sftp::mkdir_remote_dir(host_id, &accumulated);
            }
        }
    }
    Ok(())
}

/// Scans local and remote directories, producing an actionable diff plan.
pub fn plan_sync(
    host_id: &str,
    local_dir: &str,
    remote_dir: &str,
    direction: SyncDirection,
) -> Result<SyncPlan, CatermError> {
    let resolved_local = resolve_path(local_dir);
    if !resolved_local.exists() {
        return Err(sync_err(format!(
            "Local directory does not exist: {}",
            resolved_local.display()
        )));
    }
    if !resolved_local.is_dir() {
        return Err(sync_err(format!(
            "Local path is not a directory: {}",
            resolved_local.display()
        )));
    }

    let local_files = collect_local_files(&resolved_local)?;
    let remote_files = collect_remote_files(host_id, remote_dir)?;

    let mut to_upload = Vec::new();
    let mut to_download = Vec::new();
    let mut conflicts = Vec::new();

    match direction {
        SyncDirection::LocalToRemote => {
            for (rel_path, (l_path, l_size, l_mtime)) in &local_files {
                let remote_path = sftp::join_remote(remote_dir, rel_path)?;
                if let Some(r_entry) = remote_files.get(rel_path) {
                    if *l_mtime != r_entry.mtime || *l_size != r_entry.size {
                        to_upload.push(SyncEntry {
                            local_path: l_path.to_string_lossy().to_string(),
                            remote_path,
                            size: *l_size,
                            mtime: *l_mtime,
                        });
                    }
                } else {
                    to_upload.push(SyncEntry {
                        local_path: l_path.to_string_lossy().to_string(),
                        remote_path,
                        size: *l_size,
                        mtime: *l_mtime,
                    });
                }
            }
        }
        SyncDirection::RemoteToLocal => {
            for (rel_path, r_entry) in &remote_files {
                let local_path = resolved_local.join(rel_path).to_string_lossy().to_string();
                if let Some((_, l_size, l_mtime)) = local_files.get(rel_path) {
                    if r_entry.mtime != *l_mtime || r_entry.size != *l_size {
                        to_download.push(SyncEntry {
                            local_path,
                            remote_path: r_entry.path.clone(),
                            size: r_entry.size,
                            mtime: r_entry.mtime,
                        });
                    }
                } else {
                    to_download.push(SyncEntry {
                        local_path,
                        remote_path: r_entry.path.clone(),
                        size: r_entry.size,
                        mtime: r_entry.mtime,
                    });
                }
            }
        }
        SyncDirection::TwoWay => {
            let mut all_keys: std::collections::BTreeSet<&String> =
                std::collections::BTreeSet::new();
            for k in local_files.keys() {
                all_keys.insert(k);
            }
            for k in remote_files.keys() {
                all_keys.insert(k);
            }

            for rel_path in all_keys {
                let local_opt = local_files.get(rel_path);
                let remote_opt = remote_files.get(rel_path);

                match (local_opt, remote_opt) {
                    (Some((l_path, l_size, l_mtime)), None) => {
                        let remote_path = sftp::join_remote(remote_dir, rel_path)?;
                        to_upload.push(SyncEntry {
                            local_path: l_path.to_string_lossy().to_string(),
                            remote_path,
                            size: *l_size,
                            mtime: *l_mtime,
                        });
                    }
                    (None, Some(r_entry)) => {
                        let local_path =
                            resolved_local.join(rel_path).to_string_lossy().to_string();
                        to_download.push(SyncEntry {
                            local_path,
                            remote_path: r_entry.path.clone(),
                            size: r_entry.size,
                            mtime: r_entry.mtime,
                        });
                    }
                    (Some((l_path, l_size, l_mtime)), Some(r_entry)) => {
                        let local_path_str = l_path.to_string_lossy().to_string();
                        let remote_path_str = r_entry.path.clone();

                        if *l_mtime == r_entry.mtime && *l_size == r_entry.size {
                            // in sync
                        } else if *l_mtime == r_entry.mtime && *l_size != r_entry.size {
                            conflicts.push(SyncConflict {
                                local_path: local_path_str,
                                remote_path: remote_path_str,
                                local_mtime: *l_mtime,
                                remote_mtime: r_entry.mtime,
                            });
                        } else if *l_mtime > r_entry.mtime {
                            to_upload.push(SyncEntry {
                                local_path: local_path_str,
                                remote_path: remote_path_str,
                                size: *l_size,
                                mtime: *l_mtime,
                            });
                        } else {
                            to_download.push(SyncEntry {
                                local_path: local_path_str,
                                remote_path: remote_path_str,
                                size: r_entry.size,
                                mtime: r_entry.mtime,
                            });
                        }
                    }
                    (None, None) => {}
                }
            }
        }
    }

    to_upload.sort_by(|a, b| a.local_path.cmp(&b.local_path));
    to_download.sort_by(|a, b| a.local_path.cmp(&b.local_path));
    conflicts.sort_by(|a, b| a.local_path.cmp(&b.local_path));

    Ok(SyncPlan {
        to_upload,
        to_download,
        conflicts,
    })
}

/// Executes a SyncPlan by uploading and downloading changed/missing files.
pub fn execute_sync(
    host_id: &str,
    plan: &SyncPlan,
    direction: SyncDirection,
) -> Result<SyncStats, CatermError> {
    let mut stats = SyncStats::default();

    if matches!(
        direction,
        SyncDirection::LocalToRemote | SyncDirection::TwoWay
    ) {
        for item in &plan.to_upload {
            ensure_remote_parent_dir(host_id, &item.remote_path)?;
            let transfer_id = format!("sync-up-{}", uuid::Uuid::new_v4());
            let noop = std::sync::Arc::new(parking_lot::Mutex::new(|_| {}));
            sftp::upload_file_with_progress(
                host_id,
                &item.local_path,
                &item.remote_path,
                &transfer_id,
                noop,
            )?;
            stats.files_uploaded += 1;
            stats.bytes_transferred += item.size;
        }
    }

    if matches!(
        direction,
        SyncDirection::RemoteToLocal | SyncDirection::TwoWay
    ) {
        for item in &plan.to_download {
            let local_path = Path::new(&item.local_path);
            if let Some(parent) = local_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let transfer_id = format!("sync-down-{}", uuid::Uuid::new_v4());
            let noop = std::sync::Arc::new(parking_lot::Mutex::new(|_| {}));
            sftp::download_file_with_progress(
                host_id,
                &item.remote_path,
                &item.local_path,
                &transfer_id,
                noop,
            )?;
            stats.files_downloaded += 1;
            stats.bytes_transferred += item.size;
        }
    }

    Ok(stats)
}

/// Starts watching a local directory for changes and automatically uploads modified files.
pub fn start_watch(
    host_id: &str,
    local_dir: &str,
    remote_dir: &str,
) -> Result<WatchHandle, CatermError> {
    let resolved_local = resolve_path(local_dir);
    if !resolved_local.exists() {
        return Err(sync_err(format!(
            "Local directory does not exist: {}",
            resolved_local.display()
        )));
    }
    if !resolved_local.is_dir() {
        return Err(sync_err(format!(
            "Local path is not a directory: {}",
            resolved_local.display()
        )));
    }

    let id = format!("watch-{}", uuid::Uuid::new_v4());
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();

    let host_id_clone = host_id.to_string();
    let remote_dir_clone = remote_dir.to_string();
    let local_dir_for_thread = resolved_local.clone();

    let thread_handle = std::thread::Builder::new()
        .name(format!("sync-watcher-{id}"))
        .spawn(move || {
            let (event_tx, event_rx) = std::sync::mpsc::channel();
            let mut watcher = match notify::recommended_watcher(move |res| {
                if let Ok(event) = res {
                    let _ = event_tx.send(event);
                }
            }) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create filesystem watcher: {e}");
                    return;
                }
            };

            if let Err(e) = watcher.watch(&local_dir_for_thread, notify::RecursiveMode::Recursive) {
                eprintln!(
                    "Failed to watch directory {}: {e}",
                    local_dir_for_thread.display()
                );
                return;
            }

            loop {
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                match event_rx.recv_timeout(std::time::Duration::from_millis(200)) {
                    Ok(initial_event) => {
                        let mut pending_paths = std::collections::HashSet::new();
                        for p in initial_event.paths {
                            pending_paths.insert(p);
                        }

                        let debounce_start = std::time::Instant::now();
                        while debounce_start.elapsed() < std::time::Duration::from_millis(500) {
                            if stop_rx.try_recv().is_ok() {
                                return;
                            }
                            let remaining = std::time::Duration::from_millis(500)
                                .saturating_sub(debounce_start.elapsed());
                            if let Ok(next_event) = event_rx.recv_timeout(remaining) {
                                for p in next_event.paths {
                                    pending_paths.insert(p);
                                }
                            }
                        }

                        for path in pending_paths {
                            if !path.is_file() {
                                continue;
                            }
                            if let Ok(rel) = path.strip_prefix(&local_dir_for_thread) {
                                let rel_str = rel.to_string_lossy().replace('\\', "/");
                                if rel_str.is_empty() {
                                    continue;
                                }
                                if let Ok(remote_path) =
                                    sftp::join_remote(&remote_dir_clone, &rel_str)
                                {
                                    let _ = ensure_remote_parent_dir(&host_id_clone, &remote_path);
                                    let transfer_id = format!("watch-up-{}", uuid::Uuid::new_v4());
                                    let noop = std::sync::Arc::new(parking_lot::Mutex::new(|_| {}));
                                    let _ = sftp::upload_file_with_progress(
                                        &host_id_clone,
                                        &path.to_string_lossy(),
                                        &remote_path,
                                        &transfer_id,
                                        noop,
                                    );
                                }
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .map_err(|e| sync_err(format!("Failed to spawn watcher thread: {e}")))?;

    let info = WatchInfo {
        id: id.clone(),
        host_id: host_id.to_string(),
        local_dir: local_dir.to_string(),
        remote_dir: remote_dir.to_string(),
        is_active: true,
    };

    let active = ActiveWatcher {
        info,
        stop_tx,
        thread_handle: Some(thread_handle),
    };

    WATCHERS.lock().insert(id.clone(), active);

    Ok(WatchHandle { id })
}

/// Stops an active watcher by ID.
pub fn stop_watch(id: &str) -> Result<(), CatermError> {
    let mut watchers = WATCHERS.lock();
    if let Some(watcher) = watchers.get_mut(id) {
        if watcher.info.is_active {
            let _ = watcher.stop_tx.send(());
            watcher.info.is_active = false;
        }
        if let Some(handle) = watcher.thread_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    } else {
        Err(sync_err(format!("Watcher with ID '{id}' not found")))
    }
}

/// Lists all registered watchers and their active status.
pub fn list_watches() -> Result<Vec<WatchInfo>, CatermError> {
    let watchers = WATCHERS.lock();
    let mut list: Vec<WatchInfo> = watchers.values().map(|w| w.info.clone()).collect();
    list.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(list)
}
