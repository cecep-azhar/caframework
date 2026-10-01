<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { submitCrashReport, dismissCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';

  let {
    report,
    onClose
  }: {
    report: ScrubbedCrashReport;
    onClose: () => void;
  } = $props();

  let showDetails = $state(false);
  let neverAgain = $state(false);
  let isSubmitting = $state(false);
  let isDeleting = $state(false);
  let sent = $state(false);

  async function handleSend() {
    isSubmitting = true;
    try {
      await submitCrashReport(report.id);
      sent = true;
      showToast(t('crash.sentThanks'), 'success');
      // If the person also asked to never be asked again, that preference still applies even
      // though this dump is already gone via the server-accepted delete inside submit.
      if (neverAgain) {
        await dismissCrashReport(report.id, true);
      }
      onClose();
    } catch (err) {
      showToast(`${t('crash.sendFailed')} ${errorText(err)}`, 'error');
    } finally {
      isSubmitting = false;
    }
  }

  async function handleDelete() {
    isDeleting = true;
    try {
      await dismissCrashReport(report.id, neverAgain);
      onClose();
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isDeleting = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
  <button
    type="button"
    class="absolute inset-0 bg-black/40 dark:bg-black/60 backdrop-blur-sm cursor-default"
    aria-label={t('crash.closeDialog')}
    onclick={onClose}
  ></button>

  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="crash-dialog-title"
    class="no-drag relative w-full max-w-lg rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-6 text-left space-y-4 text-neutral-900 dark:text-neutral-100"
  >
    <div class="flex items-center gap-3">
      <div class="w-10 h-10 rounded-xl bg-amber-500/10 dark:bg-amber-500/20 border border-amber-500/30 flex items-center justify-center text-amber-600 dark:text-amber-400 shrink-0">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
      <div>
        <h2 id="crash-dialog-title" class="text-lg font-bold text-neutral-900 dark:text-white tracking-tight">
          {t('crash.title')}
        </h2>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">
          {t('crash.subtitle')}
        </p>
      </div>
    </div>

    <div class="p-3 bg-neutral-50 dark:bg-neutral-900/60 border border-neutral-200 dark:border-neutral-800 rounded-lg space-y-1">
      <p class="text-[11px] font-semibold text-neutral-500 dark:text-neutral-400 uppercase tracking-wider">
        {t('crash.whatHappened')}
      </p>
      <p class="text-sm text-neutral-800 dark:text-neutral-200 break-words">{report.message}</p>
      {#if report.location}
        <p class="text-xs text-neutral-500 dark:text-neutral-400">
          {t('crash.location', { location: report.location })}
        </p>
      {/if}
    </div>

    <div>
      <button
        type="button"
        onclick={() => (showDetails = !showDetails)}
        class="text-xs text-sky-600 dark:text-sky-400 hover:underline font-medium transition-colors cursor-pointer"
      >
        {showDetails ? t('crash.hideDetails') : t('crash.viewDetails')}
      </button>
      {#if showDetails}
        <pre class="mt-2 p-3 bg-neutral-950 text-neutral-300 text-[11px] leading-relaxed rounded-lg overflow-auto max-h-56">{report.previewJson}</pre>
      {/if}
    </div>

    <label class="flex items-center gap-2 text-xs text-neutral-600 dark:text-neutral-400 cursor-pointer">
      <input type="checkbox" bind:checked={neverAgain} class="rounded border-neutral-300 dark:border-neutral-700" />
      {t('crash.neverAskAgain')}
    </label>

    <div class="flex items-center justify-end gap-3 pt-2 border-t border-neutral-100 dark:border-neutral-800">
      <button
        type="button"
        onclick={handleDelete}
        disabled={isSubmitting || isDeleting}
        class="px-4 py-2 text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 disabled:opacity-50 transition-colors cursor-pointer disabled:cursor-not-allowed"
      >
        {t('crash.deleteAndClose')}
      </button>
      <button
        type="button"
        onclick={handleSend}
        disabled={isSubmitting || isDeleting || sent}
        class="px-5 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-colors flex items-center gap-2 cursor-pointer disabled:cursor-not-allowed shadow-sm"
      >
        {#if isSubmitting}
          <svg class="animate-spin w-3.5 h-3.5" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>{t('crash.sending')}</span>
        {:else}
          <span>{t('crash.send')}</span>
        {/if}
      </button>
    </div>
  </div>
</div>
