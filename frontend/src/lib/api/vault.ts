import {
  cmdIsVaultInitialized,
  cmdValidateVaultPassword,
  cmdLockVault,
  cmdChangeMasterPassword,
  cmdResetVault
} from '$lib/generated/commands';

export const MIN_VAULT_PASSWORD_LEN = 8;

export function validateVaultPassword(password: string): Promise<boolean> {
  return cmdValidateVaultPassword({ password });
}

export function isVaultInitialized(): Promise<boolean> {
  return cmdIsVaultInitialized();
}

export function resetVault(): Promise<void> {
  return cmdResetVault();
}

/** Zeroizes the in-memory vault key and stops tunnels. The next unlock re-derives it. */
export function lockVault(): Promise<void> {
  return cmdLockVault();
}

/** Re-keys the encrypted database and canary to `newPassword`. Fails if `oldPassword` is wrong. */
export function changeMasterPassword(oldPassword: string, newPassword: string): Promise<void> {
  return cmdChangeMasterPassword({ oldPassword, newPassword });
}
