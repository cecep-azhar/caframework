<script module lang="ts">
  /**
   * One literal Tailwind class string per accent, never built from a template — Tailwind's JIT
   * scanner only picks up classes it can see verbatim in source, so `bg-${color}-500/10` would
   * silently produce no styles at all in a production build.
   */
  const ACCENTS = {
    violet: 'bg-violet-500/10 dark:bg-violet-500/20 border-violet-500/30 text-violet-600 dark:text-violet-400',
    sky: 'bg-sky-500/10 dark:bg-sky-500/20 border-sky-500/30 text-sky-600 dark:text-sky-400',
    emerald: 'bg-emerald-500/10 dark:bg-emerald-500/20 border-emerald-500/30 text-emerald-600 dark:text-emerald-400',
    amber: 'bg-amber-500/10 dark:bg-amber-500/20 border-amber-500/30 text-amber-600 dark:text-amber-400',
    rose: 'bg-rose-500/10 dark:bg-rose-500/20 border-rose-500/30 text-rose-600 dark:text-rose-400',
    cyan: 'bg-cyan-500/10 dark:bg-cyan-500/20 border-cyan-500/30 text-cyan-600 dark:text-cyan-400',
    indigo: 'bg-indigo-500/10 dark:bg-indigo-500/20 border-indigo-500/30 text-indigo-600 dark:text-indigo-400',
    teal: 'bg-teal-500/10 dark:bg-teal-500/20 border-teal-500/30 text-teal-600 dark:text-teal-400',
    orange: 'bg-orange-500/10 dark:bg-orange-500/20 border-orange-500/30 text-orange-600 dark:text-orange-400',
    pink: 'bg-pink-500/10 dark:bg-pink-500/20 border-pink-500/30 text-pink-600 dark:text-pink-400'
  } as const;
  export type PageHeaderAccent = keyof typeof ACCENTS;
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    icon,
    accent = 'sky',
    title,
    badge,
    subtitle,
    actions
  }: {
    /** SVG path `d` attribute(s) — a single string, or several for a multi-path icon. */
    icon: string | string[];
    accent?: PageHeaderAccent;
    title: string;
    /** Small pill next to the title, e.g. "AI Ops Assistant". */
    badge?: string;
    subtitle?: string;
    /** Right-aligned controls (host pickers, primary buttons…), same slot Prompt Studio uses. */
    actions?: Snippet;
  } = $props();

  const paths = $derived(Array.isArray(icon) ? icon : [icon]);
  const accentClass = $derived(ACCENTS[accent]);
</script>

<!-- The header pattern every workspace page shares (originally Prompt Studio's): an accent icon
     tile, title + optional pill badge, a one-line subtitle, and room for page-specific actions
     on the right. Kept as one component so the pattern can't drift page to page. -->
<header class="flex flex-col md:flex-row md:items-center justify-between gap-4 pb-4 border-b border-neutral-200 dark:border-neutral-800/80 mb-6">
  <div class="flex items-center gap-3 min-w-0">
    <div class="w-10 h-10 rounded-xl border flex items-center justify-center shrink-0 {accentClass}">
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        {#each paths as d}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" {d} />
        {/each}
      </svg>
    </div>
    <div class="min-w-0">
      <div class="flex items-center gap-2 flex-wrap">
        <h1 class="text-xl md:text-2xl font-bold text-neutral-900 dark:text-white tracking-tight">{title}</h1>
        {#if badge}
          <span class="text-xs px-2 py-0.5 rounded-full font-semibold border {accentClass}">{badge}</span>
        {/if}
      </div>
      {#if subtitle}
        <p class="text-xs md:text-sm text-neutral-500 dark:text-neutral-400">{subtitle}</p>
      {/if}
    </div>
  </div>

  {#if actions}
    <div class="flex items-center gap-3 shrink-0">
      {@render actions()}
    </div>
  {/if}
</header>
