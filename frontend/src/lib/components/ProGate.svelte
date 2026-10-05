<script lang="ts">
  import { isPro, isGrace, needsOnlineCheck, billingStore } from '$lib/stores/billing.svelte';
  import type { Snippet } from 'svelte';

  let {
    feature,
    fallback,
    children
  }: {
    feature: string;
    fallback?: Snippet;
    children: Snippet;
  } = $props();
</script>

{#if $isPro}
  {#if $isGrace}
    <div class="p-3 mb-4 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-between text-xs text-amber-600 dark:text-amber-400">
      <span>⚠️ Your subscription expired. You are in a 7-day grace period.</span>
      <a href="/settings?tab=account" class="font-semibold underline">Renew now</a>
    </div>
  {/if}
  {@render children()}
{:else if fallback}
  {@render fallback()}
{:else}
  <div class="p-6 rounded-2xl bg-neutral-900/50 border border-neutral-800 text-center space-y-3">
    <div class="w-10 h-10 rounded-full bg-indigo-500/10 text-indigo-400 flex items-center justify-center mx-auto">
      <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
    </div>
    <div>
      <h3 class="text-sm font-semibold text-white">Pro Feature: {feature}</h3>
      {#if $needsOnlineCheck}
        <p class="text-xs text-neutral-400 mt-1">Please reconnect online to verify your subscription.</p>
      {:else}
        <p class="text-xs text-neutral-400 mt-1">Upgrade to Pro to unlock unlimited access to this feature.</p>
      {/if}
    </div>
    <a
      href="/settings?tab=account"
      class="inline-block px-4 py-2 text-xs font-semibold rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white transition-colors"
    >
      Upgrade to Pro
    </a>
  </div>
{/if}
