<script lang="ts">
  export let size: number | string = 28;
  export let mode: 'light' | 'dark' | 'auto' | 'brand' | 'white' = 'brand';
  export let className: string = '';
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 512 512"
  class="shrink-0 {className}"
  xmlns="http://www.w3.org/2000/svg"
>
  <defs>
    <linearGradient id="cafLogoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#06B6D4" />
      <stop offset="100%" stop-color="#6366F1" />
    </linearGradient>
  </defs>

  <!-- Isometric Crystalline Cube Skeleton / Hollow 3 Faces -->
  <g fill="none">
    <path
      d="M256 104 L386 179 L386 329 L256 404 L126 329 L126 179 Z"
      stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : 'url(#cafLogoGrad)'}
      stroke-width="28"
      stroke-linejoin="round"
      stroke-linecap="round"
    />
    <path
      d="M256 254 L256 104"
      stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : '#06B6D4'}
      stroke-width="28"
      stroke-linecap="round"
    />
    <path
      d="M256 254 L386 329"
      stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : '#6366F1'}
      stroke-width="28"
      stroke-linecap="round"
    />
    <path
      d="M256 254 L126 329"
      stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : '#06B6D4'}
      stroke-width="28"
      stroke-linecap="round"
    />
  </g>
</svg>
