import { invoke } from '@tauri-apps/api/core';

export interface TeamRecord {
  id: string;
  name: string;
  color: string;
  avatar: string | null;
  members: string[];
  hostIds: string[];
  groupIds: string[];
  createdAt: number;
  updatedAt: number;
}

export interface TeamInput {
  id?: string;
  name: string;
  color: string;
  avatar: string | null;
  members: string[];
  hostIds: string[];
  groupIds: string[];
}

export function listTeams(): Promise<TeamRecord[]> {
  return invoke('list_teams');
}

export function saveTeam(input: TeamInput): Promise<TeamRecord> {
  return invoke('save_team', { input });
}

export function deleteTeam(id: string): Promise<void> {
  return invoke('delete_team', { id });
}
