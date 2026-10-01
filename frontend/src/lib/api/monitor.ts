import { invoke } from '@tauri-apps/api/core';

export interface HostMetrics {
  host_id: string;
  hostname: string;
  os_name: string;
  uptime: string;
  cpu_usage: number;
  mem_total_mb: number;
  mem_used_mb: number;
  disk_total_gb: number;
  disk_used_gb: number;
}

export function pollActiveMetrics(): Promise<HostMetrics[]> {
  return invoke('poll_active_metrics');
}