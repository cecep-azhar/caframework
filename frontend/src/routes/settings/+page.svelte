<script lang="ts">
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
  import { getTheme, setTheme } from '$lib/stores/theme.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { changeMasterPassword } from '$lib/api/vault';
  import { getAiSettings, saveAiSettings, type AiSettings } from '$lib/api/ai';
  import { listProfiles, saveProfile, type ProfileRecord } from '$lib/api/profiles';
  import { exportEncryptedBackup, importEncryptedBackup } from '$lib/api/prefs';
  import { APP_VERSION } from '$lib/appInfo';

  type SettingsTab = 'security' | 'profiles' | 'ai' | 'backup' | 'about';
  let currentTab = $state<SettingsTab>('security');

  // Password state
  let oldPassword = $state('');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let isChangingPassword = $state(false);

  // Backup state
  let backupPassword = $state('');
  let isExporting = $state(false);
  let isImporting = $state(false);

  // AI state
  let aiSettings = $state<any>({
    enabled: true,
    provider: 'openai',
    endpoint: 'https://api.openai.com/v1',
    api_key: '',
    model: 'gpt-4o-mini'
  });
  let isSavingAi = $state(false);

  // Profiles state
  let profilesList = $state<ProfileRecord[]>([]);
  let isAddProfileOpen = $state(false);
  let newProfileName = $state('');
  let newProfileRole = $state<'owner' | 'partner' | 'member' | 'child'>('member');
  let newProfilePin = $state('');

  onMount(async () => {
    try {
      const [ai, profs] = await Promise.all([
        getAiSettings().catch(() => aiSettings),
        listProfiles().catch(() => [])
      ]);
      aiSettings = ai;
      profilesList = profs;
    } catch {}
  });

  async function handleChangePassword() {
    if (newPassword.length < 8) {
      showToast('Password must be at least 8 characters', 'error');
      return;
    }
    if (newPassword !== confirmPassword) {
      showToast('Passwords do not match', 'error');
      return;
    }

    isChangingPassword = true;
    try {
      await changeMasterPassword(oldPassword, newPassword);
      showToast('Master password changed successfully', 'success');
      oldPassword = '';
      newPassword = '';
      confirmPassword = '';
    } catch (e: any) {
      showToast(e?.message || 'Failed to change master password', 'error');
    } finally {
      isChangingPassword = false;
    }
  }

  async function handleExportBackup() {
    if (!backupPassword) {
      showToast('Backup password is required', 'error');
      return;
    }
    isExporting = true;
    try {
      const b64Data = await exportEncryptedBackup(backupPassword);
      // Construct a typed array holding the byte data
      const uint8Array = new Uint8Array(b64Data);
      const blob = new Blob([uint8Array], { type: 'application/octet-stream' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `caframework-backup-${new Date().toISOString().slice(0, 10)}.cafbackup`;
      a.click();
      URL.revokeObjectURL(url);
      showToast('Backup exported successfully', 'success');
    } catch (e: any) {
      showToast(e?.message || 'Failed to export backup', 'error');
    } finally {
      isExporting = false;
    }
  }

  async function handleSaveAi() {
    isSavingAi = true;
    try {
      await saveAiSettings(aiSettings);
      showToast('AI settings saved', 'success');
    } catch (e: any) {
      showToast(e?.message || 'Failed to save AI settings', 'error');
    } finally {
      isSavingAi = false;
    }
  }

  async function handleAddProfile() {
    if (!newProfileName.trim()) {
      showToast('Profile name is required', 'error');
      return;
    }
    try {
      await saveProfile(
        {
          name: newProfileName.trim(),
          role: newProfileRole,
          pin: newProfilePin.trim() || undefined,
          avatar: ''
        },
        '00000000-0000-0000-0000-000000000000'
      );
      showToast('Profile added successfully', 'success');
      newProfileName = '';
      newProfilePin = '';
      isAddProfileOpen = false;
      profilesList = await listProfiles();
    } catch (e: any) {
      showToast(e?.message || 'Failed to add profile', 'error');
    }
  }
</script>

<div class="h-full flex flex-col bg-neutral-950 overflow-hidden">
  <PageHeader title="Settings" subtitle="Manage security, family profiles, AI privacy, and system backups" />

  <div class="flex-1 flex overflow-hidden">
    <!-- Settings Sidebar -->
    <div class="w-56 border-r border-neutral-800 p-4 space-y-1">
      <button
        onclick={() => (currentTab = 'security')}
        class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors {currentTab === 'security' ? 'bg-indigo-600 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-900'}"
      >
        Security & Vault
      </button>
      <button
        onclick={() => (currentTab = 'profiles')}
        class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors {currentTab === 'profiles' ? 'bg-indigo-600 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-900'}"
      >
        Family Profiles
      </button>
      <button
        onclick={() => (currentTab = 'ai')}
        class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors {currentTab === 'ai' ? 'bg-indigo-600 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-900'}"
      >
        AI Assistant & Privacy
      </button>
      <button
        onclick={() => (currentTab = 'backup')}
        class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors {currentTab === 'backup' ? 'bg-indigo-600 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-900'}"
      >
        Backup & Restore
      </button>
      <button
        onclick={() => (currentTab = 'about')}
        class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors {currentTab === 'about' ? 'bg-indigo-600 text-white' : 'text-neutral-400 hover:text-white hover:bg-neutral-900'}"
      >
        About CAFramework
      </button>
    </div>

    <!-- Settings Tab Content -->
    <div class="flex-1 overflow-y-auto p-6 max-w-2xl">
      {#if currentTab === 'security'}
        <div class="space-y-6">
          <div>
            <h2 class="text-base font-semibold text-white">Master Password</h2>
            <p class="text-xs text-neutral-400 mt-1">Changes the Argon2id zero-knowledge key protecting your SQLCipher database.</p>
          </div>

          <div class="space-y-4 bg-neutral-900/50 border border-neutral-800 p-5 rounded-2xl">
            <div>
              <label for="old-pass" class="block text-xs font-medium text-neutral-400 mb-1.5">Current Password</label>
              <input
                id="old-pass"
                type="password"
                bind:value={oldPassword}
                placeholder="••••••••••••"
                class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label for="new-pass" class="block text-xs font-medium text-neutral-400 mb-1.5">New Password</label>
                <input
                  id="new-pass"
                  type="password"
                  bind:value={newPassword}
                  placeholder="••••••••••••"
                  class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label for="confirm-pass" class="block text-xs font-medium text-neutral-400 mb-1.5">Confirm New Password</label>
                <input
                  id="confirm-pass"
                  type="password"
                  bind:value={confirmPassword}
                  placeholder="••••••••••••"
                  class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            <div class="flex justify-end pt-2">
              <button
                onclick={handleChangePassword}
                disabled={isChangingPassword || !newPassword}
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-medium rounded-xl transition-colors"
              >
                {isChangingPassword ? 'Updating...' : 'Update Master Password'}
              </button>
            </div>
          </div>
        </div>
      {:else if currentTab === 'profiles'}
        <div class="space-y-6">
          <div class="flex items-center justify-between">
            <div>
              <h2 class="text-base font-semibold text-white">Family Profiles</h2>
              <p class="text-xs text-neutral-400 mt-1">Multi-profile support with individual PIN lock & roles.</p>
            </div>
            <button
              onclick={() => (isAddProfileOpen = true)}
              class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-xl transition-colors"
            >
              + Add Member
            </button>
          </div>

          {#if isAddProfileOpen}
            <div class="p-4 bg-neutral-900 border border-neutral-800 rounded-2xl space-y-4">
              <h3 class="text-xs font-semibold uppercase tracking-wider text-neutral-400">New Family Profile</h3>
              <div class="grid grid-cols-2 gap-3">
                <input
                  type="text"
                  bind:value={newProfileName}
                  placeholder="Profile Name (e.g. Fatih)"
                  class="px-3.5 py-2 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500"
                />
                <select
                  bind:value={newProfileRole}
                  class="px-3.5 py-2 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="owner">Owner (Ayah)</option>
                  <option value="partner">Partner (Ibu)</option>
                  <option value="member">Member</option>
                  <option value="child">Child (Anak)</option>
                </select>
              </div>
              <div>
                <input
                  type="password"
                  bind:value={newProfilePin}
                  maxlength="6"
                  placeholder="Optional 4-6 Digit PIN"
                  class="w-full px-3.5 py-2 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>
              <div class="flex justify-end gap-2">
                <button
                  onclick={() => (isAddProfileOpen = false)}
                  class="px-3 py-1.5 bg-neutral-800 hover:bg-neutral-700 text-neutral-300 text-xs font-medium rounded-xl"
                >
                  Cancel
                </button>
                <button
                  onclick={handleAddProfile}
                  class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-xl"
                >
                  Save Profile
                </button>
              </div>
            </div>
          {/if}

          <div class="space-y-2">
            {#each profilesList as prof}
              <div class="flex items-center justify-between p-3.5 bg-neutral-900/40 border border-neutral-800 rounded-xl">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-full bg-neutral-800 border border-neutral-700 flex items-center justify-center font-bold text-xs text-white">
                    {prof.name.slice(0, 2).toUpperCase()}
                  </div>
                  <div>
                    <h3 class="text-xs font-medium text-white">{prof.name}</h3>
                    <span class="text-[10px] text-neutral-400 capitalize">{prof.role}</span>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <span class="text-[10px] px-2 py-0.5 rounded bg-neutral-800 text-neutral-400 font-mono">
                    {prof.has_pin ? 'PIN Set' : 'No PIN'}
                  </span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else if currentTab === 'ai'}
        <div class="space-y-6">
          <div>
            <h2 class="text-base font-semibold text-white">AI Assistant Configuration</h2>
            <p class="text-xs text-neutral-400 mt-1">Configure BYO (Bring Your Own) LLM endpoints or local Ollama instances.</p>
          </div>

          <div class="space-y-4 bg-neutral-900/50 border border-neutral-800 p-5 rounded-2xl">
            <div>
              <label for="ai-provider" class="block text-xs font-medium text-neutral-400 mb-1.5">Provider</label>
              <select
                id="ai-provider"
                bind:value={aiSettings.provider}
                class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="openai">OpenAI / OpenAI-Compatible (9Router, OpenRouter, PBS)</option>
                <option value="anthropic">Anthropic Claude</option>
                <option value="ollama">Ollama (Local / Private)</option>
                <option value="hosted">Hosted Fathforce Proxy</option>
              </select>
            </div>

            <div>
              <label for="ai-endpoint" class="block text-xs font-medium text-neutral-400 mb-1.5">Endpoint URL</label>
              <input
                id="ai-endpoint"
                type="text"
                bind:value={aiSettings.endpoint}
                placeholder="https://api.openai.com/v1"
                class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500 font-mono"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label for="ai-key" class="block text-xs font-medium text-neutral-400 mb-1.5">API Key</label>
                <input
                  id="ai-key"
                  type="password"
                  bind:value={aiSettings.api_key}
                  placeholder="sk-..."
                  class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>
              <div>
                <label for="ai-model" class="block text-xs font-medium text-neutral-400 mb-1.5">Model</label>
                <input
                  id="ai-model"
                  type="text"
                  bind:value={aiSettings.model}
                  placeholder="gpt-4o-mini"
                  class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>
            </div>

            <div class="pt-2 space-y-3">
              <label class="flex items-center gap-2.5 cursor-pointer">
                <input
                  type="checkbox"
                  checked={true}
                  class="w-4 h-4 rounded text-indigo-600 focus:ring-indigo-500"
                />
                <div>
                  <span class="text-xs font-medium text-white block">Privacy-First Mode (Anonymised Summary)</span>
                  <span class="text-[10px] text-neutral-400 block">Redacts personal identifiable info and masks account details before sending prompts to the AI provider.</span>
                </div>
              </label>
            </div>

            <div class="flex justify-end pt-2">
              <button
                onclick={handleSaveAi}
                disabled={isSavingAi}
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-medium rounded-xl transition-colors"
              >
                {isSavingAi ? 'Saving...' : 'Save AI Configuration'}
              </button>
            </div>
          </div>
        </div>
      {:else if currentTab === 'backup'}
        <div class="space-y-6">
          <div>
            <h2 class="text-base font-semibold text-white">Encrypted Backup</h2>
            <p class="text-xs text-neutral-400 mt-1">Export or restore your full encrypted vault payload safely.</p>
          </div>

          <div class="bg-neutral-900/50 border border-neutral-800 p-5 rounded-2xl space-y-4">
            <div>
              <label for="backup-pass" class="block text-xs font-medium text-neutral-400 mb-1.5">Backup Password</label>
              <input
                id="backup-pass"
                type="password"
                bind:value={backupPassword}
                placeholder="••••••••••••"
                class="w-full px-3.5 py-2.5 bg-neutral-950 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-600 focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div class="flex gap-3 pt-2">
              <button
                onclick={handleExportBackup}
                disabled={isExporting || !backupPassword}
                class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-medium rounded-xl transition-colors"
              >
                {isExporting ? 'Exporting...' : 'Export Encrypted Backup'}
              </button>
            </div>
          </div>
        </div>
      {:else if currentTab === 'about'}
        <div class="space-y-6">
          <div>
            <h2 class="text-base font-semibold text-white">About CAFramework</h2>
            <p class="text-xs text-neutral-400 mt-1">Open-source starter framework for privacy-first desktop & mobile apps.</p>
          </div>

          <div class="bg-neutral-900/40 border border-neutral-800 p-5 rounded-2xl space-y-3 text-xs text-neutral-300">
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <span class="text-neutral-500">Framework Version</span>
              <span class="font-mono text-white">v{APP_VERSION}</span>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <span class="text-neutral-500">Architecture Stack</span>
              <span>Tauri 2 + Rust Edition 2024 + Svelte 5 + SQLCipher</span>
            </div>
            <div class="flex justify-between py-1 border-b border-neutral-800">
              <span class="text-neutral-500">License</span>
              <span>MIT (with CATerm Attribution Notice)</span>
            </div>
            <div class="flex justify-between py-1">
              <span class="text-neutral-500">Author</span>
              <span>Cecep Azhar (Fathforce)</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
