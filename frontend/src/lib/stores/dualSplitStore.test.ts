import { describe, it, expect, beforeEach } from 'vitest';
import {
  getGlobalLayout,
  setGlobalLayout,
  getTabGroups,
  getActiveGroupId,
  getActiveGroup,
  setActiveGroupId,
  createTabGroup,
  splitPaneInGroup,
  closePaneInGroup,
  renameTabGroup,
  setTabGroupColor,
  createWorkspaceSnapshot,
  restoreWorkspaceSnapshot,
  resetDualSplitStore
} from './dualSplitStore.svelte';

describe('dualSplitStore', () => {
  beforeEach(() => {
    resetDualSplitStore();
  });

  it('manages global split layout state', () => {
    expect(getGlobalLayout()).toBe('single');
    setGlobalLayout('2-columns');
    expect(getGlobalLayout()).toBe('2-columns');
    setGlobalLayout('2x2-grid');
    expect(getGlobalLayout()).toBe('2x2-grid');
  });

  it('creates and manages tab groups with panes', () => {
    const initialGroup = createTabGroup('Backend Dev', { id: 'p1', title: 'Editor' });
    expect(getTabGroups().length).toBe(1);
    expect(getActiveGroupId()).toBe(initialGroup.id);
    expect(getTabGroups()[0].panes.length).toBe(1);
    expect(getTabGroups()[0].panes[0].role).toBe('primary');

    // Split in-pane horizontally
    const success = splitPaneInGroup(initialGroup.id, { id: 'p2', title: 'Logs' }, 'horizontal');
    expect(success).toBe(true);
    let currentGroup = getTabGroups().find((g) => g.id === initialGroup.id)!;
    expect(currentGroup.splitDirection).toBe('horizontal');
    expect(currentGroup.panes.length).toBe(2);
    expect(currentGroup.panes[1].role).toBe('secondary');
    expect(currentGroup.activePaneId).toBe('p2');

    // Rename & Color
    renameTabGroup(initialGroup.id, 'Main Service');
    currentGroup = getTabGroups().find((g) => g.id === initialGroup.id)!;
    expect(currentGroup.name).toBe('Main Service');
    setTabGroupColor(initialGroup.id, '#ef4444');
    currentGroup = getTabGroups().find((g) => g.id === initialGroup.id)!;
    expect(currentGroup.color).toBe('#ef4444');

    // Close secondary pane
    closePaneInGroup(initialGroup.id, 'p2');
    currentGroup = getTabGroups().find((g) => g.id === initialGroup.id)!;
    expect(currentGroup.panes.length).toBe(1);
    expect(currentGroup.splitDirection).toBe('none');

    // Close primary pane (removes group)
    closePaneInGroup(initialGroup.id, 'p1');
    expect(getTabGroups().length).toBe(0);
  });

  it('supports workspace snapshots and restores', () => {
    setGlobalLayout('2-columns');
    const groupA = createTabGroup('Group A', { id: 'pA1', title: 'Pane A1' }, '#10b981');
    splitPaneInGroup(groupA.id, { id: 'pA2', title: 'Pane A2' }, 'vertical');

    const groupB = createTabGroup('Group B', { id: 'pB1', title: 'Pane B1' }, '#8b5cf6');
    setActiveGroupId(groupB.id);

    const snapshot = createWorkspaceSnapshot('Prod Monitoring');
    expect(snapshot.name).toBe('Prod Monitoring');
    expect(snapshot.globalLayout).toBe('2-columns');
    expect(snapshot.groups.length).toBe(2);

    // Reset store
    resetDualSplitStore();
    expect(getTabGroups().length).toBe(0);
    expect(getGlobalLayout()).toBe('single');

    // Restore
    restoreWorkspaceSnapshot(snapshot);
    expect(getGlobalLayout()).toBe('2-columns');
    expect(getTabGroups().length).toBe(2);
    expect(getActiveGroupId()).toBe(groupB.id);
    expect(getTabGroups()[0].splitDirection).toBe('vertical');
  });
});
