<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { page } from '$app/state';
  import { navItems as getNavItems } from '$lib/navItems';

  let { onOpenAll }: { onOpenAll: () => void } = $props();

  const navItems = $derived(getNavItems());
  const primaryItem = $derived(navItems[0] || { key: 'notes', label: 'Notes', href: '/', path: 'M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z' });

  function isActive(href: string): boolean {
    if (href === '/') {
      return page.url.pathname === '/';
    }
    return page.url.pathname.startsWith(href);
  }
</script>

<!-- Mobile Bottom Navigation Bar: sticky bottom, backdrop blur, safe-area padded -->
<nav
  aria-label={t('shell.bottomNav') || 'Navigasi Bawah'}
  class="md:hidden fixed bottom-0 left-0 right-0 z-30 bg-white/90 dark:bg-[#121212]/90 backdrop-blur-md border-t border-neutral-200 dark:border-neutral-800 pb-[env(safe-area-inset-bottom,0px)] transition-colors select-none"
>
  <div class="h-14 grid grid-cols-5 items-stretch px-1">
    <!-- 1. Primary Work / Notes -->
    <a
      href={primaryItem.href}
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {isActive(primaryItem.href) ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={primaryItem.path} />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{primaryItem.label}</span>
    </a>

    <!-- 2. Security / Brankas -->
    <a
      href="/settings?tab=security"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {page.url.pathname === '/settings' && page.url.searchParams.get('tab') === 'security' ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('settings.security.title') || 'Keamanan'}</span>
    </a>

    <!-- 3. AI Copilot -->
    <a
      href="/settings?tab=ai"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {page.url.pathname === '/settings' && page.url.searchParams.get('tab') === 'ai' ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">Hana AI</span>
    </a>

    <!-- 4. Pengaturan -->
    <a
      href="/settings"
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium transition-colors {page.url.pathname === '/settings' && (!page.url.searchParams.get('tab') || page.url.searchParams.get('tab') === 'profile') ? 'text-sky-600 dark:text-sky-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('nav.settings') || 'Pengaturan'}</span>
    </a>

    <!-- 5. Semua (Launcher Modal Sheet) -->
    <button
      type="button"
      onclick={onOpenAll}
      class="flex flex-col items-center justify-center gap-1 text-[10px] font-medium text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 transition-colors cursor-pointer"
      aria-label={t('shell.allMenus') || 'Buka Semua Menu'}
    >
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>
      <span class="truncate max-w-[56px] leading-none">{t('shell.allMenus') || 'Semua'}</span>
    </button>
  </div>
</nav>
