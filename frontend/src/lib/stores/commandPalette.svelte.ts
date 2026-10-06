// Command palette visibility & mode store (Ctrl+K / Ctrl+Shift+P).

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
