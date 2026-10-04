export async function openExternalUrl(url: string): Promise<void> {
  if (!url) return;
  if (typeof window !== 'undefined') {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}
