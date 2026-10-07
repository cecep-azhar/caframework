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
    if (!ambient.config.enabled || !ambient.config.cardGlowEnabled || !pro.isPro) return 'none';
    if (isGradientMode) return 'none';

    const blur = ambient.config.blurRadius;
    const spread = Math.round(blur / 4);
    const color = activeColor;

    if (ambient.config.cardGlowStyle === 'neon-border') {
      return `-1px -1px 0px 1px ${color}, 0 0 ${Math.max(6, Math.round(blur * 0.4))}px 1px ${color}, -2px -2px ${blur}px 2px ${color}`;
    }

    // diffused-halo (default)
    return `0 0 ${blur}px ${spread}px ${color}, -3px -3px ${blur * 1.4}px ${spread}px ${color}`;
  });

  const animationDuration = $derived(`${ambient.config.speedSec}s`);
</script>

{#if ambient.config.enabled && ambient.config.cardGlowEnabled && pro.isPro}
  <div
    class="pointer-events-none absolute -inset-[2px] md:rounded-tl-xl z-0 overflow-visible transition-opacity duration-300 will-change-[filter,opacity]"
    style:opacity={ambient.config.intensity}
    aria-hidden="true"
  >
    {#if ambient.config.cardGlowStyle === 'chroma-beam'}
      <!-- Chroma Border Beam: Dynamic rotating conic laser sweep around the rounded corner -->
      <div class="relative w-full h-full md:rounded-tl-xl overflow-hidden p-[1.5px]">
        <div
          class="absolute -inset-[100%] card-glow-spin"
          style:animation-duration={animationDuration}
          style:background={isGradientMode
            ? 'conic-gradient(from 0deg, transparent 0deg, transparent 200deg, #ff0055 240deg, #33cc33 280deg, #0099ff 320deg, #cc00ff 360deg)'
            : `conic-gradient(from 0deg, transparent 0deg, transparent 240deg, ${activeColor}88 300deg, ${activeColor} 360deg)`}
        ></div>
        <!-- Inner mask to let the border beam glow outward while matching the main card corner -->
        <div class="w-full h-full md:rounded-tl-xl bg-transparent"></div>
      </div>
    {:else if isGradientMode}
      <!-- Gradient Underglow Canvas for RGB Chroma / Aurora -->
      <div
        class="w-full h-full md:rounded-tl-xl will-change-[filter,transform] {ambient.config.mode === 'rgb-cycle' ? 'ambient-rgb-cycle' : 'ambient-aurora-wave'} {ambient.config.effect === 'breathing' ? 'ambient-breathe' : ''}"
        style:filter="blur({ambient.config.cardGlowStyle === 'neon-border' ? Math.round(ambient.config.blurRadius * 0.35) : ambient.config.blurRadius}px)"
        style:animation-duration={animationDuration}
      ></div>
    {:else}
      <!-- Solid / App Accent Underglow Glow (Diffused Halo or Neon Border) -->
      <div
        class="w-full h-full md:rounded-tl-xl will-change-[filter,opacity] {ambient.config.effect === 'breathing' ? 'ambient-breathe' : ''} {ambient.config.effect === 'wave' ? 'ambient-pulse-slow' : ''}"
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
      background: radial-gradient(circle at 10% 20%, #ef4444, #f97316 50%, #8b5cf6 90%);
      transform: scale(1);
    }
    50% {
      background: radial-gradient(circle at 80% 80%, #f59e0b, #ef4444 50%, #ec4899 90%);
      transform: scale(1.01);
    }
  }

  @keyframes card-glow-spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  .card-glow-spin {
    animation: card-glow-spin var(--duration, 4s) linear infinite;
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
