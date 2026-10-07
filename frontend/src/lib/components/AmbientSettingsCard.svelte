<script lang="ts">
  import { getAmbientStore, type AmbientMode, type AmbientEffect } from '$lib/stores/ambient.svelte';
  import { getPro } from '$lib/stores/pro.svelte';

  let { defaultAccent = '#64748b', appName = 'CAFramework' }: { defaultAccent?: string; appName?: string } = $props();

  const ambient = getAmbientStore();
  const pro = getPro();

  const PRESET_COLORS = [
    { name: 'Crimson Red', hex: '#ef4444' },
    { name: 'Cyan Blue', hex: '#06b6d4' },
    { name: 'Emerald Green', hex: '#10b981' },
    { name: 'Royal Indigo', hex: '#6366f1' },
    { name: 'Amber Gold', hex: '#f59e0b' },
    { name: 'Neon Purple', hex: '#a855f7' },
    { name: 'Rose Pink', hex: '#f43f5e' },
    { name: 'Slate Silver', hex: '#64748b' }
  ];
</script>

<div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-6 shadow-sm text-neutral-900 dark:text-white space-y-6">
  <!-- Header -->
  <div class="flex items-start justify-between gap-4">
    <div class="space-y-1">
      <div class="flex items-center gap-2">
        <h3 class="text-base font-semibold tracking-tight">Ambient Underglow & RGB Lighting</h3>
        {#if pro.isPro}
          <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-500 border border-amber-500/30 uppercase tracking-wide">
            PRO Founder
          </span>
        {:else}
          <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-neutral-200 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 uppercase tracking-wide">
            PRO Feature
          </span>
        {/if}
      </div>
      <p class="text-xs text-neutral-500 dark:text-neutral-400 max-w-xl">
        Pencahayaan halo luar di sekeliling area kerja dengan akselerasi GPU. Mendukung mode warna solid, aksen brand, dan spektrum RGB dinamis ala keyboard mekanik.
      </p>
    </div>

    <!-- Master Toggle -->
    <div class="flex items-center gap-3">
      <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">
        {ambient.config.enabled && pro.isPro ? 'Aktif' : 'Nonaktif'}
      </span>
      <button
        type="button"
        role="switch"
        aria-checked={ambient.config.enabled}
        disabled={!pro.isPro}
        onclick={() => ambient.setEnabled(!ambient.config.enabled)}
        class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none disabled:opacity-40 disabled:cursor-not-allowed {ambient.config.enabled && pro.isPro ? 'bg-sky-600' : 'bg-neutral-300 dark:bg-neutral-700'}"
      >
        <span
          class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {ambient.config.enabled && pro.isPro ? 'translate-x-5' : 'translate-x-0'}"
        ></span>
      </button>
    </div>
  </div>

  {#if !pro.isPro}
    <div class="p-4 rounded-xl border border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300 text-xs flex items-center gap-3">
      <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
      </svg>
      <span>Fitur kustomisasi Ambient Lighting eksklusif untuk lisensi <strong>PRO Founder Edition</strong>. Masuk dengan akun Pro untuk mengaktifkannya.</span>
    </div>
  {/if}

  {#if ambient.config.enabled && pro.isPro}
    <div class="space-y-6 pt-2 border-t border-neutral-200 dark:border-neutral-800">
      
      <!-- 1. Color Mode Selection -->
      <div class="space-y-2.5">
        <label class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          Mode Warna (Color Mode)
        </label>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
          <button
            type="button"
            onclick={() => ambient.setMode('app-accent')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'app-accent' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full border border-black/20" style:background-color={defaultAccent}></span>
              <span class="text-xs font-semibold">Tema {appName}</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">Bawaan warna aplikasi</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('solid')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'solid' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full border border-black/20" style:background-color={ambient.config.customColor}></span>
              <span class="text-xs font-semibold">Warna Solid</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">Pilih warna bebas</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('rgb-cycle')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'rgb-cycle' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full bg-[linear-gradient(135deg,#ff0055,#33cc33,#0099ff)]"></span>
              <span class="text-xs font-semibold">RGB Chroma</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">Keyboard mechanical cycle</p>
          </button>

          <button
            type="button"
            onclick={() => ambient.setMode('aurora')}
            class="p-3 rounded-xl border text-left transition-all {ambient.config.mode === 'aurora' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="flex items-center gap-2 mb-1">
              <span class="w-3 h-3 rounded-full bg-[radial-gradient(circle,#06b6d4,#8b5cf6)]"></span>
              <span class="text-xs font-semibold">Aurora Drift</span>
            </div>
            <p class="text-[10px] text-neutral-500 dark:text-neutral-400">Gelombang cahaya halus</p>
          </button>
        </div>
      </div>

      <!-- Solid Color Picker & Presets (Shown when mode == 'solid') -->
      {#if ambient.config.mode === 'solid'}
        <div class="p-4 rounded-xl border border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-950 space-y-3">
          <label class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
            Palet Warna Kustom
          </label>
          <div class="flex flex-wrap items-center gap-2">
            {#each PRESET_COLORS as preset}
              <button
                type="button"
                title={preset.name}
                onclick={() => ambient.setCustomColor(preset.hex)}
                class="w-7 h-7 rounded-full border-2 transition-transform hover:scale-110 flex items-center justify-center {ambient.config.customColor === preset.hex ? 'border-white ring-2 ring-sky-500 scale-110' : 'border-black/20'}"
                style:background-color={preset.hex}
              >
                {#if ambient.config.customColor === preset.hex}
                  <svg class="w-3.5 h-3.5 text-white drop-shadow" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                {/if}
              </button>
            {/each}

            <div class="flex items-center gap-2 ml-auto">
              <span class="text-xs text-neutral-400 font-mono">Hex:</span>
              <input
                type="color"
                value={ambient.config.customColor}
                oninput={(e) => ambient.setCustomColor((e.target as HTMLInputElement).value)}
                class="w-8 h-8 rounded-lg border border-neutral-300 dark:border-neutral-700 cursor-pointer bg-transparent"
              />
              <span class="text-xs font-mono px-2 py-1 rounded bg-neutral-200 dark:bg-neutral-800">
                {ambient.config.customColor}
              </span>
            </div>
          </div>
        </div>
      {/if}

      <!-- 2. Animation Effect Selection -->
      <div class="space-y-2.5">
        <label class="block text-xs font-semibold uppercase tracking-wider text-neutral-500 dark:text-neutral-400">
          Efek Animasi (Animation Effect)
        </label>
        <div class="grid grid-cols-3 gap-2.5">
          <button
            type="button"
            onclick={() => ambient.setEffect('static')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'static' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="text-xs">Statis (Steady)</div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400 mt-0.5">Cahaya konstan</div>
          </button>

          <button
            type="button"
            onclick={() => ambient.setEffect('breathing')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'breathing' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="text-xs">Bernafas (Pulse)</div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400 mt-0.5">Siklus redup & terang</div>
          </button>

          <button
            type="button"
            onclick={() => ambient.setEffect('wave')}
            class="p-3 rounded-xl border text-center transition-all {ambient.config.effect === 'wave' ? 'border-sky-500 bg-sky-500/10 text-sky-700 dark:text-sky-300 font-semibold' : 'border-neutral-200 dark:border-neutral-800 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
          >
            <div class="text-xs">Gelombang (Wave)</div>
            <div class="text-[10px] text-neutral-500 dark:text-neutral-400 mt-0.5">Aliran dinamis</div>
          </button>
        </div>
      </div>

      <!-- 3. Sliders (Intensity, Spread/Blur, Speed) -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
        
        <!-- Brightness / Intensity -->
        <div class="space-y-1.5">
          <div class="flex justify-between text-xs">
            <span class="text-neutral-500 dark:text-neutral-400">Kecerahan</span>
            <span class="font-mono font-semibold">{Math.round(ambient.config.intensity * 100)}%</span>
          </div>
          <input
            type="range"
            min="0.1"
            max="1.0"
            step="0.05"
            value={ambient.config.intensity}
            oninput={(e) => ambient.setIntensity(parseFloat((e.target as HTMLInputElement).value))}
            class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-sky-500"
          />
        </div>

        <!-- Spread / Blur Radius -->
        <div class="space-y-1.5">
          <div class="flex justify-between text-xs">
            <span class="text-neutral-500 dark:text-neutral-400">Jangkauan Pendar</span>
            <span class="font-mono font-semibold">{ambient.config.blurRadius}px</span>
          </div>
          <input
            type="range"
            min="8"
            max="45"
            step="1"
            value={ambient.config.blurRadius}
            oninput={(e) => ambient.setBlurRadius(parseInt((e.target as HTMLInputElement).value))}
            class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-sky-500"
          />
        </div>

        <!-- Speed -->
        <div class="space-y-1.5">
          <div class="flex justify-between text-xs">
            <span class="text-neutral-500 dark:text-neutral-400">Kecepatan Animasi</span>
            <span class="font-mono font-semibold">{ambient.config.speedSec}s</span>
          </div>
          <input
            type="range"
            min="1"
            max="15"
            step="0.5"
            value={ambient.config.speedSec}
            oninput={(e) => ambient.setSpeedSec(parseFloat((e.target as HTMLInputElement).value))}
            class="w-full h-1.5 bg-neutral-200 dark:bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-sky-500"
          />
        </div>
      </div>

    </div>
  {/if}
</div>
