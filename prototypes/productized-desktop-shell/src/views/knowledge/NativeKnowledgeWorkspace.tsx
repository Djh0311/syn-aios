import {
  useCallback, useEffect, useMemo, useReducer, useRef, useState,
  type CSSProperties, type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent, type ReactNode,
} from "react";
import { parseMarkdown, type MdBlock, type MdInline } from "../../lib/knowledgeVault";
import {
  browserKnowledgeWorkspaceUiStorage,
  isKnowledgeWorkspaceAttachmentRelativePath,
  knowledgeWorkspaceAsyncCommitDisposition,
  knowledgeWorkspaceCentralTransitionDisposition,
  knowledgeWorkspaceDraftNavigationDisposition,
  knowledgeWorkspaceDraftRefreshDisposition,
  loadKnowledgeWorkspaceCentralUiPreferences,
  saveKnowledgeWorkspaceCentralUiPreferences,
} from "../../lib/knowledgeWorkspace";
import {
  knowledgeWorkspace,
  type KnowledgeWorkspaceClient,
  type KnowledgeWorkspaceEntry,
  type KnowledgeWorkspaceMarkdownDocument,
  type KnowledgeWorkspaceSearchResult,
  type KnowledgeWorkspaceSnapshot,
} from "../../lib/tauri";
import {
  knowledgeOpenRelayCanAcknowledgeOpened,
  sameKnowledgeOpenRelayIntent,
  type KnowledgeOpenRelayIntent,
  type KnowledgeOpenRelayOutcome,
} from "../../lib/knowledgeOpenRelay";
import {
  PRIMARY_KNOWLEDGE_GROUP_ID, SECONDARY_KNOWLEDGE_GROUP_ID,
  initialKnowledgeWorkbenchLayoutState, knowledgeWorkbenchActiveGroup,
  knowledgeWorkbenchActiveTab, knowledgeWorkbenchCloseFocusTarget,
  knowledgeWorkbenchListActionForKey, knowledgeWorkbenchMarkdownPaths,
  knowledgeWorkbenchMoveListSelection, knowledgeWorkbenchOverlayDismissesForKey,
  knowledgeWorkbenchPanelDomId, knowledgeWorkbenchShortcutTarget,
  knowledgeWorkbenchSplitRatioForKey, knowledgeWorkbenchTabDomId,
  knowledgeWorkbenchTabShortcutTarget, restoreKnowledgeWorkbenchOverlayFocus,
  reduceKnowledgeWorkbenchCentralState, reduceKnowledgeWorkbenchLayout,
  type KnowledgeWorkbenchOverlay, type KnowledgeWorkbenchCentralState,
  type KnowledgeWorkbenchCentralSurface, type KnowledgeWorkbenchCentralTab,
  type KnowledgeWorkbenchGroupId, type KnowledgeWorkbenchMarkdownTab,
  type KnowledgeWorkbenchTabGroup,
} from "../../lib/knowledgeWorkbenchLayout";
import { KnowledgeGraphView } from "./KnowledgeGraphView";
import { KnowledgeCanvasView } from "./KnowledgeCanvasView";
import { KnowledgeWorkspaceMaintenancePanel } from "./KnowledgeWorkspaceMaintenancePanel";
import { KnowledgeWorkbenchShell } from "./KnowledgeWorkbenchShell";
import { KnowledgeActivityRail } from "./KnowledgeActivityRail";
import { KnowledgeContextSidebar } from "./KnowledgeContextSidebar";

type WorkspaceLoadState = "loading" | "ready" | "unavailable";
type WorkspaceCommandKind = "markdown" | "directory";
type WorkspaceSearchStatus = "idle" | "loading" | "results" | "empty" | "error";
type WorkspaceCommandStage = "list" | "create";
type WorkspaceEditorAsyncRequest = Readonly<{
  draftRevision: number;
  generation: number;
  currentRelativePath: string | null;
}>;

const workspaceCommandDefinitions: ReadonlyArray<Readonly<{
  kind: WorkspaceCommandKind;
  label: string;
  detail: string;
  keywords: string;
}>> = [
  {
    kind: "markdown",
    label: "新建 Markdown",
    detail: "进入固定 vault 内 Markdown 的受限相对路径表单。",
    keywords: "markdown note 新建 笔记",
  },
  {
    kind: "directory",
    label: "新建目录",
    detail: "进入固定 vault 内目录的受限相对路径表单。",
    keywords: "directory folder 新建 目录",
  },
];

export function NativeKnowledgeWorkspace({
  client = knowledgeWorkspace,
  knowledgeOpenIntent = null,
  onKnowledgeOpenIntentOutcome,
  requestedMarkdownRelativePath = null,
  requestedMarkdownRequestId = 0,
  requestedAttachmentReference = null,
  requestedAttachmentRequestId = 0,
  refreshRequestId = 0,
  onWorkspaceMutation,
  sourceSidebar = null,
  sourceContext = null,
  statusContent = null,
}: {
  client?: KnowledgeWorkspaceClient;
  knowledgeOpenIntent?: KnowledgeOpenRelayIntent | null;
  onKnowledgeOpenIntentOutcome?: (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome) => Promise<boolean>;
  requestedMarkdownRelativePath?: string | null;
  requestedMarkdownRequestId?: number;
  requestedAttachmentReference?: string | null;
  requestedAttachmentRequestId?: number;
  refreshRequestId?: number;
  onWorkspaceMutation?: () => void;
  sourceSidebar?: ReactNode;
  sourceContext?: ((selectedRelativePath: string | null) => ReactNode) | null;
  statusContent?: ReactNode;
}) {
  // The offline runner renders this component through React SSR. Keep that path
  // meaningful and hook-free: it must never invoke Tauri or touch a vault.
  if (typeof window === "undefined") {
    return <NativeKnowledgeWorkspaceStaticShell sourceSidebar={sourceSidebar} sourceContext={sourceContext} statusContent={statusContent} />;
  }
  return (
    <NativeKnowledgeWorkspaceInner
      client={client}
      knowledgeOpenIntent={knowledgeOpenIntent}
      onKnowledgeOpenIntentOutcome={onKnowledgeOpenIntentOutcome}
      requestedMarkdownRelativePath={requestedMarkdownRelativePath}
      requestedMarkdownRequestId={requestedMarkdownRequestId}
      requestedAttachmentReference={requestedAttachmentReference}
      requestedAttachmentRequestId={requestedAttachmentRequestId}
      refreshRequestId={refreshRequestId}
      onWorkspaceMutation={onWorkspaceMutation}
      sourceSidebar={sourceSidebar}
      sourceContext={sourceContext}
      statusContent={statusContent}
    />
  );
}

function NativeKnowledgeWorkspaceStaticShell({
  sourceSidebar,
  sourceContext,
  statusContent,
}: {
  sourceSidebar: ReactNode;
  sourceContext: ((selectedRelativePath: string | null) => ReactNode) | null;
  statusContent: ReactNode;
}) {
  return (
    <KnowledgeWorkbenchShell
      activityRail={
        // SSR 静态投影：与实时壳同一个 ribbon 呈现，但保持 hook-free、不接 dispatch。
        <KnowledgeActivityRail
          items={[
            { icon: "files", label: "文件", active: true, onSelect: () => {} },
            { icon: "search", label: "搜索", active: false, onSelect: () => {} },
            { icon: "graph", label: "关系图", active: false, onSelect: () => {} },
            { icon: "canvas", label: "Canvas", active: false, onSelect: () => {} },
            { icon: "command", label: "Syn 命令", onSelect: () => {} },
            { icon: "maintenance", label: "设置与维护", onSelect: () => {} },
          ]}
        />
      }
      leftSidebar={
        <>
          <section className="native-workspace-spine" aria-label="文件与目录">
            <p className="eyebrow">文件与目录</p>
            <strong>固定 Syn vault</strong>
            <span>桌面壳打开后读取已验证的目录树。</span>
          </section>
          {sourceSidebar}
        </>
      }
      centralWorkspace={
        <section className="knowledge-workbench-central" aria-label="中央标签组工作区">
          <div className="knowledge-workbench-groups" data-knowledge-group-count="1">
            <section
              className="knowledge-workbench-group is-active-group"
              aria-label="主标签组"
              data-knowledge-tab-group={PRIMARY_KNOWLEDGE_GROUP_ID}
              data-active-group="true"
            >
              <header className="knowledge-workbench-group__header">
                <div
                  className="knowledge-workbench-group__tabs"
                  role="tablist"
                  aria-label="主标签组"
                  data-knowledge-group-tablist
                >
                  <span className="native-workspace-tabs-empty">从目录或快速打开选择一条 Markdown 笔记</span>
                </div>
                <div className="knowledge-workbench-group__tools" aria-label="主标签组工具">
                  <span className="knowledge-workbench-group__current">当前组</span>
                  <button type="button" aria-label="在主标签组中快速打开">+</button>
                </div>
              </header>
              <div className="knowledge-workbench-group__empty">
                <p className="native-workspace-static-title">Syn 原生工作区（Syn 原生知识工作区）</p>
                <p>打开一条 Markdown 笔记后，可在源码与安全渲染预览之间切换。Markdown 与 JSON Canvas 均在 Syn 内原生接入；无需安装 Obsidian。</p>
                <span>真实标签页 / 向右分栏阅读 / Markdown 源码 / 渲染预览</span>
              </div>
            </section>
          </div>
        </section>
      }
      rightSidebar={
        <>
          <section className="native-workspace-margin" aria-label="反链与属性">
            <p className="eyebrow">反链与属性</p>
            <span>标签、属性和反向引用会从当前笔记投影。</span>
          </section>
          {sourceContext?.(null)}
        </>
      }
      statusBar={
        <>
          <span>快速打开</span>
          <span>Syn 命令</span>
          <span>文件发生外部变更时会保留本地草稿，先重新读取，绝不静默覆盖。</span>
          {statusContent}
        </>
      }
    />
  );
}

