<script lang="ts">
  import { getLocale, setLocale, LOCALES, t } from '$lib/i18n/index.svelte';

  const current = $derived(getLocale());
</script>

<!-- Same segmented-control look as the light/dark toggle next to it, so the two read as one
     family of controls rather than two different widget styles bolted together. -->
<div class="flex items-center p-0.5 rounded-lg border border-neutral-200 dark:border-neutral-800" role="group" aria-label={t('language.label')}>
  {#each LOCALES as loc (loc.code)}
    <button
      onclick={() => setLocale(loc.code)}
      aria-pressed={current === loc.code}
      class="px-1.5 py-1 rounded-md text-[10px] font-bold tracking-wide transition-colors {current === loc.code
        ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white shadow-sm'
        : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
      title={t(`language.${loc.code}`)}
      aria-label={t(`language.${loc.code}`)}
    >
      {loc.code.toUpperCase()}
    </button>
  {/each}
</div>
