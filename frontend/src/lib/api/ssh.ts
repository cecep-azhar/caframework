// JS <-> Rust binding for SSH sessions. Backed by the real `ssh2` engine in
// crates/caf-core/src/ssh.rs — `sshConnect` only ever takes a saved host id, never raw
// connection details: the backend resolves the host record and its encrypted-at-rest
// credential itself (crate::store::load_host_for_connect) so plaintext never has to be
// re-sent over IPC on every connect.
//
// PTY output is *pushed* from Rust as `ssh://output` events (see `install_ssh_event_bridge` in
// crates/caf-app/src/lib.rs). Subscribe with `onSshOutput` before calling `sshConnect` so no
// byte of the shell banner can land before the listener exists.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface SshSession {
  sessionId: string;
  hostId: string;
}

export interface SshOutputEvent {
  sessionId: string;
  data: string;
}

export interface SshClosedEvent {
  sessionId: string;
}

export function sshConnect(hostId: string): Promise<SshSession> {
  return invoke('ssh_connect', { hostId });
}

export function sshWrite(sessionId: string, data: string): Promise<string> {
  return invoke('ssh_write', { sessionId, data });
}

/// Drains whatever the backend buffered. Only produces data when no event sink is installed
/// (CLI, tests) — inside the app, output arrives via `onSshOutput` instead.
export function sshRead(sessionId: string): Promise<string> {
  return invoke('ssh_read', { sessionId });
}

export function sshResize(sessionId: string, cols: number, rows: number): Promise<void> {
  return invoke('ssh_resize', { sessionId, cols, rows });
}

export function sshDisconnect(sessionId: string): Promise<void> {
  return invoke('ssh_disconnect', { sessionId });
}

/** Subscribe to PTY output for every session; filter by `sessionId` in the handler. */
export function onSshOutput(handler: (event: SshOutputEvent) => void): Promise<UnlistenFn> {
  return listen<SshOutputEvent>('ssh://output', (e) => handler(e.payload));
}

/** Fires once per session when its PTY channel reaches EOF (remote logout, dropped link). */
export function onSshClosed(handler: (event: SshClosedEvent) => void): Promise<UnlistenFn> {
  return listen<SshClosedEvent>('ssh://closed', (e) => handler(e.payload));
}
