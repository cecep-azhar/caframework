// Per-device host preferences the vault schema has no column for: starred hosts and when each
// host last had a session opened. Local UI state only — not synced, not in backups.

const STORAGE_KEY = 'caframework_host_prefs_v1';

interface StoredPrefs {
  favorites: string[];
  lastUsed: Record<string, number>;
}

function load(): StoredPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? (JSON.parse(raw) as Partial<StoredPrefs>) : {};
    return {
      favorites: Array.isArray(parsed.favorites) ? parsed.favorites : [],
      lastUsed: parsed.lastUsed && typeof parsed.lastUsed === 'object' ? parsed.lastUsed : {}
    };
  } catch {
    return { favorites: [], lastUsed: {} };
  }
}

const initial = typeof localStorage === 'undefined' ? { favorites: [], lastUsed: {} } : load();
let favorites = $state<string[]>(initial.favorites);
let lastUsed = $state<Record<string, number>>(initial.lastUsed);

function persist() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ favorites, lastUsed }));
  } catch {
    // Storage blocked: preferences still apply for this session.
  }
}

export function isFavorite(hostId: string): boolean {
  return favorites.includes(hostId);
}

export function toggleFavorite(hostId: string) {
  favorites = isFavorite(hostId) ? favorites.filter((id) => id !== hostId) : [...favorites, hostId];
  persist();
}

export function lastUsedAt(hostId: string): number | undefined {
  return lastUsed[hostId];
}

export function markHostUsed(hostId: string) {
  lastUsed = { ...lastUsed, [hostId]: Date.now() };
  persist();
}
