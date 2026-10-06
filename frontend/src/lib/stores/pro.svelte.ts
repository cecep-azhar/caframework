// CATerm Pro state for the UI: who is signed in, what this device is entitled to, and whether
// the Pro server is reachable. Entitlement comes from the signed token checked offline by the
// backend, so being offline never downgrades a paying user (within the 14-day grace).
import {
  proStatus,
  proServerAvailable,
  proCommitPending,
  proSync,
  proErrorCode,
  type ProStatus,
  type SyncOutcome
} from '$lib/api/pro';

/** How often an unlocked app re-syncs its licence (the server re-issues a 14-day token). */
const SYNC_INTERVAL_MS = 6 * 60 * 60 * 1000;

let status = $state<ProStatus | null>(null);
let serverAvailable = $state<boolean | null>(null);
let syncing = $state(false);
let syncTimer: ReturnType<typeof setInterval> | null = null;

export function getPro() {
  return {
    get status() {
      return status;
    },
    /** This account's own Pro plan or trial is valid on this device. */
    get isPro() {
      const ent = status?.entitlement;
      return ent?.state === 'valid' && ent.features.includes('cloud_sync');
    },
    /** Access through someone else's team only (free account in an entitled team). */
    get isTeamMember() {
      const ent = status?.entitlement;
      return ent?.state === 'valid' && ent.tier === 'team';
    },
    /** null while unknown (not checked yet / browser preview). */
    get serverAvailable() {
      return serverAvailable;
    },
    get syncing() {
      return syncing;
    }
  };
}

export async function refreshProStatus(): Promise<void> {
  try {
    status = await proStatus();
  } catch {
    // Browser preview or vault locked mid-call: keep what we had.
  }
}

export async function checkProServer(): Promise<boolean> {
  try {
    serverAvailable = await proServerAvailable();
  } catch {
    serverAvailable = false;
  }
  return serverAvailable;
}

/** Heartbeat. Throws on failure so callers can show why; the status is refreshed either way. */
export async function syncPro(): Promise<SyncOutcome | null> {
  if (syncing) return null;
  syncing = true;
  try {
    const outcome = await proSync();
    status = outcome.status;
    return outcome;
  } catch (err) {
    if (proErrorCode(err) === 'SESSION_EXPIRED') await refreshProStatus();
    throw err;
  } finally {
    syncing = false;
  }
}

export function setProStatus(next: ProStatus): void {
  status = next;
}

/**
 * Called once the vault is open: saves a sign-in made on the lock screen, then syncs in the
 * background (and every few hours while the app stays unlocked).
 */
export async function onVaultUnlocked(): Promise<void> {
  try {
    await proCommitPending();
  } catch {
    // Nothing pending, or no backend (browser preview).
  }
  await refreshProStatus();
  void checkProServer();
  if (status?.signedIn) void syncPro().catch(() => {});
  if (syncTimer) clearInterval(syncTimer);
  syncTimer = setInterval(() => {
    if (status?.signedIn) void syncPro().catch(() => {});
  }, SYNC_INTERVAL_MS);
}

/** Vault locked or app signed out: forget in-memory Pro state until the next unlock. */
export function onVaultLocked(): void {
  if (syncTimer) clearInterval(syncTimer);
  syncTimer = null;
  status = null;
}
