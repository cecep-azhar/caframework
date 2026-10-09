<script lang="ts">
  let { active }: { active: 'routing' | 'personas' | 'habits' | 'skills' | 'quick' } = $props();
</script>

<div class="border-b border-neutral-200 dark:border-neutral-800 flex gap-4 overflow-x-auto text-sm font-medium" role="tablist">
  <a
    href="/settings/ai-routing"
    role="tab"
    aria-selected={active === 'routing'}
    class="pb-3 whitespace-nowrap transition-colors border-b-2 {active === 'routing'
      ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold'
      : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
  >
    Providers & Routing Matrix
  </a>
  <a
    href="/settings/ai-skills"
    role="tab"
    aria-selected={active === 'skills'}
    class="pb-3 whitespace-nowrap transition-colors border-b-2 {active === 'skills'
      ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold'
      : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
  >
    Custom Skills (@skills)
  </a>
  <a
    href="/settings/ai-personas"
    role="tab"
    aria-selected={active === 'personas'}
    class="pb-3 whitespace-nowrap transition-colors border-b-2 {active === 'personas'
      ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold'
      : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
  >
    Personas & Prompt Rules
  </a>
  <a
    href="/settings/ai-habits"
    role="tab"
    aria-selected={active === 'habits'}
    class="pb-3 whitespace-nowrap transition-colors border-b-2 {active === 'habits'
      ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold'
      : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
  >
    Persistent Habit Memory (FTS5)
  </a>
  <a
    href="/settings?tab=ai"
    role="tab"
    aria-selected={active === 'quick'}
    class="pb-3 whitespace-nowrap transition-colors border-b-2 {active === 'quick'
      ? 'border-sky-500 text-neutral-900 dark:text-white font-semibold'
      : 'border-transparent text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'}"
  >
    Quick AI Config
  </a>
</div>
