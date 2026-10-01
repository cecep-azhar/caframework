import { invoke } from '@tauri-apps/api/core';

export interface ProfileRecord {
  id: string;
  name: string;
  role: 'owner' | 'partner' | 'member' | 'child';
  pin_hash?: string;
  avatar_url?: string;
  rev: number;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
  origin_device_id: string;
}

export interface ProfileInput {
  id?: string;
  name: string;
  role: 'owner' | 'partner' | 'member' | 'child';
  pin?: string;
  avatar_url?: string;
}

export async function listProfiles(): Promise<ProfileRecord[]> {
  return await invoke<ProfileRecord[]>('list_profiles');
}

export async function saveProfile(input: ProfileInput, callerProfileId: string): Promise<ProfileRecord> {
  return await invoke<ProfileRecord>('save_profile', { input, callerProfileId });
}

export async function verifyPin(profileId: string, pin: string): Promise<boolean> {
  return await invoke<boolean>('verify_pin', { profileId, pin });
}
