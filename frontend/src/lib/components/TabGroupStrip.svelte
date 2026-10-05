<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import {
    getTabGroups,
    getActiveGroupId,
    setActiveGroupId,
    closePaneInGroup,
    type TabGroup
  } from '$lib/stores/dualSplitStore.svelte';

  interface Props {
    onNewGroup?: () => void;
  }

  let { onNewGroup }: Props = $props();

  const tabGroups = $derived(getTabGroups());
  const activeGroupId = $derived(getActiveGroupId());

  function selectGroup(id: string) {
    setActiveGroupId(id);
  }
</script>

<div class="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-none select-none">
  {#each tabGroups as group (group.id)}
    {@const isActive = group.id === activeGroupId}
    <div
      class="group flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border text-xs transition-all cursor-pointer {isActive ? 'bg-neutral-800/90 border-indigo-500/50 text-white shadow-sm' : 'bg-neutral-900/60 border-neutral-800 text-neutral-400 hover:text-neutral-200 hover:border-neutral-700'}"
      onclick={() => selectGroup(group.id)}
      role="button"
      tabindex="0"
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') selectGroup(group.id); }}
    >
      <span
        class="w-2 h-2 rounded-full shrink-0"
        style="background-color: {group.color || '#6366f1'}"
      ></span>

      <span class="font-medium truncate max-w-[120px]">{group.name}</span>

      {#if group.panes.length > 1}
        <span class="px-1.5 py-0.2 rounded text-[10px] font-mono bg-neutral-700/60 text-neutral-300">
          {group.panes.length} {group.splitDirection === 'horizontal' ? 'H' : 'V'}
        </span>
      {/if}

      <button
        type="button"
        class="opacity-0 group-hover:opacity-100 p-0.5 rounded text-neutral-500 hover:text-rose-400 hover:bg-neutral-700/50 transition-opacity"
        title={t('dualSplit.closeGroup')}
        aria-label={t('dualSplit.closeGroup')}
        onclick={(e) => {
          e.stopPropagation();
          for (const pane of [...group.panes]) {
            closePaneInGroup(group.id, pane.id);
          }
        }}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/each}

  {#if onNewGroup}
    <button
      type="button"
      onclick={onNewGroup}
      class="flex items-center justify-center p-1.5 rounded-lg border border-dashed border-neutral-700 text-neutral-400 hover:text-white hover:border-neutral-500 transition-colors"
      title={t('dualSplit.addGroup')}
      aria-label={t('dualSplit.addGroup')}
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
    </button>
  {/if}
</div>
