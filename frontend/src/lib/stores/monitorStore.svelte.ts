import { pollActiveMetrics, type HostMetrics } from '../api/monitor';
import { getTabs } from './sessionTabs.svelte';

export const monitorState = $state({
    metrics: [] as HostMetrics[],
    lastUpdated: null as Date | null,
    isPolling: false
});

let timer: ReturnType<typeof setInterval> | null = null;

export async function fetchMetrics() {
    if (typeof document !== 'undefined' && document.hidden) return;
    const hasActiveSessions = getTabs().length > 0;
    if (!(typeof window !== 'undefined' && (window.location.pathname.includes('/monitoring') || hasActiveSessions))) return;
    // C-18: `monitorState.isPolling` was tracked but never checked here, so it did nothing to
    // stop overlapping polls. Each tick calls into `poll_active_metrics`, which runs a blocking
    // SSH exec (including a `top -bn1` sample) against every host with an open pane — on a slow
    // host, or several hosts, one tick can easily take longer than the 5s interval. Without this
    // guard, the next `setInterval` firing starts a second poll on top of the first; since each
    // host's exec session is a single pooled, mutex-serialized session
    // (`crate::ssh::with_exec_session`), the second poll's requests don't run in parallel with
    // the first's, they just queue up behind it — and if that host stays slow, every subsequent
    // tick adds another queued poll faster than they drain, growing an unbounded backlog for as
    // long as the pane stays open.
    if (monitorState.isPolling) return;
    try {
        monitorState.isPolling = true;
        const res = await pollActiveMetrics();
        monitorState.metrics = res;
        monitorState.lastUpdated = new Date();
    } catch (e) {
        console.error("Monitor poll failed:", e);
    } finally {
        monitorState.isPolling = false;
    }
}

export function startMonitoring(intervalMs = 5000) {
    if (timer) return;
    fetchMetrics();
    timer = setInterval(fetchMetrics, intervalMs);
}

export function stopMonitoring() {
    if (timer) {
        clearInterval(timer);
        timer = null;
    }
}
