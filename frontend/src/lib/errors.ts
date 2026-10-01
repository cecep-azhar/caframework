/**
 * Readable text for anything thrown across the Tauri boundary.
 *
 * A failing `#[tauri::command]` rejects with the serialized `CatermError` — a plain object of
 * `{ code, message, domain }`, not an `Error`. Code that reached for `String(err)` printed
 * `[object Object]`, which is how a perfectly descriptive backend message ("Respons AI tidak
 * sesuai format...") reached the user as no information at all.
 */
export function errorText(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err instanceof Error) return err.message;

  if (err && typeof err === 'object') {
    const record = err as Record<string, unknown>;
    const message = record.message ?? record.error ?? record.reason;
    if (typeof message === 'string' && message.length > 0) {
      const code = typeof record.code === 'string' ? record.code : undefined;
      return code ? `${message} (${code})` : message;
    }
    try {
      return JSON.stringify(err);
    } catch {
      // Circular or otherwise unserializable — fall through to the generic message.
    }
  }

  return String(err);
}
