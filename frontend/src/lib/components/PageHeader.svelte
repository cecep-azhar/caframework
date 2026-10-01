<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    subtitle,
    children
  }: {
    title: string;
    subtitle?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="flex items-center justify-between pb-4 border-b border-neutral-800/80">
  <div>
    <h2 class="text-xl font-bold tracking-tight text-white">{title}</h2>
    {#if subtitle}
      <p class="text-xs text-neutral-400 mt-0.5">{subtitle}</p>
    {/if}
  </div>

  {#if children}
    <div class="flex items-center gap-2">
      {@render children()}
    </div>
  {/if}
</div>
