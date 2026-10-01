import { invoke } from '@tauri-apps/api/core';

export interface NoteRecord {
  id: string;
  title: string;
  content: string;
  tags: string[];
  rev: number;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
  origin_device_id: string;
  owner_profile_id: string;
  visibility: 'shared' | 'private_summary' | 'private';
}

export interface NoteInput {
  id?: string;
  title: string;
  content: string;
  tags: string[];
  visibility?: 'shared' | 'private_summary' | 'private';
  owner_profile_id: string;
}

export async function listNotes(callerProfileId: string, isOwner: boolean): Promise<NoteRecord[]> {
  return await invoke<NoteRecord[]>('list_notes', { callerProfileId, isOwner });
}

export async function saveNote(input: NoteInput, callerProfileId: string): Promise<NoteRecord> {
  return await invoke<NoteRecord>('save_note', { input, callerProfileId });
}

export async function deleteNote(id: string, callerProfileId: string): Promise<void> {
  await invoke('delete_note', { id, callerProfileId });
}
