import { describe, it, expect } from 'vitest';
import { errorText } from './errors';

describe('errorText smoke tests', () => {
  it('parses structured error object with code, message, and domain', () => {
    const err = {
      code: 'ERR_TIMEOUT',
      message: 'Connection timed out',
      domain: 'network'
    };
    expect(errorText(err)).toBe('Connection timed out (ERR_TIMEOUT)');
  });

  it('handles non-object error (string)', () => {
    const err = 'Plain error message';
    expect(errorText(err)).toBe('Plain error message');
  });

  it('handles non-object error (number/primitive)', () => {
    const err = 404;
    expect(errorText(err)).toBe('404');
  });

  it('handles Error instance', () => {
    const err = new Error('Standard JS error');
    expect(errorText(err)).toBe('Standard JS error');
  });
});
