<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { SPONSOR_TIERS, SPONSORS, type SponsorTierId } from '$lib/data/sponsors';
  import { GITHUB_SPONSORS_URL, KOFI_URL } from '$lib/appInfo';

  const TIER_LABEL_KEY: Record<SponsorTierId, string> = {
    platinum: 'sponsorWall.tierPlatinum',
    gold: 'sponsorWall.tierGold',
    silver: 'sponsorWall.tierSilver',
    contributor: 'sponsorWall.tierContributor'
  };

  const bySponsorTier = (tier: SponsorTierId) => SPONSORS.filter((s) => s.tier === tier);

  function initialsOf(name: string): string {
    return name.trim().slice(0, 1).toUpperCase() || '?';
  }
</script>

<div class="p-6 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-5">
  <div class="space-y-1">
    <h3 class="text-base font-semibold text-neutral-900 dark:text-white">{t('sponsorWall.title')}</h3>
    <p class="text-xs text-neutral-500 dark:text-neutral-400">{t('sponsorWall.subtitle')}</p>
  </div>

  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
    {#each SPONSOR_TIERS as tier (tier.id)}
      {@const sponsors = bySponsorTier(tier.id)}
      <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50/60 dark:bg-neutral-950/40 space-y-3">
        <div class="flex items-center gap-2">
          <span
            class="w-2.5 h-2.5 rounded-full shrink-0"
            style="background: linear-gradient(135deg, {tier.from}, {tier.to});"
          ></span>
          <span class="text-sm font-semibold text-neutral-900 dark:text-white">{t(TIER_LABEL_KEY[tier.id])}</span>
          <span class="text-[11px] text-neutral-400 dark:text-neutral-500 ml-auto">
            {tier.minUsd ? t('sponsorWall.minAmount', { amount: tier.minUsd }) : t('sponsorWall.anyAmount')}
          </span>
        </div>

        {#if sponsors.length > 0}
          <ul class="flex flex-wrap gap-2">
            {#each sponsors as sponsor (sponsor.name)}
              <li>
                <a
                  href={sponsor.url ?? KOFI_URL}
                  target="_blank"
                  rel="noreferrer"
                  class="flex items-center gap-1.5 pl-1 pr-2.5 py-1 rounded-full bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 hover:border-sky-500/50 transition-colors text-xs font-medium text-neutral-700 dark:text-neutral-300"
                >
                  <span
                    class="w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold text-white shrink-0"
                    style="background: linear-gradient(135deg, {tier.from}, {tier.to});"
                  >
                    {initialsOf(sponsor.name)}
                  </span>
                  {sponsor.name}
                </a>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="text-xs text-neutral-400 dark:text-neutral-500 italic">{t('sponsorWall.emptyTier')}</p>
        {/if}
      </div>
    {/each}
  </div>

  <div class="flex flex-wrap items-center gap-3">
    <a
      href={KOFI_URL}
      target="_blank"
      rel="noreferrer"
      class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-[#ff5e5b] text-white text-sm font-semibold hover:opacity-90 transition-opacity"
    >
      <span aria-hidden="true">☕</span>
      {t('sponsorWall.cta')}
    </a>
    <a
      href={GITHUB_SPONSORS_URL}
      target="_blank"
      rel="noreferrer"
      class="text-xs text-neutral-500 dark:text-neutral-400 hover:underline"
    >
      {t('sponsorWall.ctaGithub')} ↗
    </a>
  </div>
</div>
