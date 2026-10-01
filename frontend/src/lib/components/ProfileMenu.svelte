<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import ProfileAvatar from './ProfileAvatar.svelte';
  import AboutModal from './AboutModal.svelte';
  import { getProfile } from '$lib/stores/profile.svelte';
  import { getFeedbackPromptState } from '$lib/stores/feedbackStore.svelte';

  let {
    collapsed = false,
    onLock,
    onSignOut,
    onNavigate
  }: {
    collapsed?: boolean;
    onLock: () => void;
    onSignOut: () => void;
    /** Called after any menu action, e.g. so the mobile drawer can close itself. */
    onNavigate?: () => void;
  } = $props();

  const profile = getProfile();
  const feedbackPrompt = getFeedbackPromptState();

  let open = $state(false);
  let showAbout = $state(false);
  let root: HTMLDivElement | undefined = $state();

  function run(action: () => void) {
    open = false;
    action();
    onNavigate?.();
  }

  function handleOutsidePointer(e: PointerEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }

  const planLabel = $derived(profile.plan === 'free' ? t('profileMenu.planFree') : t('profileMenu.planPro'));

  const itemBase = 'w-full flex items-center gap-3 px-3 py-2 text-left transition-colors';
  const itemClass = `${itemBase} text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/60`;
</script>

<svelte:window
  onpointerdown={handleOutsidePointer}
  onkeydown={(e) => {
    if (e.key === 'Escape') open = false;
  }}
/>

<div class="relative no-drag" bind:this={root}>
  {#if open}
    <div
      role="menu"
      aria-label={t('profileMenu.account')}
      class="absolute z-50 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl overflow-hidden text-sm {collapsed ? 'left-full bottom-0 ml-3 w-60' : 'bottom-full left-0 right-0 mb-2'}"
    >
      <button
        type="button"
        role="menuitem"
        onclick={() => run(() => goto('/settings?tab=profile'))}
        class="w-full flex items-center gap-3 px-3 py-3 text-left hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors"
        title={t('profileMenu.editProfile')}
      >
        <ProfileAvatar avatar={profile.avatar} name={profile.name} size={36} pro={profile.plan === 'pro'} />
        <span class="min-w-0">
          <span class="block font-semibold text-neutral-900 dark:text-white truncate">{profile.name}</span>
          <span class="block text-xs text-neutral-500 dark:text-neutral-400">{planLabel}</span>
        </span>
      </button>

      <div class="border-t border-neutral-100 dark:border-neutral-800 py-1">
        <button type="button" role="menuitem" onclick={() => run(() => goto('/settings'))} class={itemClass}>
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 2.8l7.8 4.6v9.2L12 21.2l-7.8-4.6V7.4z" /><circle cx="12" cy="12" r="3" stroke-width="1.8" /></svg>
          {t('nav.settings')}
        </button>
        <button type="button" role="menuitem" onclick={() => run(() => (showAbout = true))} class={itemClass}>
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" stroke-width="1.8" /><path stroke-linecap="round" stroke-width="1.8" d="M12 11v5M12 8h.01" /></svg>
          {t('profileMenu.about')}
        </button>
        <button type="button" role="menuitem" onclick={() => run(() => goto('/contribution'))} class={itemClass}>
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" /></svg>
          {t('nav.contribution')}
        </button>
        <button type="button" role="menuitem" onclick={() => run(() => feedbackPrompt.open())} class={itemClass}>
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 8a4 4 0 118 0v1H8zM7 9h10v5a5 5 0 01-10 0zM12 14v5M3 13h4M17 13h4M4 8l3 2M20 8l-3 2M4 19l3-2M20 19l-3-2" /></svg>
          {t('profileMenu.reportBug')}
        </button>
      </div>

      <div class="border-t border-neutral-100 dark:border-neutral-800 py-1">
        <button type="button" role="menuitem" onclick={() => run(onLock)} class={itemClass}>
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="5" y="11" width="14" height="10" rx="2" stroke-width="1.8" /><path stroke-linecap="round" stroke-width="1.8" d="M8 11V7a4 4 0 018 0v4M12 15v2" /></svg>
          {t('profileMenu.lockScreen')}
        </button>
      </div>

      <div class="border-t border-neutral-100 dark:border-neutral-800 py-1">
        <button
          type="button"
          role="menuitem"
          onclick={() => run(onSignOut)}
          class="{itemBase} text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30"
        >
          <svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15 4h3a2 2 0 012 2v12a2 2 0 01-2 2h-3M10 16l-4-4 4-4M6 12h10" /></svg>
          {t('profileMenu.signOut')}
        </button>
      </div>
    </div>
  {/if}

  <button
    type="button"
    onclick={() => (open = !open)}
    aria-haspopup="menu"
    aria-expanded={open}
    title={collapsed ? `${profile.name} · ${planLabel}` : t('profileMenu.accountMenu')}
    class="w-full flex items-center rounded-xl border transition-colors {collapsed ? 'justify-center p-1.5 border-transparent' : 'gap-2.5 p-2 border-neutral-200 dark:border-neutral-800'} {open ? 'bg-neutral-100 dark:bg-neutral-800/70' : 'hover:bg-neutral-100 dark:hover:bg-neutral-900'}"
  >
    <span class="relative shrink-0">
      <ProfileAvatar avatar={profile.avatar} name={profile.name} size={collapsed ? 30 : 34} pro={profile.plan === 'pro'} />
      <!-- Vault is unlocked whenever this menu is visible. -->
      <span class="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-emerald-500 ring-2 ring-white dark:ring-[#0e0e0e]"></span>
    </span>
    {#if !collapsed}
      <span class="min-w-0 flex-1 text-left">
        <span class="flex items-center gap-1.5">
          <span class="text-[13px] font-semibold text-neutral-900 dark:text-white truncate">{profile.name}</span>
          <span class="shrink-0 px-1.5 py-px rounded border border-neutral-300 dark:border-neutral-700 text-[9px] font-semibold tracking-wider text-neutral-500 dark:text-neutral-400">
            {planLabel.toUpperCase()}
          </span>
        </span>
        <span class="block text-[11px] font-mono text-neutral-500 dark:text-neutral-500 truncate">{t('profileMenu.vaultEncrypted')}</span>
      </span>
      <svg class="w-4 h-4 shrink-0 text-neutral-400 transition-transform {open ? '-rotate-90' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
    {/if}
  </button>
</div>

{#if showAbout}
  <AboutModal onClose={() => (showAbout = false)} />
{/if}
