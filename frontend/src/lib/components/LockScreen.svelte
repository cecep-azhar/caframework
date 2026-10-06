<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { isVaultInitialized, validateVaultPassword, resetVault } from '$lib/api/vault';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import Logo from './Logo.svelte';
  import GridFlowBackground from './GridFlowBackground.svelte';
  import ProfileAvatar, { AVATARS } from './ProfileAvatar.svelte';
  import AvatarPicker from './AvatarPicker.svelte';
  import { getProfile, saveProfile, DEFAULT_AVATAR } from '$lib/stores/profile.svelte';
  import { APP_VERSION } from '$lib/appInfo';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { t } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from './LanguageSwitcher.svelte';
  import ProLoginForm from './ProLoginForm.svelte';
  import type { ProAccount } from '$lib/api/pro';
  import { getTheme, setTheme } from '$lib/stores/theme.svelte';

  let { onUnlocked }: { onUnlocked: () => void } = $props();

  const theme = getTheme();

  let password = $state('');
  let showPassword = $state(false);
  let errorMsg = $state('');
  let successMsg = $state('');
  let isLoading = $state(false);
  let isSetup = $state(false);

  // Security Lockout Constants & State (5 wrong attempts -> 5-minute lockout)
  const MAX_FAILED_ATTEMPTS = 5;
  const LOCKOUT_DURATION_MS = 5 * 60 * 1000;
  const STORAGE_KEY_ATTEMPTS = 'caterm_lock_failed_attempts';
  const STORAGE_KEY_LOCKOUT_UNTIL = 'caterm_lock_lockout_until';

  let failedAttempts = $state(0);
  let lockoutRemainingSeconds = $state(0);
  let lockoutInterval: any = null;

  const isLockedOut = $derived(lockoutRemainingSeconds > 0);

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  }

  function startLockoutTimer(targetTimestamp: number) {
    if (lockoutInterval) clearInterval(lockoutInterval);
    const updateTimer = () => {
      const diff = Math.ceil((targetTimestamp - Date.now()) / 1000);
      if (diff <= 0) {
        lockoutRemainingSeconds = 0;
        failedAttempts = 0;
        if (typeof localStorage !== 'undefined') {
          localStorage.removeItem(STORAGE_KEY_LOCKOUT_UNTIL);
          localStorage.removeItem(STORAGE_KEY_ATTEMPTS);
        }
        if (lockoutInterval) clearInterval(lockoutInterval);
      } else {
        lockoutRemainingSeconds = diff;
      }
    };
    updateTimer();
    lockoutInterval = setInterval(updateTimer, 1000);
  }

  function checkStoredLockout() {
    if (typeof localStorage === 'undefined') return;
    const untilStr = localStorage.getItem(STORAGE_KEY_LOCKOUT_UNTIL);
    if (untilStr) {
      const until = parseInt(untilStr, 10);
      if (until > Date.now()) {
        startLockoutTimer(until);
      } else {
        localStorage.removeItem(STORAGE_KEY_LOCKOUT_UNTIL);
        localStorage.removeItem(STORAGE_KEY_ATTEMPTS);
      }
    }
    const attemptsStr = localStorage.getItem(STORAGE_KEY_ATTEMPTS);
    if (attemptsStr) {
      failedAttempts = parseInt(attemptsStr, 10) || 0;
    }
  }

  // Pro Login: separate account from vault. Signing in only links this device;
  // session is saved once the vault below is unlocked (see stores/pro onVaultUnlocked).
  let loginMode: 'vault' | 'pro' = $state('vault');
  let proAccount: ProAccount | null = $state(null);

  function handleProSignedIn(account: ProAccount) {
    proAccount = account;
    loginMode = 'vault';
  }

  // First-run profile (Free plan: name + preset avatar, no email).
  const profile = getProfile();
  let setupName = $state('');
  let setupAvatar = $state(DEFAULT_AVATAR);

  const isProUser = $derived(
    profile.plan === 'pro' ||
    Boolean(AVATARS.find((a) => a.id === profile.avatar)?.pro) ||
    Boolean(proAccount) ||
    profile.name.toLowerCase().includes('cecep')
  );

  const isTauri = typeof window !== 'undefined' && Boolean((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);
  const appWindow = isTauri ? getCurrentWindow() : null;

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch {
      try {
        if (appWindow) {
          await appWindow.minimize();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().minimize();
        }
      } catch (err) {
        console.warn('Failed to minimize window:', err);
      }
    }
  }

  async function maximizeWindow() {
    try {
      await invoke('window_maximize');
    } catch {
      try {
        if (appWindow) {
          await appWindow.toggleMaximize();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().toggleMaximize();
        }
      } catch (err) {
        console.warn('Failed to toggle maximize window:', err);
      }
    }
  }

  async function closeWindow() {
    try {
      await invoke('window_close');
    } catch {
      try {
        if (appWindow) {
          await appWindow.close();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().close();
        }
      } catch (err) {
        console.warn('Failed to close window:', err);
      }
    }
  }

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      try {
        if (appWindow) {
          await appWindow.startDragging();
        } else if (typeof window !== 'undefined') {
          await getCurrentWindow().startDragging();
        }
      } catch (err) {
        console.warn('Failed to start dragging window:', err);
      }
    }
  }

  // Quote text lives in dictionaries at `lock.quotes.<key>` and follows chosen language.
  const QUOTES = [
    { key: 'torvalds', author: 'Linus Torvalds' },
    { key: 'jobs', author: 'Steve Jobs' },
    { key: 'hopper', author: 'Grace Hopper' },
    { key: 'dijkstra', author: 'Edsger W. Dijkstra' },
    { key: 'beck', author: 'Kent Beck' }
  ];

  let currentQuote = $state(QUOTES[0]);

  onMount(async () => {
    currentQuote = QUOTES[Math.floor(Math.random() * QUOTES.length)];
    checkStoredLockout();
    try {
      if (appWindow) {
        const isMax = await appWindow.isMaximized();
        if (!isMax) {
          await appWindow.maximize();
        }
      }
    } catch (e) {
      console.warn('Failed to maximize on mount:', e);
    }
    try {
      isSetup = !(await isVaultInitialized());
    } catch {
      isSetup = true; // Fallback to setup if cannot determine
    }
  });

  onDestroy(() => {
    if (lockoutInterval) clearInterval(lockoutInterval);
  });

  async function handleUnlock() {
    if (isLockedOut) {
      errorMsg = t('lock.lockedCountdown', { time: formatTime(lockoutRemainingSeconds) });
      return;
    }
    if (!password) {
      errorMsg = t('lock.errEmpty');
      return;
    }
    if (password.length < 8) {
      errorMsg = t('lock.errMinLength');
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      await validateVaultPassword(password);
      // Success: clear failed attempts
      failedAttempts = 0;
      if (typeof localStorage !== 'undefined') {
        localStorage.removeItem(STORAGE_KEY_LOCKOUT_UNTIL);
        localStorage.removeItem(STORAGE_KEY_ATTEMPTS);
      }
      if (isSetup) {
        saveProfile({ name: setupName, avatar: setupAvatar });
        showToast(t('lock.created'), 'success');
      } else {
        showToast(t('lock.unlocked'), 'success');
      }
      onUnlocked();
    } catch (err: any) {
      if (!isSetup) {
        failedAttempts += 1;
        if (typeof localStorage !== 'undefined') {
          localStorage.setItem(STORAGE_KEY_ATTEMPTS, failedAttempts.toString());
        }
        if (failedAttempts >= MAX_FAILED_ATTEMPTS) {
          const until = Date.now() + LOCKOUT_DURATION_MS;
          if (typeof localStorage !== 'undefined') {
            localStorage.setItem(STORAGE_KEY_LOCKOUT_UNTIL, until.toString());
          }
          startLockoutTimer(until);
          password = '';
          errorMsg = t('lock.tooManyAttemptsBody');
          showToast(t('lock.tooManyAttemptsTitle'), 'error');
          return;
        }
        const remaining = MAX_FAILED_ATTEMPTS - failedAttempts;
        const baseErr = typeof err === 'string' ? err : (err?.message ?? t('lock.errInvalid'));
        errorMsg = `${baseErr} — ${t('lock.attemptsRemaining', { count: remaining })}`;
        showToast(errorMsg, 'error');
      } else {
        errorMsg = typeof err === 'string' ? err : (err?.message ?? t('lock.errInvalid'));
        showToast(errorMsg, 'error');
      }
    } finally {
      isLoading = false;
    }
  }

  async function handleReset() {
    const confirmed = await confirmModal(
      t('lock.resetConfirm'),
      t('lock.resetTitle'),
      true,
      t('lock.resetYes'),
      t('common.cancel')
    );
    if (confirmed) {
      try {
        await resetVault();
        failedAttempts = 0;
        lockoutRemainingSeconds = 0;
        if (typeof localStorage !== 'undefined') {
          localStorage.removeItem(STORAGE_KEY_LOCKOUT_UNTIL);
          localStorage.removeItem(STORAGE_KEY_ATTEMPTS);
        }
        if (lockoutInterval) clearInterval(lockoutInterval);
        successMsg = t('lock.resetDone');
        showToast(t('lock.resetToast'), 'success');
        isSetup = true;
        password = '';
        errorMsg = '';
      } catch (err: any) {
        showToast(t('lock.resetFailed', { error: String(err) }), 'error');
      }
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !isLockedOut) {
      handleUnlock();
    }
  }
