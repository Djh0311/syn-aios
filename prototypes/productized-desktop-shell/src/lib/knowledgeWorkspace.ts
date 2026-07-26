// N5 only persists disposable workspace chrome. Markdown, Canvas, attachment
// bytes, revisions and recovery data stay in the fixed-vault host commands.

import {
  DEFAULT_KNOWLEDGE_SPLIT_RATIO,
  MAX_KNOWLEDGE_SPLIT_RATIO,
  MIN_KNOWLEDGE_SPLIT_RATIO,
  PRIMARY_KNOWLEDGE_GROUP_ID,
  SECONDARY_KNOWLEDGE_GROUP_ID,
  createKnowledgeWorkbenchCentralState,
  createKnowledgeWorkbenchMarkdownTab,
  createKnowledgeWorkbenchSurfaceTab,
  knowledgeWorkbenchMarkdownPaths,
  reduceKnowledgeWorkbenchCentralState,
  type KnowledgeWorkbenchCentralTab,
  type KnowledgeWorkbenchCentralState,
  type KnowledgeWorkbenchCentralSurface,
  type KnowledgeWorkbenchGroupId,
  type KnowledgeWorkbenchMarkdownProjection,
  type KnowledgeWorkbenchTabGroup,
} from "./knowledgeWorkbenchLayout";

export const KNOWLEDGE_WORKSPACE_UI_PREFERENCES_KEY = "syn-native-knowledge-workspace-ui-v1";
const MAX_WORKSPACE_UI_TABS = 12;
const MAX_WORKSPACE_UI_PATH_CHARS = 512;
const KNOWLEDGE_WORKSPACE_ATTACHMENT_EXTENSIONS = new Set([
  "png", "jpg", "jpeg", "gif", "webp", "pdf", "txt", "csv",
]);

export type KnowledgeWorkspaceViewMode = "source" | "preview" | "split";

export type KnowledgeWorkspaceDraftRefreshDisposition = "replace" | "preserve" | "conflict";
export type KnowledgeWorkspaceDraftNavigationDisposition = "open" | "preserve";
export type KnowledgeWorkspaceAsyncCommitDisposition = "apply" | "preserve";

// Refresh sources (focus, manual refresh, and fixed-vault mutations) share
// this policy. A dirty editor is never a disposable cache: only clean drafts
// may be replaced, while a concurrent disk change makes the protection
// visible and blocks the next optimistic write.
export function knowledgeWorkspaceDraftRefreshDisposition({
  draftIsDirty,
  externallyChanged,
}: Readonly<{
  draftIsDirty: boolean;
  externallyChanged: boolean;
}>): KnowledgeWorkspaceDraftRefreshDisposition {
  if (!draftIsDirty) return "replace";
  return externallyChanged ? "conflict" : "preserve";
}

// Navigation has a stricter rule than explicit reload/cancel: changing the
// current document must not replace any unsaved local draft.
export function knowledgeWorkspaceDraftNavigationDisposition({
  draftIsDirty,
}: Readonly<{
  draftIsDirty: boolean;
}>): KnowledgeWorkspaceDraftNavigationDisposition {
  return draftIsDirty ? "preserve" : "open";
}

// A fixed-vault read can outlive a user edit or a later navigation. Those
// results are observational only until all three local identities still match;
// otherwise the caller must preserve the editor as it is now.
export function knowledgeWorkspaceAsyncCommitDisposition({
  requestDraftRevision,
  currentDraftRevision,
  requestGeneration,
  currentGeneration,
  requestCurrentRelativePath,
  currentRelativePath,
}: Readonly<{
  requestDraftRevision: number;
  currentDraftRevision: number;
  requestGeneration: number;
  currentGeneration: number;
  requestCurrentRelativePath: string | null;
  currentRelativePath: string | null;
}>): KnowledgeWorkspaceAsyncCommitDisposition {
  return requestDraftRevision === currentDraftRevision
    && requestGeneration === currentGeneration
    && requestCurrentRelativePath === currentRelativePath
    ? "apply"
    : "preserve";
}

