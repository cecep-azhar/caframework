import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface EntitlementState {
  tier: 'free' | 'pro' | 'pro_grace' | 'free_needs_online_check';
  billingType: string;
  isPro: boolean;
  expiresAt: string | null;
  graceUntil: string | null;
  warning: string | null;
}

export interface BillingPlan {
  planId: string;
  billingType: 'monthly' | 'yearly' | 'lifetime';
  priceMinor: number;
  currency: 'IDR' | 'USD';
}

function createBillingStore() {
  const { subscribe, set, update } = writable<EntitlementState>({
    tier: 'free',
    billingType: 'free',
    isPro: false,
    expiresAt: null,
    graceUntil: null,
    warning: null
  });

  return {
    subscribe,
    set,
    update,
    async refresh(): Promise<EntitlementState> {
      try {
        const state = await invoke<EntitlementState>('billing_get_effective_state');
        set(state);
        return state;
      } catch (err) {
        console.error('Failed to refresh entitlement state:', err);
        throw err;
      }
    },
    async requestOtp(email: string): Promise<void> {
      return invoke('billing_request_otp', { email });
    },
    async verifyOtp(email: string, otpCode: string): Promise<void> {
      await invoke('billing_verify_otp', { email, otpCode });
      await this.refresh();
    },
    async activateLicense(licenseKey: string): Promise<void> {
      await invoke('billing_activate_license', { licenseKey });
      await this.refresh();
    },
    async createCheckout(planId: string): Promise<{ checkoutId: string; checkoutUrl: string; completionMode: 'poll' | 'license_key' }> {
      return invoke('billing_create_checkout', { planId });
    },
    async pollCheckout(checkoutId: string): Promise<string> {
      const status = await invoke<string>('billing_poll_checkout', { checkoutId });
      if (status === 'paid') {
        await this.refresh();
      }
      return status;
    },
    async signOut(): Promise<void> {
      await invoke('billing_sign_out');
      set({
        tier: 'free',
        billingType: 'free',
        isPro: false,
        expiresAt: null,
        graceUntil: null,
        warning: null
      });
    }
  };
}

export const billingStore = createBillingStore();
export const isPro = derived(billingStore, ($b) => $b.isPro);
export const isGrace = derived(billingStore, ($b) => $b.tier === 'pro_grace');
export const needsOnlineCheck = derived(billingStore, ($b) => $b.tier === 'free_needs_online_check');
