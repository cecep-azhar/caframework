<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface Props {
    title: string;
    badge?: string;
    onSplitRight?: () => void;
    onSplitDown?: () => void;
    onClose?: () => void;
    actions?: Snippet;
  }

  let {
    title,
    badge,
    onSplitRight,
    onSplitDown,
    onClose,
    actions
  }: Props = $props();
</script>

<div class="h-9 px-3 bg-neutral-900 border-b border-neutral-800 flex items-center justify-between shrink-0 select-none text-xs">
  <!-- Left info -->
  <div class="flex items-center gap-2 min-w-0">
    <span class="font-medium text-neutral-200 truncate">{title}</span>
    {#if badge}
      <span class="px-1.5 py-0.5 rounded text-[10px] font-mono bg-neutral-800 border border-neutral-700 text-neutral-400">
        {badge}
      </span>
    {/if}
  </div>

  <!-- Right Actions: custom actions -> split right/down -> close -->
  <div class="flex items-center gap-1">
    {#if actions}
      {@render actions()}
    {/if}

    {#if onSplitRight}
      <button
        type="button"
        onclick={onSplitRight}
        class="p-1 rounded text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors"
        title={t('dualSplit.splitRight')}
        aria-label={t('dualSplit.splitRight')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M12 4v16" />
        </svg>
      </button>
    {/if}

    {#if onSplitDown}
      <button
        type="button"
        onclick={onSplitDown}
        class="p-1 rounded text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors"
        title={t('dualSplit.splitDown')}
        aria-label={t('dualSplit.splitDown')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M4 12h16" />
        </svg>
      </button>
    {/if}

    {#if onClose}
      <button
        type="button"
        onclick={onClose}
        class="p-1 rounded text-neutral-400 hover:text-rose-400 hover:bg-neutral-800 transition-colors"
        title={t('dualSplit.closePane')}
        aria-label={t('dualSplit.closePane')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    {/if}
  </div>
</div>
