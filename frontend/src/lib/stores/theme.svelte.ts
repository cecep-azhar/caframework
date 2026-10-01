// Light/dark theme, shared. It lived as local state inside +layout.svelte, which was fine for
// Tailwind (`dark:` classes react to the `.dark` class on <html>) but invisible to anything
// that paints its own colours in JavaScript — xterm.js being the one that matters: the
// terminal stayed black in light mode because nothing ever told it the theme had changed.

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

/**
 * xterm.js paints on a canvas and knows nothing about CSS, so it needs the palette handed to
 * it explicitly. ANSI colours are shared; only the surface colours differ, so the two themes
 * stay recognisably the same terminal.
 */
export function terminalTheme(name: ThemeName) {
  const ansi = {
    black: '#3f3f46',
    red: '#ef4444',
    green: '#22c55e',
    yellow: '#eab308',
    blue: '#3b82f6',
    magenta: '#a855f7',
    cyan: '#06b6d4',
    white: '#e4e4e7',
    brightBlack: '#71717a',
    brightRed: '#f87171',
    brightGreen: '#4ade80',
    brightYellow: '#facc15',
    brightBlue: '#60a5fa',
    brightMagenta: '#c084fc',
    brightCyan: '#22d3ee',
    brightWhite: '#fafafa'
  };

  if (name === 'light') {
    return {
      ...ansi,
      // Darker variants of the ANSI set: the defaults above are tuned for a black surface and
      // wash out badly on white.
      red: '#dc2626',
      green: '#15803d',
      yellow: '#a16207',
      blue: '#1d4ed8',
      magenta: '#7e22ce',
      cyan: '#0e7490',
      white: '#52525b',
      brightWhite: '#27272a',
      background: '#fafafa',
      foreground: '#18181b',
      cursor: '#0284c7',
      cursorAccent: '#fafafa',
      selectionBackground: '#bae6fd'
    };
  }

  return {
    ...ansi,
    background: '#09090b',
    foreground: '#e4e4e7',
    cursor: '#38bdf8',
    cursorAccent: '#09090b',
    selectionBackground: '#155e75'
  };
}
