<script lang="ts">
  import { submitFeedback } from '$lib/api/feedback';
  import { showToast } from '$lib/stores/uiNotifications.svelte';
  import { errorText } from '$lib/errors';

  const STORAGE_KEY_PREFIX = 'caframework_feedback_state';
  const APP_VERSION = 'v2';

  let {
    dismissible = false,
    onDismiss,
    onSubmitted
  }: {
    dismissible?: boolean;
    onDismiss?: () => void;
    onSubmitted?: () => void;
  } = $props();

  let rating = $state(5);
  let hoverRating = $state(0);
  let name = $state('');
  let profession = $state('');
  let content = $state('');
  let isSubmitting = $state(false);
  let submitted = $state(false);
  let errorMessage = $state('');

  const ratingLabels: Record<number, string> = {
    1: 'Poor',
    2: 'Fair',
    3: 'Good',
    4: 'Very Good',
    5: 'Excellent'
  };

  function storageKey(action: 'submitted' | 'dismissed'): string {
    return `${STORAGE_KEY_PREFIX}_${APP_VERSION}_${action}`;
  }

  function handleDismiss() {
    try {
      localStorage.setItem(storageKey('dismissed'), 'true');
    } catch {
      // LocalStorage access might fail in private browsing
    }
    onDismiss?.();
  }

  async function handleSubmit(event: Event) {
    event.preventDefault();
    if (rating < 1 || rating > 5) {
      errorMessage = 'Please select a rating between 1 and 5 stars.';
      return;
    }

    isSubmitting = true;
    errorMessage = '';

    try {
      // Backend enforces non-empty content
      const effectiveContent = content.trim().length > 0
        ? content.trim()
        : `${rating} stars (no notes)`;

      await submitFeedback(rating, effectiveContent, name, profession);

      try {
        localStorage.setItem(storageKey('submitted'), 'true');
      } catch {
        // LocalStorage fallback
      }

      submitted = true;
      showToast('Thank you! Feedback submitted successfully.', 'success');
      onSubmitted?.();
    } catch (err) {
      const msg = errorText(err);
      errorMessage = msg;
      showToast(msg, 'error');
    } finally {
      isSubmitting = false;
    }
  }

  function handleReset() {
    submitted = false;
    content = '';
    rating = 5;
    errorMessage = '';
  }
</script>

