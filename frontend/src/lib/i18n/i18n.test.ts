import { describe, it, expect } from 'vitest';
import en from './locales/en';
import id from './locales/id';

function getLeafKeys(obj: Record<string, any>, prefix = ''): string[] {
  let keys: string[] = [];
  for (const k of Object.keys(obj)) {
    const val = obj[k];
    const path = prefix ? `${prefix}.${k}` : k;
    if (typeof val === 'object' && val !== null && !Array.isArray(val)) {
      keys = keys.concat(getLeafKeys(val, path));
    } else {
      keys.push(path);
    }
  }
  return keys;
}

function getValue(obj: Record<string, any>, path: string): any {
  return path.split('.').reduce((o, k) => o?.[k], obj);
}

describe('i18n completeness and dictionaries (F15.2)', () => {
  const enKeys = getLeafKeys(en);
  const idKeys = getLeafKeys(id);

  it('all keys in en exist in id and have non-empty strings', () => {
    for (const key of enKeys) {
      const enVal = getValue(en, key);
      const idVal = getValue(id, key);
      expect(typeof enVal).toBe('string');
      expect(enVal.trim().length).toBeGreaterThan(0);
      expect(typeof idVal).toBe('string');
      expect(idVal.trim().length).toBeGreaterThan(0);
    }
  });

  it('no extra keys in id that do not exist in en', () => {
    for (const key of idKeys) {
      const enVal = getValue(en, key);
      expect(enVal).toBeDefined();
    }
  });
});
