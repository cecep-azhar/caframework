<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';

  // Referral link for cecep-azhar's Hostinger affiliate program.
  const AFFILIATE_URL =
    'https://www.hostinger.com/id/cart?product=vps%3Avps_kvm_2&period=12&referral_type=cart_link&REFERRALCODE=LRBCECEPA9SV&referral_id=01a0d7e4-490f-70ec-a28f-0d0344e575de';
  const BRAND = '#673de6';
</script>

<div class="p-6 rounded-2xl bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 space-y-4">
  <div class="space-y-2">
    <span class="inline-flex items-center bg-white rounded-lg px-3 py-2 border border-neutral-200">
      <img src="/hostinger-logo.jpg" alt="Hostinger" class="h-4 w-auto" />
    </span>
    <p class="text-xs text-neutral-500 dark:text-neutral-400">{t('vpsRecommendation.tagline')}</p>
  </div>

  <p class="text-sm text-neutral-600 dark:text-neutral-300 leading-relaxed">{t('vpsRecommendation.body')}</p>

  <a
    href={AFFILIATE_URL}
    target="_blank"
    rel="noreferrer sponsored"
    class="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl text-white text-sm font-semibold hover:opacity-90 transition-opacity"
    style="background: {BRAND};"
  >
    {t('vpsRecommendation.cta')}
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3" /></svg>
  </a>

  <p class="text-[11px] text-neutral-400 dark:text-neutral-500">{t('vpsRecommendation.affiliateDisclosure')}</p>
</div>
