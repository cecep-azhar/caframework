<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { listNotes, saveNote, deleteNote, type NoteRecord, type NoteInput } from '$lib/api/notes';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let notes = $state<NoteRecord[]>([]);
  let searchQuery = $state('');
  let isLoading = $state(false);
  let isAddModalOpen = $state(false);
  let editingId = $state<string | null>(null);

  // Form state
  let formTitle = $state('');
  let formContent = $state('');
  let formTags = $state('');
  let formVisibility = $state<'shared' | 'private_summary' | 'private'>('shared');
  let currentProfileId = $state('018e0000-0000-7000-8000-000000000001'); // Default Owner profile

  let filteredNotes = $derived(
    notes.filter((n) => {
      const q = searchQuery.toLowerCase().trim();
      if (!q) return true;
      return (
        n.title.toLowerCase().includes(q) ||
        n.content.toLowerCase().includes(q) ||
        n.tags.some((t) => t.toLowerCase().includes(q))
      );
    })
  );

  async function loadNotes() {
    isLoading = true;
    try {
      notes = await listNotes(currentProfileId, true);
    } catch (e: any) {
      showToast(e?.message || 'Failed to load notes', 'error');
    } finally {
      isLoading = false;
    }
  }

  function openAddModal() {
    editingId = null;
    formTitle = '';
    formContent = '';
    formTags = '';
    formVisibility = 'shared';
    isAddModalOpen = true;
  }

  function openEditModal(note: NoteRecord) {
    editingId = note.id;
    formTitle = note.title;
    formContent = note.content;
    formTags = note.tags.join(', ');
    formVisibility = note.visibility;
    isAddModalOpen = true;
  }

  async function handleSaveNote() {
    if (!formTitle.trim()) {
      showToast(t('common.required'), 'error');
      return;
    }

    const tags = formTags
      .split(',')
      .map((x) => x.trim())
      .filter(Boolean);

    const input: NoteInput = {
      id: editingId ?? undefined,
      title: formTitle.trim(),
      content: formContent.trim(),
      tags,
      visibility: formVisibility,
      owner_profile_id: currentProfileId
    };

    try {
      await saveNote(input, currentProfileId);
      showToast(t('notes.saveSuccess'), 'success');
      isAddModalOpen = false;
      await loadNotes();
    } catch (e: any) {
      showToast(e?.message || 'Failed to save note', 'error');
    }
  }

  async function handleDeleteNote(id: string) {
    const ok = await confirmModal(
      t('notes.deleteConfirm'),
      t('common.delete'),
      true,
      t('common.delete'),
      t('common.cancel')
    );
    if (!ok) return;

    try {
      await deleteNote(id, currentProfileId);
      showToast(t('notes.deleteSuccess'), 'success');
      await loadNotes();
    } catch (e: any) {
      showToast(e?.message || 'Failed to delete note', 'error');
    }
  }

  onMount(() => {
    void loadNotes();
  });
</script>

