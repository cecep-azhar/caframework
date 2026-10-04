<script lang="ts">
  import { Window } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  
  const appWindow = new Window('main');
  
  let isMaximized = false;
  let error = '';

  const minimize = async () => {
    try {
      if (appWindow) await appWindow.minimize();
    } catch (e) {
      console.error("Failed to minimize", e);
      error = "Failed to minimize";
    }
  };
  
  const toggleMaximize = async () => {
    try {
      if (appWindow) {
        if (await appWindow.isMaximized()) {
          await appWindow.unmaximize();
          isMaximized = false;
        } else {
          await appWindow.maximize();
          isMaximized = true;
        }
      }
    } catch (e) {
      console.error("Failed to toggle maximize", e);
      error = "Failed to maximize/unmaximize";
    }
  };
  
  const close = async () => {
    try {
      if (appWindow) await appWindow.close();
    } catch (e) {
        console.error("Failed to close", e);
        error = "Failed to close";
    }
  };

  onMount(() => {
    const checkMaximized = async () => {
      try {
        if (appWindow) {
          isMaximized = await appWindow.isMaximized();
        }
      } catch (e) {
        console.error("Failed to check if maximized", e);
      }
    };
    checkMaximized();
  });
</script>

<div data-tauri-drag-region class="titlebar">
  <div class="titlebar-left">
    {#if error}
      <span class="error-msg">{error}</span>
    {/if}
  </div>
  <div class="titlebar-right">
    <button class="titlebar-button" id="titlebar-minimize" on:click={minimize} aria-label={t('lock.minimize')}>
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="0" y="4" width="10" height="1" fill="currentColor"/>
      </svg>
    </button>
    <button class="titlebar-button" id="titlebar-maximize" on:click={toggleMaximize} aria-label={t('lock.maximize')}>
      {#if isMaximized}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="0.5" y="2.5" width="7" height="7" stroke="currentColor"/>
          <path d="M2.5 2.5V0.5H9.5V7.5H7.5" stroke="currentColor"/>
        </svg>
      {:else}
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor"/>
        </svg>
      {/if}
    </button>
    <button class="titlebar-button titlebar-close" id="titlebar-close" on:click={close} aria-label={t('lock.close')}>
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 30px;
    background: var(--bg-color, #1e1e1e);
    user-select: none;
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 9999;
    color: var(--text-color, #fff);
  }
  
  .titlebar-left {
    padding-left: 10px;
    display: flex;
    align-items: center;
    pointer-events: none; /* Allows click-through to drag region */
  }

  .titlebar-right {
    display: flex;
    height: 100%;
  }
  
  .titlebar-button {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    width: 46px;
    height: 100%;
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .titlebar-button:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  
  .titlebar-button.titlebar-close:hover {
    background: #e81123;
    color: white;
  }
  
  .error-msg {
    color: #ff5555;
    font-size: 12px;
  }
</style>
