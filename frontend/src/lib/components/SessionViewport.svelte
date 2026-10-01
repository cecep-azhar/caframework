<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import type TerminalPaneComponent from '$lib/components/TerminalPane.svelte';
  import SessionFileManager from '$lib/components/SessionFileManager.svelte';
  import { listHosts } from '$lib/api/hosts';
  import { getTabs, openTab, closeSessionTab, tabLabel, LOCAL_HOST_ID, localTerminalHost } from '$lib/stores/sessionTabs.svelte';
  import {
    getSessionView,
    setSelectedTabId,
    setShowFiles,
    setLayout
  } from '$lib/stores/sessionView.svelte';

  const view = getSessionView();
  let loadError = $state('');
  let isWideViewport = $state(true);

  onMount(() => {
    const query = window.matchMedia('(min-width: 640px)');
    const sync = () => (isWideViewport = query.matches);
    sync();
    query.addEventListener('change', sync);
    return () => query.removeEventListener('change', sync);
  });

  // Watch URL params for opening new tabs
  let handledUrl = '';
  $effect(() => {
    const url = page.url.href;
    const params = page.url.searchParams;
    if (url === handledUrl) return;

    const ids: string[] = [];
    const single = params.get('host');
    if (single) ids.push(single);
    const multi = params.get('hosts');
    if (multi) {
      for (const id of multi.split(',').map((s) => s.trim()).filter(Boolean)) ids.push(id);
    }
    if (ids.length === 0) return;

    handledUrl = url;

    listHosts()
      .then((hosts) => {
        loadError = '';
        let lastOpened = '';
        for (const id of ids) {
          if (id === LOCAL_HOST_ID) {
            lastOpened = openTab(localTerminalHost());
            continue;
          }
          const host = hosts.find((h) => h.id === id);
          if (host) lastOpened = openTab(host);
        }
        if (lastOpened) setSelectedTabId(lastOpened);
      })
      .catch(() => {
        loadError = t('session.backendMissing');
      });
  });

  const tabs = $derived(getTabs());

  // xterm and the terminal pane (~300 KB of JS) load on first use instead of at startup: the
  // viewport lives in the layout, so a static import would ship xterm to every user who
  // never opens a session. Once loaded it stays loaded for later tabs.
  let TerminalPane = $state<typeof TerminalPaneComponent | null>(null);
  $effect(() => {
    if (tabs.length > 0 && !TerminalPane) {
      import('$lib/components/TerminalPane.svelte').then((mod) => (TerminalPane = mod.default));
    }
  });

  $effect(() => {
    if (tabs.length > 0 && (!view.selectedTabId || !tabs.some((tab) => tab.id === view.selectedTabId))) {
      setSelectedTabId(tabs[0].id);
    }
  });

  const activeTab = $derived.by(() => {
    if (tabs.length === 0) return undefined;
    return tabs.find((tab) => tab.id === view.selectedTabId) ?? tabs[0];
  });

  const activeHost = $derived(activeTab?.host);
  const effectiveLayout = $derived(!isWideViewport || tabs.length === 1 ? 1 : view.layout);

  const containerClass = $derived(
    effectiveLayout === 2
      ? 'flex flex-col h-full gap-1 sm:gap-1.5'
      : effectiveLayout === 3
        ? 'flex h-full gap-1 sm:gap-1.5'
        : effectiveLayout === 4
          ? 'grid grid-cols-2 auto-rows-fr h-full gap-1 sm:gap-1.5'
          : 'relative h-full w-full'
  );

  function paneClass(tabId: string): string {
    if (effectiveLayout !== 1) return 'min-h-0 min-w-0 flex-1';
    return tabId === activeTab?.id
      ? 'absolute inset-0 z-10'
      : 'absolute inset-0 invisible pointer-events-none';
  }

  function close(tabId: string) {
    closeSessionTab(tabId);
  }
</script>

<div class="h-full w-full flex flex-col bg-neutral-100 dark:bg-neutral-950 overflow-hidden">
  <div class="flex-1 w-full min-h-0 overflow-hidden p-1 sm:p-1.5">
    {#if loadError}
      <div class="p-4 text-sm text-amber-700 dark:text-amber-400 bg-amber-50 dark:bg-neutral-900 border border-amber-200 dark:border-neutral-800 rounded">{loadError}</div>
    {:else if tabs.length === 0}
      <div class="h-full flex flex-col items-center justify-center gap-3 text-center border border-dashed border-neutral-300 dark:border-neutral-800 rounded-lg p-4">
        <p class="text-neutral-800 dark:text-neutral-300 font-medium">{t('session.emptyTitle')}</p>
        <p class="text-neutral-500 text-sm max-w-sm">{t('session.emptyBody')}</p>
        <a href="/" class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-sm font-medium rounded-md transition-colors">{t('session.goToHosts')}</a>
      </div>
    {:else}
      <div class="flex h-full gap-1 sm:gap-1.5 overflow-hidden">
        <div class="flex-1 min-w-0 h-full overflow-hidden flex flex-col">
          {#if tabs.length > 1 && !isWideViewport}
            <div class="flex items-center gap-1 overflow-x-auto pb-1.5 mb-1 scrollbar-none shrink-0">
              {#each tabs as tab (tab.id)}
                <button
                  onclick={() => setSelectedTabId(tab.id)}
                  class="px-2.5 py-1 rounded text-xs font-mono transition-colors shrink-0 flex items-center gap-1.5 {view.selectedTabId === tab.id ? 'bg-sky-600 text-white font-semibold shadow-xs' : 'bg-white dark:bg-neutral-900 text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 border border-neutral-200 dark:border-neutral-800'}"
                >
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                  <span class="truncate max-w-[120px]">{tabLabel(tab)}</span>
                </button>
              {/each}
            </div>
          {/if}

          <div class="flex-1 min-h-0">
            <div class={containerClass}>
              {#each tabs as tab (tab.id)}
                <div class={paneClass(tab.id)}>
                  {#if TerminalPane}
                  <TerminalPane
                    host={tab.host}
                    label={tabLabel(tab)}
                    isActive={effectiveLayout === 1 ? tab.id === activeTab?.id : true}
                    onSplitRight={effectiveLayout === 1 && tabs.length > 1 ? () => setLayout(3) : undefined}
                    onSplitDown={effectiveLayout === 1 && tabs.length > 1 ? () => setLayout(2) : undefined}
                    onClose={() => close(tab.id)}
                  />
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        </div>

        {#if view.showFiles && activeHost}
          <div class="hidden md:block w-80 lg:w-96 shrink-0 h-full overflow-hidden">
            <SessionFileManager
              host={activeHost}
              availableHosts={tabs.map((tab) => tab.host)}
              onSelectHost={(h) => setSelectedTabId(tabs.find((tab) => tab.host.id === h.id)?.id ?? '')}
              onClose={() => setShowFiles(false)}
            />
          </div>

          <div class="md:hidden fixed inset-0 z-50 bg-black/40 dark:bg-black/70 backdrop-blur-xs flex flex-col justify-end p-2">
            <div class="w-full h-full max-h-[94vh] flex flex-col bg-white dark:bg-neutral-950 rounded-lg shadow-2xl border border-neutral-200 dark:border-neutral-800 overflow-hidden">
              <SessionFileManager
                host={activeHost}
                availableHosts={tabs.map((tab) => tab.host)}
                onSelectHost={(h) => setSelectedTabId(tabs.find((tab) => tab.host.id === h.id)?.id ?? '')}
                onClose={() => setShowFiles(false)}
              />
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