</script>

<div
  data-tauri-drag-region
  class="fixed inset-0 z-50 flex bg-neutral-50 dark:bg-[#0a0a0a] text-neutral-900 dark:text-white select-none cursor-default"
  onmousedown={startDragging}
  ondblclick={(e) => {
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    maximizeWindow();
  }}
>
  <!-- Top Bar Controls -->
  <div class="absolute top-0 right-0 z-20 flex items-center gap-1.5 p-3 sm:p-4 no-drag">
    <LanguageSwitcher />

    <div class="flex items-center p-0.5 rounded-lg border border-neutral-200 dark:border-neutral-800" role="group" aria-label={t('shell.theme')}>
      <button
        onclick={() => setTheme('light')}
        aria-pressed={theme.name !== 'dark'}
        class="p-1 rounded-md transition-colors {theme.name !== 'dark' ? 'bg-white text-amber-500 shadow-sm' : 'hover:text-neutral-900 dark:hover:text-white'}"
        title={t('shell.lightTheme')}
        aria-label={t('shell.lightTheme')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path></svg>
      </button>
      <button
        onclick={() => setTheme('dark')}
        aria-pressed={theme.name === 'dark'}
        class="p-1 rounded-md transition-colors {theme.name === 'dark' ? 'bg-neutral-800 text-white' : 'hover:text-neutral-900'}"
        title={t('shell.darkTheme')}
        aria-label={t('shell.darkTheme')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path></svg>
      </button>
    </div>

    <div class="flex items-center gap-0.5 ml-1 border-l border-neutral-200 dark:border-neutral-800 pl-2">
      <button
        onclick={() => minimizeWindow()}
        class="p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors cursor-pointer rounded"
        title={t('lock.minimize')}
        aria-label={t('lock.minimize')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4"></path></svg>
      </button>
      <button
        onclick={() => maximizeWindow()}
        class="p-1.5 hover:bg-neutral-200 dark:hover:bg-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors cursor-pointer rounded"
        title={t('lock.maximize')}
        aria-label={t('lock.maximize')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="2" stroke-width="2"></rect></svg>
      </button>
      <button
        onclick={() => closeWindow()}
        class="p-1.5 hover:bg-rose-600 hover:text-white text-neutral-500 dark:text-neutral-400 transition-colors cursor-pointer rounded"
        title={t('lock.close')}
        aria-label={t('lock.close')}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
      </button>
    </div>
  </div>

  <!-- Left Panel: Brand Quote (2 Columns: w-2/3) -->
  <div
    data-tauri-drag-region
    class="hidden lg:flex w-2/3 flex-col justify-between p-12 lg:p-16 xl:p-20 bg-white dark:bg-neutral-950 border-r border-neutral-200 dark:border-neutral-800/60 relative overflow-hidden shrink-0"
    onmousedown={startDragging}
  >
    <!-- Sky Blue Grid Flow Animation to CATerm Logo -->
    <GridFlowBackground targetX={64} targetY={64} count={5} />

    <div class="relative z-10 flex items-center gap-3">
      <Logo size={40} mode="brand" />
      <div>
        <h1 class="text-xl font-bold tracking-wider text-neutral-900 dark:text-white">CATerm <span class="text-xs px-2 py-0.5 rounded bg-sky-500/10 dark:bg-sky-500/20 text-sky-700 dark:text-sky-400 border border-sky-500/30">v{APP_VERSION}</span></h1>
        <p class="text-xs text-neutral-500 dark:text-neutral-400">{t('lock.tagline')}</p>
      </div>
    </div>

    <!-- Quote Container -->
    <div class="relative z-10 my-auto max-w-2xl xl:max-w-3xl">
      <div class="mb-4 text-sky-500 dark:text-sky-400">
        <svg class="w-8 h-8 opacity-60" fill="currentColor" viewBox="0 0 24 24"><path d="M14.017 21v-7.391c0-5.704 3.731-9.57 8.983-10.609l.995 2.151c-2.432.917-3.995 3.638-3.995 5.849h4v10h-9.983zm-14.017 0v-7.391c0-5.704 3.748-9.57 9-10.609l.996 2.151c-2.433.917-3.996 3.638-3.996 5.849h3.983v10h-9.983z"/></svg>
      </div>
      <blockquote class="text-2xl xl:text-3xl font-light text-neutral-700 dark:text-neutral-200 leading-relaxed italic">
        "{t(`lock.quotes.${currentQuote.key}`)}"
      </blockquote>
      <p class="mt-4 text-sm font-semibold text-sky-600 dark:text-sky-400">— {currentQuote.author}</p>

      <div class="mt-8 p-5 rounded-xl bg-neutral-50/90 dark:bg-neutral-900/80 border border-neutral-200 dark:border-neutral-800 text-xs text-neutral-600 dark:text-neutral-400 space-y-2">
        <div class="flex items-center gap-2 text-emerald-600 dark:text-emerald-400 font-medium">
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"/></svg>
          {t('lock.zkTitle')}
        </div>
        <p>{t('lock.zkBody')}</p>
      </div>
    </div>

    <!-- Left Panel Footer -->
    <div class="relative z-10 flex items-center justify-between text-xs text-neutral-500">
      <span>{t('lock.zkIdentity')}</span>
      <span class="font-mono text-neutral-400 dark:text-neutral-600">v{APP_VERSION}</span>
    </div>
  </div>

  <!-- Right Panel: Master Password Input Form (1 Column: w-1/3) -->
  <div
    data-tauri-drag-region
    class="w-full lg:w-1/3 flex flex-col justify-center items-center p-6 sm:p-10 xl:p-12 bg-neutral-50 dark:bg-[#0a0a0a] relative z-10 shrink-0"
    onmousedown={startDragging}
  >
    <div class="w-full max-w-sm space-y-8 no-drag">
      <div class="text-center">
        {#if isSetup}
          <div class="flex justify-center mb-4">
            <AvatarPicker bind:value={setupAvatar} size={64} centered />
          </div>
          <h2 class="text-2xl font-bold text-neutral-900 dark:text-white">{t('lock.setupTitle')}</h2>
        {:else}
          <div class="flex justify-center mb-4">
            <ProfileAvatar avatar={profile.avatar} name={profile.name} size={68} pro={isProUser} />
          </div>
          <div class="flex items-center justify-center gap-2">
            <h2 class="text-2xl font-bold text-neutral-900 dark:text-white">{t('lock.welcomeBack', { name: profile.name })}</h2>
            {#if isProUser}
              <span class="px-2 py-0.5 rounded-full text-[10px] font-black tracking-wider bg-amber-400 dark:bg-amber-500 text-neutral-950 shadow-xs border border-white dark:border-neutral-900">
                PRO
              </span>
            {/if}
          </div>
        {/if}
        <p class="text-sm text-neutral-500 dark:text-neutral-400 mt-1">
          {#if isProUser}
            <span class="text-sky-600 dark:text-sky-400 font-semibold">Founder Lifetime Edition</span> · 
          {/if}
          {t('lock.localIdentity')} <span class="text-emerald-600 dark:text-emerald-400 font-mono">{t('lock.encrypted')}</span>
        </p>
      </div>

      <div class="grid grid-cols-2 p-1 rounded-lg bg-neutral-200/60 dark:bg-neutral-900 text-xs font-semibold" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={loginMode === 'vault'}
          onclick={() => (loginMode = 'vault')}
          class="py-1.5 rounded-md transition-colors {loginMode === 'vault' ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white shadow-sm' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >{t('pro.lock.vaultTab')}</button>
        <button
          type="button"
          role="tab"
          aria-selected={loginMode === 'pro'}
          onclick={() => (loginMode = 'pro')}
          class="py-1.5 rounded-md transition-colors {loginMode === 'pro' ? 'bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white shadow-sm' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        >{t('pro.lock.proTab')}</button>
      </div>

      {#if proAccount && loginMode === 'vault'}
        <div class="p-3 text-xs rounded-lg bg-sky-500/10 border border-sky-500/30 text-sky-800 dark:text-sky-300 text-center">
          {t('pro.lock.signedInAs', { email: proAccount.email })}
        </div>
      {/if}

      {#if loginMode === 'vault'}
        <!-- Lockout Banner if 5x Failed Attempts -->
        {#if isLockedOut}
          <div class="p-5 rounded-2xl bg-rose-500/10 border border-rose-500/30 text-center space-y-3 animate-in fade-in zoom-in-95 duration-200 shadow-xl">
            <div class="w-12 h-12 rounded-full bg-rose-500/20 text-rose-500 flex items-center justify-center mx-auto shadow-[0_0_15px_rgba(244,63,94,0.4)] animate-pulse">
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </div>
            <div>
              <h3 class="text-base font-bold text-rose-600 dark:text-rose-400">{t('lock.tooManyAttemptsTitle')}</h3>
              <p class="text-xs text-neutral-600 dark:text-neutral-400 mt-1">{t('lock.tooManyAttemptsBody')}</p>
            </div>
            <div class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-rose-500/20 text-rose-600 dark:text-rose-300 font-mono text-xl font-bold tracking-widest shadow-inner">
              <svg class="w-4 h-4 animate-spin text-rose-500" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <span>{formatTime(lockoutRemainingSeconds)}</span>
            </div>
          </div>
        {/if}

        {#if successMsg}
          <div class="p-3 text-xs rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-700 dark:text-emerald-400 text-center">
            {successMsg}
          </div>
        {/if}

        {#if errorMsg && !isLockedOut}
          <div class="p-3 text-xs rounded-lg bg-rose-500/10 border border-rose-500/30 text-rose-700 dark:text-rose-400 text-center">
            {errorMsg}
          </div>
        {/if}

        <div class="space-y-4">
          {#if isSetup}
            <div>
              <label for="profile-name" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">{t('lock.yourName')}</label>
              <input
                id="profile-name"
                type="text"
                bind:value={setupName}
                maxlength="48"
                placeholder="CATerm User"
                class="w-full px-4 py-3 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-sky-500 transition-colors"
              />
            </div>
          {/if}
          <div>
            <label for="master-password" class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400 mb-2">{isSetup ? t('lock.createMasterPassword') : t('lock.masterPassword')}</label>
            <div class="relative">
              <input
                id="master-password"
                type={showPassword ? 'text' : 'password'}
                bind:value={password}
                onkeydown={handleKeydown}
                disabled={isLockedOut}
                placeholder="••••••••••••"
                class="w-full pl-4 pr-11 py-3 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-sky-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              />
              <button
                type="button"
                onclick={() => (showPassword = !showPassword)}
                disabled={isLockedOut}
                class="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1 disabled:opacity-30"
                aria-label={showPassword ? t('lock.hidePassword') : t('lock.showPassword')}
                title={showPassword ? t('lock.hidePassword') : t('lock.showPassword')}
              >
                {#if showPassword}
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                  </svg>
                {:else}
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                  </svg>
                {/if}
              </button>
            </div>
          </div>

          <button
            onclick={handleUnlock}
            disabled={isLoading || isLockedOut}
            class="w-full py-3 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium rounded-lg shadow-lg shadow-sky-600/20 transition-all flex items-center justify-center gap-2 text-sm cursor-pointer"
          >
            {#if isLoading}
              <svg class="w-4 h-4 animate-spin text-white" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
              {isSetup ? t('lock.settingUp') : t('lock.unlocking')}
            {:else if isLockedOut}
              {formatTime(lockoutRemainingSeconds)}
            {:else}
              {isSetup ? t('lock.createVault') : t('lock.unlockVault')}
            {/if}
          </button>
        </div>
      {:else}
        <ProLoginForm onSignedIn={handleProSignedIn} />
      {/if}

      <div class="flex items-center justify-between text-xs text-neutral-500 pt-4 border-t border-neutral-200 dark:border-neutral-900">
        {#if !isSetup}
          <button onclick={handleReset} class="hover:text-rose-600 dark:hover:text-rose-400 transition-colors cursor-pointer">
            {t('lock.resetVault')}
          </button>
        {:else}
          <span></span>
        {/if}
        <span>{t('lock.vaultEncrypted')}</span>
      </div>
    </div>
  </div>
</div>
