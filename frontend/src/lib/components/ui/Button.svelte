<script lang="ts">
  import type { Snippet } from 'svelte';

  type ButtonVariant = 'primary' | 'secondary' | 'row' | 'row-danger' | 'icon' | 'ghost';
  type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';

  let {
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    title = undefined,
    onclick = undefined,
    class: customClass = '',
    children
  }: {
    variant?: ButtonVariant;
    size?: ButtonSize;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    title?: string;
    onclick?: (e: MouseEvent) => void;
    class?: string;
    children?: Snippet;
  } = $props();

  const baseClasses = "inline-flex items-center justify-center font-medium transition-all select-none disabled:opacity-50 disabled:pointer-events-none active:scale-[0.98]";

  const variantClasses: Record<ButtonVariant, string> = {
    primary: "bg-sky-600 hover:bg-sky-500 text-white font-semibold shadow-md shadow-sky-600/20 active:scale-95",
    secondary: "border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 shadow-2xs",
    row: "bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium",
    'row-danger': "bg-rose-500/10 hover:bg-rose-600 text-rose-600 dark:text-rose-400 hover:text-white font-medium",
    icon: "border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 text-neutral-500 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 shadow-2xs",
    ghost: "text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800"
  };

  const sizeClasses: Record<ButtonSize, string> = {
    sm: "h-7 px-2.5 rounded-lg text-xs gap-1",
    md: "h-9 px-4 rounded-xl text-xs gap-1.5",
    lg: "h-10 px-5 rounded-xl text-sm gap-2",
    icon: "h-9 w-9 rounded-xl p-0"
  };
</script>

<button
  {type}
  {disabled}
  {title}
  {onclick}
  class="{baseClasses} {variantClasses[variant]} {variant === 'icon' ? sizeClasses.icon : sizeClasses[size]} {customClass}"
>
  {#if children}
    {@render children()}
  {/if}
</button>
