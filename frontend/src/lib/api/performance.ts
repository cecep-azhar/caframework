import {
  cmdGetPerformancePrefs,
  cmdSetPerformancePrefs,
  type PerformancePrefs
} from '$lib/generated/commands';

export type { PerformancePrefs };

export function getPerformancePrefs(): Promise<PerformancePrefs> {
  return cmdGetPerformancePrefs();
}

export function setPerformancePrefs(prefs: PerformancePrefs): Promise<void> {
  return cmdSetPerformancePrefs({ prefs });
}
