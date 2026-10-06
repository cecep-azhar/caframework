<script lang="ts">
  // The single AI configuration form. It used to live only inside a modal in Prompt Studio,
  // which meant the AI credentials were the one setting you could not find on the Settings
  // page. Extracted as a component so Settings owns it and everything else links there.
  import { onMount } from 'svelte';
  import { aiChat, getAiSettings, saveAiSettings, type AiSettings } from '$lib/api/ai';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';

  let { onSaved }: { onSaved?: (settings: AiSettings) => void } = $props();

  const DEFAULT_SETTINGS: AiSettings = {
    provider: 'custom',
    base_url: 'http://100.76.150.46:3007/v1',
    api_key: '',
    model: 'claude-3-5-sonnet'
  };

  let settings = $state<AiSettings>({ ...DEFAULT_SETTINGS });
  let showApiKey = $state(false);
  let isSaving = $state(false);
  let isLoading = $state(true);

  // A connection that fails silently is what made the assistant look broken for no visible
  // reason, so the settings page can prove the endpoint works before you rely on it.
  let isTesting = $state(false);
  let testResult = $state<{ ok: boolean; text: string } | null>(null);

  onMount(async () => {
    try {
      const saved = await getAiSettings();
      settings = { ...DEFAULT_SETTINGS, ...saved };
    } catch {
      // Backend unavailable (e.g. browser preview): keep defaults, the save will report.
    } finally {
      isLoading = false;
    }
  });

  async function handleSave(event: Event) {
    event.preventDefault();
    isSaving = true;
    try {
      await saveAiSettings(settings);
      showToast(t('aiSettings.saved'), 'success');
      onSaved?.(settings);
    } catch (err) {
      showToast(errorText(err), 'error');
    } finally {
      isSaving = false;
    }
  }

  function handleReset() {
    settings = { ...DEFAULT_SETTINGS };
    testResult = null;
  }

  /** Saves first: the backend reads the stored settings, not whatever is typed in the form. */
  async function handleTest() {
    isTesting = true;
    testResult = null;
    try {
      await saveAiSettings(settings);
      const reply = await aiChat([{ role: 'user', content: 'ping' }]);
      const preview = reply.reply.trim().slice(0, 120) || t('aiSettings.emptyResponse');
      testResult = { ok: true, text: t('aiSettings.connectedReply', { preview }) };
    } catch (err) {
      testResult = { ok: false, text: errorText(err) };
    } finally {
      isTesting = false;
    }
  }
</script>

<form onsubmit={handleSave} class="space-y-4">
  <div>
    <label for="ai-provider-select" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
      {t('aiSettings.provider')}
    </label>
    <select
      id="ai-provider-select"
      bind:value={settings.provider}
      disabled={isLoading}
      class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-violet-500 transition-colors"
    >
      <option value="custom">{t('aiSettings.providerCustom')}</option>
      <option value="openai">{t('aiSettings.providerOpenai')}</option>
      <option value="ollama">{t('aiSettings.providerOllama')}</option>
      <option value="anthropic">{t('aiSettings.providerAnthropic')}</option>
    </select>
  </div>

  <div>
    <label for="ai-base-url-input" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
      {t('aiSettings.baseUrl')}
    </label>
    <input
      id="ai-base-url-input"
      type="text"
      bind:value={settings.base_url}
      disabled={isLoading}
      placeholder="http://100.76.150.46:3007/v1"
      class="w-full font-mono bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-xs md:text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-violet-500 transition-colors"
    />
    <p class="text-[11px] text-neutral-500 mt-1">
      {t('aiSettings.baseUrlHintBefore')} <span class="font-mono">/chat/completions</span>.
    </p>
  </div>

  <div>
    <label for="ai-api-key-input" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
      {t('aiSettings.apiKey')}
    </label>
    <div class="relative">
      <input
        id="ai-api-key-input"
        type={showApiKey ? 'text' : 'password'}
        bind:value={settings.api_key}
        disabled={isLoading}
        placeholder="sk-..."
        class="w-full font-mono bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-xs md:text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-violet-500 transition-colors pr-10"
      />
      <button
        type="button"
        onclick={() => (showApiKey = !showApiKey)}
        class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200"
        aria-label={showApiKey ? t('aiSettings.hideApiKey') : t('aiSettings.showApiKey')}
      >
        {#if showApiKey}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
          </svg>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
          </svg>
        {/if}
      </button>
    </div>
    <p class="text-[11px] text-neutral-500 mt-1">
      {t('aiSettings.apiKeyHint')}
    </p>
  </div>

  <div>
    <label for="ai-model-input" class="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
      {t('aiSettings.model')}
    </label>
    <input
      id="ai-model-input"
      type="text"
      bind:value={settings.model}
      disabled={isLoading}
      placeholder="claude-3-5-sonnet"
      class="w-full font-mono bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-xs md:text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-violet-500 transition-colors"
    />
  </div>

  {#if testResult}
    <div
      class="p-3 rounded-lg text-xs border {testResult.ok
        ? 'bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-900 text-emerald-700 dark:text-emerald-300'
        : 'bg-rose-50 dark:bg-rose-950/40 border-rose-200 dark:border-rose-900 text-rose-700 dark:text-rose-300'}"
    >
      <p class="font-semibold mb-0.5">{testResult.ok ? t('aiSettings.connectionOk') : t('aiSettings.connectionFailed')}</p>
      <p class="break-words">{testResult.text}</p>
    </div>
  {/if}

  <div class="flex items-center justify-between pt-3 border-t border-neutral-200 dark:border-neutral-800 gap-2">
    <button
      type="button"
      onclick={handleReset}
      class="text-xs text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 transition-colors shrink-0"
    >
      {t('aiSettings.resetDefault')}
    </button>
    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={handleTest}
        disabled={isTesting || isLoading}
        class="px-4 py-2 rounded-lg text-xs font-semibold border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors"
      >
        {isTesting ? t('aiSettings.testing') : t('aiSettings.testConnection')}
      </button>
      <button
        type="submit"
        disabled={isSaving || isLoading}
        class="px-5 py-2 rounded-lg text-xs font-semibold bg-violet-600 hover:bg-violet-500 disabled:opacity-50 text-white shadow transition-colors"
      >
        {isSaving ? t('aiSettings.saving') : t('aiSettings.saveSettings')}
      </button>
    </div>
  </div>
</form>
