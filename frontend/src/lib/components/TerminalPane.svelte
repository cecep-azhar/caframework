<script lang="ts">
  import { onMount } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebglAddon } from '@xterm/addon-webgl';
  import { CanvasAddon } from '@xterm/addon-canvas';
  import '@xterm/xterm/css/xterm.css';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import {
    sshConnect,
    sshWrite,
    sshResize,
    sshDisconnect,
    onSshOutput,
    onSshClosed,
    type SshSession
  } from '$lib/api/ssh';
  import type { HostRecord } from '$lib/api/hosts';
  import { listSnippets, type SnippetRecord } from '$lib/api/snippets';
  import { setActiveSession, clearActiveSession } from '$lib/stores/activeSession.svelte';
  import { getTheme, terminalTheme } from '$lib/stores/theme.svelte';
  import { LINUX_COMMANDS, commandDescription, type AutocompleteItem } from '$lib/data/terminalCommands';
  import { t } from '$lib/i18n/index.svelte';
  import TerminalAutocomplete from '$lib/components/TerminalAutocomplete.svelte';
  import { terminalKeyAction } from '$lib/shortcuts';
  import { getTerminalPrefs, stepTerminalFontSize, setTerminalFontSize, DEFAULT_FONT_SIZE } from '$lib/stores/terminalPrefs.svelte';
  import { copyText } from '$lib/utils/clipboard';

  let {
    host,
    label,
    isActive = true,
    onSplitRight,
    onSplitDown,
    onClose
  }: {
    host: HostRecord;
    /** Disambiguated tab name, e.g. "YPC (2)" when the host is open more than once. */
    label?: string;
    /** Whether this pane is the one the user is looking at (drives focus + snippet target). */
    isActive?: boolean;
    onSplitRight?: () => void;
    onSplitDown?: () => void;
    onClose?: () => void;
  } = $props();

  function errorMessage(err: unknown): string {
    if (err && typeof err === 'object' && 'message' in err) {
      return String((err as { message?: unknown }).message);
    }
    return String(err);
  }

  const paneLabel = $derived(label ?? host.label);
  const theme = getTheme();
  const terminalPrefs = getTerminalPrefs();
  let status = $state<'connecting' | 'connected' | 'offline'>('connecting');
  let terminalContainer: HTMLDivElement;
  let rootContainer: HTMLDivElement | undefined = $state();

  let session = $state<SshSession | null>(null);
  let term: Terminal | null = null;
  let fitAddon: FitAddon | null = null;

  // Autocomplete state
  let autocompleteEnabled = $state(true);
  let cursorX = $state(20);
  let cursorY = $state(60);
  let cursorTop = $state(42);
  let currentInputLine = $state('');
  let commandHistory = $state<string[]>([]);
  let userSnippets = $state<SnippetRecord[]>([]);
  let suggestions = $state<AutocompleteItem[]>([]);
  let selectedSuggestionIndex = $state(0);

  // Load user snippets on mount
  onMount(() => {
    listSnippets()
      .then((snips) => {
        userSnippets = snips;
      })
      .catch(() => {});
  });

  // Places the popup anchor just below the terminal cursor. The WebGL/Canvas renderers draw
  // the cursor on a <canvas> (there is no `.xterm-cursor` element), so derive the cell size
  // from the rendered `.xterm-screen` box instead of xterm's private render dimensions.
  function updateCursorPosition() {
    if (!rootContainer || !term) return;
    const screenEl = terminalContainer?.querySelector('.xterm-screen') as HTMLElement | null;
    if (!screenEl || term.cols === 0 || term.rows === 0) return;

    const rootRect = rootContainer.getBoundingClientRect();
    const screenRect = screenEl.getBoundingClientRect();
    const cellWidth = screenRect.width / term.cols;
    const cellHeight = screenRect.height / term.rows;
    const buffer = term.buffer.active;

    cursorX = Math.round(screenRect.left - rootRect.left + buffer.cursorX * cellWidth);
    cursorTop = Math.round(screenRect.top - rootRect.top + buffer.cursorY * cellHeight);
    cursorY = Math.round(screenRect.top - rootRect.top + (buffer.cursorY + 1) * cellHeight);
  }

  // Autocomplete only makes sense against the normal shell line buffer. An alternate-screen
  // program (vim, less, top, htop, ...) owns the screen and its own line editing, so Tab/↑/↓/Esc
  // must reach it untouched and we must not keep suggesting against stale command-line text.
  function isAlternateScreen(): boolean {
    return term?.buffer.active.type === 'alternate';
  }

  function updateSuggestions(input: string) {
    if (!autocompleteEnabled || !input.trim() || isAlternateScreen()) {
      suggestions = [];
      selectedSuggestionIndex = 0;
      return;
    }

    const trimmed = input.trim();
    const lastWord = input.split(/\s+/).filter(Boolean).pop() || '';
    const results: AutocompleteItem[] = [];
    const seen = new Set<string>();

    // 1. Check history matches first (most relevant to user's current session)
    for (let i = commandHistory.length - 1; i >= 0; i--) {
      const hist = commandHistory[i];
      if (hist.toLowerCase().startsWith(trimmed.toLowerCase()) && !seen.has(hist)) {
        seen.add(hist);
        results.push({
          text: hist,
          type: 'history',
          desc: t('terminal.recentCommand')
        });
        if (results.length >= 3) break;
      }
    }

    // 2. User snippets
    for (const snip of userSnippets) {
      if (
        (snip.command.toLowerCase().startsWith(trimmed.toLowerCase()) ||
          snip.label.toLowerCase().includes(trimmed.toLowerCase())) &&
        !seen.has(snip.command)
      ) {
        seen.add(snip.command);
        results.push({
          text: snip.command,
          type: 'snippet',
          desc: snip.description || snip.label,
          sourceLabel: snip.label
        });
        if (results.length >= 6) break;
      }
    }

    // 3. Common Linux commands matching whole input or last word
    for (const cmd of LINUX_COMMANDS) {
      if (
        (cmd.text.toLowerCase().startsWith(trimmed.toLowerCase()) ||
          (lastWord && cmd.text.toLowerCase().startsWith(lastWord.toLowerCase()))) &&
        !seen.has(cmd.text)
      ) {
        seen.add(cmd.text);
        results.push({
          text: cmd.text,
          type: 'command',
          desc: commandDescription(cmd)
        });
        if (results.length >= 8) break;
      }
    }

    suggestions = results;
    selectedSuggestionIndex = 0;
    if (results.length > 0) {
      updateCursorPosition();
      requestAnimationFrame(updateCursorPosition);
    }
  }

  function applySuggestion(item: AutocompleteItem) {
    if (!session || !term) return;

    // Calculate characters to replace
    const currentLen = currentInputLine.length;
    // Backspace previous typed chars on the active line in terminal
    let backspaces = '';
    for (let i = 0; i < currentLen; i++) {
      backspaces += '\b \b';
    }

    // Erase what user typed on the terminal screen & backend pty line
    let erasePty = '';
    for (let i = 0; i < currentLen; i++) {
      erasePty += '\x7f';
    }

    // Write the completed command text
    void sshWrite(session.sessionId, erasePty + item.text).catch(() => {});

    currentInputLine = item.text;
    suggestions = [];
    selectedSuggestionIndex = 0;
    term.focus();
  }

  function dismissSuggestions() {
    suggestions = [];
    selectedSuggestionIndex = 0;
  }

  function injectCommand(cmd: string) {
    if (!session) return;
    void sshWrite(session.sessionId, cmd + '\r').catch(() => {});
  }

  function markActive() {
    if (session) {
      setActiveSession({ sessionId: session.sessionId, label: paneLabel, inject: injectCommand });
    }
  }

  $effect(() => {
    const palette = terminalTheme(theme.name);
    if (term) term.options.theme = palette;
  });

  // Font size is shared by all panes; a change re-fits the grid and tells the PTY.
  $effect(() => {
    const size = terminalPrefs.fontSize;
    if (!term || term.options.fontSize === size) return;
    term.options.fontSize = size;
    fitAddon?.fit();
    if (session) void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
  });

  $effect(() => {
    if (!isActive || !session) return;
    markActive();
    fitAddon?.fit();
    if (term) {
      void sshResize(session.sessionId, term.cols, term.rows).catch(() => {});
      term.focus();
    }
  });

  onMount(() => {
    const terminal = new Terminal({
      theme: terminalTheme(theme.name),
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      fontSize: terminalPrefs.fontSize,
      cursorBlink: true,
      scrollback: 1000
    });
    term = terminal;

    const fit = new FitAddon();
    fitAddon = fit;
    terminal.loadAddon(fit);
    terminal.open(terminalContainer);

    // Hardware-accelerated terminal renderer with graceful fallbacks: WebGL -> Canvas -> DOM
    try {
    	const webgl = new WebglAddon();
    	webgl.onContextLoss(() => {
    		webgl.dispose();
    	});
    	terminal.loadAddon(webgl);
    } catch {
    	try {
    		const canvasAddon = new CanvasAddon();
    		terminal.loadAddon(canvasAddon);
    	} catch {
    		// Fall back to default DOM renderer
    	}
    }

    fit.fit();

    let disposed = false;
    const unlisteners: UnlistenFn[] = [];
    const early = new Map<string, string[]>();

    function keepUnlisten(unlisten: UnlistenFn) {
      if (disposed) unlisten();
      else unlisteners.push(unlisten);
    }

    let reportedFailure = false;
    function reportFailure(what: string, err: unknown) {
      if (disposed || reportedFailure) return;
      reportedFailure = true;
      terminal.write(`\r\n\x1b[31m${what}: ${errorMessage(err)}\x1b[0m\r\n`);
    }

    terminal.writeln(
      `\x1b[1;32m${t('terminal.welcome')}\x1b[0m — ${t('terminal.connectingTo')} \x1b[1;36m${host.label}\x1b[0m (${host.address})...`
    );

    void (async () => {
      try {
        keepUnlisten(
          await onSshOutput(({ sessionId, data }) => {
            if (disposed) return;
            if (session) {
              if (sessionId === session.sessionId) terminal.write(data);
              return;
            }
            const buffered = early.get(sessionId) ?? [];
            buffered.push(data);
            early.set(sessionId, buffered);
          })
        );

        keepUnlisten(
          await onSshClosed(({ sessionId }) => {
            if (disposed || !session || sessionId !== session.sessionId) return;
            status = 'offline';
            terminal.write(`\r\n\x1b[33m${t('terminal.closedByRemote')}\x1b[0m\r\n`);
          })
        );
      } catch (err) {
        if (!disposed) {
          status = 'offline';
          terminal.write(
            `\r\n\x1b[31m${t('terminal.subscribeFailed')}: ${errorMessage(err)}\x1b[0m\r\n`
          );
        }
        return;
      }

      try {
        const opened = await sshConnect(host.id);
        if (disposed) {
          void sshDisconnect(opened.sessionId).catch(() => {});
          return;
        }

        session = opened;
        status = 'connected';
        fit.fit();
        void sshResize(opened.sessionId, terminal.cols, terminal.rows).catch((err) =>
          reportFailure(t('terminal.resizeFailed'), err)
        );
        terminal.write(`\r\n\x1b[32m${t('terminal.connected')}\x1b[0m (${t('terminal.session')} ${opened.sessionId})\r\n`);

        for (const chunk of early.get(opened.sessionId) ?? []) terminal.write(chunk);
        early.clear();
      } catch (err) {
        if (disposed) return;
        status = 'offline';
        terminal.write(`\r\n\x1b[31m${t('terminal.sshFailed')}: ${errorMessage(err)}\x1b[0m\r\n`);
      }
    })();

    // Intercept keystrokes for copy/paste, font size and autocomplete. App-wide shortcuts
    // (tabs, palette, …) never get here: the layout catches them in the capture phase.
    terminal.attachCustomKeyEventHandler((event: KeyboardEvent) => {
      const action = terminalKeyAction(event, terminal.hasSelection());
      if (action) {
        if (action === 'paste') {
          // Not preventDefault: the browser's own paste event reaches xterm, which pastes
          // (bracketed where the shell wants it) without needing clipboard-read permission.
          return false;
        }
        event.preventDefault();
        if (action === 'copy') {
          const text = terminal.getSelection();
          terminal.clearSelection();
          void copyText(text).then(() => terminal.focus());
        } else if (action === 'fontUp') {
          stepTerminalFontSize(1);
        } else if (action === 'fontDown') {
          stepTerminalFontSize(-1);
        } else {
          setTerminalFontSize(DEFAULT_FONT_SIZE);
        }
        return false;
      }
      if (suggestions.length > 0 && !isAlternateScreen()) {
        if (event.type === 'keydown') {
          if (event.key === 'Tab') {
            event.preventDefault();
            const picked = suggestions[selectedSuggestionIndex];
            if (picked) applySuggestion(picked);
            return false;
          }
          if (event.key === 'ArrowDown') {
            event.preventDefault();
            selectedSuggestionIndex = (selectedSuggestionIndex + 1) % suggestions.length;
            return false;
          }
          if (event.key === 'ArrowUp') {
            event.preventDefault();
            selectedSuggestionIndex =
              (selectedSuggestionIndex - 1 + suggestions.length) % suggestions.length;
            return false;
          }
          if (event.key === 'Escape') {
            event.preventDefault();
            dismissSuggestions();
            return false;
          }
        }
      }
      return true;
    });

    terminal.onData((data) => {
      // Send keystroke to the real Rust SSH PTY backend
      if (session) {
        void sshWrite(session.sessionId, data).catch((err) => reportFailure(t('terminal.sendFailed'), err));
      }

      if (isAlternateScreen()) {
        // Don't track command-line state or keep suggestions alive while a full-screen program
        // (vim, less, top, ...) owns the terminal — see attachCustomKeyEventHandler above.
        if (suggestions.length > 0) suggestions = [];
        return;
      }

      // Track active line buffer for smart autocomplete
      if (data === '\r' || data === '\n') {
        const finishedCmd = currentInputLine.trim();
        if (finishedCmd) {
          commandHistory = [...commandHistory.filter((c) => c !== finishedCmd), finishedCmd];
        }
        currentInputLine = '';
        suggestions = [];
      } else if (data === '\x7f' || data === '\x08') {
        // Backspace
        currentInputLine = currentInputLine.slice(0, -1);
        updateSuggestions(currentInputLine);
        updateCursorPosition();
      } else if (data === '\x03') {
        // Ctrl+C
        currentInputLine = '';
        suggestions = [];
      } else if (!data.includes('\x1b') && data >= ' ') {
        currentInputLine += data;
        updateSuggestions(currentInputLine);
        updateCursorPosition();
      }
    });

    // Keystrokes are sent before the shell echoes them back, so follow the real cursor once
    // the echo has been rendered instead of using the pre-echo position.
    terminal.onCursorMove(() => {
      if (suggestions.length > 0) updateCursorPosition();
    });

    let fitQueued = false;
    const applyFit = () => {
      fitQueued = false;
      if (terminalContainer.clientWidth === 0 || terminalContainer.clientHeight === 0) return;
      fit.fit();
      if (session) {
        void sshResize(session.sessionId, terminal.cols, terminal.rows).catch(() => {});
      }
      if (suggestions.length > 0) {
        updateCursorPosition();
      }
    };
    const observer = new ResizeObserver(() => {
      if (fitQueued) return;
      fitQueued = true;
      requestAnimationFrame(applyFit);
    });
    observer.observe(terminalContainer);

    const handleClick = () => {
      markActive();
      terminal.focus();
    };
    terminalContainer.addEventListener('click', handleClick);

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
      unlisteners.length = 0;
      observer.disconnect();
      terminalContainer.removeEventListener('click', handleClick);
      if (session) {
        clearActiveSession(session.sessionId);
        void sshDisconnect(session.sessionId).catch(() => {});
      }
      term = null;
      fitAddon = null;
      terminal.dispose();
    };
  });