<div class="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 rounded-lg p-6 space-y-5 text-neutral-900 dark:text-neutral-100 relative shadow-sm">
  {#if dismissible}
    <button
      type="button"
      onclick={handleDismiss}
      class="absolute top-4 right-4 text-neutral-400 hover:text-neutral-700 dark:hover:text-white p-1 rounded transition-colors"
      aria-label="Close"
      title="Close"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  {/if}

  <div>
    <h3 class="text-lg font-semibold text-neutral-900 dark:text-white tracking-tight">Send Feedback</h3>
    <p class="text-sm text-neutral-600 dark:text-neutral-400 mt-1">
      Help us improve CAFramework. Share your rating or report any issues you encountered.
    </p>
  </div>

  {#if submitted}
    <div class="p-5 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 rounded-lg space-y-3">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-emerald-500/10 rounded-full text-emerald-600 dark:text-emerald-400">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <div>
          <h4 class="text-sm font-semibold text-neutral-900 dark:text-white">Feedback Submitted!</h4>
          <p class="text-xs text-neutral-600 dark:text-neutral-300 mt-0.5">
            Thank you for your time and feedback. Your input helps us build a better CAFramework.
          </p>
        </div>
      </div>
      <div class="pt-2 flex gap-3">
        <button
          type="button"
          onclick={handleReset}
          class="text-xs text-sky-600 dark:text-sky-400 hover:underline font-medium transition-colors"
        >
          Send another feedback
        </button>
        {#if dismissible}
          <button
            type="button"
            onclick={handleDismiss}
            class="text-xs text-neutral-500 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 font-medium transition-colors"
          >
            Close
          </button>
        {/if}
      </div>
    </div>
  {:else}
    <form onsubmit={handleSubmit} class="space-y-4">
      <!-- Star Rating -->
      <div class="space-y-1.5">
        <label for="feedback-stars" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 uppercase tracking-wider">
          Experience Rating
        </label>
        <div id="feedback-stars" class="flex items-center gap-2">
          <div class="flex items-center gap-1" role="radiogroup" aria-label="Star rating">
            {#each [1, 2, 3, 4, 5] as star}
              <button
                type="button"
                role="radio"
                aria-checked={rating === star}
                aria-label="{star} out of 5 stars"
                onclick={() => (rating = star)}
                onmouseenter={() => (hoverRating = star)}
                onmouseleave={() => (hoverRating = 0)}
                class="p-1 text-neutral-400 dark:text-neutral-600 hover:scale-110 transition-transform focus:outline-none focus:ring-1 focus:ring-sky-500 rounded"
              >
                <svg
                  class="w-7 h-7 transition-colors {(hoverRating || rating) >= star ? 'text-amber-400 fill-amber-400' : 'text-neutral-300 dark:text-neutral-700'}"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
                  />
                </svg>
              </button>
            {/each}
          </div>
          <span class="text-xs font-medium text-neutral-600 dark:text-neutral-400 ml-2">
            {ratingLabels[hoverRating || rating] || ''}
          </span>
        </div>
      </div>

      <!-- Name & Activity Inputs -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div class="space-y-1.5">
          <label for="feedback-name" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 uppercase tracking-wider">
            Name <span class="text-neutral-500 font-normal normal-case">(optional)</span>
          </label>
          <input
            id="feedback-name"
            type="text"
            bind:value={name}
            maxlength="100"
            placeholder="Your name (optional)"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-md text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500"
          />
        </div>

        <div class="space-y-1.5">
          <label for="feedback-profession" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 uppercase tracking-wider">
            Activity / Role <span class="text-neutral-500 font-normal normal-case">(optional)</span>
          </label>
          <input
            id="feedback-profession"
            type="text"
            bind:value={profession}
            maxlength="100"
            placeholder="e.g. DevOps, Sysadmin (optional)"
            class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-md text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500"
          />
        </div>
      </div>

      <!-- Textarea (Optional) -->
      <div class="space-y-1.5">
        <label for="feedback-content" class="block text-xs font-medium text-neutral-700 dark:text-neutral-300 uppercase tracking-wider">
          Message or Notes <span class="text-neutral-500 font-normal normal-case">(optional)</span>
        </label>
        <textarea
          id="feedback-content"
          bind:value={content}
          rows="4"
          maxlength="2000"
          placeholder="Share what you like or what could be improved in CAFramework..."
          class="w-full px-3 py-2 bg-neutral-50 dark:bg-neutral-950 border border-neutral-300 dark:border-neutral-800 rounded-md text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:border-sky-500 resize-y"
        ></textarea>
        <div class="flex justify-between items-center text-xs text-neutral-500 dark:text-neutral-400">
          <span>Maximum 2000 characters</span>
          <span>{content.length} / 2000</span>
        </div>
      </div>

      {#if errorMessage}
        <div class="p-3 bg-red-50 dark:bg-red-950/40 border border-red-200 dark:border-red-800/60 rounded text-xs text-red-600 dark:text-red-400">
          {errorMessage}
        </div>
      {/if}

      <!-- Disclosure & Legal Moderation Note -->
      <div class="p-3 bg-neutral-50 dark:bg-neutral-950/80 border border-neutral-200 dark:border-neutral-800 rounded text-xs text-neutral-600 dark:text-neutral-400 flex items-start gap-2">
        <svg class="w-4 h-4 text-neutral-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p>
          Feedback is sent to the CAFramework team for review and may be featured on our website (with permission).
        </p>
      </div>

      <!-- Actions -->
      <div class="flex items-center gap-3 pt-1">
        <button
          type="submit"
          disabled={isSubmitting}
          class="px-5 py-2.5 bg-sky-600 hover:bg-sky-500 disabled:opacity-50 text-white text-sm font-medium rounded-md transition-colors flex items-center gap-2 cursor-pointer disabled:cursor-not-allowed shadow-sm"
        >
          {#if isSubmitting}
            <svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            <span>Sending...</span>
          {:else}
            <span>Send Feedback</span>
          {/if}
        </button>

        {#if dismissible}
          <button
            type="button"
            onclick={handleDismiss}
            class="px-4 py-2 text-sm text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200 transition-colors"
          >
            Maybe later
          </button>
        {/if}
      </div>
    </form>
  {/if}
</div>
