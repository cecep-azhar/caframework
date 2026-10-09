<script lang="ts">
  import { onMount } from 'svelte';
  import {
    aiGetSkills,
    aiSaveSkill,
    aiDeleteSkill,
    type CustomSkill
  } from '$lib/api/aiRouting';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import AiNavTabs from '$lib/components/AiNavTabs.svelte';

  const CARD = 'bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 shadow-sm text-neutral-900 dark:text-white';
  const LABEL = 'block text-xs font-semibold text-neutral-600 dark:text-neutral-400 uppercase tracking-wide mb-1.5';
  const INPUT = 'w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 transition';
  const TEXTAREA = 'w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg text-xs text-neutral-900 dark:text-white font-mono focus:outline-none focus:border-sky-500 transition';

  let skills = $state<CustomSkill[]>([]);
  let isLoading = $state(true);
  let showModal = $state(false);
  let isSaving = $state(false);

  // Form State
  let formId = $state<string | null>(null);
  let formName = $state('');
  let formTitle = $state('');
  let formDescription = $state('');
  let formCategory = $state('devops');
  let formTriggers = $state('');
  let formInstructions = $state('');
  let formTools = $state('terminal, file_reader');
  let formEnabled = $state(true);

  onMount(async () => {
    await loadSkills();
  });

  async function loadSkills() {
    isLoading = true;
    try {
      skills = await aiGetSkills();
    } catch (e) {
      showToast(`Gagal memuat skills: ${errorText(e)}`, 'error');
    } finally {
      isLoading = false;
    }
  }

  function openCreate() {
    formId = null;
    formName = '';
    formTitle = '';
    formDescription = '';
    formCategory = 'devops';
    formTriggers = '@my-skill, my task';
    formInstructions = 'Role: You are an expert engineer.\nInstructions:\n1. Inspect context\n2. Provide concise solution';
    formTools = 'terminal, file_reader';
    formEnabled = true;
    showModal = true;
  }

  function openEdit(skill: CustomSkill) {
    formId = skill.id;
    formName = skill.name;
    formTitle = skill.title;
    formDescription = skill.description;
    formCategory = skill.category;
    formTriggers = skill.triggers.join(', ');
    formInstructions = skill.system_instructions;
    formTools = skill.allowed_tools.join(', ');
    formEnabled = skill.is_enabled;
    showModal = true;
  }

  async function handleSave() {
    if (!formName.trim() || !formTitle.trim()) {
      showToast('Nama dan Title skill wajib diisi', 'error');
      return;
    }

    isSaving = true;
    try {
      const triggersArray = formTriggers
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean);
      const toolsArray = formTools
        .split(',')
        .map((t) => t.trim())
        .filter(Boolean);

      const skillToSave: CustomSkill = {
        id: formId || `skill-${Date.now()}`,
        name: formName.trim().toLowerCase().replace(/\s+/g, '-'),
        title: formTitle.trim(),
        description: formDescription.trim(),
        category: formCategory.trim(),
        triggers: triggersArray.length > 0 ? triggersArray : [`@${formName.trim()}`],
        system_instructions: formInstructions,
        allowed_tools: toolsArray,
        is_builtin: false,
        is_enabled: formEnabled,
        created_at: Math.floor(Date.now() / 1000)
      };

      await aiSaveSkill(skillToSave);
      showToast('Skill berhasil disimpan!', 'success');
      showModal = false;
      await loadSkills();
    } catch (e) {
      showToast(`Gagal menyimpan skill: ${errorText(e)}`, 'error');
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(skill: CustomSkill) {
    if (skill.is_builtin) {
      showToast('Skill bawaan sistem (builtin) tidak dapat dihapus', 'error');
      return;
    }

    const ok = await confirmModal(
      `Hapus skill "@${skill.name}"?`,
      'Hapus Custom Skill',
      true,
      'Hapus',
      'Batal'
    );
    if (!ok) return;

    try {
      await aiDeleteSkill(skill.id);
      showToast(`Skill @${skill.name} berhasil dihapus`, 'success');
      await loadSkills();
    } catch (e) {
      showToast(`Gagal menghapus skill: ${errorText(e)}`, 'error');
    }
  }
</script>

<div class="p-6 max-w-6xl mx-auto space-y-6">
  <PageHeader
    title="Custom Skills Catalog (@skills)"
    subtitle="Skill modular yang disuntikkan secara otomatis ke Master AI Router saat trigger dipanggil dalam terminal atau floating chat."
  />

  <AiNavTabs active="skills" />

  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-sm font-bold text-neutral-900 dark:text-white">Active Skills ({skills.length})</h3>
      <p class="text-xs text-neutral-500">Panggil skill langsung dengan mengetik prefix <code class="text-sky-500 font-mono">@nama-skill</code>.</p>
    </div>
    <button
      onclick={openCreate}
      class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold rounded-xl shadow-sm transition flex items-center gap-1.5"
    >
      <span>+ Tambah Skill Baru</span>
    </button>
  </div>

  {#if isLoading}
    <div class="p-12 text-center text-xs text-neutral-400">Memuat katalog skills...</div>
  {:else if skills.length === 0}
    <div class="p-12 text-center text-xs text-neutral-400 border border-neutral-200 dark:border-neutral-800 rounded-xl bg-neutral-50 dark:bg-neutral-900/40">
      Belum ada skill terdaftar. Klik "+ Tambah Skill Baru" untuk membuat skill pertama.
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each skills as s}
        <div class="{CARD} space-y-3 relative flex flex-col justify-between">
          <div>
            <div class="flex items-start justify-between gap-2">
              <div>
                <div class="flex items-center gap-2">
                  <span class="text-sm font-bold text-sky-600 dark:text-sky-400 font-mono">@{s.name}</span>
                  {#if s.is_builtin}
                    <span class="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-500 uppercase font-bold">BUILTIN</span>
                  {/if}
                  <span class="text-[9px] px-1.5 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-400 uppercase font-semibold">{s.category}</span>
                </div>
                <h4 class="text-xs font-semibold text-neutral-900 dark:text-white mt-1">{s.title}</h4>
              </div>

              <div class="flex items-center gap-1 shrink-0">
                <button
                  type="button"
                  onclick={() => openEdit(s)}
                  class="p-1.5 text-neutral-400 hover:text-sky-500 transition"
                  title="Edit Skill"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"/></svg>
                </button>
                {#if !s.is_builtin}
                  <button
                    type="button"
                    onclick={() => handleDelete(s)}
                    class="p-1.5 text-neutral-400 hover:text-rose-500 transition"
                    title="Hapus Skill"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
                  </button>
                {/if}
              </div>
            </div>

            <p class="text-xs text-neutral-500 mt-2 line-clamp-2">{s.description}</p>
          </div>

          <div class="pt-3 border-t border-neutral-100 dark:border-neutral-800 text-[11px] space-y-1">
            <div class="flex flex-wrap items-center gap-1">
              <span class="text-neutral-400 text-[10px] uppercase font-bold">Triggers:</span>
              {#each s.triggers as t}
                <span class="px-1.5 py-0.2 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 font-mono text-[10px]">{t}</span>
              {/each}
            </div>
            <div class="flex flex-wrap items-center gap-1">
              <span class="text-neutral-400 text-[10px] uppercase font-bold">Tools:</span>
              {#each s.allowed_tools as tool}
                <span class="px-1.5 py-0.2 rounded bg-emerald-500/10 text-emerald-500 font-mono text-[10px]">{tool}</span>
              {/each}
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- MODAL FORM -->
{#if showModal}
  <div class="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4">
    <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-2xl w-full max-w-xl max-h-[90vh] overflow-y-auto p-6 space-y-4 shadow-2xl">
      <div class="flex items-center justify-between pb-3 border-b border-neutral-200 dark:border-neutral-800">
        <h3 class="text-sm font-bold text-neutral-900 dark:text-white">
          {formId ? 'Edit Custom Skill' : 'Buat Custom Skill Baru'}
        </h3>
        <button onclick={() => (showModal = false)} class="text-neutral-400 hover:text-neutral-200">✕</button>
      </div>

      <div class="space-y-3 text-xs">
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class={LABEL}>Skill Name (Unique Identifier)</label>
            <input type="text" bind:value={formName} placeholder="k8s-triage" class={INPUT} />
          </div>
          <div>
            <label class={LABEL}>Category</label>
            <input type="text" bind:value={formCategory} placeholder="devops, sre, security" class={INPUT} />
          </div>
        </div>

        <div>
          <label class={LABEL}>Skill Title</label>
          <input type="text" bind:value={formTitle} placeholder="Kubernetes Pod Crash Triage" class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>Description</label>
          <input type="text" bind:value={formDescription} placeholder="Ringkasan singkat fungsi skill" class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>Triggers (Koma pisah)</label>
          <input type="text" bind:value={formTriggers} placeholder="@k8s-triage, k8s crash, pod error" class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>Allowed Tools (Koma pisah)</label>
          <input type="text" bind:value={formTools} placeholder="terminal, file_reader, kubctl" class={INPUT} />
        </div>

        <div>
          <label class={LABEL}>System Instructions (SOP Prompt)</label>
          <textarea rows="6" bind:value={formInstructions} class={TEXTAREA} placeholder="Instruksi spesifik SOP saat skill ini aktif..."></textarea>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-3 border-t border-neutral-200 dark:border-neutral-800">
        <button
          onclick={() => (showModal = false)}
          class="px-4 py-2 rounded-xl text-xs text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800"
        >
          Batal
        </button>
        <button
          onclick={handleSave}
          disabled={isSaving}
          class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white rounded-xl text-xs font-semibold disabled:opacity-50"
        >
          {isSaving ? 'Menyimpan...' : 'Simpan Skill'}
        </button>
      </div>
    </div>
  </div>
{/if}
