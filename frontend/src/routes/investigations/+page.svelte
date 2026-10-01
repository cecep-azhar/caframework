<script lang="ts">
  import { onMount } from 'svelte';
  import { listInvestigations, saveInvestigation, deleteInvestigation, type InvestigationRecord } from '$lib/api/investigations';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let isAddModalOpen = $state(false);
  let backendAvailable = $state(true);
  let investigations = $state<InvestigationRecord[]>([]);
  let availableHosts = $state<HostRecord[]>([]);
  let editingId = $state<string | null>(null);

  let newInvestigation = $state({
    title: "",
    host_id: "" as string | null,
    status: "OPEN",
    notes: "",
    evidence: "[]"
  });

  onMount(async () => {
    try {
      [investigations, availableHosts] = await Promise.all([
        listInvestigations(),
        listHosts()
      ]);
    } catch {
      backendAvailable = false;
    }
  });

  function hostLabel(id: string | null) {
    if (!id) return t('investigations.global');
    return availableHosts.find((h) => h.id === id)?.label ?? id;
  }

  function openAddModal() {
    editingId = null;
    newInvestigation = { title: "", host_id: "", status: "OPEN", notes: "", evidence: "[]" };
    isAddModalOpen = true;
  }

  function openEditModal(inv: InvestigationRecord) {
    editingId = inv.id;
    newInvestigation = {
      title: inv.title,
      host_id: inv.host_id,
      status: inv.status,
      notes: inv.notes,
      evidence: inv.evidence
    };
    isAddModalOpen = true;
  }

  async function createInvestigation(e: Event) {
    e.preventDefault();
    if (!newInvestigation.title) return;

    try {
      const saved = await saveInvestigation({
        id: editingId ?? undefined,
        title: newInvestigation.title,
        host_id: newInvestigation.host_id || null,
        status: newInvestigation.status,
        notes: newInvestigation.notes,
        evidence: newInvestigation.evidence
      });
      investigations = editingId
        ? investigations.map((i) => (i.id === saved.id ? saved : i))
        : [saved, ...investigations];
    } catch {
      backendAvailable = false;
    }

    editingId = null;
    isAddModalOpen = false;
  }

  async function removeInvestigation(id: string) {
    try {
      await deleteInvestigation(id);
      investigations = investigations.filter((i) => i.id !== id);
    } catch {
      backendAvailable = false;
    }
  }

  function formatDate(ts: number) {
    return new Date(ts).toLocaleString(intlLocale());
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z']}
    accent="rose"
    title={t('investigations.title')}
    subtitle={t('investigations.subtitle')}
  >
    {#snippet actions()}
      <button
        onclick={openAddModal}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-lg transition-colors flex items-center gap-2 shadow shadow-sky-600/20">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
        {t('investigations.newInvestigation')}
      </button>
    {/snippet}
  </PageHeader>
  {#if !backendAvailable}
    <p class="text-amber-500 text-xs -mt-4">{t('common.backendUnavailable')}</p>
  {/if}

  <div class="grid grid-cols-1 gap-4">
    {#each investigations as inv}
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 flex flex-col justify-between hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors shadow-sm dark:shadow-none">
        <div>
          <div class="flex justify-between items-start">
            <h3 class="font-semibold text-neutral-900 dark:text-white text-lg">{inv.title}</h3>
            <span class="px-2 py-1 text-xs rounded border {inv.status === 'OPEN' ? 'border-amber-500/50 text-amber-600 dark:text-amber-500' : 'border-green-500/50 text-green-600 dark:text-green-500'}">
              {inv.status === 'OPEN' ? t('investigations.statusOpen') : inv.status === 'IN_PROGRESS' ? t('investigations.statusInProgress') : t('investigations.statusClosed')}
            </span>
          </div>
          <p class="text-neutral-500 dark:text-neutral-400 text-sm mt-1">{t('investigations.host')}: {hostLabel(inv.host_id)}</p>
          <p class="text-neutral-400 dark:text-neutral-500 text-xs mt-1">{t('investigations.opened')}: {formatDate(inv.created_at)}</p>
          {#if inv.notes}
            <div class="mt-3 p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded text-neutral-700 dark:text-neutral-300 text-sm whitespace-pre-wrap">{inv.notes}</div>
          {/if}
          <div class="mt-3 flex gap-2">
            <span class="text-xs text-neutral-500 dark:text-neutral-400 border border-neutral-300 dark:border-neutral-700 px-2 py-1 rounded">{t('investigations.evidenceAttached')}: {inv.evidence !== '[]' ? t('investigations.evidenceYes') : t('investigations.evidenceNone')}</span>
          </div>
        </div>
        <div class="mt-4 pt-3 border-t border-neutral-100 dark:border-neutral-800/80 flex justify-end gap-2">
          <button onclick={() => openEditModal(inv)} class="px-3 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-sky-600 hover:text-white rounded text-xs text-neutral-700 dark:text-neutral-300 transition-colors">{t('common.edit')}</button>
          <button onclick={() => removeInvestigation(inv.id)} class="px-3 py-1 bg-neutral-100 dark:bg-neutral-800 hover:bg-red-600 hover:text-white rounded text-xs text-neutral-700 dark:text-neutral-300 transition-colors">{t('common.delete')}</button>
        </div>
      </div>
    {/each}
    {#if investigations.length === 0}
      <div class="text-center py-10 text-neutral-500 dark:text-neutral-500 border border-dashed border-neutral-300 dark:border-neutral-800 rounded-lg">
        {t('investigations.emptyTitle')}
      </div>
    {/if}
  </div>
</div>

{#if isAddModalOpen}
  <div class="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4 z-50">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 max-w-lg w-full space-y-4 max-h-[90vh] overflow-y-auto shadow-2xl">
      <h3 class="text-xl font-bold text-neutral-900 dark:text-white">{editingId ? t('investigations.editInvestigation') : t('investigations.newInvestigation')}</h3>
      
      <form onsubmit={createInvestigation} class="space-y-4">
        <div>
          <label for="inv-title" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('investigations.incidentTitle')}</label>
          <input id="inv-title" bind:value={newInvestigation.title} required placeholder={t('investigations.incidentTitlePlaceholder')} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500" />
        </div>

        <div>
          <label for="inv-host" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('investigations.targetHost')}</label>
          <select id="inv-host" bind:value={newInvestigation.host_id} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500">
            <option value="">{t('investigations.globalMultipleHosts')}</option>
            {#each availableHosts as h}
              <option value={h.id}>{h.label}</option>
            {/each}
          </select>
        </div>

        <div>
          <label for="inv-status" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('investigations.status')}</label>
          <select id="inv-status" bind:value={newInvestigation.status} class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500">
            <option value="OPEN">{t('investigations.statusOpen')}</option>
            <option value="IN_PROGRESS">{t('investigations.statusInProgress')}</option>
            <option value="CLOSED">{t('investigations.statusClosed')}</option>
          </select>
        </div>

        <div>
          <label for="inv-notes" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('investigations.manualNotes')}</label>
          <textarea id="inv-notes" bind:value={newInvestigation.notes} placeholder={t('investigations.notesPlaceholder')} rows="4" class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500"></textarea>
        </div>

        <div>
          <label for="inv-evidence" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 uppercase mb-1">{t('investigations.evidencePayload')}</label>
          <textarea id="inv-evidence" bind:value={newInvestigation.evidence} placeholder="[]" rows="2" class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded text-sm text-neutral-500 dark:text-neutral-400 font-mono focus:outline-none focus:border-sky-500"></textarea>
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
            {t('common.save')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
