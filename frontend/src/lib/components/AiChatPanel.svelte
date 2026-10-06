<script lang="ts">
  import { onMount } from 'svelte';
  import { aiChat, type AiChatMessage, type AiPlanStep } from '$lib/api/ai';
  import { getProfile, saveProfile } from '$lib/stores/profile.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';
  import { t } from '$lib/i18n/index.svelte';
  import { goto } from '$app/navigation';

  let { onClose }: { onClose: () => void } = $props();

  interface StepRun {
    status: 'pending' | 'running' | 'ok' | 'failed';
    output: string;
  }

  let messages = $state<AiChatMessage[]>([]);
  let draft = $state('');
  let isSending = $state(false);
  let errorMsg = $state('');

  let proposedSteps = $state<AiPlanStep[]>([]);
  let acceptedSteps = $state<boolean[]>([]);
  let runs = $state<StepRun[]>([]);
  let isExecuting = $state(false);

  let scroller: HTMLDivElement | undefined = $state();

  function scrollToBottom() {
    requestAnimationFrame(() => {
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    });
  }

  function cleanMessageContent(content: string): string {
    return content.replace(/\n?\[Active UI Context:[^\]]*\]/g, '').trim();
  }

  async function send() {
    const text = draft.trim();
    if (!text || isSending) return;

    errorMsg = '';
    draft = '';
    messages = [...messages, { role: 'user', content: text }];
    isSending = true;
    scrollToBottom();

    try {
      const reply = await aiChat(messages);
      messages = [...messages, { role: 'assistant', content: reply.reply }];

      if (reply.ready && reply.steps && reply.steps.length > 0) {
        proposedSteps = reply.steps;
        acceptedSteps = reply.steps.map(() => true);
        runs = reply.steps.map(() => ({ status: 'pending', output: '' }));
      } else {
        proposedSteps = [];
      }
    } catch (err) {
      errorMsg = errorText(err);
    } finally {
      isSending = false;
      scrollToBottom();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void send();
    }
  }

  async function executeStepItem(step: AiPlanStep, i: number) {
    runs[i] = { status: 'running', output: '' };
    try {
      if (step.action_type === 'caterm_action' || step.action_type === 'caf_action') {
        const action = (step.action_name || '').toLowerCase();
        const params = step.action_params || {};

        if (action === 'update_profile') {
          const profile = getProfile();
          saveProfile({
            name: params.name || profile.name,
            avatar: params.avatar || profile.avatar
          });
          runs[i] = { status: 'ok', output: `Profil berhasil diperbarui: ${params.name || profile.name}` };
          showToast('Profil diperbarui!', 'success');
          return true;
        }

        if (action === 'navigate' && params.route) {
          goto(params.route);
          runs[i] = { status: 'ok', output: `Navigasi ke ${params.route}` };
          return true;
        }
      }

      runs[i] = { status: 'ok', output: 'Tindakan selesai.' };
      return true;
    } catch (e: any) {
      runs[i] = { status: 'failed', output: String(e) };
      return false;
    }
  }

  async function executeAll() {
    if (isExecuting || proposedSteps.length === 0) return;
    isExecuting = true;

    for (let i = 0; i < proposedSteps.length; i++) {
      if (!acceptedSteps[i]) continue;
      const success = await executeStepItem(proposedSteps[i], i);
      if (!success) break;
    }

    isExecuting = false;
  }
</script>

<aside
  class="w-80 md:w-96 border-l border-neutral-200 dark:border-neutral-800/80 bg-white dark:bg-[#121418] flex flex-col h-full shrink-0 shadow-2xl relative z-20"
