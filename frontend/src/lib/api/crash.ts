import { invoke } from '@tauri-apps/api/core';

/**
 * A scrubbed, human-reviewable view of the newest pending crash dump (if any). Everything here
 * has already passed through `caf_core::crash`'s PII scrubber before it ever reaches the
 * frontend process — see caframework-crash-reporting-spec-v1.md §4. `previewJson` is exactly the
 * envelope that would be POSTed on submit, so "view details" shows the user precisely what
 * would be sent, not a paraphrase of it.
 */
export interface ScrubbedCrashReport {
  id: string;
  timestamp: string;
  appVersion: string;
  osName: string;
  message: string;
  location?: string;
  previewJson: string;
}

/** Null when there is nothing pending, reporting is disabled, or the dump directory is empty. */
export async function getPendingCrashReport(): Promise<ScrubbedCrashReport | null> {
  return invoke<ScrubbedCrashReport | null>('get_pending_crash_report');
}

/** Sends the scrubbed report (proxied through GCC — the real collector's DSN never reaches the
 * client) and deletes the local dump only once the server has actually accepted it. */
export async function submitCrashReport(reportId: string): Promise<void> {
  return invoke<void>('submit_crash_report', { reportId });
}

/** Deletes the dump without sending it. `neverAgain` also disables the panic hook's future
 * writes and clears any other dumps already on disk (crash.rs::dismiss_crash_report). */
export async function dismissCrashReport(reportId: string, neverAgain = false): Promise<void> {
  return invoke<void>('dismiss_crash_report', { reportId, neverAgain });
}
