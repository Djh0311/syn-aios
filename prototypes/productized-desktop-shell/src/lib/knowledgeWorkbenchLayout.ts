export type KnowledgeWorkbenchCentralSurface = "graph" | "canvas" | "maintenance";
export type KnowledgeWorkbenchLeftView = "files" | "search" | "sources";
export type KnowledgeWorkbenchOverlay = "quick-open" | "command" | null;

export type KnowledgeWorkbenchLayoutState = Readonly<{
  leftView: KnowledgeWorkbenchLeftView;
  leftCollapsed: boolean;
  rightCollapsed: boolean;
  overlay: KnowledgeWorkbenchOverlay;
}>;

export type KnowledgeWorkbenchLayoutAction =
  | Readonly<{ type: "select-left-view"; view: KnowledgeWorkbenchLeftView }>
  | Readonly<{ type: "collapse-left" }>
  | Readonly<{ type: "collapse-right" }>
  | Readonly<{ type: "toggle-right" }>
  | Readonly<{ type: "open-overlay"; overlay: Exclude<KnowledgeWorkbenchOverlay, null> }>
  | Readonly<{ type: "close-overlay" }>;

export const initialKnowledgeWorkbenchLayoutState: KnowledgeWorkbenchLayoutState = Object.freeze({
  leftView: "files",
  leftCollapsed: false,
  rightCollapsed: false,
  overlay: null,
});

export function reduceKnowledgeWorkbenchLayout(
  state: KnowledgeWorkbenchLayoutState,
  action: KnowledgeWorkbenchLayoutAction,
): KnowledgeWorkbenchLayoutState {
  switch (action.type) {
    case "select-left-view":
      return { ...state, leftView: action.view, leftCollapsed: false };
    case "collapse-left":
      return { ...state, leftCollapsed: true };
    case "collapse-right":
      return { ...state, rightCollapsed: true };
    case "toggle-right":
      return { ...state, rightCollapsed: !state.rightCollapsed };
    case "open-overlay":
      return { ...state, overlay: action.overlay };
    case "close-overlay":
      return { ...state, overlay: null };
  }
}

export const PRIMARY_KNOWLEDGE_GROUP_ID = "knowledge-group-primary";
export const SECONDARY_KNOWLEDGE_GROUP_ID = "knowledge-group-secondary";
export const DEFAULT_KNOWLEDGE_SPLIT_RATIO = 50;
export const MIN_KNOWLEDGE_SPLIT_RATIO = 30;
export const MAX_KNOWLEDGE_SPLIT_RATIO = 70;

export type KnowledgeWorkbenchGroupId =
  | typeof PRIMARY_KNOWLEDGE_GROUP_ID
  | typeof SECONDARY_KNOWLEDGE_GROUP_ID;
export type KnowledgeWorkbenchMarkdownProjection = "source" | "preview";
export type KnowledgeWorkbenchMarkdownTab = Readonly<{
  id: string;
  kind: "markdown";
  relativePath: string;
  projection: KnowledgeWorkbenchMarkdownProjection;
}>;
export type KnowledgeWorkbenchSurfaceTab = Readonly<{
  id: string;
  kind: "surface";
  surface: KnowledgeWorkbenchCentralSurface;
}>;
export type KnowledgeWorkbenchCentralTab =
  | KnowledgeWorkbenchMarkdownTab
  | KnowledgeWorkbenchSurfaceTab;
export type KnowledgeWorkbenchTabGroup = Readonly<{
  id: KnowledgeWorkbenchGroupId;
  tabs: ReadonlyArray<KnowledgeWorkbenchCentralTab>;
  activeTabId: string | null;
}>;
export type KnowledgeWorkbenchCentralState = Readonly<{
  groups: ReadonlyArray<KnowledgeWorkbenchTabGroup>;
  activeGroupId: KnowledgeWorkbenchGroupId;
  splitRatio: number;
}>;

