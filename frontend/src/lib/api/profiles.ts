import {
  cmdListProfiles,
  cmdSaveProfile,
  cmdVerifyPin,
  type ProfileRecord,
  type ProfileInput
} from '$lib/generated/commands';

export type { ProfileRecord, ProfileInput };

export async function listProfiles(): Promise<ProfileRecord[]> {
  return await cmdListProfiles();
}

export async function saveProfile(input: ProfileInput, callerProfileId: string): Promise<ProfileRecord> {
  return await cmdSaveProfile({ input, callerProfileId });
}

export async function verifyPin(profileId: string, pin: string): Promise<boolean> {
  return await cmdVerifyPin({ profileId, pin });
}
