// Tracks which connected TerminalPane is "active" so Snippets can inject a command into it
// (T7). A pane registers itself active on connect and whenever its terminal gets focus/click,
// and clears itself on unmount. This is intentionally page-agnostic: Session and any other
// route hosting a TerminalPane share the same active-session slot.

export interface ActiveSession {
  sessionId: string;
  label: string;
  inject: (command: string) => void;
}

let current = $state<ActiveSession | null>(null);

export function getActiveSession(): ActiveSession | null {
  return current;
}

export function setActiveSession(session: ActiveSession) {
  current = session;
}

export function clearActiveSession(sessionId: string) {
  if (current?.sessionId === sessionId) {
    current = null;
  }
}

export function hasActiveSession(): boolean {
  return current !== null;
}

/** Injects `command` into the active terminal session. Returns false if none is connected. */
export function injectIntoActiveSession(command: string): boolean {
  if (!current) return false;
  current.inject(command);
  return true;
}
