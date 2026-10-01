// JS <-> Rust binding for group persistence. Mirrors `caf_core::groups` 1:1 — see
// crates/caf-core/src/groups.rs for the source of truth on shapes and error codes.
// `hostIds` is the Hosts<->Groups link: the backend rejects any id that isn't a real host.
import { invoke } from '@tauri-apps/api/core';

export interface GroupRecord {
  id: string;
  name: string;
  color: string;
  hostIds: string[];
  createdAt: number;
  updatedAt: number;
}

export interface GroupInput {
  id?: string;
  name: string;
  color: string;
  hostIds: string[];
}

export function listGroups(): Promise<GroupRecord[]> {
  return invoke('list_groups');
}

export function saveGroup(input: GroupInput): Promise<GroupRecord> {
  return invoke('save_group', { input });
}

export function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id });
}
