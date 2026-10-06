<script lang="ts">
  // Settings → Subscription → Team: the owner's team (invite, members, pending invites), the
  // team this account belongs to, and invitations addressed to it. Invitees need no Pro, only a
  // free CATerm account with the invited email (pricing notes §2 v1.3).
  import { onMount } from 'svelte';
  import {
    proTeam,
    proTeamInvite,
    proTeamCancelInvite,
    proTeamRemoveMember,
    proTeamAccept,
    proTeamDecline,
    proTeamLeave,
    type ProPerson,
    type ProTeamView,
    type SyncOutcome
  } from '$lib/api/pro';
  import { getPro, syncPro } from '$lib/stores/pro.svelte';
  import { proErrorMessage } from '$lib/pro/errors';
  import { PRO_PRICING } from '$lib/pro/pricing';
  import { showToast, confirmModal } from '$lib/stores/uiNotifications.svelte';
  import { t, intlLocale, getLocale } from '$lib/i18n/index.svelte';

  /** Receives the heartbeat after joining/leaving, so the page can show the device-limit dialog. */
  let { onOutcome }: { onOutcome?: (outcome: SyncOutcome | null, successText: string) => void } = $props();

  const pro = getPro();
  let view = $state<ProTeamView | null>(null);
  let loadError = $state<string | null>(null);
  let busy = $state(false);
  let inviteEmail = $state('');

  const MUTED = 'text-neutral-500 dark:text-neutral-400';
  const SMALL_BUTTON = 'px-3 py-1.5 text-xs font-medium rounded-md border border-neutral-300 dark:border-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors';
  const LIST = 'divide-y divide-neutral-200 dark:divide-neutral-800 border border-neutral-200 dark:border-neutral-800 rounded-md bg-white dark:bg-neutral-900';
  const DANGER_LINK = 'text-rose-600 dark:text-rose-400 hover:underline shrink-0 disabled:opacity-50';

  const used = $derived(view?.owned ? view.owned.members.length + view.owned.invites.length : 0);
  const full = $derived(!!view && used >= view.maxMembers);

  function who(person: ProPerson): string {
    return person.name || person.email;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString(intlLocale(), { day: 'numeric', month: 'short', year: 'numeric' });
  }

  async function load() {
    loadError = null;
    try {
      view = await proTeam();
    } catch (err) {
      loadError = proErrorMessage(err);
    }
  }

  /** Runs a team call that returns the new view; `after` runs on success. */
  async function run(call: () => Promise<ProTeamView>, after?: (next: ProTeamView) => void | Promise<void>) {
    busy = true;
    try {
      const next = await call();
      view = next;
      await after?.(next);
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    } finally {
      busy = false;
    }
  }

  /** Membership changed: fetch a new licence token so this device gains or loses access now. */
  async function resync(successText: string) {
    try {
      const outcome = await syncPro();
      if (onOutcome) onOutcome(outcome, successText);
      else showToast(successText, 'success');
    } catch (err) {
      showToast(proErrorMessage(err), 'error');
    }
  }

  function invite(event: SubmitEvent) {
    event.preventDefault();
    const email = inviteEmail.trim();
    if (!email) return;
    void run(
      () => proTeamInvite(email, getLocale()),
      () => {
        inviteEmail = '';
        showToast(t('pro.team.invited', { email }), 'success');
      }
    );
  }

  async function removeMember(accountId: string, name: string) {
    const ok = await confirmModal(t('pro.team.removeConfirm', { name }), t('pro.team.removeTitle'), true, t('pro.team.remove'), t('common.cancel'));
    if (ok) void run(() => proTeamRemoveMember(accountId), () => showToast(t('pro.team.removed'), 'info'));
  }

  function cancelInvite(id: string) {
    void run(() => proTeamCancelInvite(id), () => showToast(t('pro.team.inviteCancelled'), 'info'));
  }

  function accept(id: string, owner: ProPerson) {
    void run(() => proTeamAccept(id), () => resync(t('pro.team.accepted', { owner: who(owner) })));
  }

  function decline(id: string) {
    void run(() => proTeamDecline(id), () => showToast(t('pro.team.declined'), 'info'));
  }

  async function leave(owner: ProPerson) {
    const ok = await confirmModal(t('pro.team.leaveConfirm', { owner: who(owner) }), t('pro.team.leave'), true, t('pro.team.leave'), t('common.cancel'));
    if (ok) void run(() => proTeamLeave(), () => resync(t('pro.team.left')));
  }

  onMount(load);
