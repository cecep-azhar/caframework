<script lang="ts">
  interface Props {
    message?: string;
  }

  let { message = 'Loading...' }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-16 px-4 text-center">
  <div class="relative w-10 h-10 mb-4">
    <div class="w-10 h-10 rounded-full border-2 border-indigo-500/20 border-t-indigo-500 animate-spin"></div>
  </div>
  <p class="text-sm text-zinc-400 font-medium">{message}</p>
</div>
