// Light/dark theme store. Applies `.dark` class to <html> for Tailwind CSS reactivity.

export type ThemeName = 'dark' | 'light';

const STORAGE_KEY = 'caframework-theme';

const state = $state<{ name: ThemeName }>({ name: 'dark' });

export function getTheme() {
  return state;
}

export function isDark(): boolean {
  return state.name === 'dark';
}

/** Mirrors the choice onto <html> so Tailwind's `dark:` variants follow along. */
function applyToDocument(name: ThemeName) {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('dark', name === 'dark');
}

export function setTheme(name: ThemeName) {
  state.name = name;
  applyToDocument(name);
  try {
    localStorage.setItem(STORAGE_KEY, name);
  } catch {
    // Private mode / blocked storage: the theme still applies for this session.
  }
}

export function toggleTheme() {
  setTheme(state.name === 'dark' ? 'light' : 'dark');
}

/** Reads the saved choice once at startup. Defaults to dark, as CAFramework always has. */
export function initTheme() {
  let saved: string | null = null;
  try {
    saved = localStorage.getItem(STORAGE_KEY);
  } catch {
    saved = null;
  }
  setTheme(saved === 'light' ? 'light' : 'dark');
}
