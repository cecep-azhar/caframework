// JS <-> Rust binding for vault password policy. Backed by `caf_core::vault` — see
// crates/caf-core/src/vault.rs for the source of truth.
import { invoke } from '@tauri-apps/api/core';

export const MIN_VAULT_PASSWORD_LEN = 8;

export function validateVaultPassword(password: string): Promise<void> {
  return invoke('validate_vault_password', { password });
}

export function isVaultInitialized(): Promise<boolean> {
  return invoke('is_vault_initialized');
}

export function resetVault(): Promise<void> {
  return invoke('reset_vault');
}

/** Zeroizes the in-memory vault key and stops tunnels. The next unlock re-derives it. */
export function lockVault(): Promise<void> {
  return invoke('lock_vault');
}

/** Re-keys the encrypted database and canary to `newPassword`. Fails if `currentPassword` is wrong. */
export function changeMasterPassword(currentPassword: string, newPassword: string): Promise<void> {
  return invoke('change_master_password', { currentPassword, newPassword });
}
