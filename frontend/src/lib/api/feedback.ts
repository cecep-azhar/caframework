import {
  cmdSubmitFeedback
} from '$lib/generated/commands';

/** `rating` 1-5, `content` feedback text, optional `name` and `profession` (activity). */
export function submitFeedback(
  rating: number,
  content: string,
  name?: string,
  profession?: string
): Promise<void> {
  return cmdSubmitFeedback({
    rating,
    content,
    name: name?.trim() || null,
    profession: profession?.trim() || null,
  });
}
