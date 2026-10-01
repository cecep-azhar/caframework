<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import ProfileAvatar, { AVATARS } from './ProfileAvatar.svelte';
  import { onMount } from 'svelte';

  let { value = $bindable('rocket'), size = 48, centered = true }: { value: string; size?: number; centered?: boolean } = $props();

  let isOpen = $state(false);
  let containerRef = $state<HTMLDivElement | null>(null);

  function togglePicker(e: MouseEvent) {
    e.stopPropagation();
    isOpen = !isOpen;
  }

  function selectAvatar(id: string, e: MouseEvent) {
    e.stopPropagation();
    value = id;
    isOpen = false;
  }

  function handleClickOutside(e: MouseEvent) {
    if (isOpen && containerRef && !containerRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && isOpen) {
      isOpen = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleKeydown);
    return () => {
      document.removeEventListener('click', handleClickOutside);
      document.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<div
  bind:this={containerRef}
  class="relative inline-flex flex-col {centered ? 'items-center justify-center' : 'items-start'} no-drag"
>
  <!-- Main Centered Avatar Button -->
  <button
    type="button"
    onclick={togglePicker}
    aria-haspopup="dialog"
    aria-expanded={isOpen}
    aria-label={t('avatars.pickerLabel')}
    title={t('avatars.pickerLabel')}
    class="relative group rounded-full p-1 ring-2 ring-transparent hover:ring-sky-500/50 dark:hover:ring-sky-400/50 focus:outline-none focus:ring-sky-500 transition-all duration-200 cursor-pointer"
  >
    <div class="relative flex items-center justify-center">
      <ProfileAvatar avatar={value} {size} />
      <!-- Edit Badge Icon -->
      <div
        class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full bg-sky-500 text-white flex items-center justify-center shadow-md border-2 border-white dark:border-neutral-900 group-hover:scale-110 group-hover:bg-sky-400 transition-transform"
        aria-hidden="true"
      >
        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
        </svg>
      </div>
    </div>
  </button>

  <!-- Dropdown / Popover for selecting avatars -->
  {#if isOpen}
    <div
      class="absolute top-full mt-3 z-50 p-3 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl shadow-2xl animate-in fade-in zoom-in-95 duration-150 w-64"
      role="dialog"
      aria-label={t('avatars.pickerLabel')}
    >
      <div class="text-[11px] font-semibold uppercase tracking-wider text-neutral-400 dark:text-neutral-500 mb-2 px-1 text-center">
        {t('avatars.pickerLabel')}
      </div>
      <div class="grid grid-cols-5 gap-2" role="radiogroup">
        {#each AVATARS as option (option.id)}
          <button
            type="button"
            role="radio"
            aria-checked={value === option.id}
            aria-label={t(`avatars.${option.id}`)}
            title={t(`avatars.${option.id}`)}
            onclick={(e) => selectAvatar(option.id, e)}
            class="justify-self-center flex items-center justify-center rounded-full p-1 ring-2 ring-offset-2 ring-offset-white dark:ring-offset-neutral-900 transition-all cursor-pointer {value === option.id ? 'ring-sky-500 scale-105 shadow-sm' : 'ring-transparent hover:ring-neutral-300 dark:hover:ring-neutral-700 hover:scale-105'}"
          >
            <ProfileAvatar avatar={option.id} size={32} />
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
