import { invoke } from '@tauri-apps/api/core';

export interface SftpFileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink?: boolean;
  size: number;
  mtime: number;
  mode?: number;
}

export interface SftpProgressPayload {
  transfer_id: string;
  bytes_transferred: number;
  total_bytes: number;
  speed_bps: number;
}

export function listRemoteDir(hostId: string, remotePath: string): Promise<SftpFileEntry[]> {
  return invoke('list_remote_dir', { hostId, remotePath });
}

export function sftpStat(hostId: string, remotePath: string): Promise<SftpFileEntry> {
  return invoke('sftp_stat', { hostId, remotePath });
}

export function readRemoteFile(hostId: string, remotePath: string): Promise<number[]> {
  return invoke('read_remote_file', { hostId, remotePath });
}

export function writeRemoteFile(hostId: string, remotePath: string, data: number[]): Promise<void> {
  return invoke('write_remote_file', { hostId, remotePath, data });
}

export function mkdirRemoteDir(hostId: string, remotePath: string): Promise<void> {
  return invoke('mkdir_remote_dir', { hostId, remotePath });
}

export function deleteRemoteFile(
  hostId: string,
  remotePath: string,
  isDir = false,
  recursive = false
): Promise<void> {
  return invoke('delete_remote_file', { hostId, remotePath, isDir, recursive });
}

export function renameRemoteFile(
  hostId: string,
  oldPath: string,
  newPath: string
): Promise<void> {
  return invoke('sftp_rename', { hostId, oldPath, newPath });
}

export function copyRemoteFile(
  hostId: string,
  srcPath: string,
  dstPath: string
): Promise<void> {
  return invoke('sftp_copy', { hostId, srcPath, dstPath });
}

export function sftpChmod(
  hostId: string,
  remotePath: string,
  mode: number
): Promise<void> {
  return invoke('sftp_chmod', { hostId, remotePath, mode });
}

export function sftpUpload(
  hostId: string,
  localPath: string,
  remotePath: string,
  transferId: string
): Promise<void> {
  return invoke('sftp_upload', { hostId, localPath, remotePath, transferId });
}

export function sftpDownload(
  hostId: string,
  remotePath: string,
  localPath: string,
  transferId: string
): Promise<void> {
  return invoke('sftp_download', { hostId, remotePath, localPath, transferId });
}

export function sftpCancel(transferId: string): Promise<void> {
  return invoke('sftp_cancel', { transferId });
}

export function sftpCompress(
  hostId: string,
  parentDir: string,
  items: string[],
  archiveName: string
): Promise<void> {
  return invoke('sftp_compress', { hostId, parentDir, items, archiveName });
}

export function sftpExtract(
  hostId: string,
  archivePath: string,
  destDir: string
): Promise<void> {
  return invoke('sftp_extract', { hostId, archivePath, destDir });
}

export interface RemoteSearchItem {
  path: string;
  name: string;
  size: number;
  mtime: number;
  is_dir: boolean;
}

export function searchRemoteFiles(
  hostId: string,
  basePath: string,
  pattern: string,
  maxResults?: number,
  minSize?: number,
  maxSize?: number
): Promise<RemoteSearchItem[]> {
  return invoke('search_remote_files', {
    hostId,
    basePath,
    pattern,
    maxResults: maxResults ?? 100,
    minSize: minSize ?? null,
    maxSize: maxSize ?? null
  });
}

export interface ChecksumComparison {
  local_checksum: string;
  remote_checksum: string;
  matches: boolean;
}

export function calculateRemoteChecksum(
  hostId: string,
  path: string,
  algorithm: 'sha256' | 'md5'
): Promise<string> {
  return invoke('calculate_remote_checksum', { hostId, path, algorithm });
}

export function calculateLocalChecksum(
  path: string,
  algorithm: 'sha256' | 'md5'
): Promise<string> {
  return invoke('calculate_local_checksum', { path, algorithm });
}

export function compareFileChecksums(
  hostId: string,
  remotePath: string,
  localPath: string,
  algorithm: 'sha256' | 'md5'
): Promise<ChecksumComparison> {
  return invoke('compare_file_checksums', { hostId, remotePath, localPath, algorithm });
}
