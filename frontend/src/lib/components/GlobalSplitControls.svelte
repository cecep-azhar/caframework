<script lang="ts">
  import {
    getGlobalLayout,
    setGlobalLayout,
    type GlobalSplitLayout
  } from '$lib/stores/dualSplitStore.svelte';
  import { t } from '$lib/i18n/index.svelte';

  interface Props {
    groupCount?: number;
  }

  let { groupCount = 1 }: Props = $props();

  const currentLayout = $derived(getGlobalLayout());

  const options = $derived<Array<{ value: GlobalSplitLayout; label: string; minGroups: number; icon: string }>>([
    { value: 'single', label: t('dualSplit.single'), minGroups: 1, icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z' },
    { value: '2-columns', label: t('dualSplit.twoColumns'), minGroups: 2, icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M12 4v16' },
    { value: '2-rows', label: t('dualSplit.twoRows'), minGroups: 2, icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M4 12h16' },
    { value: '2x2-grid', label: t('dualSplit.grid'), minGroups: 3, icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M12 4v16 M4 12h16' }
  ]);
</script>

<div class="flex items-center gap-1 p-1 bg-neutral-900 border border-neutral-800 rounded-lg select-none">
  {#each options as opt}
    <button
      type="button"
      disabled={groupCount < opt.minGroups}
      onclick={() => setGlobalLayout(opt.value)}
      class="px-2 py-1 flex items-center gap-1 rounded text-xs font-medium transition-colors disabled:opacity-30 {currentLayout === opt.value ? 'bg-indigo-600 text-white shadow-xs' : 'text-neutral-400 hover:text-white hover:bg-neutral-800'}"
      title={`${opt.label} ${t('dualSplit.viewSuffix')}`}
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={opt.icon} />
      </svg>
      <span class="text-[10px] hidden sm:inline">{opt.label}</span>
    </button>
  {/each}
</div>
