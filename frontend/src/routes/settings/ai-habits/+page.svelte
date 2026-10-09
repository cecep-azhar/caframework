<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import AiNavTabs from '$lib/components/AiNavTabs.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import {
    type HabitFact,
    aiGetHabits,
    aiSearchHabits,
    aiToggleHabitPin,
    aiDeleteHabit
  } from '$lib/api/aiRouting';

  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const INPUT = 'w-full px-3 py-2 text-sm rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-sky-500';

  let habits = $state<HabitFact[]>([]);
  let searchQuery = $state('');
  let loading = $state(true);
  let searching = $state(false);

  const pinnedCount = $derived(habits.filter((h) => h.is_pinned).length);

  async function loadData() {
    loading = true;
    try {
      habits = await aiGetHabits();
    } catch (err) {
      showToast('Failed to load habit memories: ' + String(err), 'error');
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  async function handleSearch() {
    const q = searchQuery.trim();
    if (!q) {
      await loadData();
      return;
    }
    searching = true;
    try {
      habits = await aiSearchHabits(q, 50);
    } catch (err) {
      showToast('Habit search failed: ' + String(err), 'error');
    } finally {
      searching = false;
    }
  }

  async function handleTogglePin(h: HabitFact) {
    try {
      const isPinned = await aiToggleHabitPin(h.id);
      h.is_pinned = isPinned;
      showToast(isPinned ? `Pinned "${h.key_tag}" to priority memory.` : `Unpinned "${h.key_tag}".`, 'info');
    } catch (err) {
      showToast('Failed to toggle pin: ' + String(err), 'error');
    }
  }

  async function handleDeleteHabit(h: HabitFact) {
    const confirmed = await confirmModal(`Delete habit fact "${h.fact_content}"?`);
    if (!confirmed) return;
    try {
      await aiDeleteHabit(h.id);
      habits = habits.filter((item) => item.id !== h.id);
      showToast('Habit memory deleted.', 'success');
    } catch (err) {
      showToast('Failed to delete habit: ' + String(err), 'error');
    }
  }
</script>

<div class="max-w-6xl mx-auto space-y-6">
  <PageHeader
    icon="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
    accent="teal"
    title="Persistent Habit Memory (FTS5)"
    badge="Zero-Knowledge Local Storage"
    subtitle="Passive heuristic learning of your shell commands, container runtimes, and developer conventions."
  />

  <AiNavTabs active="habits" />

  <!-- Privacy & Zero-Knowledge Banner -->
  <div class="p-4 rounded-lg bg-teal-500/10 border border-teal-500/30 flex items-start gap-3 text-teal-800 dark:text-teal-200 text-xs">
    <svg class="w-5 h-5 text-teal-600 dark:text-teal-400 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
    </svg>
    <div class="space-y-1">
      <div class="font-semibold text-teal-900 dark:text-teal-100">100% On-Device Zero-Knowledge Architecture</div>
      <p>
        Learned habits are indexed locally using SQLite FTS5 with Porter tokenization inside your encrypted SQLCipher database. They are dynamically injected at Stage 3 of prompt assembly when relevant to your prompt, and <strong>never synced to external telemetry</strong>.
      </p>
    </div>
  </div>

  {#if loading}
    <div class="{CARD} flex items-center justify-center py-12 text-sm {MUTED}">
      <svg class="w-5 h-5 animate-spin mr-3 text-sky-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
      </svg>
      Loading Habit Memories...
    </div>
  {:else}
    <div class="{CARD} space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <span class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
            Total Habits: <strong class="text-neutral-900 dark:text-white">{habits.length}</strong>
          </span>
          <span class="text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
            Pinned Priority: <strong class="text-amber-600 dark:text-amber-400">{pinnedCount}</strong>
          </span>
        </div>

        <!-- Search Bar -->
        <div class="flex gap-2 max-w-sm w-full">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search habits via FTS5..."
            class="{INPUT} text-xs"
            onkeydown={(e) => { if (e.key === 'Enter') handleSearch(); }}
          />
          <button
            onclick={handleSearch}
            disabled={searching}
            class="px-3 py-1.5 text-xs font-semibold rounded bg-teal-600 hover:bg-teal-500 disabled:opacity-50 text-white transition-colors"
          >
            {searching ? '...' : 'Search'}
          </button>
          {#if searchQuery}
            <button
              onclick={() => { searchQuery = ''; loadData(); }}
              class="px-2.5 py-1.5 text-xs font-medium rounded border border-neutral-300 dark:border-neutral-700 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800"
            >
              Clear
            </button>
          {/if}
        </div>
      </div>

      {#if habits.length === 0}
        <div class="{SUBCARD} text-center py-8 text-neutral-500 dark:text-neutral-400 text-xs">
          {#if searchQuery}
            No habits matched FTS5 query "{searchQuery}".
          {:else}
            No developer habits recorded yet. As you execute commands in the terminal (e.g. git, podman, nvim), CATerm will passively learn your preferences!
          {/if}
        </div>
      {:else}
        <div class="overflow-x-auto border border-neutral-200 dark:border-neutral-800 rounded-lg">
          <table class="w-full text-left text-xs">
            <thead class="bg-neutral-100 dark:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300 uppercase tracking-wider font-semibold border-b border-neutral-200 dark:border-neutral-800">
              <tr>
                <th class="px-4 py-3">Tag / Key</th>
                <th class="px-4 py-3">Learned Habit Fact</th>
                <th class="px-4 py-3">Category</th>
                <th class="px-4 py-3">Confidence</th>
                <th class="px-4 py-3">Occurrences</th>
                <th class="px-4 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
              {#each habits as h (h.id)}
                <tr class="hover:bg-neutral-50 dark:hover:bg-neutral-800/30 transition-colors">
                  <td class="px-4 py-3 font-mono font-medium text-neutral-900 dark:text-white flex items-center gap-1.5">
                    {#if h.is_pinned}
                      <span class="text-amber-500 text-sm" title="Pinned to Top-K memory">★</span>
                    {/if}
                    {h.key_tag}
                  </td>
                  <td class="px-4 py-3 text-neutral-800 dark:text-neutral-200 max-w-md">
                    {h.fact_content}
                  </td>
                  <td class="px-4 py-3">
                    <span class="px-2 py-0.5 rounded text-[10px] font-mono font-medium bg-neutral-200 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300">
                      {h.category}
                    </span>
                  </td>
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-2">
                      <div class="w-16 bg-neutral-200 dark:bg-neutral-700 h-1.5 rounded-full overflow-hidden">
                        <div
                          class="bg-teal-500 h-full rounded-full"
                          style="width: {Math.min(100, Math.round(h.confidence_score * 100))}%"
                        ></div>
                      </div>
                      <span class="font-mono text-[11px] text-neutral-600 dark:text-neutral-400">
                        {(h.confidence_score * 100).toFixed(0)}%
                      </span>
                    </div>
                  </td>
                  <td class="px-4 py-3 font-mono text-neutral-600 dark:text-neutral-400">
                    {h.occurrence_count}x
                  </td>
                  <td class="px-4 py-3 text-right space-x-2">
                    <button
                      onclick={() => handleTogglePin(h)}
                      class="text-amber-600 dark:text-amber-400 hover:underline font-medium"
                    >
                      {h.is_pinned ? 'Unpin' : 'Pin'}
                    </button>
                    <button
                      onclick={() => handleDeleteHabit(h)}
                      class="text-rose-600 dark:text-rose-400 hover:underline font-medium"
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>
