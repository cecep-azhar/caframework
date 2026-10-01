<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { getAiSettings, chatWithAi, type AiSettings } from '$lib/api/ai';
  import { showToast } from '$lib/stores/uiNotifications.svelte';

  let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

  let inputPrompt = $state('');
  let messages = $state<{ role: 'user' | 'assistant'; content: string }[]>([
    {
      role: 'assistant',
      content: 'Hello! I am your AI assistant. How can I assist you with your data today?'
    }
  ]);
  let isSending = $state(false);

  async function handleSend() {
    if (!inputPrompt.trim() || isSending) return;
    const prompt = inputPrompt.trim();
    messages.push({ role: 'user', content: prompt });
    inputPrompt = '';
    isSending = true;

    try {
      const response = await chatWithAi(prompt);
      messages.push({ role: 'assistant', content: response });
    } catch (e: any) {
      showToast(e?.message || 'Failed to get AI response', 'error');
      messages.push({
        role: 'assistant',
        content: `Sorry, I encountered an error: ${e?.message || 'Check your AI API settings'}`
      });
    } finally {
      isSending = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-y-0 right-0 z-50 w-96 bg-neutral-900 border-l border-neutral-800 shadow-2xl flex flex-col animate-in slide-in-from-right duration-200">
    <div class="p-4 border-b border-neutral-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div class="w-2.5 h-2.5 rounded-full bg-indigo-500 animate-pulse"></div>
        <h3 class="text-sm font-semibold text-white">AI Assistant</h3>
      </div>
      <button
        onclick={onClose}
        class="p-1 text-neutral-400 hover:text-white rounded-md hover:bg-neutral-800 transition-colors"
        aria-label="Close AI Assistant"
      >
        <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Messages list -->
    <div class="flex-1 overflow-y-auto p-4 space-y-3 text-xs">
      {#each messages as msg}
        <div class="flex flex-col {msg.role === 'user' ? 'items-end' : 'items-start'}">
          <div
            class="max-w-[85%] rounded-2xl px-3.5 py-2.5 leading-relaxed {msg.role === 'user' ? 'bg-indigo-600 text-white rounded-br-none' : 'bg-neutral-800 text-neutral-200 rounded-bl-none border border-neutral-700/60'}"
          >
            {msg.content}
          </div>
        </div>
      {/each}
      {#if isSending}
        <div class="flex items-center gap-2 text-neutral-400 text-xs py-2">
          <svg class="w-4 h-4 animate-spin text-indigo-500" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>Thinking...</span>
        </div>
      {/if}
    </div>

    <!-- Input area -->
    <div class="p-3 border-t border-neutral-800 bg-neutral-950">
      <form
        onsubmit={(e) => {
          e.preventDefault();
          handleSend();
        }}
        class="flex gap-2"
      >
        <input
          type="text"
          bind:value={inputPrompt}
          placeholder="Ask a question..."
          class="flex-1 px-3.5 py-2 bg-neutral-900 border border-neutral-800 rounded-xl text-xs text-white placeholder-neutral-500 focus:outline-none focus:border-indigo-500"
        />
        <button
          type="submit"
          disabled={isSending || !inputPrompt.trim()}
          class="px-3 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-medium rounded-xl transition-colors shrink-0"
        >
          Send
        </button>
      </form>
    </div>
  </div>
{/if}
