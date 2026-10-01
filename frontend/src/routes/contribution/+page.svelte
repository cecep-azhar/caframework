<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import SponsorWall from '$lib/components/SponsorWall.svelte';
  import VpsRecommendation from '$lib/components/VpsRecommendation.svelte';
  import { getProfile } from '$lib/stores/profile.svelte';
  import { KOFI_URL, PAYPAL_URL } from '$lib/appInfo';

  const profile = getProfile();
  let copied = $state(false);

  function copyKofi() {
    navigator.clipboard.writeText(KOFI_URL);
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }
</script>

<div class="max-w-4xl mx-auto space-y-8 pb-12">
  <PageHeader
    icon={['M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z']}
    accent="rose"
    title={t('contribution.title')}
    subtitle={t('contribution.subtitle')}
  />

  <!-- Main Support Banner -->
  <div class="p-6 rounded-2xl bg-gradient-to-br from-sky-50 to-white dark:from-neutral-900 dark:to-neutral-950 border border-neutral-200 dark:border-neutral-800 shadow-xl space-y-6 text-neutral-900 dark:text-white">
    <div class="space-y-2">
      <span class="text-xs uppercase tracking-wider font-semibold text-sky-600 dark:text-sky-400">{t('contribution.directDonation')}</span>
      <h2 class="text-xl font-bold">{t('contribution.heading')}</h2>
      <p class="text-neutral-600 dark:text-neutral-300 text-sm leading-relaxed">
        {t('contribution.body')}
      </p>
    </div>

    <!-- Ko-fi (primary) + PayPal (secondary) -->
    <div class="flex flex-col sm:flex-row items-start sm:items-center gap-4 pt-2">
      <a
        href={KOFI_URL}
        target="_blank"
        rel="noopener"
        class="inline-flex items-center gap-2.5 px-6 py-3 bg-[#ff5e5b] hover:bg-[#e54d4a] text-white font-semibold rounded-xl text-sm shadow-lg shadow-rose-900/30 transition-all hover:scale-[1.02]"
      >
        <span aria-hidden="true">☕</span>
        <span>{t('contribution.donateVia')}</span>
      </a>

      <button
        onclick={copyKofi}
        class="px-4 py-3 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-xl text-sm font-medium border border-neutral-300 dark:border-neutral-700 transition-colors"
      >
        {copied ? t('contribution.copiedLink') : t('contribution.copyUrl')}
      </button>

      <a
        href={PAYPAL_URL}
        target="_blank"
        rel="noopener"
        class="text-sm text-sky-700 dark:text-sky-400 hover:underline"
      >
        {t('contribution.donatePaypal')} ↗
      </a>
    </div>
  </div>

  {#if profile.plan !== 'pro'}
    <!-- Sponsors + VPS recommendation: hidden once the user is Pro, no ads/promo clutter for paying users. -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <SponsorWall />
      <VpsRecommendation />
    </div>
  {/if}

  <!-- Other Contribution Ways -->
  <div class="space-y-4 pt-2">
    <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('contribution.otherWays')}</h3>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <a
        href="https://github.com/cecep-azhar/caframework"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-amber-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group"
      >
        <div class="text-amber-500 flex items-center justify-between">
          <svg class="w-5 h-5 fill-current" viewBox="0 0 24 24"><path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-amber-400 font-mono flex items-center gap-1">{t('common.visit')} ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">{t('contribution.starTitle')}</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">{t('contribution.starBody')}</p>
      </a>

      <a
        href="https://github.com/cecep-azhar/caframework/issues"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-sky-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group"
      >
        <div class="text-sky-500 flex items-center justify-between">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-sky-400 font-mono flex items-center gap-1">{t('common.open')} ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">{t('contribution.issuesTitle')}</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">{t('contribution.issuesBody')}</p>
      </a>

      <div class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2">
        <div class="text-emerald-500">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"/></svg>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">{t('contribution.shareTitle')}</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">{t('contribution.shareBody')}</p>
      </div>

      <a
        href="https://caframework.fathforce.com"
        target="_blank"
        rel="noreferrer"
        class="p-5 rounded-xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-2 hover:border-violet-500/50 hover:bg-neutral-50 dark:hover:bg-neutral-850 transition-all block group md:col-span-3"
      >
        <div class="text-violet-500 flex items-center justify-between">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/></svg>
          <span class="text-xs text-neutral-400 group-hover:text-violet-400 font-mono flex items-center gap-1">{t('common.visit')} ↗</span>
        </div>
        <h4 class="font-bold text-neutral-900 dark:text-white text-sm">{t('contribution.websiteTitle')}</h4>
        <p class="text-xs text-neutral-500 dark:text-neutral-400 leading-relaxed">{t('contribution.websiteBody')}</p>
      </a>
    </div>
  </div>
</div>
