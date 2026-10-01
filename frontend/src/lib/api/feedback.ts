import { invoke } from '@tauri-apps/api/core';

/** `rating` 1-5, `content` feedback text, optional `name` and `profession` (activity). */
export function submitFeedback(
  rating: number,
  content: string,
  name?: string,
  profession?: string
): Promise<void> {
  return invoke('submit_feedback', {
    rating,
    content,
    name: name?.trim() || null,
    profession: profession?.trim() || null,
  });
}
