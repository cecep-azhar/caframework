import { invoke } from '@tauri-apps/api/core';

export type ForwardType = 'local' | 'remote' | 'dynamic';

export interface TunnelRecord {
  id: string;
  hostId: string;
  forwardType: ForwardType;
  bindAddr: string;
  bindPort: number;
  targetAddr: string;
  targetPort: number;
  isActive: boolean;
}

export interface TunnelInput {
  hostId: string;
  forwardType: ForwardType;
  bindAddr?: string;
  bindPort: number;
  targetAddr: string;
  targetPort: number;
}

export function listTunnels(): Promise<TunnelRecord[]> {
  return invoke('list_tunnels');
}

export function saveTunnel(input: TunnelInput): Promise<TunnelRecord> {
  return invoke('save_tunnel', { input });
}

export function deleteTunnel(id: string): Promise<void> {
  return invoke('delete_tunnel', { id });
}

export function startTunnel(id: string): Promise<void> {
  return invoke('start_tunnel', { id });
}

export function stopTunnel(id: string): Promise<void> {
  return invoke('stop_tunnel', { id });
}