import { intlLocale } from './i18n/index.svelte';

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(intlLocale(), options).format(value);
}

export function formatCurrency(value: number, currency = 'USD', options?: Intl.NumberFormatOptions): string {
  const isIdr = currency.toUpperCase() === 'IDR';
  const defaultFractionDigits = isIdr ? 0 : 2;
  return new Intl.NumberFormat(intlLocale(), {
    style: 'currency',
    currency,
    minimumFractionDigits: options?.minimumFractionDigits ?? defaultFractionDigits,
    maximumFractionDigits: options?.maximumFractionDigits ?? defaultFractionDigits,
    ...options
  }).format(value);
}

export function formatDate(date: Date | number | string, options?: Intl.DateTimeFormatOptions): string {
  const d = typeof date === 'string' || typeof date === 'number' ? new Date(date) : date;
  return new Intl.DateTimeFormat(intlLocale(), {
    dateStyle: 'medium',
    ...options
  }).format(d);
}

export function formatDateTime(date: Date | number | string, options?: Intl.DateTimeFormatOptions): string {
  const d = typeof date === 'string' || typeof date === 'number' ? new Date(date) : date;
  return new Intl.DateTimeFormat(intlLocale(), {
    dateStyle: 'medium',
    timeStyle: 'short',
    ...options
  }).format(d);
}

export function formatRelative(timestamp: number | Date): string {
  const now = Date.now();
  const time = typeof timestamp === 'number' ? timestamp : timestamp.getTime();
  const diffSec = Math.round((time - now) / 1000);

  const rtf = new Intl.RelativeTimeFormat(intlLocale(), { numeric: 'auto' });

  if (Math.abs(diffSec) < 60) {
    return rtf.format(diffSec, 'second');
  }
  const diffMin = Math.round(diffSec / 60);
  if (Math.abs(diffMin) < 60) {
    return rtf.format(diffMin, 'minute');
  }
  const diffHour = Math.round(diffMin / 60);
  if (Math.abs(diffHour) < 24) {
    return rtf.format(diffHour, 'hour');
  }
  const diffDay = Math.round(diffHour / 24);
  return rtf.format(diffDay, 'day');
}
