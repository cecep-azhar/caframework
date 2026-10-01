// Workspace persistence store for CAFramework.
// Saves and restores multi-host session layouts and connection states.

import { listHosts, type HostRecord } from '$lib/api/hosts';
import { openTab, closeAllTabs, LOCAL_HOST_ID, localTerminalHost } from '$lib/stores/sessionTabs.svelte';
import {
  getSessionView,
  setLayout,
  setShowFiles,
  isPaneLayout
} from '$lib/stores/sessionView.svelte';

export interface Workspace {
  id: string;
  name: string;
  hostIds: string[];
  layout: number; // 1=single, 2=split horizontal, 3=split vertical, 4=grid 2x2
  showFiles: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface WorkspaceInput {
  name: string;
  hostIds: string[];
  layout?: number;
  showFiles?: boolean;
  id?: string;
}

const STORAGE_KEY = 'caframework_workspaces_v2';

/**
 * Workspaces saved before this fix stored session *tab* ids (`tab-<hostId>-<n>`) instead of
 * host ids, so restoring matched nothing and opened 0 hosts. Recover the host id from them.
 */
function toHostId(id: string): string {
  return /^tab-(.+)-\d+$/.exec(id)?.[1] ?? id;
}

function loadInitialWorkspaces(): Workspace[] {
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return [];
  }
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      const migrated = (parsed as Workspace[]).map((ws) => ({
        ...ws,
        hostIds: (ws.hostIds ?? []).map(toHostId)
      }));
      if (JSON.stringify(migrated) !== raw) persistWorkspaces(migrated);
      return migrated;
    }
  } catch (e) {
    console.error('Failed to load workspaces from localStorage:', e);
  }
  return [];
}

function persistWorkspaces(items: Workspace[]): void {
  if (typeof window === 'undefined' || typeof localStorage === 'undefined') {
    return;
  }
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
  } catch (e) {
    console.error('Failed to save workspaces to localStorage:', e);
  }
}

let workspaces = $state<Workspace[]>(loadInitialWorkspaces());
// Layout / file-panel state is NOT owned here: `sessionView` is the single source of truth
// shared by the app header and the Session route. Keeping a second copy in this store meant a
// workspace restore and the header could disagree about the current layout.

export function getWorkspaces(): Workspace[] {
  return workspaces;
}

export function getWorkspace(id: string): Workspace | undefined {
  return workspaces.find((w) => w.id === id);
}

export function saveWorkspace(
  nameOrInput: string | WorkspaceInput,
  hostIds?: string[],
  layout: number = 1,
  showFiles: boolean = true,
  id?: string
): Workspace {
  let nameStr: string;
  let ids: string[];
  let effLayout = layout;
  let effShowFiles = showFiles;
  let targetId = id;

  if (typeof nameOrInput === 'object') {
    nameStr = nameOrInput.name;
    ids = nameOrInput.hostIds || [];
    effLayout = nameOrInput.layout ?? 1;
    effShowFiles = nameOrInput.showFiles ?? true;
    targetId = nameOrInput.id;
  } else {
    nameStr = nameOrInput;
    ids = hostIds || [];
  }

  const trimmedName = nameStr.trim() || 'Untitled Workspace';
  const now = Date.now();

  const existingIndex = targetId
    ? workspaces.findIndex((w) => w.id === targetId)
    : workspaces.findIndex((w) => w.name.toLowerCase() === trimmedName.toLowerCase());

  let ws: Workspace;
  if (existingIndex >= 0) {
    ws = {
      ...workspaces[existingIndex],
      name: trimmedName,
      hostIds: [...ids],
      layout: effLayout,
      showFiles: effShowFiles,
      updatedAt: now
    };
    const updated = [...workspaces];
    updated[existingIndex] = ws;
    workspaces = updated;
  } else {
    ws = {
      id: targetId || `ws-${now}-${Math.random().toString(36).substring(2, 8)}`,
      name: trimmedName,
      hostIds: [...ids],
      layout: effLayout,
      showFiles: effShowFiles,
      createdAt: now,
      updatedAt: now
    };
    workspaces = [ws, ...workspaces];
  }

  persistWorkspaces(workspaces);
  return ws;
}

export function deleteWorkspace(id: string): void {
  workspaces = workspaces.filter((w) => w.id !== id);
  persistWorkspaces(workspaces);
}

export function getCurrentLayout(): number {
  return getSessionView().layout;
}

export function setCurrentLayout(val: number): void {
  if (isPaneLayout(val)) setLayout(val);
}

export function getCurrentShowFiles(): boolean {
  return getSessionView().showFiles;
}

export function setCurrentShowFiles(val: boolean): void {
  setShowFiles(val);
}

/**
 * Restores a workspace: closes existing session tabs, opens tabs for all
 * hosts referenced by hostIds (if found in host database), and updates layout & file explorer states.
 */
export async function restoreWorkspace(workspace: Workspace): Promise<{ openedCount: number; missingCount: number }> {
  let allHosts: HostRecord[] = [];
  try {
    allHosts = await listHosts();
  } catch (err) {
    console.error('Failed to list hosts when restoring workspace:', err);
  }

  const resolved = workspace.hostIds
    .map((hostId) => (hostId === LOCAL_HOST_ID ? localTerminalHost() : allHosts.find((h) => h.id === hostId)))
    .filter((host): host is HostRecord => Boolean(host));
  const missingCount = workspace.hostIds.length - resolved.length;

  // Nothing to open: leave the sessions the user already has alone instead of closing them
  // for an empty result.
  if (resolved.length === 0) return { openedCount: 0, missingCount };

  closeAllTabs();
  for (const host of resolved) openTab(host);

  setCurrentLayout(workspace.layout || 1);
  // Restoring connects every host; like any new connection, Files stays closed until asked for.
  setShowFiles(false);

  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('caframework:workspace-loaded', { detail: workspace }));
  }

  return { openedCount: resolved.length, missingCount };
}
