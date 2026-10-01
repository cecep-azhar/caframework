// Thin wrapper over @tauri-apps/plugin-updater. The check and the signature verification both
// run in Rust against `plugins.updater` in tauri.conf.json (GitHub `latest.json` + minisign
// pubkey); nothing here talks to the network directly.
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type { Update };

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__);
}

/** Resolves to the pending update, or null when this build is already the latest. */
export async function checkForAppUpdate(): Promise<Update | null> {
  if (!isTauriRuntime()) {
    throw new Error('Updates are only available in the desktop app.');
  }
  return check();
}

/**
 * Downloads, verifies and installs `update`, then restarts. On Windows the installer exits the
 * app itself during install, so `relaunch` is only reached on macOS/Linux.
 */
export async function installAppUpdate(
  update: Update,
  onProgress?: (downloaded: number, total: number | null) => void
): Promise<void> {
  let downloaded = 0;
  let total: number | null = null;
  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === 'Started') {
      total = event.data.contentLength ?? null;
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      onProgress?.(downloaded, total);
    }
  });
  await relaunch();
}
