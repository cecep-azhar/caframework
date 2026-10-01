<script lang="ts">
  // Slide-over shown by the (i) button on a host card. That button used to be a decoration
  // with no onclick at all; this is what it now opens.
  import { invoke } from '@tauri-apps/api/core';
  import { saveHost, type HostRecord, type ConnectionProtocol } from '$lib/api/hosts';
  import { openSession } from '$lib/nav';
  import { errorText } from '$lib/errors';
  import { t, intlLocale } from '$lib/i18n/index.svelte';

  let { host, onClose }: { host: HostRecord; onClose: () => void } = $props();

  interface CommandLog {
    id: string;
    event_type: string;
    timestamp: number;
    host_id: string | null;
    details: string;
  }

  let logs = $state<CommandLog[]>([]);
  let isLoadingLogs = $state(true);
  let logError = $state('');

  // Re-fetch whenever the panel is pointed at a different host.
  $effect(() => {
    const hostId = host.id;
    isLoadingLogs = true;
    logError = '';
    invoke<CommandLog[]>('get_command_logs', { hostId, search: null })
      .then((rows) => {
        logs = rows.slice(0, 50);
      })
      .catch((err) => {
        logError = errorText(err);
        logs = [];
      })
      .finally(() => {
        isLoadingLogs = false;
      });
  });

  function formatTime(seconds: number): string {
    const d = new Date(seconds * 1000);
    return Number.isNaN(d.getTime()) ? '-' : d.toLocaleString(intlLocale());
  }

  const authLabel = $derived.by(() => {
    const auth = host.authMethod;
    switch (auth.type) {
      case 'password':
        return t('hostDetail.authPassword');
      case 'key':
        return t('hostDetail.authKeyFile', { path: auth.path || '-' });
      case 'keyId':
        return t('hostDetail.authVaultKey');
    }
  });
</script>

<div class="fixed inset-0 z-50 flex justify-end">
  <button
    type="button"
    class="absolute inset-0 bg-black/50 backdrop-blur-xs"
    onclick={onClose}
    aria-label={t('hostDetail.closePanel')}
  ></button>

  <aside
    class="relative w-full max-w-md h-full bg-white dark:bg-neutral-950 border-l border-neutral-200 dark:border-neutral-800 shadow-2xl flex flex-col"
    aria-label={t('hostDetail.ariaLabel', { label: host.label })}
  >
    <header class="p-4 border-b border-neutral-200 dark:border-neutral-800 flex items-start justify-between gap-3 shrink-0">
      <div class="min-w-0">
        <h2 class="text-base font-bold text-neutral-900 dark:text-white truncate">{host.label}</h2>
        <p class="text-xs font-mono text-sky-600 dark:text-sky-400 truncate">
          {host.username}@{host.address}:{host.port}
        </p>
      </div>
      <button
        onclick={onClose}
        class="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors shrink-0"
        aria-label={t('common.close')}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </header>

    <div class="flex-1 overflow-y-auto p-4 space-y-5">
      <section class="space-y-2">
        <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500">{t('hostDetail.connection')}</h3>
        <dl class="text-xs space-y-1.5">
          <div class="flex justify-between items-center gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.protocol')}</dt>
            <dd>
              <select
                value={host.protocol || 'ssh'}
                onchange={async (e) => {
                  const proto = (e.currentTarget as HTMLSelectElement).value as ConnectionProtocol;
                  await saveHost({
                    id: host.id,
                    label: host.label,
                    address: host.address,
                    port: host.port,
                    username: host.username,
                    authMethod: host.authMethod,
                    tags: host.tags,
                    os: host.os,
                    protocol: proto,
                  });
                  host.protocol = proto;
                }}
                class="text-xs bg-neutral-100 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 rounded px-2 py-0.5 text-neutral-800 dark:text-neutral-200 focus:outline-none focus:border-cyan-500 font-mono"
              >
                <option value="ssh">SSH / SFTP</option>
                <option value="ftp">FTP</option>
                <option value="ftps">FTPS</option>
                <option value="webdav">WebDAV</option>
                <option value="s3">S3</option>
              </select>
            </dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.address')}</dt>
            <dd class="font-mono text-neutral-800 dark:text-neutral-200 truncate">{host.address}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.port')}</dt>
            <dd class="font-mono text-neutral-800 dark:text-neutral-200">{host.port}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.username')}</dt>
            <dd class="font-mono text-neutral-800 dark:text-neutral-200 truncate">{host.username}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.authentication')}</dt>
            <dd class="text-neutral-800 dark:text-neutral-200 truncate text-right">{authLabel}</dd>
          </div>
          <div class="flex justify-between gap-3">
            <dt class="text-neutral-500 dark:text-neutral-400">{t('hostDetail.credentials')}</dt>
            <dd class={host.hasSecret ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'}>
              {host.hasSecret ? t('hostDetail.encryptedStored') : t('hostDetail.notSet')}
            </dd>
          </div>
        </dl>
      </section>

      {#if host.tags.length > 0}
        <section class="space-y-2">
          <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500">{t('hostDetail.tags')}</h3>
          <div class="flex flex-wrap gap-1">
            {#each host.tags as tag}
              <span class="px-2 py-0.5 rounded text-[10px] bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 border border-neutral-200 dark:border-neutral-700">{tag}</span>
            {/each}
          </div>
        </section>
      {/if}

      <section class="space-y-2">
        <div class="flex items-center justify-between">
          <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500">
            {t('hostDetail.recentActivity')}
          </h3>
          <a href="/command-logs" class="text-[11px] text-sky-600 dark:text-sky-400 hover:underline">
            {t('hostDetail.allLogs')}
          </a>
        </div>

        {#if isLoadingLogs}
          <p class="text-xs text-neutral-500">{t('hostDetail.loadingLogs')}</p>
        {:else if logError}
          <p class="text-xs text-amber-600 dark:text-amber-400">{t('hostDetail.failedLogs', { error: logError })}</p>
        {:else if logs.length === 0}
          <p class="text-xs text-neutral-500">{t('hostDetail.noActivity')}</p>
        {:else}
          <ul class="space-y-1.5">
            {#each logs as log (log.id)}
              <li class="p-2 rounded-lg bg-neutral-50 dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[10px] font-semibold uppercase tracking-wide text-sky-600 dark:text-sky-400">
                    {log.event_type}
                  </span>
                  <span class="text-[10px] text-neutral-500 shrink-0">{formatTime(log.timestamp)}</span>
                </div>
                <p class="mt-1 text-[11px] font-mono text-neutral-700 dark:text-neutral-300 break-all">
                  {log.details}
                </p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>

    <footer class="p-4 border-t border-neutral-200 dark:border-neutral-800 flex items-center gap-2 shrink-0">
      <button
        onclick={() => openSession(host.id)}
        class="flex-1 px-3 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-medium text-center transition-colors"
      >
        {t('common.connect')}
      </button>
      <a
        href="/sftp?host={host.id}"
        class="px-3 py-2 rounded-lg text-xs font-medium border border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
      >
        SFTP
      </a>
    </footer>
  </aside>
</div>
