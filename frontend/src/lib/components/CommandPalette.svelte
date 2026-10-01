<script lang="ts">
  // Ctrl+K / Ctrl+Shift+P: jump to a host, an open session, a page or an action by typing.
  // Ctrl+Shift+T opens it in "hosts" mode to start a new session.
  import { goto } from '$app/navigation';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { openSession } from '$lib/nav';
  import { navItems, settingsNavItem } from '$lib/navItems';
  import { getPalette, closePalette } from '$lib/stores/commandPalette.svelte';
  import { getTabs, tabLabel, LOCAL_HOST_ID } from '$lib/stores/sessionTabs.svelte';
  import { setSelectedTabId, setShowFiles } from '$lib/stores/sessionView.svelte';
  import { isFavorite, lastUsedAt } from '$lib/stores/hostPrefs.svelte';
  import { toggleTheme } from '$lib/stores/theme.svelte';
  import { toggleAiChat } from '$lib/stores/aiChat.svelte';
  import { t } from '$lib/i18n/index.svelte';

  let { onLock }: { onLock: () => void } = $props();

  type Group = 'hosts' | 'sessions' | 'pages' | 'actions';
  interface Item {
    id: string;
    group: Group;
    label: string;
    hint?: string;
    run: () => void;
  }

  const palette = getPalette();
  let query = $state('');
  let active = $state(0);
  let hosts = $state<HostRecord[]>([]);
  let input: HTMLInputElement | undefined = $state();
  let list: HTMLUListElement | undefined = $state();

  // Fresh host list and a clean query every time it opens.
  $effect(() => {
    if (!palette.open) return;
    query = '';
    active = 0;
    listHosts()
      .then((all) => (hosts = all))
      .catch(() => (hosts = []));
    requestAnimationFrame(() => input?.focus());
  });

  function hostRank(host: HostRecord): number {
    return (isFavorite(host.id) ? 1e15 : 0) + (lastUsedAt(host.id) ?? 0);
  }

  function showTab(id: string) {
    setSelectedTabId(id);
    void goto('/session');
  }

  const items = $derived.by((): Item[] => {
    const result: Item[] = [
      { id: 'local', group: 'hosts', label: t('palette.localTerminal'), hint: 'localhost', run: () => void openSession(LOCAL_HOST_ID) },
      ...[...hosts]
        .sort((a, b) => hostRank(b) - hostRank(a))
        .map((host) => ({
          id: `host:${host.id}`,
          group: 'hosts' as const,
          label: host.label,
          hint: `${host.username}@${host.address}${host.port && host.port !== 22 ? `:${host.port}` : ''}`,
          run: () => void openSession(host.id)
        }))
    ];
    if (palette.mode === 'hosts') return result;

    for (const tab of getTabs()) {
      result.push({ id: `tab:${tab.id}`, group: 'sessions', label: tabLabel(tab), hint: tab.host.address, run: () => showTab(tab.id) });
    }
    for (const page of [...navItems(), settingsNavItem()]) {
      result.push({ id: `page:${page.href}`, group: 'pages', label: page.label, run: () => void goto(page.href) });
    }
    result.push(
      { id: 'action:theme', group: 'actions', label: t('palette.toggleTheme'), run: toggleTheme },
      {
        id: 'action:ai',
        group: 'actions',
        label: t('palette.toggleAi'),
        run: () => {
          setShowFiles(false);
          toggleAiChat();
        }
      },
      { id: 'action:lock', group: 'actions', label: t('palette.lockVault'), run: onLock }
    );
    return result;
  });

  /** Every word of the query must appear in the label or hint; label-prefix matches rank first. */
  const filtered = $derived.by(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length === 0) return items;
    return items
      .filter((item) => {
        const haystack = `${item.label} ${item.hint ?? ''}`.toLowerCase();
        return words.every((word) => haystack.includes(word));
      })
      .sort((a, b) => Number(b.label.toLowerCase().startsWith(words[0])) - Number(a.label.toLowerCase().startsWith(words[0])));
  });

  $effect(() => {
    // Keep the highlight on a real row as the list shrinks.
    if (active >= filtered.length) active = Math.max(0, filtered.length - 1);
  });

  function choose(item: Item | undefined) {
    if (!item) return;
    closePalette();
    item.run();
  }

  function move(step: number) {
    if (filtered.length === 0) return;
    active = (active + step + filtered.length) % filtered.length;
    requestAnimationFrame(() => list?.querySelector(`[data-index="${active}"]`)?.scrollIntoView({ block: 'nearest' }));
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      move(1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      move(-1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      choose(filtered[active]);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      closePalette();
    }
  }

  const GROUP_LABEL: Record<Group, string> = {
    hosts: 'palette.groupHosts',
    sessions: 'palette.groupSessions',
    pages: 'palette.groupPages',
    actions: 'palette.groupActions'
  };
</script>

{#if palette.open}
  <div
    class="fixed inset-0 z-[300] bg-black/30 dark:bg-black/60 backdrop-blur-[2px] flex items-start justify-center px-4 pt-[12vh]"
    role="presentation"
    onpointerdown={(e) => {
      if (e.target === e.currentTarget) closePalette();
    }}
  >
    <div
      class="w-full max-w-xl rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl overflow-hidden"
      role="dialog"
      aria-modal="true"
      aria-label={t('palette.title')}
    >
      <div class="flex items-center gap-2 px-3 border-b border-neutral-200 dark:border-neutral-800">
        <svg class="w-4 h-4 text-neutral-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          bind:this={input}
          bind:value={query}
          onkeydown={onKeydown}
          oninput={() => (active = 0)}
          placeholder={palette.mode === 'hosts' ? t('palette.placeholderHosts') : t('palette.placeholder')}
          aria-label={t('palette.title')}
          aria-controls="palette-list"
          aria-activedescendant={filtered[active] ? `palette-item-${active}` : undefined}
          class="flex-1 min-w-0 py-3 bg-transparent text-sm text-neutral-900 dark:text-white placeholder:text-neutral-400 focus:outline-none"
        />
        <kbd class="px-1.5 py-0.5 rounded border border-neutral-200 dark:border-neutral-700 text-[10px] font-mono text-neutral-500">Esc</kbd>
      </div>

      <ul bind:this={list} id="palette-list" role="listbox" class="max-h-[50vh] overflow-y-auto py-1">
        {#each filtered as item, i (item.id)}
          {#if i === 0 || filtered[i - 1].group !== item.group}
            <li role="presentation" class="px-3 pt-2 pb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-500">{t(GROUP_LABEL[item.group])}</li>
          {/if}
          <li
            id="palette-item-{i}"
            data-index={i}
            role="option"
            aria-selected={i === active}
            onpointermove={() => (active = i)}
            onclick={() => choose(item)}
            onkeydown={() => {}}
            class="mx-1 px-2.5 py-1.5 rounded-md flex items-center justify-between gap-3 cursor-pointer text-sm {i === active
              ? 'bg-sky-600 text-white'
              : 'text-neutral-800 dark:text-neutral-200'}"
          >
            <span class="truncate">{item.label}</span>
            {#if item.hint}
              <span class="truncate text-xs font-mono {i === active ? 'text-sky-100' : 'text-neutral-500'}">{item.hint}</span>
            {/if}
          </li>
        {:else}
          <li class="px-4 py-6 text-center text-sm text-neutral-500">{t('palette.empty')}</li>
        {/each}
      </ul>
    </div>
  </div>
{/if}
