import {
  cmdGetPendingCrashReport,
  cmdDismissCrashReport,
  type ScrubbedCrashReport
} from '$lib/generated/commands';

export type { ScrubbedCrashReport };

/** Null when there is nothing pending, reporting is disabled, or the dump directory is empty. */
export async function getPendingCrashReport(): Promise<ScrubbedCrashReport | null> {
  return await cmdGetPendingCrashReport();
}

/** Sends the scrubbed report (proxied through GCC — the real collector's DSN never reaches the
 * client) and deletes the local dump only once the server has actually accepted it. */
export async function submitCrashReport(reportId: string): Promise<void> {
  // To be implemented in F12 with remote endpoint
  console.warn('submitCrashReport endpoint pending F12');
}

/** Deletes the dump without sending it. `neverAgain` also disables the panic hook's future
 * writes and clears any other dumps already on disk (crash.rs::dismiss_crash_report). */
export async function dismissCrashReport(reportId: string, neverAgain = false): Promise<void> {
  await cmdDismissCrashReport({ id: reportId, neverAgain });
}
