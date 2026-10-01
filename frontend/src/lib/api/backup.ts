import { invoke } from '@tauri-apps/api/core';

export function exportEncryptedBackup(passphrase: string): Promise<string> {
  return invoke('export_encrypted_backup', { passphrase });
}

export function importEncryptedBackup(encryptedB64: string, passphrase: string): Promise<number> {
  return invoke('import_encrypted_backup', { encryptedB64, passphrase });
}