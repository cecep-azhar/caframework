// Minimal, dependency-free i18n: a reactive locale plus a `t()` lookup over nested dictionaries.
// English (`locales/en.ts`) is authored first and its inferred shape (`Dictionary`) is what every
// other locale is typed against, so a translation file missing a key fails at compile time
// instead of silently showing the wrong language to one user.
//
// Adding a language later: create `locales/xx.ts` exporting a `Dictionary`-typed default, then
// add one entry to `LOCALES` below. Nothing else in the app needs to change — every page already
// goes through `t()`.
import en, { type Dictionary } from './locales/en';
import id from './locales/id';

export type Locale = 'en' | 'id';

const DICTIONARIES: Record<Locale, Dictionary> = { en, id };

export const LOCALES: { code: Locale; label: string }[] = [
  { code: 'en', label: 'English' },
  { code: 'id', label: 'Indonesia' }
];

const STORAGE_KEY = 'caframework_locale';

function detectInitialLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === 'en' || stored === 'id') return stored;
  } catch {
    // Private mode / blocked storage: fall through to detection below.
  }
  try {
    // First run only: if the OS/browser is set to Indonesian, greet in Indonesian: after that,
    // the explicit choice in localStorage above always wins.
    if (typeof navigator !== 'undefined' && navigator.language?.toLowerCase().startsWith('id')) {
      return 'id';
    }
  } catch {
    // Ignore — default below.
  }
  return 'en';
}

let locale = $state<Locale>(typeof window === 'undefined' ? 'en' : detectInitialLocale());

export function getLocale(): Locale {
  return locale;
}

export function setLocale(next: Locale) {
  locale = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Private mode / blocked storage: the choice still applies for this session.
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = next;
  }
}

const INTL_TAGS: Record<Locale, string> = { en: 'en-US', id: 'id-ID' };

/** BCP 47 tag for `Intl`/`toLocaleString`, so dates and numbers follow the chosen language too. */
export function intlLocale(): string {
  return INTL_TAGS[locale];
}

/** Dot-path lookup, e.g. `resolve(dict, 'hosts.title')`. */
function resolve(dict: Dictionary, path: string): unknown {
  return path.split('.').reduce<unknown>((node, key) => {
    if (node && typeof node === 'object' && key in node) {
      return (node as Record<string, unknown>)[key];
    }
    return undefined;
  }, dict);
}

function interpolate(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (match, name) => {
    const value = vars[name];
    return value === undefined ? match : String(value);
  });
}

/**
 * Translates `key` (dot-path into the dictionary, e.g. `'hosts.addHost'`) in the current locale.
 * `vars.count` picks the `_plural` sibling key when `count !== 1` (English/Indonesian both only
 * need singular vs. plural, unlike languages with richer plural rules — this is intentionally
 * simple, not a full CLDR plural engine). Falls back to English, then to the raw key, so a gap
 * in one locale degrades instead of crashing.
 */
export function t(key: string, vars?: Record<string, string | number>): string {
  const effectiveKey = vars && typeof vars.count === 'number' && vars.count !== 1 ? `${key}_plural` : key;

  const primary = resolve(DICTIONARIES[locale], effectiveKey) ?? resolve(DICTIONARIES[locale], key);
  const fallback = () => resolve(DICTIONARIES.en, effectiveKey) ?? resolve(DICTIONARIES.en, key);
  const value = typeof primary === 'string' ? primary : fallback();

  if (typeof value !== 'string') {
    if (import.meta.env.DEV) console.warn(`[i18n] missing key: ${key}`);
    return key;
  }
  return interpolate(value, vars);
}