>
  <!-- Header -->
  <div class="h-12 border-b border-neutral-200 dark:border-neutral-800 flex items-center justify-between px-4 shrink-0">
    <div class="flex items-center gap-2">
      <div class="w-6 h-6 rounded-md bg-rose-500/10 text-rose-500 flex items-center justify-center">
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
        </svg>
      </div>
      <div>
        <h3 class="text-xs font-bold text-neutral-900 dark:text-white">Hana AI</h3>
        <p class="text-[10px] text-neutral-500">CAFramework Co-Pilot</p>
      </div>
    </div>
    <button
      type="button"
      onclick={onClose}
      class="p-1 rounded-md text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
      title="Tutup Panel AI"
    >
      ×
    </button>
  </div>

  <!-- Messages List -->
  <div bind:this={scroller} class="flex-1 overflow-y-auto p-4 space-y-3 text-xs">
    {#if messages.length === 0}
      <div class="py-12 text-center space-y-2 text-neutral-400">
        <div class="w-10 h-10 rounded-full bg-rose-500/10 text-rose-500 flex items-center justify-center mx-auto">
          🌸
        </div>
        <p class="font-medium text-neutral-700 dark:text-neutral-300">Ada yang bisa Hana bantu?</p>
        <p class="text-[11px] max-w-xs mx-auto">
          Tanyakan seputar navigasi aplikasi, pengaturan profil, manajemen brankas, atau otomatisasi.
        </p>
      </div>
    {/if}

    {#each messages as message, idx (idx)}
      <div class="flex {message.role === 'user' ? 'justify-end' : 'justify-start'}">
        <div
          class="max-w-[85%] rounded-2xl px-3 py-2 text-xs leading-relaxed {message.role === 'user' ? 'bg-sky-600 text-white rounded-br-none shadow-sm' : 'bg-neutral-100 dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 border border-neutral-200 dark:border-neutral-800 rounded-bl-none shadow-xs'}"
        >
          <div class="whitespace-pre-wrap">{cleanMessageContent(message.content)}</div>
        </div>
      </div>
    {/each}

    {#if isSending}
      <div class="flex justify-start">
        <div class="bg-neutral-100 dark:bg-neutral-900 rounded-2xl px-3 py-2 text-xs text-neutral-400 flex items-center gap-1.5 border border-neutral-200 dark:border-neutral-800">
          <span class="w-1.5 h-1.5 rounded-full bg-rose-500 animate-bounce"></span>
          <span class="w-1.5 h-1.5 rounded-full bg-rose-500 animate-bounce [animation-delay:0.2s]"></span>
          <span class="w-1.5 h-1.5 rounded-full bg-rose-500 animate-bounce [animation-delay:0.4s]"></span>
        </div>
      </div>
    {/if}

    <!-- Proposed Steps Plan Box -->
    {#if proposedSteps.length > 0}
      <div class="mt-4 p-3.5 rounded-xl bg-neutral-950 border border-neutral-800 text-neutral-200 space-y-2.5 font-mono text-[11px]">
        <div class="flex items-center justify-between font-sans">
          <span class="font-bold text-white text-xs">📋 Rencana Aksi</span>
          <button
            type="button"
            onclick={executeAll}
            disabled={isExecuting}
            class="px-2.5 py-1 rounded-lg text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white transition-colors disabled:opacity-50"
          >
            {isExecuting ? 'Menjalankan...' : '✓ Approve & Jalankan'}
          </button>
        </div>

        <div class="space-y-1.5">
          {#each proposedSteps as step, i}
            <div class="p-2 rounded-lg bg-neutral-900 border border-neutral-800/80 flex items-center justify-between gap-2">
              <span class="truncate">{step.title}</span>
              <span class="text-[10px] uppercase font-bold {runs[i]?.status === 'ok' ? 'text-emerald-400' : runs[i]?.status === 'failed' ? 'text-rose-400' : 'text-neutral-500'}">
                {runs[i]?.status || 'pending'}
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Input Field -->
  <div class="p-3 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50/50 dark:bg-neutral-950/50">
    {#if errorMsg}
      <p class="text-[11px] text-rose-500 mb-2 px-1">{errorMsg}</p>
    {/if}
    <div class="relative flex items-end gap-1.5">
      <textarea
        bind:value={draft}
        onkeydown={handleKeydown}
        rows="2"
        placeholder="Ketik instruksi atau pertanyaan..."
        class="w-full resize-none pl-3 pr-8 py-2 text-xs rounded-xl bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-800 text-neutral-900 dark:text-white focus:outline-none focus:border-sky-500 shadow-2xs placeholder-neutral-400"
      ></textarea>
      <button
        type="button"
        onclick={send}
        disabled={!draft.trim() || isSending}
        class="absolute right-2 bottom-2 p-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white disabled:opacity-30 transition-all cursor-pointer"
        title="Kirim pesan"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
        </svg>
      </button>
    </div>
  </div>
</aside>
