import { invoke } from '@tauri-apps/api/core';

export type SyncDirection = 'LocalToRemote' | 'RemoteToLocal' | 'TwoWay';

export interface SyncEntry {
  local_path: string;
  remote_path: string;
  size: number;
  mtime: number;
}

export interface SyncConflict {
  local_path: string;
  remote_path: string;
  local_mtime: number;
  remote_mtime: number;
}

export interface SyncPlan {
  to_upload: SyncEntry[];
  to_download: SyncEntry[];
  conflicts: SyncConflict[];
}

export interface SyncStats {
  files_uploaded: number;
  files_downloaded: number;
  bytes_transferred: number;
}

export interface WatchHandle {
  id: string;
}

export interface WatchInfo {
  id: string;
  host_id: string;
  local_dir: string;
  remote_dir: string;
  is_active: boolean;
}

export function planSync(
  hostId: string,
  localDir: string,
  remoteDir: string,
  direction: SyncDirection
): Promise<SyncPlan> {
  return invoke('plan_sync', { hostId, localDir, remoteDir, direction });
}

export function executeSync(
  hostId: string,
  plan: SyncPlan,
  direction: SyncDirection
): Promise<SyncStats> {
  return invoke('execute_sync', { hostId, plan, direction });
}

export function startWatch(
  hostId: string,
  localDir: string,
  remoteDir: string
): Promise<WatchHandle> {
  return invoke('start_watch', { hostId, localDir, remoteDir });
}

export function stopWatch(id: string): Promise<void> {
  return invoke('stop_watch', { id });
}

export function listWatches(): Promise<WatchInfo[]> {
  return invoke('list_watches');
}
