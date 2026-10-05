<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import AiChatPanel from '$lib/components/AiChatPanel.svelte';
  import { getAiChatState, toggleAiChat, closeAiChat } from '$lib/stores/aiChat.svelte';
  import { page } from '$app/state';
  import { getTheme, initTheme } from '$lib/stores/theme.svelte';
  import { getToasts } from '$lib/stores/uiNotifications.svelte';
  import FeedbackModal from '$lib/components/FeedbackModal.svelte';
  import CrashReportModal from '$lib/components/CrashReportModal.svelte';
  import { getPendingCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import ProfileMenu from '$lib/components/ProfileMenu.svelte';
  import { getFeedbackPromptState } from '$lib/stores/feedbackStore.svelte';
  import { checkForUpdates } from '$lib/stores/updater.svelte';
  import { lockVault } from '$lib/api/vault';
  import { APP_VERSION } from '$lib/appInfo';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
  import HeaderQuickControls from '$lib/components/HeaderQuickControls.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
  import { APP_CONFIG } from '$lib/generated/app';
  import { navItems as getNavItems, settingsNavItem } from '$lib/navItems';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import { getPalette, openPalette, closePalette } from '$lib/stores/commandPalette.svelte';

  let isCollapsed = $state(false);
  let isVaultUnlocked = $state(false);
  let pendingCrashReport = $state<ScrubbedCrashReport | null>(null);

  const theme = getTheme();
  const aiChat = getAiChatState();
  const feedbackPrompt = getFeedbackPromptState();
  const toasts = $derived(getToasts());
  const palette = getPalette();

  const navItems = $derived(getNavItems());
  const settingsItem = $derived(settingsNavItem());

  onMount(() => {
    initTheme();
    invoke<boolean>('is_vault_unlocked')
      .then((unlocked) => {
        isVaultUnlocked = unlocked;
      })
      .catch(() => {
        isVaultUnlocked = false;
      });

    getPendingCrashReport()
      .then((report) => {
        pendingCrashReport = report;
      })
      .catch(() => {});

    checkForUpdates().catch(() => {});
  });

  async function handleLock() {
    await lockVault();
    isVaultUnlocked = false;
  }

  let { children } = $props();
</script>

<div class="flex h-screen w-screen overflow-hidden bg-neutral-950 text-neutral-100 select-none font-sans pt-[env(safe-area-inset-top,0px)] pb-[env(safe-area-inset-bottom,0px)] pl-[env(safe-area-inset-left,0px)] pr-[env(safe-area-inset-right,0px)]">
  {#if !isVaultUnlocked}
    <LockScreen onUnlocked={() => (isVaultUnlocked = true)} />
  {:else}
    <!-- Sidebar / Navigation Drawer -->
    <aside
      class="flex flex-col bg-neutral-900/90 border-r border-neutral-800/80 shrink-0 select-none {isCollapsed ? 'w-16' : 'w-60'}"
    >
      <!-- Top Brand Header -->
      <div class="flex items-center justify-between px-3.5 py-4 border-b border-neutral-800/60">
        {#if !isCollapsed}
          <div class="flex items-center gap-2.5 overflow-hidden">
            <div class="w-7 h-7 rounded-lg bg-indigo-600 flex items-center justify-center font-bold text-white shadow-lg shadow-indigo-600/30 text-xs">
              {APP_CONFIG.name.slice(0, 2).toUpperCase()}
            </div>
            <div class="flex flex-col min-w-0">
              <span class="font-bold text-sm tracking-tight text-neutral-100 truncate">{APP_CONFIG.name}</span>
              <span class="text-[10px] text-neutral-500 font-mono">v{APP_VERSION}</span>
            </div>
          </div>
        {:else}
          <div class="w-8 h-8 mx-auto rounded-lg bg-indigo-600 flex items-center justify-center font-bold text-white shadow-lg shadow-indigo-600/30 text-xs">
            {APP_CONFIG.name.slice(0, 2).toUpperCase()}
          </div>
        {/if}

        <button
          onclick={() => (isCollapsed = !isCollapsed)}
          class="p-1.5 rounded-lg text-neutral-400 hover:text-white hover:bg-neutral-800/80 transition-colors"
          title={isCollapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            {#if isCollapsed}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
            {:else}
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
            {/if}
          </svg>
        </button>
      </div>

      <!-- Search / Command Palette trigger -->
      <div class="p-2 border-b border-neutral-800/40">
        <button
          onclick={() => openPalette('all')}
          class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl bg-neutral-950/60 border border-neutral-800/60 text-neutral-400 hover:text-white hover:border-neutral-700 transition-all text-xs"
        >
          <svg class="w-4 h-4 text-neutral-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {#if !isCollapsed}
            <span class="flex-1 text-left">{t('common.search')}</span>
            <kbd class="px-1.5 py-0.5 text-[10px] font-mono bg-neutral-800 border border-neutral-700 rounded text-neutral-400">Ctrl+K</kbd>
          {/if}
        </button>
      </div>

      <!-- Navigation Links -->
      <nav class="flex-1 py-3 px-2 space-y-1 overflow-y-auto">
        {#each navItems as item}
          {@const active = page.url.pathname === item.href}
          <a
            href={item.href}
            class="flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium transition-colors {active ? 'bg-indigo-600 text-white shadow-sm' : 'text-neutral-400 hover:text-white hover:bg-neutral-800/60'}"
            title={isCollapsed ? item.label : undefined}
          >
            <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={item.path} />
            </svg>
            {#if !isCollapsed}
              <span class="truncate">{item.label}</span>
            {/if}
          </a>
        {/each}
      </nav>

      <!-- Bottom System Controls -->
      <div class="p-2 border-t border-neutral-800/60 space-y-1">
        <a
          href={settingsItem.href}
          class="flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium transition-colors {page.url.pathname === settingsItem.href ? 'bg-neutral-800 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-800/60'}"
          title={isCollapsed ? settingsItem.label : undefined}
        >
          <svg class="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={settingsItem.path} />
          </svg>
          {#if !isCollapsed}
            <span class="truncate">{settingsItem.label}</span>
          {/if}
        </a>

        <div class="flex items-center justify-between pt-2 px-1">
          {#if !isCollapsed}
            <ProfileMenu onLock={handleLock} onSignOut={handleLock} />
            <div class="flex items-center gap-1">
              <LanguageSwitcher />
              <button
                onclick={handleLock}
                class="p-1.5 rounded-lg text-neutral-400 hover:text-rose-400 hover:bg-neutral-800/80 transition-colors"
                title="Lock Vault"
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </button>
            </div>
          {:else}
            <button
              onclick={handleLock}
              class="w-full flex justify-center p-2 rounded-lg text-neutral-400 hover:text-rose-400 hover:bg-neutral-800/80 transition-colors"
              title="Lock Vault"
            >
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </button>
          {/if}
        </div>
      </div>
    </aside>

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0 bg-neutral-950 overflow-hidden">
      <!-- Top Bar with Global Header Controls and Window Controls -->
      <header
        data-tauri-drag-region
        class="h-10 border-b border-neutral-800/80 bg-neutral-900/40 backdrop-blur px-3 flex items-center justify-between shrink-0 select-none cursor-default"
      >
        <div class="flex items-center gap-2 pointer-events-none">
          <div class="w-2 h-2 rounded-full bg-emerald-500 shadow-sm shadow-emerald-500/50"></div>
          <span class="text-xs text-neutral-400 font-medium truncate">Vault Encrypted (SQLCipher)</span>
        </div>

        <div class="flex items-center gap-1.5 no-drag">
          <!-- Ask AI Trigger -->
          <button
            type="button"
            onclick={() => toggleAiChat()}
            class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-indigo-600/10 border border-indigo-500/30 text-indigo-400 hover:bg-indigo-600/20 hover:text-indigo-300 transition-colors text-xs font-medium cursor-pointer"
            title="Ask AI Assistant"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <span class="hidden sm:inline">Ask AI</span>
          </button>

          <!-- Unified Quick Controls (Split, Panels, Theme, Language) -->
          <HeaderQuickControls />

          <!-- Window Controls (Minimize, Maximize, Close) -->
          <div class="pl-1 border-l border-neutral-800/80">
            <WindowControls />
          </div>
        </div>
      </header>

      <!-- Viewport Body -->
      <main class="flex-1 overflow-hidden relative">
        {@render children?.()}
      </main>
    </div>

    <!-- Modals and Overlays -->
    <AiChatPanel isOpen={aiChat.open} onClose={() => closeAiChat()} />
    <CommandPalette />
    <NotificationCenter />

    {#if pendingCrashReport}
      <CrashReportModal
        report={pendingCrashReport}
        onClose={() => (pendingCrashReport = null)}
      />
    {/if}

    {#if feedbackPrompt.show}
      <FeedbackModal onClose={() => feedbackPrompt.close()} />
    {/if}
  {/if}
</div>
