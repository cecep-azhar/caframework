<script lang="ts">
  import { getAmbientStore } from '$lib/stores/ambient.svelte';
  import { getPro } from '$lib/stores/pro.svelte';

  let { defaultAccent = '#64748b' }: { defaultAccent?: string } = $props();

  const ambient = getAmbientStore();
  const pro = getPro();

  // Active color based on mode
  const activeColor = $derived.by(() => {
    if (ambient.config.mode === 'app-accent') return defaultAccent;
    if (ambient.config.mode === 'solid') return ambient.config.customColor;
    return defaultAccent;
  });

  // Calculate underglow box shadow & gradient styles
  const isGradientMode = $derived(
    ambient.config.mode === 'rgb-cycle' || ambient.config.mode === 'aurora'
  );

  const glowBoxShadow = $derived.by(() => {
    if (!ambient.config.enabled || !pro.isPro) return 'none';
    if (isGradientMode) return 'none'; // Gradient mode uses background filter

    const blur = ambient.config.blurRadius;
    const spread = Math.round(blur / 4);
    const color = activeColor;
    return `0 0 ${blur}px ${spread}px ${color}, -2px -2px ${blur * 1.2}px ${spread}px ${color}`;
  });

  const animationDuration = $derived(`${ambient.config.speedSec}s`);
</script>

{#if ambient.config.enabled && pro.isPro}
  <div
    class="pointer-events-none absolute -inset-[2px] md:rounded-tl-2xl z-0 overflow-visible transition-opacity duration-300 will-change-[filter,opacity]"
    style:opacity={ambient.config.intensity}
    aria-hidden="true"
  >
    {#if isGradientMode}
      <!-- Gradient Underglow Canvas for RGB Chroma / Aurora -->
      <div
        class="w-full h-full md:rounded-tl-2xl will-change-[filter,transform] {ambient.config.mode === 'rgb-cycle' ? 'ambient-rgb-cycle' : 'ambient-aurora-wave'} {ambient.config.effect === 'breathing' ? 'ambient-breathe' : ''}"
        style:filter="blur({ambient.config.blurRadius}px)"
        style:animation-duration={animationDuration}
      ></div>
    {:else}
      <!-- Solid / App Accent Underglow Glow -->
      <div
        class="w-full h-full md:rounded-tl-2xl will-change-[filter,opacity] {ambient.config.effect === 'breathing' ? 'ambient-breathe' : ''} {ambient.config.effect === 'wave' ? 'ambient-pulse-slow' : ''}"
        style:box-shadow={glowBoxShadow}
        style:animation-duration={animationDuration}
      ></div>
    {/if}
  </div>
{/if}

<style>
  @keyframes ambient-breathe {
    0%, 100% {
      opacity: 0.35;
      transform: scale(0.998);
    }
    50% {
      opacity: 1;
      transform: scale(1.002);
    }
  }

  @keyframes ambient-pulse-slow {
    0%, 100% {
      opacity: 0.5;
    }
    50% {
      opacity: 0.95;
    }
  }

  @keyframes ambient-rgb-cycle {
    0% {
      background: linear-gradient(135deg, #ff0055, #ff9900, #33cc33, #0099ff, #cc00ff);
      filter: hue-rotate(0deg);
    }
    100% {
      background: linear-gradient(135deg, #ff0055, #ff9900, #33cc33, #0099ff, #cc00ff);
      filter: hue-rotate(360deg);
    }
  }

  @keyframes ambient-aurora-wave {
    0%, 100% {
      background: radial-gradient(circle at 10% 20%, #06b6d4, #3b82f6 50%, #8b5cf6 90%);
      transform: scale(1);
    }
    50% {
      background: radial-gradient(circle at 80% 80%, #10b981, #06b6d4 50%, #ec4899 90%);
      transform: scale(1.01);
    }
  }

  .ambient-breathe {
    animation: ambient-breathe var(--duration, 4s) ease-in-out infinite;
  }

  .ambient-pulse-slow {
    animation: ambient-pulse-slow var(--duration, 4s) ease-in-out infinite;
  }

  .ambient-rgb-cycle {
    animation: ambient-rgb-cycle var(--duration, 6s) linear infinite;
  }

  .ambient-aurora-wave {
    animation: ambient-aurora-wave var(--duration, 8s) ease-in-out infinite;
  }
</style>