</script>

<div
	bind:this={rootContainer}
	class="relative flex flex-col h-full bg-white dark:bg-[#09090b] border {status === 'offline' ? 'border-rose-500/60 ring-1 ring-rose-500/30' : 'border-neutral-200 dark:border-neutral-800'} rounded-md overflow-hidden transition-colors"
>
	<!-- Top Bar -->
	<div class="h-7 sm:h-8 bg-neutral-100 dark:bg-neutral-900/90 border-b border-neutral-200 dark:border-neutral-800 px-2.5 flex items-center justify-between text-xs font-mono text-neutral-500 dark:text-neutral-400 shrink-0">
		<div class="flex items-center gap-2 min-w-0">
			<span
				class="w-2.5 h-2.5 rounded-full shrink-0 transition-all duration-300 {status === 'connected'
					? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]'
					: status === 'connecting'
						? 'bg-amber-400 animate-pulse shadow-[0_0_6px_rgba(251,191,36,0.6)]'
						: 'bg-rose-500 shadow-[0_0_10px_rgba(244,63,94,0.9)] animate-pulse ring-2 ring-rose-500/30'}"
				title={status === 'connected' ? t('terminal.connected') : status === 'connecting' ? t('terminal.connecting') : t('terminal.sshFailed')}
			></span>
			<span class="text-neutral-900 dark:text-white font-medium truncate">{paneLabel}</span>
			<span class="text-neutral-500 dark:text-neutral-500 hidden sm:inline truncate">({host.address})</span>
		</div>
    <div class="flex items-center gap-2 text-neutral-500 dark:text-neutral-400 shrink-0">
      <!-- Autocomplete toggle button -->
      <button
        onclick={() => { autocompleteEnabled = !autocompleteEnabled; if (!autocompleteEnabled) suggestions = []; }}
        class="px-1.5 py-0.5 rounded text-[10px] font-sans font-semibold transition-colors flex items-center gap-1 {autocompleteEnabled ? 'bg-sky-500/15 dark:bg-sky-500/20 text-sky-700 dark:text-sky-400 border border-sky-500/30' : 'bg-neutral-200 dark:bg-neutral-800 text-neutral-500 dark:text-neutral-400 border border-transparent'}"
        title={t('terminal.toggleAutocompleteTitle')}
        aria-label={t('terminal.toggleAutocomplete')}
      >
        <span class="w-1.5 h-1.5 rounded-full {autocompleteEnabled ? 'bg-sky-400' : 'bg-neutral-500'}"></span>
        <span>{t('terminal.autocomplete')}</span>
      </button>

      {#if onSplitRight}
        <button onclick={onSplitRight} class="hover:text-sky-600 dark:hover:text-sky-400 p-0.5 rounded transition-colors" title={t('terminal.splitRightTitle')} aria-label={t('terminal.splitRight')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M12 3v18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onSplitDown}
        <button onclick={onSplitDown} class="hover:text-sky-600 dark:hover:text-sky-400 p-0.5 rounded transition-colors" title={t('terminal.splitDownTitle')} aria-label={t('terminal.splitDown')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" d="M3 12h18M5 3h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2z"></path></svg>
        </button>
      {/if}
      {#if onClose}
        <button onclick={onClose} class="hover:text-rose-600 dark:hover:text-rose-400 p-0.5 rounded transition-colors" title={t('terminal.closePane')} aria-label={t('terminal.closePane')}>
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-width="2" stroke-linecap="round" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      {/if}
      <span class="ml-1 text-neutral-500 hidden sm:inline">SSH</span>
      <span class="text-neutral-500 hidden sm:inline">UTF-8</span>
    </div>
  </div>

  <!-- Terminal canvas container -->
  <div bind:this={terminalContainer} class="flex-1 p-1 overflow-hidden outline-none relative"></div>

  <!-- Smart Autocomplete Overlay -->
  {#if autocompleteEnabled && suggestions.length > 0}
    <TerminalAutocomplete
      {suggestions}
      selectedIndex={selectedSuggestionIndex}
      {cursorX}
      {cursorY}
      {cursorTop}
      containerWidth={rootContainer?.clientWidth ?? 0}
      containerHeight={rootContainer?.clientHeight ?? 0}
      onSelect={applySuggestion}
      onClose={dismissSuggestions}
    />
  {/if}
</div>
