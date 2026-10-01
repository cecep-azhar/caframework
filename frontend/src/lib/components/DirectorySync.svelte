<script lang="ts">
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import { onMount } from 'svelte';
  import {
    planSync,
    executeSync,
    startWatch,
    stopWatch,
    listWatches,
    type SyncDirection,
    type SyncPlan,
    type SyncStats,
    type WatchInfo
  } from '$lib/api/sync';

  let {
    hostId,
    initialLocalPath = '~',
    initialRemotePath = '/',
    onClose
  }: {
    hostId: string;
    initialLocalPath?: string;
    initialRemotePath?: string;
    onClose?: () => void;
  } = $props();

  let localDir = $state('');
  let remoteDir = $state('');
  let direction = $state<SyncDirection>('LocalToRemote');

  $effect(() => {
    if (!localDir) localDir = initialLocalPath;
    if (!remoteDir) remoteDir = initialRemotePath;
  });

  let isPlanning = $state(false);
  let isExecuting = $state(false);
  let plan = $state<SyncPlan | null>(null);
  let stats = $state<SyncStats | null>(null);

  let activeTab = $state<'upload' | 'download' | 'conflicts'>('upload');

  // File watcher state
  let watches = $state<WatchInfo[]>([]);
  let isStartingWatch = $state(false);
  let watchError = $state('');

  let errorMessage = $state('');
  let successMessage = $state('');

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + (sizes[i] ?? 'B');
  }

  function formatMtime(secs: number): string {
    if (!secs) return '-';
    return new Date(secs * 1000).toLocaleString(intlLocale());
  }

  async function refreshWatches() {
    try {
      watches = await listWatches();
    } catch (e: any) {
      console.error('Failed to list watches:', e);
    }
  }

  onMount(() => {
    void refreshWatches();
  });

  async function handlePlan() {
    if (!hostId) {
      errorMessage = t('dirSync.noHost');
      return;
    }
    errorMessage = '';
    successMessage = '';
    stats = null;
    isPlanning = true;
    try {
      plan = await planSync(hostId, localDir, remoteDir, direction);
      if (plan.to_upload.length > 0) {
        activeTab = 'upload';
      } else if (plan.to_download.length > 0) {
        activeTab = 'download';
      } else if (plan.conflicts.length > 0) {
        activeTab = 'conflicts';
      }
    } catch (e: any) {
      errorMessage = e?.message ?? String(e);
      plan = null;
    } finally {
      isPlanning = false;
    }
  }

  async function handleExecute() {
    if (!hostId || !plan) return;
    errorMessage = '';
    successMessage = '';
    isExecuting = true;
    try {
      stats = await executeSync(hostId, plan, direction);
      successMessage = t('dirSync.syncComplete', { uploaded: stats.files_uploaded, downloaded: stats.files_downloaded, bytes: formatSize(stats.bytes_transferred) });
      // Clear plan after successful run
      plan = null;
    } catch (e: any) {
      errorMessage = e?.message ?? String(e);
    } finally {
      isExecuting = false;
    }
  }

  async function handleStartWatch() {
    if (!hostId) {
      watchError = t('dirSync.noHost');
      return;
    }
    watchError = '';
    isStartingWatch = true;
    try {
      await startWatch(hostId, localDir, remoteDir);
      await refreshWatches();
      successMessage = t('dirSync.watchStarted', { local: localDir, remote: remoteDir });
    } catch (e: any) {
      watchError = e?.message ?? String(e);
    } finally {
      isStartingWatch = false;
    }
  }

  async function handleStopWatch(id: string) {
    watchError = '';
    try {
      await stopWatch(id);
      await refreshWatches();
    } catch (e: any) {
      watchError = e?.message ?? String(e);
    }
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-[#151921] text-neutral-800 dark:text-neutral-100 rounded-lg overflow-hidden">
  <!-- Header -->
  <div class="px-4 py-3 bg-neutral-100 dark:bg-[#1c212c] border-b border-neutral-200 dark:border-slate-800 flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="text-base">🔄</span>
      <h2 class="text-sm font-bold tracking-wide">{t('dirSync.title')}</h2>
    </div>
    {#if onClose}
      <button
        type="button"
        onclick={onClose}
        class="p-1 hover:bg-neutral-200 dark:hover:bg-slate-700 rounded text-neutral-500 hover:text-neutral-800 dark:hover:text-white text-xs transition"
        title={t('common.close')}
        aria-label={t('common.close')}
      >
        ✕
      </button>
    {/if}
  </div>

  <!-- Main Scrollable Body -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4 text-xs">
    <!-- Messages -->
    {#if errorMessage}
      <div class="p-2.5 rounded bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400">
        {errorMessage}
      </div>
    {/if}
    {#if successMessage}
      <div class="p-2.5 rounded bg-emerald-500/10 border border-emerald-500/30 text-emerald-600 dark:text-emerald-400">
        {successMessage}
      </div>
    {/if}

    <!-- Configuration Form -->
    <div class="p-3 bg-neutral-50 dark:bg-[#1a1f29] border border-neutral-200 dark:border-slate-800 rounded-lg space-y-3">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label for="sync-local-dir" class="block font-semibold text-neutral-600 dark:text-slate-400 mb-1">
            {t('dirSync.localDir')}
          </label>
          <input
            id="sync-local-dir"
            type="text"
            bind:value={localDir}
            placeholder={t('dirSync.localDirPlaceholder')}
            class="w-full bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2.5 py-1.5 font-mono focus:outline-none focus:border-cyan-500"
          />
        </div>
        <div>
          <label for="sync-remote-dir" class="block font-semibold text-neutral-600 dark:text-slate-400 mb-1">
            {t('dirSync.remoteDir')}
          </label>
          <input
            id="sync-remote-dir"
            type="text"
            bind:value={remoteDir}
            placeholder={t('dirSync.remoteDirPlaceholder')}
            class="w-full bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2.5 py-1.5 font-mono focus:outline-none focus:border-cyan-500"
          />
        </div>
      </div>

      <div class="flex flex-wrap items-center justify-between gap-3 pt-1">
        <div class="flex items-center gap-2">
          <label for="sync-direction" class="font-semibold text-neutral-600 dark:text-slate-400">
            {t('dirSync.direction')}
          </label>
          <select
            id="sync-direction"
            bind:value={direction}
            class="bg-white dark:bg-slate-900 border border-neutral-300 dark:border-slate-700 rounded px-2 py-1 focus:outline-none focus:border-cyan-500"
          >
            <option value="LocalToRemote">{t('dirSync.dirUp')}</option>
            <option value="RemoteToLocal">{t('dirSync.dirDown')}</option>
            <option value="TwoWay">{t('dirSync.dirTwoWay')}</option>
          </select>
        </div>

        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={handlePlan}
            disabled={isPlanning || isExecuting}
            class="px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white rounded font-medium flex items-center gap-1.5 transition cursor-pointer"
          >
            {#if isPlanning}
              <span class="animate-spin">⏳</span> {t('dirSync.scanning')}
            {:else}
              <span>🔍</span> {t('dirSync.previewDiff')}
            {/if}
          </button>

          {#if plan && (plan.to_upload.length > 0 || plan.to_download.length > 0)}
            <button
              type="button"
              onclick={handleExecute}
              disabled={isExecuting}
              class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white rounded font-medium flex items-center gap-1.5 transition cursor-pointer shadow-sm"
            >
              {#if isExecuting}
                <span class="animate-spin">⏳</span> {t('dirSync.synchronizing')}
              {:else}
                <span>🚀</span> {t('dirSync.executeSync', { count: plan.to_upload.length + plan.to_download.length })}
              {/if}
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Stats Card -->
    {#if stats}
      <div class="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg flex items-center justify-around text-center">
        <div>
          <div class="text-xs text-neutral-500 dark:text-slate-400">{t('dirSync.filesUploaded')}</div>
          <div class="text-base font-bold text-emerald-600 dark:text-emerald-400">{stats.files_uploaded}</div>
        </div>
        <div class="h-8 w-px bg-emerald-500/20"></div>
        <div>
          <div class="text-xs text-neutral-500 dark:text-slate-400">{t('dirSync.filesDownloaded')}</div>
          <div class="text-base font-bold text-emerald-600 dark:text-emerald-400">{stats.files_downloaded}</div>
        </div>
        <div class="h-8 w-px bg-emerald-500/20"></div>
        <div>
          <div class="text-xs text-neutral-500 dark:text-slate-400">{t('dirSync.dataTransferred')}</div>
          <div class="text-base font-bold text-cyan-600 dark:text-cyan-400">{formatSize(stats.bytes_transferred)}</div>
        </div>
      </div>
    {/if}

    <!-- Diff Plan Preview -->
    {#if plan}
      <div class="border border-neutral-200 dark:border-slate-800 rounded-lg overflow-hidden bg-neutral-50 dark:bg-[#181c24]">
        <!-- Plan Tabs -->
        <div class="flex items-center border-b border-neutral-200 dark:border-slate-800 bg-neutral-100 dark:bg-[#1c212c]">
          <button
            type="button"
            onclick={() => (activeTab = 'upload')}
            class={`px-3 py-2 font-semibold border-b-2 flex items-center gap-1.5 transition cursor-pointer ${
              activeTab === 'upload'
                ? 'border-cyan-500 text-cyan-600 dark:text-cyan-400 bg-white dark:bg-[#181c24]'
                : 'border-transparent text-neutral-600 dark:text-slate-400 hover:text-neutral-900 dark:hover:text-white'
            }`}
          >
            <span>{t('dirSync.toUpload')}</span>
            <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-cyan-100 dark:bg-cyan-900/60 text-cyan-700 dark:text-cyan-300">
              {plan.to_upload.length}
            </span>
          </button>
          <button
            type="button"
            onclick={() => (activeTab = 'download')}
            class={`px-3 py-2 font-semibold border-b-2 flex items-center gap-1.5 transition cursor-pointer ${
              activeTab === 'download'
                ? 'border-cyan-500 text-cyan-600 dark:text-cyan-400 bg-white dark:bg-[#181c24]'
                : 'border-transparent text-neutral-600 dark:text-slate-400 hover:text-neutral-900 dark:hover:text-white'
            }`}
          >
            <span>{t('dirSync.toDownload')}</span>
            <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-cyan-100 dark:bg-cyan-900/60 text-cyan-700 dark:text-cyan-300">
              {plan.to_download.length}
            </span>
          </button>
          <button
            type="button"
            onclick={() => (activeTab = 'conflicts')}
            class={`px-3 py-2 font-semibold border-b-2 flex items-center gap-1.5 transition cursor-pointer ${
              activeTab === 'conflicts'
                ? 'border-amber-500 text-amber-600 dark:text-amber-400 bg-white dark:bg-[#181c24]'
                : 'border-transparent text-neutral-600 dark:text-slate-400 hover:text-neutral-900 dark:hover:text-white'
            }`}
          >
            <span>{t('dirSync.conflicts')}</span>
            <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-amber-100 dark:bg-amber-900/60 text-amber-700 dark:text-amber-300">
              {plan.conflicts.length}
            </span>
          </button>
        </div>

        <!-- Plan Table -->
        <div class="max-h-60 overflow-y-auto font-mono text-[11px]">
          {#if activeTab === 'upload'}
            {#if plan.to_upload.length === 0}
              <div class="p-6 text-center text-neutral-400 dark:text-slate-500 italic">{t('dirSync.nothingToUpload')}</div>
            {:else}
              <table class="w-full text-left">
                <thead class="sticky top-0 bg-neutral-200/80 dark:bg-slate-900/80 border-b border-neutral-200 dark:border-slate-800 text-neutral-600 dark:text-slate-400">
                  <tr>
                    <th class="py-1.5 px-3">{t('dirSync.colLocalFile')}</th>
                    <th class="py-1.5 px-3">{t('dirSync.colRemoteTarget')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colSize')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colModified')}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-slate-800/40">
                  {#each plan.to_upload as item}
                    <tr class="hover:bg-cyan-50/50 dark:hover:bg-cyan-950/20">
                      <td class="py-1.5 px-3 text-neutral-800 dark:text-slate-200 truncate max-w-xs">{item.local_path}</td>
                      <td class="py-1.5 px-3 text-neutral-500 dark:text-slate-400 truncate max-w-xs">{item.remote_path}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatSize(item.size)}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatMtime(item.mtime)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          {:else if activeTab === 'download'}
            {#if plan.to_download.length === 0}
              <div class="p-6 text-center text-neutral-400 dark:text-slate-500 italic">{t('dirSync.nothingToDownload')}</div>
            {:else}
              <table class="w-full text-left">
                <thead class="sticky top-0 bg-neutral-200/80 dark:bg-slate-900/80 border-b border-neutral-200 dark:border-slate-800 text-neutral-600 dark:text-slate-400">
                  <tr>
                    <th class="py-1.5 px-3">{t('dirSync.colRemoteSource')}</th>
                    <th class="py-1.5 px-3">{t('dirSync.colLocalTarget')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colSize')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colModified')}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-slate-800/40">
                  {#each plan.to_download as item}
                    <tr class="hover:bg-cyan-50/50 dark:hover:bg-cyan-950/20">
                      <td class="py-1.5 px-3 text-neutral-800 dark:text-slate-200 truncate max-w-xs">{item.remote_path}</td>
                      <td class="py-1.5 px-3 text-neutral-500 dark:text-slate-400 truncate max-w-xs">{item.local_path}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatSize(item.size)}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatMtime(item.mtime)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          {:else if activeTab === 'conflicts'}
            {#if plan.conflicts.length === 0}
              <div class="p-6 text-center text-neutral-400 dark:text-slate-500 italic">{t('dirSync.noConflicts')}</div>
            {:else}
              <table class="w-full text-left">
                <thead class="sticky top-0 bg-neutral-200/80 dark:bg-slate-900/80 border-b border-neutral-200 dark:border-slate-800 text-neutral-600 dark:text-slate-400">
                  <tr>
                    <th class="py-1.5 px-3">{t('dirSync.colFilePath')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colLocalMtime')}</th>
                    <th class="py-1.5 px-3 text-right">{t('dirSync.colRemoteMtime')}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-neutral-200 dark:divide-slate-800/40">
                  {#each plan.conflicts as item}
                    <tr class="hover:bg-amber-50/50 dark:hover:bg-amber-950/20">
                      <td class="py-1.5 px-3 text-amber-700 dark:text-amber-300 font-medium truncate max-w-sm">{item.local_path}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatMtime(item.local_mtime)}</td>
                      <td class="py-1.5 px-3 text-right text-neutral-500 dark:text-slate-400 whitespace-nowrap">{formatMtime(item.remote_mtime)}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          {/if}
        </div>
      </div>
    {/if}

    <!-- Live File Watcher Section (WinSCP Keep Remote Up to Date) -->
    <div class="p-3 bg-neutral-50 dark:bg-[#1a1f29] border border-neutral-200 dark:border-slate-800 rounded-lg space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="font-bold flex items-center gap-1.5 text-neutral-800 dark:text-neutral-100">
            <span>👁️</span> {t('dirSync.watchTitle')}
          </h3>
          <p class="text-[11px] text-neutral-500 dark:text-slate-400">
            {t('dirSync.watchBody')}
          </p>
        </div>
        <button
          type="button"
          onclick={handleStartWatch}
          disabled={isStartingWatch}
          class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded font-medium flex items-center gap-1.5 transition cursor-pointer"
        >
          {#if isStartingWatch}
            <span class="animate-spin">⏳</span> {t('dirSync.starting')}
          {:else}
            <span>▶</span> {t('dirSync.startWatch')}
          {/if}
        </button>
      </div>

      {#if watchError}
        <div class="p-2 text-[11px] rounded bg-rose-500/10 border border-rose-500/30 text-rose-600 dark:text-rose-400">
          {watchError}
        </div>
      {/if}

      <!-- Watches List -->
      <div class="space-y-2">
        <div class="text-[11px] font-semibold text-neutral-600 dark:text-slate-400 uppercase tracking-wider">
          {t('dirSync.registered', { count: watches.length })}
        </div>

        {#if watches.length === 0}
          <div class="p-3 text-center text-neutral-400 dark:text-slate-500 text-xs italic bg-white dark:bg-slate-900 rounded border border-neutral-200 dark:border-slate-800">
            {t('dirSync.noWatchers')}
          </div>
        {:else}
          <div class="divide-y divide-neutral-200 dark:divide-slate-800 border border-neutral-200 dark:border-slate-800 rounded bg-white dark:bg-slate-900 overflow-hidden font-mono text-[11px]">
            {#each watches as w}
              <div class="p-2.5 flex items-center justify-between gap-3">
                <div class="space-y-0.5 truncate">
                  <div class="flex items-center gap-2">
                    <span class={`inline-block w-2 h-2 rounded-full ${w.is_active ? 'bg-emerald-500 animate-pulse' : 'bg-neutral-400'}`}></span>
                    <span class="font-semibold text-neutral-800 dark:text-neutral-200">{w.id}</span>
                    <span class={`text-[10px] px-1.5 py-0.2 rounded font-sans ${w.is_active ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-400' : 'bg-neutral-200 dark:bg-slate-700 text-neutral-500 dark:text-slate-400'}`}>
                      {w.is_active ? t('dirSync.active') : t('dirSync.stopped')}
                    </span>
                  </div>
                  <div class="text-neutral-500 dark:text-slate-400 truncate">
                    <span>{t('dirSync.local', { path: w.local_dir })}</span> ➔ <span>{t('dirSync.remote', { path: w.remote_dir })}</span>
                  </div>
                </div>

                {#if w.is_active}
                  <button
                    type="button"
                    onclick={() => handleStopWatch(w.id)}
                    class="px-2.5 py-1 bg-rose-600/90 hover:bg-rose-500 text-white rounded text-[11px] font-sans font-medium transition cursor-pointer shrink-0"
                  >
                    {t('dirSync.stop')}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
