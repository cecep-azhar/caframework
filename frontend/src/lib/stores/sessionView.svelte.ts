// View state for the terminal workspace: which pane layout is active, whether the remote file
// (SFTP) panel is showing, and which open host the "active" controls apply to.
//
// This lives outside the Session route on purpose. The controls that drive it (Files toggle,
// split buttons) used to sit in a second header bar owned by `/session`, stacked underneath the
// app's own header — two rows of chrome, ~84px, for one row of information. They now render in
// the single app header, which means the layout and the route have to read and write the same
// state. Module-level `$state` survives SPA navigation, so leaving `/session` and coming back
// preserves the arrangement, exactly like the open tabs themselves.

/** 1 = single, 2 = split horizontal, 3 = split vertical, 4 = grid 2x2. */
export type PaneLayout = 1 | 2 | 3 | 4;

export const PANE_LAYOUTS: PaneLayout[] = [1, 2, 3, 4];

export function isPaneLayout(value: unknown): value is PaneLayout {
  return typeof value === 'number' && PANE_LAYOUTS.includes(value as PaneLayout);
}

const view = $state({
  layout: 1 as PaneLayout,
  /** Off until the user asks for it: the file panel costs a third of the terminal's width. */
  showFiles: false,
  /** Tab id the header controls and the side panel follow; '' means "first open tab".
      Keyed by tab, not host, because the same host can be open in several tabs. */
  selectedTabId: ''
});

export function getSessionView() {
  return view;
}

export function setLayout(layout: PaneLayout) {
  view.layout = layout;
}

export function setShowFiles(show: boolean) {
  view.showFiles = show;
}

export function setSelectedTabId(tabId: string) {
  view.selectedTabId = tabId;
}

/** Called when the last tab closes so the next session doesn't inherit a stale split. */
export function resetLayout() {
  view.layout = 1;
}
