<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import AiNavTabs from '$lib/components/AiNavTabs.svelte';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import {
    type AiProvider,
    type AiProviderConfig,
    type TaskRouteRule,
    type TaskType,
    type ProviderType,
    type SystemPersona,
    aiGetProviders,
    aiSaveProvider,
    aiDeleteProvider,
    aiGetRoutingMatrix,
    aiSaveRoutingRule,
    aiGetPersonas
  } from '$lib/api/aiRouting';

  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 shadow-sm dark:shadow-none text-neutral-900 dark:text-white';
  const SUBCARD = 'border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950';
  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const INPUT = 'w-full px-3 py-2 text-sm rounded-md border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-sky-500';
  const LABEL = 'block text-xs font-semibold uppercase tracking-wider text-neutral-700 dark:text-neutral-300 mb-1';

  let providers = $state<AiProvider[]>([]);
  let routingRules = $state<TaskRouteRule[]>([]);
  let personas = $state<SystemPersona[]>([]);
  let loading = $state(true);

  // Modal / Form state for Add/Edit Provider
  let showProviderModal = $state(false);
  let editingProviderId = $state<string | null>(null);
  let formName = $state('');
  let formType = $state<ProviderType>('openai_compatible');
  let formBaseUrl = $state('');
  let formApiKey = $state('');
  let formModel = $state('');
  let formActive = $state(true);
  let savingProvider = $state(false);

  // Task descriptions
  const TASK_INFO: Record<TaskType, { title: string; desc: string }> = {
    chat: {
      title: 'Interactive AI Chat',
      desc: 'Real-time multi-turn conversational terminal assistant & execution planner.'
    },
    error_diagnostic: {
      title: 'Terminal Error Auto-Diagnostic',
      desc: 'Automatic diagnosis of non-zero exit codes & bash crash traces.'
    },
    prompt_studio: {
      title: 'Prompt Studio & Templates',
      desc: 'Automated script generation, cron tasks, and playbook templates.'
    },
    command_autocomplete: {
      title: 'Command Autocomplete & Suggestion',
      desc: 'Fast, lightweight sub-command prediction and flag completion.'
    },
    security_review: {
      title: 'Command Security Review',
      desc: 'Pre-execution risk evaluation for destructive CLI commands.'
    }
  };

  async function loadData() {
    loading = true;
    try {
      const [provs, matrix, pers] = await Promise.all([
        aiGetProviders(),
        aiGetRoutingMatrix(),
        aiGetPersonas()
      ]);
      providers = provs;
      routingRules = matrix;
      personas = pers;
    } catch (err) {
      showToast('Failed to load AI Routing data: ' + String(err), 'error');
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  function openAddProvider() {
    editingProviderId = null;
    formName = '';
    formType = 'openai_compatible';
    formBaseUrl = 'https://api.openai.com/v1';
    formApiKey = '';
    formModel = 'gpt-4o';
    formActive = true;
    showProviderModal = true;
  }

  function openEditProvider(p: AiProvider) {
    editingProviderId = p.id;
    formName = p.name;
    formType = p.provider_type;
    formBaseUrl = p.base_url;
    formApiKey = '';
    formModel = p.default_model;
    formActive = p.is_active;
    showProviderModal = true;
  }

  function handleTypeChange() {
    if (formType === 'caterm_hosted') {
      formBaseUrl = 'https://ai.caterm.com/v1';
      formModel = 'caterm-omni-fast';
    } else if (formType === 'anthropic') {
      formBaseUrl = 'https://api.anthropic.com/v1';
      formModel = 'claude-3-7-sonnet';
    } else if (formType === 'openai_compatible') {
      formBaseUrl = 'https://api.openai.com/v1';
      formModel = 'gpt-4o';
    } else if (formType === 'ollama') {
      formBaseUrl = 'http://localhost:11434';
      formModel = 'llama3.2';
    }
  }

  async function handleSaveProvider() {
    if (!formName.trim() || !formBaseUrl.trim() || !formModel.trim()) {
      showToast('Name, Base URL, and Default Model are required.', 'error');
      return;
    }
    savingProvider = true;
    try {
      const config: AiProviderConfig = {
        id: editingProviderId ?? `prov-${Date.now()}`,
        name: formName.trim(),
        provider_type: formType,
        base_url: formBaseUrl.trim(),
        api_key: formApiKey.trim() ? formApiKey.trim() : null,
        default_model: formModel.trim(),
        is_active: formActive
      };
      await aiSaveProvider(config);
      showToast('Provider saved successfully.', 'success');
      showProviderModal = false;
      await loadData();
    } catch (err) {
      showToast('Failed to save provider: ' + String(err), 'error');
    } finally {
      savingProvider = false;
    }
  }

  async function handleDeleteProvider(p: AiProvider) {
    if (p.id === 'caterm-hosted') {
      showToast('Built-in CATerm Hosted provider cannot be deleted.', 'error');
      return;
    }
    const confirmed = await confirmModal(
      `Delete AI Provider "${p.name}"? This action cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await aiDeleteProvider(p.id);
      showToast(`Provider "${p.name}" deleted.`, 'success');
      await loadData();
    } catch (err) {
      showToast('Failed to delete provider: ' + String(err), 'error');
    }
  }

  async function handleSaveRoutingRule(rule: TaskRouteRule) {
    try {
      await aiSaveRoutingRule(rule);
      showToast(`Routing rule for "${rule.task_type}" updated.`, 'success');
    } catch (err) {
      showToast('Failed to save routing rule: ' + String(err), 'error');
    }
  }
</script>

<div class="max-w-6xl mx-auto space-y-6">
  <PageHeader
    icon="M13 10V3L4 14h7v7l9-11h-7z"
    accent="sky"
    title="AI Routing & Fallback Matrix"
    badge="Multi-Provider Engine"
    subtitle="Zero-knowledge encrypted model routing with automatic failover and local privacy boundaries."
  />

  <AiNavTabs active="routing" />

  {#if loading}
    <div class="{CARD} flex items-center justify-center py-12 text-sm {MUTED}">
      <svg class="w-5 h-5 animate-spin mr-3 text-sky-500" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
      </svg>
      Loading AI Provider & Routing matrix...
    </div>
  {:else}
    <!-- AI Providers Section -->
    <div class="{CARD} space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 class="text-base font-semibold text-neutral-900 dark:text-white">Registered AI Providers</h2>
          <p class="{MUTED} text-xs mt-0.5">
            BYO API keys (Anthropic, OpenAI, DeepSeek, Ollama) and CATerm Pro Hosted AI. All keys are encrypted at rest with SQLCipher Argon2id.
          </p>
        </div>
        <button
          onclick={openAddProvider}
          class="px-3.5 py-1.5 text-xs font-semibold rounded-md bg-sky-600 hover:bg-sky-500 text-white shadow-sm transition-colors flex items-center gap-1.5 self-start sm:self-auto"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Add Provider
        </button>
      </div>

      <div class="overflow-x-auto border border-neutral-200 dark:border-neutral-800 rounded-lg">
        <table class="w-full text-left text-xs">
          <thead class="bg-neutral-100 dark:bg-neutral-800/60 text-neutral-700 dark:text-neutral-300 uppercase tracking-wider font-semibold border-b border-neutral-200 dark:border-neutral-800">
            <tr>
              <th class="px-4 py-3">Name</th>
              <th class="px-4 py-3">Type</th>
              <th class="px-4 py-3">Base URL</th>
              <th class="px-4 py-3">Default Model</th>
              <th class="px-4 py-3">API Key Status</th>
              <th class="px-4 py-3">Status</th>
              <th class="px-4 py-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-neutral-200 dark:divide-neutral-800">
            {#each providers as p (p.id)}
              <tr class="hover:bg-neutral-50 dark:hover:bg-neutral-800/30 transition-colors">
                <td class="px-4 py-3 font-medium text-neutral-900 dark:text-white">
                  {p.name}
                  {#if p.id === 'caterm-hosted'}
                    <span class="ml-1.5 px-1.5 py-0.5 text-[10px] font-bold rounded bg-sky-500/10 text-sky-600 dark:text-sky-400 border border-sky-500/20">
                      DEFAULT
                    </span>
                  {/if}
                </td>
                <td class="px-4 py-3 font-mono text-neutral-600 dark:text-neutral-300">
                  <span class="px-2 py-0.5 rounded text-[11px] font-medium {p.provider_type === 'caterm_hosted' ? 'bg-purple-500/10 text-purple-600 dark:text-purple-400' : p.provider_type === 'anthropic' ? 'bg-amber-500/10 text-amber-600 dark:text-amber-400' : p.provider_type === 'ollama' ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' : 'bg-blue-500/10 text-blue-600 dark:text-blue-400'}">
                    {p.provider_type}
                  </span>
                </td>
                <td class="px-4 py-3 font-mono text-neutral-500 dark:text-neutral-400 truncate max-w-xs">{p.base_url}</td>
                <td class="px-4 py-3 font-mono text-neutral-800 dark:text-neutral-200">{p.default_model}</td>
                <td class="px-4 py-3">
                  {#if p.provider_type === 'caterm_hosted'}
                    <span class="text-neutral-500 dark:text-neutral-400">Pro Token Handshake</span>
                  {:else if p.provider_type === 'ollama'}
                    <span class="text-emerald-600 dark:text-emerald-400">No Key Needed (Local)</span>
                  {:else if p.has_api_key}
                    <span class="text-emerald-600 dark:text-emerald-400 font-medium flex items-center gap-1">
                      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
                      Encrypted in Vault
                    </span>
                  {:else}
                    <span class="text-amber-600 dark:text-amber-400 font-medium">Missing Key</span>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  <span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold {p.is_active ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' : 'bg-neutral-500/10 text-neutral-500'}">
                    <span class="w-1.5 h-1.5 rounded-full {p.is_active ? 'bg-emerald-500' : 'bg-neutral-400'}"></span>
                    {p.is_active ? 'ACTIVE' : 'INACTIVE'}
                  </span>
                </td>
                <td class="px-4 py-3 text-right space-x-2">
                  <button
                    onclick={() => openEditProvider(p)}
                    class="text-sky-600 dark:text-sky-400 hover:underline font-medium"
                  >
                    Edit
                  </button>
                  {#if p.id !== 'caterm-hosted'}
                    <button
                      onclick={() => handleDeleteProvider(p)}
                      class="text-rose-600 dark:text-rose-400 hover:underline font-medium"
                    >
                      Delete
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>

    <!-- Task Routing Matrix Section -->
    <div class="{CARD} space-y-5">
      <div>
        <h2 class="text-base font-semibold text-neutral-900 dark:text-white">Deterministic Task Routing Matrix</h2>
        <p class="{MUTED} text-xs mt-0.5">
          Assign specialized models for specific terminal operations. If primary provider returns 429, 503, or times out (>15s), the fallback model seamlessly executes.
        </p>
      </div>

      <div class="space-y-4">
        {#each routingRules as rule (rule.task_type)}
          <div class="{SUBCARD} space-y-3">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-neutral-200 dark:border-neutral-800 pb-2">
              <div>
                <h3 class="text-sm font-semibold text-neutral-900 dark:text-white flex items-center gap-2">
                  <span class="px-2 py-0.5 rounded text-xs font-mono font-medium bg-neutral-200 dark:bg-neutral-800 text-neutral-800 dark:text-neutral-200">
                    {rule.task_type}
                  </span>
                  {TASK_INFO[rule.task_type]?.title ?? rule.task_type}
                </h3>
                <p class="{MUTED} text-xs mt-0.5">{TASK_INFO[rule.task_type]?.desc ?? ''}</p>
              </div>
              <button
                onclick={() => handleSaveRoutingRule(rule)}
                class="px-3 py-1 text-xs font-semibold rounded bg-sky-600 hover:bg-sky-500 text-white transition-colors self-start sm:self-auto"
              >
                Save Rule
              </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 text-xs">
              <!-- Primary Provider & Model -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-sky-600 dark:text-sky-400 uppercase tracking-wide text-[10px]">Primary Execution</div>
                <div>
                  <label class={LABEL}>Provider</label>
                  <select bind:value={rule.primary_provider_id} class={INPUT}>
                    {#each providers as prov}
                      <option value={prov.id}>{prov.name} ({prov.provider_type})</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class={LABEL}>Model</label>
                  <input type="text" bind:value={rule.primary_model} class={INPUT} placeholder="e.g. claude-3-7-sonnet" />
                </div>
              </div>

              <!-- Fallback Provider & Model -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-amber-600 dark:text-amber-400 uppercase tracking-wide text-[10px]">Fallback Failover (15s Timeout)</div>
                <div>
                  <label class={LABEL}>Provider</label>
                  <select bind:value={rule.fallback_provider_id} class={INPUT}>
                    <option value={null}>None (Fail Fast)</option>
                    {#each providers as prov}
                      <option value={prov.id}>{prov.name} ({prov.provider_type})</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class={LABEL}>Model</label>
                  <input type="text" bind:value={rule.fallback_model} class={INPUT} placeholder="e.g. llama3.2 or gpt-4o" />
                </div>
              </div>

              <!-- Persona & System Prompt -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-purple-600 dark:text-purple-400 uppercase tracking-wide text-[10px]">System Persona Context</div>
                <div>
                  <label class={LABEL}>Active Persona</label>
                  <select bind:value={rule.system_persona_id} class={INPUT}>
                    <option value={null}>Global Default Persona</option>
                    {#each personas as pers}
                      <option value={pers.id}>{pers.title} {pers.is_global_default ? '(Default)' : ''}</option>
                    {/each}
                  </select>
                </div>
              </div>

              <!-- Hyperparameters -->
              <div class="space-y-2 p-3 rounded-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800">
                <div class="font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wide text-[10px]">Generation Parameters</div>
                <div class="flex items-center justify-between gap-2">
                  <label class={LABEL}>Temp ({rule.temperature.toFixed(2)})</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    bind:value={rule.temperature}
                    class="w-24 accent-sky-500"
                  />
                </div>
                <div>
                  <label class={LABEL}>Max Tokens</label>
                  <input
                    type="number"
                    min="100"
                    max="8192"
                    step="128"
                    bind:value={rule.max_tokens}
                    class={INPUT}
                  />
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- Modal: Add / Edit Provider -->
{#if showProviderModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 max-w-md w-full shadow-xl space-y-4 text-neutral-900 dark:text-white">
      <div class="flex items-center justify-between">
        <h3 class="text-base font-semibold">
          {editingProviderId ? 'Edit AI Provider' : 'Add New AI Provider'}
        </h3>
        <button
          onclick={() => (showProviderModal = false)}
          class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-200 text-lg font-bold"
        >
          &times;
        </button>
      </div>

      <div class="space-y-3 text-xs">
        <div>
          <label class={LABEL}>Provider Name</label>
          <input type="text" bind:value={formName} placeholder="e.g. Personal Anthropic" class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>Provider Protocol / Type</label>
          <select bind:value={formType} onchange={handleTypeChange} class={INPUT}>
            <option value="caterm_hosted">CATerm Hosted AI (OmniRoute Pro)</option>
            <option value="anthropic">Anthropic Messages API (/v1/messages)</option>
            <option value="openai_compatible">OpenAI Compatible (OpenAI, DeepSeek, Groq)</option>
            <option value="ollama">Local Ollama (/api/chat)</option>
          </select>
        </div>

        <div>
          <label class={LABEL}>Base URL</label>
          <input type="text" bind:value={formBaseUrl} class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>Default Model</label>
          <input type="text" bind:value={formModel} class={INPUT} />
        </div>

        {#if formType !== 'ollama' && formType !== 'caterm_hosted'}
          <div>
            <label class={LABEL}>
              API Key {editingProviderId ? '(Leave blank to keep current key)' : ''}
            </label>
            <input
              type="password"
              bind:value={formApiKey}
              placeholder="sk-..."
              class={INPUT}
            />
          </div>
        {/if}

        <div class="flex items-center gap-2 pt-2">
          <input
            type="checkbox"
            id="provider-active"
            bind:checked={formActive}
            class="rounded border-neutral-300 dark:border-neutral-700 text-sky-600 focus:ring-sky-500"
          />
          <label for="provider-active" class="text-xs text-neutral-700 dark:text-neutral-300 font-medium">
            Enable this provider in Routing Matrix
          </label>
        </div>
      </div>

      <div class="flex justify-end gap-3 pt-3 border-t border-neutral-200 dark:border-neutral-800">
        <button
          onclick={() => (showProviderModal = false)}
          class="px-4 py-2 text-xs font-medium rounded-md border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleSaveProvider}
          disabled={savingProvider}
          class="px-4 py-2 text-xs font-semibold rounded-md bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white transition-colors"
        >
          {savingProvider ? 'Saving...' : 'Save Provider'}
        </button>
      </div>
    </div>
  </div>
{/if}