export type KnowledgeWorkbenchCentralAction =
  | Readonly<{
    type: "open-markdown";
    relativePath: string;
    groupId?: KnowledgeWorkbenchGroupId;
    projection?: KnowledgeWorkbenchMarkdownProjection;
  }>
  | Readonly<{
    type: "open-surface";
    surface: KnowledgeWorkbenchCentralSurface;
    groupId?: KnowledgeWorkbenchGroupId;
  }>
  | Readonly<{ type: "activate-tab"; groupId: KnowledgeWorkbenchGroupId; tabId: string }>
  | Readonly<{ type: "close-tab"; groupId: KnowledgeWorkbenchGroupId; tabId: string }>
  | Readonly<{ type: "cycle-tab"; direction: "next" | "previous" }>
  | Readonly<{ type: "split-right" }>
  | Readonly<{ type: "merge-groups" }>
  | Readonly<{ type: "set-split-ratio"; ratio: number }>
  | Readonly<{
    type: "set-projection";
    groupId: KnowledgeWorkbenchGroupId;
    projection: KnowledgeWorkbenchMarkdownProjection;
  }>
  | Readonly<{ type: "set-active-group"; groupId: KnowledgeWorkbenchGroupId }>
  | Readonly<{ type: "prune-markdown"; availableRelativePaths: ReadonlyArray<string> }>;

export function knowledgeWorkbenchMarkdownTabId(relativePath: string): string {
  return `markdown:${relativePath}`;
}

export function knowledgeWorkbenchSurfaceTabId(surface: KnowledgeWorkbenchCentralSurface): string {
  return `surface:${surface}`;
}

export function createKnowledgeWorkbenchMarkdownTab(
  relativePath: string,
  projection: KnowledgeWorkbenchMarkdownProjection,
): KnowledgeWorkbenchMarkdownTab {
  return {
    id: knowledgeWorkbenchMarkdownTabId(relativePath),
    kind: "markdown",
    relativePath,
    projection,
  };
}

export function createKnowledgeWorkbenchSurfaceTab(
  surface: KnowledgeWorkbenchCentralSurface,
): KnowledgeWorkbenchSurfaceTab {
  return {
    id: knowledgeWorkbenchSurfaceTabId(surface),
    kind: "surface",
    surface,
  };
}

export function createKnowledgeWorkbenchCentralState({
  markdownTabs = [],
  selectedRelativePath = null,
  projection = "source",
}: Readonly<{
  markdownTabs?: ReadonlyArray<string>;
  selectedRelativePath?: string | null;
  projection?: KnowledgeWorkbenchMarkdownProjection;
}> = {}): KnowledgeWorkbenchCentralState {
  const uniqueMarkdownTabs = [...new Set(markdownTabs)];
  const tabs = uniqueMarkdownTabs.map((relativePath) => (
    createKnowledgeWorkbenchMarkdownTab(relativePath, projection)
  ));
  const activeRelativePath = selectedRelativePath && uniqueMarkdownTabs.includes(selectedRelativePath)
    ? selectedRelativePath
    : uniqueMarkdownTabs[0] ?? null;
  const primaryGroup: KnowledgeWorkbenchTabGroup = {
    id: PRIMARY_KNOWLEDGE_GROUP_ID,
    tabs,
    activeTabId: activeRelativePath ? knowledgeWorkbenchMarkdownTabId(activeRelativePath) : null,
  };
  return {
    groups: [primaryGroup],
    activeGroupId: PRIMARY_KNOWLEDGE_GROUP_ID,
    splitRatio: DEFAULT_KNOWLEDGE_SPLIT_RATIO,
  };
}

export function knowledgeWorkbenchActiveGroup(
  state: KnowledgeWorkbenchCentralState,
): KnowledgeWorkbenchTabGroup {
  return state.groups.find((group) => group.id === state.activeGroupId)
    ?? state.groups[0]
    ?? {
      id: PRIMARY_KNOWLEDGE_GROUP_ID,
      tabs: [],
      activeTabId: null,
    };
}

export function knowledgeWorkbenchActiveTab(
  state: KnowledgeWorkbenchCentralState,
  groupId: KnowledgeWorkbenchGroupId = state.activeGroupId,
): KnowledgeWorkbenchCentralTab | null {
  const group = state.groups.find((candidate) => candidate.id === groupId);
  if (!group?.activeTabId) return null;
  return group.tabs.find((tab) => tab.id === group.activeTabId) ?? null;
}

export function knowledgeWorkbenchMarkdownPaths(
  state: KnowledgeWorkbenchCentralState,
): ReadonlyArray<string> {
  const paths: string[] = [];
  for (const group of state.groups) {
    for (const tab of group.tabs) {
      if (tab.kind === "markdown" && !paths.includes(tab.relativePath)) paths.push(tab.relativePath);
    }
  }
  return paths;
}