<div class="flex flex-col h-full overflow-y-auto p-6 space-y-6">
  <PageHeader
    title={t('notes.title')}
    subtitle={t('notes.subtitle')}
  >
    <div class="flex items-center gap-3">
      <div class="relative">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder={t('common.search')}
          class="w-64 px-3 py-1.5 text-sm bg-neutral-900/60 border border-neutral-800 rounded-lg text-neutral-200 placeholder-neutral-500 focus:outline-none focus:border-indigo-500"
        />
      </div>
      <button
        onclick={openAddModal}
        class="flex items-center gap-2 px-3.5 py-1.5 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors shadow-sm"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {t('notes.addNote')}
      </button>
    </div>
  </PageHeader>

  {#if isLoading}
    <div class="flex flex-col items-center justify-center py-20 text-neutral-500">
      <div class="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      <p class="mt-4 text-sm">{t('common.loading')}</p>
    </div>
  {:else if filteredNotes.length === 0}
    <div class="flex flex-col items-center justify-center py-20 border border-dashed border-neutral-800 rounded-2xl bg-neutral-900/20 text-center p-6">
      <div class="w-12 h-12 rounded-full bg-neutral-800/80 flex items-center justify-center text-neutral-400 mb-4">
        <svg class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
        </svg>
      </div>
      <h3 class="text-base font-semibold text-neutral-300">{t('notes.noNotes')}</h3>
      <p class="text-sm text-neutral-500 mt-1 max-w-sm">{t('notes.subtitle')}</p>
      <button
        onclick={openAddModal}
        class="mt-5 px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors"
      >
        {t('notes.addNote')}
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each filteredNotes as note (note.id)}
        <div class="group relative flex flex-col justify-between p-5 bg-neutral-900/50 hover:bg-neutral-900/80 border border-neutral-800/80 hover:border-neutral-700/80 rounded-xl transition-all duration-150">
          <div>
            <div class="flex items-start justify-between gap-2 mb-2">
              <h4 class="font-semibold text-neutral-200 group-hover:text-white line-clamp-1">{note.title}</h4>
              <span class="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-neutral-800 text-neutral-400 border border-neutral-700/50">
                {note.visibility}
              </span>
            </div>
            <p class="text-sm text-neutral-400 line-clamp-3 whitespace-pre-wrap">{note.content}</p>
          </div>

          <div class="mt-4 pt-3 border-t border-neutral-800/60 flex items-center justify-between">
            <div class="flex flex-wrap gap-1">
              {#each note.tags as tag}
                <span class="text-xs px-2 py-0.5 rounded bg-neutral-800/60 text-neutral-400">
                  #{tag}
                </span>
              {/each}
            </div>

            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onclick={() => openEditModal(note)}
                class="p-1.5 text-neutral-400 hover:text-white hover:bg-neutral-800 rounded-md transition-colors"
                title={t('common.edit')}
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
              <button
                onclick={() => handleDeleteNote(note.id)}
                class="p-1.5 text-neutral-400 hover:text-red-400 hover:bg-neutral-800 rounded-md transition-colors"
                title={t('common.delete')}
              >
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if isAddModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
    <div class="w-full max-w-lg bg-neutral-900 border border-neutral-800 rounded-2xl p-6 shadow-2xl space-y-4">
      <h3 class="text-lg font-semibold text-white">
        {editingId ? t('notes.editNote') : t('notes.addNote')}
      </h3>

      <div class="space-y-3">
        <div>
          <label for="modal-note-title" class="block text-xs font-medium text-neutral-400 mb-1">{t('notes.noteTitle')}</label>
          <input
            id="modal-note-title"
            type="text"
            bind:value={formTitle}
            placeholder={t('notes.noteTitle')}
            class="w-full px-3 py-2 text-sm bg-neutral-800 border border-neutral-700 rounded-lg text-white focus:outline-none focus:border-indigo-500"
          />
        </div>

        <div>
          <label for="modal-note-content" class="block text-xs font-medium text-neutral-400 mb-1">{t('notes.noteContent')}</label>
          <textarea
            id="modal-note-content"
            rows="5"
            bind:value={formContent}
            placeholder={t('notes.noteContent')}
            class="w-full px-3 py-2 text-sm bg-neutral-800 border border-neutral-700 rounded-lg text-white focus:outline-none focus:border-indigo-500"
          ></textarea>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="modal-note-tags" class="block text-xs font-medium text-neutral-400 mb-1">{t('notes.tags')}</label>
            <input
              id="modal-note-tags"
              type="text"
              bind:value={formTags}
              placeholder="work, ideas, todo"
              class="w-full px-3 py-2 text-sm bg-neutral-800 border border-neutral-700 rounded-lg text-white focus:outline-none focus:border-indigo-500"
            />
          </div>
          <div>
            <label for="modal-note-visibility" class="block text-xs font-medium text-neutral-400 mb-1">{t('notes.visibility')}</label>
            <select
              id="modal-note-visibility"
              bind:value={formVisibility}
              class="w-full px-3 py-2 text-sm bg-neutral-800 border border-neutral-700 rounded-lg text-white focus:outline-none focus:border-indigo-500"
            >
              <option value="shared">{t('notes.visibilityShared')}</option>
              <option value="private_summary">{t('notes.visibilitySummary')}</option>
              <option value="private">{t('notes.visibilityPrivate')}</option>
            </select>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-3 pt-3 border-t border-neutral-800">
        <button
          onclick={() => (isAddModalOpen = false)}
          class="px-4 py-2 text-sm font-medium text-neutral-400 hover:text-white hover:bg-neutral-800 rounded-lg transition-colors"
        >
          {t('common.cancel')}
        </button>
        <button
          onclick={handleSaveNote}
          class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-500 rounded-lg transition-colors shadow-sm"
        >
          {t('common.save')}
        </button>
      </div>
    </div>
  </div>
{/if}
