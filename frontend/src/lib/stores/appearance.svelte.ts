// Appearance/performance prefs kept in localStorage. Separate from profile.svelte.ts
// because these are device-level preferences, not identity.

const STORAGE_KEY = 'caf_appearance_v1';

interface StoredAppearance {
  reduceMotion: boolean;
}

function load(): StoredAppearance | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<StoredAppearance>;
    if (typeof parsed.reduceMotion !== 'boolean') return null;
    return { reduceMotion: parsed.reduceMotion };
  } catch {
    return null;
  }
}

const initial = typeof localStorage === 'undefined' ? null : load();
// Decorative canvas background (lock screen grid flow) is on by default; users who want to
// claw back the GPU-accelerated process it keeps resident can turn it off in Settings ->
// Performance.
let reduceMotion = $state(initial?.reduceMotion ?? false);

export function getAppearance() {
  return {
    get reduceMotion() {
      return reduceMotion;
    }
  };
}

export function setReduceMotion(next: boolean) {
  reduceMotion = next;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ reduceMotion }));
  } catch {
    // Storage blocked: the change still applies for this session.
  }
}
