import { invoke } from '@tauri-apps/api/core';

export interface PerformancePrefs {
  hardware_acceleration: boolean;
}

export async function getPerformancePrefs(): Promise<PerformancePrefs> {
  return await invoke<PerformancePrefs>('get_performance_prefs');
}

export async function savePerformancePrefs(prefs: PerformancePrefs): Promise<void> {
  await invoke('save_performance_prefs', { prefs });
}

export async function exportEncryptedBackup(password: string): Promise<string> {
  return await invoke<string>('export_vault_backup', { password });
}

export async function importEncryptedBackup(encryptedData: string, password: string): Promise<boolean> {
  return await invoke<boolean>('import_vault_backup', { encryptedData, password });
}
