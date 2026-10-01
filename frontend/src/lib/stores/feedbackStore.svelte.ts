const STORAGE_PREFIX = 'caframework_feedback_v2';

let showPrompt = $state(false);
let openedManually = false;

export function isFeedbackDismissedOrSubmitted(): boolean {
  if (typeof localStorage === 'undefined') return true;
  return (
    localStorage.getItem(`${STORAGE_PREFIX}_dismissed`) === 'true' ||
    localStorage.getItem(`${STORAGE_PREFIX}_submitted`) === 'true'
  );
}

export function recordSessionClosed() {
  if (typeof localStorage === 'undefined') return;
  if (isFeedbackDismissedOrSubmitted()) return;

  const currentCount = parseInt(localStorage.getItem(`${STORAGE_PREFIX}_closed_count`) || '0', 10);
  const nextCount = currentCount + 1;
  localStorage.setItem(`${STORAGE_PREFIX}_closed_count`, nextCount.toString());

  if (nextCount === 3) {
    showPrompt = true;
  }
}

export function getFeedbackPromptState() {
  return {
    get show() {
      return showPrompt;
    },
    /** Opened on request (profile menu > Report bug), not by the 3rd-session trigger. */
    open() {
      openedManually = true;
      showPrompt = true;
    },
    close() {
      showPrompt = false;
      // Closing a form the user asked for is not a "never ask me" for the automatic prompt.
      if (!openedManually && typeof localStorage !== 'undefined') {
        localStorage.setItem(`${STORAGE_PREFIX}_dismissed`, 'true');
      }
      openedManually = false;
    },
    markSubmitted() {
      showPrompt = false;
      openedManually = false;
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(`${STORAGE_PREFIX}_submitted`, 'true');
      }
    }
  };
}
