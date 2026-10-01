<script lang="ts">
  export let size: number | string = 28;
  export let mode: 'light' | 'dark' | 'auto' | 'brand' | 'white' = 'brand';
  export let className: string = '';
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 1334 946"
  fill="currentColor"
  class="shrink-0 {mode === 'white' ? 'text-white' : mode === 'light' ? 'text-black' : 'text-[#ff0000]'} {className}"
  xmlns="http://www.w3.org/2000/svg"
>
  <!-- Left Segment -->
  <path
    fill-rule="evenodd"
    clip-rule="evenodd"
    d="M581 0L0 301V644L581 945V546L421 473L581 399V0ZM529 86L52 332V613L529 859V574L308 473L529 371V86Z"
  />
  <!-- Right Segment -->
  <path
    fill-rule="evenodd"
    clip-rule="evenodd"
    d="M753 0L1334 301V644L753 945V546L913 473L753 399V0ZM805 86L1282 332V613L805 859V574L1026 473L805 371V86Z"
  />
</svg>
