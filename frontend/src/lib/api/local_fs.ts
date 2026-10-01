import { invoke } from '@tauri-apps/api/core';

export interface LocalFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  mtime: number;
  mode: number;
}

export function localListDir(path: string): Promise<LocalFileEntry[]> {
  return invoke('local_list_dir', { path });
}

export function localStat(path: string): Promise<LocalFileEntry> {
  return invoke('local_stat', { path });
}

export function localMkdir(path: string): Promise<void> {
  return invoke('local_mkdir', { path });
}

export function localDelete(
  path: string,
  isDir = false,
  recursive = false
): Promise<void> {
  return invoke('local_delete', { path, isDir, recursive });
}

export function localRename(oldPath: string, newPath: string): Promise<void> {
  return invoke('local_rename', { oldPath, newPath });
}

export function localReadFile(path: string): Promise<number[]> {
  return invoke('local_read_file', { path });
}

export function localWriteFile(path: string, data: number[]): Promise<void> {
  return invoke('local_write_file', { path, data });
}

export function calculateLocalChecksum(path: string, algorithm: 'sha256' | 'md5'): Promise<string> {
  return invoke('calculate_local_checksum', { path, algorithm });
}
