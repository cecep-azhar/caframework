// Terminal font size, shared by every open pane (Ctrl + / Ctrl − / Ctrl 0 while a terminal has
// focus). Browser zoom is off in release builds, so this is how text gets bigger. A per-device
// convenience, so localStorage is enough; storage failures just mean it isn't remembered.

const STORAGE_KEY = 'caframework_terminal_font_size';
export const DEFAULT_FONT_SIZE = 13;
const MIN_FONT_SIZE = 8;
const MAX_FONT_SIZE = 32;

function clamp(size: number): number {
  return Math.min(MAX_FONT_SIZE, Math.max(MIN_FONT_SIZE, Math.round(size)));
}

function load(): number {
  try {
    const stored = Number(localStorage.getItem(STORAGE_KEY));
    return Number.isFinite(stored) && stored > 0 ? clamp(stored) : DEFAULT_FONT_SIZE;
  } catch {
    return DEFAULT_FONT_SIZE;
  }
}

let fontSize = $state(typeof localStorage === 'undefined' ? DEFAULT_FONT_SIZE : load());

export function getTerminalPrefs() {
  return {
    get fontSize() {
      return fontSize;
    }
  };
}

export function setTerminalFontSize(size: number) {
  fontSize = clamp(size);
  try {
    localStorage.setItem(STORAGE_KEY, String(fontSize));
  } catch {
    // Not remembered; still applies for this session.
  }
}

export function stepTerminalFontSize(delta: number) {
  setTerminalFontSize(fontSize + delta);
}
