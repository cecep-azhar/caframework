<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import type { AutocompleteItem } from '$lib/data/terminalCommands';

  let {
    suggestions = [],
    selectedIndex = 0,
    cursorX = 20,
    cursorY = 60,
    cursorTop = 42,
    containerWidth = 0,
    containerHeight = 0,
    onSelect,
    onClose
  }: {
    suggestions: AutocompleteItem[];
    selectedIndex: number;
    cursorX?: number;
    cursorY?: number;
    cursorTop?: number;
    containerWidth?: number;
    containerHeight?: number;
    onSelect: (item: AutocompleteItem) => void;
    onClose: () => void;
  } = $props();

  let popupEl: HTMLDivElement | undefined = $state();

  const GAP = 4;
  const EDGE = 8;

  // Measured popup size (falls back to the design size until the first layout).
  let popupWidth = $state(360);
  let popupHeight = $state(240);

  let parentWidth = $derived(
    containerWidth > 0
      ? containerWidth
      : (popupEl?.parentElement?.clientWidth ?? (typeof window !== 'undefined' ? window.innerWidth : 800))
  );

  let parentHeight = $derived(
    containerHeight > 0
      ? containerHeight
      : (popupEl?.parentElement?.clientHeight ?? (typeof window !== 'undefined' ? window.innerHeight : 600))
  );

  // Start at the cursor column; shift left only as far as needed to stay inside the pane.
  let clampedX = $derived(
    Math.max(EDGE, Math.min(cursorX, parentWidth - popupWidth - EDGE))
  );

  // Prefer below the cursor line; flip above it when there is not enough room underneath.
  let clampedY = $derived(
    cursorY + GAP + popupHeight > parentHeight - EDGE && cursorTop - GAP - popupHeight >= EDGE
      ? cursorTop - GAP - popupHeight
      : Math.max(EDGE, Math.min(cursorY + GAP, parentHeight - popupHeight - EDGE))
  );
</script>

{#if suggestions.length > 0}
  <div
    bind:this={popupEl}
    bind:offsetWidth={popupWidth}
    bind:offsetHeight={popupHeight}
    class="absolute z-40 max-w-lg min-w-[320px] bg-white/95 dark:bg-[#121215]/95 backdrop-blur-md border border-neutral-200 dark:border-neutral-800 text-neutral-900 dark:text-neutral-200 rounded-lg shadow-2xl overflow-hidden font-mono text-xs select-none animate-in fade-in zoom-in-95 duration-100"
    style="left: {clampedX}px; top: {clampedY}px;"
  >
    <!-- Header bar -->
    <div class="px-2.5 py-1.5 bg-neutral-100/90 dark:bg-neutral-900/80 border-b border-neutral-200 dark:border-neutral-700/50 flex items-center justify-between text-[10px] text-neutral-600 dark:text-neutral-400">
      <div class="flex items-center gap-1.5 font-semibold text-neutral-800 dark:text-neutral-300">
        <span class="w-1.5 h-1.5 rounded-full bg-sky-500 dark:bg-sky-400 animate-pulse"></span>
        <span>{t('terminal.smartAutocomplete')}</span>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-neutral-200 dark:bg-neutral-700/50 text-neutral-700 dark:text-neutral-300">{t('terminal.tabToComplete')}</span>
        <button
          onclick={onClose}
          class="hover:text-neutral-900 dark:hover:text-neutral-200 transition-colors p-0.5 rounded"
          title={t('terminal.dismiss')}
          aria-label={t('terminal.dismiss')}
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Suggestion items list -->
    <div class="max-h-56 overflow-y-auto divide-y divide-neutral-100 dark:divide-neutral-800/40 py-1">
      {#each suggestions as item, idx}
        <button
          type="button"
          class="w-full px-3 py-1.5 flex items-center justify-between gap-3 text-left transition-colors {idx === selectedIndex ? 'bg-sky-500/15 dark:bg-sky-600/30 text-sky-900 dark:text-white border-l-2 border-sky-500 dark:border-sky-400' : 'text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800/50'}"
          onclick={() => onSelect(item)}
        >
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <!-- Badge type -->
            {#if item.type === 'history'}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-amber-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/30 shrink-0">
                {t('terminal.badgeHistory')}
              </span>
            {:else if item.type === 'snippet'}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 shrink-0">
                {t('terminal.badgeSnippet')}
              </span>
            {:else}
              <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold bg-sky-500/20 text-sky-700 dark:text-sky-300 border border-sky-500/30 shrink-0">
                {t('terminal.badgeCommand')}
              </span>
            {/if}

            <span class="font-mono text-xs font-semibold truncate text-neutral-900 dark:text-white">
              {item.text}
            </span>
          </div>

          {#if item.desc || item.sourceLabel}
            <span class="text-[11px] text-neutral-500 dark:text-neutral-400 truncate max-w-[200px] text-right">
              {item.sourceLabel ? `[${item.sourceLabel}] ` : ''}{item.desc || ''}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}
