// Single source of truth for version string prints. Was hardcoded in four places
// and drifted (sidebar said v2.1.7 while build was 2.1.8). package.json bumped in
// lockstep with Cargo workspace / tauri.conf.json on release.
import { version } from '../../package.json';

export const APP_VERSION: string = version;
export const APP_NAME = 'CAFramework';
export const REPO_URL = 'https://github.com/cecep-azhar/caterm';
export const WEBSITE_URL = 'https://caterm.fathforce.com';
export const AUTHOR_URL = 'https://cecepazhar.com';
export const PRICING_URL = `${WEBSITE_URL}/#pricing`;
export const GITHUB_SPONSORS_URL = 'https://github.com/sponsors/cecep-azhar';
/** Primary donation / sponsor-tier channel (Silver $5, Gold $25, Platinum $100 per month). */
export const KOFI_URL = 'https://ko-fi.com/cecepazhar';
/** Secondary one-time donation channel. */
export const PAYPAL_URL = 'https://paypal.me/cecepazhar';

/**
 * Pricing page for the signed-in CATerm account: the landing page passes the account id on to
 * Lemon Squeezy as checkout custom data, so the subscription lands on this account whatever email
 * is typed at checkout. Only the opaque account id travels, never the email.
 */
export function pricingUrl(accountId?: string | null): string {
  if (!accountId || !/^[0-9a-f-]{36}$/i.test(accountId)) return PRICING_URL;
  return `${WEBSITE_URL}/?from=app&account=${accountId}#pricing`;
}

export function releaseNotesUrl(ver: string): string {
  return `${REPO_URL}/releases/tag/v${ver.replace(/^v/, '')}`;
}
