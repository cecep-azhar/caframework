<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { listHosts, saveHost, deleteHost, type HostRecord, type HostInput, type ConnectionProtocol } from '$lib/api/hosts';
  import { listGroups } from '$lib/api/groups';
  import { listKeys, type KeyRecord } from '$lib/api/keys';
  import HostDetailPanel from '$lib/components/HostDetailPanel.svelte';
  import OsIcon from '$lib/components/OsIcon.svelte';
  import { openSession } from '$lib/nav';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { isFavorite, toggleFavorite, lastUsedAt } from '$lib/stores/hostPrefs.svelte';
  import { getTabs } from '$lib/stores/sessionTabs.svelte';
  import { monitorState } from '$lib/stores/monitorStore.svelte';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let hosts = $state<HostRecord[]>([]);
  let vaultKeys = $state<KeyRecord[]>([]);
  let searchQuery = $state('');
  let isAddModalOpen = $state(false);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let editingId = $state<string | null>(null);
  let editingHadSecret = $state(false);
  let formProtocol = $state<ConnectionProtocol>('ssh');

  // Bulk selection + detail slide-over. Both buttons on the host card were previously inert
  // markup with no onclick; this is the state behind them.
  let selectedIds = $state<string[]>([]);
  let detailHost = $state<HostRecord | null>(null);

  const isSelected = (id: string) => selectedIds.includes(id);

  function toggleSelected(id: string) {
    selectedIds = isSelected(id) ? selectedIds.filter((x) => x !== id) : [...selectedIds, id];
  }

  function clearSelection() {
    selectedIds = [];
  }

  /** Opens every selected host as tabs in one session, which is what a split view needs. */
  function connectSelected() {
    if (selectedIds.length === 0) return;
    const ids = [...selectedIds];
    clearSelection();
    void openSession(ids);
  }

  async function deleteSelected() {
    const count = selectedIds.length;
    if (count === 0) return;
    const ok = await confirmModal(
      t('hosts.deleteSelected', { count }),
      t('hosts.deleteSelectedTitle'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (!ok) return;

    const failed: string[] = [];
    for (const id of selectedIds) {
      try {
        await deleteHost(id);
      } catch {
        failed.push(hosts.find((h) => h.id === id)?.label ?? id);
      }
    }
    clearSelection();
    await refreshHosts();

    if (failed.length > 0) {
      showToast(t('hosts.deleteFailedList', { names: failed.join(', ') }), 'error');
    } else {
      showToast(t('hosts.hostDeleted'), 'success');
    }
  }

  // Form State
  let formLabel = $state('');
  let formAddress = $state('');
  let formPort = $state(22);
  let formUsername = $state('root');
  let formAuthType = $state<'password' | 'key' | 'keyId'>('password');
  let formKeyPath = $state('');
  let formKeyId = $state('');
  let formSecret = $state('');
  let showFormSecret = $state(false);
  let formTags = $state('');
  let formOs = $state('');
  let unlistenOsDetected: UnlistenFn | null = null;

  onMount(async () => {
    await refreshHosts();
    try {
      unlistenOsDetected = await listen<{ hostId: string; os: string }>('host:os_detected', (event) => {
        const { hostId, os } = event.payload;
        const targetIndex = hosts.findIndex((h) => h.id === hostId);
        if (targetIndex !== -1) {
          hosts[targetIndex] = { ...hosts[targetIndex], os };
        }
        if (detailHost && detailHost.id === hostId) {
          detailHost = { ...detailHost, os };
        }
      });
    } catch (e) {
      console.error('Failed to listen for host:os_detected', e);
    }
  });

  onDestroy(() => {
    if (unlistenOsDetected) {
      unlistenOsDetected();
    }
  });

  async function refreshHosts() {
    try {
      hosts = await listHosts();
      vaultKeys = await listKeys();
      groupCount = (await listGroups()).length;
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  function openAddModal() {
    editingId = null;
    editingHadSecret = false;
    formLabel = '';
    formAddress = '';
    formPort = 22;
    formUsername = 'root';
    formProtocol = 'ssh';
    formAuthType = 'password';
    formKeyPath = '';
    formSecret = '';
    formTags = '';
    formOs = '';
    errorMsg = '';
    isAddModalOpen = true;
  }

  function openEditModal(host: HostRecord) {
    editingId = host.id;
    editingHadSecret = host.hasSecret;
    formLabel = host.label;
    formAddress = host.address;
    formPort = host.port;
    formUsername = host.username;
    formProtocol = host.protocol || 'ssh';
    formAuthType = host.authMethod.type;
    formKeyPath = host.authMethod.type === 'key' ? host.authMethod.path : '';
    formKeyId = host.authMethod.type === 'keyId' ? host.authMethod.id : '';
    formSecret = '';
    formTags = host.tags.join(', ');
    formOs = host.os || '';
    errorMsg = '';
    isAddModalOpen = true;
  }

  async function handleSaveHost() {
    if (!formLabel || !formAddress || !formUsername) {
      errorMsg = t('hosts.form.requiredFields');
      showToast(errorMsg, 'error');
      return;
    }
    isLoading = true;
    errorMsg = '';
    try {
      const input: HostInput = {
        id: editingId ?? undefined,
        label: formLabel,
        address: formAddress,
        port: Number(formPort) || 22,
        username: formUsername,
        protocol: formProtocol,
        authMethod: formAuthType === 'password' ? { type: 'password' } : (formAuthType === 'keyId' ? { type: 'keyId', id: formKeyId } : { type: 'key', path: formKeyPath }),
        tags: formTags.split(',').map(t => t.trim()).filter(Boolean),
        os: formOs ? formOs : undefined,
        // Blank = leave whatever's stored untouched (edit) or no secret at all (create).
        secret: formSecret ? formSecret : undefined
      };
      await saveHost(input);
      isAddModalOpen = false;
      showToast(editingId ? t('hosts.hostUpdated') : t('hosts.hostAdded'), 'success');
      await refreshHosts();
    } catch (err: any) {
      errorMsg = typeof err === 'string' ? err : (err?.message || t('hosts.saveFailed'));
      showToast(errorMsg, 'error');
    } finally {
      isLoading = false;
    }
  }

  async function handleDelete(id: string) {
    const confirmed = await confirmModal(t('hosts.deleteConfirm'), t('hosts.deleteTitle'), true, t('common.delete'), t('common.cancel'));
    if (confirmed) {
      try {
        await deleteHost(id);
        showToast(t('hosts.hostDeleted'), 'success');
        await refreshHosts();
      } catch (err: any) {
        showToast(err?.message || 'Failed to delete host', 'error');
      }
    }
  }

  async function handleClone(host: HostRecord) {
    try {
      const input: HostInput = {
        label: `${host.label} (Copy)`,
        address: host.address,
        port: host.port,
        username: host.username,
        protocol: host.protocol,
        authMethod: host.authMethod,
        tags: [...host.tags],
        os: host.os
      };
      await saveHost(input);
      showToast(t('hosts.hostCloned', { name: host.label }), 'success');
      await refreshHosts();
    } catch (err: any) {
      showToast(err?.message || 'Failed to clone host', 'error');
    }
  }

  // --- List: search, filters, sort ------------------------------------------------------------
  type SortKey = 'name' | 'lastUsed' | 'ip';
  let groupCount = $state(0);
  let sortBy = $state<SortKey>('name');
  let starredOnly = $state(false);
  let connectedOnly = $state(false);
  let filterOpen = $state(false);
  let openMenuId = $state<string | null>(null);
  let focusedIndex = $state(0);
  let searchInput: HTMLInputElement | undefined = $state();

  const sessionTabs = $derived(getTabs());

  function sessionCount(hostId: string): number {
    return sessionTabs.filter((t) => t.host.id === hostId).length;
  }

  function matchesSearch(h: HostRecord, q: string): boolean {
    if (!q) return true;
    return (
      h.label.toLowerCase().includes(q) ||
      h.address.toLowerCase().includes(q) ||
      h.username.toLowerCase().includes(q) ||
      h.tags.some((t) => t.toLowerCase().includes(q))
    );
  }

  const byName = (a: HostRecord, b: HostRecord) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' });
  const comparators: Record<SortKey, (a: HostRecord, b: HostRecord) => number> = {
    name: byName,
    lastUsed: (a, b) => (lastUsedAt(b.id) ?? 0) - (lastUsedAt(a.id) ?? 0) || byName(a, b),
    // `numeric` orders 10.0.0.9 before 10.0.0.10.
    ip: (a, b) => a.address.localeCompare(b.address, undefined, { numeric: true }) || byName(a, b)
  };

  // Starred hosts always lead, then the chosen order.
  let filteredHosts = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    return hosts
      .filter((h) => matchesSearch(h, q))
      .filter((h) => !starredOnly || isFavorite(h.id))
      .filter((h) => !connectedOnly || sessionCount(h.id) > 0)
      .sort((a, b) => Number(isFavorite(b.id)) - Number(isFavorite(a.id)) || comparators[sortBy](a, b));
  });

  const activeIndex = $derived(filteredHosts.length === 0 ? -1 : Math.min(focusedIndex, filteredHosts.length - 1));
  const filtersActive = $derived(starredOnly || connectedOnly);

  // --- Card status ------------------------------------------------------------------------------
  const OS_LABELS: Record<string, string> = {
    ubuntu: 'Ubuntu', debian: 'Debian', fedora: 'Fedora', redhat: 'Red Hat', centos: 'CentOS',
    rocky: 'Rocky Linux', almalinux: 'AlmaLinux', arch: 'Arch Linux', manjaro: 'Manjaro',
    alpine: 'Alpine Linux', opensuse: 'openSUSE', mint: 'Linux Mint', kali: 'Kali Linux',
    popos: 'Pop!_OS', raspberry: 'Raspberry Pi OS', amazon: 'Amazon Linux', oracle: 'Oracle Linux',
    freebsd: 'FreeBSD', openbsd: 'OpenBSD', netbsd: 'NetBSD', mikrotik: 'RouterOS', cisco: 'Cisco IOS',
    windows: 'Windows', macos: 'macOS', android: 'Android', linux: 'Linux'
  };

  /** Live metrics give the full distro string ("Debian GNU/Linux 12 (bookworm)") while connected. */
  function osName(host: HostRecord): string | null {
    const live = monitorState.metrics.find((m) => m.host_id === host.id)?.os_name;
    if (live) return live;
    if (!host.os) return null;
    return OS_LABELS[host.os] ?? host.os.charAt(0).toUpperCase() + host.os.slice(1);
  }

  function relativeTime(ts: number): string {
    const minutes = Math.floor((Date.now() - ts) / 60000);
    if (minutes < 1) return t('time.justNow');
    if (minutes < 60) return t('time.minutesAgo', { n: minutes });
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return t('time.hoursAgo', { n: hours });
    const days = Math.floor(hours / 24);
    return days < 30 ? t('time.daysAgo', { n: days }) : new Date(ts).toLocaleDateString(intlLocale());
  }

  function hostStatus(host: HostRecord): { live: boolean; text: string } {
    const sessions = sessionCount(host.id);
    if (sessions > 0) {
      const uptime = monitorState.metrics.find((m) => m.host_id === host.id)?.uptime;
      const label = t('hosts.sessionsCount', { count: sessions });
      return { live: true, text: uptime ? `${label} · up ${uptime}` : label };
    }
    const used = lastUsedAt(host.id);
    return { live: false, text: used ? t('hosts.usedAgo', { time: relativeTime(used) }) : t('hosts.neverConnected') };
  }

  function hostAddress(host: HostRecord): string {
    return `${host.username}@${host.address}${host.port !== 22 ? `:${host.port}` : ''}`;
  }

  // --- Keyboard: ↑↓ select, ↵ connect, ⇧↵ details ------------------------------------------------
  function handleListKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isAddModalOpen) {
      isAddModalOpen = false;
      return;
    }
    if (isAddModalOpen || detailHost) return;
    const target = e.target instanceof Element ? e.target : null;
    const typing = target?.closest('input, textarea, select, [contenteditable="true"]');
    if (typing && target !== searchInput) return;
    if (filteredHosts.length === 0) return;

    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const step = e.key === 'ArrowDown' ? 1 : -1;
      focusedIndex = Math.max(0, Math.min(filteredHosts.length - 1, activeIndex + step));
      document.getElementById(`host-card-${filteredHosts[focusedIndex]?.id}`)?.scrollIntoView({ block: 'nearest' });
    } else if (e.key === 'Enter' && activeIndex >= 0) {
      const host = filteredHosts[activeIndex];
      if (!host) return;
      e.preventDefault();
      if (e.shiftKey) detailHost = host;
      else void openSession(host.id);
    }
  }

  const kbdClass =
    'px-1.5 py-0.5 rounded border border-neutral-200 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-900 font-mono text-[10px] text-neutral-600 dark:text-neutral-300';
  const menuItemBase = 'w-full flex items-center px-3 py-1.5 text-left transition-colors';
  const menuItem = `${menuItemBase} text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800/60`;

  function handleOutsidePointer(e: PointerEvent) {
    const target = e.target instanceof Element ? e.target : null;
    if (openMenuId && !target?.closest('[data-host-menu]')) openMenuId = null;
    if (filterOpen && !target?.closest('[data-filter-menu]')) filterOpen = false;
  }
