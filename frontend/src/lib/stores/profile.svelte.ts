// Local profile shown in the sidebar, the profile menu and the lock screen. Free plan is fully
// local: no email, no account — just a display name and one of the preset avatars. Kept in
// localStorage (not the vault) on purpose: the lock screen has to greet the user by name
// *before* the vault is unlocked.
//
// The plan comes from CAFramework Pro (a separate email account, see stores/pro): a profile shows
// as Pro only while this device holds a valid signed licence. The name/avatar stay local.
import { getPro } from '$lib/stores/pro.svelte';

const STORAGE_KEY = 'caframework_profile_v1';
export const DEFAULT_PROFILE_NAME = 'CAFramework User';
export const DEFAULT_AVATAR = 'rocket';

export type Plan = 'free' | 'pro';

interface StoredProfile {
  name: string;
  avatar: string;
}

function load(): StoredProfile | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<StoredProfile>;
    if (typeof parsed.name !== 'string') return null;
    return { name: parsed.name, avatar: typeof parsed.avatar === 'string' ? parsed.avatar : DEFAULT_AVATAR };
  } catch {
    return null;
  }
}

const initial = typeof localStorage === 'undefined' ? null : load();
let name = $state(initial?.name || DEFAULT_PROFILE_NAME);
let avatar = $state(initial?.avatar || DEFAULT_AVATAR);

export function getProfile() {
  return {
    get name() {
      return name;
    },
    get avatar() {
      return avatar;
    },
    get plan(): Plan {
      // Pro only when this device holds a valid signed licence (see stores/pro).
      return getPro().isPro ? 'pro' : 'free';
    }
  };
}

export function saveProfile(next: { name: string; avatar: string }) {
  name = next.name.trim() || DEFAULT_PROFILE_NAME;
  avatar = next.avatar || DEFAULT_AVATAR;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ name, avatar }));
  } catch {
    // Storage blocked: the change still applies for this session.
  }
}

export function initialsOf(fullName: string): string {
  const parts = fullName.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return '?';
  const first = parts[0]?.[0] ?? '';
  const second = parts.length > 1 ? (parts[1]?.[0] ?? '') : '';
  return (first + second).toUpperCase();
}
