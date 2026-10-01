// Command palette visibility. `hosts` mode (Ctrl+Shift+T, "new session") lists only things that
// open a terminal; `all` (Ctrl+K / Ctrl+Shift+P) adds open sessions, pages and actions.

export type PaletteMode = 'all' | 'hosts';

const palette = $state({ open: false, mode: 'all' as PaletteMode });

export function getPalette() {
  return palette;
}

export function openPalette(mode: PaletteMode = 'all') {
  palette.mode = mode;
  palette.open = true;
}

export function closePalette() {
  palette.open = false;
}