export function knowledgeWorkbenchTabDomId(
  groupId: KnowledgeWorkbenchGroupId,
  tabId: string,
): string {
  return `knowledge-tab-${groupId}-${encodeURIComponent(tabId).replaceAll("%", "_")}`;
}

export function knowledgeWorkbenchPanelDomId(
  groupId: KnowledgeWorkbenchGroupId,
  tabId: string,
): string {
  return `knowledge-panel-${groupId}-${encodeURIComponent(tabId).replaceAll("%", "_")}`;
}

export function knowledgeWorkbenchCloseFocusTarget(
  group: Pick<KnowledgeWorkbenchTabGroup, "tabs">,
  tabId: string,
): Readonly<{ kind: "tab"; tabId: string } | { kind: "group-tools" }> {
  const closingIndex = group.tabs.findIndex((tab) => tab.id === tabId);
  if (closingIndex < 0) return { kind: "group-tools" };
  const rightNeighbor = group.tabs[closingIndex + 1];
  if (rightNeighbor) return { kind: "tab", tabId: rightNeighbor.id };
  const leftNeighbor = group.tabs[closingIndex - 1];
  return leftNeighbor ? { kind: "tab", tabId: leftNeighbor.id } : { kind: "group-tools" };
}

export function reduceKnowledgeWorkbenchCentralState(
  state: KnowledgeWorkbenchCentralState,
  action: KnowledgeWorkbenchCentralAction,
): KnowledgeWorkbenchCentralState {
  switch (action.type) {
    case "open-markdown":
      return openMarkdownTab(state, action.relativePath, action.groupId, action.projection);
    case "open-surface":
      return openSurfaceTab(state, action.surface, action.groupId);
    case "activate-tab":
      return activateCentralTab(state, action.groupId, action.tabId);
    case "close-tab":
      return closeCentralTab(state, action.groupId, action.tabId);
    case "cycle-tab":
      return cycleCentralTab(state, action.direction);
    case "split-right":
      return splitCentralGroupRight(state);
    case "merge-groups":
      return mergeCentralGroups(state);
    case "set-split-ratio":
      if (!Number.isFinite(action.ratio)) return state;
      return {
        ...state,
        splitRatio: Math.min(
          MAX_KNOWLEDGE_SPLIT_RATIO,
          Math.max(MIN_KNOWLEDGE_SPLIT_RATIO, action.ratio),
        ),
      };
    case "set-projection":
      return setCentralMarkdownProjection(state, action.groupId, action.projection);
    case "set-active-group":
      return state.groups.some((group) => group.id === action.groupId)
        ? { ...state, activeGroupId: action.groupId }
        : state;
    case "prune-markdown":
      return pruneCentralMarkdownTabs(state, new Set(action.availableRelativePaths));
  }
}

export function knowledgeWorkbenchTabShortcutTarget(input: Readonly<{
  key: string;
  metaKey: boolean;
  shiftKey: boolean;
  altKey?: boolean;
  ctrlKey?: boolean;
}>): "quick-open" | "next-tab" | "previous-tab" | null {
  if (
    input.key.toLocaleLowerCase() === "t"
    && input.metaKey
    && !input.shiftKey
    && !input.altKey
    && !input.ctrlKey
  ) {
    return "quick-open";
  }
  if (
    input.key === "Tab"
    && input.ctrlKey
    && !input.metaKey
    && !input.altKey
  ) {
    return input.shiftKey ? "previous-tab" : "next-tab";
  }
  return null;
}

export function knowledgeWorkbenchSplitRatioForKey(ratio: number, key: string): number {
  if (key === "ArrowLeft") return Math.max(MIN_KNOWLEDGE_SPLIT_RATIO, ratio - 5);
  if (key === "ArrowRight") return Math.min(MAX_KNOWLEDGE_SPLIT_RATIO, ratio + 5);
  return ratio;
}