function NativeKnowledgeWorkspaceInner({
  client,
  knowledgeOpenIntent,
  onKnowledgeOpenIntentOutcome,
  requestedMarkdownRelativePath,
  requestedMarkdownRequestId,
  requestedAttachmentReference,
  requestedAttachmentRequestId,
  refreshRequestId,
  onWorkspaceMutation,
  sourceSidebar,
  sourceContext,
  statusContent,
}: {
  client: KnowledgeWorkspaceClient;
  knowledgeOpenIntent: KnowledgeOpenRelayIntent | null;
  onKnowledgeOpenIntentOutcome?: (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome) => Promise<boolean>;
  requestedMarkdownRelativePath: string | null;
  requestedMarkdownRequestId: number;
  requestedAttachmentReference: string | null;
  requestedAttachmentRequestId: number;
  refreshRequestId: number;
  onWorkspaceMutation?: () => void;
  sourceSidebar: ReactNode;
  sourceContext: ((selectedRelativePath: string | null) => ReactNode) | null;
  statusContent: ReactNode;
}) {
  const [initialPreferences] = useState(() => loadKnowledgeWorkspaceCentralUiPreferences(browserKnowledgeWorkspaceUiStorage()));
  const [preferencesHydrated, setPreferencesHydrated] = useState(false);
  const restoredPreferences = useRef(false);
  const lastRefreshRequestId = useRef(refreshRequestId);
  const lastAttachmentRequestId = useRef(0);
  const lastRelayIntentRef = useRef<KnowledgeOpenRelayIntent | null>(null);
  const relayAckAttemptedIntentRef = useRef<KnowledgeOpenRelayIntent | null>(null);
  const centralTabButtonRefs = useRef(new Map<string, HTMLButtonElement>());
  const groupToolButtonRefs = useRef(new Map<KnowledgeWorkbenchGroupId, HTMLButtonElement>());
  const centralGroupsRef = useRef<HTMLDivElement | null>(null);
  const quickOpenTargetGroupRef = useRef<KnowledgeWorkbenchGroupId>(PRIMARY_KNOWLEDGE_GROUP_ID);
  const draftRevisionRef = useRef(0);
  const editorGenerationRef = useRef(0);
  const selectedRelativePathRef = useRef<string | null>(null);
  const [loadState, setLoadState] = useState<WorkspaceLoadState>("loading");
  const [snapshot, setSnapshot] = useState<KnowledgeWorkspaceSnapshot | null>(null);
  const [selected, setSelected] = useState<KnowledgeWorkspaceMarkdownDocument | null>(null);
  const [centralState, setCentralState] = useState<KnowledgeWorkbenchCentralState>(
    () => initialPreferences.centralState,
  );
  const centralStateRef = useRef<KnowledgeWorkbenchCentralState>(initialPreferences.centralState);
  const [draft, setDraft] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ReadonlyArray<KnowledgeWorkspaceSearchResult>>([]);
  const [searchStatus, setSearchStatus] = useState<WorkspaceSearchStatus>("idle");
  const [searchCurrentIndex, setSearchCurrentIndex] = useState(-1);
  const [notice, setNotice] = useState<string | null>(null);
  const [needsReload, setNeedsReload] = useState(false);
  const [pendingWikilink, setPendingWikilink] = useState<string | null>(null);
  const [commandKind, setCommandKind] = useState<WorkspaceCommandKind>("markdown");
  const [commandPath, setCommandPath] = useState("");
  const [commandStage, setCommandStage] = useState<WorkspaceCommandStage>("list");
  const [commandQuery, setCommandQuery] = useState("");
  const [commandCurrentIndex, setCommandCurrentIndex] = useState(0);
  const [busy, setBusy] = useState<"refresh" | "save" | "search" | "create" | null>(null);
  const [relayIntentAwaitingFocus, setRelayIntentAwaitingFocus] = useState<KnowledgeOpenRelayIntent | null>(null);
  const [workbenchLayout, dispatchWorkbenchLayout] = useReducer(
    reduceKnowledgeWorkbenchLayout,
    initialKnowledgeWorkbenchLayoutState,
  );
  const {
    leftView,
    leftCollapsed,
    rightCollapsed,
    overlay: activeOverlay,
  } = workbenchLayout;
  const [workbenchRefreshRequestId, setWorkbenchRefreshRequestId] = useState(0);
  const searchRequestIdRef = useRef(0);
  const searchQueryRef = useRef("");
  const overlayTriggerRef = useRef<HTMLElement | null>(null);
  const overlayDialogRef = useRef<HTMLDivElement | null>(null);
  const leftSearchInputRef = useRef<HTMLInputElement | null>(null);
  const quickOpenInputRef = useRef<HTMLInputElement | null>(null);
  const commandFilterInputRef = useRef<HTMLInputElement | null>(null);
  const commandPathInputRef = useRef<HTMLInputElement | null>(null);

  const activeCentralGroup = useMemo(() => knowledgeWorkbenchActiveGroup(centralState), [centralState]);
  const activeCentralTab = useMemo(() => knowledgeWorkbenchActiveTab(centralState), [centralState]);
  const saveOwnerGroupId = useMemo(
    () => centralState.groups.find((group) => (
      knowledgeWorkbenchActiveTab(centralState, group.id)?.kind === "markdown"
    ))?.id ?? null,
    [centralState],
  );

  const updateCentralState = useCallback((
    updater: KnowledgeWorkbenchCentralState | ((current: KnowledgeWorkbenchCentralState) => KnowledgeWorkbenchCentralState),
  ): KnowledgeWorkbenchCentralState => {
    const next = typeof updater === "function" ? updater(centralStateRef.current) : updater;
    centralStateRef.current = next;
    setCentralState(next);
    return next;
  }, []);

  const focusCentralTab = useCallback((groupId: KnowledgeWorkbenchGroupId, tabId: string) => {
    window.setTimeout(() => centralTabButtonRefs.current
      .get(centralTabRefKey(groupId, tabId))?.focus({ preventScroll: true }), 0);
  }, []);

  const focusCentralGroupTools = useCallback((groupId: KnowledgeWorkbenchGroupId) => {
    window.setTimeout(() => groupToolButtonRefs.current.get(groupId)?.focus({ preventScroll: true }), 0);
  }, []);

  const requestWorkbenchMutation = useCallback(() => {
    setWorkbenchRefreshRequestId((current) => current + 1);
    onWorkspaceMutation?.();
  }, [onWorkspaceMutation]);

  const replaceDraft = useCallback((nextDraft: string) => {
    draftRevisionRef.current += 1;
    setDraft(nextDraft);
  }, []);
  const updateDraft = useCallback((updater: (current: string) => string) => {
    draftRevisionRef.current += 1;
    setDraft(updater);
  }, []);
  const replaceSelectedDocument = useCallback((document: KnowledgeWorkspaceMarkdownDocument) => {
    selectedRelativePathRef.current = document.relative_path;
    setSelected(document);
  }, []);
  const clearSelectedDocument = useCallback(() => {
    editorGenerationRef.current += 1;
    selectedRelativePathRef.current = null;
    setSelected(null);
  }, []);
  const beginEditorAsyncRequest = useCallback((): WorkspaceEditorAsyncRequest => {
    const generation = editorGenerationRef.current + 1;
    editorGenerationRef.current = generation;
    return {
      draftRevision: draftRevisionRef.current,
      generation,
      currentRelativePath: selectedRelativePathRef.current,
    };
  }, []);
  const canCommitEditorAsyncRequest = useCallback((request: WorkspaceEditorAsyncRequest): boolean => (
    knowledgeWorkspaceAsyncCommitDisposition({
      requestDraftRevision: request.draftRevision,
      currentDraftRevision: draftRevisionRef.current,
      requestGeneration: request.generation,
      currentGeneration: editorGenerationRef.current,
      requestCurrentRelativePath: request.currentRelativePath,
      currentRelativePath: selectedRelativePathRef.current,
    }) === "apply"
  ), []);

  const entries = snapshot?.entries ?? [];
  const sortedEntries = useMemo(
    () => [...entries].sort((left, right) => compareWorkspaceEntries(left, right)),
    [entries],
  );
  const filteredCommands = useMemo(() => {
    const normalizedQuery = commandQuery.trim().toLocaleLowerCase();
    if (!normalizedQuery) return workspaceCommandDefinitions;
    return workspaceCommandDefinitions.filter((command) => (
      `${command.label} ${command.detail} ${command.keywords}`.toLocaleLowerCase().includes(normalizedQuery)
    ));
  }, [commandQuery]);
  const attachmentReferences = useMemo(
    () => workspaceAttachmentReferenceStatus(draft, entries),
    [draft, entries],
  );

  useEffect(() => {
    setCommandCurrentIndex((current) => {
      if (!filteredCommands.length) return -1;
      return current < 0 ? 0 : Math.min(current, filteredCommands.length - 1);
    });
  }, [filteredCommands.length]);

  const reloadSnapshot = useCallback(async (): Promise<KnowledgeWorkspaceSnapshot | null> => {
    try {
      const nextSnapshot = await client.snapshot();
      setSnapshot(nextSnapshot);
      setLoadState("ready");
      return nextSnapshot;
    } catch {
      setLoadState("unavailable");
      setNotice("知识工作区暂时读不到；当前草稿没有被改写。");
      return null;
    }
  }, [client]);

  useEffect(() => {
    void reloadSnapshot();
  }, [reloadSnapshot]);

  const openDocument = useCallback(
    async (relativePath: string, { discardDraft = false }: Readonly<{ discardDraft?: boolean }> = {}): Promise<boolean> => {
      const draftIsProtected = selected !== null && (needsReload || draft !== selected.body);
      if (
        !discardDraft
        && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve"
      ) {
        setNotice("当前 Markdown 草稿尚未保存；请先保存或取消编辑后再切换笔记。");
        return false;
      }
      const request = beginEditorAsyncRequest();
      try {
        const document = await client.readMarkdown(relativePath);
        if (document.relative_path !== relativePath) {
          setNotice("读取结果没有精确对应请求的 Markdown；已拒绝切换，也没有改写当前内容。");
          return false;
        }
        if (!canCommitEditorAsyncRequest(request)) {
          setNotice("读取期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
          return false;
        }
        replaceSelectedDocument(document);
        replaceDraft(document.body);
        setNeedsReload(false);
        setPendingWikilink(null);
        setNotice(null);
        return true;
      } catch {
        setNotice("这条 Markdown 笔记暂时读不到；没有改写现有内容。");
        return false;
      }
    },
    [beginEditorAsyncRequest, canCommitEditorAsyncRequest, client, draft, needsReload, replaceDraft, replaceSelectedDocument, selected],
  );

  const openMarkdownInWorkbench = useCallback(async (
    relativePath: string,
    groupId?: KnowledgeWorkbenchGroupId,
    projection?: "source" | "preview",
  ): Promise<boolean> => {
    const alreadySelected = selectedRelativePathRef.current === relativePath;
    if (!alreadySelected && !await openDocument(relativePath)) return false;
    const next = updateCentralState((current) => reduceKnowledgeWorkbenchCentralState(current, {
      type: "open-markdown",
      relativePath,
      groupId,
      projection,
    }));
    const focusedGroup = knowledgeWorkbenchActiveGroup(next);
    const focusedTab = knowledgeWorkbenchActiveTab(next);
    if (focusedTab) focusCentralTab(focusedGroup.id, focusedTab.id);
    return true;
  }, [focusCentralTab, openDocument, updateCentralState]);

  useEffect(() => {
    if (!snapshot) return;
    const availableMarkdownPaths = snapshot.entries
      .filter((entry) => entry.kind === "markdown")
      .map((entry) => entry.relative_path);
    updateCentralState((current) => reduceKnowledgeWorkbenchCentralState(current, {
      type: "prune-markdown",
      availableRelativePaths: availableMarkdownPaths,
    }));
  }, [snapshot, updateCentralState]);

  useEffect(() => {
    if (!snapshot || restoredPreferences.current) return;
    restoredPreferences.current = true;
    const savedSelected = initialPreferences.selectedRelativePath;
    const canRestore = savedSelected && snapshot.entries.some(
      (entry) => entry.kind === "markdown" && entry.relative_path === savedSelected,
    );
    if (!canRestore || !savedSelected) {
      setPreferencesHydrated(true);
      return;
    }
    void openDocument(savedSelected).finally(() => setPreferencesHydrated(true));
  }, [initialPreferences.selectedRelativePath, openDocument, snapshot]);

  useEffect(() => {
    if (!preferencesHydrated) return;
    saveKnowledgeWorkspaceCentralUiPreferences(browserKnowledgeWorkspaceUiStorage(), {
      version: 2,
      selectedRelativePath: selected?.relative_path ?? null,
      centralState,
    });
  }, [centralState, preferencesHydrated, selected?.relative_path]);

  // N3 relationship nodes return only a validated relative_path. Reuse the
  // same fixed-client read path as the tree and quick-open controls; no route,
  // external launcher, or generic path surface is introduced here.
  useEffect(() => {
    if (!requestedMarkdownRelativePath) return;
    void openMarkdownInWorkbench(requestedMarkdownRelativePath, undefined, "source");
  }, [openMarkdownInWorkbench, requestedMarkdownRelativePath, requestedMarkdownRequestId]);

  const acknowledgeRelayIntentOutcome = useCallback(
    async (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome): Promise<boolean> => {
      if (!onKnowledgeOpenIntentOutcome) {
        setNotice("知识打开请求无法由当前界面确认；没有把它显示为已打开。");
        return false;
      }
      try {
        const acknowledged = await onKnowledgeOpenIntentOutcome(intent, outcome);
        if (!acknowledged) {
          setNotice("知识打开尚未获 Syn 主进程确认；没有把它显示为已打开。");
        }
        return acknowledged;
      } catch {
        setNotice("知识打开确认没有完成；没有把它显示为已打开。");
        return false;
      }
    },
    [onKnowledgeOpenIntentOutcome],
  );

  // A host relay intent follows the same fixed-client typed read as every
  // native tree/graph action.  Dirty or stale drafts make `openDocument` fail
  // closed, and the host receives an exact rejected acknowledgement instead of
  // a synthetic success.
  useEffect(() => {
    if (!knowledgeOpenIntent || sameKnowledgeOpenRelayIntent(lastRelayIntentRef.current, knowledgeOpenIntent)) return;
    lastRelayIntentRef.current = knowledgeOpenIntent;
    setRelayIntentAwaitingFocus(null);
    void (async () => {
      const opened = await openMarkdownInWorkbench(knowledgeOpenIntent.relativePath, undefined, "source");
      if (!sameKnowledgeOpenRelayIntent(lastRelayIntentRef.current, knowledgeOpenIntent)) return;
      if (!opened) {
        relayAckAttemptedIntentRef.current = knowledgeOpenIntent;
        await acknowledgeRelayIntentOutcome(knowledgeOpenIntent, "rejected");
        return;
      }
      setRelayIntentAwaitingFocus(knowledgeOpenIntent);
    })();
  }, [acknowledgeRelayIntentOutcome, knowledgeOpenIntent, openMarkdownInWorkbench]);

  // This effect runs only after React has committed the selected document and
  // tab.  It focuses that exact tab and verifies document.activeElement before
  // the sole `opened` acknowledgement is permitted.
  useEffect(() => {
    const intent = relayIntentAwaitingFocus;
    if (!intent || sameKnowledgeOpenRelayIntent(relayAckAttemptedIntentRef.current, intent)) return;
    relayAckAttemptedIntentRef.current = intent;
    void (async () => {
      const selectedRelativePath = selected?.relative_path ?? null;
      const draftIsDirty = selected !== null && (needsReload || draft !== selected.body);
      const relayGroup = [
        knowledgeWorkbenchActiveGroup(centralState),
        ...centralState.groups,
      ].find((group, index, groups) => (
        groups.findIndex((candidate) => candidate.id === group.id) === index
        && group.tabs.some((candidate) => (
          candidate.kind === "markdown"
          && candidate.relativePath === intent.relativePath
          && group.activeTabId === candidate.id
        ))
      ));
      const relayTab = relayGroup?.tabs.find((candidate) => (
        candidate.kind === "markdown" && candidate.relativePath === intent.relativePath
      ));
      const tab = relayGroup && relayTab
        ? centralTabButtonRefs.current.get(centralTabRefKey(relayGroup.id, relayTab.id)) ?? null
        : null;
      let focusedRelativePath: string | null = null;
      if (!draftIsDirty && tab && selectedRelativePath === intent.relativePath) {
        tab.focus({ preventScroll: true });
        if (document.activeElement === tab) focusedRelativePath = intent.relativePath;
      }
      const outcome = knowledgeOpenRelayCanAcknowledgeOpened(intent, {
        typedReadCompleted: !draftIsDirty && selectedRelativePath === intent.relativePath,
        selectedRelativePath,
        focusedRelativePath,
      })
        ? "opened"
        : "rejected";
      if (outcome === "rejected") {
        setNotice("知识打开没有完成目标笔记的选中和聚焦；没有把它显示为已打开。");
      }
      const acknowledged = await acknowledgeRelayIntentOutcome(intent, outcome);
      if (acknowledged) {
        setRelayIntentAwaitingFocus((current) => (sameKnowledgeOpenRelayIntent(current, intent) ? null : current));
      }
    })();
  }, [acknowledgeRelayIntentOutcome, centralState, draft, needsReload, relayIntentAwaitingFocus, selected]);

  const insertAttachmentReference = useCallback((reference: string) => {
    if (!selected) {
      setNotice("先打开一条 Markdown 笔记，再插入固定 vault 内的附件引用。");
      return;
    }
    const next = updateCentralState((current) => reduceKnowledgeWorkbenchCentralState(current, {
      type: "open-markdown",
      relativePath: selected.relative_path,
      projection: "source",
    }));
    const targetGroup = knowledgeWorkbenchActiveGroup(next);
    const targetTab = knowledgeWorkbenchActiveTab(next);
    if (targetTab) focusCentralTab(targetGroup.id, targetTab.id);
    updateDraft((current) => `${current}${current.endsWith("\n") || !current ? "" : "\n"}${reference}\n`);
    setNotice("已把受限附件相对引用加入本地 Markdown 草稿；点击保存才会写入固定 vault。");
  }, [focusCentralTab, selected, updateCentralState, updateDraft]);

  useEffect(() => {
    if (
      !requestedAttachmentReference
      || requestedAttachmentRequestId === 0
      || requestedAttachmentRequestId === lastAttachmentRequestId.current
    ) return;
    lastAttachmentRequestId.current = requestedAttachmentRequestId;
    insertAttachmentReference(requestedAttachmentReference);
  }, [insertAttachmentReference, requestedAttachmentReference, requestedAttachmentRequestId]);

  const refreshWorkspace = useCallback(async () => {
    const request = beginEditorAsyncRequest();
    setBusy("refresh");
    const nextSnapshot = await reloadSnapshot();
    if (!canCommitEditorAsyncRequest(request)) {
      setNotice("刷新期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
      setBusy(null);
      return;
    }
    if (!nextSnapshot || !selected) {
      setBusy(null);
      return;
    }
    try {
      const current = await client.readMarkdown(selected.relative_path);
      if (!canCommitEditorAsyncRequest(request)) {
        setNotice("刷新期间本地草稿或当前笔记已变化；已保留，不会回填迟到内容。");
        return;
      }
      const externallyChanged =
        current.mtime_ms !== selected.mtime_ms || current.content_hash !== selected.content_hash;
      const disposition = knowledgeWorkspaceDraftRefreshDisposition({
        draftIsDirty: needsReload || draft !== selected.body,
        externallyChanged,
      });
      if (disposition === "conflict") {
        setNeedsReload(true);
        setNotice("文件已在另一窗口或外部程序修改。已保留本地草稿；请重新读取后再保存。");
      } else if (disposition === "preserve") {
        setNotice("已刷新已验证的目录树；当前 Markdown 草稿尚未保存，已保留。");
      } else {
        replaceSelectedDocument(current);
        replaceDraft(current.body);
        setNeedsReload(false);
        setNotice("已刷新已验证的目录树和当前笔记。");
      }
    } catch {
      setNotice("目录树已刷新，但当前笔记暂时读不到；没有覆盖本地草稿。");
    } finally {
      setBusy(null);
    }
  }, [beginEditorAsyncRequest, canCommitEditorAsyncRequest, client, draft, needsReload, reloadSnapshot, replaceDraft, replaceSelectedDocument, selected]);

  useEffect(() => {
    if (refreshRequestId === 0 || refreshRequestId === lastRefreshRequestId.current) return;
    lastRefreshRequestId.current = refreshRequestId;
    void refreshWorkspace();
  }, [refreshRequestId, refreshWorkspace]);

  useEffect(() => {
    const refreshAfterFocusOrVaultSave = () => void refreshWorkspace();
    window.addEventListener("focus", refreshAfterFocusOrVaultSave);
    window.addEventListener("syn-knowledge-vault-saved", refreshAfterFocusOrVaultSave);
    return () => {
      window.removeEventListener("focus", refreshAfterFocusOrVaultSave);
      window.removeEventListener("syn-knowledge-vault-saved", refreshAfterFocusOrVaultSave);
    };
  }, [refreshWorkspace]);

  const refreshWorkspaceManually = useCallback(() => {
    void refreshWorkspace();
    requestWorkbenchMutation();
  }, [refreshWorkspace, requestWorkbenchMutation]);

  const saveDraft = useCallback(async () => {
    if (!selected || needsReload) return;
    const request = beginEditorAsyncRequest();
    setBusy("save");
    try {
      await client.writeMarkdown(selected.relative_path, draft, selected.mtime_ms, selected.content_hash);
      if (!canCommitEditorAsyncRequest(request)) {
        if (
          request.currentRelativePath === selectedRelativePathRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setNeedsReload(true);
        }
        setNotice("保存已提交，但期间本地草稿或当前笔记已变化；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      const updated = await client.readMarkdown(selected.relative_path);
      if (!canCommitEditorAsyncRequest(request)) {
        if (
          request.currentRelativePath === selectedRelativePathRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setNeedsReload(true);
        }
        setNotice("保存后的读取结果已过期；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      replaceSelectedDocument(updated);
      replaceDraft(updated.body);
      setNeedsReload(false);
      setNotice("已保存并重新读取当前 Markdown 笔记。");
      void reloadSnapshot();
      requestWorkbenchMutation();
    } catch (error) {
      if (isWorkspaceWriteConflict(error)) {
        setNeedsReload(true);
        setNotice("文件已在另一窗口或外部程序修改。已保留本地草稿；请重新读取后再保存。");
      } else {
        setNotice("保存没有完成；当前草稿仍保留，未静默覆盖文件。");
      }
    } finally {
      setBusy(null);
    }
  }, [beginEditorAsyncRequest, canCommitEditorAsyncRequest, client, draft, needsReload, reloadSnapshot, replaceDraft, replaceSelectedDocument, requestWorkbenchMutation, selected]);

  const reloadSelected = useCallback(() => {
    if (!selected) return;
    void openDocument(selected.relative_path, { discardDraft: true }).then((reloaded) => {
      if (reloaded) setNotice("已重新读取当前笔记；此前本地草稿未被自动写回。");
    });
  }, [openDocument, selected]);

  const updateSearchQuery = useCallback((nextQuery: string) => {
    searchQueryRef.current = nextQuery;
    setSearchQuery(nextQuery);
    setSearchResults([]);
    setSearchCurrentIndex(-1);
    setSearchStatus("idle");
  }, []);

  const runSearch = useCallback(async () => {
    const query = searchQuery.trim();
    if (!query) {
      setSearchResults([]);
      setSearchCurrentIndex(-1);
      setSearchStatus("idle");
      return;
    }
    const requestId = searchRequestIdRef.current + 1;
    searchRequestIdRef.current = requestId;
    setBusy("search");
    setSearchStatus("loading");
    try {
      const result = await client.search(query);
      if (requestId !== searchRequestIdRef.current || searchQueryRef.current.trim() !== query) return;
      setSearchResults(result.results);
      setSearchCurrentIndex(result.results.length ? 0 : -1);
      setSearchStatus(result.results.length ? "results" : "empty");
      setNotice(result.results.length ? `找到 ${result.results.length} 条匹配。` : "没有找到匹配；可新建 Markdown 笔记。");
    } catch {
      if (requestId !== searchRequestIdRef.current || searchQueryRef.current.trim() !== query) return;
      setSearchResults([]);
      setSearchCurrentIndex(-1);
      setSearchStatus("error");
      setNotice("搜索没有完成；查询只会发送给固定 Syn vault。");
    } finally {
      if (requestId === searchRequestIdRef.current) {
        setBusy((current) => current === "search" ? null : current);
      }
    }
  }, [client, searchQuery]);

  const openWikilink = useCallback(
    (title: string) => {
      const matches = entries.filter(
        (entry) =>
          entry.kind === "markdown" &&
          [entry.relative_path, stripMarkdownExtension(entry.relative_path), entry.title ?? ""]
            .map((candidate) => candidate.trim().toLocaleLowerCase())
            .includes(title.trim().toLocaleLowerCase()),
      );
      if (matches.length === 1) {
        void openMarkdownInWorkbench(matches[0].relative_path);
        return;
      }
      setPendingWikilink(title);
      if (matches.length > 1) {
        setNotice(`「${title}」对应多条笔记；请通过快速打开选择，避免猜测跳转目标。`);
      } else {
        setNotice(`「${title}」还没有对应笔记。可在下方 Syn 命令面板中由你确认新建。`);
      }
    },
    [entries, openMarkdownInWorkbench],
  );

  const prepareWikilinkCreate = useCallback(() => {
    if (!pendingWikilink) return;
    setCommandKind("markdown");
    setCommandPath(defaultMarkdownPathForTitle(pendingWikilink));
    setCommandStage("create");
  }, [pendingWikilink]);

  const createWorkspaceEntry = useCallback(async () => {
    const relativePath = commandPath.trim();
    if (!relativePath) {
      setNotice("先填写 vault 内的相对路径。目录和文件名会由固定客户端再次校验。");
      return;
    }
    const draftIsProtected = selected !== null && (needsReload || draft !== selected.body);
    if (
      commandKind === "markdown"
      && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve"
    ) {
      setNotice("当前 Markdown 草稿尚未保存；请先保存或取消编辑后再新建并打开笔记。");
      return;
    }
    setBusy("create");
    try {
      if (commandKind === "directory") {
        await client.createDirectory(relativePath);
        setNotice("已创建目录并刷新目录树。");
      } else {
        await client.createMarkdown(relativePath, "");
        setNotice("已创建 Markdown 笔记；现在可在 Syn 内编辑。");
      }
      await reloadSnapshot();
      if (commandKind === "markdown") await openMarkdownInWorkbench(relativePath);
      setCommandPath("");
      setPendingWikilink(null);
      requestWorkbenchMutation();
    } catch {
      setNotice("创建没有完成。路径、扩展名和固定 vault 边界都没有被放宽。");
    } finally {
      setBusy(null);
    }
  }, [client, commandKind, commandPath, draft, needsReload, openMarkdownInWorkbench, reloadSnapshot, requestWorkbenchMutation, selected]);

  const activateCentralTabInGroup = useCallback(async (
    groupId: KnowledgeWorkbenchGroupId, tabId: string,
  ): Promise<boolean> => {
    const current = centralStateRef.current;
    const group = current.groups.find((candidate) => candidate.id === groupId);
    const tab = group?.tabs.find((candidate) => candidate.id === tabId);
    if (!group || !tab) return false;
    if (tab.kind === "markdown" && selectedRelativePathRef.current !== tab.relativePath) {
      const disposition = knowledgeWorkspaceCentralTransitionDisposition({
        draftIsDirty: selected !== null && draft !== selected.body,
        needsReload,
        intent: "switch-markdown",
      });
      if (disposition === "preserve") {
        setNotice("当前 Markdown 草稿尚未保存或存在冲突；已拒绝切换标签，草稿仍保留。");
        return false;
      }
      if (!await openDocument(tab.relativePath)) return false;
    }
    const next = updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "activate-tab", groupId, tabId },
    ));
    const nextGroup = knowledgeWorkbenchActiveGroup(next);
    const nextTab = knowledgeWorkbenchActiveTab(next);
    if (nextTab) focusCentralTab(nextGroup.id, nextTab.id);
    return true;
  }, [draft, focusCentralTab, needsReload, openDocument, selected, updateCentralState]);

  const openCentralSurface = useCallback((
    surface: KnowledgeWorkbenchCentralSurface, groupId?: KnowledgeWorkbenchGroupId,
  ) => {
    const next = updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "open-surface", surface, groupId },
    ));
    const nextGroup = knowledgeWorkbenchActiveGroup(next);
    const nextTab = knowledgeWorkbenchActiveTab(next);
    if (nextTab) focusCentralTab(nextGroup.id, nextTab.id);
  }, [focusCentralTab, updateCentralState]);

  const closeCentralTab = useCallback(async (
    groupId: KnowledgeWorkbenchGroupId, tab: KnowledgeWorkbenchCentralTab,
  ) => {
    const current = centralStateRef.current;
    const group = current.groups.find((candidate) => candidate.id === groupId);
    if (!group) return;
    const focusTarget = knowledgeWorkbenchCloseFocusTarget(group, tab.id);
    const closingSelectedMarkdown = tab.kind === "markdown"
      && selectedRelativePathRef.current === tab.relativePath;
    if (closingSelectedMarkdown && knowledgeWorkspaceCentralTransitionDisposition({
      draftIsDirty: selected !== null && draft !== selected.body,
      needsReload,
      intent: "close-markdown",
    }) === "preserve") {
      setNotice("当前 Markdown 草稿尚未保存或存在冲突；已拒绝关闭标签，草稿仍保留。");
      return;
    }

    let next = reduceKnowledgeWorkbenchCentralState(
      current, { type: "close-tab", groupId, tabId: tab.id },
    );
    if (focusTarget.kind === "tab") {
      const targetGroup = next.groups.find((candidate) => candidate.id === groupId);
      if (targetGroup?.tabs.some((candidate) => candidate.id === focusTarget.tabId)) {
        next = reduceKnowledgeWorkbenchCentralState(
          next, { type: "activate-tab", groupId, tabId: focusTarget.tabId },
        );
      }
    }

    const nextActiveTab = knowledgeWorkbenchActiveTab(next);
    if (
      closingSelectedMarkdown
      && nextActiveTab?.kind === "markdown"
      && nextActiveTab.relativePath !== selectedRelativePathRef.current
      && !await openDocument(nextActiveTab.relativePath)
    ) {
      return;
    }

    const selectedPathBeforeClose = selectedRelativePathRef.current;
    const selectedStillTabbed = selectedPathBeforeClose !== null && next.groups.some((candidate) => (
      candidate.tabs.some((candidateTab) => (
        candidateTab.kind === "markdown" && candidateTab.relativePath === selectedPathBeforeClose
      ))
    ));
    if (closingSelectedMarkdown && !selectedStillTabbed && nextActiveTab?.kind !== "markdown") {
      clearSelectedDocument();
      replaceDraft("");
      setNeedsReload(false);
    }

    updateCentralState(next);
    if (focusTarget.kind === "tab") focusCentralTab(groupId, focusTarget.tabId);
    else focusCentralGroupTools(groupId);
  }, [
    clearSelectedDocument, draft, focusCentralGroupTools, focusCentralTab,
    needsReload, openDocument, replaceDraft, selected, updateCentralState,
  ]);

  const splitCentralGroupRight = useCallback(() => {
    const current = centralStateRef.current;
    const next = reduceKnowledgeWorkbenchCentralState(current, { type: "split-right" });
    if (next === current) return;
    updateCentralState(next);
    const secondaryTab = knowledgeWorkbenchActiveTab(next, SECONDARY_KNOWLEDGE_GROUP_ID);
    if (secondaryTab) focusCentralTab(SECONDARY_KNOWLEDGE_GROUP_ID, secondaryTab.id);
  }, [focusCentralTab, updateCentralState]);

  const mergeCentralGroups = useCallback(() => {
    const current = centralStateRef.current;
    const next = reduceKnowledgeWorkbenchCentralState(current, { type: "merge-groups" });
    if (next === current) return;
    const wouldDiscardLastDraft = selectedRelativePathRef.current !== null
      && !knowledgeWorkbenchMarkdownPaths(next).includes(selectedRelativePathRef.current);
    if (knowledgeWorkspaceCentralTransitionDisposition({
      draftIsDirty: selected !== null && draft !== selected.body,
      needsReload,
      intent: "merge-groups",
      wouldDiscardLastDraft,
    }) === "preserve") {
      setNotice("合并会丢掉最后一个未保存草稿投影；已拒绝合并。");
      return;
    }
    updateCentralState(next);
    const nextTab = knowledgeWorkbenchActiveTab(next);
    if (nextTab) focusCentralTab(PRIMARY_KNOWLEDGE_GROUP_ID, nextTab.id);
    else focusCentralGroupTools(PRIMARY_KNOWLEDGE_GROUP_ID);
  }, [draft, focusCentralGroupTools, focusCentralTab, needsReload, selected, updateCentralState]);

  const setCentralProjection = useCallback((
    groupId: KnowledgeWorkbenchGroupId, projection: "source" | "preview",
  ) => {
    const next = updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "set-projection", groupId, projection },
    ));
    const tab = knowledgeWorkbenchActiveTab(next, groupId);
    if (tab) focusCentralTab(groupId, tab.id);
  }, [focusCentralTab, updateCentralState]);

  const resizeCentralGroupsFromPointer = useCallback((clientX: number) => {
    const bounds = centralGroupsRef.current?.getBoundingClientRect();
    if (!bounds || bounds.width <= 0) return;
    const ratio = ((clientX - bounds.left) / bounds.width) * 100;
    updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "set-split-ratio", ratio },
    ));
  }, [updateCentralState]);

  const handleSeparatorPointerDown = useCallback((event: ReactPointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.currentTarget.setPointerCapture(event.pointerId);
    resizeCentralGroupsFromPointer(event.clientX);
  }, [resizeCentralGroupsFromPointer]);

  const handleSeparatorPointerMove = useCallback((event: ReactPointerEvent<HTMLDivElement>) => {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
    resizeCentralGroupsFromPointer(event.clientX);
  }, [resizeCentralGroupsFromPointer]);

  const handleSeparatorPointerUp = useCallback((event: ReactPointerEvent<HTMLDivElement>) => {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }, []);

  const handleSeparatorKeyDown = useCallback((event: ReactKeyboardEvent<HTMLDivElement>) => {
    const ratio = knowledgeWorkbenchSplitRatioForKey(centralStateRef.current.splitRatio, event.key);
    if (ratio === centralStateRef.current.splitRatio) return;
    event.preventDefault();
    updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "set-split-ratio", ratio },
    ));
  }, [updateCentralState]);

  const openWorkbenchOverlay = useCallback((
    overlay: Exclude<KnowledgeWorkbenchOverlay, null>, trigger: HTMLElement | null,
    groupId: KnowledgeWorkbenchGroupId = centralStateRef.current.activeGroupId,
  ) => {
    overlayTriggerRef.current = trigger;
    quickOpenTargetGroupRef.current = groupId;
    updateCentralState(reduceKnowledgeWorkbenchCentralState(
      centralStateRef.current, { type: "set-active-group", groupId },
    ));
    if (overlay === "command") {
      setCommandStage("list");
      setCommandQuery("");
      setCommandCurrentIndex(0);
    }
    dispatchWorkbenchLayout({ type: "open-overlay", overlay });
  }, [updateCentralState]);
  const closeWorkbenchOverlay = useCallback(({ restoreFocus = true }: Readonly<{ restoreFocus?: boolean }> = {}) => {
    dispatchWorkbenchLayout({ type: "close-overlay" });
    if (!restoreFocus) return;
    restoreKnowledgeWorkbenchOverlayFocus(
      overlayTriggerRef.current,
      (callback) => window.setTimeout(callback, 0),
    );
  }, []);

  const activateSearchResult = useCallback(async (relativePath: string, closeOverlayAfterOpen: boolean): Promise<boolean> => {
    const targetGroupId = closeOverlayAfterOpen
      ? quickOpenTargetGroupRef.current
      : centralStateRef.current.activeGroupId;
    const opened = await openMarkdownInWorkbench(relativePath, targetGroupId);
    if (opened && closeOverlayAfterOpen) closeWorkbenchOverlay({ restoreFocus: false });
    return opened;
  }, [closeWorkbenchOverlay, openMarkdownInWorkbench]);

  const selectCommand = useCallback((kind: WorkspaceCommandKind) => {
    setCommandKind(kind);
    setCommandPath("");
    setCommandStage("create");
  }, []);

  const handleSearchListKeyDown = useCallback((event: Pick<KeyboardEvent, "key" | "preventDefault">, closeOverlayAfterOpen: boolean) => {
    const action = knowledgeWorkbenchListActionForKey(event.key);
    if (!action) return;
    if (action === "dismiss") {
      return;
    }
    if (action === "previous" || action === "next") {
      event.preventDefault();
      setSearchCurrentIndex((current) => knowledgeWorkbenchMoveListSelection(current, searchResults.length, action));
      return;
    }
    event.preventDefault();
    const currentResult = searchResults[searchCurrentIndex];
    if (currentResult) {
      void activateSearchResult(currentResult.relative_path, closeOverlayAfterOpen);
    } else if (searchQuery.trim() && busy !== "search") {
      void runSearch();
    }
  }, [activateSearchResult, busy, closeWorkbenchOverlay, runSearch, searchCurrentIndex, searchQuery, searchResults]);

  const handleCommandListKeyDown = useCallback((event: Pick<KeyboardEvent, "key" | "preventDefault">) => {
    const action = knowledgeWorkbenchListActionForKey(event.key);
    if (!action) return;
    if (action === "dismiss") {
      return;
    }
    if (action === "previous" || action === "next") {
      event.preventDefault();
      setCommandCurrentIndex((current) => knowledgeWorkbenchMoveListSelection(current, filteredCommands.length, action));
      return;
    }
    event.preventDefault();
    const currentCommand = filteredCommands[commandCurrentIndex];
    if (currentCommand) selectCommand(currentCommand.kind);
  }, [closeWorkbenchOverlay, commandCurrentIndex, filteredCommands, selectCommand]);

  useEffect(() => {
    if (!activeOverlay) return;
    const target = activeOverlay === "quick-open"
      ? quickOpenInputRef.current
      : commandStage === "list"
        ? commandFilterInputRef.current
        : commandPathInputRef.current;
    target?.focus();
  }, [activeOverlay, commandStage]);

  const cycleCentralTabs = useCallback(async (direction: "next" | "previous") => {
    const current = centralStateRef.current;
    const next = reduceKnowledgeWorkbenchCentralState(current, { type: "cycle-tab", direction });
    if (next === current) return;
    const nextGroup = knowledgeWorkbenchActiveGroup(next);
    const nextTab = knowledgeWorkbenchActiveTab(next);
    if (!nextTab) return;
    if (nextTab.kind === "markdown" && selectedRelativePathRef.current !== nextTab.relativePath) {
      if (knowledgeWorkspaceCentralTransitionDisposition({
        draftIsDirty: selected !== null && draft !== selected.body,
        needsReload,
        intent: "switch-markdown",
      }) === "preserve") {
        setNotice("当前 Markdown 草稿尚未保存或存在冲突；已拒绝循环切换标签。");
        return;
      }
      if (!await openDocument(nextTab.relativePath)) return;
    }
    updateCentralState(next);
    focusCentralTab(nextGroup.id, nextTab.id);
  }, [draft, focusCentralTab, needsReload, openDocument, selected, updateCentralState]);

  useEffect(() => {
    const handleWorkbenchShortcut = (event: KeyboardEvent) => {
      if (activeOverlay) return;
      const tabTarget = knowledgeWorkbenchTabShortcutTarget({
        key: event.key,
        metaKey: event.metaKey,
        shiftKey: event.shiftKey,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
      });
      if (tabTarget) {
        event.preventDefault();
        const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
        if (tabTarget === "quick-open") {
          openWorkbenchOverlay("quick-open", trigger, centralStateRef.current.activeGroupId);
        } else {
          void cycleCentralTabs(tabTarget === "next-tab" ? "next" : "previous");
        }
        return;
      }
      const target = knowledgeWorkbenchShortcutTarget({
        key: event.key,
        metaKey: event.metaKey,
        shiftKey: event.shiftKey,
        altKey: event.altKey,
        ctrlKey: event.ctrlKey,
      });
      if (!target) return;
      event.preventDefault();
      const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      if (target === "left-search") {
        dispatchWorkbenchLayout({ type: "select-left-view", view: "search" });
        window.setTimeout(() => leftSearchInputRef.current?.focus(), 0);
        return;
      }
      openWorkbenchOverlay(target, trigger);
    };
    window.addEventListener("keydown", handleWorkbenchShortcut);
    return () => window.removeEventListener("keydown", handleWorkbenchShortcut);
  }, [activeOverlay, cycleCentralTabs, openWorkbenchOverlay]);

  const overlay = activeOverlay ? (
    <div className="syn-knowledge-overlay-backdrop" role="presentation">
      <div
        className="syn-knowledge-overlay"
        role="dialog"
        aria-modal="true"
        aria-label={activeOverlay === "quick-open" ? "快速打开" : "Syn 命令"}
        tabIndex={-1}
        ref={overlayDialogRef}
        onKeyDown={(event) => {
          if (knowledgeWorkbenchOverlayDismissesForKey(event.key)) {
            event.preventDefault();
            closeWorkbenchOverlay();
          }
        }}
      >
        {activeOverlay === "quick-open" ? (
          <>
            <div className="syn-knowledge-overlay__compact-head">
              <p className="eyebrow">快速打开</p>
              <button className="text-button" type="button" onClick={() => closeWorkbenchOverlay()}>关闭</button>
            </div>
            <div className="native-workspace-quick-open" aria-label="快速打开">
              <label htmlFor="native-knowledge-quick-open">快速打开</label>
              <input
                id="native-knowledge-quick-open"
                ref={quickOpenInputRef}
                role="combobox"
                aria-autocomplete="list"
                aria-controls="native-knowledge-quick-open-results"
                aria-activedescendant={searchCurrentIndex >= 0 ? `native-knowledge-quick-open-option-${searchCurrentIndex}` : undefined}
                aria-expanded={searchStatus === "results"}
                value={searchQuery}
                placeholder="搜索标题、正文或标签"
                onChange={(event) => updateSearchQuery(event.target.value)}
                onKeyDown={(event) => handleSearchListKeyDown(event, true)}
              />
              <button className="secondary-button" type="button" onClick={() => void runSearch()} disabled={busy !== null || !searchQuery.trim()}>
                {busy === "search" ? "正在查找…" : "查找"}
              </button>
              {searchStatus === "loading" ? <p className="native-workspace-search-state" role="status">正在查找固定 Syn vault…</p> : null}
              {searchStatus === "empty" ? <p className="native-workspace-search-state" role="status">没有匹配项。</p> : null}
              {searchStatus === "error" ? <p className="native-workspace-search-state" role="status">搜索没有完成；没有改写工作区。</p> : null}
              {searchStatus === "results" ? (
                <div className="native-workspace-search-results" id="native-knowledge-quick-open-results" role="listbox" aria-label="快速打开结果" onKeyDown={(event) => handleSearchListKeyDown(event, true)}>
                  {searchResults.map((result, index) => (
                    <button
                      type="button"
                      key={result.relative_path}
                      id={`native-knowledge-quick-open-option-${index}`}
                      role="option"
                      aria-selected={searchCurrentIndex === index}
                      className={searchCurrentIndex === index ? "is-current" : ""}
                      onFocus={() => setSearchCurrentIndex(index)}
                      onClick={() => {
                        setSearchCurrentIndex(index);
                        void activateSearchResult(result.relative_path, true);
                      }}
                    >
                      <strong>{result.title}</strong>
                      <span>{result.relative_path}</span>
                      <small>{result.snippet}</small>
                    </button>
                  ))}
                </div>
              ) : null}
              <div className="native-workspace-overlay-hints" aria-label="快速打开键盘提示"><span>↑↓ 选择</span><span>Enter 打开</span><span>Esc 关闭</span></div>
            </div>
          </>
        ) : (
          <>
            <div className="syn-knowledge-overlay__compact-head">
              <p className="eyebrow">Syn 命令</p>
              <button className="text-button" type="button" onClick={() => closeWorkbenchOverlay()}>关闭</button>
            </div>
            {commandStage === "list" ? (
              <div className="native-workspace-command-listing" aria-label="Syn 命令列表">
                <label htmlFor="native-knowledge-command-filter">筛选 Syn 命令</label>
                <input
                  id="native-knowledge-command-filter"
                  ref={commandFilterInputRef}
                  role="combobox"
                  aria-autocomplete="list"
                  aria-controls="native-knowledge-command-results"
                  aria-activedescendant={commandCurrentIndex >= 0 ? `native-knowledge-command-option-${commandCurrentIndex}` : undefined}
                  aria-expanded={filteredCommands.length > 0}
                  value={commandQuery}
                  placeholder="筛选命令"
                  onChange={(event) => {
                    setCommandQuery(event.target.value);
                    setCommandCurrentIndex(0);
                  }}
                  onKeyDown={handleCommandListKeyDown}
                />
                {filteredCommands.length ? (
                  <div className="native-workspace-command-results" id="native-knowledge-command-results" role="listbox" aria-label="Syn 命令结果" onKeyDown={handleCommandListKeyDown}>
                    {filteredCommands.map((command, index) => (
                      <button
                        type="button"
                        id={`native-knowledge-command-option-${index}`}
                        key={command.kind}
                        role="option"
                        aria-selected={commandCurrentIndex === index}
                        className={commandCurrentIndex === index ? "is-current" : ""}
                        onFocus={() => setCommandCurrentIndex(index)}
                        onClick={() => {
                          setCommandCurrentIndex(index);
                          selectCommand(command.kind);
                        }}
                      >
                        <strong>{command.label}</strong>
                        <small>{command.detail}</small>
                      </button>
                    ))}
                  </div>
                ) : <p className="native-workspace-search-state" role="status">没有匹配的受限 Syn 命令。</p>}
                <div className="native-workspace-overlay-hints" aria-label="Syn 命令键盘提示"><span>↑↓ 选择</span><span>Enter 继续</span><span>Esc 关闭</span></div>
                {pendingWikilink ? (
                  <div className="native-workspace-wikilink-create">
                    <span>「{pendingWikilink}」还没有唯一可打开的笔记。</span>
                    <button className="secondary-button" type="button" onClick={prepareWikilinkCreate}>用此标题准备新建</button>
                  </div>
                ) : null}
              </div>
            ) : (
              <div className="native-workspace-command-create">
                <p>只接受固定 Syn vault 内的相对路径；不会打开 shell、外部目录或任意文件系统。</p>
                <div className="native-workspace-command-controls">
                  <div className="native-workspace-command-kind" aria-label="新建类型">
                    <button className={commandKind === "markdown" ? "is-active" : ""} type="button" aria-pressed={commandKind === "markdown"} onClick={() => setCommandKind("markdown")}>新建 Markdown</button>
                    <button className={commandKind === "directory" ? "is-active" : ""} type="button" aria-pressed={commandKind === "directory"} onClick={() => setCommandKind("directory")}>新建目录</button>
                  </div>
                  <input
                    ref={commandPathInputRef}
                    aria-label="新建条目的相对路径"
                    value={commandPath}
                    placeholder={commandKind === "markdown" ? "research/idea.md" : "research"}
                    onChange={(event) => setCommandPath(event.target.value)}
                  />
                  <button className="secondary-button" type="button" onClick={() => void createWorkspaceEntry()} disabled={busy !== null || !commandPath.trim()}>
                    {busy === "create" ? "正在创建…" : "创建"}
                  </button>
                </div>
                <button className="text-button" type="button" onClick={() => setCommandStage("list")}>返回命令列表</button>
                {pendingWikilink ? <div className="native-workspace-wikilink-create"><span>「{pendingWikilink}」还没有唯一可打开的笔记。</span></div> : null}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  ) : null;

  const draftIsProtected = selected !== null && (needsReload || draft !== selected.body);
  const splitStyle = {
    "--knowledge-primary-track": `${centralState.splitRatio}%`,
    "--knowledge-secondary-track": `${100 - centralState.splitRatio}%`,
  } as CSSProperties;

  const activateCentralGroup = (groupId: KnowledgeWorkbenchGroupId) => {
    if (centralStateRef.current.activeGroupId === groupId) return;
    updateCentralState(reduceKnowledgeWorkbenchCentralState(centralStateRef.current, {
      type: "set-active-group",
      groupId,
    }));
  };

  const renderMarkdownGroupPanel = (
    group: KnowledgeWorkbenchTabGroup,
    tab: KnowledgeWorkbenchMarkdownTab,
  ) => {
    const selectedMatchesTab = selected?.relative_path === tab.relativePath;
    return (
      <div className="native-workspace-paper knowledge-workbench-markdown-panel">
        <div className="native-workspace-document-head">
          <div>
            <p className="eyebrow">{tab.relativePath}</p>
            <h3>{selectedMatchesTab ? selected?.title : "正在准备 Markdown 投影"}</h3>
          </div>
          <span className="knowledge-workbench-projection-label">
            {tab.projection === "source" ? "源码投影" : "阅读投影"}
          </span>
        </div>
        {selectedMatchesTab ? (
          <div className={`native-workspace-document native-workspace-document--${tab.projection}`}>
            {tab.projection === "source" ? (
              <label className="native-workspace-source">
                <span>Markdown 源码</span>
                <textarea
                  value={draft}
                  onChange={(event) => replaceDraft(event.target.value)}
                  spellCheck={false}
                  aria-label={`Markdown 源码：${tab.relativePath}`}
                  disabled={needsReload || busy === "save"}
                />
              </label>
            ) : (
              <div className="native-workspace-preview" aria-label={`渲染预览：${tab.relativePath}`}>
                <span>渲染预览</span>
                <WorkspaceMarkdownPreview
                  body={draft}
                  onOpenWikilink={openWikilink}
                  attachmentReferences={attachmentReferences}
                />
              </div>
            )}
          </div>
        ) : (
          <div className="native-workspace-empty-document" role="status">
            <p>这条标签尚未完成受限读取；现有草稿没有被替换。</p>
          </div>
        )}
        {group.id === saveOwnerGroupId ? (
          <div className="native-workspace-save-row">
            <span>{selected ? "源码与预览共享同一草稿；保存会检查 mtime 与内容 hash。" : "选择笔记后可开始编辑。"}</span>
            {selected ? (
              <button
                className="primary-button"
                type="button"
                onClick={() => void saveDraft()}
                disabled={busy !== null || needsReload}
              >
                {busy === "save" ? "正在保存…" : "保存 Markdown"}
              </button>
            ) : null}
          </div>
        ) : (
          <div className="native-workspace-save-row knowledge-workbench-shared-draft-note">
            <span>与另一组共享同一 Markdown 草稿和冲突状态。</span>
          </div>
        )}
      </div>
    );
  };

  const renderCentralTabPanel = (
    group: KnowledgeWorkbenchTabGroup,
    tab: KnowledgeWorkbenchCentralTab,
  ) => {
    if (tab.kind === "markdown") return renderMarkdownGroupPanel(group, tab);
    if (tab.surface === "graph") {
      return (
        <KnowledgeGraphView
          onOpenMarkdown={(relativePath) => openMarkdownInWorkbench(relativePath, group.id)}
          refreshRequestId={workbenchRefreshRequestId}
        />
      );
    }
    if (tab.surface === "canvas") {
      return (
        <KnowledgeCanvasView
          refreshRequestId={workbenchRefreshRequestId}
          onWorkspaceMutation={requestWorkbenchMutation}
        />
      );
    }
    return (
      <KnowledgeWorkspaceMaintenancePanel
        refreshRequestId={workbenchRefreshRequestId}
        onWorkspaceMutation={requestWorkbenchMutation}
        onInsertMarkdownReference={insertAttachmentReference}
      />
    );
  };

  const renderCentralGroup = (group: KnowledgeWorkbenchTabGroup) => {
    const groupIsActive = group.id === centralState.activeGroupId;
    const groupLabel = group.id === PRIMARY_KNOWLEDGE_GROUP_ID ? "主标签组" : "右标签组";
    const groupActiveTab = knowledgeWorkbenchActiveTab(centralState, group.id);
    const groupHasActiveMarkdown = groupActiveTab?.kind === "markdown";
    return (
      <section
        className={`knowledge-workbench-group${groupIsActive ? " is-active-group" : ""}`}
        aria-label={groupLabel}
        data-knowledge-tab-group={group.id}
        data-active-group={groupIsActive ? "true" : "false"}
        key={group.id}
        onPointerDown={() => activateCentralGroup(group.id)}
      >
        <header className="knowledge-workbench-group__header">
          <div
            className="knowledge-workbench-group__tabs"
            role="tablist"
            aria-label={groupLabel}
            data-knowledge-group-tablist
          >
            {group.tabs.map((tab) => {
              const tabIsActive = group.activeTabId === tab.id;
              const tabDomId = knowledgeWorkbenchTabDomId(group.id, tab.id);
              const panelDomId = knowledgeWorkbenchPanelDomId(group.id, tab.id);
              const label = knowledgeWorkbenchCentralTabLabel(tab);
              const dirty = tab.kind === "markdown"
                && tab.relativePath === selected?.relative_path
                && draftIsProtected;
              const accessibleLabel = tab.kind === "markdown"
                ? `${tab.relativePath}，${tab.projection === "source" ? "Markdown 源码" : "渲染预览"}${dirty ? "，未保存" : ""}`
                : label;
              return (
                <div className={`knowledge-workbench-tab${tabIsActive ? " is-active" : ""}`} role="presentation" key={tab.id}>
                  <button
                    type="button"
                    role="tab"
                    id={tabDomId}
                    aria-controls={panelDomId}
                    aria-selected={tabIsActive}
                    aria-label={accessibleLabel}
                    tabIndex={tabIsActive ? 0 : -1}
                    title={tab.kind === "markdown" ? tab.relativePath : label}
                    ref={(element) => {
                      const key = centralTabRefKey(group.id, tab.id);
                      if (element) centralTabButtonRefs.current.set(key, element);
                      else centralTabButtonRefs.current.delete(key);
                    }}
                    onClick={() => void activateCentralTabInGroup(group.id, tab.id)}
                  >
                    <span>{label}</span>
                    {dirty ? <span className="knowledge-workbench-tab__dirty" aria-hidden="true">●</span> : null}
                  </button>
                  <button className="knowledge-workbench-tab__close" type="button" aria-label={`关闭 ${tab.kind === "markdown" ? tab.relativePath : label}`} onClick={() => void closeCentralTab(group.id, tab)}>×</button>
                </div>
              );
            })}
            {!group.tabs.length ? (
              <span className="native-workspace-tabs-empty">从目录或快速打开选择一条 Markdown 笔记</span>
            ) : null}
          </div>
          <div className="knowledge-workbench-group__tools" aria-label={`${groupLabel}工具`}>
            {groupIsActive ? <span className="knowledge-workbench-group__current">当前组</span> : null}
            {groupHasActiveMarkdown ? (
              <div className="knowledge-workbench-projection-controls" aria-label={`${groupLabel} Markdown 投影`}>
                <button className={groupActiveTab.projection === "source" ? "is-active" : ""} type="button" aria-label={`${groupLabel}显示 Markdown 源码`} aria-pressed={groupActiveTab.projection === "source"} onClick={() => setCentralProjection(group.id, "source")}>源码</button>
                <button className={groupActiveTab.projection === "preview" ? "is-active" : ""} type="button" aria-label={`${groupLabel}显示渲染预览`} aria-pressed={groupActiveTab.projection === "preview"} onClick={() => setCentralProjection(group.id, "preview")}>预览</button>
              </div>
            ) : null}
            <button
              type="button"
              aria-label={`在${groupLabel}中快速打开`}
              title="快速打开（⌘T）"
              ref={(element) => {
                if (element) groupToolButtonRefs.current.set(group.id, element);
                else groupToolButtonRefs.current.delete(group.id);
              }}
              onClick={(event) => openWorkbenchOverlay("quick-open", event.currentTarget, group.id)}
            >
              +
            </button>
            {centralState.groups.length === 1 ? (
              <button
                type="button"
                aria-label="向右分栏"
                title="向右分栏"
                disabled={!groupHasActiveMarkdown}
                onClick={splitCentralGroupRight}
              >
                分栏
              </button>
            ) : null}
            {centralState.groups.length === 2 && group.id === SECONDARY_KNOWLEDGE_GROUP_ID ? (
              <button type="button" aria-label="合并分栏" title="合并分栏" onClick={mergeCentralGroups}>合并</button>
            ) : null}
            {group.id === PRIMARY_KNOWLEDGE_GROUP_ID ? (
              <button
                type="button"
                aria-label="刷新工作区"
                title="刷新工作区"
                onClick={refreshWorkspaceManually}
                disabled={busy !== null}
              >
                {busy === "refresh" ? "刷新中" : "刷新"}
              </button>
            ) : null}
          </div>
        </header>
        {groupActiveTab ? (
          group.tabs.map((tab) => {
            const tabIsActive = tab.id === group.activeTabId;
            return (
              <section
                className="knowledge-workbench-group__panel"
                role="tabpanel"
                id={knowledgeWorkbenchPanelDomId(group.id, tab.id)}
                aria-labelledby={knowledgeWorkbenchTabDomId(group.id, tab.id)}
                tabIndex={tabIsActive ? 0 : -1}
                hidden={!tabIsActive}
                data-knowledge-group-panel={tabIsActive ? "active" : "inactive"}
                key={tab.id}
              >
                {tabIsActive ? renderCentralTabPanel(group, tab) : null}
              </section>
            );
          })
        ) : (
          <div className="knowledge-workbench-group__empty" data-knowledge-group-empty>
            <p>这个标签组还没有内容。</p>
            <span>按 ⌘T 或使用 + 从固定 Syn vault 快速打开。</span>
          </div>
        )}
      </section>
    );
  };

  const centralWorkspace = (
    <section className="knowledge-workbench-central" aria-label="中央标签组工作区">
      {needsReload ? (
        <div className="native-workspace-conflict knowledge-workbench-central__conflict" aria-label="外部修改冲突">
          <span>外部改动已被保护。左右投影仍共享当前草稿；重新读取前不会保存或覆盖文件。</span>
          <button className="secondary-button" type="button" onClick={reloadSelected} disabled={busy !== null}>重新读取</button>
        </div>
      ) : null}
      <div
        className={`knowledge-workbench-groups${centralState.groups.length === 2 ? " is-split" : ""}`}
        ref={centralGroupsRef}
        style={splitStyle}
        data-knowledge-group-count={centralState.groups.length}
      >
        {renderCentralGroup(centralState.groups[0]!)}
        {centralState.groups.length === 2 ? (
          <>
            <div className="knowledge-workbench-separator" role="separator"
              aria-label="调整标签组分隔比例" aria-orientation="vertical"
              aria-valuemin={30} aria-valuemax={70} aria-valuenow={Math.round(centralState.splitRatio)}
              tabIndex={0} onPointerDown={handleSeparatorPointerDown} onPointerMove={handleSeparatorPointerMove}
              onPointerUp={handleSeparatorPointerUp} onPointerCancel={handleSeparatorPointerUp}
              onKeyDown={handleSeparatorKeyDown} />
            {renderCentralGroup(centralState.groups[1]!)}
          </>
        ) : null}
      </div>
    </section>
  );

  return (
    <KnowledgeWorkbenchShell
      leftCollapsed={leftCollapsed}
      rightCollapsed={rightCollapsed}
      overlay={overlay}
      activityRail={
        <KnowledgeActivityRail
          items={[
            { icon: "files", label: "文件", active: leftView === "files", onSelect: () => dispatchWorkbenchLayout({ type: "select-left-view", view: "files" }) },
            { icon: "search", label: "搜索", active: leftView === "search", onSelect: () => dispatchWorkbenchLayout({ type: "select-left-view", view: "search" }) },
            { icon: "graph", label: "关系图", active: activeCentralTab?.kind === "surface" && activeCentralTab.surface === "graph", onSelect: () => openCentralSurface("graph") },
            { icon: "canvas", label: "Canvas", active: activeCentralTab?.kind === "surface" && activeCentralTab.surface === "canvas", onSelect: () => openCentralSurface("canvas") },
            { icon: "command", label: "Syn 命令", onSelect: (event) => openWorkbenchOverlay("command", event.currentTarget) },
            { icon: "maintenance", label: "设置与维护", active: activeCentralTab?.kind === "surface" && activeCentralTab.surface === "maintenance", onSelect: () => openCentralSurface("maintenance") },
            { icon: "sources", label: "来源", active: leftView === "sources", onSelect: () => dispatchWorkbenchLayout({ type: "select-left-view", view: "sources" }) },
            { icon: "context", label: "切换右侧上下文", active: !rightCollapsed, onSelect: () => dispatchWorkbenchLayout({ type: "toggle-right" }) },
          ]}
        />
      }
      leftSidebar={
        <>
          <div className="syn-knowledge-sidebar-tabs" role="tablist" aria-label="知识左侧视图">
            <button role="tab" aria-selected={leftView === "files"} type="button" onClick={() => dispatchWorkbenchLayout({ type: "select-left-view", view: "files" })}>文件</button>
            <button role="tab" aria-selected={leftView === "search"} type="button" onClick={() => dispatchWorkbenchLayout({ type: "select-left-view", view: "search" })}>搜索</button>
            <button role="tab" aria-selected={leftView === "sources"} type="button" onClick={() => dispatchWorkbenchLayout({ type: "select-left-view", view: "sources" })}>来源</button>
            <button className="text-button" type="button" aria-label="折叠左侧栏" aria-expanded={!leftCollapsed} onClick={() => dispatchWorkbenchLayout({ type: "collapse-left" })}>折叠</button>
          </div>
          {leftView === "files" ? (
            <section className="native-workspace-spine" aria-label="文件与目录">
              <div className="native-workspace-section-head">
                <div><p className="eyebrow">文件与目录</p><strong>固定 Syn vault</strong></div>
                <span>{entries.length}</span>
              </div>
              {loadState === "loading" ? <p className="muted small-note">正在读取已验证的目录树…</p> : null}
              {loadState === "unavailable" ? <p className="muted small-note">目录树暂时读不到；不会以外部应用替代。</p> : null}
              {loadState === "ready" ? (
                <div className="native-workspace-tree">
                  {sortedEntries.map((entry) => <WorkspaceTreeEntry entry={entry} key={entry.relative_path} selectedPath={selected?.relative_path ?? null} onOpen={openMarkdownInWorkbench} />)}
                  {!sortedEntries.length ? <p className="muted small-note">这里还没有文件。用 Syn 命令新建第一条笔记。</p> : null}
                </div>
              ) : null}
            </section>
          ) : null}
          {leftView === "search" ? (
            <section className="syn-knowledge-search-panel" aria-label="知识搜索">
              <p className="eyebrow">搜索</p>
              <label htmlFor="native-knowledge-left-search">搜索固定 Syn vault</label>
              <div className="syn-knowledge-search-panel__query">
                <input
                  id="native-knowledge-left-search"
                  ref={leftSearchInputRef}
                  role="combobox"
                  aria-autocomplete="list"
                  aria-controls="native-knowledge-left-search-results"
                  aria-activedescendant={searchCurrentIndex >= 0 ? `native-knowledge-left-search-option-${searchCurrentIndex}` : undefined}
                  aria-expanded={searchStatus === "results"}
                  value={searchQuery}
                  placeholder="搜索标题、正文或标签"
                  onChange={(event) => updateSearchQuery(event.target.value)}
                  onKeyDown={(event) => handleSearchListKeyDown(event, false)}
                />
                <button className="secondary-button" type="button" onClick={() => void runSearch()} disabled={busy !== null || !searchQuery.trim()}>
                  {busy === "search" ? "正在查找…" : "搜索"}
                </button>
              </div>
              {searchStatus === "idle" ? <p>搜索结果只来自固定 Syn vault。</p> : null}
              {searchStatus === "loading" ? <p className="native-workspace-search-state" role="status">正在查找固定 Syn vault…</p> : null}
              {searchStatus === "empty" ? <p className="native-workspace-search-state" role="status">没有匹配项。</p> : null}
              {searchStatus === "error" ? <p className="native-workspace-search-state" role="status">搜索没有完成；没有改写工作区。</p> : null}
              {searchStatus === "results" ? (
                <div className="native-workspace-search-results native-workspace-search-results--sidebar" id="native-knowledge-left-search-results" role="listbox" aria-label="知识搜索结果" onKeyDown={(event) => handleSearchListKeyDown(event, false)}>
                  {searchResults.map((result, index) => (
                    <button
                      type="button"
                      key={result.relative_path}
                      id={`native-knowledge-left-search-option-${index}`}
                      role="option"
                      aria-selected={searchCurrentIndex === index}
                      className={searchCurrentIndex === index ? "is-current" : ""}
                      onFocus={() => setSearchCurrentIndex(index)}
                      onClick={() => {
                        setSearchCurrentIndex(index);
                        void activateSearchResult(result.relative_path, false);
                      }}
                    >
                      <strong>{result.title}</strong>
                      <span>{result.relative_path}</span>
                      <small>{result.snippet}</small>
                    </button>
                  ))}
                </div>
              ) : null}
            </section>
          ) : null}
          {leftView === "sources" ? sourceSidebar : null}
        </>
      }
      centralWorkspace={centralWorkspace}
      rightSidebar={
        <KnowledgeContextSidebar
          selected={selected}
          rightCollapsed={rightCollapsed}
          onCollapseRight={() => dispatchWorkbenchLayout({ type: "collapse-right" })}
          onOpen={openMarkdownInWorkbench}
          sourceContext={sourceContext}
        />
      }
      statusBar={
        <>
          <span>{selected?.relative_path ?? "未选择文件"}</span>
          <span>{knowledgeWorkbenchCentralTabStatusLabel(activeCentralTab)}</span>
          <span>{activeCentralGroup.id === PRIMARY_KNOWLEDGE_GROUP_ID ? "主标签组" : "右标签组"} · {centralState.groups.length} 组</span>
          <span>{needsReload ? "保存冲突：需重新读取" : busy === "save" ? "正在保存" : "已保存 / 本地草稿"}</span>
          <span>{draft ? `字数 ${draft.trim().split(/\s+/u).filter(Boolean).length}` : "字数 0"}</span>
          {attachmentReferences.references.length ? <span>附件引用 {attachmentReferences.references.length}</span> : null}
          {notice ? <span className="state-warning">{notice}</span> : null}
          {statusContent}
        </>
      }
    />
  );
}

function WorkspaceTreeEntry({
  entry,
  selectedPath,
  onOpen,
}: {
  entry: KnowledgeWorkspaceEntry;
  selectedPath: string | null;
  onOpen: (relativePath: string) => void;
}) {
  const label = entry.kind === "directory" ? lastPathSegment(entry.relative_path) : entry.title ?? lastPathSegment(entry.relative_path);
  const style = { paddingInlineStart: `${10 + workspacePathDepth(entry.relative_path) * 12}px` };
  if (entry.kind !== "markdown") {
    return (
      <div className="native-workspace-tree-static" style={style} title={entry.relative_path}>
        <span>{entry.kind === "directory" ? "目录" : entry.kind === "canvas" ? "Canvas" : "附件"}</span>
        <strong>{label}</strong>
      </div>
    );
  }
  return (
    <button
      className={`native-workspace-tree-note${selectedPath === entry.relative_path ? " is-selected" : ""}`}
      type="button"
      style={style}
      onClick={() => onOpen(entry.relative_path)}
      title={entry.relative_path}
    >
      <span>MD</span>
      <strong>{label}</strong>
    </button>
  );
}

export type WorkspaceAttachmentReferenceStatus = Readonly<{
  references: ReadonlyArray<string>;
  missing: ReadonlyArray<string>;
}>;

// Markdown attachment syntax remains plain, safe text in the native preview;
// it is never converted into an external href, URI or file path. This small
// projection only tells the user whether a fixed-vault attachment currently
// exists, so a missing reference remains recoverable rather than invisible.
export function workspaceAttachmentReferenceStatus(
  body: string,
  entries: ReadonlyArray<Pick<KnowledgeWorkspaceEntry, "relative_path" | "kind">>,
): WorkspaceAttachmentReferenceStatus {
  const references: string[] = [];
  for (const match of body.matchAll(/!\[[^\]\r\n]*\]\((attachments\/[^)\r\n]+)\)/gu)) {
    const relativePath = match[1]?.trim() ?? "";
    if (isKnowledgeWorkspaceAttachmentRelativePath(relativePath) && !references.includes(relativePath)) {
      references.push(relativePath);
    }
  }
  const attachmentPaths = new Set(
    entries.filter((entry) => entry.kind === "attachment").map((entry) => entry.relative_path),
  );
  return {
    references,
    missing: references.filter((relativePath) => !attachmentPaths.has(relativePath)),
  };
}

function WorkspaceMarkdownPreview({
  body,
  onOpenWikilink,
  attachmentReferences,
}: {
  body: string;
  onOpenWikilink: (title: string) => void;
  attachmentReferences: WorkspaceAttachmentReferenceStatus;
}) {
  const blocks = parseMarkdown(body);
  return (
    <div className="native-workspace-markdown">
      {attachmentReferences.references.length ? (
        <aside className="native-workspace-attachment-reference-status" aria-label="受限附件引用状态">
          <strong>受限附件引用</strong>
          <span>仅使用固定 vault 内的 <code>attachments/…</code> 相对路径；预览不会把它变成外部打开链接。</span>
          {attachmentReferences.missing.length ? (
            <p className="state-warning">缺失附件：{attachmentReferences.missing.join(" · ")}。可在附件区导入、刷新或按单条备份恢复；笔记没有被改写。</p>
          ) : (
            <span>当前引用均能在已验证的附件目录中找到。</span>
          )}
        </aside>
      ) : null}
      {blocks.map((block, index) => (
        <WorkspaceMarkdownBlock block={block} key={index} onOpenWikilink={onOpenWikilink} />
      ))}
    </div>
  );
}

function WorkspaceMarkdownBlock({
  block,
  onOpenWikilink,
}: {
  block: MdBlock;
  onOpenWikilink: (title: string) => void;
}) {
  if (block.kind === "heading") {
    const Tag = (`h${block.level}`) as "h1";
    return (
      <Tag>
        <WorkspaceInlineSegments inlines={block.inlines} onOpenWikilink={onOpenWikilink} />
      </Tag>
    );
  }
  if (block.kind === "code_block") return <pre>{block.text}</pre>;
  if (block.kind === "list") {
    const items = block.items.map((item, index) => (
      <li key={index}>
        <WorkspaceInlineSegments inlines={item} onOpenWikilink={onOpenWikilink} />
      </li>
    ));
    return block.ordered ? <ol>{items}</ol> : <ul>{items}</ul>;
  }
  return (
    <p>
      <WorkspaceInlineSegments inlines={block.inlines} onOpenWikilink={onOpenWikilink} />
    </p>
  );
}

function WorkspaceInlineSegments({
  inlines,
  onOpenWikilink,
}: {
  inlines: MdInline[];
  onOpenWikilink: (title: string) => void;
}) {
  return (
    <>
      {inlines.map((segment, index) => {
        if (segment.kind === "bold") return <strong key={index}>{segment.text}</strong>;
        if (segment.kind === "italic") return <em key={index}>{segment.text}</em>;
        if (segment.kind === "code") return <code key={index}>{segment.text}</code>;
        if (segment.kind === "wikilink") {
          return (
            <button className="native-workspace-wikilink" type="button" key={index} onClick={() => onOpenWikilink(segment.title)}>
              {segment.title}
            </button>
          );
        }
        if (segment.kind === "link") {
          return (
            <a href={segment.url} target="_blank" rel="noreferrer" key={index}>
              {segment.url}
            </a>
          );
        }
        return <span key={index}>{segment.text}</span>;
      })}
    </>
  );
}

function compareWorkspaceEntries(left: KnowledgeWorkspaceEntry, right: KnowledgeWorkspaceEntry): number {
  if (left.kind === "directory" && right.kind !== "directory") return -1;
  if (left.kind !== "directory" && right.kind === "directory") return 1;
  return left.relative_path.localeCompare(right.relative_path, "zh-Hans-CN", { sensitivity: "base" });
}

function centralTabRefKey(groupId: KnowledgeWorkbenchGroupId, tabId: string): string {
  return `${groupId}:${tabId}`;
}

function knowledgeWorkbenchCentralTabLabel(tab: KnowledgeWorkbenchCentralTab): string {
  if (tab.kind === "markdown") return stripMarkdownExtension(lastPathSegment(tab.relativePath));
  switch (tab.surface) {
    case "graph":
      return "关系图";
    case "canvas":
      return "Canvas";
    case "maintenance":
      return "维护";
  }
}

function knowledgeWorkbenchCentralTabStatusLabel(tab: KnowledgeWorkbenchCentralTab | null): string {
  if (!tab) return "空标签组";
  if (tab.kind === "surface") return knowledgeWorkbenchCentralTabLabel(tab);
  return tab.projection === "source" ? "Markdown 源码" : "Markdown 预览";
}

function workspacePathDepth(relativePath: string): number {
  return Math.max(0, relativePath.split("/").length - 1);
}

function lastPathSegment(relativePath: string): string {
  return relativePath.split("/").at(-1) ?? relativePath;
}

function stripMarkdownExtension(relativePath: string): string {
  return relativePath.endsWith(".md") ? relativePath.slice(0, -3) : relativePath;
}

function defaultMarkdownPathForTitle(title: string): string {
  const stem = title
    .trim()
    .normalize("NFKC")
    .replace(/[\\/:*?\[\]{}'"=|<>]/gu, "-")
    .replace(/\s+/gu, "-")
    .replace(/^-+|-+$/gu, "");
  return `${stem && stem !== "." && stem !== ".." ? stem : "untitled"}.md`;
}

function isWorkspaceWriteConflict(error: unknown): boolean {
  const message = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  return message.includes("knowledge_vault_conflict") || message.includes("knowledge_workspace_conflict");
}
