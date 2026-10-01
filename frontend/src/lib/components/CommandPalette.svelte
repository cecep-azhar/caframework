<script lang="ts">
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
  import { getPalette, closePalette } from '$lib/stores/commandPalette.svelte';
  import { navItems } from '$lib/navItems';

  const palette = getPalette();
  let query = $state('');
  let selectedIndex = $state(0);

  interface CommandItem {
    id: string;
    title: string;
    category: string;
    action: () => void;
  }

  const staticCommands: CommandItem[] = [
    {
      id: 'nav-notes',
      title: 'Go to Notes',
      category: 'Navigation',
      action: () => {
        goto('/');
        closePalette();
      }
    },
    {
      id: 'nav-settings',
      title: 'Go to Settings',
      category: 'Navigation',
      action: () => {
        goto('/settings');
        closePalette();
      }
    }
  ];

  const commands = $derived(
    query.trim() === ''
      ? staticCommands
      : staticCommands.filter(
          (c) =>
            c.title.toLowerCase().includes(query.toLowerCase()) ||
            c.category.toLowerCase().includes(query.toLowerCase())
        )
  );

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePalette();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % Math.max(1, commands.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + commands.length) % Math.max(1, commands.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (commands[selectedIndex]) {
        commands[selectedIndex].action();
      }
    }
  }
</script>

{#if palette.open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/60 backdrop-blur-sm p-4"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => {
      if (e.target === e.currentTarget) closePalette();
    }}
    onkeydown={(e) => {
      if (e.key === 'Escape') closePalette();
    }}
  >
    <div
      class="w-full max-w-xl bg-neutral-900 border border-neutral-800 rounded-2xl shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150"
    >
      <!-- Search input -->
      <div class="flex items-center px-4 border-b border-neutral-800">
        <svg class="w-5 h-5 text-neutral-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          type="text"
          bind:value={query}
          onkeydown={handleKeyDown}
          placeholder="Type a command or search..."
          class="w-full px-4 py-3.5 bg-transparent text-white placeholder-neutral-500 focus:outline-none text-sm"
        />
      </div>

      <!-- Results list -->
      <div class="max-h-80 overflow-y-auto p-2">
        {#if commands.length === 0}
          <div class="p-4 text-center text-sm text-neutral-500">
            No matching commands found.
          </div>
        {:else}
          {#each commands as cmd, idx}
            <button
              onclick={() => cmd.action()}
              onmouseenter={() => (selectedIndex = idx)}
              class="w-full flex items-center justify-between px-3 py-2.5 rounded-xl text-left text-sm transition-colors {selectedIndex === idx ? 'bg-indigo-600 text-white' : 'text-neutral-300 hover:bg-neutral-800'}"
            >
              <span>{cmd.title}</span>
              <span
                class="text-xs px-2 py-0.5 rounded font-mono {selectedIndex === idx ? 'bg-indigo-700 text-indigo-200' : 'bg-neutral-800 text-neutral-500'}"
              >
                {cmd.category}
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}
