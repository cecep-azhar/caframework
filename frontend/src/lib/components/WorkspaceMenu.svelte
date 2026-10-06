<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';

  interface WorkspaceItem {
    id: string;
    name: string;
    description: string;
    color: string;
  }

  let workspaces = $state<WorkspaceItem[]>([
    { id: 'ws-default', name: 'Default Workspace', description: 'Primary working area', color: '#10b981' },
    { id: 'ws-research', name: 'Research & Labs', description: 'Experimental sandbox', color: '#06b6d4' },
    { id: 'ws-prod', name: 'Production Fleet', description: 'Critical deployments', color: '#8b5cf6' }
  ]);

  let activeId = $state('ws-default');
  let open = $state(false);
  let root: HTMLDivElement | undefined = $state();

  const activeWorkspace = $derived(
    workspaces.find(w => w.id === activeId) || workspaces[0]
  );

  function handleOutside(e: PointerEvent) {
    if (open && root && !root.contains(e.target as Node)) {
      open = false;
    }
  }

  function selectWorkspace(w: WorkspaceItem) {
    activeId = w.id;
    open = false;
    showToast(`Beralih ke ruang kerja: ${w.name}`, 'info');
  }
</script>

<svelte:window
  onpointerdown={handleOutside}
  onkeydown={(e) => {
    if (e.key === 'Escape') open = false;
  }}
/>

<div class="relative inline-flex items-center no-drag" bind:this={root}>
  <!-- Icon-only Workspace Trigger Button (Matched with CATerm) -->
  <button
    type="button"
    onclick={() => (open = !open)}
    class="p-1.5 rounded-lg border transition-colors flex items-center justify-center {open ? 'bg-sky-500/15 border-sky-500/40 text-sky-600 dark:text-sky-400' : 'bg-neutral-50 dark:bg-neutral-900 border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800/60'}"
    title={`Workspaces: ${activeWorkspace.name}`}
    aria-label={`Workspaces: ${activeWorkspace.name}`}
  >
    <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
    </svg>
  </button>

  {#if open}
    <div
      role="menu"
      aria-label="Workspaces Menu"
      class="absolute right-0 top-full mt-2 z-50 w-64 rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-2 space-y-1 text-xs animate-in fade-in zoom-in-95 duration-100"
    >
      <div class="px-2.5 py-1.5 border-b border-neutral-100 dark:border-neutral-800 flex items-center justify-between">
        <span class="font-bold text-neutral-900 dark:text-white">Pilih Ruang Kerja</span>
        <span class="text-[10px] text-neutral-400 font-mono">{workspaces.length} Ruang</span>
      </div>

      <div class="space-y-1 py-1">
        {#each workspaces as ws (ws.id)}
          <button
            type="button"
            role="menuitem"
            onclick={() => selectWorkspace(ws)}
            class="w-full flex items-center gap-2.5 p-2 rounded-xl text-left transition-colors {activeId === ws.id ? 'bg-sky-500/10 border border-sky-500/30 text-sky-700 dark:text-sky-300' : 'hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-700 dark:text-neutral-300 border border-transparent'}"
          >
            <span class="w-2.5 h-2.5 rounded-full shrink-0 shadow-xs" style="background-color: {ws.color}"></span>
            <div class="min-w-0 flex-1">
              <span class="block font-semibold truncate text-neutral-900 dark:text-white">{ws.name}</span>
              <span class="block text-[10px] text-neutral-500 truncate">{ws.description}</span>
            </div>
            {#if activeId === ws.id}
              <svg class="w-4 h-4 text-sky-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
              </svg>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
