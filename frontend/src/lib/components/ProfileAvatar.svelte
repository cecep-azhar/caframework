<script module lang="ts">
  /** The preset avatars a profile can pick from (30 comprehensive icons including 20 Pro Premium icons). */
  export const AVATARS = [
    // 10 Core / Free Avatars
    { id: 'cat', label: 'Cat', from: '#f59e0b', to: '#ea580c', pro: false },
    { id: 'fox', label: 'Fox', from: '#fb923c', to: '#c2410c', pro: false },
    { id: 'mountain', label: 'Mountain', from: '#38bdf8', to: '#4f46e5', pro: false },
    { id: 'tree', label: 'Forest', from: '#34d399', to: '#047857', pro: false },
    { id: 'rocket', label: 'Rocket', from: '#0ea5e9', to: '#1e3a8a', pro: false },
    { id: 'code', label: 'Code', from: '#a78bfa', to: '#6d28d9', pro: false },
    { id: 'terminal', label: 'Terminal', from: '#525252', to: '#171717', pro: false },
    { id: 'bolt', label: 'Lightning', from: '#facc15', to: '#d97706', pro: false },
    { id: 'home', label: 'Home', from: '#f472b6', to: '#be185d', pro: false },
    { id: 'family', label: 'Family', from: '#2dd4bf', to: '#0f766e', pro: false },

    // 20 Pro Premium Avatars (Marked with PRO badge)
    { id: 'crown', label: 'Crown', from: '#fbbf24', to: '#b45309', pro: true },
    { id: 'diamond', label: 'Diamond', from: '#38bdf8', to: '#0284c7', pro: true },
    { id: 'shield', label: 'Shield', from: '#10b981', to: '#047857', pro: true },
    { id: 'zap', label: 'Energy', from: '#f59e0b', to: '#dc2626', pro: true },
    { id: 'cpu', label: 'Processor', from: '#8b5cf6', to: '#4c1d95', pro: true },
    { id: 'brain', label: 'Neural AI', from: '#6366f1', to: '#312e81', pro: true },
    { id: 'orbit', label: 'Orbit', from: '#06b6d4', to: '#0e7490', pro: true },
    { id: 'fire', label: 'Flame', from: '#f43f5e', to: '#9f1239', pro: true },
    { id: 'star', label: 'Star', from: '#eab308', to: '#a16207', pro: true },
    { id: 'heart', label: 'Heart', from: '#ec4899', to: '#9d174d', pro: true },
    { id: 'sparkles', label: 'Sparkles', from: '#c084fc', to: '#7e22ce', pro: true },
    { id: 'cube', label: 'Matrix Cube', from: '#64748b', to: '#1e293b', pro: true },
    { id: 'compass', label: 'Compass', from: '#14b8a6', to: '#0f766e', pro: true },
    { id: 'feather', label: 'Feather', from: '#2dd4bf', to: '#059669', pro: true },
    { id: 'key', label: 'Master Key', from: '#d97706', to: '#78350f', pro: true },
    { id: 'atom', label: 'Atom', from: '#3b82f6', to: '#1d4ed8', pro: true },
    { id: 'ghost', label: 'Cyber Ghost', from: '#a855f7', to: '#581c87', pro: true },
    { id: 'globe', label: 'Global Mesh', from: '#10b981', to: '#065f46', pro: true },
    { id: 'eye', label: 'Cyber Eye', from: '#d946ef', to: '#701a75', pro: true },
    { id: 'target', label: 'Radar Target', from: '#ef4444', to: '#991b1b', pro: true }
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

<span class="relative inline-flex items-center justify-center shrink-0" style="width: {size}px; height: {size}px;">
  {#if pro}
    <!-- Glowing Aurora Rotating Halo -->
    <span class="pro-glow-ring" aria-hidden="true"></span>
    <!-- Pulsing Cyan/Violet Edge Ring -->
    <span class="pro-pulse-aura" aria-hidden="true"></span>
    <!-- Meteor Orbit Trails -->
    <span
      class="meteor-orbit"
      style="--meteor-stroke: {Math.max(2, Math.round(size * 0.06))}px;"
      aria-hidden="true"
    >
      <span class="meteor-arc meteor-arc--1"></span>
      <span class="meteor-arc meteor-arc--2"></span>
      <span class="meteor-arc meteor-arc--3"></span>
    </span>
  {/if}
  <span
    class="relative z-10 inline-flex items-center justify-center rounded-full shrink-0 overflow-hidden select-none {className}"
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
      {:else if preset.id === 'crown'}
        <path d="M5 18h14l1-11-4.5 4.5L12 4l-4.5 7.5L3 7z" fill="white" />
        <circle cx="12" cy="3" r="1" fill="white" />
        <circle cx="3" cy="6" r="1" fill="white" />
        <circle cx="21" cy="6" r="1" fill="white" />
      {:else if preset.id === 'diamond'}
        <path d="M6 3h12l4 6-10 12L2 9z" />
        <path d="M2 9h20M12 21L8 9M12 21l4-12M6 3l2 6M18 3l-2 6" />
      {:else if preset.id === 'shield'}
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
        <path d="M9 12l2 2 4-4" />
      {:else if preset.id === 'zap'}
        <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
      {:else if preset.id === 'cpu'}
        <rect x="4" y="4" width="16" height="16" rx="2" />
        <rect x="9" y="9" width="6" height="6" />
        <path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3" />
      {:else if preset.id === 'brain'}
        <path d="M9.5 2A4.5 4.5 0 0 0 5 6.5c0 1.2.5 2.3 1.3 3.1A4.5 4.5 0 0 0 5 13.5 4.5 4.5 0 0 0 9.5 18c.2 0 .3 0 .5-.03V2z" />
        <path d="M14.5 2A4.5 4.5 0 0 1 19 6.5c0 1.2-.5 2.3-1.3 3.1A4.5 4.5 0 0 1 19 13.5 4.5 4.5 0 0 1 14.5 18c-.2 0-.3 0-.5-.03V2z" />
      {:else if preset.id === 'orbit'}
        <circle cx="12" cy="12" r="3" fill="white" />
        <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(-30 12 12)" />
        <circle cx="20" cy="8" r="1.5" fill="white" />
      {:else if preset.id === 'fire'}
        <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" fill="white" />
      {:else if preset.id === 'star'}
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" fill="white" />
      {:else if preset.id === 'heart'}
        <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" fill="white" />
      {:else if preset.id === 'sparkles'}
        <path d="M12 3l1.5 4.5L18 9l-4.5 1.5L12 15l-1.5-4.5L6 9l4.5-1.5z" fill="white" />
        <path d="M18 15l1 2.5L21.5 18.5 19 19.5 18 22l-1-2.5-2.5-1 2.5-1z" fill="white" />
        <path d="M5 16l.8 1.7L7.5 18.5 5.8 19.3 5 21l-.8-1.7L2.5 18.5 4.2 17.7z" fill="white" />
      {:else if preset.id === 'cube'}
        <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
        <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
        <line x1="12" y1="22.08" x2="12" y2="12" />
      {:else if preset.id === 'compass'}
        <circle cx="12" cy="12" r="10" />
        <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76" fill="white" />
      {:else if preset.id === 'feather'}
        <path d="M20.24 12.24a6 6 0 0 0-8.49-8.49L5 10.5V19h8.5z" />
        <line x1="16" y1="8" x2="2" y2="22" />
        <line x1="17.5" y1="15" x2="9" y2="15" />
      {:else if preset.id === 'key'}
        <path d="M21 2l-2 2m-1.5 1.5L14 9a5 5 0 1 0-7 7 5 5 0 0 0 7-7l3.5-3.5L19 4l2-2z" />
        <circle cx="7.5" cy="16.5" r="1.5" />
      {:else if preset.id === 'atom'}
        <circle cx="12" cy="12" r="2" fill="white" />
        <ellipse cx="12" cy="12" rx="9" ry="3.5" transform="rotate(30 12 12)" />
        <ellipse cx="12" cy="12" rx="9" ry="3.5" transform="rotate(-30 12 12)" />
        <ellipse cx="12" cy="12" rx="9" ry="3.5" transform="rotate(90 12 12)" />
      {:else if preset.id === 'ghost'}
        <path d="M9 10h.01M15 10h.01M12 2a8 8 0 0 0-8 8v12l3-3 2.5 2.5L12 19l2.5 2.5L17 19l3 3V10a8 8 0 0 0-8-8z" />
      {:else if preset.id === 'globe'}
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
      {:else if preset.id === 'eye'}
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
        <circle cx="12" cy="12" r="3" fill="white" />
      {:else if preset.id === 'target'}
        <circle cx="12" cy="12" r="10" />
        <circle cx="12" cy="12" r="6" />
        <circle cx="12" cy="12" r="2" fill="white" />
      {/if}
    </svg>
  {:else}
    <span class="font-semibold text-white" style="font-size: {Math.round(size * 0.4)}px;">{initialsOf(name)}</span>
  {/if}
  </span>
</span>

<style>
  /* Radiant Glowing Aura */
  .pro-glow-ring {
    position: absolute;
    inset: -3px;
    border-radius: 9999px;
    background: conic-gradient(from 0deg, #38bdf8, #818cf8, #c084fc, #f472b6, #fb923c, #38bdf8);
    filter: blur(3.5px);
    opacity: 0.85;
    animation: pro-spin-glow 3.5s linear infinite;
    z-index: 0;
  }
  .pro-pulse-aura {
    position: absolute;
    inset: -2px;
    border-radius: 9999px;
    border: 1.5px solid rgba(56, 189, 248, 0.9);
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.8), inset 0 0 5px rgba(192, 132, 252, 0.6);
    animation: pro-pulse-ring 2.2s ease-in-out infinite alternate;
    z-index: 1;
  }
  @keyframes pro-spin-glow {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
  @keyframes pro-pulse-ring {
    0% { transform: scale(0.98); opacity: 0.7; }
    100% { transform: scale(1.06); opacity: 1; }
  }

  .meteor-orbit {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 2;
  }
  /* Each arc is a conic-gradient comet trail, clipped to a thin ring band so the
     head + fading tail are drawn curving along the circle itself, not a straight line. */
  .meteor-arc {
    position: absolute;
    inset: -12%;
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
    .pro-glow-ring, .pro-pulse-aura, .meteor-arc {
      animation: none;
    }
  }
</style>
