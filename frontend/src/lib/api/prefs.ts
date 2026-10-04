import {
  cmdGetPerformancePrefs,
  cmdSetPerformancePrefs,
  cmdExportEncryptedBackup,
  cmdImportEncryptedBackup,
  type PerformancePrefs
} from '$lib/generated/commands';

export type { PerformancePrefs };

export async function getPerformancePrefs(): Promise<PerformancePrefs> {
  return await cmdGetPerformancePrefs();
}

export async function savePerformancePrefs(prefs: PerformancePrefs): Promise<void> {
  await cmdSetPerformancePrefs({ prefs });
}

export async function exportEncryptedBackup(password: string): Promise<number[]> {
  return await cmdExportEncryptedBackup({ password });
}

export async function importEncryptedBackup(data: number[], password: string): Promise<void> {
  await cmdImportEncryptedBackup({ data, password });
}
