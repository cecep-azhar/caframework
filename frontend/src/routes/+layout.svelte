<script lang="ts">
  import "../app.css";
  import LockScreen from '$lib/components/LockScreen.svelte';
  import NotificationCenter from '$lib/components/NotificationCenter.svelte';
  import AiChatPanel from '$lib/components/AiChatPanel.svelte';
  import { getAiChatState, toggleAiChat, closeAiChat } from '$lib/stores/aiChat.svelte';
  import { page } from '$app/state';
  import { getTheme, initTheme } from '$lib/stores/theme.svelte';
  import { getToasts, showToast } from '$lib/stores/uiNotifications.svelte';
  import FeedbackModal from '$lib/components/FeedbackModal.svelte';
  import CrashReportModal from '$lib/components/CrashReportModal.svelte';
  import { getPendingCrashReport, type ScrubbedCrashReport } from '$lib/api/crash';
  import ProfileMenu from '$lib/components/ProfileMenu.svelte';
  import { getFeedbackPromptState } from '$lib/stores/feedbackStore.svelte';
  import { checkForUpdates } from '$lib/stores/updater.svelte';
  import { lockVault } from '$lib/api/vault';
  import { APP_VERSION, APP_NAME } from '$lib/appInfo';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import HeaderQuickControls from '$lib/components/HeaderQuickControls.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
  import WorkspaceMenu from '$lib/components/WorkspaceMenu.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { navItems as getNavItems, settingsNavItem } from '$lib/navItems';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import { getPalette } from '$lib/stores/commandPalette.svelte';
  import AmbientGlow from '$lib/components/AmbientGlow.svelte';
  import { getAmbientStore } from '$lib/stores/ambient.svelte';
  import { getPro } from '$lib/stores/pro.svelte';

  let isCollapsed = $state(false);
  let isVaultUnlocked = $state(false);
  let mobileDrawerOpen = $state(false);
  let pendingCrashReport = $state<ScrubbedCrashReport | null>(null);

  const theme = getTheme();
  const aiChat = getAiChatState();
  const feedbackPrompt = getFeedbackPromptState();
  const toasts = $derived(getToasts());
  const palette = getPalette();
  const ambient = getAmbientStore();
  const pro = getPro();

  const cardBorderClass = $derived.by(() => {
    if (!ambient.config.enabled || !ambient.config.cardGlowEnabled || !pro.isPro) {
      return 'border-t border-neutral-200 dark:border-neutral-800 md:border-l';
    }

    if (ambient.config.cardGlowStyle === 'neon-border') {
      return 'border-t border-sky-400 dark:border-sky-400 md:border-l shadow-[inset_0_0_8px_rgba(56,189,248,0.15)]';
    }

    if (ambient.config.cardGlowStyle === 'chroma-beam') {
      return 'border-t border-sky-400/50 dark:border-sky-400/40 md:border-l';
    }

    // diffused-halo
    return 'border-t border-sky-400/30 dark:border-sky-400/25 md:border-l';
  });

  const navItems = $derived(getNavItems());
  const settingsItem = $derived(settingsNavItem());

  function isActive(href: string): boolean {
    if (href === '/') return page.url.pathname === '/';
    return page.url.pathname.startsWith(href);
  }

  const currentSection = $derived(
    [...navItems, settingsItem].find((item) => isActive(item.href)) ?? navItems[0]
  );

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

  function toggleSidebar() {
    isCollapsed = !isCollapsed;
  }

  function handleAiToggle() {
    toggleAiChat();
  }

  function startDragging(e: MouseEvent) {
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
      return;
    }
    invoke('window_start_dragging').catch(() => {});
  }

  function maximizeWindow() {
    invoke('window_maximize').catch(() => {});
  }

  let { children } = $props();
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') {
      mobileDrawerOpen = false;
      closeAiChat();
    }
  }}
/>

