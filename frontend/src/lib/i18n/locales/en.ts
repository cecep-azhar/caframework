export const en = {
  common: {
    appTitle: 'CAFramework',
    save: 'Save',
    cancel: 'Cancel',
    delete: 'Delete',
    edit: 'Edit',
    search: 'Search...',
    loading: 'Loading...',
    error: 'Error',
    retry: 'Retry',
    confirm: 'Confirm',
    close: 'Close',
    back: 'Back',
    done: 'Done',
    copy: 'Copy',
    copied: 'Copied',
    actions: 'Actions',
    emptyState: 'No items found',
    noResults: 'No matching results',
    required: 'Required'
  },
  nav: {
    notes: 'Notes',
    settings: 'Settings'
  },
  vault: {
    title: 'Vault Security',
    masterPassword: 'Master Password',
    confirmPassword: 'Confirm Password',
    enterPassword: 'Enter Master Password',
    setPassword: 'Set Master Password',
    unlock: 'Unlock Vault',
    locked: 'Vault Locked',
    changePassword: 'Change Master Password',
    oldPassword: 'Old Password',
    newPassword: 'New Password',
    resetVault: 'Reset Vault',
    resetWarning: 'This will delete all local data permanently.',
    unlockedSuccess: 'Vault unlocked successfully',
    invalidPassword: 'Invalid password'
  },
  profiles: {
    title: 'Profiles',
    selectProfile: 'Select Profile',
    addProfile: 'Add Profile',
    editProfile: 'Edit Profile',
    name: 'Profile Name',
    role: 'Role',
    pin: 'PIN (Optional)',
    enterPin: 'Enter 4-6 digit PIN',
    roles: {
      owner: 'Owner',
      partner: 'Partner',
      member: 'Member',
      child: 'Child'
    },
    switchProfile: 'Switch Profile',
    activeProfile: 'Active'
  },
  notes: {
    title: 'Notes',
    subtitle: 'Generic synced notes slice demonstrating CAFramework conventions',
    addNote: 'New Note',
    editNote: 'Edit Note',
    noteTitle: 'Note Title',
    noteContent: 'Write your note content here...',
    tags: 'Tags (comma separated)',
    visibility: 'Visibility',
    visibilityShared: 'Shared (Everyone in family)',
    visibilitySummary: 'Private Summary (Aggregated only)',
    visibilityPrivate: 'Private (Owner only)',
    noNotes: 'No notes yet. Create your first note!',
    deleteConfirm: 'Are you sure you want to delete this note?',
    saveSuccess: 'Note saved successfully',
    deleteSuccess: 'Note deleted successfully'
  },
  settings: {
    title: 'Settings',
    general: 'General',
    appearance: 'Appearance',
    theme: 'Theme',
    themes: {
      system: 'System',
      dark: 'Dark',
      light: 'Light'
    },
    language: 'Language',
    security: 'Security & Profiles',
    ai: 'AI Assistant',
    about: 'About',
    performance: 'Performance',
    hardwareAcceleration: 'Hardware Acceleration',
    version: 'Version'
  },
  ai: {
    title: 'AI Assistant',
    endpoint: 'API Endpoint',
    apiKey: 'API Key',
    model: 'Model Name',
    privacyMode: 'Privacy Filter',
    privacyDesc: 'Anonymise and redact sensitive names & numbers before sending to AI',
    send: 'Send',
    promptPlaceholder: 'Ask anything...',
    systemPrompt: 'System Guardrail Prompt'
  },
  pro: {
    title: 'Pro License',
    status: 'Status',
    active: 'Pro Active',
    free: 'Free Tier',
    manage: 'Manage Subscription',
    activate: 'Activate License Key'
  },
  feedback: {
    title: 'Send Feedback',
    rating: 'How is your experience?',
    content: 'Your feedback or bug report...',
    name: 'Your Name (optional)',
    submit: 'Submit Feedback',
    success: 'Thank you for your feedback!'
  }
};

export type Dictionary = typeof en;
export default en;