function openMarkdownTab(
  state: KnowledgeWorkbenchCentralState,
  relativePath: string,
  requestedGroupId?: KnowledgeWorkbenchGroupId,
  requestedProjection?: KnowledgeWorkbenchMarkdownProjection,
): KnowledgeWorkbenchCentralState {
  const targetGroupId = resolveCentralGroupId(state, requestedGroupId);
  const existingGroup = state.groups.find((group) => (
    group.tabs.some((tab) => tab.kind === "markdown" && tab.relativePath === relativePath)
    && (group.id === targetGroupId || !state.groups.some((candidate) => (
      candidate.id === targetGroupId
      && candidate.tabs.some((tab) => tab.kind === "markdown" && tab.relativePath === relativePath)
    )))
  ));
  const groupId = existingGroup?.id ?? targetGroupId;
  const groups = state.groups.map((group) => {
    if (group.id !== groupId) return group;
    const existing = group.tabs.find(
      (tab): tab is KnowledgeWorkbenchMarkdownTab => tab.kind === "markdown" && tab.relativePath === relativePath,
    );
    const active = group.activeTabId
      ? group.tabs.find((tab) => tab.id === group.activeTabId)
      : null;
    const projection = existing?.projection
      ?? requestedProjection
      ?? (active?.kind === "markdown" ? active.projection : "source");
    const tab = existing ?? createKnowledgeWorkbenchMarkdownTab(relativePath, projection);
    return {
      ...group,
      tabs: existing ? group.tabs : [...group.tabs, tab],
      activeTabId: tab.id,
    };
  });
  return synchronizeVisibleMarkdownGroups(
    { ...state, groups, activeGroupId: groupId },
    groupId,
    relativePath,
  );
}

function activateCentralTab(
  state: KnowledgeWorkbenchCentralState,
  groupId: KnowledgeWorkbenchGroupId,
  tabId: string,
): KnowledgeWorkbenchCentralState {
  const group = state.groups.find((candidate) => candidate.id === groupId);
  const tab = group?.tabs.find((candidate) => candidate.id === tabId);
  if (!group || !tab) return state;
  const next = {
    ...state,
    groups: state.groups.map((candidate) => (
      candidate.id === groupId ? { ...candidate, activeTabId: tabId } : candidate
    )),
    activeGroupId: groupId,
  };
  return tab.kind === "markdown"
    ? synchronizeVisibleMarkdownGroups(next, groupId, tab.relativePath)
    : next;
}

function synchronizeVisibleMarkdownGroups(
  state: KnowledgeWorkbenchCentralState,
  sourceGroupId: KnowledgeWorkbenchGroupId,
  relativePath: string,
): KnowledgeWorkbenchCentralState {
  const groups = state.groups.map((group) => {
    if (group.id === sourceGroupId) return group;
    const active = group.activeTabId
      ? group.tabs.find((tab) => tab.id === group.activeTabId)
      : null;
    if (active?.kind !== "markdown") return group;
    const existing = group.tabs.find(
      (tab): tab is KnowledgeWorkbenchMarkdownTab => tab.kind === "markdown" && tab.relativePath === relativePath,
    );
    const mirrored = existing ?? createKnowledgeWorkbenchMarkdownTab(relativePath, active.projection);
    return {
      ...group,
      tabs: existing ? group.tabs : [...group.tabs, mirrored],
      activeTabId: mirrored.id,
    };
  });
  return { ...state, groups };
}

function openSurfaceTab(
  state: KnowledgeWorkbenchCentralState,
  surface: KnowledgeWorkbenchCentralSurface,
  requestedGroupId?: KnowledgeWorkbenchGroupId,
): KnowledgeWorkbenchCentralState {
  const tabId = knowledgeWorkbenchSurfaceTabId(surface);
  const existingGroup = state.groups.find((group) => group.tabs.some((tab) => tab.id === tabId));
  const groupId = existingGroup?.id ?? resolveCentralGroupId(state, requestedGroupId);
  return {
    ...state,
    groups: state.groups.map((group) => {
      if (group.id !== groupId) return group;
      const existing = group.tabs.some((tab) => tab.id === tabId);
      return {
        ...group,
        tabs: existing ? group.tabs : [...group.tabs, createKnowledgeWorkbenchSurfaceTab(surface)],
        activeTabId: tabId,
      };
    }),
    activeGroupId: groupId,
  };
}