</script>

<svelte:window onkeydown={handleListKeydown} onpointerdown={handleOutsidePointer} />

<div class="max-w-5xl mx-auto space-y-6">
	<PageHeader
		icon={['M4 17l6-5-6-5', 'M12 19h8']}
		accent="sky"
		title={t('hosts.title')}
		subtitle={t('hosts.subtitle', { count: hosts.length, groups: groupCount })}
	>
		{#snippet actions()}
			<button
				onclick={() => openSession('local')}
				class="w-10 h-10 flex items-center justify-center rounded-lg border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-950 text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors"
				title={t('hosts.localTerminal')}
				aria-label={t('hosts.localTerminal')}
			>
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><rect x="3" y="4" width="18" height="12" rx="2" stroke-width="1.8" /><path stroke-linecap="round" stroke-width="1.8" d="M8 20h8M12 16v4" /></svg>
			</button>
			<button
				onclick={openAddModal}
				class="h-10 px-4 flex items-center gap-2 rounded-lg text-sm font-medium bg-neutral-900 text-white hover:bg-neutral-700 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 transition-colors"
			>
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v14m7-7H5" /></svg>
				{t('hosts.addHost')}
			</button>
		{/snippet}
	</PageHeader>

	<!-- Search and filter -->
	<div class="flex gap-2">
    <div class="relative flex-1">
      <svg class="w-4 h-4 absolute left-3.5 top-1/2 -translate-y-1/2 text-neutral-400 dark:text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
      <input
        bind:this={searchInput}
        type="text"
        bind:value={searchQuery}
        oninput={() => (focusedIndex = 0)}
        placeholder={t('hosts.searchPlaceholder')}
        aria-label={t('hosts.searchPlaceholder')}
        class="w-full h-11 pl-10 pr-4 rounded-lg border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-950/70 text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600 transition-colors"
      />
    </div>

    <div class="relative" data-filter-menu>
      <button
        onclick={() => (filterOpen = !filterOpen)}
        aria-haspopup="true"
        aria-expanded={filterOpen}
        class="relative w-11 h-11 flex items-center justify-center rounded-lg border bg-white dark:bg-neutral-950/70 transition-colors {filterOpen || filtersActive ? 'border-neutral-400 dark:border-neutral-600 text-neutral-900 dark:text-white' : 'border-neutral-200 dark:border-neutral-800 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white'}"
        title={t('hosts.filterHosts')}
        aria-label={t('hosts.filterHosts')}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M3 5h18l-7 8.5V19l-4 2v-7.5z" /></svg>
        {#if filtersActive}
          <span class="absolute top-2 right-2 w-1.5 h-1.5 rounded-full bg-sky-500"></span>
        {/if}
      </button>
      {#if filterOpen}
        <div class="absolute right-0 top-full mt-2 z-30 w-52 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl p-1.5 text-sm">
          <p class="px-2.5 pt-1.5 pb-1 text-[10px] font-semibold uppercase tracking-wider text-neutral-400">{t('hosts.show')}</p>
          <label class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-200 cursor-pointer">
            <input type="checkbox" bind:checked={starredOnly} class="rounded" />
            {t('hosts.starredOnly')}
          </label>
          <label class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md hover:bg-neutral-100 dark:hover:bg-neutral-800/60 text-neutral-700 dark:text-neutral-200 cursor-pointer">
            <input type="checkbox" bind:checked={connectedOnly} class="rounded" />
            {t('hosts.connectedOnly')}
          </label>
        </div>
      {/if}
    </div>
  </div>

  <!-- Keyboard hints + sort -->
  <div class="mt-3 flex items-center gap-4 text-[11px] text-neutral-500 dark:text-neutral-400">
    <!-- Keyboard hints are noise on touch-only phones. -->
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>↑↓</kbd> {t('hosts.selectHint')}</span>
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>↵</kbd> {t('hosts.connectHint')}</span>
    <span class="hidden md:flex items-center gap-1.5"><kbd class={kbdClass}>⇧↵</kbd> {t('hosts.detailsHint')}</span>
    <label class="ml-auto flex items-center gap-1.5">
      {t('hosts.sortBy')}
      <select
        bind:value={sortBy}
        class="bg-transparent text-neutral-700 dark:text-neutral-200 font-medium focus:outline-none cursor-pointer"
      >
        <option value="name">{t('hosts.sortName')}</option>
        <option value="lastUsed">{t('hosts.sortLastUsed')}</option>
        <option value="ip">{t('hosts.sortIp')}</option>
      </select>
    </label>
  </div>

  <!-- Bulk actions: only present once at least one host is ticked, so the page stays quiet
       when you are not doing a multi-host operation. -->
  {#if selectedIds.length > 0}
    <div class="mt-4 flex flex-wrap items-center gap-3 p-3 rounded-lg bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-900">
      <span class="text-sm font-medium text-sky-800 dark:text-sky-300">
        {t('hosts.hostsSelected', { count: selectedIds.length })}
      </span>
      <div class="flex items-center gap-2 ml-auto">
        <button
          onclick={connectSelected}
          class="px-3 py-1.5 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-medium transition-colors"
        >
          {t('hosts.connectAll')}
        </button>
        <button
          onclick={deleteSelected}
          class="px-3 py-1.5 rounded-lg text-xs font-medium border border-rose-300 dark:border-rose-900 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors"
        >
          {t('common.delete')}
        </button>
        <button
          onclick={clearSelection}
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          {t('hosts.deselect')}
        </button>
      </div>
    </div>
  {/if}

  <!-- Host cards -->
  {#if hosts.length === 0}
    <div class="mt-6 p-12 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl text-center flex flex-col items-center justify-center">
      <div class="w-12 h-12 rounded-full bg-neutral-100 dark:bg-neutral-800 flex items-center justify-center text-neutral-400 dark:text-neutral-500 mb-4">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 17l6-5-6-5M12 19h8" /></svg>
      </div>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-1">{t('hosts.emptyTitleNoHosts')}</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-sm mb-6">{t('hosts.emptyBodyNoHosts')}</p>
      <button
        onclick={openAddModal}
        class="h-10 px-4 rounded-lg text-sm font-medium bg-neutral-900 text-white hover:bg-neutral-700 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 transition-colors">
        + {t('hosts.addHost')}
      </button>
    </div>
  {:else if filteredHosts.length === 0}
    <div class="mt-6 p-10 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-xl text-center">
      <p class="text-sm text-neutral-500 dark:text-neutral-400">{t('hosts.emptyTitleNoMatch')}</p>
      <button
        onclick={() => { searchQuery = ''; starredOnly = false; connectedOnly = false; }}
        class="mt-3 text-sm font-medium text-sky-600 dark:text-sky-400 hover:underline">
        {t('hosts.clearFilters')}
      </button>
    </div>
  {:else}
    <div class="mt-4 grid grid-cols-1 lg:grid-cols-2 gap-3" role="listbox" aria-label={t('hosts.title')}>
      {#each filteredHosts as host, i (host.id)}
        {@const status = hostStatus(host)}
        {@const os = osName(host)}
        {@const focused = i === activeIndex}
        <div
          id="host-card-{host.id}"
          role="option"
          aria-selected={focused}
          tabindex="-1"
          onpointerdown={() => (focusedIndex = i)}
          ondblclick={(e) => { if (!(e.target as HTMLElement).closest('button, a')) void openSession(host.id); }}
          class="group rounded-xl border p-4 transition-colors bg-white dark:bg-neutral-900/40 {isSelected(host.id) ? 'border-sky-500 ring-1 ring-sky-500/40' : focused ? 'border-neutral-900 dark:border-neutral-200' : 'border-neutral-200 dark:border-neutral-800 hover:border-neutral-300 dark:hover:border-neutral-700'}"
        >
          <!-- Identity -->
          <div class="flex items-start gap-3">
            <span class="w-10 h-10 rounded-lg bg-neutral-100 dark:bg-white flex items-center justify-center shrink-0">
              <OsIcon os={host.os} name={host.label} tags={host.tags} address={host.address} size={22} />
            </span>
            <div class="min-w-0 flex-1">
              <p class="font-semibold text-neutral-900 dark:text-white truncate">{host.label}</p>
              <p class="text-xs font-mono text-neutral-500 dark:text-neutral-400 truncate" title="{host.username}@{host.address}:{host.port}">{hostAddress(host)}</p>
            </div>
            <button
              onclick={() => toggleFavorite(host.id)}
              aria-pressed={isFavorite(host.id)}
              class="p-1.5 -mr-1 -mt-1 rounded-md transition-colors {isFavorite(host.id) ? 'text-amber-500' : 'text-neutral-300 dark:text-neutral-600 hover:text-amber-500'}"
              title={isFavorite(host.id) ? t('hosts.unstar') : t('hosts.star')}
              aria-label={isFavorite(host.id) ? `${t('hosts.unstar')} ${host.label}` : `${t('hosts.star')} ${host.label}`}
            >
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill={isFavorite(host.id) ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"><path d="M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.8L12 16.9l-5.25 2.7 1-5.8L3.5 9.7l5.9-.9z" /></svg>
            </button>
          </div>

          <!-- OS + status + tags -->
          <div class="mt-3 flex flex-wrap items-center gap-1.5 text-[11px]">
            {#if os}
              <span class="inline-flex items-center gap-1.5 max-w-[16rem] px-2 py-0.5 rounded-md border border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-300" title={os}>
                <OsIcon os={host.os} name={host.label} tags={host.tags} address={host.address} size={12} />
                <span class="truncate">{os}</span>
              </span>
            {/if}
            <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md {status.live ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400' : 'text-neutral-500 dark:text-neutral-400'}">
              <span class="w-1.5 h-1.5 rounded-full {status.live ? 'bg-emerald-500 animate-pulse' : 'bg-neutral-300 dark:bg-neutral-600'}"></span>
              {status.text}
            </span>
            {#each host.tags.slice(0, 3) as tag}
              <span class="px-1.5 py-0.5 rounded text-neutral-500 dark:text-neutral-400 bg-neutral-100 dark:bg-neutral-800/70">#{tag}</span>
            {/each}
          </div>

          <!-- Actions -->
          <div class="mt-3 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center gap-1.5">
            <span class="text-[11px] text-neutral-400 dark:text-neutral-500 truncate">
              {host.authMethod.type === 'password' ? t('hosts.authPassword') : t('hosts.authKey')} · {host.hasSecret ? t('hosts.passwordSaved') : host.authMethod.type === 'password' ? t('hosts.passwordNotSaved') : t('hosts.noPassphrase')}
            </span>

            <button
              onclick={() => openEditModal(host)}
              class="ml-auto w-8 h-8 flex items-center justify-center rounded-lg text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
              title={t('hosts.editHost')}
              aria-label={`${t('common.edit')} ${host.label}`}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M16.9 4.1a2.1 2.1 0 013 3L8.5 18.5 4 20l1.5-4.5z" /></svg>
            </button>

            <div class="relative" data-host-menu>
              <button
                onclick={() => (openMenuId = openMenuId === host.id ? null : host.id)}
                aria-haspopup="menu"
                aria-expanded={openMenuId === host.id}
                class="w-8 h-8 flex items-center justify-center rounded-lg text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                title={t('hosts.moreActions')}
                aria-label={`${t('hosts.moreActions')} ${host.label}`}
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><circle cx="5" cy="12" r="1.6" /><circle cx="12" cy="12" r="1.6" /><circle cx="19" cy="12" r="1.6" /></svg>
              </button>
              {#if openMenuId === host.id}
                <div role="menu" class="absolute right-0 top-full mt-1 z-30 w-48 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-[#141414] shadow-2xl py-1 text-sm">
                  <a role="menuitem" href="/sftp?host={host.id}" class={menuItem}>{t('hosts.filesSftp')}</a>
                  <button role="menuitem" onclick={() => { openMenuId = null; detailHost = host; }} class={menuItem}>{t('common.details')}</button>
                  <button role="menuitem" onclick={() => { openMenuId = null; handleClone(host); }} class={menuItem}>{t('common.clone')}</button>
                  <button role="menuitem" onclick={() => { openMenuId = null; toggleSelected(host.id); }} class={menuItem}>
                    {isSelected(host.id) ? t('hosts.deselect') : t('hosts.selectForBatch')}
                  </button>
                  <div class="my-1 border-t border-neutral-100 dark:border-neutral-800"></div>
                  <button role="menuitem" onclick={() => { openMenuId = null; handleDelete(host.id); }} class="{menuItemBase} text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30">{t('common.delete')}</button>
                </div>
              {/if}
            </div>

            <button
              onclick={() => openSession(host.id)}
              class="h-8 px-3 flex items-center gap-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold transition-colors"
              title={t('hosts.connectTo', { name: host.label })}
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 7l5 5-5 5M13 17h6" /></svg>
              {t('common.connect')}
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if detailHost}
  <HostDetailPanel host={detailHost} onClose={() => (detailHost = null)} />
{/if}

{#if isAddModalOpen}
  <!-- Add / edit host drawer -->
  <div class="fixed inset-0 z-50 flex justify-end">
    <button
      type="button"
      class="absolute inset-0 bg-black/50 dark:bg-black/70 cursor-default"
      aria-label={t('hosts.closeEditor')}
      onclick={() => (isAddModalOpen = false)}
    ></button>
    <div
      role="dialog"
      aria-modal="true"
      aria-label={editingId ? 'Edit host' : 'Add host'}
      class="relative h-full w-full max-w-lg overflow-y-auto bg-white dark:bg-[#141414] border-l border-neutral-200 dark:border-neutral-800 p-6 space-y-4 shadow-2xl"
    >
      <div class="flex items-center justify-between border-b border-neutral-200 dark:border-neutral-800 pb-3">
        <h3 class="text-lg font-bold text-neutral-900 dark:text-white">{editingId ? 'Edit SSH Host' : 'Add New SSH Host'}</h3>
        <button onclick={() => isAddModalOpen = false} class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300 text-xl font-bold">×</button>
      </div>

      {#if errorMsg}
        <div class="p-3 text-xs rounded bg-rose-500/10 border border-rose-500/30 text-rose-500 dark:text-rose-400">
          {errorMsg}
        </div>
      {/if}

      <div class="space-y-4 text-sm">
        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Host Label / Name *</label>
          <input 
            type="text" 
            bind:value={formLabel} 
            placeholder="Production VPS / YPC Server"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Host / IP Address *</label>
            <input 
              type="text" 
              bind:value={formAddress} 
              placeholder={t('hosts.addressPlaceholder')}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
          </div>
          <div>
            <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Port</label>
            <input 
              type="number" 
              bind:value={formPort} 
              placeholder="22"
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Protocol</label>
          <select
            bind:value={formProtocol}
            onchange={(e) => {
              const proto = (e.currentTarget as HTMLSelectElement).value;
              if (proto === 'ftp' && formPort === 22) formPort = 21;
              else if (proto === 'ssh' && formPort === 21) formPort = 22;
              else if ((proto === 'webdav' || proto === 's3') && (formPort === 22 || formPort === 21)) formPort = 443;
            }}
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none text-xs"
          >
            <option value="ssh">SSH / SFTP</option>
            <option value="ftp">FTP</option>
            <option value="ftps">FTPS</option>
            <option value="webdav">WebDAV</option>
            <option value="s3">S3</option>
          </select>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Username *</label>
          <input 
            type="text" 
            bind:value={formUsername} 
            placeholder="root / cecep"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">Authentication Method</label>
          <div class="flex gap-4 mb-2">
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="password" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              {t('hosts.form.authPasswordOption')}
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="keyId" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              {t('hosts.form.authVaultKey')}
            </label>
            <label class="flex items-center gap-2 text-xs text-neutral-700 dark:text-neutral-300">
              <input type="radio" value="key" bind:group={formAuthType} class="text-sky-600 focus:ring-0" />
              {t('hosts.form.authLocalFile')}
            </label>
          </div>
          {#if formAuthType === 'key'}
            <input
              type="text"
              bind:value={formKeyPath}
              placeholder={t('hosts.form.keyPathPlaceholder')}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 mb-3 shadow-sm dark:shadow-none"
            />
          {:else if formAuthType === 'keyId'}
            <select
              bind:value={formKeyId}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 mb-3 shadow-sm dark:shadow-none"
            >
              <option value="" disabled>{t('hosts.form.selectVaultKey')}</option>
              {#each vaultKeys as k}
                <option value={k.id}>{k.name} ({k.algorithm})</option>
              {/each}
            </select>
          {/if}
          {#if formAuthType !== 'keyId'}
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">
            {formAuthType === 'key' ? t('hosts.form.keyPassphraseOptional') : t('hosts.form.passwordField')}
          </label>
          <div class="relative">
            <input
              type={showFormSecret ? 'text' : 'password'}
              bind:value={formSecret}
              placeholder={editingId && editingHadSecret ? t('hosts.form.leaveBlankUnchanged') : (formAuthType === 'key' ? t('hosts.form.leaveBlankNoPassphrase') : t('hosts.form.requiredToConnect'))}
              class="w-full pl-3 pr-10 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
            />
            <button
              type="button"
              onclick={() => (showFormSecret = !showFormSecret)}
              class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 p-1"
              aria-label={showFormSecret ? t('hosts.form.hideSecret') : t('hosts.form.showSecret')}
              title={showFormSecret ? t('hosts.form.hideSecret') : t('hosts.form.showSecret')}
            >
              {#if showFormSecret}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              {:else}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              {/if}
            </button>
          </div>
          {/if}
          <p class="text-neutral-500 text-xs mt-1">{t('hosts.form.encryptedNote')}</p>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">{t('hosts.form.osField')}</label>
          <div class="flex items-center gap-2">
            <OsIcon os={formOs} name={formLabel} tags={formTags.split(',')} address={formAddress} size={20} />
            <select
              bind:value={formOs}
              class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none text-xs"
            >
              <option value="">{t('hosts.form.osAutoDetect')}</option>
              <optgroup label={t('hosts.form.osPopularGroup')}>
                <option value="ubuntu">Ubuntu</option>
                <option value="debian">Debian</option>
                <option value="fedora">Fedora</option>
                <option value="redhat">Red Hat / RHEL</option>
                <option value="centos">CentOS</option>
                <option value="rocky">Rocky Linux</option>
                <option value="almalinux">AlmaLinux</option>
                <option value="arch">Arch Linux</option>
                <option value="manjaro">Manjaro</option>
                <option value="alpine">Alpine Linux</option>
                <option value="opensuse">openSUSE / SUSE</option>
                <option value="mint">Linux Mint</option>
                <option value="kali">Kali Linux</option>
                <option value="popos">Pop!_OS</option>
                <option value="gentoo">Gentoo</option>
                <option value="void">Void Linux</option>
                <option value="nixos">NixOS</option>
                <option value="endeavour">EndeavourOS</option>
                <option value="elementary">Elementary OS</option>
                <option value="zorin">Zorin OS</option>
                <option value="raspberry">Raspberry Pi OS / Raspbian</option>
                <option value="amazon">Amazon Linux</option>
                <option value="oracle">Oracle Linux</option>
                <option value="slackware">Slackware</option>
                <option value="mageia">Mageia</option>
                <option value="solus">Solus</option>
                <option value="tails">Tails</option>
                <option value="deepin">Deepin</option>
                <option value="clear">Clear Linux</option>
                <option value="garuda">Garuda Linux</option>
                <option value="steam">SteamOS</option>
                <option value="coreos">CoreOS / Flatcar</option>
                <option value="devuan">Devuan</option>
                <option value="parrot">Parrot OS</option>
                <option value="mx">MX Linux</option>
                <option value="lubuntu">Lubuntu / Xubuntu / Kubuntu</option>
                <option value="linux">{t('hosts.osGenericLinux')}</option>
              </optgroup>
              <optgroup label={t('hosts.form.osOtherGroup')}>
                <option value="windows">Windows</option>
                <option value="macos">macOS / Apple</option>
                <option value="android">Android</option>
                <option value="ios">iOS</option>
                <option value="freebsd">FreeBSD</option>
                <option value="openbsd">OpenBSD</option>
                <option value="netbsd">NetBSD</option>
                <option value="mikrotik">MikroTik / RouterOS</option>
                <option value="cisco">Cisco IOS</option>
                <option value="server">{t('hosts.osGenericServer')}</option>
              </optgroup>
            </select>
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-neutral-700 dark:text-neutral-400 mb-1">{t('hosts.form.tagsField')}</label>
          <input 
            type="text" 
            bind:value={formTags} 
            placeholder={t('hosts.form.tagsPlaceholder')}
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-sm dark:shadow-none"
          />
        </div>
      </div>

      <div class="flex justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
        <button 
          onclick={() => isAddModalOpen = false} 
          class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded-lg text-sm font-medium transition-colors">
          {t('common.cancel')}
        </button>
        <button 
          onclick={handleSaveHost}
          disabled={isLoading}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium shadow-lg shadow-sky-600/20">
          {isLoading ? t('common.saving') : t('common.save')}
        </button>
      </div>
    </div>
  </div>
{/if}
