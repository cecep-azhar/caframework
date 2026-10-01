<script lang="ts">
  import { onMount } from 'svelte';
  import { listKeys, generateKey, importKey, deleteKey, deployPublicKey, type KeyRecord } from '$lib/api/keys';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { confirmModal } from '$lib/stores/uiNotifications.svelte';

  let keys: KeyRecord[] = [];
  let hosts: HostRecord[] = [];
  let isLoading = true;
  let errorMsg = '';
  let successMsg = '';

  // Deploy modal
  let showDeployModal = false;
  let deployKeyId = '';
  let deployHostId = '';
  let isDeploying = false;

  // Generate modal
  let showGenerateModal = false;
  let generateName = '';
  let generateAlg = 'Ed25519';
  let isGenerating = false;

  // Import modal
  let showImportModal = false;
  let importName = '';
  let importPem = '';
  let importPassphrase = '';
  let showImportPassphrase = false;
  let isImporting = false;

  async function loadKeys() {
    isLoading = true;
    errorMsg = '';
    successMsg = '';
    try {
      [keys, hosts] = await Promise.all([listKeys(), listHosts()]);
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isLoading = false;
    }
  }

  onMount(loadKeys);

  async function handleGenerate() {
    if (!generateName.trim()) return;
    isGenerating = true;
    errorMsg = '';
    successMsg = '';
    try {
      await generateKey({ name: generateName, algorithm: generateAlg });
      showGenerateModal = false;
      generateName = '';
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isGenerating = false;
    }
  }

  async function handleImport() {
    if (!importName.trim() || !importPem.trim()) return;
    isImporting = true;
    errorMsg = '';
    successMsg = '';
    try {
      await importKey(importName, importPem, importPassphrase || undefined);
      showImportModal = false;
      importName = '';
      importPem = '';
      importPassphrase = '';
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isImporting = false;
    }
  }

  async function handleDelete(id: string, name: string) {
    const confirmed = await confirmModal(t('sshKeys.deleteConfirm', { name }), t('common.delete'), true, t('common.delete'), t('common.cancel'));
    if (!confirmed) return;
    errorMsg = '';
    successMsg = '';
    try {
      await deleteKey(id);
      await loadKeys();
    } catch (e: any) {
      errorMsg = String(e);
    }
  }

  function openDeployModal(keyId: string) {
    deployKeyId = keyId;
    deployHostId = hosts.length > 0 ? hosts[0].id : '';
    showDeployModal = true;
    errorMsg = '';
    successMsg = '';
  }

  async function handleDeploy() {
    if (!deployKeyId || !deployHostId) return;
    isDeploying = true;
    errorMsg = '';
    successMsg = '';
    try {
      await deployPublicKey(deployHostId, deployKeyId);
      showDeployModal = false;
      const targetHost = hosts.find(h => h.id === deployHostId);
      successMsg = t('sshKeys.deployedSuccess', { host: targetHost?.label || deployHostId });
    } catch (e: any) {
      errorMsg = String(e);
    } finally {
      isDeploying = false;
    }
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z']}
    accent="rose"
    title={t('sshKeys.title')}
    subtitle={t('sshKeys.subtitle')}
  >
    {#snippet actions()}
      <button onclick={() => showImportModal = true} class="px-3.5 py-2 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-200 rounded-lg text-sm font-medium transition-colors border border-neutral-200 dark:border-neutral-700">
        {t('sshKeys.import')}
      </button>
      <button onclick={() => showGenerateModal = true} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow shadow-sky-600/20">
        {t('sshKeys.generate')}
      </button>
    {/snippet}
  </PageHeader>

  {#if errorMsg}
    <div class="p-4 bg-red-500/10 border border-red-500/20 rounded-lg text-red-600 dark:text-red-400 text-sm">
      {errorMsg}
    </div>
  {/if}

  {#if successMsg}
    <div class="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-lg text-emerald-600 dark:text-emerald-400 text-sm">
      {successMsg}
    </div>
  {/if}

  {#if isLoading}
    <div class="p-8 text-center text-neutral-500 text-sm">{t('sshKeys.loading')}</div>
  {:else if keys.length === 0}
    <div class="p-8 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-white dark:bg-neutral-900/30 text-center flex flex-col items-center justify-center shadow-sm dark:shadow-none">
      <svg class="w-12 h-12 text-neutral-400 dark:text-neutral-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
      </svg>
      <h3 class="text-lg font-medium text-neutral-900 dark:text-white mb-2">{t('sshKeys.emptyTitle')}</h3>
      <p class="text-sm text-neutral-500 dark:text-neutral-400 max-w-md">{t('sshKeys.emptyBody')}</p>
      <div class="mt-6 flex gap-3">
        <button onclick={() => showImportModal = true} class="px-4 py-2 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-900 dark:text-white rounded-lg text-sm font-medium transition-colors border border-neutral-300 dark:border-neutral-700">
          {t('sshKeys.importKey')}
        </button>
        <button onclick={() => showGenerateModal = true} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-sky-600/20">
          {t('sshKeys.generateKeypair')}
        </button>
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each keys as key (key.id)}
        <div class="flex flex-col p-4 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl hover:border-neutral-300 dark:hover:border-neutral-700 transition-colors group shadow-sm dark:shadow-none">
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-medium text-neutral-900 dark:text-white flex items-center gap-2">
              <svg class="w-4 h-4 text-rose-500 dark:text-rose-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"></path></svg>
              {key.name}
            </h3>
            <span class="text-xs px-2 py-0.5 rounded-full bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 font-mono">
              {key.algorithm}
            </span>
          </div>

          <div class="space-y-2 mb-4">
            <div class="bg-neutral-50 dark:bg-black/30 rounded p-2 text-xs font-mono text-neutral-500 break-all select-all">
              {key.fingerprint}
            </div>
          </div>

          <div class="mt-auto pt-3 border-t border-neutral-100 dark:border-neutral-800/50 flex justify-between items-center opacity-70 group-hover:opacity-100 transition-opacity">
            <span class="text-xs text-neutral-500" title={key.createdAt}>
              {new Date(key.createdAt).toLocaleDateString(intlLocale())}
            </span>
            <div class="flex items-center gap-2">
              <button
                onclick={() => openDeployModal(key.id)}
                class="text-xs text-sky-600 dark:text-sky-400 hover:text-sky-500 dark:hover:text-sky-300 font-medium px-2 py-1 rounded hover:bg-sky-500/10 transition-colors"
              >
                {t('sshKeys.deployToServer')}
              </button>
              <button
                onclick={() => handleDelete(key.id, key.name)}
                class="text-xs text-rose-600 dark:text-rose-500 hover:text-rose-500 dark:hover:text-rose-400 font-medium px-2 py-1 rounded hover:bg-rose-500/10 transition-colors"
              >
                {t('common.delete')}
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showDeployModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 dark:bg-black/60 backdrop-blur-sm">
  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl w-full max-w-md shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-neutral-900 dark:text-white mb-4">{t('sshKeys.deployTitle')}</h3>

    <div class="space-y-4">
      <div>
        <label for="deploy-host" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.targetHost')}</label>
        {#if hosts.length === 0}
          <div class="p-3 bg-neutral-50 dark:bg-neutral-950 border border-neutral-200 dark:border-neutral-800 rounded-lg text-sm text-neutral-500">
            {t('sshKeys.noHostsAddOne')}
          </div>
        {:else}
          <select id="deploy-host" bind:value={deployHostId} class="w-full bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors">
            {#each hosts as h}
              <option value={h.id}>{h.label} ({h.username}@{h.address})</option>
            {/each}
          </select>
        {/if}
      </div>
      <p class="text-xs text-neutral-500">
        {t('sshKeys.deployNote')}
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button onclick={() => showDeployModal = false} class="px-4 py-2 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors text-sm font-medium">{t('common.cancel')}</button>
      <button onclick={handleDeploy} disabled={!deployHostId || isDeploying} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isDeploying ? t('sshKeys.deploying') : t('sshKeys.deployAction')}
      </button>
    </div>
  </div>
</div>
{/if}

{#if showGenerateModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 dark:bg-black/60 backdrop-blur-sm">
  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl w-full max-w-md shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-neutral-900 dark:text-white mb-4">{t('sshKeys.generateTitle')}</h3>

    <div class="space-y-4">
      <div>
        <label for="gen-name" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.keyName')}</label>
        <input id="gen-name" type="text" bind:value={generateName} placeholder={t('sshKeys.keyNamePlaceholder')} class="w-full bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors" />
      </div>
      <div>
        <label for="gen-alg" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.algorithm')}</label>
        <select id="gen-alg" bind:value={generateAlg} class="w-full bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors">
          <option value="Ed25519">{t('sshKeys.algoEd25519')}</option>
          <option value="RSA-4096">{t('sshKeys.algoRsa')}</option>
        </select>
      </div>
      <p class="text-xs text-neutral-500">
        {t('sshKeys.generateNote')}
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button onclick={() => showGenerateModal = false} class="px-4 py-2 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors text-sm font-medium">{t('common.cancel')}</button>
      <button onclick={handleGenerate} disabled={!generateName.trim() || isGenerating} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isGenerating ? t('sshKeys.generating') : t('sshKeys.generateAction')}
      </button>
    </div>
  </div>
</div>
{/if}

{#if showImportModal}
<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 dark:bg-black/60 backdrop-blur-sm">
  <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl w-full max-w-2xl shadow-2xl p-6">
    <h3 class="text-lg font-semibold text-neutral-900 dark:text-white mb-4">{t('sshKeys.importTitle')}</h3>

    <div class="space-y-4">
      <div>
        <label for="imp-name" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.keyName')}</label>
        <input id="imp-name" type="text" bind:value={importName} placeholder="e.g. AWS Legacy Key" class="w-full bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors" />
      </div>
      <div>
        <label for="imp-pem" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.privateKeyPem')}</label>
        <textarea id="imp-pem" bind:value={importPem} rows="6" placeholder={t('sshKeys.privateKeyPlaceholder')} class="w-full font-mono bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg px-3 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors resize-none"></textarea>
      </div>
      <div>
        <label for="imp-pass" class="block text-xs font-medium text-neutral-500 dark:text-neutral-400 mb-1">{t('sshKeys.passphraseIfEncrypted')}</label>
        <div class="relative">
          <input
            id="imp-pass"
            type={showImportPassphrase ? 'text' : 'password'}
            bind:value={importPassphrase}
            placeholder={t('sshKeys.passphrasePlaceholder')}
            class="w-full bg-neutral-50 dark:bg-black/40 border border-neutral-300 dark:border-neutral-800 rounded-lg pl-3 pr-10 py-2 text-sm text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition-colors"
          />
          <button
            type="button"
            onclick={() => (showImportPassphrase = !showImportPassphrase)}
            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors p-1"
            aria-label={showImportPassphrase ? t('sshKeys.hidePassphrase') : t('sshKeys.showPassphrase')}
            title={showImportPassphrase ? t('sshKeys.hidePassphrase') : t('sshKeys.showPassphrase')}
          >
            {#if showImportPassphrase}
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
      </div>
      <p class="text-xs text-neutral-500">
        {t('sshKeys.importNote')}
      </p>
    </div>

    <div class="flex justify-end gap-3 mt-6">
      <button onclick={() => showImportModal = false} class="px-4 py-2 text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors text-sm font-medium">{t('common.cancel')}</button>
      <button onclick={handleImport} disabled={!importName.trim() || !importPem.trim() || isImporting} class="px-4 py-2 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white rounded-lg text-sm font-medium transition-colors">
        {isImporting ? t('sshKeys.importing') : t('sshKeys.importAction')}
      </button>
    </div>
  </div>
</div>
{/if}
