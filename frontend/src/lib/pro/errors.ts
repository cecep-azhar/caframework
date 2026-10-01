import { proErrorCode } from '$lib/api/pro';
import { errorText } from '$lib/errors';
import { t } from '$lib/i18n/index.svelte';

/** A translated message for a failed Pro call: `pro.errors.<CODE>` when known, else the generic text. */
export function proErrorMessage(err: unknown): string {
  const code = proErrorCode(err);
  const key = code ? `pro.errors.${code}` : '';
  const text = key ? t(key) : '';
  return text && text !== key ? text : errorText(err);
}
