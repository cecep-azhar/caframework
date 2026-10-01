import { goto } from '$app/navigation';

/**
 * Navigates to the Session route asking it to open a terminal for each host.
 *
 * The `n` parameter is a nonce, and it is the whole point: connecting to a host that is already
 * open must open a *second* session, but a plain `<a href="/session?host=X">` clicked twice is
 * the same URL both times, so the Session route would see no change and open nothing. A fresh
 * nonce makes every Connect a distinct navigation.
 */
export function openSession(hostIds: string | string[]): Promise<void> {
  const ids = (Array.isArray(hostIds) ? hostIds : [hostIds]).filter(Boolean);
  if (ids.length === 0) return Promise.resolve();

  const params = new URLSearchParams();
  if (ids.length === 1) {
    params.set('host', ids[0]);
  } else {
    params.set('hosts', ids.join(','));
  }
  params.set('n', String(Date.now()));

  return goto(`/session?${params.toString()}`);
}
