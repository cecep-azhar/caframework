<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    getWorkspaces,
    saveWorkspace,
    deleteWorkspace,
    restoreWorkspace,
    getCurrentLayout,
    getCurrentShowFiles,
    type Workspace
  } from '$lib/stores/workspaceStore.svelte';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t, intlLocale } from '$lib/i18n/index.svelte';

  interface Props {
    hostIds?: string[];
    layout?: number;
    showFiles?: boolean;
    onLoad?: (ws: Workspace) => void;
    onLoadWorkspace?: (ws: Workspace) => void;
    onSave?: (ws: Workspace) => void;
    onSaveWorkspace?: (ws: Workspace) => void;
    class?: string;
  }

  let {
    hostIds,
    layout,
    showFiles,
    onLoad,
    onLoadWorkspace,
    onSave,
    onSaveWorkspace,
    class: className = ''
  }: Props = $props();

  let isOpen = $state(false);
  let isSaveModalOpen = $state(false);
  let workspaceName = $state('');
  let isSaving = $state(false);
  let isLoadingWs = $state(false);
  let allHosts = $state<HostRecord[]>([]);

  let dropdownRef: HTMLDivElement | null = $state(null);

  const workspaces = $derived(getWorkspaces());
  const sessionTabs = $derived(getTabs());

  // Host ids, not tab ids: tab ids are per-session (`tab-<hostId>-<n>`) and never match a host
  // on restore. A host open twice is listed twice so both sessions come back.
  const activeHostIds = $derived(
    hostIds && hostIds.length > 0
      ? hostIds
      : sessionTabs.map((tab) => tab.host.id)
  );

  const effectiveLayout = $derived(
    layout !== undefined ? layout : getCurrentLayout()
  );

  const effectiveShowFiles = $derived(
    showFiles !== undefined ? showFiles : getCurrentShowFiles()
  );

  onMount(() => {
    refreshHosts();

    function handleOutsideClick(event: MouseEvent) {
      if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
        isOpen = false;
      }
    }

    window.addEventListener('click', handleOutsideClick);
    return () => {
      window.removeEventListener('click', handleOutsideClick);
    };
  });

  async function refreshHosts() {
    try {
      allHosts = await listHosts();
    } catch {
      // Backend may not be connected in tests
    }
  }

  function getLayoutName(l: number): string {
    switch (l) {
      case 2:
        return t('workspaces.layoutSplitH');
      case 3:
        return t('workspaces.layoutSplitV');
      case 4:
        return t('workspaces.layoutGrid');
      default:
        return t('workspaces.layoutSingle');
    }
  }

  function formatTimeAgo(ts: number): string {
    if (!ts) return '';
    const sec = Math.floor((Date.now() - ts) / 1000);
    if (sec < 60) return t('time.justNow');
    const min = Math.floor(sec / 60);
    if (min < 60) return t('time.minutesAgo', { n: min });
    const hrs = Math.floor(min / 60);
    if (hrs < 24) return t('time.hoursAgo', { n: hrs });
    const days = Math.floor(hrs / 24);
    if (days < 7) return t('time.daysAgo', { n: days });
    return new Date(ts).toLocaleDateString(intlLocale());
  }

  function getHostLabels(ids: string[]): string[] {
    return ids.map((id) => {
      const h = allHosts.find((item) => item.id === id);
      return h ? h.label : id;
    });
  }

  function openSaveModal() {
    isOpen = false;
    workspaceName = '';
    isSaveModalOpen = true;
  }

  function closeSaveModal() {
    isSaveModalOpen = false;
    workspaceName = '';
  }

  async function handleSaveWorkspace() {
    const trimmed = workspaceName.trim();
    if (!trimmed) {
      showToast(t('workspaces.errName'), 'error');
      return;
    }

    if (activeHostIds.length === 0) {
      showToast(t('workspaces.errNoHosts'), 'error');
      return;
    }

    try {
      isSaving = true;
      const saved = saveWorkspace(
        trimmed,
        activeHostIds,
        effectiveLayout,
        effectiveShowFiles
      );

      closeSaveModal();
      showToast(t('workspaces.saved', { name: saved.name }), 'success');

      if (onSave) onSave(saved);
      if (onSaveWorkspace) onSaveWorkspace(saved);
    } catch (err: any) {
      showToast(err?.message || t('workspaces.saveFailed'), 'error');
    } finally {
      isSaving = false;
    }
  }

  async function handleLoadWorkspace(ws: Workspace) {
    isOpen = false;
    isLoadingWs = true;
    try {
      const result = await restoreWorkspace(ws);
      if (result.openedCount === 0) {
        showToast(t('workspaces.notLoaded', { name: ws.name }), 'error');
        return;
      }
      if (result.missingCount > 0) {
        showToast(
          t('workspaces.partial', { name: ws.name, opened: result.openedCount, total: ws.hostIds.length, missing: result.missingCount }),
          'info'
        );
      } else {
        showToast(t('workspaces.loaded', { name: ws.name, count: result.openedCount }), 'success');
      }

      if (onLoad) onLoad(ws);
      if (onLoadWorkspace) onLoadWorkspace(ws);

      // Client-side navigation only. `window.location.href` reloaded the whole app, which
      // reset the layout's `isUnlocked` state and dumped the user back on the lock screen —
      // loading a workspace looked exactly like being logged out.
      if (typeof window !== 'undefined' && window.location.pathname !== '/session') {
        await goto('/session');
      }
    } catch (err: any) {
      showToast(err?.message || t('workspaces.loadFailed'), 'error');
    } finally {
      isLoadingWs = false;
    }
  }

  async function handleDeleteWorkspace(ws: Workspace) {
    const confirmed = await confirmModal(
      t('workspaces.deleteConfirm', { name: ws.name }),
      t('workspaces.deleteHeading'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (confirmed) {
      deleteWorkspace(ws.id);
      showToast(t('workspaces.deleted', { name: ws.name }), 'info');
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isOpen = false;
      isSaveModalOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="relative inline-block text-left {className}" bind:this={dropdownRef}>
  <!-- Sleek "Workspaces" Trigger Button -->
  <button
    type="button"
    onclick={(e) => {
      e.stopPropagation();
      isOpen = !isOpen;
    }}
    class="px-2.5 py-1 rounded text-xs font-medium border transition-colors flex items-center gap-1.5 {isOpen
      ? 'bg-sky-600/15 text-sky-600 dark:text-sky-400 border-sky-500/30'
      : 'bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 border-neutral-300 dark:border-neutral-800 hover:text-neutral-900 dark:hover:text-white hover:border-neutral-400 dark:hover:border-neutral-700'}"
    title={t('workspaces.manage')}
    aria-expanded={isOpen}
    aria-haspopup="true"
  >
    <!-- Bookmark / Folder Icon -->
    <svg class="w-3.5 h-3.5 text-sky-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
      />
    </svg>
    <span class="font-medium hidden sm:inline">{t('workspaces.title')}</span>
    {#if workspaces.length > 0}
      <span
        class="ml-0.5 px-1.5 py-0.2 rounded-full text-[10px] font-mono font-semibold bg-sky-500/15 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 border border-sky-500/30"
      >
        {workspaces.length}
      </span>
    {/if}
    <svg
      class="w-3 h-3 text-neutral-400 transition-transform duration-150"
      class:rotate-180={isOpen}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  <!-- Dropdown Menu -->
  {#if isOpen}
    <div
      class="fixed sm:absolute right-4 sm:right-0 top-14 sm:top-full mt-1.5 z-50 w-[calc(100vw-2rem)] sm:w-84 max-w-sm bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl shadow-xl overflow-hidden animate-in fade-in zoom-in-95 duration-100 select-none"
      role="dialog"
      aria-modal="true"
    >
      <!-- Dropdown Header -->
      <div
        class="px-3.5 py-2.5 border-b border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-950 flex items-center justify-between"
      >
        <div class="flex items-center gap-1.5">
          <svg class="w-4 h-4 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
            />
          </svg>
          <span class="text-xs font-semibold text-neutral-900 dark:text-white">{t('workspaces.title')}</span>
        </div>
        <button
          type="button"
          onclick={openSaveModal}
          class="px-2 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded-md text-xs font-medium transition-colors flex items-center gap-1 shadow-xs"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          <span>{t('workspaces.saveCurrent')}</span>
        </button>
      </div>

      <!-- Workspaces List -->
      <div class="max-h-64 overflow-y-auto divide-y divide-neutral-100 dark:divide-neutral-800/60 p-1">
        {#if workspaces.length === 0}
          <div class="py-8 px-4 text-center text-xs text-neutral-500 dark:text-neutral-400 space-y-2">
            <div class="w-8 h-8 mx-auto rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center text-neutral-400 dark:text-neutral-500">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
            </div>
            <p class="font-medium text-neutral-800 dark:text-neutral-200">{t('workspaces.emptyTitle')}</p>
            <p class="text-[11px] text-neutral-400 dark:text-neutral-500 max-w-[220px] mx-auto">
              {t('workspaces.emptyBody')}
            </p>
          </div>
        {:else}
          {#each workspaces as ws (ws.id)}
            <div
              class="p-2.5 flex items-center justify-between hover:bg-neutral-100/70 dark:hover:bg-neutral-800/50 rounded-lg transition-colors group"
            >
              <div class="min-w-0 flex-1 pr-2">
                <div class="text-xs font-semibold text-neutral-900 dark:text-white truncate group-hover:text-sky-600 dark:group-hover:text-sky-400 transition-colors">
                  {ws.name}
                </div>
                <div class="text-[10px] text-neutral-500 dark:text-neutral-400 flex items-center flex-wrap gap-1.5 mt-0.5">
                  <span class="px-1.5 py-0.2 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 font-mono">
                    {t('workspaces.hostCount', { count: ws.hostIds.length })}
                  </span>
                  <span>•</span>
                  <span>{getLayoutName(ws.layout)}</span>
                  {#if ws.updatedAt}
                    <span class="text-neutral-400 dark:text-neutral-500">• {formatTimeAgo(ws.updatedAt)}</span>
                  {/if}
                </div>
              </div>
              <div class="flex items-center gap-1 shrink-0">
                <button
                  type="button"
                  onclick={() => handleLoadWorkspace(ws)}
                  disabled={isLoadingWs}
                  class="px-2 py-1 bg-sky-600/10 hover:bg-sky-600 text-sky-600 hover:text-white dark:bg-sky-500/20 dark:text-sky-300 dark:hover:bg-sky-600 dark:hover:text-white border border-sky-500/30 rounded text-xs font-medium transition-colors shadow-xs"
                  title={t('workspaces.loadTitle', { name: ws.name })}
                >
                  {t('workspaces.load')}
                </button>
                <button
                  type="button"
                  onclick={() => handleDeleteWorkspace(ws)}
                  class="p-1 text-neutral-400 hover:text-rose-500 hover:bg-rose-500/10 rounded transition-colors"
                  title={t('workspaces.deleteTitle', { name: ws.name })}
                  aria-label={t('workspaces.deleteTitle', { name: ws.name })}
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                    />
                  </svg>
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- "Save Workspace" Modal / Form -->
{#if isSaveModalOpen}
  <div
    class="fixed inset-0 z-[99999] bg-black/40 dark:bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in duration-150"
  >
    <div
      class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 max-w-md w-full shadow-2xl space-y-4 animate-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
      aria-labelledby="save-workspace-title"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-sky-500/10 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 flex items-center justify-center">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
            </svg>
          </div>
          <div>
            <h3 id="save-workspace-title" class="font-bold text-neutral-900 dark:text-white text-base">{t('workspaces.saveTitle')}</h3>
            <p class="text-[11px] text-neutral-500 dark:text-neutral-400">{t('workspaces.saveSubtitle')}</p>
          </div>
        </div>
        <button
          type="button"
          onclick={closeSaveModal}
          class="p-1 rounded-lg text-neutral-400 hover:text-neutral-600 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          aria-label={t('workspaces.closeDialog')}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Form Section -->
      <div class="space-y-3 pt-1">
        <div>
          <label for="workspace-name-input" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 mb-1.5">
            {t('workspaces.nameLabel')}
          </label>
          <input
            id="workspace-name-input"
            type="text"
            bind:value={workspaceName}
            onkeydown={(e) => e.key === 'Enter' && handleSaveWorkspace()}
            placeholder={t('workspaces.namePlaceholder')}
            class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500 transition-colors"
          />
        </div>

        <!-- Current Workspace Summary Card -->
        <div class="p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800/80 rounded-xl space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-neutral-500 dark:text-neutral-400">{t('workspaces.activeHosts')}</span>
            <span class="px-2 py-0.5 rounded-full text-xs font-semibold bg-sky-500/10 dark:bg-sky-500/20 text-sky-600 dark:text-sky-400 border border-sky-500/30">
              {t('workspaces.hostCount', { count: activeHostIds.length })}
            </span>
          </div>

          {#if activeHostIds.length > 0}
            <div class="flex flex-wrap gap-1 pt-1">
              {#each getHostLabels(activeHostIds) as label}
                <span class="px-2 py-0.5 rounded bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-[11px] text-neutral-800 dark:text-neutral-200 font-mono">
                  {label}
                </span>
              {/each}
            </div>
          {:else}
            <div class="p-2 rounded bg-amber-500/10 border border-amber-500/20 text-amber-600 dark:text-amber-400 text-xs">
              {t('workspaces.noActiveHosts')}
            </div>
          {/if}

          <div class="grid grid-cols-2 gap-2 pt-1 border-t border-neutral-200 dark:border-neutral-800/60 text-[11px]">
            <div>
              <span class="text-neutral-500 dark:text-neutral-400">{t('workspaces.layout')}</span>
              <span class="font-medium text-neutral-700 dark:text-neutral-300 ml-1">
                {getLayoutName(effectiveLayout)}
              </span>
            </div>
            <div>
              <span class="text-neutral-500 dark:text-neutral-400">{t('workspaces.filesPanel')}</span>
              <span class="font-medium text-neutral-700 dark:text-neutral-300 ml-1">
                {effectiveShowFiles ? t('workspaces.open') : t('workspaces.closed')}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Modal Footer / Actions -->
      <div class="flex justify-end gap-2 pt-2 border-t border-neutral-100 dark:border-neutral-800">
        <button
          type="button"
          onclick={closeSaveModal}
          class="px-3.5 py-1.5 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg text-xs font-medium transition-colors"
        >
          {t('common.cancel')}
        </button>
        <button
          type="button"
          onclick={handleSaveWorkspace}
          disabled={!workspaceName.trim() || activeHostIds.length === 0 || isSaving}
          class="px-4 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-40 disabled:cursor-not-allowed text-white rounded-lg text-xs font-medium transition-colors shadow-lg shadow-sky-600/20 flex items-center gap-1.5"
        >
          {#if isSaving}
            <span class="animate-spin text-xs">●</span>
          {/if}
          <span>{t('workspaces.saveTitle')}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
