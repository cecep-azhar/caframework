import { describe, it, expect } from 'vitest';
import { formatNumber, formatCurrency, formatDate, formatDateTime, formatRelative } from './format';
import { setLocale } from './i18n/index.svelte';

describe('Locale formatters (F15.4)', () => {
  it('formats numbers correctly per locale', () => {
    setLocale('en');
    expect(formatNumber(1234567.5)).toBe('1,234,567.5');

    setLocale('id');
    expect(formatNumber(1234567.5)).toBe('1.234.567,5');
  });

  it('formats currency correctly per locale', () => {
    setLocale('en');
    expect(formatCurrency(100, 'USD').replace(/\u00a0/g, ' ')).toBe('$100.00');

    setLocale('id');
    // In IDR, decimals default to 0
    expect(formatCurrency(50000, 'IDR').replace(/\u00a0/g, ' ')).toMatch(/Rp\s?50\.000/);
  });

  it('formats dates and relative time', () => {
    const d = new Date('2026-10-04T12:00:00Z');
    setLocale('en');
    expect(formatDate(d)).toBeDefined();
    expect(formatDateTime(d)).toBeDefined();

    setLocale('id');
    expect(formatDate(d)).toBeDefined();
    expect(formatDateTime(d)).toBeDefined();

    expect(formatRelative(Date.now() - 60000)).toBeDefined();
  });
});
