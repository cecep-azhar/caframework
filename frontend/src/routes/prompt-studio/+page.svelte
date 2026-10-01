<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { listHosts, type HostRecord } from '$lib/api/hosts';
  import { getTabs, localTerminalHost, LOCAL_HOST_ID } from '$lib/stores/sessionTabs.svelte';
  import { getSessionView } from '$lib/stores/sessionView.svelte';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import {
    getAiSettings,
    aiGeneratePlan,
    aiExecuteStep,
    getProAiUsage,
    type AiPlanStep,
    type AiExecutionPlan,
    type AiExecutionResult,
    type AiSettings,
    type ProAiUsage
  } from '$lib/api/ai';
  import { proErrorMessage } from '$lib/pro/errors';
  import { t } from '$lib/i18n/index.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  interface ExtendedStep extends AiPlanStep {
    enabled: boolean;
    status: 'pending' | 'running' | 'success' | 'failed';
    result?: AiExecutionResult;
    isExpanded?: boolean;
  }

  // Hosts state
  let hosts = $state<HostRecord[]>([]);
  let selectedHostId = $state<string>('');
  let isLoadingHosts = $state(true);

  // AI Settings state
  const DEFAULT_SETTINGS: AiSettings = {
    provider: 'custom',
    base_url: 'http://100.76.150.46:3007/v1',
    api_key: '',
    model: 'deepseek-chat'
  };

  let settings = $state<AiSettings>({ ...DEFAULT_SETTINGS });

  // Hosted AI (CAFramework Pro pooled quota via OmniRoute) vs. Bring-your-own key. Persisted per
  // device only — it's a local preference, not something the account carries around.
  const AI_MODE_KEY = 'caframework_ai_mode';
  let aiMode = $state<'byo' | 'hosted'>('byo');
  let hostedUsage = $state<ProAiUsage | null>(null);
  let isLoadingHostedUsage = $state(false);

  try {
    const savedMode = localStorage.getItem(AI_MODE_KEY);
    if (savedMode === 'hosted' || savedMode === 'byo') aiMode = savedMode;
  } catch {
    // Private browsing / storage blocked: default to BYO.
  }

  async function loadHostedUsage() {
    isLoadingHostedUsage = true;
    try {
      hostedUsage = await getProAiUsage();
    } catch {
      hostedUsage = null;
    } finally {
      isLoadingHostedUsage = false;
    }
  }

  function setAiMode(mode: 'byo' | 'hosted') {
    aiMode = mode;
    try {
      localStorage.setItem(AI_MODE_KEY, mode);
    } catch {
      // ignore
    }
    if (mode === 'hosted' && !hostedUsage) {
      void loadHostedUsage();
    }
  }

  let hostedQuotaExceeded = $derived(
    hostedUsage !== null && hostedUsage.entitled && hostedUsage.remaining <= 0
  );

  // Prompt & Plan state
  let goal = $state('');
  let isGenerating = $state(false);
  let plan = $state<AiExecutionPlan | null>(null);
  let steps = $state<ExtendedStep[]>([]);
  let viewMode = $state<'input' | 'review' | 'executing' | 'completed'>('input');

  // Execution state
  let isExecuting = $state(false);
  let currentExecutingIndex = $state<number>(-1);
  let executionSuccessCount = $state<number>(0);
  let executionFailedCount = $state<number>(0);
  let copiedStepIndex = $state<number | null>(null);

  // Quick prompt suggestion chips
  // Each translated goal keeps the keyword `generateLocalPlan` matches on (laravel, docker,
  // node, keamanan/security, postgres), so the offline planner picks the same template in
  // every language.
  const quickPrompts = $derived(
    (['laravel', 'docker', 'node', 'hardening', 'postgres'] as const).map((key) => ({
      label: t(`promptStudio.quick.${key}Label`),
      goal: t(`promptStudio.quick.${key}Goal`)
    }))
  );

  onMount(async () => {
    const tasks = [loadHosts(), loadSettings()];
    if (aiMode === 'hosted') tasks.push(loadHostedUsage());
    await Promise.all(tasks);
  });

  async function loadHosts() {
  	isLoadingHosts = true;
  	try {
  		const data = await listHosts();
  		hosts = [localTerminalHost(), ...data];
  		const activeTabs = getTabs();
  		const view = getSessionView();
  		const activeTab = activeTabs.find((t) => t.id === view.selectedTabId) || activeTabs[0];
  		if (activeTab?.host?.id) {
  			selectedHostId = activeTab.host.id;
  		} else if (data.length > 0 && data[0]?.id) {
  			selectedHostId = data[0].id;
  		} else {
  			selectedHostId = LOCAL_HOST_ID;
  		}
  	} catch {
  		hosts = [localTerminalHost()];
  		selectedHostId = LOCAL_HOST_ID;
  	} finally {
  		isLoadingHosts = false;
  	}
  }

  async function loadSettings() {
    try {
      const saved = await getAiSettings();
      if (saved && saved.base_url) {
        settings = saved;
        return;
      }
    } catch {
      // fallback to localStorage
    }

    try {
      const cached = localStorage.getItem('caframework_ai_settings');
      if (cached) {
        settings = JSON.parse(cached);
      }
    } catch {
      // ignore
    }
  }

  function selectQuickPrompt(promptGoal: string) {
    goal = promptGoal;
  }

  function generateLocalPlan(goalText: string): AiExecutionPlan {
    const lower = goalText.toLowerCase();
    if (lower.includes('laravel')) {
      return {
        summary: t('promptStudio.tpl.laravel.summary'),
        requirements: ['PHP 8.3', 'Composer', 'MySQL 8.0', 'Node.js 20', 'Git', 'Nginx'],
        steps: [
          {
            step_number: 1,
            title: t('promptStudio.tpl.laravel.s1Title'),
            description: t('promptStudio.tpl.laravel.s1Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y curl zip unzip git software-properties-common ca-certificates apt-transport-https',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: t('promptStudio.tpl.laravel.s2Title'),
            description: t('promptStudio.tpl.laravel.s2Desc'),
            command: 'sudo add-apt-repository -y ppa:ondrej/php && sudo apt-get update && sudo apt-get install -y php8.3 php8.3-cli php8.3-common php8.3-fpm php8.3-mysql php8.3-mbstring php8.3-xml php8.3-curl php8.3-zip php8.3-bcmath php8.3-intl php8.3-gd php8.3-sqlite3',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: t('promptStudio.tpl.laravel.s3Title'),
            description: t('promptStudio.tpl.laravel.s3Desc'),
            command: 'curl -sS https://getcomposer.org/installer -o /tmp/composer-setup.php && sudo php /tmp/composer-setup.php --install-dir=/usr/local/bin --filename=composer && rm /tmp/composer-setup.php',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: t('promptStudio.tpl.laravel.s4Title'),
            description: t('promptStudio.tpl.laravel.s4Desc'),
            command: 'sudo apt-get install -y mysql-server && sudo systemctl enable --now mysql',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: t('promptStudio.tpl.laravel.s5Title'),
            description: t('promptStudio.tpl.laravel.s5Desc'),
            command: 'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && sudo apt-get install -y nodejs',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 6,
            title: t('promptStudio.tpl.laravel.s6Title'),
            description: t('promptStudio.tpl.laravel.s6Desc'),
            command: 'php -v && composer --version && node -v && npm -v && mysql --version',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('docker')) {
      return {
        summary: t('promptStudio.tpl.docker.summary'),
        requirements: ['Docker CE', 'Docker Compose v2', 'containerd.io', t('promptStudio.req.userPermissions')],
        steps: [
          {
            step_number: 1,
            title: t('promptStudio.tpl.docker.s1Title'),
            description: t('promptStudio.tpl.docker.s1Desc'),
            command: 'for pkg in docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc; do sudo apt-get remove -y $pkg; done || true',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: t('promptStudio.tpl.docker.s2Title'),
            description: t('promptStudio.tpl.docker.s2Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y ca-certificates curl && sudo install -m 0755 -d /etc/apt/keyrings && sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc && sudo chmod a+r /etc/apt/keyrings/docker.asc && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \\"$VERSION_CODENAME\\") stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: t('promptStudio.tpl.docker.s3Title'),
            description: t('promptStudio.tpl.docker.s3Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: t('promptStudio.tpl.docker.s4Title'),
            description: t('promptStudio.tpl.docker.s4Desc'),
            command: 'sudo systemctl enable --now docker && sudo usermod -aG docker $USER',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: t('promptStudio.tpl.docker.s5Title'),
            description: t('promptStudio.tpl.docker.s5Desc'),
            command: 'docker --version && docker compose version && sudo docker run --rm hello-world',
            is_sudo: true,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('node') || lower.includes('fullstack')) {
      return {
        summary: t('promptStudio.tpl.node.summary'),
        requirements: ['Node.js 20 LTS', 'npm', 'pnpm', 'PM2', 'Nginx', 'Build Essential'],
        steps: [
          {
            step_number: 1,
            title: t('promptStudio.tpl.node.s1Title'),
            description: t('promptStudio.tpl.node.s1Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y curl git build-essential',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: t('promptStudio.tpl.node.s2Title'),
            description: t('promptStudio.tpl.node.s2Desc'),
            command: 'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: t('promptStudio.tpl.node.s3Title'),
            description: t('promptStudio.tpl.node.s3Desc'),
            command: 'sudo apt-get install -y nodejs',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: t('promptStudio.tpl.node.s4Title'),
            description: t('promptStudio.tpl.node.s4Desc'),
            command: 'sudo npm install -g pnpm pm2',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 5,
            title: t('promptStudio.tpl.node.s5Title'),
            description: t('promptStudio.tpl.node.s5Desc'),
            command: 'sudo apt-get install -y nginx && sudo systemctl enable --now nginx',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 6,
            title: t('promptStudio.tpl.node.s6Title'),
            description: t('promptStudio.tpl.node.s6Desc'),
            command: 'node -v && npm -v && pnpm -v && pm2 -v && nginx -v',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('hardening') || lower.includes('security') || lower.includes('keamanan') || lower.includes('penguatan')) {
      return {
        summary: t('promptStudio.tpl.hardening.summary'),
        requirements: ['UFW Firewall', 'Fail2ban', t('promptStudio.req.sshHardening'), 'Unattended Upgrades'],
        steps: [
          {
            step_number: 1,
            title: t('promptStudio.tpl.hardening.s1Title'),
            description: t('promptStudio.tpl.hardening.s1Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y ufw fail2ban unattended-upgrades',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: t('promptStudio.tpl.hardening.s2Title'),
            description: t('promptStudio.tpl.hardening.s2Desc'),
            command: 'sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw allow 22/tcp comment "SSH" && sudo ufw --force enable',
            is_sudo: true,
            is_danger: true
          },
          {
            step_number: 3,
            title: t('promptStudio.tpl.hardening.s3Title'),
            description: t('promptStudio.tpl.hardening.s3Desc'),
            command: 'sudo systemctl enable --now fail2ban && sudo fail2ban-client status',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: t('promptStudio.tpl.hardening.s4Title'),
            description: t('promptStudio.tpl.hardening.s4Desc'),
            command: 'sudo sed -i -E "s/^#?PermitRootLogin.*/PermitRootLogin no/" /etc/ssh/sshd_config && sudo sed -i -E "s/^#?PasswordAuthentication.*/PasswordAuthentication no/" /etc/ssh/sshd_config && (sudo systemctl reload ssh || sudo systemctl reload sshd)',
            is_sudo: true,
            is_danger: true
          },
          {
            step_number: 5,
            title: t('promptStudio.tpl.hardening.s5Title'),
            description: t('promptStudio.tpl.hardening.s5Desc'),
            command: 'sudo dpkg-reconfigure -f noninteractive unattended-upgrades && sudo systemctl status unattended-upgrades --no-pager',
            is_sudo: true,
            is_danger: false
          }
        ]
      };
    }
    if (lower.includes('postgres') || lower.includes('redis')) {
      return {
        summary: t('promptStudio.tpl.postgres.summary'),
        requirements: ['PostgreSQL 16', 'Postgres Contrib', 'Redis Server', t('promptStudio.req.serviceConfig')],
        steps: [
          {
            step_number: 1,
            title: t('promptStudio.tpl.postgres.s1Title'),
            description: t('promptStudio.tpl.postgres.s1Desc'),
            command: 'sudo apt-get update && sudo apt-get install -y postgresql postgresql-contrib',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 2,
            title: t('promptStudio.tpl.postgres.s2Title'),
            description: t('promptStudio.tpl.postgres.s2Desc'),
            command: 'sudo apt-get install -y redis-server && sudo systemctl enable --now redis-server',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 3,
            title: t('promptStudio.tpl.postgres.s3Title'),
            description: t('promptStudio.tpl.postgres.s3Desc'),
            command: 'sudo systemctl enable --now postgresql && sudo -u postgres psql -c "SELECT version();"',
            is_sudo: true,
            is_danger: false
          },
          {
            step_number: 4,
            title: t('promptStudio.tpl.postgres.s4Title'),
            description: t('promptStudio.tpl.postgres.s4Desc'),
            command: 'redis-cli ping',
            is_sudo: false,
            is_danger: false
          }
        ]
      };
    }

    // Generic fallback
    return {
      summary: t('promptStudio.tpl.generic.summary', { goal: goalText }),
      requirements: ['Bash', t('promptStudio.req.aptManager'), 'Systemd'],
      steps: [
        {
          step_number: 1,
          title: t('promptStudio.tpl.generic.s1Title'),
          description: t('promptStudio.tpl.generic.s1Desc'),
          command: 'sudo apt-get update',
          is_sudo: true,
          is_danger: false
        },
        {
          step_number: 2,
          title: t('promptStudio.tpl.generic.s2Title', { goal: goalText.slice(0, 35) }),
          description: t('promptStudio.tpl.generic.s2Desc', { goal: goalText }),
          command: `echo "Configuring for: ${goalText.replace(/"/g, '\\"')}"`,
          is_sudo: false,
          is_danger: false
        },
        {
          step_number: 3,
          title: t('promptStudio.tpl.generic.s3Title'),
          description: t('promptStudio.tpl.generic.s3Desc'),
          command: 'uptime && free -h && df -h /',
          is_sudo: false,
          is_danger: false
        }
      ]
    };
  }

  async function handleGeneratePlan() {
    if (!goal.trim()) {
      showToast(t('promptStudio.errGoal'), 'error');
      return;
    }

    if (!selectedHostId && hosts.length > 0) {
      selectedHostId = hosts[0].id;
    }

    isGenerating = true;
    try {
      let resultPlan: AiExecutionPlan;
      const hosted = aiMode === 'hosted';
      try {
        resultPlan = await aiGeneratePlan(goal, selectedHostId || undefined, hosted);
        if (hosted) void loadHostedUsage(); // refresh the quota bar after a counted request
      } catch (err) {
        if (hosted) {
          // Unlike BYO, a hosted failure (quota exhausted, not entitled, gateway down) is
          // surfaced rather than papered over with a fake template plan — the request was
          // meant to spend real pooled quota, so silence here would just be confusing.
          showToast(proErrorMessage(err), 'error');
          void loadHostedUsage();
          return;
        }
        // BYO backend unreachable entirely — fall back to the in-page template planner.
        // Marked as `builtin` so the badge on the plan says so instead of implying the model
        // wrote it.
        resultPlan = { ...generateLocalPlan(goal), source: 'builtin' };
      }

      plan = resultPlan;
      steps = resultPlan.steps.map((s, idx) => ({
        ...s,
        step_number: s.step_number || idx + 1,
        enabled: true,
        status: 'pending',
        is_sudo: s.is_sudo ?? s.command.includes('sudo'),
        is_danger: s.is_danger ?? /rm\s+-rf|ufw\s+--force|fdisk|mkfs|shutdown|reboot/i.test(s.command),
        isExpanded: false
      }));

      viewMode = 'review';
      showToast(t('promptStudio.planReady'), 'success');
    } catch (err: any) {
      showToast(err?.message || t('promptStudio.planFailed'), 'error');
    } finally {
      isGenerating = false;
    }
  }

  function handleReset() {
    plan = null;
    steps = [];
    viewMode = 'input';
    isExecuting = false;
    currentExecutingIndex = -1;
    executionSuccessCount = 0;
    executionFailedCount = 0;
  }

  function toggleStep(index: number) {
    if (steps[index]) {
      steps[index].enabled = !steps[index].enabled;
    }
  }

  function toggleAllSteps(enabled: boolean) {
    steps = steps.map(s => ({ ...s, enabled }));
  }

  async function copyCommand(command: string, index: number) {
    try {
      await navigator.clipboard.writeText(command);
      copiedStepIndex = index;
      setTimeout(() => {
        if (copiedStepIndex === index) copiedStepIndex = null;
      }, 2000);
    } catch {
      showToast(t('promptStudio.copyFailed'), 'error');
    }
  }

  async function executeSingleStep(step: ExtendedStep): Promise<AiExecutionResult> {
    const startTime = Date.now();
    try {
      return await aiExecuteStep(selectedHostId, {
        step_number: step.step_number,
        title: step.title,
        command: step.command,
        description: step.description,
        is_sudo: step.is_sudo,
        is_danger: step.is_danger
      });
    } catch (err: any) {
      const errMsg = err?.message || String(err);
      return {
        step_number: step.step_number,
        success: false,
        stdout: '',
        stderr: errMsg,
        exit_code: 1,
        duration_ms: Date.now() - startTime
      };
    }
  }

  async function handleExecutePlan() {
    const approvedSteps = steps.filter((s) => s.enabled);
    if (approvedSteps.length === 0) {
      showToast(t('promptStudio.errSelectStep'), 'error');
      return;
    }

    viewMode = 'executing';
    isExecuting = true;
    executionSuccessCount = 0;
    executionFailedCount = 0;

    for (let i = 0; i < steps.length; i++) {
      const step = steps[i];
      if (!step.enabled) {
        continue;
      }

      currentExecutingIndex = i;
      step.status = 'running';
      step.isExpanded = true;

      const result = await executeSingleStep(step);
      step.result = result;

      if (result.success) {
        step.status = 'success';
        executionSuccessCount++;
      } else {
        step.status = 'failed';
        executionFailedCount++;
      }
    }

    currentExecutingIndex = -1;
    isExecuting = false;
    viewMode = 'completed';

    if (executionFailedCount === 0) {
      showToast(t('promptStudio.allDoneToast'), 'success');
    } else {
      showToast(t('promptStudio.someFailedToast', { count: executionFailedCount }), 'error');
    }
  }

  let selectedHost = $derived(hosts.find((h) => h.id === selectedHostId));
  let approvedCount = $derived(steps.filter((s) => s.enabled).length);
</script>

<div class="max-w-5xl mx-auto space-y-6 text-neutral-800 dark:text-neutral-200">
  <PageHeader
    icon={['M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z']}
    accent="violet"
    title={t('promptStudio.title')}
    badge={t('promptStudio.badge')}
    subtitle={t('promptStudio.subtitle')}
  >
    {#snippet actions()}
      <!-- Target Host Dropdown -->
      <div class="flex items-center gap-2 bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg px-3 py-1.5 shadow-sm">
        <label for="target-host-select" class="text-xs text-neutral-500 dark:text-neutral-400 font-medium whitespace-nowrap">{t('promptStudio.targetHost')}</label>
        {#if isLoadingHosts}
          <span class="text-xs text-neutral-400">{t('promptStudio.loadingHosts')}</span>
        {:else if hosts.length === 0}
          <span class="text-xs text-amber-600 dark:text-amber-500 font-medium">{t('promptStudio.noHosts')}</span>
        {:else}
          <select
            id="target-host-select"
            bind:value={selectedHostId}
            class="bg-transparent text-xs md:text-sm font-semibold text-neutral-900 dark:text-white focus:outline-none cursor-pointer max-w-[200px] truncate"
          >
            {#each hosts as host}
            	<option value={host.id} class="bg-white dark:bg-neutral-900 text-neutral-900 dark:text-white">
            		{host.id === LOCAL_HOST_ID ? '🖥️ ' : ''}{host.label} ({host.username}@{host.address}{host.port > 0 ? `:${host.port}` : ''})
            	</option>
            {/each}
          </select>
        {/if}
      </div>

      <!-- AI config now lives on the Settings page so there is exactly one place for it. -->
      <a
        href="/settings"
        title={t('promptStudio.aiSettingsTitle')}
        aria-label={t('promptStudio.aiSettings')}
        class="p-2 rounded-lg bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 text-neutral-600 dark:text-neutral-300 hover:text-neutral-900 dark:hover:text-white hover:bg-neutral-100 dark:hover:bg-neutral-800 shadow-sm transition-colors inline-flex"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </a>
    {/snippet}
  </PageHeader>

  <!-- Goal Input Section -->
  <section class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-4 md:p-5 shadow-sm space-y-4">
    <div>
      <label for="ai-ops-goal-input" class="block text-sm font-semibold text-neutral-800 dark:text-neutral-200 mb-2">
        {t('promptStudio.goalLabel')}
      </label>
      <textarea
        id="ai-ops-goal-input"
        bind:value={goal}
        rows="3"
        placeholder={t('promptStudio.goalPlaceholder')}
        class="w-full bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-lg p-3 text-sm text-neutral-900 dark:text-white placeholder-neutral-400 dark:placeholder-neutral-600 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-colors"
      ></textarea>
    </div>

    <!-- AI Mode: hosted pooled quota (CAFramework Pro) vs. bring-your-own key -->
    <div class="flex flex-wrap items-center gap-3 p-3 rounded-lg bg-neutral-50 dark:bg-neutral-950/60 border border-neutral-200 dark:border-neutral-800">
      <div class="inline-flex rounded-lg border border-neutral-200 dark:border-neutral-800 overflow-hidden shrink-0" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={aiMode === 'byo'}
          onclick={() => setAiMode('byo')}
          class="px-3 py-1.5 text-xs font-semibold transition-colors {aiMode === 'byo'
            ? 'bg-violet-600 text-white'
            : 'bg-white dark:bg-neutral-900 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
        >
          {t('promptStudio.byoAi')}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={aiMode === 'hosted'}
          onclick={() => setAiMode('hosted')}
          class="px-3 py-1.5 text-xs font-semibold transition-colors {aiMode === 'hosted'
            ? 'bg-violet-600 text-white'
            : 'bg-white dark:bg-neutral-900 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800'}"
        >
          {t('promptStudio.hostedAi')}
        </button>
      </div>

      {#if aiMode === 'byo'}
        <span class="text-xs text-neutral-500 dark:text-neutral-400">{t('promptStudio.byoAiHint')}</span>
      {:else if isLoadingHostedUsage}
        <span class="text-xs text-neutral-400 animate-pulse">{t('common.loading')}</span>
      {:else if hostedUsage && !hostedUsage.enabled}
        <span class="text-xs text-amber-600 dark:text-amber-500">{t('promptStudio.hostedUnavailable')}</span>
      {:else if hostedUsage && !hostedUsage.entitled}
        <span class="text-xs text-amber-600 dark:text-amber-500">{t('promptStudio.hostedNotEntitled')}</span>
      {:else if hostedUsage}
        <div class="flex-1 min-w-[180px] flex items-center gap-2">
          <div class="flex-1 h-1.5 rounded-full bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
            <div
              class="h-full rounded-full transition-all {hostedQuotaExceeded ? 'bg-rose-500' : 'bg-violet-500'}"
              style="width: {Math.min(100, (hostedUsage.used / Math.max(hostedUsage.limit, 1)) * 100)}%"
            ></div>
          </div>
          <span class="text-xs whitespace-nowrap {hostedQuotaExceeded ? 'text-rose-600 dark:text-rose-400 font-semibold' : 'text-neutral-500 dark:text-neutral-400'}">
            {t(hostedUsage.pooled ? 'promptStudio.hostedQuotaPooled' : 'promptStudio.hostedQuota', {
              used: hostedUsage.used,
              limit: hostedUsage.limit
            })}
          </span>
        </div>
      {:else}
        <span class="text-xs text-neutral-500 dark:text-neutral-400">{t('promptStudio.hostedAiHint')}</span>
      {/if}
    </div>

    {#if aiMode === 'hosted' && hostedQuotaExceeded}
      <p class="text-xs text-rose-600 dark:text-rose-400">
        {t('promptStudio.hostedQuotaExceeded', {
          date: hostedUsage ? new Date(hostedUsage.resetsAt * 1000).toLocaleDateString() : ''
        })}
      </p>
    {/if}

    <!-- Quick Prompt Suggestion Chips -->
    <div class="space-y-1.5">
      <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">{t('promptStudio.quickSuggestions')}</span>
      <div class="flex flex-wrap gap-2">
        {#each quickPrompts as qp}
          <button
            type="button"
            onclick={() => selectQuickPrompt(qp.goal)}
            class="px-3 py-1.5 rounded-lg text-xs font-medium bg-neutral-100 dark:bg-neutral-800 hover:bg-violet-500/10 dark:hover:bg-violet-500/20 text-neutral-700 dark:text-neutral-300 hover:text-violet-600 dark:hover:text-violet-400 border border-neutral-200 dark:border-neutral-700 transition-colors cursor-pointer"
          >
            {qp.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Action Bar -->
    <div class="flex items-center justify-between pt-2">
      <div class="text-xs text-neutral-500 dark:text-neutral-400 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
        {#if aiMode === 'hosted'}
          {t('promptStudio.hostedAi')}
        {:else}
          {t('promptStudio.activeModel')} <span class="font-mono text-neutral-700 dark:text-neutral-300 font-semibold">{settings.model}</span>
        {/if}
      </div>

      <button
        type="button"
        onclick={handleGeneratePlan}
        disabled={isGenerating || !goal.trim() || (aiMode === 'hosted' && hostedQuotaExceeded)}
        class="px-5 py-2.5 rounded-lg bg-violet-600 hover:bg-violet-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-medium text-sm flex items-center gap-2 shadow-md shadow-violet-600/20 transition-all cursor-pointer"
      >
        {#if isGenerating}
          <svg class="animate-spin w-4 h-4 text-white" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>{t('promptStudio.analyzing')}</span>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span>{t('promptStudio.generate')}</span>
        {/if}
      </button>
    </div>
  </section>

  <!-- Review & Confirmation View ('Konfirmasi Seluruhnya') -->
  {#if plan && (viewMode === 'review' || viewMode === 'executing' || viewMode === 'completed')}
    <section class="space-y-4">
      <!-- Plan Summary & Requirements Card -->
      <div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-xl p-5 shadow-sm space-y-3">
        <div class="flex flex-col md:flex-row md:items-start justify-between gap-3">
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-lg font-bold text-neutral-900 dark:text-white">{t('promptStudio.planTitle')}</h2>
              <span class="text-xs px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-semibold border border-emerald-500/20">
                {t('promptStudio.stepsSelected', { approved: approvedCount, total: steps.length })}
              </span>
              <!-- Says which planner actually produced this. The LLM path silently falls back
                   to the offline template planner when the endpoint is unreachable, and without
                   this badge a broken connection still looked like a working assistant. -->
              {#if plan?.source === 'llm'}
                <span class="text-xs px-2 py-0.5 rounded bg-violet-500/10 text-violet-600 dark:text-violet-400 font-semibold border border-violet-500/20" title={t('promptStudio.sourceLlmTitle')}>
                  {t('promptStudio.sourceLlm')}
                </span>
              {:else}
                <span class="text-xs px-2 py-0.5 rounded bg-amber-500/10 text-amber-600 dark:text-amber-400 font-semibold border border-amber-500/20" title={t('promptStudio.sourceBuiltinTitle')}>
                  {t('promptStudio.sourceBuiltin')}
                </span>
              {/if}
            </div>
            <p class="text-sm text-neutral-600 dark:text-neutral-400 mt-1 leading-relaxed">{plan.summary}</p>
          </div>

          <!-- Quick Actions: Select All / Deselect All -->
          {#if viewMode === 'review'}
            <div class="flex items-center gap-2 shrink-0 text-xs">
              <button
                type="button"
                onclick={() => toggleAllSteps(true)}
                class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium transition-colors"
              >
                {t('promptStudio.selectAll')}
              </button>
              <button
                type="button"
                onclick={() => toggleAllSteps(false)}
                class="px-2.5 py-1 rounded bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium transition-colors"
              >
                {t('promptStudio.deselectAll')}
              </button>
            </div>
          {/if}
        </div>

        <!-- Requirement Tags -->
        {#if plan.requirements && plan.requirements.length > 0}
          <div class="pt-2 border-t border-neutral-100 dark:border-neutral-800/80 flex flex-wrap items-center gap-2">
            <span class="text-xs font-medium text-neutral-500 dark:text-neutral-400">{t('promptStudio.components')}</span>
            {#each plan.requirements as req}
              <span class="text-xs font-medium px-2.5 py-1 rounded-md bg-violet-500/10 text-violet-600 dark:text-violet-400 border border-violet-500/20">
                {req}
              </span>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Step-by-Step Checklist of Commands -->
      <div class="space-y-3">
        {#each steps as step, index}
          <div
            class="bg-white dark:bg-neutral-900 border transition-all rounded-xl overflow-hidden shadow-sm {step.enabled ? 'border-neutral-300 dark:border-neutral-800' : 'border-neutral-200 dark:border-neutral-800/40 opacity-60'}"
          >
            <!-- Step Header Bar -->
            <div class="p-4 flex items-start gap-3 bg-neutral-50/50 dark:bg-neutral-900/50">
              <!-- Include / Exclude Checkbox -->
              <div class="pt-0.5">
                <input
                  type="checkbox"
                  id={`step-check-${index}`}
                  checked={step.enabled}
                  disabled={viewMode === 'executing'}
                  onchange={() => toggleStep(index)}
                  class="w-4 h-4 rounded text-violet-600 focus:ring-violet-500 border-neutral-300 dark:border-neutral-700 cursor-pointer"
                />
              </div>

              <!-- Step Number Badge -->
              <div class="w-6 h-6 rounded-full bg-neutral-200 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 font-mono text-xs font-bold flex items-center justify-center shrink-0">
                {step.step_number}
              </div>

              <!-- Step Info & Badges -->
              <div class="flex-1 min-w-0">
                <div class="flex flex-wrap items-center gap-2 mb-1">
                  <label for={`step-check-${index}`} class="font-semibold text-sm text-neutral-900 dark:text-white cursor-pointer truncate">
                    {step.title}
                  </label>

                  <!-- Sudo Badge -->
                  {#if step.is_sudo}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30 uppercase">
                      {t('promptStudio.sudo')}
                    </span>
                  {/if}

                  <!-- Danger / Risk Badge -->
                  {#if step.is_danger}
                    <span class="text-[10px] font-mono font-bold px-1.5 py-0.5 rounded bg-rose-500/15 text-rose-600 dark:text-rose-400 border border-rose-500/30 uppercase">
                      {t('promptStudio.risk')}
                    </span>
                  {/if}

                  <!-- Status Pill (for executing & completed views) -->
                  {#if viewMode === 'executing' || viewMode === 'completed'}
                    {#if step.status === 'pending'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-neutral-200 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400">
                        {t('promptStudio.statusPending')}
                      </span>
                    {:else if step.status === 'running'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-sky-500/20 text-sky-600 dark:text-sky-400 border border-sky-500/30 flex items-center gap-1.5 animate-pulse">
                        <svg class="animate-spin w-3 h-3" fill="none" viewBox="0 0 24 24">
                          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        {t('promptStudio.statusRunning')}
                      </span>
                    {:else if step.status === 'success'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                        {t('promptStudio.statusSuccess')}
                      </span>
                    {:else if step.status === 'failed'}
                      <span class="text-[10px] font-medium px-2 py-0.5 rounded bg-rose-500/20 text-rose-600 dark:text-rose-400 border border-rose-500/30 flex items-center gap-1">
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                        {t('promptStudio.statusFailed')}
                      </span>
                    {/if}
                  {/if}
                </div>
                <p class="text-xs text-neutral-500 dark:text-neutral-400">{step.description}</p>
              </div>

              <!-- Action: Copy Command Button -->
              <button
                type="button"
                onclick={() => copyCommand(step.command, index)}
                title={t('promptStudio.copyTitle')}
                class="px-2.5 py-1 text-xs rounded bg-neutral-200/70 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300 flex items-center gap-1.5 transition-colors cursor-pointer shrink-0"
              >
                {#if copiedStepIndex === index}
                  <svg class="w-3.5 h-3.5 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-emerald-600 dark:text-emerald-500 font-semibold">{t('promptStudio.copied')}</span>
                {:else}
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                  <span>{t('promptStudio.copy')}</span>
                {/if}
              </button>
            </div>

            <!-- Editable Bash Command Block -->
            <div class="px-4 pb-4">
              <label for={`step-command-${index}`} class="sr-only">{t('promptStudio.commandLabel', { n: step.step_number })}</label>
              <div class="relative">
                <textarea
                  id={`step-command-${index}`}
                  bind:value={step.command}
                  rows="2"
                  disabled={viewMode === 'executing'}
                  class="w-full font-mono text-xs bg-neutral-50 text-emerald-700 dark:bg-neutral-950 dark:text-emerald-400 border border-neutral-300 dark:border-neutral-800 rounded-lg p-3 focus:outline-none focus:border-violet-500 transition-colors resize-y leading-relaxed"
                ></textarea>
              </div>
            </div>

            <!-- Live Output Viewer (For executing / completed view) -->
            {#if (viewMode === 'executing' || viewMode === 'completed') && (step.result || step.status === 'running')}
              <div class="border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-950 p-3">
                <div class="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400 mb-2 font-mono">
                  <div class="flex items-center gap-2">
                    <span class="w-2 h-2 rounded-full {step.status === 'running' ? 'bg-sky-400 animate-pulse' : step.status === 'success' ? 'bg-emerald-400' : 'bg-rose-400'}"></span>
                    <span>{t('promptStudio.outputTitle', { n: step.step_number })}</span>
                    {#if step.result?.duration_ms}
                      <span class="text-neutral-500">({step.result.duration_ms}ms)</span>
                    {/if}
                  </div>
                  {#if step.result?.exit_code !== undefined}
                    <span class="px-1.5 py-0.5 rounded text-[10px] {step.result.exit_code === 0 ? 'bg-emerald-50 dark:bg-emerald-950 text-emerald-700 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800' : 'bg-rose-50 dark:bg-rose-950 text-rose-700 dark:text-rose-400 border border-rose-200 dark:border-rose-800'}">
                      {t('promptStudio.exitCode', { code: step.result.exit_code })}
                    </span>
                  {/if}
                </div>

                <!-- Terminal-like stdout/stderr block -->
                <div class="bg-white dark:bg-black text-neutral-800 dark:text-neutral-200 border border-neutral-200 dark:border-transparent font-mono text-xs rounded p-3 overflow-x-auto max-h-48 overflow-y-auto space-y-1">
                  {#if step.status === 'running' && !step.result}
                    <div class="text-sky-600 dark:text-sky-400 flex items-center gap-2">
                      <span class="animate-pulse">▶</span>
                      <span>{t('promptStudio.executingOnHost', { host: selectedHost ? selectedHost.label : '' })}</span>
                    </div>
                  {/if}

                  {#if step.result?.stdout}
                    <pre class="text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap">{step.result.stdout}</pre>
                  {/if}

                  {#if step.result?.stderr}
                    <pre class="text-rose-600 dark:text-rose-400 whitespace-pre-wrap">{step.result.stderr}</pre>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Action Buttons for Review Mode -->
      {#if viewMode === 'review'}
        <div class="flex flex-col sm:flex-row items-center justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-800">
          <button
            type="button"
            onclick={handleReset}
            class="w-full sm:w-auto px-5 py-2.5 rounded-lg bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-800 dark:text-neutral-200 font-medium text-sm transition-colors cursor-pointer"
          >
            {t('promptStudio.cancelReset')}
          </button>

          <button
            type="button"
            onclick={handleExecutePlan}
            disabled={approvedCount === 0}
            class="w-full sm:w-auto px-6 py-2.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold text-sm flex items-center justify-center gap-2 shadow-lg shadow-emerald-600/25 transition-all cursor-pointer"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
            <span>{t('promptStudio.approveExecute', { count: approvedCount })}</span>
          </button>
        </div>
      {/if}

      <!-- Live Execution Progress Bar -->
      {#if viewMode === 'executing'}
        <div class="bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800/60 rounded-xl p-4 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <svg class="animate-spin w-5 h-5 text-sky-500" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <div>
              <p class="font-semibold text-sm text-sky-900 dark:text-sky-200">
                {t('promptStudio.executingStep', { current: currentExecutingIndex + 1, total: steps.length })}
              </p>
              <p class="text-xs text-sky-700 dark:text-sky-400">{t('promptStudio.sequential')}</p>
            </div>
          </div>
          <span class="text-xs font-mono font-bold px-2.5 py-1 rounded bg-sky-500/20 text-sky-600 dark:text-sky-300">
            {t('promptStudio.progress', { ok: executionSuccessCount, failed: executionFailedCount })}
          </span>
        </div>
      {/if}

      <!-- Completion Banner -->
      {#if viewMode === 'completed'}
        <div class="bg-white dark:bg-neutral-900 border {executionFailedCount === 0 ? 'border-emerald-500/50 dark:border-emerald-500/30 bg-emerald-50/20 dark:bg-emerald-950/20' : 'border-amber-500/50 dark:border-amber-500/30 bg-amber-50/20 dark:bg-amber-950/20'} rounded-xl p-5 shadow-md space-y-4">
          <div class="flex items-start gap-3">
            <div class="w-10 h-10 rounded-full {executionFailedCount === 0 ? 'bg-emerald-500/20 text-emerald-600 dark:text-emerald-500' : 'bg-amber-500/20 text-amber-600 dark:text-amber-500'} flex items-center justify-center shrink-0">
              {#if executionFailedCount === 0}
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              {:else}
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              {/if}
            </div>
            <div class="flex-1">
              <h3 class="font-bold text-base text-neutral-900 dark:text-white">
                {executionFailedCount === 0 ? t('promptStudio.allSucceeded') : t('promptStudio.completedWarnings')}
              </h3>
              <p class="text-xs md:text-sm text-neutral-600 dark:text-neutral-400 mt-0.5">
                {t('promptStudio.succeededOn', { count: executionSuccessCount })} <span class="font-semibold text-neutral-900 dark:text-white">{selectedHost ? selectedHost.label : t('promptStudio.targetFallback')}</span>.
                {#if executionFailedCount > 0}
                  {t('promptStudio.failedSteps', { count: executionFailedCount })}
                {/if}
              </p>
            </div>
          </div>

          <!-- Navigation Shortcuts -->
          <div class="flex flex-wrap items-center gap-3 pt-2 border-t border-neutral-200 dark:border-neutral-800">
            {#if selectedHostId}
              <a
                href={`/session?host=${selectedHostId}`}
                class="px-4 py-2 rounded-lg bg-sky-600 hover:bg-sky-500 text-white font-medium text-xs flex items-center gap-2 shadow transition-colors"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                <span>{t('promptStudio.openTerminal')}</span>
              </a>
            {/if}

            <a
              href="/command-logs"
              class="px-4 py-2 rounded-lg bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 dark:hover:text-white text-neutral-800 dark:text-neutral-300 font-medium text-xs flex items-center gap-2 transition-colors"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <span>{t('promptStudio.viewLogs')}</span>
            </a>

            <button
              type="button"
              onclick={handleReset}
              class="ml-auto px-4 py-2 rounded-lg text-xs font-medium text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-white transition-colors"
            >
              {t('promptStudio.startNew')}
            </button>
          </div>
        </div>
      {/if}
    </section>
  {/if}

  <!-- AI Settings Modal -->

</div>