export type KnowledgeWorkspaceUiPreferences = Readonly<{
  version: 1;
  tabs: ReadonlyArray<string>;
  selectedRelativePath: string | null;
  viewMode: KnowledgeWorkspaceViewMode;
}>;

export type KnowledgeWorkspaceUiStorage = Readonly<{
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
}>;

export const DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES: KnowledgeWorkspaceUiPreferences = Object.freeze({
  version: 1,
  tabs: Object.freeze([]),
  selectedRelativePath: null,
  viewMode: "split",
});

export function browserKnowledgeWorkspaceUiStorage(): KnowledgeWorkspaceUiStorage | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

export function loadKnowledgeWorkspaceUiPreferences(
  storage: KnowledgeWorkspaceUiStorage | null | undefined,
): KnowledgeWorkspaceUiPreferences {
  if (!storage) return DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES;
  try {
    const serialized = storage.getItem(KNOWLEDGE_WORKSPACE_UI_PREFERENCES_KEY);
    if (!serialized) return DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES;
    return normalizeKnowledgeWorkspaceUiPreferences(JSON.parse(serialized));
  } catch {
    return DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES;
  }
}

export function saveKnowledgeWorkspaceUiPreferences(
  storage: Pick<KnowledgeWorkspaceUiStorage, "setItem"> | null | undefined,
  value: unknown,
): KnowledgeWorkspaceUiPreferences {
  const preferences = normalizeKnowledgeWorkspaceUiPreferences(value);
  if (!storage) return preferences;
  try {
    storage.setItem(KNOWLEDGE_WORKSPACE_UI_PREFERENCES_KEY, JSON.stringify(preferences));
  } catch {
    // A full or disabled browser storage must never block the fixed-vault UI.
  }
  return preferences;
}

export function normalizeKnowledgeWorkspaceUiPreferences(value: unknown): KnowledgeWorkspaceUiPreferences {
  if (!isRecord(value) || value.version !== 1) return DEFAULT_KNOWLEDGE_WORKSPACE_UI_PREFERENCES;
  const tabs = Array.isArray(value.tabs)
    ? uniqueMarkdownPaths(value.tabs).slice(0, MAX_WORKSPACE_UI_TABS)
    : [];
  const selectedRelativePath =
    typeof value.selectedRelativePath === "string"
    && tabs.includes(value.selectedRelativePath)
    ? value.selectedRelativePath
    : null;
  const viewMode = isWorkspaceViewMode(value.viewMode) ? value.viewMode : "split";
  return {
    version: 1,
    tabs,
    selectedRelativePath,
    viewMode,
  };
}

export type KnowledgeWorkspaceCentralUiPreferences = Readonly<{
  version: 2;
  selectedRelativePath: string | null;
  centralState: KnowledgeWorkbenchCentralState;
}>;

export const DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES: KnowledgeWorkspaceCentralUiPreferences = Object.freeze({
  version: 2,
  selectedRelativePath: null,
  centralState: createKnowledgeWorkbenchCentralState(),
});

export function loadKnowledgeWorkspaceCentralUiPreferences(
  storage: KnowledgeWorkspaceUiStorage | null | undefined,
): KnowledgeWorkspaceCentralUiPreferences {
  if (!storage) return DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES;
  try {
    const serialized = storage.getItem(KNOWLEDGE_WORKSPACE_UI_PREFERENCES_KEY);
    if (!serialized) return DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES;
    return normalizeKnowledgeWorkspaceCentralUiPreferences(JSON.parse(serialized));
  } catch {
    return DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES;
  }
}

