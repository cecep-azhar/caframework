<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import Logo from './Logo.svelte';
  import { APP_VERSION, REPO_URL, WEBSITE_URL, AUTHOR_URL } from '$lib/appInfo';

  let { onClose }: { onClose: () => void } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onClose()} />

<div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
  <button
    type="button"
    class="absolute inset-0 bg-black/60 backdrop-blur-sm cursor-default"
    aria-label={t('about.closeDialog')}
    onclick={onClose}
  ></button>

  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="about-title"
    class="no-drag relative w-full max-w-sm rounded-2xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-6 text-center"
  >
    <button
      type="button"
      onclick={onClose}
      class="absolute top-3 right-3 p-1.5 rounded-lg text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
      aria-label={t('common.close')}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
    </button>

    <div class="flex justify-center mb-3">
      <Logo size={48} mode="brand" />
    </div>
    <h2 id="about-title" class="text-lg font-bold text-neutral-900 dark:text-white">CATerm</h2>
    <p class="mt-1 text-xs font-mono text-sky-600 dark:text-sky-400">v{APP_VERSION}</p>
    <p class="mt-3 text-sm text-neutral-500 dark:text-neutral-400">
      {t('about.tagline')}
    </p>

    <dl class="mt-5 text-xs text-left divide-y divide-neutral-100 dark:divide-neutral-800 border-y border-neutral-100 dark:border-neutral-800">
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">{t('about.license')}</dt>
        <dd class="text-neutral-900 dark:text-neutral-200 font-medium">MIT</dd>
      </div>
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">{t('about.website')}</dt>
        <dd>
          <a href={WEBSITE_URL} target="_blank" rel="noreferrer" class="text-sky-600 dark:text-sky-400 hover:underline font-mono">caterm.fathforce.com</a>
        </dd>
      </div>
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">{t('about.source')}</dt>
        <dd>
          <a href={REPO_URL} target="_blank" rel="noreferrer" class="text-sky-600 dark:text-sky-400 hover:underline font-mono">cecep-azhar/caterm</a>
        </dd>
      </div>
      <div class="flex justify-between py-2">
        <dt class="text-neutral-500">{t('about.builtBy')}</dt>
        <dd>
          <a href={AUTHOR_URL} target="_blank" rel="noreferrer" class="text-sky-600 dark:text-sky-400 hover:underline font-medium">Cecep Azhar</a>
        </dd>
      </div>
    </dl>

    <!-- Dedication Card -->
    <div class="mt-4 p-3 rounded-xl bg-sky-500/5 dark:bg-sky-500/10 border border-sky-500/20 dark:border-sky-500/30 text-left">
      <div class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider text-sky-600 dark:text-sky-400 mb-1">
        <svg class="w-3 h-3 text-sky-500 shrink-0" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
        </svg>
        <span>{t('about.dedication')}</span>
      </div>
      <p class="text-xs text-neutral-600 dark:text-neutral-300 italic leading-relaxed">
        {t('about.dedicationText')}
      </p>
    </div>

    <div class="mt-4 text-[11px] text-neutral-500 dark:text-neutral-400 flex flex-col items-center justify-center gap-1">
      <div>
        {t('about.copyright')} <a href={AUTHOR_URL} target="_blank" rel="noreferrer" class="text-neutral-700 dark:text-neutral-300 hover:text-sky-600 dark:hover:text-sky-400 hover:underline font-medium">Cecep Azhar</a>
      </div>
      <div class="flex items-center gap-1.5 text-neutral-400 dark:text-neutral-500">
        <span>{t('about.builtWithLove')}</span>
        <svg class="w-3 h-3 text-rose-500 fill-rose-500 inline-block shrink-0" viewBox="0 0 24 24" aria-label={t('about.love')}>
          <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
        </svg>
        <span>{t('about.fromIndonesia')}</span>
        <span class="inline-flex items-center justify-center overflow-hidden rounded-sm border border-neutral-300 dark:border-neutral-700 w-4 h-2.5 shrink-0" title="Indonesia">
          <svg viewBox="0 0 3 2" class="w-full h-full">
            <rect width="3" height="1" fill="#e70011"/>
            <rect y="1" width="3" height="1" fill="#ffffff"/>
          </svg>
        </span>
      </div>
    </div>
  </div>
</div>
