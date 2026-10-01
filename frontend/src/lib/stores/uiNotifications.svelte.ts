// Elegant, unified UI Notifications & Confirm Dialog store (Zero browser alert/confirm).
import { t } from '$lib/i18n/index.svelte';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'info';
  message: string;
}

export interface ConfirmDialogState {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText: string;
  cancelText: string;
  danger: boolean;
  resolve?: (value: boolean) => void;
}

let toasts = $state<Toast[]>([]);
let confirmDialog = $state<ConfirmDialogState>({
  isOpen: false,
  title: '',
  message: '',
  confirmText: 'Confirm',
  cancelText: 'Cancel',
  danger: false
});

export function getToasts(): Toast[] {
  return toasts;
}

export function showToast(message: string, type: 'success' | 'error' | 'info' = 'info') {
  const id = `toast-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
  toasts = [...toasts, { id, type, message }];
  setTimeout(() => {
    removeToast(id);
  }, 4000);
}

export function removeToast(id: string) {
  toasts = toasts.filter((toast) => toast.id !== id);
}

export function getConfirmDialog(): ConfirmDialogState {
  return confirmDialog;
}

export function confirmModal(
  message: string,
  // Defaults resolve at call time, so they follow the language active when the dialog opens.
  title = t('notify.confirmTitle'),
  danger = false,
  confirmText = t('notify.continue'),
  cancelText = t('common.cancel')
): Promise<boolean> {
  return new Promise((resolve) => {
    confirmDialog = {
      isOpen: true,
      title,
      message,
      confirmText,
      cancelText,
      danger,
      resolve
    };
  });
}

export function resolveConfirm(result: boolean) {
  if (confirmDialog.resolve) {
    confirmDialog.resolve(result);
  }
  confirmDialog.isOpen = false;
  confirmDialog.resolve = undefined;
}