export function saveKnowledgeWorkspaceCentralUiPreferences(
  storage: Pick<KnowledgeWorkspaceUiStorage, "setItem"> | null | undefined,
  value: unknown,
): KnowledgeWorkspaceCentralUiPreferences {
  const preferences = normalizeKnowledgeWorkspaceCentralUiPreferences(value);
  if (!storage) return preferences;
  try {
    storage.setItem(KNOWLEDGE_WORKSPACE_UI_PREFERENCES_KEY, JSON.stringify(preferences));
  } catch {
    // Disposable chrome stays fail-soft when browser storage is unavailable.
  }
  return preferences;
}

export function normalizeKnowledgeWorkspaceCentralUiPreferences(
  value: unknown,
): KnowledgeWorkspaceCentralUiPreferences {
  if (!isRecord(value)) return DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES;
  if (value.version === 1) return migrateLegacyKnowledgeWorkspaceUiPreferences(value);
  if (value.version !== 2) return DEFAULT_KNOWLEDGE_WORKSPACE_CENTRAL_UI_PREFERENCES;

  const rawSelectedRelativePath = typeof value.selectedRelativePath === "string"
    && isKnowledgeWorkspaceMarkdownRelativePath(value.selectedRelativePath)
    ? value.selectedRelativePath
    : null;
  let centralState = normalizeKnowledgeWorkbenchCentralState(value.centralState);
  const availableMarkdownPaths = knowledgeWorkbenchMarkdownPaths(centralState);
  const selectedRelativePath = rawSelectedRelativePath
    && availableMarkdownPaths.includes(rawSelectedRelativePath)
    ? rawSelectedRelativePath
    : null;

  if (selectedRelativePath) {
    centralState = synchronizeNormalizedSelectedMarkdown(centralState, selectedRelativePath);
  }

  return {
    version: 2,
    selectedRelativePath,
    centralState,
  };
}

export type KnowledgeWorkspaceCentralTransitionIntent =
  | "switch-markdown"
  | "close-markdown"
  | "merge-groups";
export type KnowledgeWorkspaceCentralTransitionDisposition = "apply" | "preserve";

export function knowledgeWorkspaceCentralTransitionDisposition({
  draftIsDirty,
  needsReload,
  intent,
  wouldDiscardLastDraft = false,
}: Readonly<{
  draftIsDirty: boolean;
  needsReload: boolean;
  intent: KnowledgeWorkspaceCentralTransitionIntent;
  wouldDiscardLastDraft?: boolean;
}>): KnowledgeWorkspaceCentralTransitionDisposition {
  if (!draftIsDirty && !needsReload) return "apply";
  if (intent === "merge-groups" && !wouldDiscardLastDraft) return "apply";
  return "preserve";
}

export function isKnowledgeWorkspaceAttachmentRelativePath(relativePath: string): boolean {
  if (!isSafeWorkspaceRelativePath(relativePath) || !relativePath.startsWith("attachments/")) return false;
  const leaf = relativePath.split("/").at(-1) ?? "";
  const extension = leaf.split(".").at(-1) ?? "";
  return leaf.includes(".") && KNOWLEDGE_WORKSPACE_ATTACHMENT_EXTENSIONS.has(extension);
}

export function workspaceAttachmentMarkdownReference(relativePath: string, displayName: string): string {
  if (!isKnowledgeWorkspaceAttachmentRelativePath(relativePath)) {
    throw new Error("knowledge_workspace_invalid_attachment_reference");
  }
  const alt = displayName.replace(/[\[\]\r\n]/gu, " ").trim() || "附件";
  return `![${alt}](${relativePath})`;
}

export function workspaceAttachmentCanvasReference(relativePath: string): string {
  if (!isKnowledgeWorkspaceAttachmentRelativePath(relativePath)) {
    throw new Error("knowledge_workspace_invalid_attachment_reference");
  }
  return relativePath;
}

function uniqueMarkdownPaths(values: unknown[]): string[] {
  const paths: string[] = [];
  for (const value of values) {
    if (typeof value !== "string" || !isKnowledgeWorkspaceMarkdownRelativePath(value) || paths.includes(value)) continue;
    paths.push(value);
  }
  return paths;
}

