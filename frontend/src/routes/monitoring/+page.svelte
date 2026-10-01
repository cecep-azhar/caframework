<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { type HostMetrics } from '$lib/api/monitor';
  import { monitorState } from '$lib/stores/monitorStore.svelte';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let hosts = $state<HostRecord[]>([]);
  let errorMsg = $state('');
  let isLoading = $derived(monitorState.isPolling && monitorState.metrics.length === 0);
  let metrics = $derived(monitorState.metrics);

  async function loadHosts() {
    try {
      hosts = await listHosts();
      errorMsg = '';
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  onMount(() => {
    loadHosts();
  });

  function getHostLabel(hostId: string): string {
    const host = hosts.find(h => h.id === hostId);
    return host ? host.label || host.address : hostId;
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z']}
    accent="pink"
    title={t('monitoring.title')}
    subtitle={t('monitoring.subtitle')}
  />

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-600 dark:text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 text-center text-neutral-500 text-sm">{t('monitoring.loading')}</div>
  {:else if metrics.length === 0}
    <div class="p-8 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-white dark:bg-neutral-900/30 text-center flex flex-col items-center justify-center shadow-sm dark:shadow-none">
      <svg class="w-12 h-12 text-neutral-400 dark:text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path></svg>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-2">{t('monitoring.emptyTitle')}</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-md">{t('monitoring.emptyBody')}</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each metrics as m (m.host_id)}
        {@const ramPct = (m.mem_used_mb / m.mem_total_mb) * 100}
        {@const diskPct = (m.disk_used_gb / m.disk_total_gb) * 100}
        <div class="p-6 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-white dark:bg-neutral-900/40 hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors shadow-sm dark:shadow-none">
          <div class="flex items-center justify-between mb-4">
            <span class="font-medium text-neutral-900 dark:text-white text-sm">{getHostLabel(m.host_id)}</span>
            <span class="px-2 py-0.5 rounded text-xs font-mono bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> {t('monitoring.online')}
            </span>
          </div>

          <div class="text-xs text-neutral-500 dark:text-neutral-500 mb-4 pb-4 border-b border-neutral-100 dark:border-neutral-800/50">
            <div><span class="text-neutral-500 dark:text-neutral-400">{t('monitoring.os')}:</span> {m.os_name}</div>
            <div><span class="text-neutral-500 dark:text-neutral-400">{t('monitoring.uptime')}:</span> {m.uptime}</div>
            <div><span class="text-neutral-500 dark:text-neutral-400">{t('monitoring.hostname')}:</span> {m.hostname}</div>
          </div>

          <div class="space-y-4 text-sm">
            <div>
              <div class="flex justify-between text-xs text-neutral-500 dark:text-neutral-400 mb-1.5">
                <span>{t('monitoring.cpuUsage')}</span>
                <span>{m.cpu_usage.toFixed(1)}%</span>
              </div>
              <div class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden">
                <div class="h-full bg-sky-500 rounded-full transition-all duration-500" style="width: {m.cpu_usage}%"></div>
              </div>
            </div>

            <div>
              <div class="flex justify-between text-xs text-neutral-500 dark:text-neutral-400 mb-1.5">
                <span>{t('monitoring.ramUsage')}</span>
                <span>{(m.mem_used_mb / 1024).toFixed(1)} GB / {(m.mem_total_mb / 1024).toFixed(1)} GB</span>
              </div>
              <div class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden">
                <div class="h-full bg-indigo-500 rounded-full transition-all duration-500" style="width: {ramPct}%"></div>
              </div>
            </div>

            <div>
              <div class="flex justify-between text-xs text-neutral-500 dark:text-neutral-400 mb-1.5">
                <span>{t('monitoring.rootDisk')}</span>
                <span>{m.disk_used_gb.toFixed(1)} GB / {m.disk_total_gb.toFixed(1)} GB</span>
              </div>
              <div class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-800 rounded-full overflow-hidden">
                <div class="h-full bg-amber-500 rounded-full transition-all duration-500" style="width: {diskPct}%"></div>
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
