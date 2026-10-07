<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';
  import { navItems as getNavItems } from '$lib/navItems';

  let {
    open = $bindable(false),
    onClose
  }: {
    open: boolean;
    onClose: () => void;
  } = $props();

  let searchQuery = $state('');

  interface MenuItem {
    id: string;
    label: string;
    href: string;
    icon: string;
    color: string;
  }

  const generatedNav = $derived(getNavItems());

  // Modules adaptable for CAFramework starter / vertical slices & settings
  const defaultModules: MenuItem[] = [
    {
      id: 'profile',
      label: 'Profil',
      href: '/settings?tab=profile',
      icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z',
      color: 'text-rose-500 bg-rose-500/10'
    },
    {
      id: 'security',
      label: 'Keamanan Vault',
      href: '/settings?tab=security',
      icon: 'M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z',
      color: 'text-amber-500 bg-amber-500/10'
    },
    {
      id: 'ai',
      label: 'Hana AI Copilot',
      href: '/settings?tab=ai',
      icon: 'M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09z',
      color: 'text-purple-500 bg-purple-500/10'
    },
    {
      id: 'backup',
      label: 'Backup & Pulihkan',
      href: '/settings?tab=backup',
      icon: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12',
      color: 'text-sky-500 bg-sky-500/10'
    },
    {
      id: 'updates',
      label: 'Pembaruan',
      href: '/settings?tab=updates',
      icon: 'M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15',
      color: 'text-emerald-500 bg-emerald-500/10'
    },
    {
      id: 'performance',
      label: 'Performa',
      href: '/settings?tab=performance',
      icon: 'M13 10V3L4 14h7v7l9-11h-7z',
      color: 'text-yellow-500 bg-yellow-500/10'
    },
    {
      id: 'shortcuts',
      label: 'Pintasan Keyboard',
      href: '/settings?tab=shortcuts',
      icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4',
      color: 'text-indigo-500 bg-indigo-500/10'
    },
    {
      id: 'ambient',
      label: 'Ambient Lighting',
      href: '/settings?tab=profile',
      icon: 'M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z',
      color: 'text-fuchsia-500 bg-fuchsia-500/10'
    },
    {
      id: 'settings',
      label: 'Semua Pengaturan',
      href: '/settings',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065zM15 12a3 3 0 11-6 0 3 3 0 016 0z',
      color: 'text-zinc-500 bg-zinc-500/10'
    }
  ];

  const allModules = $derived([
    ...generatedNav.map((n) => ({
      id: n.key,
      label: n.label,
      href: n.href,
      icon: n.path,
      color: 'text-cyan-500 bg-cyan-500/10'
    })),
    ...defaultModules
  ]);

  const filteredModules = $derived(
    searchQuery.trim() === ''
      ? allModules
      : allModules.filter((m) =>
          m.label.toLowerCase().includes(searchQuery.trim().toLowerCase())
        )
  );

  function navigateTo(href: string) {
    onClose();
    void goto(href);
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm transition-opacity md:hidden"
    onclick={onClose}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
    tabindex="-1"
    role="button"
    aria-label={t('common.close') || 'Tutup'}
  ></div>

  <!-- Slide-up Bottom Sheet -->
  <div
    class="fixed inset-x-0 bottom-0 z-50 max-h-[85vh] bg-white dark:bg-[#141414] border-t border-neutral-200 dark:border-neutral-800 rounded-t-3xl shadow-2xl flex flex-col md:hidden pb-[max(1rem,env(safe-area-inset-bottom,0px))] transition-transform transform duration-200 ease-out"
    role="dialog"
    aria-modal="true"
    aria-label={t('shell.allMenus') || 'Semua Menu Modul'}
  >
    <!-- Drag handle indicator -->
    <div class="w-full flex justify-center pt-3 pb-2 cursor-grab active:cursor-grabbing">
      <div class="w-10 h-1 rounded-full bg-neutral-300 dark:bg-neutral-700"></div>
    </div>

    <!-- Header & Search filter bar -->
    <div class="px-5 pb-3 pt-1 space-y-3 shrink-0">
      <div class="flex items-center justify-between">
        <h2 class="text-base font-bold text-neutral-900 dark:text-white tracking-tight">Semua Menu Modul</h2>
        <button
          type="button"
          onclick={onClose}
          class="p-1.5 rounded-full hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-white transition-colors cursor-pointer"
          aria-label={t('common.close') || 'Tutup'}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Search Filter Bar -->
      <div class="relative">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-neutral-400">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder={t('common.search') || 'Cari modul atau menu...'}
          class="w-full pl-9 pr-8 py-2 rounded-xl text-xs bg-neutral-100 dark:bg-neutral-800/70 border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-white placeholder-neutral-400 focus:outline-none focus:ring-2 focus:ring-sky-500/40"
        />
        {#if searchQuery}
          <button
            type="button"
            onclick={() => (searchQuery = '')}
            aria-label={t('common.clear') || 'Hapus pencarian'}
            class="absolute inset-y-0 right-0 pr-3 flex items-center text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 cursor-pointer"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        {/if}
      </div>
    </div>

    <!-- 4x4 Modules Grid Area -->
    <div class="px-5 py-2 overflow-y-auto scrollbar-none flex-1 min-h-0">
      {#if filteredModules.length === 0}
        <div class="py-10 text-center text-xs text-neutral-500">
          Modul tidak ditemukan untuk "{searchQuery}"
        </div>
      {:else}
        <div class="grid grid-cols-4 gap-3 py-1">
          {#each filteredModules as item (item.id)}
            <button
              type="button"
              onclick={() => navigateTo(item.href)}
              class="flex flex-col items-center justify-center p-2.5 rounded-2xl hover:bg-neutral-100 dark:hover:bg-neutral-800/80 active:scale-95 transition-all text-center cursor-pointer group"
            >
              <div class="w-11 h-11 rounded-2xl flex items-center justify-center mb-1.5 shadow-sm {item.color} group-hover:scale-105 transition-transform">
                <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d={item.icon} />
                </svg>
              </div>
              <span class="text-[11px] font-medium text-neutral-700 dark:text-neutral-300 group-hover:text-neutral-900 dark:group-hover:text-white line-clamp-1 break-all">
                {item.label}
              </span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
