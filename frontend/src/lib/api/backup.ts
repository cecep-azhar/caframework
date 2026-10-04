import { invoke_exportEncryptedBackup, invoke_importEncryptedBackup } from './generated_bindings';

export function exportEncryptedBackup(password: string): Promise<number[]> {
  return invoke_exportEncryptedBackup(password);
}

export function importEncryptedBackup(data: number[], password: string): Promise<void> {
  return invoke_importEncryptedBackup(data, password);
}