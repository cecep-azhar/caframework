// PRO Ambient Underglow & RGB Chroma Store (CADS Standard)
// Persisted in localStorage for instant Zero-Latency hydration.

export type AmbientMode = 'app-accent' | 'solid' | 'rgb-cycle' | 'aurora';
export type AmbientEffect = 'static' | 'breathing' | 'wave';

export interface AmbientConfig {
  enabled: boolean;
  mode: AmbientMode;
  customColor: string;
  effect: AmbientEffect;
  intensity: number; // 0.1 to 1.0
  blurRadius: number; // 8 to 40 px
  speedSec: number; // 2 to 15 s
}

const STORAGE_KEY = 'caf-ambient-lighting-v1';

const DEFAULT_CONFIG: AmbientConfig = {
  enabled: true,
  mode: 'app-accent',
  customColor: '#64748b',
  effect: 'breathing',
  intensity: 0.65,
  blurRadius: 20,
  speedSec: 4
};

function loadStoredConfig(): AmbientConfig {
  if (typeof localStorage === 'undefined') return { ...DEFAULT_CONFIG };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_CONFIG };
    return { ...DEFAULT_CONFIG, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

class AmbientStore {
  config = $state<AmbientConfig>(loadStoredConfig());

  save() {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(this.config));
      } catch {}
    }
  }

  setEnabled(val: boolean) {
    this.config.enabled = val;
    this.save();
  }

  setMode(val: AmbientMode) {
    this.config.mode = val;
    this.save();
  }

  setCustomColor(val: string) {
    this.config.customColor = val;
    this.save();
  }

  setEffect(val: AmbientEffect) {
    this.config.effect = val;
    this.save();
  }

  setIntensity(val: number) {
    this.config.intensity = Math.max(0.1, Math.min(1.0, val));
    this.save();
  }

  setBlurRadius(val: number) {
    this.config.blurRadius = Math.max(6, Math.min(50, val));
    this.save();
  }

  setSpeedSec(val: number) {
    this.config.speedSec = Math.max(1, Math.min(20, val));
    this.save();
  }

  reset() {
    this.config = { ...DEFAULT_CONFIG };
    this.save();
  }
}

let instance: AmbientStore | null = null;

export function getAmbientStore(): AmbientStore {
  if (!instance) {
    instance = new AmbientStore();
  }
  return instance;
}
