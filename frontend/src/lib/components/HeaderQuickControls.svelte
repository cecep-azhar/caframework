<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getTheme, setTheme } from '$lib/stores/theme.svelte';
  import LanguageSwitcher from './LanguageSwitcher.svelte';

  interface Props {
    layout: number;
    showFiles: boolean;
    aiOpen: boolean;
    sessionCount: number;
    splitOptions: Array<{ value: number; title: string; minTabs: number; path?: string }>;
    onSetLayout: (layout: number) => void;
    onToggleFiles: () => void;
    onToggleAi: () => void;
  }

  let {
    layout,
    showFiles,
    aiOpen,
    sessionCount,
    splitOptions,
    onSetLayout,
    onToggleFiles,
    onToggleAi
  }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef: HTMLDivElement | null = $state(null);

  const theme = getTheme();
  const isDark = $derived(theme.name === 'dark');

  function handleOutside(e: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      window.addEventListener('click', handleOutside);
      return () => window.removeEventListener('click', handleOutside);
    }
  });
</script>

<div class="relative no-drag" bind:this={dropdownRef}>
  <button
    type="button"
    onclick={() => (isOpen = !isOpen)}
    class="p-1.5 rounded-md transition-colors flex items-center justify-center {isOpen ? 'bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-white' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800'}"
    title={t('shell.preferencesTitle') || 'Preferences & Display Options'}
    aria-label="Preferences & Display Options"
    aria-expanded={isOpen}
  >
    <!-- Modern Sliders / Control Hub SVG Icon -->
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
    </svg>
  </button>

  {#if isOpen}
    <div
      class="absolute right-0 top-full mt-1.5 z-50 w-64 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl shadow-xl overflow-hidden animate-in fade-in zoom-in-95 duration-100 select-none p-3 space-y-3.5 text-xs"
      role="menu"
    >
      <!-- Split View Layouts (if sessions > 1) -->
      {#if sessionCount > 1}
        <div>
          <span class="block text-[10px] font-semibold uppercase tracking-wider text-neutral-400 dark:text-neutral-500 mb-1.5">
            {t('shell.splitLayout') || 'Global Split Layout'}
          </span>
          <div class="grid grid-cols-4 gap-1 p-1 bg-neutral-100 dark:bg-neutral-950 rounded-lg border border-neutral-200 dark:border-neutral-800">
            {#each splitOptions as option}
              <button
                type="button"
                disabled={sessionCount < option.minTabs}
                onclick={() => { onSetLayout(option.value); isOpen = false; }}
                class="py-1.5 flex flex-col items-center justify-center rounded transition-colors disabled:opacity-30 {layout === option.value ? 'bg-sky-600 text-white shadow-xs' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200 dark:hover:bg-neutral-800'}"
                title={option.title}
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  {#if option.value === 1}
                    <rect x="3" y="3" width="18" height="18" rx="2" stroke-width="2"></rect>
                  {:else}
                    <path stroke-width="2" d={option.path}></path>
                  {/if}
                </svg>
                <span class="text-[9px] mt-0.5">{option.value === 1 ? '1' : option.value === 2 ? '2H' : option.value === 3 ? '2V' : '4G'}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Quick Panes (Files SFTP & AI Assistant) -->
      <div>
        <span class="block text-[10px] font-semibold uppercase tracking-wider text-neutral-400 dark:text-neutral-500 mb-1.5">
          {t('shell.panels') || 'Quick Panels'}
        </span>
        <div class="grid grid-cols-2 gap-2">
          {#if sessionCount > 0}
            <button
              type="button"
              onclick={() => { onToggleFiles(); isOpen = false; }}
              class="flex items-center gap-2 p-2 rounded-lg border transition-colors {showFiles ? 'bg-sky-500/15 border-sky-500/40 text-sky-600 dark:text-sky-400' : 'bg-neutral-50 dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60'}"
            >
              <svg class="w-4 h-4 shrink-0 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"></path>
              </svg>
              <span class="font-medium text-xs truncate">{showFiles ? (t('shell.hideFiles') || 'Files: ON') : (t('shell.showFiles') || 'Files')}</span>
            </button>
          {/if}

          <button
            type="button"
            onclick={() => { onToggleAi(); isOpen = false; }}
            class="flex items-center gap-2 p-2 rounded-lg border transition-colors {aiOpen ? 'bg-rose-500/15 border-rose-500/40 text-rose-600 dark:text-rose-400' : 'bg-neutral-50 dark:bg-neutral-950 border-neutral-200 dark:border-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/60'}"
          >
            <svg class="w-3.5 h-3.5 text-rose-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
            </svg>
            <span class="font-medium text-xs truncate">{aiOpen ? 'Hana: ON' : 'Hana AI'}</span>
          </button>
        </div>
      </div>

      <!-- Language & Theme Switchers -->
      <div class="pt-2 border-t border-neutral-200 dark:border-neutral-800/80 flex items-center justify-between gap-2">
        <div class="flex items-center gap-1.5">
          <span class="text-[10px] text-neutral-500">{t('shell.language') || 'Lang'}:</span>
          <LanguageSwitcher />
        </div>

        <div class="flex items-center p-0.5 rounded-lg border border-neutral-200 dark:border-neutral-800 bg-neutral-100 dark:bg-neutral-950">
          <button
            type="button"
            onclick={() => setTheme('light')}
            class="p-1 rounded-md transition-colors {!isDark ? 'bg-white text-amber-500 shadow-sm' : 'text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
            title={t('shell.lightTheme')}
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path></svg>
          </button>
          <button
            type="button"
            onclick={() => setTheme('dark')}
            class="p-1 rounded-md transition-colors {isDark ? 'bg-neutral-800 text-white' : 'text-neutral-400 hover:text-neutral-900'}"
            title={t('shell.darkTheme')}
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path></svg>
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
