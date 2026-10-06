<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { listNotes, saveNote, deleteNote, type NoteRecord, type NoteInput } from '$lib/api/notes';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import LoadingState from '$lib/components/LoadingState.svelte';

  let notes = $state<NoteRecord[]>([]);
  let searchQuery = $state('');
  let isLoading = $state(false);
  let isAddModalOpen = $state(false);
  let editingId = $state<string | null>(null);

  // Form state
  let formTitle = $state('');
  let formContent = $state('');
  let formCategory = $state('Personal');
  let formVisibility = $state<'shared' | 'private_summary' | 'private'>('shared');
  let currentProfileId = $state('018e0000-0000-7000-8000-000000000001'); // Default Owner profile

  let filteredNotes = $derived(
    notes.filter((n) => {
      const q = searchQuery.toLowerCase().trim();
      if (!q) return true;
      return (
        n.title.toLowerCase().includes(q) ||
        n.content.toLowerCase().includes(q) ||
        (n.category && n.category.toLowerCase().includes(q))
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
    formCategory = 'Personal';
    formVisibility = 'shared';
    isAddModalOpen = true;
  }

  function editNote(note: NoteRecord) {
    editingId = note.id;
    formTitle = note.title;
    formContent = note.content;
    formCategory = note.category || 'Personal';
    formVisibility = (note.visibility as any) || 'shared';
    isAddModalOpen = true;
  }

  async function handleSaveNote() {
    if (!formTitle.trim()) {
      showToast(t('common.required'), 'error');
      return;
    }

    const input: NoteInput = {
      title: formTitle.trim(),
      content: formContent.trim(),
      category: formCategory.trim() || null,
      visibility: formVisibility
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

  function toggleFavorite(note: NoteRecord) {
    const updated = { ...note, is_favorite: !note.is_favorite };
    notes = notes.map((n) => (n.id === note.id ? updated : n));
  }

  function formatDate(timestamp: number): string {
    if (!timestamp) return 'Just now';
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  onMount(() => {
    void loadNotes();
  });
</script>

<div class="flex flex-col h-full overflow-y-auto px-6 py-5 space-y-4 max-w-7xl mx-auto w-full">
  <!-- Page Header -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-sky-100 dark:bg-sky-950/60 border border-sky-200/80 dark:border-sky-800/60 flex items-center justify-center text-sky-600 dark:text-sky-400 font-mono font-bold text-xs shadow-2xs">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
      </div>
      <div>
        <h1 class="text-lg font-bold tracking-tight text-neutral-900 dark:text-neutral-100 leading-tight">{t('notes.title')}</h1>
        <p class="text-[11px] text-neutral-500 dark:text-neutral-400">{notes.length} saved notes in encrypted vault</p>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <!-- Display toggle button -->
      <button
        type="button"
        class="p-1.5 rounded-lg border border-neutral-200 dark:border-neutral-700/80 bg-white dark:bg-neutral-800/80 text-neutral-500 hover:text-neutral-800 dark:hover:text-white shadow-2xs transition-colors"
        title={'Display view'}
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>

      <!-- Add Note Button -->
      <button
        onclick={openAddModal}
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-white bg-neutral-900 hover:bg-neutral-800 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-white rounded-lg shadow-sm transition-colors cursor-pointer"
      >
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 4v16m8-8H4" />
        </svg>
        <span>{t('notes.addNote')}</span>
      </button>
    </div>
  </div>

  <!-- Search Bar -->
  <div class="relative">
    <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-neutral-400">
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
    </div>
    <input
      type="text"
      bind:value={searchQuery}
      placeholder={'Search by title, content, or category...'}
      class="w-full pl-9 pr-8 py-2 text-xs bg-white dark:bg-neutral-900/90 border border-neutral-200/90 dark:border-neutral-800 rounded-xl text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 focus:outline-none focus:ring-1 focus:ring-sky-500 shadow-2xs"
    />
    <div class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none text-neutral-400">
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
      </svg>
    </div>
  </div>

  <!-- Shortcuts Guide & Sort Header -->
  <div class="flex items-center justify-between text-[11px] text-neutral-400 pt-0.5">
    <div class="flex items-center gap-3 px-2 py-1 rounded-lg bg-neutral-100/60 dark:bg-neutral-900/60 border border-neutral-200/60 dark:border-neutral-800/60">
      <span class="flex items-center gap-1">
        <kbd class="px-1 py-0.2 bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-[10px] text-neutral-500 font-mono">↑↓</kbd>
        <span class="text-neutral-500 dark:text-neutral-400">select</span>
      </span>
      <span class="flex items-center gap-1">
        <kbd class="px-1 py-0.2 bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-[10px] text-neutral-500 font-mono">↵</kbd>
        <span class="text-neutral-500 dark:text-neutral-400">open</span>
      </span>
      <span class="flex items-center gap-1">
        <kbd class="px-1 py-0.2 bg-white dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded text-[10px] text-neutral-500 font-mono">⇧↵</kbd>
        <span class="text-neutral-500 dark:text-neutral-400">details</span>
      </span>
    </div>

    <div class="flex items-center gap-1.5">
      <span class="text-neutral-400 text-xs">Sort</span>
      <button
        type="button"
        class="flex items-center gap-1 px-2.5 py-1 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg text-xs font-medium text-neutral-700 dark:text-neutral-300 hover:bg-neutral-50 shadow-2xs transition-colors"
      >
        <span>Title</span>
        <svg class="w-3 h-3 text-neutral-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
    </div>
  </div>

  {#if isLoading}
    <LoadingState message={t('common.loading')} />
  {:else if filteredNotes.length === 0}
    <EmptyState
      title={t('notes.noNotes')}
      description={t('notes.subtitle')}
      icon="notes"
      actionLabel={t('notes.addNote')}
      onaction={openAddModal}
    />
  {:else}
    <!-- Notes 2-Column Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5 pt-1">
      {#each filteredNotes as note (note.id)}
        <div
          class="group relative flex flex-col justify-between p-3.5 bg-white dark:bg-neutral-900/90 border border-neutral-200/90 dark:border-neutral-800 rounded-2xl shadow-2xs hover:shadow-md transition-all duration-150 space-y-2.5"
        >
          <!-- Card Top Row: Icon, Title, Category Pill, Star -->
          <div class="flex items-start justify-between gap-2">
            <div class="flex items-center gap-2.5 min-w-0">
              <!-- Note Category Avatar Circle -->
              <div
                class="w-7 h-7 rounded-full flex items-center justify-center font-bold text-white text-xs shrink-0 shadow-2xs bg-indigo-600"
              >
                <span class="text-[11px] font-bold">{(note.title || 'N').slice(0, 1).toUpperCase()}</span>
              </div>

              <!-- Title & Category Pill -->
              <div class="flex items-center gap-2 min-w-0">
                <h3 class="font-bold text-sm text-neutral-900 dark:text-neutral-100 truncate">{note.title}</h3>
                {#if note.category}
                  <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-400 border border-sky-200/60 dark:border-sky-800/40 shrink-0">
                    <span class="w-1.5 h-1.5 rounded-full bg-sky-500"></span>
                    {note.category}
                  </span>
                {/if}
              </div>
            </div>

            <!-- Star Toggle -->
            <button
              onclick={() => toggleFavorite(note)}
              class="p-1 text-neutral-300 hover:text-amber-400 transition-colors"
              title={'Star host'}
            >
              {#if note.is_favorite}
                <svg class="w-4 h-4 text-amber-400 fill-amber-400" viewBox="0 0 24 24">
                  <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                </svg>
              {:else}
                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
              {/if}
            </button>
          </div>

          <!-- Note Content Excerpt -->
          <p class="text-xs text-neutral-500 dark:text-neutral-400 line-clamp-2 leading-relaxed whitespace-pre-wrap">
            {note.content || '(Empty note content)'}
          </p>

          <!-- Badges & Tags Row -->
          <div class="flex flex-wrap items-center gap-1.5 pt-0.5">
            <!-- Visibility Tag -->
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 uppercase font-mono">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
              {note.visibility}
            </span>

            <!-- Updated Time -->
            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-neutral-100 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400">
              <span>Updated {formatDate(note.updated_at)}</span>
            </span>
          </div>

          <!-- Footer Action Bar -->
          <div class="pt-2 border-t border-neutral-100 dark:border-neutral-800/80 flex items-center justify-between">
            <div class="text-[11px] text-neutral-400 dark:text-neutral-500 font-mono">
              AES-256-GCM · Encrypted
            </div>

            <div class="flex items-center gap-1.5">
              <!-- Edit button -->
              <button
                onclick={() => editNote(note)}
                type="button"
                class="p-1 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                title={'Edit host'}
              >
                <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                </svg>
              </button>

              <!-- Delete button -->
              <button
                onclick={() => handleDeleteNote(note.id)}
                type="button"
                class="p-1 rounded text-neutral-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                title={'More options'}
              >
                <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>

              <!-- Open / View Button -->
              <button
                onclick={() => editNote(note)}
                class="flex items-center gap-1 px-3 py-1 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-xs font-semibold shadow-xs transition-colors cursor-pointer"
              >
                <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
                <span>View</span>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Add/Edit Note Modal -->
{#if isAddModalOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-xs p-4">
    <div class="w-full max-w-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl p-5 shadow-2xl space-y-3.5 text-xs text-neutral-800 dark:text-neutral-200">
      <div class="flex items-center justify-between pb-2 border-b border-neutral-200 dark:border-neutral-800">
        <h3 class="text-sm font-bold text-neutral-900 dark:text-white">
          {editingId ? t('notes.editNote') : t('notes.addNote')}
        </h3>
        <button
          onclick={() => (isAddModalOpen = false)}
          class="p-1 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-white"
          aria-label={'Close modal'}
          title={'Close'}
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-2.5">
        <div>
          <label for="modal-note-title" class="block text-[11px] font-semibold text-neutral-600 dark:text-neutral-400 mb-1">{t('notes.noteTitle')}</label>
          <input
            id="modal-note-title"
            type="text"
            bind:value={formTitle}
            placeholder={'Enter note title...'}
            class="w-full px-3 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-sky-500"
          />
        </div>

        <div>
          <label for="modal-note-content" class="block text-[11px] font-semibold text-neutral-600 dark:text-neutral-400 mb-1">{t('notes.noteContent')}</label>
          <textarea
            id="modal-note-content"
            rows="5"
            bind:value={formContent}
            placeholder={'Write encrypted content here...'}
            class="w-full px-3 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-sky-500"
          ></textarea>
        </div>

        <div class="grid grid-cols-2 gap-2.5">
          <div>
            <label for="modal-note-category" class="block text-[11px] font-semibold text-neutral-600 dark:text-neutral-400 mb-1">Category</label>
            <input
              id="modal-note-category"
              type="text"
              bind:value={formCategory}
              placeholder={'Personal, Work, Ideas'}
              class="w-full px-3 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-sky-500"
            />
          </div>

          <div>
            <label for="modal-note-visibility" class="block text-[11px] font-semibold text-neutral-600 dark:text-neutral-400 mb-1">{t('notes.visibility')}</label>
            <select
              id="modal-note-visibility"
              bind:value={formVisibility}
              class="w-full px-2.5 py-1.5 text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg text-neutral-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-sky-500"
            >
              <option value="shared">{t('notes.visibilityShared')}</option>
              <option value="private_summary">{t('notes.visibilitySummary')}</option>
              <option value="private">{t('notes.visibilityPrivate')}</option>
            </select>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2 border-t border-neutral-200 dark:border-neutral-800">
        <button
          onclick={() => (isAddModalOpen = false)}
          class="px-3 py-1.5 text-xs font-medium text-neutral-500 hover:text-neutral-800 dark:hover:text-white rounded-lg transition-colors"
        >
          {t('common.cancel')}
        </button>
        <button
          onclick={handleSaveNote}
          class="px-3.5 py-1.5 text-xs font-semibold text-white bg-sky-600 hover:bg-sky-500 rounded-lg shadow-sm transition-colors cursor-pointer"
        >
          {t('common.save')}
        </button>
      </div>
    </div>
  </div>
{/if}
