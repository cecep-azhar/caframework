import { describe, it, expect } from 'vitest';
import en from './locales/en';
import id from './locales/id';
import * as fs from 'fs';
import * as path from 'path';

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

function getAllFiles(dir: string, ext = ['.svelte', '.ts']): string[] {
  let results: string[] = [];
  const list = fs.readdirSync(dir);
  for (const file of list) {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    if (stat && stat.isDirectory()) {
      results = results.concat(getAllFiles(filePath, ext));
    } else if (ext.some(e => file.endsWith(e)) && !file.endsWith('.test.ts')) {
      results.push(filePath);
    }
  }
  return results;
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

  it('all static t(...) calls in frontend/src exist in dictionaries', () => {
    const srcDir = path.resolve(__dirname, '../../');
    const files = getAllFiles(srcDir);
    const missingKeys: { file: string; key: string }[] = [];

    const regex = /\bt\(\s*['"]([a-zA-Z0-9_.]+)['"]/g;

    for (const file of files) {
      const content = fs.readFileSync(file, 'utf-8');
      let match;
      while ((match = regex.exec(content)) !== null) {
        const key = match[1];
        const enVal = getValue(en, key);
        if (typeof enVal !== 'string') {
          missingKeys.push({ file: path.relative(srcDir, file), key });
        }
      }
    }

    if (missingKeys.length > 0) {
      console.error('Missing i18n keys found:', missingKeys);
    }
    expect(missingKeys).toEqual([]);
  });
});
