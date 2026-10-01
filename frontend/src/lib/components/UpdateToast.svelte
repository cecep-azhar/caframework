<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getUpdater, installUpdate, dismissUpdateToast } from '$lib/stores/updater.svelte';
  import { releaseNotesUrl } from '$lib/appInfo';

  const updater = getUpdater();

  const percent = $derived(
    updater.progress.total ? Math.min(100, Math.round((updater.progress.downloaded / updater.progress.total) * 100)) : null
  );
</script>

{#if updater.showToast}
  <div
    role="status"
    class="no-drag pointer-events-auto w-[340px] max-w-[calc(100vw-2.5rem)] rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white/95 dark:bg-[#141414]/95 backdrop-blur-md shadow-2xl p-3.5"
  >
    <div class="flex items-start gap-3">
      <svg class="w-5 h-5 mt-0.5 shrink-0 text-neutral-900 dark:text-white" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path fill-rule="evenodd" d="M12 2a10 10 0 100 20 10 10 0 000-20zm-1 9a1 1 0 112 0v5a1 1 0 11-2 0v-5zm1-4a1.25 1.25 0 100 2.5A1.25 1.25 0 0012 7z" clip-rule="evenodd" />
      </svg>
      <div class="flex-1 min-w-0">
        <p class="text-sm font-semibold text-neutral-900 dark:text-white">{t('updater.available', { version: updater.version })}</p>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
          {t('updater.readyBody', { version: updater.version })}
          <a href={releaseNotesUrl(updater.version)} class="underline underline-offset-2 hover:text-neutral-900 dark:hover:text-white">{t('updater.changelog')}</a>
        </p>
      </div>
      {#if updater.status !== 'downloading'}
        <button
          type="button"
          onclick={dismissUpdateToast}
          class="p-1 -mt-1 -mr-1 rounded-md text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
          aria-label={t('updater.dismiss')}
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
        </button>
      {/if}
    </div>

    <div class="mt-3 flex items-center justify-end gap-2">
      {#if updater.status === 'downloading'}
        <div class="flex-1 h-1.5 rounded-full bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
          <div class="h-full bg-sky-500 transition-all {percent === null ? 'w-1/3 animate-pulse' : ''}" style={percent === null ? '' : `width: ${percent}%`}></div>
        </div>
        <span class="text-[11px] font-mono text-neutral-500 w-10 text-right">{percent === null ? '…' : `${percent}%`}</span>
      {:else}
        <button
          type="button"
          onclick={installUpdate}
          class="px-3 py-1.5 rounded-md text-xs font-semibold bg-neutral-900 text-white hover:bg-neutral-700 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 transition-colors"
        >
          {t('updater.installRestart')}
        </button>
      {/if}
    </div>
  </div>
{/if}
