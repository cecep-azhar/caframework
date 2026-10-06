// Master Layout & Dual-Level Split Foundation for CAFramework products
// (CAGames, CAMark, CABook, CATable, CAProduct, CASuite, etc.)

export type GlobalSplitLayout = 'single' | '2-columns' | '2-rows' | '2x2-grid';
export type InPaneSplitDirection = 'horizontal' | 'vertical' | 'none';

export interface PaneItem<T = unknown> {
  id: string;
  title: string;
  data?: T;
  role?: 'primary' | 'secondary';
}

export interface TabGroup<T = unknown> {
  id: string;
  name: string;
  color?: string;
  splitDirection: InPaneSplitDirection;
  panes: PaneItem<T>[];
  activePaneId?: string;
}

export interface WorkspaceSnapshot<T = unknown> {
  id: string;
  name: string;
  globalLayout: GlobalSplitLayout;
  groups: TabGroup<T>[];
  activeGroupId?: string;
  createdAt: number;
  updatedAt: number;
}

export const GROUP_PALETTE = [
  '#0284c7', // Sky
  '#10b981', // Emerald
  '#8b5cf6', // Violet
  '#f59e0b', // Amber
  '#ec4899', // Pink
  '#06b6d4', // Cyan
  '#ef4444', // Red
  '#84cc16', // Lime
];

let globalLayoutState = $state<GlobalSplitLayout>('single');
let tabGroupsState = $state<TabGroup<any>[]>([]);
let activeGroupIdState = $state<string>('');

export function getGlobalLayout(): GlobalSplitLayout {
  return globalLayoutState;
}

export function setGlobalLayout(layout: GlobalSplitLayout): void {
  globalLayoutState = layout;
}

export function getTabGroups<T = unknown>(): TabGroup<T>[] {
  return tabGroupsState as TabGroup<T>[];
}

export function getActiveGroupId(): string {
  return activeGroupIdState;
}

export function setActiveGroupId(id: string): void {
  activeGroupIdState = id;
}

export function getActiveGroup<T = unknown>(): TabGroup<T> | undefined {
  return (tabGroupsState.find((g) => g.id === activeGroupIdState) ?? tabGroupsState[0]) as TabGroup<T> | undefined;
}

export function createTabGroup<T = unknown>(name: string, initialPane: PaneItem<T>, color?: string): TabGroup<T> {
  const id = `group-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
  const group: TabGroup<T> = {
    id,
    name: name.trim() || 'Untitled Group',
    color: color || GROUP_PALETTE[tabGroupsState.length % GROUP_PALETTE.length],
    splitDirection: 'none',
    panes: [{ ...initialPane, role: 'primary' }],
    activePaneId: initialPane.id
  };
  tabGroupsState = [...tabGroupsState, group];
  if (!activeGroupIdState) {
    activeGroupIdState = id;
  }
  return group;
}

export function splitPaneInGroup<T = unknown>(
  groupId: string,
  newPane: PaneItem<T>,
  direction: 'horizontal' | 'vertical'
): boolean {
  const group = tabGroupsState.find((g) => g.id === groupId);
  if (!group) return false;

  const paneToAdd: PaneItem<T> = {
    ...newPane,
    role: group.panes.length === 0 ? 'primary' : 'secondary'
  };
  const updatedGroup: TabGroup<any> = {
    ...group,
    splitDirection: direction,
    panes: [...group.panes, paneToAdd],
    activePaneId: paneToAdd.id
  };
  tabGroupsState = tabGroupsState.map((g) => (g.id === groupId ? updatedGroup : g));
  return true;
}

export function closePaneInGroup(groupId: string, paneId: string): void {
  const group = tabGroupsState.find((g) => g.id === groupId);
  if (!group) return;

  const remainingPanes = group.panes.filter((p) => p.id !== paneId);

  if (remainingPanes.length === 0) {
    tabGroupsState = tabGroupsState.filter((g) => g.id !== groupId);
    if (activeGroupIdState === groupId) {
      activeGroupIdState = tabGroupsState[0]?.id ?? '';
    }
  } else {
    const updatedGroup: TabGroup<any> = {
      ...group,
      panes: remainingPanes,
      splitDirection: remainingPanes.length <= 1 ? 'none' : group.splitDirection,
      activePaneId: group.activePaneId === paneId ? remainingPanes[0].id : group.activePaneId
    };
    tabGroupsState = tabGroupsState.map((g) => (g.id === groupId ? updatedGroup : g));
  }
}

export function renameTabGroup(groupId: string, name: string): void {
  tabGroupsState = tabGroupsState.map((g) =>
    g.id === groupId ? { ...g, name: name.trim() || 'Untitled Group' } : g
  );
}

export function setTabGroupColor(groupId: string, color: string): void {
  tabGroupsState = tabGroupsState.map((g) =>
    g.id === groupId ? { ...g, color } : g
  );
}

export function resetDualSplitStore(): void {
  globalLayoutState = 'single';
  tabGroupsState = [];
  activeGroupIdState = '';
}

export function createWorkspaceSnapshot<T = unknown>(name: string, id?: string): WorkspaceSnapshot<T> {
  const now = Date.now();
  return {
    id: id || `ws-${now}-${Math.random().toString(36).substring(2, 7)}`,
    name: name.trim() || 'Default Workspace',
    globalLayout: globalLayoutState,
    groups: JSON.parse(JSON.stringify(tabGroupsState)),
    activeGroupId: activeGroupIdState,
    createdAt: now,
    updatedAt: now
  };
}

export function restoreWorkspaceSnapshot<T = unknown>(snapshot: WorkspaceSnapshot<T>): void {
  globalLayoutState = snapshot.globalLayout || 'single';
  tabGroupsState = JSON.parse(JSON.stringify(snapshot.groups || []));
  activeGroupIdState = snapshot.activeGroupId || tabGroupsState[0]?.id || '';
}
