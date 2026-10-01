<script lang="ts">
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import { onMount } from 'svelte';
  import { listRemoteDir, type SftpFileEntry } from '$lib/api/sftp';
  import type { HostRecord } from '$lib/api/hosts';

  let {
    host,
    availableHosts = [],
    onSelectHost,
    onClose
  }: {
    host: HostRecord;
    availableHosts?: HostRecord[];
    onSelectHost?: (h: HostRecord) => void;
    onClose?: () => void;
  } = $props();

  let currentPath = $state('/');
  let pathInputValue = $state('/');
  let files = $state<SftpFileEntry[]>([]);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let searchQuery = $state('');
  let retryTimer: ReturnType<typeof setTimeout> | null = null;

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  function formatDate(mtime: number): string {
    if (!mtime) return '-';
    const d = new Date(mtime * 1000);
    return d.toLocaleDateString(intlLocale(), {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  async function fetchFiles(retryCount = 0) {
    if (!host?.id) return;
    if (retryTimer) {
      clearTimeout(retryTimer);
      retryTimer = null;
    }
    isLoading = true;
    errorMsg = '';

    try {
      const res = await listRemoteDir(host.id, currentPath);
      files = res;
      pathInputValue = currentPath;
    } catch (e: any) {
      const msg = typeof e === 'string' ? e : (e?.message || t('fileManager.loadFailed'));
      if (msg.toLowerCase().includes('no active ssh session') && retryCount < 4) {
        errorMsg = t('fileManager.waitingSsh');
        retryTimer = setTimeout(() => {
          fetchFiles(retryCount + 1);
        }, 1200);
        return;
      }
      errorMsg = msg;
      files = [];
    } finally {
      isLoading = false;
    }
  }

  // Navigate to target path
  function navigateTo(targetPath: string) {
    currentPath = targetPath;
    pathInputValue = targetPath;
    fetchFiles();
  }

  // Up one directory
  function goUp() {
    if (currentPath === '/' || !currentPath) return;
    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    currentPath = parts.length === 0 ? '/' : '/' + parts.join('/');
    pathInputValue = currentPath;
    fetchFiles();
  }

  // Direct path submit
  function handlePathSubmit() {
    let target = pathInputValue.trim();
    if (!target) target = '/';
    if (!target.startsWith('/')) target = '/' + target;
    currentPath = target;
    fetchFiles();
  }

  // Breadcrumbs breakdown
  const breadcrumbs = $derived.by(() => {
    if (currentPath === '/' || !currentPath) {
      return [{ name: '/', path: '/' }];
    }
    const parts = currentPath.split('/').filter(Boolean);
    const crumbs = [{ name: '/', path: '/' }];
    let accum = '';
    for (const part of parts) {
      accum += '/' + part;
      crumbs.push({ name: part, path: accum });
    }
    return crumbs;
  });

  // Filtered files
  const filteredFiles = $derived.by(() => {
    if (!searchQuery.trim()) return files;
    const q = searchQuery.toLowerCase();
    return files.filter(f => f.name.toLowerCase().includes(q));
  });

  const folderCount = $derived(files.filter(f => f.is_dir).length);
  const fileCount = $derived(files.filter(f => !f.is_dir).length);

  // When host changes
  $effect(() => {
    if (host?.id) {
      currentPath = '/';
      pathInputValue = '/';
      fetchFiles();
    }
  });

  onMount(() => {
    fetchFiles();
    return () => {
      if (retryTimer) clearTimeout(retryTimer);
    };
  });
</script>

<div class="flex flex-col h-full bg-white dark:bg-[#09090b] border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden text-neutral-700 dark:text-neutral-300">
  <!-- Header Bar -->
  <div class="h-8 bg-neutral-50 dark:bg-neutral-900 border-b border-neutral-200 dark:border-neutral-800 px-3 flex items-center justify-between text-xs shrink-0 select-none">
    <div class="flex items-center gap-2 min-w-0">
      <svg class="w-4 h-4 text-sky-500 dark:text-sky-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
      </svg>
      {#if availableHosts.length > 1 && onSelectHost}
        <select
          value={host.id}
          onchange={(e) => {
            const h = availableHosts.find(item => item.id === (e.target as HTMLSelectElement).value);
            if (h) onSelectHost(h);
          }}
          class="bg-white dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-700 rounded px-1.5 py-0.5 text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 max-w-[130px] truncate"
        >
          {#each availableHosts as h}
            <option value={h.id}>{h.label || h.address}</option>
          {/each}
        </select>
      {:else}
        <span class="text-neutral-900 dark:text-white font-medium truncate">{host.label}</span>
      {/if}
      <span class="text-neutral-500 font-mono text-[11px] hidden sm:inline">SFTP</span>
    </div>

    <div class="flex items-center gap-1.5 text-neutral-500 dark:text-neutral-400">
      <!-- Refresh -->
      <button
        onclick={() => fetchFiles()}
        class="p-1 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded transition-colors"
        title={t('fileManager.refresh')}
        aria-label={t('fileManager.refresh')}
      >
        <svg class="w-3.5 h-3.5 {isLoading ? 'animate-spin text-sky-500 dark:text-sky-400' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
        </svg>
      </button>

      <!-- Open Full SFTP Page -->
      <a
        href="/sftp?host={host.id}"
        class="p-1 hover:text-emerald-600 dark:hover:text-emerald-400 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded transition-colors"
        title={t('fileManager.openFull')}
        aria-label={t('fileManager.openFull')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
        </svg>
      </a>

      <!-- Close Panel -->
      {#if onClose}
        <button
          onclick={onClose}
          class="p-1 sm:p-1 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-neutral-200 dark:hover:bg-neutral-800 rounded transition-colors flex items-center gap-1"
          title={t('fileManager.closePanel')}
          aria-label={t('fileManager.closePanel')}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
          <span class="md:hidden text-[11px] text-rose-600 dark:text-rose-400 font-medium">{t('common.close')}</span>
        </button>
      {/if}
    </div>
  </div>

  <!-- Path Navigation & Breadcrumbs -->
  <div class="p-2 border-b border-neutral-200 dark:border-neutral-800/80 bg-neutral-50/60 dark:bg-neutral-950/40 space-y-1.5 shrink-0">
    <!-- Path input row -->
    <div class="flex items-center gap-1">
      <button
        onclick={goUp}
        disabled={currentPath === '/' || !currentPath}
        class="p-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 hover:border-neutral-400 dark:hover:border-neutral-700 disabled:opacity-40 rounded text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors shrink-0"
        title={t('fileManager.goUp')}
        aria-label={t('fileManager.goUp')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"></path>
        </svg>
      </button>

      <input
        type="text"
        bind:value={pathInputValue}
        onkeydown={(e) => e.key === 'Enter' && handlePathSubmit()}
        placeholder="/remote/path"
        class="flex-1 min-w-0 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded px-2 py-1 text-xs text-neutral-900 dark:text-white font-mono focus:outline-none focus:border-sky-500 transition-colors"
      />

      <button
        onclick={handlePathSubmit}
        class="px-2 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded text-xs font-medium transition-colors shrink-0"
      >
        {t('fileManager.go')}
      </button>
    </div>

    <!-- Breadcrumb trail -->
    <div class="flex items-center gap-1 overflow-x-auto text-[11px] text-neutral-500 dark:text-neutral-400 py-0.5 scrollbar-none font-mono">
      {#each breadcrumbs as crumb, i}
        {#if i > 0}
          <span class="text-neutral-400 dark:text-neutral-600">/</span>
        {/if}
        <button
          onclick={() => navigateTo(crumb.path)}
          class="hover:text-neutral-900 dark:hover:text-white hover:underline truncate max-w-[120px] transition-colors {crumb.path === currentPath ? 'text-sky-600 dark:text-sky-400 font-semibold' : ''}"
          title={crumb.path}
        >
          {crumb.name}
        </button>
      {/each}
    </div>

    <!-- Search filter -->
    <div class="relative">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={t('fileManager.filterPlaceholder')}
        class="w-full bg-white dark:bg-neutral-900/80 border border-neutral-300 dark:border-neutral-800/80 rounded px-2 py-1 text-xs text-neutral-800 dark:text-neutral-200 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-700 transition-colors pl-6"
      />
      <svg class="w-3 h-3 text-neutral-400 dark:text-neutral-500 absolute left-2 top-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
      </svg>
      {#if searchQuery}
        <button
          onclick={() => searchQuery = ''}
          class="absolute right-2 top-1.5 text-neutral-400 dark:text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 text-xs"
          aria-label={t('common.clear')}
        >
          ×
        </button>
      {/if}
    </div>
  </div>

  <!-- Main File List Area -->
  <div class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 bg-white dark:bg-[#09090b]">
    {#if errorMsg}
      <div class="p-3 m-2 bg-amber-500/10 border border-amber-500/20 rounded text-amber-700 dark:text-amber-400 text-xs space-y-2">
        <p class="font-mono break-all">{errorMsg}</p>
        <button
          onclick={() => fetchFiles()}
          class="px-2 py-1 bg-amber-500/15 dark:bg-amber-500/20 hover:bg-amber-500/25 dark:hover:bg-amber-500/30 text-amber-800 dark:text-amber-300 rounded text-[11px] font-medium transition-colors"
        >
          {t('fileManager.retry')}
        </button>
      </div>
    {:else if isLoading && files.length === 0}
      <div class="p-8 text-center text-xs text-neutral-500 space-y-2">
        <div class="w-5 h-5 border-2 border-sky-500 border-t-transparent rounded-full animate-spin mx-auto"></div>
        <p>{t('fileManager.loading')}</p>
      </div>
    {:else if filteredFiles.length === 0}
      <div class="p-8 text-center text-xs text-neutral-500">
        {#if searchQuery}
          <p>{t('fileManager.noMatch', { query: searchQuery })}</p>
        {:else}
          <p>{t('fileManager.empty')}</p>
        {/if}
      </div>
    {:else}
      <table class="w-full text-left text-xs border-collapse">
        <thead class="sticky top-0 bg-neutral-50/95 dark:bg-neutral-950/90 text-neutral-500 text-[10px] uppercase font-semibold border-b border-neutral-200 dark:border-neutral-800 select-none backdrop-blur-sm z-10">
          <tr>
            <th class="px-2.5 py-1.5">{t('fileManager.colName')}</th>
            <th class="px-2 py-1.5 text-right w-16">{t('fileManager.colSize')}</th>
            <th class="px-2.5 py-1.5 text-right w-24 hidden sm:table-cell">{t('fileManager.colModified')}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-neutral-100 dark:divide-neutral-900/60 font-mono text-[11px]">
          {#each filteredFiles as file (file.path)}
            <tr
              class="hover:bg-neutral-100 dark:hover:bg-neutral-800/50 transition-colors group cursor-pointer {file.is_dir ? 'text-neutral-800 dark:text-neutral-200' : 'text-neutral-600 dark:text-neutral-400'}"
              onclick={() => {
                if (file.is_dir) navigateTo(file.path);
              }}
            >
              <!-- Name & Icon -->
              <td class="px-2.5 py-1.5 flex items-center gap-2 min-w-0">
                {#if file.is_dir}
                  <svg class="w-4 h-4 text-amber-500 shrink-0" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z"></path>
                  </svg>
                  <span class="truncate group-hover:text-amber-600 dark:group-hover:text-amber-300 transition-colors font-medium">
                    {file.name}
                  </span>
                {:else}
                  <svg class="w-4 h-4 text-neutral-400 dark:text-neutral-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"></path>
                  </svg>
                  <span class="truncate group-hover:text-neutral-900 dark:group-hover:text-white transition-colors">
                    {file.name}
                  </span>
                {/if}
              </td>

              <!-- Size -->
              <td class="px-2 py-1.5 text-right whitespace-nowrap text-[10px] text-neutral-500">
                {#if file.is_dir}
                  <span class="text-neutral-400 dark:text-neutral-600">{t('fileManager.dir')}</span>
                {:else}
                  {formatSize(file.size)}
                {/if}
              </td>

              <!-- Modified Date -->
              <td class="px-2.5 py-1.5 text-right whitespace-nowrap text-[10px] text-neutral-500 hidden sm:table-cell">
                {formatDate(file.mtime)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <!-- Status Footer -->
  <div class="h-6 bg-neutral-50 dark:bg-neutral-950 border-t border-neutral-200 dark:border-neutral-800 px-3 flex items-center justify-between text-[10px] text-neutral-500 font-mono shrink-0 select-none">
    <span>{t('fileManager.summary', { folders: folderCount, files: fileCount })}</span>
    <span>{currentPath}</span>
  </div>
</div>
