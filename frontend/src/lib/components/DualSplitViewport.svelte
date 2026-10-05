<script lang="ts">
  import type { Snippet } from 'svelte';
  import {
    getGlobalLayout,
    getTabGroups,
    getActiveGroup,
    type GlobalSplitLayout,
    type TabGroup
  } from '$lib/stores/dualSplitStore.svelte';

  interface Props {
    renderPane: Snippet<[{ item: any; group: TabGroup<any> }]>;
  }

  let { renderPane }: Props = $props();

  const globalLayout = $derived(getGlobalLayout());
  const tabGroups = $derived(getTabGroups());
  const activeGroup = $derived(getActiveGroup());

  // Macro layout styling
  const macroContainerClass = $derived(
    globalLayout === '2-rows'
      ? 'flex flex-col h-full gap-2'
      : globalLayout === '2-columns'
        ? 'flex h-full gap-2'
        : globalLayout === '2x2-grid'
          ? 'grid grid-cols-2 auto-rows-fr h-full gap-2'
          : 'relative h-full w-full'
  );

  function groupInPaneClass(group: TabGroup<any>): string {
    if (globalLayout !== 'single') return 'min-h-0 min-w-0 flex-1';
    return group.id === activeGroup?.id
      ? 'absolute inset-0 z-10'
      : 'absolute inset-0 invisible pointer-events-none';
  }

  function getMicroSplitClass(group: TabGroup<any>): string {
    if (group.panes.length <= 1 || group.splitDirection === 'none') {
      return 'h-full w-full';
    }
    return group.splitDirection === 'horizontal'
      ? 'flex flex-col h-full gap-1'
      : 'flex h-full gap-1';
  }
</script>

<div class="h-full w-full overflow-hidden">
  {#if tabGroups.length === 0}
    <div class="h-full flex items-center justify-center border border-dashed border-neutral-800 rounded-xl text-neutral-500 text-sm">
      No active tab groups
    </div>
  {:else}
    <div class={macroContainerClass}>
      {#each tabGroups as group (group.id)}
        <div class="{groupInPaneClass(group)} flex flex-col rounded-lg border border-neutral-800 overflow-hidden bg-neutral-900/50">
          <div class="flex-1 min-h-0 min-w-0 {getMicroSplitClass(group)}">
            {#each group.panes as pane (pane.id)}
              <div class="flex-1 min-h-0 min-w-0 h-full w-full overflow-hidden flex flex-col">
                {@render renderPane({ item: pane, group })}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
