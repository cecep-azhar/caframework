<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { t, intlLocale } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  interface CommandLog {
    id: string;
    event_type: string;
    timestamp: number;
    host_id: string | null;
    details: string;
  }

  interface ParsedAiExec {
    isAiExec: boolean;
    command: string;
    exitCode: number | null;
    extra?: string;
  }

  let logs = $state<CommandLog[]>([]);
  let hostFilter = $state('');
  let eventFilter = $state('');
  let dateFilter = $state('');
  let searchQuery = $state('');
  let isLoading = $state(false);

  // Query param support: e.g. /command-logs?filter=AI_AUTOMATION
  $effect(() => {
    const qFilter = page.url.searchParams.get('filter');
    if (qFilter && !eventFilter) {
      eventFilter = qFilter;
    }
  });

  onMount(async () => {
    const qFilter = page.url.searchParams.get('filter');
    if (qFilter) {
      eventFilter = qFilter;
    }
    await fetchLogs();
  });

  async function fetchLogs() {
    isLoading = true;
    try {
      let maxLimit = 1000;
      try {
        const stored = localStorage.getItem('caframework_max_audit_records');
        if (stored) maxLimit = parseInt(stored, 10) || 1000;
      } catch {}

      logs = await invoke<CommandLog[]>('get_command_logs', {
        hostId: null,
        search: null,
        limit: maxLimit
      });
    } catch (e) {
      console.error('Failed to fetch logs', e);
      showToast(t('commandLogs.loadFailed'), 'error');
    } finally {
      isLoading = false;
    }
  }

  function parseAiExec(details: string): ParsedAiExec {
    if (!details || !details.includes('[AI-EXEC]')) {
      return { isAiExec: false, command: '', exitCode: null };
    }

    const afterTag = details.replace('[AI-EXEC]', '').trim();
    let exitCode: number | null = null;
    let command = '';
    let extra = '';

    const exitMatch =
      afterTag.match(/(?:exit(?:_code)?|status|code)\s*[:=]\s*(-?\d+)/i) ||
      afterTag.match(/->\s*exit\s*(-?\d+)/i) ||
      afterTag.match(/\(exit:\s*(-?\d+)\)/i);

    if (exitMatch) {
      exitCode = parseInt(exitMatch[1], 10);
    }

    const cmdKeyMatch = afterTag.match(/(?:cmd|command)\s*[:=]\s*(?:"([^"]+)"|'([^']+)'|([^,;|]+))/i);
    if (cmdKeyMatch) {
      command = (cmdKeyMatch[1] || cmdKeyMatch[2] || cmdKeyMatch[3] || '').trim();
      let rest = afterTag.replace(cmdKeyMatch[0], '');
      if (exitMatch) {
        rest = rest.replace(exitMatch[0], '');
      }
      extra = rest.replace(/^[\s,;|()\-]+|[\s,;|()\-]+$/g, '').trim();
    } else {
      const quoteMatch = afterTag.match(/^"([^"]+)"|^'([^']+)'/);
      if (quoteMatch) {
        command = quoteMatch[1] || quoteMatch[2];
        let rest = afterTag.slice(quoteMatch[0].length);
        if (exitMatch) {
          rest = rest.replace(exitMatch[0], '');
        }
        extra = rest.replace(/^[\s,;|()\-]+|[\s,;|()\-]+$/g, '').trim();
      } else if (exitMatch && exitMatch.index !== undefined) {
        command = afterTag.substring(0, exitMatch.index).replace(/^[\s,;|()\-]+|[\s,;|()\-]+$/g, '').trim();
        extra = afterTag.substring(exitMatch.index + exitMatch[0].length).replace(/^[\s,;|()\-]+|[\s,;|()\-]+$/g, '').trim();
      } else {
        command = afterTag;
      }
    }

    if ((command.startsWith('"') && command.endsWith('"')) || (command.startsWith("'") && command.endsWith("'"))) {
      command = command.slice(1, -1);
    }

    return {
      isAiExec: true,
      command: command || afterTag,
      exitCode,
      extra: extra || undefined
    };
  }

  // Pagination
  const PAGE_SIZE_DEFAULT = 50;
  let pageSize = $state(PAGE_SIZE_DEFAULT);
  let currentPage = $state(1);

  // Reset page when filters change
  $effect(() => {
    hostFilter; eventFilter; dateFilter; searchQuery;
    currentPage = 1;
  });

  let filteredLogs = $derived(
    logs.filter((log) => {
      if (hostFilter && log.host_id !== hostFilter) return false;
      if (eventFilter && log.event_type !== eventFilter) return false;

      if (dateFilter) {
        const logDate = new Date(log.timestamp).toISOString().split('T')[0];
        if (logDate !== dateFilter) return false;
      }

      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        if (
          !log.details.toLowerCase().includes(query) &&
          !log.event_type.toLowerCase().includes(query) &&
          !(log.host_id && log.host_id.toLowerCase().includes(query))
        ) {
          return false;
        }
      }
      return true;
    })
  );

  let totalPages = $derived(Math.max(1, Math.ceil(filteredLogs.length / pageSize)));
  let paginatedLogs = $derived(
    filteredLogs.slice((currentPage - 1) * pageSize, currentPage * pageSize)
  );

  let aiAutomationCount = $derived(logs.filter((l) => l.event_type === 'AI_AUTOMATION').length);
  let aiPlanCount = $derived(logs.filter((l) => l.event_type === 'AI_PLAN').length);

  // Quotes a CSV field and neutralizes formula injection: Excel/Sheets/LibreOffice treat a
  // cell whose *first* character is one of =+-@ (or a tab/CR, which some parsers also honor as
  // a formula prefix) as a formula to evaluate on open, not as text. A command log's `details`
  // is largely attacker- or user-influenced text (an SSH command, an AI-exec note) — without
  // this, a row like `=cmd|'/c calc'!A1` opens as a live formula in whatever spreadsheet app
  // the exported file is opened in. Prefixing with a bare `'` is the standard mitigation: every
  // major spreadsheet app treats a leading apostrophe as "this cell is text", and it's invisible
  // once opened. Every field is escaped and quoted here, not just `details` — `id`/`eventType`/
  // `hostId` were previously written unquoted-content-unescaped (`host_id` especially, since
  // it's a free-form value on some connection types), so any of them containing a `"` would have
  // broken the CSV's column alignment for that row and everything after it.
  function csvField(value: string): string {
    const text = value ?? '';
    const guarded = /^[=+\-@\t\r]/.test(text) ? `'${text}` : text;
    return `"${guarded.replace(/"/g, '""')}"`;
  }

  async function exportLogs() {
    try {
      const csvContent =
        'ID,EventType,Timestamp,HostID,Details\n' +
        filteredLogs
          .map((l) =>
            [
              csvField(l.id),
              csvField(l.event_type),
              csvField(new Date(l.timestamp).toISOString()),
              csvField(l.host_id || ''),
              csvField(l.details)
            ].join(',')
          )
          .join('\n');

      let saved = false;
      try {
        const filePath = await save({
          filters: [
            {
              name: 'CSV',
              extensions: ['csv']
            }
          ]
        });
        if (filePath) {
          await writeTextFile(filePath, csvContent);
          saved = true;
          showToast(t('commandLogs.exportedFile'), 'success');
        }
      } catch {
        // Fallback to standard web download
      }

      if (!saved) {
        const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `caframework-audit-logs-${new Date().toISOString().split('T')[0]}.csv`;
        a.click();
        // Revoking immediately after click() is a race: click() only *schedules* the download,
        // it doesn't start it synchronously, and some webview engines cancel an in-flight
        // download when its blob URL disappears out from under it. Deferring the revoke gives
        // the download a moment to actually begin reading the blob first.
        setTimeout(() => URL.revokeObjectURL(url), 1000);
        showToast(t('commandLogs.exportedDownload'), 'success');
      }
    } catch (e) {
      console.error('Export failed', e);
      showToast(t('commandLogs.exportFailed'), 'error');
    }
  }

  function formatTime(ts: number) {
    return new Date(ts).toLocaleString(intlLocale());
  }
