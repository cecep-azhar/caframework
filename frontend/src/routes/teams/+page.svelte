<script lang="ts">
  import { onMount } from 'svelte';
  import { listTeams, saveTeam, deleteTeam, type TeamRecord } from '$lib/api/teams';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { listGroups, type GroupRecord } from '$lib/api/groups';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let teams = $state<TeamRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let availableGroups = $state<GroupRecord[]>([]);
  let editingId = $state<string | null>(null);

  let newTeam = $state({
    name: "",
    color: "#3b82f6",
    avatar: "",
    membersStr: "",
    selectedHosts: [] as string[],
    selectedGroups: [] as string[]
  });

  const colors = ["#ef4444", "#f97316", "#f59e0b", "#10b981", "#3b82f6", "#8b5cf6", "#ec4899"];

  onMount(async () => {
    try {
      [teams, availableHosts, availableGroups] = await Promise.all([
        listTeams(),
        listHosts(),
        listGroups()
      ]);
    } catch {
      backendAvailable = false;
    }
  });

  function hostLabel(id: string) {
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function groupLabel(id: string) {
    return availableGroups.find((g) => g.id === id)?.name ?? id;
  }

  function openAddModal() {
    editingId = null;
    newTeam = { name: "", color: "#3b82f6", avatar: "", membersStr: "", selectedHosts: [], selectedGroups: [] };
    isAddModalOpen = true;
  }

  function openEditModal(team: TeamRecord) {
    editingId = team.id;
    newTeam = {
      name: team.name,
      color: team.color,
      avatar: team.avatar ?? "",
      membersStr: team.members.join(", "),
      selectedHosts: [...team.hostIds],
      selectedGroups: [...team.groupIds]
    };
    isAddModalOpen = true;
  }

  async function createTeam(e: Event) {
    e.preventDefault();
    if (!newTeam.name) return;

    const membersArray = newTeam.membersStr.split(',').map(m => m.trim()).filter(m => m.length > 0);

    try {
      const saved = await saveTeam({
        id: editingId ?? undefined,
        name: newTeam.name,
        color: newTeam.color,
        avatar: newTeam.avatar || null,
        members: membersArray,
        hostIds: newTeam.selectedHosts,
        groupIds: newTeam.selectedGroups
      });
      teams = editingId
        ? teams.map((t) => (t.id === saved.id ? saved : t))
        : [...teams, saved];
    } catch {
      backendAvailable = false;
    }

    editingId = null;
    isAddModalOpen = false;
  }

  async function removeTeam(id: string) {
    try {
      await deleteTeam(id);
      teams = teams.filter((t) => t.id !== id);
    } catch {
      backendAvailable = false;
    }
  }

  function toggleHost(hostId: string) {
    if (newTeam.selectedHosts.includes(hostId)) {
      newTeam.selectedHosts = newTeam.selectedHosts.filter(id => id !== hostId);
    } else {
      newTeam.selectedHosts.push(hostId);
    }
  }

  function toggleGroup(groupId: string) {
    if (newTeam.selectedGroups.includes(groupId)) {
      newTeam.selectedGroups = newTeam.selectedGroups.filter(id => id !== groupId);
    } else {
      newTeam.selectedGroups.push(groupId);
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z']}
    accent="teal"
    title={t('teams.title')}
    subtitle={t('teams.subtitle')}
  >
    {#snippet actions()}
      <button
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-lg transition-colors flex items-center gap-2 shadow shadow-sky-600/20">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        {t('teams.createTeam')}
      </button>
    {/snippet}
  </PageHeader>
  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <!-- Teams Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#each teams as team}
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors shadow-sm dark:shadow-none">
        <div>
          <div class="flex items-center gap-3">
            {#if team.avatar}
              <div class="w-8 h-8 rounded-full overflow-hidden border border-neutral-300 dark:border-neutral-700 flex items-center justify-center bg-neutral-100 dark:bg-neutral-800">
                <span class="text-xs">{team.avatar}</span>
              </div>
            {:else}
              <span class="w-4 h-4 rounded-full inline-block" style="background-color: {team.color}"></span>
            {/if}
            <h3 class="font-semibold text-neutral-900 dark:text-white text-lg">{team.name}</h3>
          </div>
          <p class="text-neutral-500 dark:text-neutral-400 text-sm mt-2">{t('teams.membersCount', { count: team.members.length })}</p>
          {#if team.members.length > 0}
            <div class="flex gap-1 flex-wrap mt-1">
              {#each team.members as member}
                <span class="text-xs text-neutral-500 dark:text-neutral-500">{member}</span>
              {/each}
            </div>
          {/if}

          <div class="mt-3">
            {#if team.hostIds.length > 0}
              <p class="text-neutral-500 dark:text-neutral-400 text-xs mt-2 uppercase font-medium">{t('teams.hostsLabel')}</p>
              <div class="flex gap-1 flex-wrap mt-1">
                {#each team.hostIds as hostId}
                  <span class="px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 rounded text-xs">{hostLabel(hostId)}</span>
                {/each}
              </div>
            {/if}
            {#if team.groupIds.length > 0}
              <p class="text-neutral-500 dark:text-neutral-400 text-xs mt-2 uppercase font-medium">{t('teams.groupsLabel')}</p>
              <div class="flex gap-1 flex-wrap mt-1">
                {#each team.groupIds as groupId}
                  <span class="px-2 py-0.5 bg-neutral-100 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 text-neutral-600 dark:text-neutral-400 rounded text-xs">{groupLabel(groupId)}</span>
                {/each}
              </div>
            {/if}
          </div>
        </div>
        <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex justify-end gap-2">
          <button onclick={() => openEditModal(team)} class="px-3 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-sky-600 hover:text-white rounded text-xs text-neutral-700 dark:text-neutral-300 transition-colors">{t('common.edit')}</button>
          <button onclick={() => removeTeam(team.id)} class="px-3 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-red-600 hover:text-white rounded text-xs text-neutral-700 dark:text-neutral-300 transition-colors">{t('common.delete')}</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<!-- Create Team Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4 max-h-[90vh] overflow-y-auto shadow-2xl">
      <h3 class="text-xl font-bold text-neutral-900 dark:text-white">{editingId ? t('teams.editTeam') : t('teams.createTeam')}</h3>

      <form onsubmit={createTeam} class="space-y-4">
        <div>
          <label for="team-name" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('teams.teamName')}</label>
          <input id="team-name" bind:value={newTeam.name} required placeholder={t('teams.teamNamePlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <label for="team-avatar" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('teams.avatar')}</label>
          <input id="team-avatar" bind:value={newTeam.avatar} placeholder={t('teams.avatarPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <label for="team-members" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('teams.members')}</label>
          <input id="team-members" bind:value={newTeam.membersStr} placeholder={t('teams.membersPlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <label for="team-cloud-account" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase">{t('teams.accountUsername')}</label>
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20 font-medium hidden items-center gap-1">
              <svg class="w-3 h-3 text-amber-500 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path></svg>
              {t('teams.proAvailable')}
            </span>
          </div>
          <div class="relative">
            <input
              id="team-cloud-account"
              disabled
              placeholder={t('teams.accountUsernamePlaceholder')}
              class="w-full px-3 py-2 bg-neutral-100 dark:bg-neutral-900/50 border border-neutral-200 dark:border-neutral-800/80 rounded text-sm text-neutral-500 dark:text-neutral-500 cursor-not-allowed select-none"
            />
          </div>
          <p class="text-[11px] text-neutral-500 dark:text-neutral-500 mt-1 hidden items-center gap-1.5">
            <svg class="w-3.5 h-3.5 text-amber-500/80 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            {t('teams.proNote')}
          </p>
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('teams.colorSwatch')}</span>
          <div class="flex gap-2">
            {#each colors as c}
              <button
                type="button"
                onclick={() => newTeam.color = c}
                class="w-7 h-7 rounded-full transition-transform border-2 {newTeam.color === c ? 'border-neutral-900 dark:border-white scale-110' : 'border-transparent'}"
                style="background-color: {c}"
                aria-label={t('common.selectColor', { color: c })}>
              </button>
            {/each}
          </div>
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('teams.assignHosts')}</span>
          <div class="space-y-1 max-h-32 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded">
            {#each availableHosts as h}
              <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-900 rounded cursor-pointer text-sm text-neutral-700 dark:text-neutral-300">
                <input
                  type="checkbox"
                  checked={newTeam.selectedHosts.includes(h.id)}
                  onchange={() => toggleHost(h.id)}
                  class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
                />
                <span>{h.label}</span>
              </label>
            {/each}
            {#if availableHosts.length === 0}
              <span class="text-xs text-neutral-500">{t('teams.noHostsAvailable')}</span>
            {/if}
          </div>
        </div>

        <div>
          <span class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-2">{t('teams.assignGroups')}</span>
          <div class="space-y-1 max-h-32 overflow-y-auto p-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded">
            {#each availableGroups as g}
              <label class="flex items-center gap-2 p-1.5 hover:bg-neutral-100 dark:hover:bg-neutral-900 rounded cursor-pointer text-sm text-neutral-700 dark:text-neutral-300">
                <input
                  type="checkbox"
                  checked={newTeam.selectedGroups.includes(g.id)}
                  onchange={() => toggleGroup(g.id)}
                  class="rounded bg-white dark:bg-neutral-900 border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
                />
                <span>{g.name}</span>
              </label>
            {/each}
            {#if availableGroups.length === 0}
              <span class="text-xs text-neutral-500">{t('teams.noGroupsAvailable')}</span>
            {/if}
          </div>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800 mt-4">
          <button
            type="button"
            onclick={() => isAddModalOpen = false}
            class="px-4 py-2 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 rounded text-sm font-medium mt-2">
            {t('common.cancel')}
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded text-sm font-medium mt-2">
            {t('teams.saveTeam')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
