<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getTheme, setTheme } from '$lib/stores/theme.svelte';
  import { getLocale, setLocale, LOCALES } from '$lib/i18n/index.svelte';
  import {
    getGlobalLayout,
    setGlobalLayout,
    type GlobalSplitLayout
  } from '$lib/stores/dualSplitStore.svelte';
  import { getAiChatState, toggleAiChat } from '$lib/stores/aiChat.svelte';

  let isOpen = $state(false);
  let containerRef: HTMLDivElement | null = null;

  const theme = getTheme();
  const currentLocale = $derived(getLocale());
  const currentLayout = $derived(getGlobalLayout());
  const aiChat = getAiChatState();

  const layoutOptions: Array<{ value: GlobalSplitLayout; label: string; icon: string }> = [
    {
      value: 'single',
      label: 'Single',
      icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z'
    },
    {
      value: '2-columns',
      label: '2 Columns',
      icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M12 4v16'
    },
    {
      value: '2-rows',
      label: '2 Rows',
      icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M4 12h16'
    },
    {
      value: '2x2-grid',
      label: '2x2 Grid',
      icon: 'M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z M12 4v16 M4 12h16'
    }
  ];

  function toggleDropdown() {
    isOpen = !isOpen;
  }

  function closeDropdown() {
    isOpen = false;
  }

  function handleDocumentClick(e: MouseEvent) {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleDocumentClick);
    return () => {
      document.removeEventListener('click', handleDocumentClick);
    };
  });
</script>

<div class="relative no-drag" bind:this={containerRef}>
  <!-- Quick Controls Trigger Button -->
  <button
    type="button"
    onclick={toggleDropdown}
    class="p-1.5 rounded-lg text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors cursor-pointer {isOpen ? 'bg-neutral-800 text-white' : ''}"
    title="Quick Controls & Preferences"
    aria-label="Quick Controls & Preferences"
    aria-expanded={isOpen}
  >
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
    </svg>
  </button>

  <!-- Dropdown Hub Menu -->
  {#if isOpen}
    <div
      class="absolute right-0 top-full mt-2 w-72 rounded-xl bg-neutral-900/95 border border-neutral-800 shadow-2xl backdrop-blur-md p-3 z-50 text-neutral-200 text-xs space-y-3.5 select-none"
    >
      <!-- Section 1: Global Split Layout -->
      <div>
        <div class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400 mb-1.5 flex items-center justify-between">
          <span>Global Layout</span>
          <span class="text-[10px] text-indigo-400 font-mono">{currentLayout}</span>
        </div>
        <div class="grid grid-cols-2 gap-1.5">
          {#each layoutOptions as opt}
            <button
              type="button"
              onclick={() => {
                setGlobalLayout(opt.value);
              }}
              class="flex items-center gap-2 px-2.5 py-1.5 rounded-lg border text-left transition-colors {currentLayout === opt.value
                ? 'bg-indigo-600/20 border-indigo-500/50 text-indigo-300 font-medium'
                : 'bg-neutral-800/60 border-neutral-700/50 text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'}"
            >
              <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={opt.icon} />
              </svg>
              <span class="truncate">{opt.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Section 2: Quick Panels -->
      <div class="border-t border-neutral-800 pt-3">
        <div class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400 mb-1.5">
          Quick Panels
        </div>
        <button
          type="button"
          onclick={() => {
            toggleAiChat();
            closeDropdown();
          }}
          class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg bg-neutral-800/60 border border-neutral-700/50 hover:bg-neutral-800 text-neutral-300 hover:text-white transition-colors"
        >
          <div class="flex items-center gap-2">
            <svg class="w-3.5 h-3.5 text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <span>Ask AI Assistant</span>
          </div>
          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded {aiChat.open ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' : 'bg-neutral-700/50 text-neutral-400'}">
            {aiChat.open ? 'OPEN' : 'CLOSED'}
          </span>
        </button>
      </div>

      <!-- Section 3: Preferences (Theme & Language) -->
      <div class="border-t border-neutral-800 pt-3 space-y-2.5">
        <div class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400">
          Preferences
        </div>

        <!-- Theme Segmented Control -->
        <div class="flex items-center justify-between">
          <span class="text-neutral-400">Theme</span>
          <div class="flex items-center p-0.5 rounded-lg bg-neutral-950 border border-neutral-800">
            <button
              type="button"
              onclick={() => setTheme('light')}
              class="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-colors {theme.name === 'light' ? 'bg-white text-neutral-950 shadow-sm' : 'text-neutral-400 hover:text-white'}"
            >
              <svg class="w-3 h-3 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
              <span>Light</span>
            </button>
            <button
              type="button"
              onclick={() => setTheme('dark')}
              class="flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-colors {theme.name === 'dark' ? 'bg-neutral-800 text-white shadow-sm' : 'text-neutral-400 hover:text-white'}"
            >
              <svg class="w-3 h-3 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
              <span>Dark</span>
            </button>
          </div>
        </div>

        <!-- Language Segmented Control -->
        <div class="flex items-center justify-between">
          <span class="text-neutral-400">Language</span>
          <div class="flex items-center p-0.5 rounded-lg bg-neutral-950 border border-neutral-800">
            {#each LOCALES as loc}
              <button
                type="button"
                onclick={() => setLocale(loc.code)}
                class="px-2 py-1 rounded-md text-[11px] font-bold uppercase transition-colors {currentLocale === loc.code ? 'bg-indigo-600 text-white shadow-sm' : 'text-neutral-400 hover:text-white'}"
              >
                {loc.code}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