</script>

<div class="max-w-5xl mx-auto space-y-6">
  <PageHeader
    icon={['M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z']}
    accent="sky"
    title={t('commandLogs.title')}
    subtitle={t('commandLogs.subtitle')}
  >
    {#snippet actions()}
      <button
        onclick={fetchLogs}
        disabled={isLoading}
        class="px-3 py-2 bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-200 rounded-lg text-sm transition-colors flex items-center gap-1.5"
        title={t('commandLogs.reload')}
      >
        <svg class="w-4 h-4 {isLoading ? 'animate-spin' : ''}" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        <span>{t('commandLogs.refresh')}</span>
      </button>
      <button
        onclick={exportLogs}
        class="px-4 py-2 bg-sky-600 hover:bg-sky-500 text-white font-medium text-sm rounded-lg transition-colors shadow flex items-center gap-1.5"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        <span>{t('commandLogs.exportCsv')}</span>
      </button>
    {/snippet}
  </PageHeader>

  <!-- Quick Filter Buttons -->
  <div class="flex items-center gap-2 mb-4 overflow-x-auto pb-1 text-xs">
    <span class="text-neutral-500 dark:text-neutral-400 font-medium shrink-0">{t('commandLogs.quickFilter')}</span>
    <button
      type="button"
      onclick={() => (eventFilter = '')}
      class="px-2.5 py-1 rounded-md transition-colors shrink-0 {eventFilter === ''
        ? 'bg-neutral-800 text-white dark:bg-white dark:text-black font-semibold'
        : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700'}"
    >
      {t('commandLogs.all', { count: logs.length })}
    </button>
    <button
      type="button"
      onclick={() => (eventFilter = eventFilter === 'AI_AUTOMATION' ? '' : 'AI_AUTOMATION')}
      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-all shrink-0 {eventFilter === 'AI_AUTOMATION'
        ? 'bg-purple-600 text-white font-semibold ring-2 ring-purple-400/50 shadow-sm'
        : 'bg-purple-500/15 text-purple-700 dark:text-purple-300 hover:bg-purple-500/25 border border-purple-500/30'}"
    >
      <svg class="w-3.5 h-3.5 text-purple-500 dark:text-purple-400" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2l2.4 6.6L21 11l-6.6 2.4L12 20l-2.4-6.6L3 11l6.6-2.4L12 2z" />
      </svg>
      <span>{t('commandLogs.aiAutomation')}</span>
      {#if aiAutomationCount > 0}
        <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-purple-900/40 text-purple-800 dark:text-purple-200 font-semibold">{aiAutomationCount}</span>
      {/if}
    </button>
    <button
      type="button"
      onclick={() => (eventFilter = eventFilter === 'AI_PLAN' ? '' : 'AI_PLAN')}
      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md transition-all shrink-0 {eventFilter === 'AI_PLAN'
        ? 'bg-indigo-600 text-white font-semibold ring-2 ring-indigo-400/50 shadow-sm'
        : 'bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 hover:bg-indigo-500/25 border border-indigo-500/30'}"
    >
      <svg class="w-3.5 h-3.5 text-indigo-500 dark:text-indigo-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
      </svg>
      <span>{t('commandLogs.aiPlan')}</span>
      {#if aiPlanCount > 0}
        <span class="px-1.5 py-0.2 rounded-full text-[10px] bg-indigo-900/40 text-indigo-800 dark:text-indigo-200 font-semibold">{aiPlanCount}</span>
      {/if}
    </button>
    <button
      type="button"
      onclick={() => (eventFilter = eventFilter === 'PTY_COMMAND' ? '' : 'PTY_COMMAND')}
      class="px-2.5 py-1 rounded-md transition-colors shrink-0 {eventFilter === 'PTY_COMMAND'
        ? 'bg-blue-600 text-white font-semibold ring-2 ring-blue-400/50'
        : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700'}"
    >
      {t('commandLogs.ptyCommand')}
    </button>
    <button
      type="button"
      onclick={() => (eventFilter = eventFilter === 'KEY_DEPLOY' ? '' : 'KEY_DEPLOY')}
      class="px-2.5 py-1 rounded-md transition-colors shrink-0 {eventFilter === 'KEY_DEPLOY'
        ? 'bg-cyan-600 text-white font-semibold ring-2 ring-cyan-400/50'
        : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700'}"
    >
      {t('commandLogs.keyDeploy')}
    </button>
  </div>

  <!-- Search & Filter Controls -->
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 mb-4">
    <div class="relative sm:col-span-2 lg:col-span-1">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder={t('commandLogs.searchPlaceholder')}
        class="w-full px-3 py-2 bg-neutral-50 dark:bg-[#2D2D2D] rounded-lg border border-neutral-300 dark:border-neutral-700 text-sm focus:outline-none focus:border-purple-500 transition-colors"
      />
    </div>
    <div>
      <input
        type="text"
        bind:value={hostFilter}
        placeholder={t('commandLogs.hostIdPlaceholder')}
        class="w-full px-3 py-2 bg-neutral-50 dark:bg-[#2D2D2D] rounded-lg border border-neutral-300 dark:border-neutral-700 text-sm focus:outline-none focus:border-purple-500 transition-colors"
      />
    </div>
    <div>
      <select
        bind:value={eventFilter}
        class="w-full px-3 py-2 bg-neutral-50 dark:bg-[#2D2D2D] rounded-lg border border-neutral-300 dark:border-neutral-700 text-sm focus:outline-none focus:border-purple-500 transition-colors"
      >
        <option value="">{t('commandLogs.allEvents')}</option>
        <option value="AI_AUTOMATION">{t('commandLogs.events.AI_AUTOMATION')}</option>
        <option value="AI_PLAN">{t('commandLogs.events.AI_PLAN')}</option>
        <option value="PTY_COMMAND">{t('commandLogs.events.PTY_COMMAND')}</option>
        <option value="TUNNEL_START">{t('commandLogs.events.TUNNEL_START')}</option>
        <option value="TUNNEL_STOP">{t('commandLogs.events.TUNNEL_STOP')}</option>
        <option value="VAULT_LOCK">{t('commandLogs.events.VAULT_LOCK')}</option>
        <option value="VAULT_UNLOCK">{t('commandLogs.events.VAULT_UNLOCK')}</option>
        <option value="KEY_DEPLOY">{t('commandLogs.events.KEY_DEPLOY')}</option>
      </select>
    </div>
    <div>
      <input
        type="date"
        bind:value={dateFilter}
        class="w-full px-3 py-2 bg-neutral-50 dark:bg-[#2D2D2D] rounded-lg border border-neutral-300 dark:border-neutral-700 text-sm focus:outline-none focus:border-purple-500 transition-colors"
      />
    </div>
  </div>

  <!-- Table Container -->
  <div class="flex-1 overflow-auto bg-neutral-50 dark:bg-[#252526] rounded-lg border border-neutral-200 dark:border-neutral-700">
    <table class="w-full text-left border-collapse">
      <thead class="sticky top-0 bg-neutral-100 dark:bg-[#2D2D2D] border-b border-neutral-200 dark:border-neutral-700 text-xs font-semibold text-neutral-600 dark:text-neutral-300">
        <tr>
          <th class="p-3 w-44">{t('commandLogs.tableTime')}</th>
          <th class="p-3 w-40">{t('commandLogs.tableEvent')}</th>
          <th class="p-3 w-32">{t('commandLogs.tableHostId')}</th>
          <th class="p-3">{t('commandLogs.tableDetails')}</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-neutral-200/60 dark:divide-neutral-700/50 text-sm">
        {#each paginatedLogs as log (log.id)}
          <tr class="hover:bg-neutral-100/70 dark:hover:bg-[#2A2D2E] transition-colors">
            <td class="p-3 whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400 font-mono">
              {formatTime(log.timestamp)}
            </td>
            <td class="p-3">
              {#if log.event_type === 'AI_AUTOMATION'}
                <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-mono font-semibold bg-purple-500/20 text-purple-700 dark:text-purple-300 border border-purple-500/40 shadow-sm shadow-purple-950/10">
                  <svg class="w-3.5 h-3.5 text-purple-600 dark:text-purple-400 animate-pulse shrink-0" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l2.4 6.6L21 11l-6.6 2.4L12 20l-2.4-6.6L3 11l6.6-2.4L12 2z" />
                  </svg>
                  AI_AUTOMATION
                </span>
              {:else if log.event_type === 'AI_PLAN'}
                <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-mono font-medium bg-indigo-500/20 text-indigo-700 dark:text-indigo-300 border border-indigo-500/30">
                  <svg class="w-3.5 h-3.5 text-indigo-600 dark:text-indigo-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                  </svg>
                  AI_PLAN
                </span>
              {:else if log.event_type === 'PTY_COMMAND'}
                <span class="px-2 py-1 rounded text-xs font-mono bg-blue-500/20 text-blue-700 dark:text-blue-300 border border-blue-500/30">
                  {log.event_type}
                </span>
              {:else if log.event_type.startsWith('TUNNEL')}
                <span class="px-2 py-1 rounded text-xs font-mono bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                  {log.event_type}
                </span>
              {:else if log.event_type.startsWith('VAULT')}
                <span class="px-2 py-1 rounded text-xs font-mono bg-amber-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/30">
                  {log.event_type}
                </span>
              {:else if log.event_type === 'KEY_DEPLOY'}
                <span class="px-2 py-1 rounded text-xs font-mono bg-cyan-500/20 text-cyan-700 dark:text-cyan-300 border border-cyan-500/30">
                  {log.event_type}
                </span>
              {:else}
                <span class="px-2 py-1 rounded text-xs font-mono bg-neutral-200 dark:bg-neutral-700/50 text-neutral-700 dark:text-neutral-300 border border-neutral-300 dark:border-neutral-700">
                  {log.event_type}
                </span>
              {/if}
            </td>
            <td class="p-3 text-neutral-500 dark:text-neutral-400 font-mono text-xs">
              {log.host_id || '-'}
            </td>
            <td class="p-3">
              {#if log.details.includes('[AI-EXEC]')}
                {@const parsed = parseAiExec(log.details)}
                <div class="flex flex-col gap-1.5 py-0.5 max-w-3xl">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono font-bold bg-purple-500/20 text-purple-700 dark:text-purple-300 border border-purple-500/30">
                      <svg class="w-3 h-3 text-purple-600 dark:text-purple-400" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2l2.4 6.6L21 11l-6.6 2.4L12 20l-2.4-6.6L3 11l6.6-2.4L12 2z" />
                      </svg>
                      AI-EXEC
                    </span>
                    {#if parsed.exitCode !== null}
                      <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-mono font-semibold {parsed.exitCode === 0 ? 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30' : 'bg-rose-500/20 text-rose-700 dark:text-rose-300 border border-rose-500/30'}">
                        <span class="w-1.5 h-1.5 rounded-full {parsed.exitCode === 0 ? 'bg-emerald-500' : 'bg-rose-500 animate-pulse'}"></span>
                        exit: {parsed.exitCode}
                      </span>
                    {/if}
                  </div>
                  <div class="font-mono text-xs bg-neutral-900 text-emerald-400 rounded-md px-3 py-2 border border-neutral-800 break-all select-text shadow-inner">
                    <span class="text-neutral-500 select-none mr-1.5 font-bold">$</span>{parsed.command}
                  </div>
                  {#if parsed.extra}
                    <div class="text-[11px] text-neutral-500 dark:text-neutral-400 font-mono break-all pl-1">
                      {parsed.extra}
                    </div>
                  {/if}
                </div>
              {:else}
                <div class="font-mono text-xs break-all text-neutral-800 dark:text-neutral-200">
                  {log.details}
                </div>
              {/if}
            </td>
          </tr>
        {/each}
        {#if filteredLogs.length === 0}
          <tr>
            <td colspan="4" class="p-10 text-center text-neutral-400 dark:text-neutral-500 text-sm">
              <div class="flex flex-col items-center justify-center gap-2">
                <svg class="w-8 h-8 opacity-40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <span>{t('commandLogs.emptyTitle')}</span>
              </div>
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>

  <!-- Pagination Footer -->
  {#if filteredLogs.length > 0}
    <div class="mt-3 flex flex-wrap items-center justify-between gap-3 text-xs text-neutral-600 dark:text-neutral-400 select-none">
      <div class="flex items-center gap-2">
        <span>{t('commandLogs.showingRecords', { from: (currentPage - 1) * pageSize + 1, to: Math.min(currentPage * pageSize, filteredLogs.length), total: filteredLogs.length })}</span>
        <span class="text-neutral-400 dark:text-neutral-600">|</span>
        <label class="flex items-center gap-1.5">
          <span>{t('commandLogs.rowsPerPage')}</span>
          <select
            bind:value={pageSize}
            class="px-2 py-1 bg-neutral-100 dark:bg-[#2D2D2D] rounded border border-neutral-300 dark:border-neutral-700 text-xs focus:outline-none"
          >
            <option value={25}>25</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
            <option value={200}>200</option>
          </select>
        </label>
      </div>

      <div class="flex items-center gap-1">
        <button
          type="button"
          disabled={currentPage <= 1}
          onclick={() => currentPage = 1}
          class="px-2 py-1 rounded bg-neutral-100 dark:bg-[#2D2D2D] border border-neutral-300 dark:border-neutral-700 disabled:opacity-30 disabled:cursor-not-allowed hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
          title={t('commandLogs.firstPage')}
        >
          «
        </button>
        <button
          type="button"
          disabled={currentPage <= 1}
          onclick={() => currentPage = Math.max(1, currentPage - 1)}
          class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-[#2D2D2D] border border-neutral-300 dark:border-neutral-700 disabled:opacity-30 disabled:cursor-not-allowed hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
        >
          {t('commandLogs.previous')}
        </button>

        <span class="px-3 py-1 font-mono text-neutral-800 dark:text-neutral-200">
          {t('commandLogs.pageOf', { current: currentPage, total: totalPages })}
        </span>

        <button
          type="button"
          disabled={currentPage >= totalPages}
          onclick={() => currentPage = Math.min(totalPages, currentPage + 1)}
          class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-[#2D2D2D] border border-neutral-300 dark:border-neutral-700 disabled:opacity-30 disabled:cursor-not-allowed hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
        >
          {t('commandLogs.next')}
        </button>
        <button
          type="button"
          disabled={currentPage >= totalPages}
          onclick={() => currentPage = totalPages}
          class="px-2 py-1 rounded bg-neutral-100 dark:bg-[#2D2D2D] border border-neutral-300 dark:border-neutral-700 disabled:opacity-30 disabled:cursor-not-allowed hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
          title={t('commandLogs.lastPage')}
        >
          »
        </button>
      </div>
    </div>
  {/if}
</div>
