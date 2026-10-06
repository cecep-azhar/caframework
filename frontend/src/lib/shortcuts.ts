// Keyboard shortcuts: one list that both the handlers and Settings → Shortcuts read, so the
// documented keys can't drift from the ones that work again.
//
// Rules that keep the terminal usable: plain Ctrl+<letter> belongs to the shell (Ctrl+C, Ctrl+D,
// Ctrl+K, Ctrl+W, …), so app shortcuts that must work while a terminal has focus use Ctrl+Shift,
// Ctrl+digit, Ctrl+Tab or Ctrl+comma. Ctrl+K opens the palette everywhere *except* in a terminal.

export const IS_MAC = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

/** An app-level command, run by the layout's capture-phase key listener. */
export type AppCommand =
  | { type: 'palette' }
  | { type: 'newSession' }
  | { type: 'tab'; index: number } // 0-based; -1 = last tab
  | { type: 'cycleTab'; step: 1 | -1 }
  | { type: 'closeTab' }
  | { type: 'settings' }
  | { type: 'lock' };

/** Ctrl on Windows/Linux, Cmd on macOS — and not both. */
function hasMod(e: KeyboardEvent): boolean {
  return IS_MAC ? e.metaKey && !e.ctrlKey : e.ctrlKey && !e.metaKey;
}

/** True when the key event comes from xterm's hidden textarea. */
export function isTerminalTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('.xterm') !== null;
}

/** An element where typing text is the point (inputs, editors) — not the terminal. */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  if (isTerminalTarget(target)) return false;
  return target.closest('input, textarea, select, [contenteditable="true"], .cm-editor') !== null;
}

export function appCommandFor(e: KeyboardEvent): AppCommand | null {
  if (e.type !== 'keydown' || e.isComposing || e.altKey) return null;

  // Tab cycling uses Ctrl on every platform (Cmd+Tab is the macOS app switcher).
  if (e.key === 'Tab' && e.ctrlKey && !e.metaKey) {
    return { type: 'cycleTab', step: e.shiftKey ? -1 : 1 };
  }
  if (!hasMod(e)) return null;

  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  if (e.shiftKey) {
    switch (key) {
      case 'p':
        return { type: 'palette' };
      case 't':
        return { type: 'newSession' };
      case 'w':
        return { type: 'closeTab' };
      case 'l':
        return { type: 'lock' };
      default:
        return null;
    }
  }
  if (key === 'k') return isTerminalTarget(e.target) ? null : { type: 'palette' };
  if (key === ',') return { type: 'settings' };
  // e.code, not e.key: layouts that need Shift for digits (AZERTY) still match.
  const digit = /^Digit([1-9])$/.exec(e.code);
  if (digit) {
    const n = Number(digit[1]);
    return { type: 'tab', index: n === 9 ? -1 : n - 1 };
  }
  return null;
}

/** Keys the terminal pane handles itself (they only make sense with a terminal focused). */
export type TerminalKeyAction = 'copy' | 'paste' | 'fontUp' | 'fontDown' | 'fontReset';

export function terminalKeyAction(e: KeyboardEvent, hasSelection: boolean): TerminalKeyAction | null {
  if (e.type !== 'keydown' || e.isComposing || e.altKey || !hasMod(e)) return null;
  const key = e.key.toLowerCase();
  // Ctrl+C copies only when something is selected; otherwise it stays the shell's interrupt.
  if (key === 'c' && (e.shiftKey || hasSelection)) return 'copy';
  if (key === 'v') return 'paste';
  if (key === '=' || key === '+') return 'fontUp';
  if (key === '-' || key === '_') return 'fontDown';
  if (key === '0' && !e.shiftKey) return 'fontReset';
  return null;
}

// ---- What Settings → Shortcuts shows ---------------------------------------------------------

export interface ShortcutDoc {
  /** i18n key under `settings.shortcuts.` */
  label: string;
  keys: string[][]; // alternatives, each a list of key names
  vars?: Record<string, string | number>;
}

export interface ShortcutGroup {
  /** i18n key under `settings.shortcuts.` */
  title: string;
  items: ShortcutDoc[];
}

const MOD = IS_MAC ? '⌘' : 'Ctrl';

export const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: 'global',
    items: [
      { label: 'commandPalette', keys: [[MOD, 'K'], [MOD, 'Shift', 'P']] },
      { label: 'openSettings', keys: [[MOD, ',']] },
      { label: 'lockVault', keys: [[MOD, 'Shift', 'L']] }
    ]
  },
  {
    title: 'sessions',
    items: [
      { label: 'newSession', keys: [[MOD, 'Shift', 'T']] },
      { label: 'switchToTabRange', keys: [[MOD, '1…8']] },
      { label: 'lastTab', keys: [[MOD, '9']] },
      { label: 'nextTab', keys: [['Ctrl', 'Tab']] },
      { label: 'previousTab', keys: [['Ctrl', 'Shift', 'Tab']] },
      { label: 'closeTab', keys: [[MOD, 'Shift', 'W']] }
    ]
  },
  {
    title: 'terminal',
    items: [
      { label: 'copySelection', keys: [[MOD, 'Shift', 'C'], [MOD, 'C']] },
      { label: 'paste', keys: [[MOD, 'V'], [MOD, 'Shift', 'V']] },
      { label: 'fontBigger', keys: [[MOD, '+']] },
      { label: 'fontSmaller', keys: [[MOD, '−']] },
      { label: 'fontReset', keys: [[MOD, '0']] },
      { label: 'autocompleteAccept', keys: [['Tab']] }
    ]
  },
  {
    title: 'sftp',
    items: [
      { label: 'sftpCopy', keys: [['F5']] },
      { label: 'sftpNewFolder', keys: [['F7']] },
      { label: 'sftpDelete', keys: [['F8'], ['Delete']] },
      { label: 'sftpRename', keys: [['F2']] },
      { label: 'sftpEdit', keys: [['F4']] }
    ]
  }
];