function closeCentralTab(
  state: KnowledgeWorkbenchCentralState,
  groupId: KnowledgeWorkbenchGroupId,
  tabId: string,
): KnowledgeWorkbenchCentralState {
  const group = state.groups.find((candidate) => candidate.id === groupId);
  if (!group || !group.tabs.some((tab) => tab.id === tabId)) return state;
  const target = knowledgeWorkbenchCloseFocusTarget(group, tabId);
  const next = {
    ...state,
    groups: state.groups.map((candidate) => {
      if (candidate.id !== groupId) return candidate;
      const tabs = candidate.tabs.filter((tab) => tab.id !== tabId);
      const activeTabId = candidate.activeTabId === tabId
        ? target.kind === "tab" && tabs.some((tab) => tab.id === target.tabId)
          ? target.tabId
          : null
        : candidate.activeTabId;
      return { ...candidate, tabs, activeTabId };
    }),
    activeGroupId: groupId,
  };
  const nextActive = knowledgeWorkbenchActiveTab(next, groupId);
  return nextActive?.kind === "markdown"
    ? synchronizeVisibleMarkdownGroups(next, groupId, nextActive.relativePath)
    : next;
}

function cycleCentralTab(
  state: KnowledgeWorkbenchCentralState,
  direction: "next" | "previous",
): KnowledgeWorkbenchCentralState {
  const group = knowledgeWorkbenchActiveGroup(state);
  if (group.tabs.length < 2) return state;
  const currentIndex = Math.max(0, group.tabs.findIndex((tab) => tab.id === group.activeTabId));
  const offset = direction === "next" ? 1 : -1;
  const nextIndex = (currentIndex + offset + group.tabs.length) % group.tabs.length;
  const nextTab = group.tabs[nextIndex];
  return nextTab ? activateCentralTab(state, group.id, nextTab.id) : state;
}

function splitCentralGroupRight(
  state: KnowledgeWorkbenchCentralState,
): KnowledgeWorkbenchCentralState {
  if (state.groups.length !== 1) return state;
  const primary = state.groups[0];
  const active = primary?.activeTabId
    ? primary.tabs.find((tab) => tab.id === primary.activeTabId)
    : null;
  if (!primary || active?.kind !== "markdown") return state;
  const primaryTabs = primary.tabs.map((tab) => (
    tab.id === active.id ? { ...tab, projection: "source" as const } : tab
  ));
  const secondaryTab = createKnowledgeWorkbenchMarkdownTab(active.relativePath, "preview");
  return {
    groups: [
      { ...primary, tabs: primaryTabs },
      {
        id: SECONDARY_KNOWLEDGE_GROUP_ID,
        tabs: [secondaryTab],
        activeTabId: secondaryTab.id,
      },
    ],
    activeGroupId: SECONDARY_KNOWLEDGE_GROUP_ID,
    splitRatio: DEFAULT_KNOWLEDGE_SPLIT_RATIO,
  };
}

function mergeCentralGroups(
  state: KnowledgeWorkbenchCentralState,
): KnowledgeWorkbenchCentralState {
  if (state.groups.length !== 2) return state;
  const primary = state.groups.find((group) => group.id === PRIMARY_KNOWLEDGE_GROUP_ID) ?? state.groups[0];
  const secondary = state.groups.find((group) => group.id === SECONDARY_KNOWLEDGE_GROUP_ID) ?? state.groups[1];
  if (!primary || !secondary) return state;
  const activeSource = state.activeGroupId === secondary.id ? secondary : primary;
  const activeTab = activeSource.activeTabId
    ? activeSource.tabs.find((tab) => tab.id === activeSource.activeTabId) ?? null
    : null;
  const mergedTabs = [...primary.tabs];
  for (const tab of secondary.tabs) {
    const existingIndex = mergedTabs.findIndex((candidate) => candidate.id === tab.id);
    if (existingIndex < 0) {
      mergedTabs.push(tab);
    } else if (activeSource.id === secondary.id && activeTab?.id === tab.id) {
      mergedTabs[existingIndex] = tab;
    }
  }
  const activeTabId = activeTab && mergedTabs.some((tab) => tab.id === activeTab.id)
    ? activeTab.id
    : primary.activeTabId && mergedTabs.some((tab) => tab.id === primary.activeTabId)
      ? primary.activeTabId
      : mergedTabs[0]?.id ?? null;
  return {
    groups: [{
      id: PRIMARY_KNOWLEDGE_GROUP_ID,
      tabs: mergedTabs,
      activeTabId,
    }],
    activeGroupId: PRIMARY_KNOWLEDGE_GROUP_ID,
    splitRatio: state.splitRatio,
  };
}