<NotificationCenter />
{#if isVaultUnlocked}
  <CommandPalette onLock={handleLock} />
{/if}

{#if !isVaultUnlocked}
  <LockScreen onUnlocked={() => (isVaultUnlocked = true)} />
{:else}
  <!-- Shell: Unified seamless background (Title bar and sidebar share the background with no dividing line cut) -->
  <div class="flex flex-col h-screen bg-neutral-100 dark:bg-[#0e0e0e] text-neutral-800 dark:text-neutral-300 font-sans transition-colors duration-150 pb-[env(safe-area-inset-bottom,0px)] select-none">
    
    <!-- Title Bar: Frameless top header across the window -->
    <header
      class="min-h-[3rem] h-[calc(3rem+env(safe-area-inset-top,0px))] pt-[env(safe-area-inset-top,0px)] flex items-center gap-1 md:gap-2 pl-2 pr-1 md:pl-3 shrink-0 select-none cursor-default"
      data-tauri-drag-region
      onmousedown={startDragging}
      ondblclick={(e) => {
        const target = e.target as HTMLElement | null;
        if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
          maximizeWindow();
        }
      }}
    >
      <!-- Left: Mobile drawer button + current section breadcrumb -->
      <div class="flex items-center gap-1 min-w-0 flex-1" data-tauri-drag-region>
        <button
          type="button"
          onclick={() => (mobileDrawerOpen = true)}
          class="md:hidden p-1 rounded text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors shrink-0"
          title={"Buka Navigasi"}
          aria-label={"Buka Navigasi"}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>

        <div class="flex gap-1 text-xs items-center overflow-x-auto scrollbar-none min-w-0 flex-1" data-tauri-drag-region>
          <span class="px-2 py-1 text-xs font-semibold shrink-0 hidden sm:flex items-center gap-1.5 text-neutral-900 dark:text-white" data-tauri-drag-region>
            <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={currentSection.path} />
            </svg>
            {currentSection.label}
          </span>
        </div>
      </div>

      <!-- Center: Draggable blank area -->
      <div class="flex-1 h-full min-w-[20px]" data-tauri-drag-region></div>

      <!-- Right: Unified Header Controls (Live dot, Workspace, Quick Controls, Notifications, Window Controls) -->
      <div class="flex items-center gap-1.5 text-neutral-500 dark:text-neutral-400 shrink-0" data-tauri-drag-region>
        <!-- 1. Live Pulse Dot Indicator -->
        <div class="hidden sm:flex items-center px-1 py-1 shrink-0" title={"Zero-Knowledge Encrypted Vault Active"}>
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shadow-[0_0_6px_rgba(16,185,129,0.8)]"></span>
        </div>

        <!-- 2. Workspace Menu Icon -->
        <WorkspaceMenu />

        <!-- 3. Header Quick Controls (Themes, Language, Settings) -->
        <HeaderQuickControls
          layout="single"
          showFiles={false}
          aiOpen={aiChat.open}
          sessionCount={1}
          splitOptions={[]}
          onSetLayout={() => {}}
          onToggleFiles={() => {}}
          onToggleAi={handleAiToggle}
        />

        <!-- 4. Notification Bell -->
        <button
          type="button"
          onclick={() => showToast('Tidak ada notifikasi baru', 'info')}
          class="p-1.5 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-200/70 dark:hover:bg-neutral-800 rounded-md transition-colors relative"
          title={"Notifikasi"}
          aria-label={"Notifikasi"}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
        </button>

        <!-- 5. Window Controls -->
        <div class="pl-1 border-l border-neutral-200 dark:border-neutral-800">
          <WindowControls />
        </div>
      </div>
    </header>

    <!-- App Body: Seamless Sidebar + Raised Main Content Panel -->
    <div class="flex-1 flex overflow-hidden">
      
      <!-- Desktop Sidebar (Seamlessly shares the background, 0px vertical jump) -->
      <aside
        class="hidden md:flex flex-col shrink-0 will-change-[width] {isCollapsed ? 'w-16' : 'w-60'}"
        aria-label={"Sidebar Navigasi"}
      >
        <!-- Brand + collapse header (Fixed h-14 container height) -->
        <div class="h-14 flex items-center shrink-0 {isCollapsed ? 'justify-center px-2' : 'justify-between pl-4 pr-3'}">
          {#if isCollapsed}
            <button
              type="button"
              onclick={toggleSidebar}
              title={"Perluas Sidebar"}
              aria-label={"Perluas Sidebar"}
              class="p-1.5 rounded-lg hover:bg-neutral-200/70 dark:hover:bg-neutral-800 transition-colors"
            >
              <Logo size={22} mode="brand" />
            </button>
          {:else}
            <div class="flex items-center gap-2 min-w-0">
              <Logo size={22} mode="brand" />
              <span class="font-bold text-neutral-900 dark:text-white text-lg tracking-tight truncate">{APP_NAME}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 text-neutral-500 dark:text-neutral-400 font-mono shrink-0">v{APP_VERSION}</span>
            </div>
            <button
              type="button"
              onclick={toggleSidebar}
              title={"Ciutkan Sidebar"}
              aria-label={"Ciutkan Sidebar"}
              class="p-1 rounded-md hover:bg-neutral-200/70 dark:hover:bg-neutral-800 text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 transition-colors shrink-0"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
              </svg>
            </button>
          {/if}
        </div>

        <!-- Nav Items -->
        <nav class="flex-1 min-h-0 overflow-y-auto scrollbar-none px-2 space-y-0.5 text-sm pt-1">
          {#each navItems as item (item.href)}
            <a
              href={item.href}
              title={isCollapsed ? item.label : ''}
              aria-current={isActive(item.href) ? 'page' : undefined}
              class="relative px-2.5 py-2 rounded-lg flex items-center gap-3 transition-colors overflow-hidden {isActive(item.href) ? 'bg-neutral-200/80 dark:bg-neutral-800/80 text-neutral-900 dark:text-white font-medium' : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200/50 dark:hover:bg-neutral-800/40 hover:text-neutral-900 dark:hover:text-white'}"
            >
              {#if isActive(item.href)}
                <!-- Active marker pinned to left edge -->
                <span class="absolute -left-2 top-1.5 bottom-1.5 w-[3px] rounded-r bg-neutral-900 dark:bg-white" aria-hidden="true"></span>
              {/if}
              <div class="w-[20px] h-[20px] flex items-center justify-center shrink-0">
                <svg class="w-[18px] h-[18px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.path} />
                </svg>
              </div>
              <span class="truncate whitespace-nowrap transition-all duration-150 {isCollapsed ? 'opacity-0 w-0 pointer-events-none hidden' : 'opacity-100 min-w-0'}">
                {item.label}
              </span>
            </a>
          {/each}
        </nav>

        <!-- Profile & Settings Footer -->
        <div class="p-2 space-y-1">
          <ProfileMenu collapsed={isCollapsed} onLock={handleLock} onSignOut={() => {}} />
        </div>
      </aside>

      <!-- Raised Main Content Panel with Outer Ambient Halo Underglow -->
      <div class="flex-1 min-w-0 flex relative overflow-visible">
        <AmbientGlow defaultAccent="#64748b" />
        
        <main class="flex-1 min-w-0 flex overflow-hidden relative z-10 bg-white dark:bg-[#161616] {cardBorderClass} md:rounded-tl-xl transition-colors duration-150">
          <div class="flex-1 min-w-0 overflow-auto text-neutral-900 dark:text-neutral-100 relative px-4 py-6 md:px-10 md:py-10 bg-[linear-gradient(to_right,#0000000a_1px,transparent_1px),linear-gradient(to_bottom,#0000000a_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff08_1px,transparent_1px),linear-gradient(to_bottom,#ffffff08_1px,transparent_1px)] bg-[size:56px_56px]">
            {@render children()}
          </div>

          <!-- Floating AI Panel Overlay (Non-destructive) -->
          {#if aiChat.open}
            <AiChatPanel onClose={closeAiChat} />
          {/if}
        </main>
      </div>
    </div>

    <!-- Floating Action Button: Hana AI (Magic Sparkle Prompt Studio Icon) -->
    {#if !aiChat.open}
      <button
        type="button"
        onclick={() => handleAiToggle()}
        class="fixed bottom-5 right-5 z-40 flex items-center gap-2 px-3.5 py-2 rounded-full bg-neutral-900/90 hover:bg-neutral-800 text-neutral-100 border border-neutral-700/80 shadow-2xl shadow-black/60 hover:scale-105 active:scale-95 transition-all text-xs font-semibold cursor-pointer group"
        title={"Hana AI (CAFramework Assistant)"}
        aria-label={"Open Hana AI"}
      >
        <svg class="w-4 h-4 text-rose-500 group-hover:rotate-12 transition-transform shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
        </svg>
        <span>Hana AI</span>
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
      </button>
    {/if}

    {#if pendingCrashReport}
      <CrashReportModal
        report={pendingCrashReport}
        onClose={() => (pendingCrashReport = null)}
      />
    {/if}

    {#if feedbackPrompt.show}
      <FeedbackModal
        onClose={() => feedbackPrompt.close()}
        onSubmitted={() => feedbackPrompt.markSubmitted()}
      />
    {/if}
  </div>
{/if}
