<script module lang="ts">
  /** The preset avatars a Free profile can pick from. Order is the picker order. */
  export const AVATARS = [
    { id: 'cat', label: 'Cat', from: '#f59e0b', to: '#ea580c' },
    { id: 'fox', label: 'Fox', from: '#fb923c', to: '#c2410c' },
    { id: 'mountain', label: 'Mountain', from: '#38bdf8', to: '#4f46e5' },
    { id: 'tree', label: 'Forest', from: '#34d399', to: '#047857' },
    { id: 'rocket', label: 'Rocket', from: '#0ea5e9', to: '#1e3a8a' },
    { id: 'code', label: 'Code', from: '#a78bfa', to: '#6d28d9' },
    { id: 'terminal', label: 'Terminal', from: '#525252', to: '#171717' },
    { id: 'bolt', label: 'Lightning', from: '#facc15', to: '#d97706' },
    { id: 'home', label: 'Home', from: '#f472b6', to: '#be185d' },
    { id: 'family', label: 'Family', from: '#2dd4bf', to: '#0f766e' }
  ] as const;
</script>

<script lang="ts">
  import { initialsOf } from '$lib/stores/profile.svelte';

  let {
    avatar,
    name = '',
    size = 32,
    pro = false,
    class: className = ''
  }: { avatar: string; name?: string; size?: number; pro?: boolean; class?: string } = $props();

  const preset = $derived(AVATARS.find((a) => a.id === avatar));
</script>

<span class="relative inline-flex shrink-0" style="width: {size}px; height: {size}px;">
  {#if pro}
    <span
      class="meteor-orbit"
      style="--meteor-stroke: {Math.max(2, Math.round(size * 0.05))}px;"
      aria-hidden="true"
    >
      <span class="meteor-arc meteor-arc--1"></span>
      <span class="meteor-arc meteor-arc--2"></span>
      <span class="meteor-arc meteor-arc--3"></span>
    </span>
  {/if}
  <span
    class="relative inline-flex items-center justify-center rounded-full shrink-0 overflow-hidden select-none {className}"
    style="width: {size}px; height: {size}px; background: linear-gradient(135deg, {preset?.from ?? '#0ea5e9'}, {preset?.to ?? '#10b981'});"
    aria-hidden="true"
  >
  {#if preset}
    <svg
      viewBox="0 0 24 24"
      width={Math.round(size * 0.6)}
      height={Math.round(size * 0.6)}
      fill="none"
      stroke="white"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      {#if preset.id === 'cat'}
        <path d="M6 10V4.5l3.5 3h5L18 4.5V10a6 6 0 0 1-12 0z" />
        <circle cx="9.5" cy="11" r="0.9" fill="white" stroke="none" />
        <circle cx="14.5" cy="11" r="0.9" fill="white" stroke="none" />
        <path d="M11 13.8h2l-1 1.2z M12 15v1.2" />
      {:else if preset.id === 'fox'}
        <path d="M4 5l4 4h8l4-4v6l-8 9-8-9z" />
        <circle cx="9" cy="12" r="0.9" fill="white" stroke="none" />
        <circle cx="15" cy="12" r="0.9" fill="white" stroke="none" />
        <circle cx="12" cy="16.5" r="0.9" fill="white" stroke="none" />
      {:else if preset.id === 'mountain'}
        <path d="M3 19l6-9 4 5 2-3 6 7z" />
        <circle cx="17" cy="6.5" r="2" />
      {:else if preset.id === 'tree'}
        <path d="M12 3l6 8h-3l4 5H5l4-5H6z" />
        <path d="M12 16v5" />
      {:else if preset.id === 'rocket'}
        <path d="M12 3c3 2.5 4.5 6 4.5 10l-2 3h-5l-2-3c0-4 1.5-7.5 4.5-10z" />
        <circle cx="12" cy="10" r="1.6" />
        <path d="M9.5 16l-2 3.5M14.5 16l2 3.5M12 17v4" />
      {:else if preset.id === 'code'}
        <path d="M8 8l-4 4 4 4M16 8l4 4-4 4M13.5 6l-3 12" />
      {:else if preset.id === 'terminal'}
        <rect x="3" y="5" width="18" height="14" rx="2" />
        <path d="M7 10l3 2-3 2M12 15h5" />
      {:else if preset.id === 'bolt'}
        <path d="M13 2L4 14h7l-1 8 9-12h-7z" fill="white" />
      {:else if preset.id === 'home'}
        <path d="M3 11l9-7 9 7" />
        <path d="M5 10v10h14V10" />
        <path d="M10 20v-5h4v5" />
      {:else if preset.id === 'family'}
        <circle cx="7" cy="8" r="2.2" />
        <circle cx="17" cy="8" r="2.2" />
        <circle cx="12" cy="12.5" r="1.8" />
        <path d="M3 20v-1.5A4 4 0 0 1 7 14.5a4 4 0 0 1 2.6 1M21 20v-1.5a4 4 0 0 0-4-4 4 4 0 0 0-2.6 1M9 20v-.5a3 3 0 0 1 6 0v.5" />
      {/if}
    </svg>
  {:else}
    <span class="font-semibold text-white" style="font-size: {Math.round(size * 0.4)}px;">{initialsOf(name)}</span>
  {/if}
  </span>
</span>

<style>
  .meteor-orbit {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }
  /* Each arc is a conic-gradient comet trail, clipped to a thin ring band so the
     head + fading tail are drawn curving along the circle itself, not a straight line. */
  .meteor-arc {
    position: absolute;
    inset: -10%;
    border-radius: 9999px;
    -webkit-mask: radial-gradient(
      closest-side,
      transparent calc(100% - var(--meteor-stroke, 3px) - 1px),
      #000 calc(100% - var(--meteor-stroke, 3px)),
      #000 100%,
      transparent 100%
    );
    mask: radial-gradient(
      closest-side,
      transparent calc(100% - var(--meteor-stroke, 3px) - 1px),
      #000 calc(100% - var(--meteor-stroke, 3px)),
      #000 100%,
      transparent 100%
    );
  }
  .meteor-arc--1 {
    background: conic-gradient(
      from 0deg,
      transparent 0deg,
      transparent 322deg,
      rgba(56, 189, 248, 0) 330deg,
      rgba(125, 211, 252, 0.9) 350deg,
      #f0f9ff 358deg,
      transparent 360deg
    );
    animation: meteor-orbit-spin 2.4s linear infinite;
  }
  .meteor-arc--2 {
    background: conic-gradient(
      from 130deg,
      transparent 0deg,
      transparent 326deg,
      rgba(56, 189, 248, 0) 333deg,
      rgba(125, 211, 252, 0.85) 350deg,
      #e0f2fe 358deg,
      transparent 360deg
    );
    animation: meteor-orbit-spin 4.1s linear infinite;
  }
  .meteor-arc--3 {
    background: conic-gradient(
      from 250deg,
      transparent 0deg,
      transparent 330deg,
      rgba(56, 189, 248, 0) 336deg,
      rgba(125, 211, 252, 0.8) 350deg,
      #dbeafe 358deg,
      transparent 360deg
    );
    animation: meteor-orbit-spin 6.3s linear infinite;
  }
  @keyframes meteor-orbit-spin {
    to {
      transform: rotate(360deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .meteor-arc {
      animation: none;
    }
  }
</style>
