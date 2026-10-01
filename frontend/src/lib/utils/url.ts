import { invoke } from '@tauri-apps/api/core';

export async function openExternalUrl(url: string): Promise<void> {
  if (!url) return;
  try {
    await invoke('open_external_url', { url });
  } catch (err) {
    console.error('Failed to open external url:', err);
  }
}