</script>

<div class="border border-neutral-200 dark:border-neutral-800 rounded-lg p-5 bg-neutral-50 dark:bg-neutral-950 space-y-4">
  <div class="flex items-start justify-between gap-3">
    <div>
      <p class="text-sm font-semibold text-neutral-900 dark:text-white">{t('pro.team.title')}</p>
      <p class="text-xs {MUTED} mt-0.5">{t('pro.team.subtitle', { count: PRO_PRICING.maxMembers - 1 })}</p>
    </div>
    <button type="button" onclick={load} disabled={busy} class={SMALL_BUTTON}>{t('pro.team.refresh')}</button>
  </div>

  {#if loadError}
    <div class="flex items-center justify-between gap-3 text-xs">
      <p class={MUTED}>{pro.serverAvailable === false ? t('pro.team.unavailable') : loadError}</p>
      <button type="button" onclick={load} class={SMALL_BUTTON}>{t('pro.team.retry')}</button>
    </div>
  {:else if !view}
    <p class="text-xs {MUTED}">{t('pro.team.loading')}</p>
  {:else}
    <!-- Invitations addressed to this account -->
    {#each view.invitations as invitation (invitation.id)}
      <div class="rounded-md border border-sky-500/40 bg-sky-50 dark:bg-sky-950/30 p-3 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="min-w-0">
          <p class="text-sm font-medium text-neutral-900 dark:text-white">{t('pro.team.invitationTitle', { owner: who(invitation.owner) })}</p>
          <p class="text-xs {MUTED} truncate">
            {invitation.owner.email} · {t('pro.team.invitationExpires', { date: formatDate(invitation.expiresAt) })}
          </p>
          {#if view.membership}
            <p class="text-xs text-amber-700 dark:text-amber-300 mt-1">{t('pro.team.leaveFirst')}</p>
          {/if}
        </div>
        <div class="flex gap-2 shrink-0">
          <button
            type="button"
            onclick={() => accept(invitation.id, invitation.owner)}
            disabled={busy || !!view.membership}
            class="px-3 py-1.5 text-xs font-semibold rounded-md bg-sky-600 hover:bg-sky-500 text-white disabled:opacity-50 transition-colors"
          >
            {t('pro.team.accept')}
          </button>
          <button type="button" onclick={() => decline(invitation.id)} disabled={busy} class={SMALL_BUTTON}>{t('pro.team.decline')}</button>
        </div>
      </div>
    {/each}

    <!-- The team this account belongs to -->
    {#if view.membership}
      {@const membership = view.membership}
      <div class="rounded-md border border-neutral-200 dark:border-neutral-800 bg-white dark:bg-neutral-900 p-3 space-y-2">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div class="min-w-0">
            <p class="text-sm font-medium text-neutral-900 dark:text-white truncate">{t('pro.team.membershipTitle', { owner: who(membership.owner) })}</p>
            <p class="text-xs {MUTED} truncate">{membership.owner.email} · {t('pro.team.joined', { date: formatDate(membership.joinedAt) })}</p>
          </div>
          <span
            class="self-start sm:self-auto text-[11px] font-semibold px-2 py-0.5 rounded-full border {membership.active
              ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-500/30'
              : 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-500/30'}"
          >
            {membership.active ? t('pro.team.membershipActive') : t('pro.team.membershipPaused')}
          </span>
        </div>
        {#if !pro.isPro}
          <p class="text-xs {MUTED}">{t('pro.team.membershipBody', { count: PRO_PRICING.syncDevices })}</p>
        {/if}
        <button type="button" onclick={() => leave(membership.owner)} disabled={busy} class="text-xs {DANGER_LINK}">{t('pro.team.leave')}</button>
      </div>
    {/if}

    <!-- The team this account owns -->
    {#if view.canInvite || view.owned}
      <div class="space-y-3">
        <div class="flex items-center justify-between gap-2">
          <p class="text-xs font-semibold uppercase tracking-wide text-neutral-600 dark:text-neutral-400">{t('pro.team.yourTeam')}</p>
          <p class="text-xs {MUTED}">{t('pro.team.places', { used, max: view.maxMembers })}</p>
        </div>

        {#if view.owned && !view.owned.active}
          <p class="text-xs rounded-md border border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-300 p-2.5">{t('pro.team.ownerInactive')}</p>
        {/if}

        {#if view.canInvite}
          <form class="flex gap-2" onsubmit={invite}>
            <input
              type="email"
              required
              bind:value={inviteEmail}
              disabled={busy || full}
              placeholder={t('pro.team.invitePlaceholder')}
              aria-label={t('pro.team.invite')}
              class="flex-1 min-w-0 px-3 py-1.5 bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 rounded-md text-sm text-neutral-900 dark:text-white placeholder:text-neutral-400 focus:outline-none focus:border-sky-500 disabled:opacity-60"
            />
            <button
              type="submit"
              disabled={busy || full || !inviteEmail.trim()}
              class="px-4 py-1.5 text-sm font-semibold rounded-md bg-sky-600 hover:bg-sky-500 text-white disabled:opacity-50 transition-colors shrink-0"
            >
              {busy ? t('pro.team.inviting') : t('pro.team.invite')}
            </button>
          </form>
          {#if full}
            <p class="text-xs {MUTED}">{t('pro.team.full')}</p>
          {/if}
        {/if}

        {#if !view.owned || used === 0}
          <p class="text-xs {MUTED}">{t('pro.team.empty')}</p>
        {:else}
          <ul class={LIST}>
            {#each view.owned.members as member (member.accountId)}
              <li class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
                <div class="min-w-0">
                  <p class="font-medium text-neutral-900 dark:text-white truncate">{member.name || member.email}</p>
                  <p class="text-neutral-500 truncate">
                    {[member.name ? member.email : '', t('pro.team.joined', { date: formatDate(member.joinedAt) }), t('pro.team.memberDevices', { count: member.activeDevices })]
                      .filter(Boolean)
                      .join(' · ')}
                  </p>
                </div>
                <button type="button" onclick={() => removeMember(member.accountId, member.name || member.email)} disabled={busy} class={DANGER_LINK}>{t('pro.team.remove')}</button>
              </li>
            {/each}
            {#each view.owned.invites as pending (pending.id)}
              <li class="flex items-center justify-between gap-3 px-3 py-2 text-xs">
                <div class="min-w-0">
                  <p class="font-medium text-neutral-700 dark:text-neutral-300 truncate">{pending.email}</p>
                  <p class="text-neutral-500 truncate">{t('pro.team.pending', { date: formatDate(pending.expiresAt) })}</p>
                </div>
                <button type="button" onclick={() => cancelInvite(pending.id)} disabled={busy} class="text-neutral-600 dark:text-neutral-300 hover:underline shrink-0 disabled:opacity-50">{t('pro.team.cancelInvite')}</button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {:else if !view.membership && view.invitations.length === 0}
      <p class="text-xs {MUTED}">{t('pro.team.upsell', { count: PRO_PRICING.maxMembers - 1 })}</p>
    {/if}
  {/if}
</div>
