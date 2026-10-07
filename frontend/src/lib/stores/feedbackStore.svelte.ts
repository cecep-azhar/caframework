const STORAGE_PREFIX = 'caframework_feedback_v2';

let showPrompt = $state(false);
let openedManually = false;

export function isFeedbackDismissedOrSubmitted(): boolean {
  if (typeof localStorage === 'undefined') return true;
  return (
    localStorage.getItem(`${STORAGE_PREFIX}_shown`) === 'true' ||
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

  if (nextCount >= 3) {
    showPrompt = true;
    localStorage.setItem(`${STORAGE_PREFIX}_shown`, 'true');
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
      // Mark as permanently dismissed/shown on close so it never auto-pops up again
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(`${STORAGE_PREFIX}_shown`, 'true');
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
