// Open/closed state for the AI chat drawer. It lives outside the component so the floating
// button in the app header can toggle a panel that is mounted once, at the layout level, and
// therefore keeps its conversation while you move between Hosts, Session and Settings.

const state = $state({ open: false });

export function getAiChatState() {
  return state;
}

export function toggleAiChat() {
  state.open = !state.open;
}

export function closeAiChat() {
  state.open = false;
}