function setCentralMarkdownProjection(
  state: KnowledgeWorkbenchCentralState,
  groupId: KnowledgeWorkbenchGroupId,
  projection: KnowledgeWorkbenchMarkdownProjection,
): KnowledgeWorkbenchCentralState {
  const target = knowledgeWorkbenchActiveTab(state, groupId);
  if (target?.kind !== "markdown") return state;
  return {
    ...state,
    groups: state.groups.map((group) => ({
      ...group,
      tabs: group.tabs.map((tab) => {
        if (tab.kind !== "markdown" || tab.id !== group.activeTabId) return tab;
        if (group.id === groupId) return { ...tab, projection };
        return projection === "source" ? { ...tab, projection: "preview" as const } : tab;
      }),
    })),
    activeGroupId: groupId,
  };
}

function pruneCentralMarkdownTabs(
  state: KnowledgeWorkbenchCentralState,
  availableRelativePaths: ReadonlySet<string>,
): KnowledgeWorkbenchCentralState {
  return {
    ...state,
    groups: state.groups.map((group) => {
      const tabs = group.tabs.filter(
        (tab) => tab.kind === "surface" || availableRelativePaths.has(tab.relativePath),
      );
      const activeTabId = group.activeTabId && tabs.some((tab) => tab.id === group.activeTabId)
        ? group.activeTabId
        : tabs[0]?.id ?? null;
      return { ...group, tabs, activeTabId };
    }),
  };
}

function resolveCentralGroupId(
  state: KnowledgeWorkbenchCentralState,
  requestedGroupId?: KnowledgeWorkbenchGroupId,
): KnowledgeWorkbenchGroupId {
  if (requestedGroupId && state.groups.some((group) => group.id === requestedGroupId)) {
    return requestedGroupId;
  }
  return state.groups.some((group) => group.id === state.activeGroupId)
    ? state.activeGroupId
    : state.groups[0]?.id ?? PRIMARY_KNOWLEDGE_GROUP_ID;
}

export function knowledgeWorkbenchOverlayDismissesForKey(key: string): boolean {
  return key === "Escape";
}

export type KnowledgeWorkbenchListAction = "previous" | "next" | "activate" | "dismiss";

export function knowledgeWorkbenchListActionForKey(key: string): KnowledgeWorkbenchListAction | null {
  switch (key) {
    case "ArrowUp":
      return "previous";
    case "ArrowDown":
      return "next";
    case "Enter":
      return "activate";
    case "Escape":
      return "dismiss";
    default:
      return null;
  }
}

// R3A freezes a clamped boundary rather than an unproven wrap-around policy:
// first/last stay current, and a non-empty list promotes an unset index to 0.
export function knowledgeWorkbenchMoveListSelection(
  currentIndex: number,
  itemCount: number,
  action: Extract<KnowledgeWorkbenchListAction, "previous" | "next">,
): number {
  if (itemCount <= 0) return -1;
  const lastIndex = itemCount - 1;
  if (currentIndex < 0) return 0;
  const normalizedCurrent = Math.min(currentIndex, lastIndex);
  return action === "previous"
    ? Math.max(0, normalizedCurrent - 1)
    : Math.min(lastIndex, normalizedCurrent + 1);
}

export function knowledgeWorkbenchShortcutTarget(input: Readonly<{
  key: string;
  metaKey: boolean;
  shiftKey: boolean;
  altKey?: boolean;
  ctrlKey?: boolean;
}>): Exclude<KnowledgeWorkbenchOverlay, null> | "left-search" | null {
  if (!input.metaKey || input.altKey || input.ctrlKey) return null;
  switch (input.key.toLocaleLowerCase()) {
    case "o":
      return input.shiftKey ? null : "quick-open";
    case "p":
      return input.shiftKey ? null : "command";
    case "f":
      return input.shiftKey ? "left-search" : null;
    default:
      return null;
  }
}

export function restoreKnowledgeWorkbenchOverlayFocus(
  trigger: Pick<HTMLElement, "focus"> | null,
  schedule: (callback: () => void) => unknown,
): void {
  schedule(() => trigger?.focus());
}
