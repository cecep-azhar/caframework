<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import {
    cmdWindowMinimize,
    cmdWindowMaximize,
    cmdWindowClose
  } from '$lib/generated/commands';
  import { t } from '$lib/i18n/index.svelte';

  let isMaximized = $state(false);

  async function updateMaximizedState() {
    try {
      const appWindow = getCurrentWindow();
      if (appWindow) {
        isMaximized = await appWindow.isMaximized();
      }
    } catch {
      // Browser preview / fallback
    }
  }

  async function handleMinimize() {
    try {
      const appWindow = getCurrentWindow();
      if (appWindow) {
        await appWindow.minimize();
      } else {
        await cmdWindowMinimize();
      }
    } catch {
      await cmdWindowMinimize().catch(() => {});
    }
  }

  async function handleMaximize() {
    try {
      const appWindow = getCurrentWindow();
      if (appWindow) {
        await appWindow.toggleMaximize();
        isMaximized = await appWindow.isMaximized();
      } else {
        await cmdWindowMaximize();
      }
    } catch {
      await cmdWindowMaximize().catch(() => {});
    }
  }

  async function handleClose() {
    try {
      const appWindow = getCurrentWindow();
      if (appWindow) {
        await appWindow.close();
      } else {
        await cmdWindowClose();
      }
    } catch {
      await cmdWindowClose().catch(() => {});
    }
  }

  onMount(() => {
    updateMaximizedState();
  });
</script>

<div class="flex items-center gap-0.5 no-drag select-none">
  <button
    type="button"
    onclick={handleMinimize}
    class="p-1.5 rounded text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors cursor-pointer"
    title={t('lock.minimize') || 'Minimize'}
    aria-label={t('lock.minimize') || 'Minimize'}
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
    </svg>
  </button>

  <button
    type="button"
    onclick={handleMaximize}
    class="p-1.5 rounded text-neutral-400 hover:text-white hover:bg-neutral-800 transition-colors cursor-pointer"
    title={t('lock.maximize') || 'Maximize'}
    aria-label={t('lock.maximize') || 'Maximize'}
  >
    {#if isMaximized}
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <rect x="4" y="8" width="12" height="12" rx="1.5" stroke-width="2" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 4h10a2 2 0 012 2v10" />
      </svg>
    {:else}
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2" />
      </svg>
    {/if}
  </button>

  <button
    type="button"
    onclick={handleClose}
    class="p-1.5 rounded text-neutral-400 hover:text-white hover:bg-rose-600 transition-colors cursor-pointer"
    title={t('lock.close') || 'Close'}
    aria-label={t('lock.close') || 'Close'}
  >
    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
    </svg>
  </button>
</div>