export function isKnowledgeWorkspaceMarkdownRelativePath(relativePath: string): boolean {
  return isSafeWorkspaceRelativePath(relativePath) && relativePath.endsWith(".md") && !relativePath.endsWith("/.md");
}

function isSafeWorkspaceRelativePath(relativePath: string): boolean {
  if (
    typeof relativePath !== "string"
    || relativePath.length === 0
    || relativePath.length > MAX_WORKSPACE_UI_PATH_CHARS
    || relativePath.startsWith("/")
    || relativePath.includes("\\")
  ) {
    return false;
  }
  return relativePath.split("/").every((segment) => (
    segment.length > 0
    && segment !== "."
    && segment !== ".."
    && !segment.startsWith(".")
    && !segment.startsWith("-")
    && !segment.includes("--")
    && !/[\p{Cc}:*?\[\]{}'"=|<>]/u.test(segment)
  ));
}

function isWorkspaceViewMode(value: unknown): value is KnowledgeWorkspaceViewMode {
  return value === "source" || value === "preview" || value === "split";
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function migrateLegacyKnowledgeWorkspaceUiPreferences(
  value: Readonly<Record<string, unknown>>,
): KnowledgeWorkspaceCentralUiPreferences {
  const legacy = normalizeKnowledgeWorkspaceUiPreferences(value);
  const initial = createKnowledgeWorkbenchCentralState({
    markdownTabs: legacy.tabs,
    selectedRelativePath: legacy.selectedRelativePath,
    projection: legacy.viewMode === "preview" ? "preview" : "source",
  });
  const centralState = legacy.viewMode === "split" && legacy.selectedRelativePath
    ? reduceKnowledgeWorkbenchCentralState(initial, { type: "split-right" })
    : initial;
  return {
    version: 2,
    selectedRelativePath: legacy.selectedRelativePath,
    centralState,
  };
}

function normalizeKnowledgeWorkbenchCentralState(value: unknown): KnowledgeWorkbenchCentralState {
  if (!isRecord(value)) return createKnowledgeWorkbenchCentralState();
  const rawGroups = Array.isArray(value.groups) ? value.groups.slice(0, 2) : [];
  const acceptedMarkdownPaths = new Set<string>();
  const acceptedSurfaces = new Set<KnowledgeWorkbenchCentralSurface>();
  const groups: KnowledgeWorkbenchTabGroup[] = [];

  for (let groupIndex = 0; groupIndex < rawGroups.length; groupIndex += 1) {
    const rawGroup = rawGroups[groupIndex];
    if (!isRecord(rawGroup)) continue;
    const groupId = groupIndex === 0
      ? PRIMARY_KNOWLEDGE_GROUP_ID
      : SECONDARY_KNOWLEDGE_GROUP_ID;
    const defaultProjection: KnowledgeWorkbenchMarkdownProjection = groupIndex === 0 ? "source" : "preview";
    const tabs: KnowledgeWorkbenchCentralTab[] = [];
    const seenGroupTabIds = new Set<string>();

    if (Array.isArray(rawGroup.tabs)) {
      for (const rawTab of rawGroup.tabs) {
        if (!isRecord(rawTab)) continue;
        if (rawTab.kind === "markdown" && typeof rawTab.relativePath === "string") {
          const relativePath = rawTab.relativePath;
          if (!isKnowledgeWorkspaceMarkdownRelativePath(relativePath)) continue;
          if (!acceptedMarkdownPaths.has(relativePath) && acceptedMarkdownPaths.size >= MAX_WORKSPACE_UI_TABS) continue;
          const projection = isWorkspaceProjection(rawTab.projection)
            ? rawTab.projection
            : defaultProjection;
          const tab = createKnowledgeWorkbenchMarkdownTab(relativePath, projection);
          if (seenGroupTabIds.has(tab.id)) continue;
          seenGroupTabIds.add(tab.id);
          acceptedMarkdownPaths.add(relativePath);
          tabs.push(tab);
          continue;
        }
        if (rawTab.kind === "surface" && isKnowledgeWorkbenchCentralSurface(rawTab.surface)) {
          if (acceptedSurfaces.has(rawTab.surface)) continue;
          const tab = createKnowledgeWorkbenchSurfaceTab(rawTab.surface);
          if (seenGroupTabIds.has(tab.id)) continue;
          seenGroupTabIds.add(tab.id);
          acceptedSurfaces.add(rawTab.surface);
          tabs.push(tab);
        }
      }
    }

    const requestedActiveTabId = typeof rawGroup.activeTabId === "string"
      ? rawGroup.activeTabId
      : null;
    groups.push({
      id: groupId,
      tabs,
      activeTabId: requestedActiveTabId && tabs.some((tab) => tab.id === requestedActiveTabId)
        ? requestedActiveTabId
        : tabs[0]?.id ?? null,
    });
  }

  if (!groups.length) {
    groups.push({
      id: PRIMARY_KNOWLEDGE_GROUP_ID,
      tabs: [],
      activeTabId: null,
    });
  }
  if (groups.length === 2 && groups[1]?.tabs.length === 0) groups.pop();

  const requestedActiveGroupId = value.activeGroupId;
  const activeGroupId: KnowledgeWorkbenchGroupId = requestedActiveGroupId === SECONDARY_KNOWLEDGE_GROUP_ID
    && groups.some((group) => group.id === SECONDARY_KNOWLEDGE_GROUP_ID)
    ? SECONDARY_KNOWLEDGE_GROUP_ID
    : PRIMARY_KNOWLEDGE_GROUP_ID;
  const splitRatio = typeof value.splitRatio === "number"
    && Number.isFinite(value.splitRatio)
    && value.splitRatio >= MIN_KNOWLEDGE_SPLIT_RATIO
    && value.splitRatio <= MAX_KNOWLEDGE_SPLIT_RATIO
    ? value.splitRatio
    : DEFAULT_KNOWLEDGE_SPLIT_RATIO;

  return {
    groups,
    activeGroupId,
    splitRatio,
  };
}

function synchronizeNormalizedSelectedMarkdown(
  state: KnowledgeWorkbenchCentralState,
  selectedRelativePath: string,
): KnowledgeWorkbenchCentralState {
  const groups = state.groups.map((group) => {
    const active = group.activeTabId
      ? group.tabs.find((tab) => tab.id === group.activeTabId) ?? null
      : null;
    if (active?.kind !== "markdown") return group;
    const existing = group.tabs.find(
      (tab) => tab.kind === "markdown" && tab.relativePath === selectedRelativePath,
    );
    const selectedTab = existing ?? createKnowledgeWorkbenchMarkdownTab(
      selectedRelativePath,
      active.projection,
    );
    return {
      ...group,
      tabs: existing ? group.tabs : [...group.tabs, selectedTab],
      activeTabId: selectedTab.id,
    };
  });
  const activeMarkdownGroups = groups.filter((group) => {
    const active = group.activeTabId
      ? group.tabs.find((tab) => tab.id === group.activeTabId)
      : null;
    return active?.kind === "markdown";
  });
  if (activeMarkdownGroups.length > 1) {
    const sourceOwnerId = activeMarkdownGroups[0]?.id;
    return {
      ...state,
      groups: groups.map((group) => ({
        ...group,
        tabs: group.tabs.map((tab) => (
          tab.kind === "markdown" && tab.id === group.activeTabId
            ? { ...tab, projection: group.id === sourceOwnerId ? "source" : "preview" }
            : tab
        )),
      })),
    };
  }
  return { ...state, groups };
}

function isWorkspaceProjection(value: unknown): value is KnowledgeWorkbenchMarkdownProjection {
  return value === "source" || value === "preview";
}

function isKnowledgeWorkbenchCentralSurface(value: unknown): value is KnowledgeWorkbenchCentralSurface {
  return value === "graph" || value === "canvas" || value === "maintenance";
}
