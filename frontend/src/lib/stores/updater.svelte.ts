// Shared update state: the floating toast (bottom-right) and Settings > Updates read the same
// check result, so a check started in one place is reflected in the other.
import { checkForAppUpdate, installAppUpdate, isTauriRuntime, type Update } from '$lib/api/updater';
import { errorText } from '$lib/errors';
import { showToast } from '$lib/stores/uiNotifications.svelte';
import { t } from '$lib/i18n/index.svelte';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'downloading'
  | 'error';

let status = $state<UpdateStatus>('idle');
let update = $state.raw<Update | null>(null);
let errorMessage = $state('');
let progress = $state<{ downloaded: number; total: number | null }>({ downloaded: 0, total: null });
let toastDismissed = $state(false);

export function getUpdater() {
  return {
    get status() {
      return status;
    },
    get version() {
      return update?.version ?? '';
    },
    get error() {
      return errorMessage;
    },
    get progress() {
      return progress;
    },
    /** The floating toast shows only for an update the user hasn't waved away this session. */
    get showToast() {
      return (status === 'available' || status === 'downloading') && !toastDismissed;
    }
  };
}

/**
 * `silent` is the startup background check: failures (offline, no release published yet,
 * `tauri dev` in a browser tab) leave the state idle instead of surfacing an error.
 */
export async function checkForUpdates({ silent = false } = {}): Promise<void> {
  if (status === 'checking' || status === 'downloading') return;
  if (silent && !isTauriRuntime()) return;
  status = 'checking';
  errorMessage = '';
  try {
    update = await checkForAppUpdate();
    status = update ? 'available' : 'up-to-date';
    if (update) toastDismissed = false;
  } catch (err) {
    update = null;
    if (silent) {
      console.warn('Background update check failed:', err);
      status = 'idle';
    } else {
      errorMessage = errorText(err);
      status = 'error';
    }
  }
}

export async function installUpdate(): Promise<void> {
  if (!update || status === 'downloading') return;
  status = 'downloading';
  progress = { downloaded: 0, total: null };
  try {
    await installAppUpdate(update, (downloaded, total) => {
      progress = { downloaded, total };
    });
  } catch (err) {
    errorMessage = errorText(err);
    status = 'error';
    showToast(t('updater.failed', { error: errorMessage }), 'error');
  }
}

export function dismissUpdateToast() {
  toastDismissed = true;
}
