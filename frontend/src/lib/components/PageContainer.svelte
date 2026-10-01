<script lang="ts">
  interface Props {
    /** Skip internal padding — use for full-bleed layouts like terminal grid */
    noPadding?: boolean;
    class?: string;
    children: import('svelte').Snippet;
  }
  let { noPadding = false, class: cls = '', children }: Props = $props();
</script>

<div
  class={`flex flex-col flex-1 min-h-0 rounded-2xl bg-neutral-50 dark:bg-[#12161f] border border-neutral-200 dark:border-white/[0.06] overflow-hidden ${noPadding ? '' : 'p-4'} ${cls}`}
>
  {@render children()}
</div>
