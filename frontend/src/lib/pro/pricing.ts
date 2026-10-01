// CAFramework Pro pricing as shown in the app. The real prices and the new-customer discount live in
// Lemon Squeezy (caframework-pricing-tiers-v1.md §1.1 and §3) — these numbers are display only and
// must be changed together with the Lemon Squeezy variants, never on their own.
export const PRO_PRICING = {
  monthly: { list: 10, intro: 6 },
  yearly: { list: 108, intro: 64.8 },
  /** Extra discount for the first 12 months / first yearly cycle of a new customer. */
  introDiscountPercent: 40,
  introMonths: 12,
  /** Permanent discount of yearly vs 12× monthly. */
  yearlyDiscountPercent: 10,
  trialDays: 7,
  /** One account covers up to this many people (owner + 6), at no extra cost. */
  maxMembers: 7,
  syncDevices: 5,
  aiRequestsPerMonth: 300
} as const;

export type BillingInterval = 'monthly' | 'yearly';

/** `$6`, `$64.80` — whole dollars without decimals, cents with exactly two. */
export function formatUsd(amount: number): string {
  return Number.isInteger(amount) ? `$${amount}` : `$${amount.toFixed(2)}`;
}
