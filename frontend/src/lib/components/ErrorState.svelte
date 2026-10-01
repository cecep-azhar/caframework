<script lang="ts">
  interface Props {
    title?: string;
    message?: string;
    code?: string;
    retryLabel?: string;
    onretry?: () => void;
  }

  let {
    title = 'Something went wrong',
    message = 'An error occurred while loading this section.',
    code,
    retryLabel = 'Try again',
    onretry,
  }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 px-4 text-center">
  <div class="w-16 h-16 rounded-2xl bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400 mb-4 shadow-inner">
    <svg class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
    </svg>
  </div>

  <h3 class="text-base font-medium text-zinc-200">{title}</h3>
  <p class="text-sm text-zinc-400 mt-1 max-w-sm">{message}</p>

  {#if code}
    <div class="mt-3 px-3 py-1 rounded bg-zinc-900 border border-zinc-800 text-xs font-mono text-rose-400/90">
      Error Code: {code}
    </div>
  {/if}

  {#if onretry}
    <button
      onclick={onretry}
      class="mt-5 inline-flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 rounded-lg text-sm font-medium transition border border-zinc-700 cursor-pointer"
    >
      <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      {retryLabel}
    </button>
  {/if}
</div>
