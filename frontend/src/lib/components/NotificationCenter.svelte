<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getToasts, removeToast, getConfirmDialog, resolveConfirm } from '$lib/stores/uiNotifications.svelte';
  import UpdateToast from './UpdateToast.svelte';

  const toasts = $derived(getToasts());
  const confirm = $derived(getConfirmDialog());
</script>

<!-- Toast Container (Bottom-Right) -->
<div class="fixed bottom-5 right-5 z-[9999] flex flex-col gap-2 max-w-sm pointer-events-none">
  {#each toasts as toast (toast.id)}
    <div
      class="pointer-events-auto p-4 rounded-xl shadow-2xl border text-sm flex items-start gap-3 backdrop-blur-md transition-all duration-300 transform translate-y-0
      {toast.type === 'success'
        ? 'bg-white/95 dark:bg-neutral-900/95 border-emerald-500/40'
        : toast.type === 'error'
          ? 'bg-white/95 dark:bg-neutral-900/95 border-rose-500/40'
          : 'bg-white/95 dark:bg-neutral-900/95 border-sky-500/40'}"
    >
      <div class="shrink-0 mt-0.5">
        {#if toast.type === 'success'}
          <svg class="w-4 h-4 text-emerald-600 dark:text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
        {:else if toast.type === 'error'}
          <svg class="w-4 h-4 text-rose-600 dark:text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        {:else}
          <svg class="w-4 h-4 text-sky-600 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
        {/if}
      </div>
      <div class="flex-1 text-xs leading-relaxed text-neutral-700 dark:text-neutral-200">
        {toast.message}
      </div>
      <button
        onclick={() => removeToast(toast.id)}
        class="shrink-0 text-neutral-400 hover:text-neutral-700 dark:text-neutral-500 dark:hover:text-neutral-300 p-0.5 rounded transition-colors"
        aria-label={t('notify.closeNotification')}
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
      </button>
    </div>
  {/each}
  <!-- Last in the column so transient toasts stack above it instead of covering it. -->
  <UpdateToast />
</div>

<!-- Global Confirm Dialog Modal -->
{#if confirm.isOpen}
  <div class="fixed inset-0 z-[99999] bg-black/40 dark:bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-6 max-w-md w-full shadow-2xl space-y-4 animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 {confirm.danger ? 'bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-500/20' : 'bg-sky-500/10 text-sky-600 dark:text-sky-400 border border-sky-500/20'}">
          {#if confirm.danger}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
          {:else}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          {/if}
        </div>
        <div>
          <h3 class="font-bold text-neutral-900 dark:text-white text-base">{confirm.title}</h3>
        </div>
      </div>

      <p class="text-xs text-neutral-600 dark:text-neutral-300 leading-relaxed pl-13">
        {confirm.message}
      </p>

      <div class="flex justify-end gap-2 pt-3 border-t border-neutral-200 dark:border-neutral-800">
        <button
          onclick={() => resolveConfirm(false)}
          class="px-4 py-2 bg-neutral-100 hover:bg-neutral-200 text-neutral-700 dark:bg-neutral-800 dark:hover:bg-neutral-700 dark:text-neutral-300 rounded-lg text-xs font-medium transition-colors"
        >
          {confirm.cancelText}
        </button>
        <button
          onclick={() => resolveConfirm(true)}
          class="px-4 py-2 rounded-lg text-xs font-medium transition-colors {confirm.danger ? 'bg-rose-600 hover:bg-rose-500 text-white shadow-lg shadow-rose-600/20' : 'bg-sky-600 hover:bg-sky-500 text-white shadow-lg shadow-sky-600/20'}"
        >
          {confirm.confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}
