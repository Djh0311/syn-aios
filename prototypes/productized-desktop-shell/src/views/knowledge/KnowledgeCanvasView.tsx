import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Background,
  BackgroundVariant,
  Controls,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import {
  knowledgeWorkspaceAsyncCommitDisposition,
  knowledgeWorkspaceDraftNavigationDisposition,
  knowledgeWorkspaceDraftRefreshDisposition,
} from "../../lib/knowledgeWorkspace";
import {
  knowledgeWorkspace,
  type JsonCanvasObject,
  type JsonCanvasValue,
  type KnowledgeWorkspaceCanvasDocument,
  type KnowledgeWorkspaceClient,
  type KnowledgeWorkspaceMutationResult,
  type KnowledgeWorkspaceSnapshot,
} from "../../lib/tauri";

type CanvasClient = Pick<
  KnowledgeWorkspaceClient,
  "snapshot" | "readCanvas" | "createCanvas" | "writeCanvas"
>;

type CanvasNodeType = "text" | "file" | "link" | "group";
type CanvasNodeData = Readonly<{
  id: string;
  type: CanvasNodeType;
  label: string;
  detail: string;
}>;
type CanvasFlowNode = Node<CanvasNodeData>;
type CanvasFlow = Readonly<{ nodes: CanvasFlowNode[]; edges: Edge[] }>;
type CanvasEditorAsyncRequest = Readonly<{
  draftRevision: number;
  generation: number;
  currentRelativePath: string | null;
}>;
type CanvasFocusableTarget = Readonly<{
  isConnected: boolean;
  focus: () => void;
}>;

type KnowledgeCanvasViewProps = Readonly<{
  client?: CanvasClient;
  staticSnapshot?: KnowledgeWorkspaceSnapshot | null;
  staticCanvas?: KnowledgeWorkspaceCanvasDocument | null;
  refreshRequestId?: number;
  onWorkspaceMutation?: () => void;
}>;

const EMPTY_CANVAS: JsonCanvasObject = { nodes: [], edges: [] };

// The client seam is intentionally limited to the four named workspace
// methods. It does not accept a command name, vault root, URL, shell action,
// or a source file path.
export async function loadKnowledgeCanvas(
  client: CanvasClient,
  relativePath: string,
): Promise<KnowledgeWorkspaceCanvasDocument> {
  return client.readCanvas(relativePath);
}

export async function createKnowledgeCanvas(
  client: CanvasClient,
  relativePath: string,
  document: JsonCanvasObject,
): Promise<KnowledgeWorkspaceMutationResult> {
  return client.createCanvas(relativePath, document);
}

export async function saveKnowledgeCanvas(
  client: CanvasClient,
  canvas: KnowledgeWorkspaceCanvasDocument,
  document: JsonCanvasObject,
): Promise<KnowledgeWorkspaceMutationResult> {
  return client.writeCanvas(canvas.relative_path, document, canvas.mtime_ms, canvas.content_hash);
}

export function knowledgeCanvasConflictNotice(): string {
  return "Canvas 已在另一窗口或外部来源发生变化；本地草稿已保留，请显式重新读取后再保存。";
}

export function canvasFilePanelCancelFocusTarget(
  actualOpener: CanvasFocusableTarget | null,
  chromeFallback: CanvasFocusableTarget | null,
  stageFallback: CanvasFocusableTarget | null,
): CanvasFocusableTarget | null {
  if (actualOpener?.isConnected) return actualOpener;
  if (chromeFallback?.isConnected) return chromeFallback;
  return stageFallback?.isConnected ? stageFallback : null;
}

export function canvasFilePanelSelectionFocusTarget(
  stage: CanvasFocusableTarget | null,
  chromeFallback: CanvasFocusableTarget | null,
): CanvasFocusableTarget | null {
  if (stage?.isConnected) return stage;
  return chromeFallback?.isConnected ? chromeFallback : null;
}

export function KnowledgeCanvasView({
  client = knowledgeWorkspace,
  staticSnapshot = null,
  staticCanvas = null,
  refreshRequestId = 0,
  onWorkspaceMutation,
}: KnowledgeCanvasViewProps) {
  // Keep the SSR branch hook-free. The offline renderer must never read a
  // vault or invoke Tauri merely to display the native Canvas entry.
  if (typeof window === "undefined") {
    return <KnowledgeCanvasStaticShell snapshot={staticSnapshot} canvas={staticCanvas} />;
  }
  return <KnowledgeCanvasBrowser
    client={client}
    refreshRequestId={refreshRequestId}
    onWorkspaceMutation={onWorkspaceMutation}
  />;
}

function KnowledgeCanvasStaticShell({
  snapshot,
  canvas,
}: {
  snapshot: KnowledgeWorkspaceSnapshot | null;
  canvas: KnowledgeWorkspaceCanvasDocument | null;
}) {
  const canvasEntries = canvasEntriesFromSnapshot(snapshot);
  const flow = canvas ? canvasDocumentToFlow(canvas.document) : null;
  const relativePath = canvas?.relative_path ?? canvasEntries[0]?.relative_path ?? "尚未打开 Canvas";
  return (
    <section className="native-knowledge-canvas" aria-label="Syn 原生 JSON Canvas">
      <header className="native-canvas-chrome" data-canvas-chrome="compact">
        <span className="native-canvas-file-trigger native-canvas-file-trigger--static">画布</span>
        <div className="native-canvas-current">
          <span className="native-canvas-current__path" title={relativePath}>{relativePath}</span>
          <span data-canvas-status="saved">{canvas ? "已保存" : "等待读取"}</span>
        </div>
        <span className="native-canvas-static-safety">固定 Syn vault · JSON Canvas 1.0</span>
      </header>
      <main className="native-canvas-workspace" aria-label="JSON Canvas 编辑区">
        <div className="native-canvas-flow-stage native-canvas-flow-stage--static" data-canvas-stage="continuous">
          <div className="native-canvas-static-summary">
            {flow ? (
              <p>{flow.nodes.length} 个节点 / {flow.edges.length} 条连线；未识别字段会随原始 JSON 保留。</p>
            ) : (
              <p>打开已验证的 .canvas 文件后在 Syn 内编辑；文件和链接节点不会启动外部程序。</p>
            )}
          </div>
          <div className="native-canvas-floating-tools native-canvas-floating-tools--static" aria-label="Canvas 节点工具">
            <span>文本</span><span>文件</span><span>链接</span><span>分组</span>
          </div>
        </div>
        <p className="native-canvas-static-note">发生冲突时保留本地草稿，并由用户点击“重新读取”；不会静默覆盖。</p>
      </main>
    </section>
  );
}

function KnowledgeCanvasBrowser({
  client,
  refreshRequestId,
  onWorkspaceMutation,
}: {
  client: CanvasClient;
  refreshRequestId: number;
  onWorkspaceMutation?: () => void;
}) {
  const lastRefreshRequestId = useRef(refreshRequestId);
  const draftRevisionRef = useRef(0);
  const editorGenerationRef = useRef(0);
  const canvasRelativePathRef = useRef<string | null>(null);
  const filePanelTriggerRef = useRef<HTMLButtonElement>(null);
  const filePanelOpenerRef = useRef<HTMLButtonElement | null>(null);
  const filePanelFirstActionRef = useRef<HTMLButtonElement>(null);
  const newCanvasPathInputRef = useRef<HTMLInputElement>(null);
  const canvasStageRef = useRef<HTMLDivElement>(null);
  const [snapshot, setSnapshot] = useState<KnowledgeWorkspaceSnapshot | null>(null);
  const [canvas, setCanvas] = useState<KnowledgeWorkspaceCanvasDocument | null>(null);
  const [draft, setDraft] = useState<JsonCanvasObject | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [filePanelOpen, setFilePanelOpen] = useState(false);
  const [newCanvasPath, setNewCanvasPath] = useState("canvas/new.canvas");
  const [newNodeType, setNewNodeType] = useState<CanvasNodeType>("text");
  const [edgeTargetId, setEdgeTargetId] = useState("");
  const [busy, setBusy] = useState<"refresh" | "read" | "create" | "save" | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [conflicted, setConflicted] = useState(false);

  const replaceCanvasDraft = useCallback((nextDraft: JsonCanvasObject | null) => {
    draftRevisionRef.current += 1;
    setDraft(nextDraft);
  }, []);
  const updateCanvasDraft = useCallback((updater: (current: JsonCanvasObject) => JsonCanvasObject) => {
    draftRevisionRef.current += 1;
    setDraft((current) => current ? updater(current) : current);
  }, []);
  const replaceCanvas = useCallback((nextCanvas: KnowledgeWorkspaceCanvasDocument) => {
    canvasRelativePathRef.current = nextCanvas.relative_path;
    setCanvas(nextCanvas);
  }, []);
  const beginCanvasAsyncRequest = useCallback((): CanvasEditorAsyncRequest => {
    const generation = editorGenerationRef.current + 1;
    editorGenerationRef.current = generation;
    return {
      draftRevision: draftRevisionRef.current,
      generation,
      currentRelativePath: canvasRelativePathRef.current,
    };
  }, []);
  const canCommitCanvasAsyncRequest = useCallback((request: CanvasEditorAsyncRequest): boolean => (
    knowledgeWorkspaceAsyncCommitDisposition({
      requestDraftRevision: request.draftRevision,
      currentDraftRevision: draftRevisionRef.current,
      requestGeneration: request.generation,
      currentGeneration: editorGenerationRef.current,
      requestCurrentRelativePath: request.currentRelativePath,
      currentRelativePath: canvasRelativePathRef.current,
    }) === "apply"
  ), []);

  const refreshCatalog = useCallback(async () => {
    setBusy("refresh");
    try {
      const nextSnapshot = await client.snapshot();
      setSnapshot(nextSnapshot);
      setNotice(null);
      return nextSnapshot;
    } catch {
      setNotice("JSON Canvas 目录暂时读不到；当前本地草稿没有被改写。");
      return null;
    } finally {
      setBusy(null);
    }
  }, [client]);

  const openCanvas = useCallback(async (
    relativePath: string,
    { discardDraft = false }: Readonly<{ discardDraft?: boolean }> = {},
  ): Promise<boolean> => {
    const draftIsProtected = Boolean(canvas && draft && (conflicted || canvasDraftDiffers(draft, canvas.document)));
    if (
      !discardDraft
      && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve"
    ) {
      setNotice("当前 Canvas 草稿尚未保存；请先保存或取消编辑后再切换画布。");
      return false;
    }
    const request = beginCanvasAsyncRequest();
    setBusy("read");
    try {
      const nextCanvas = await loadKnowledgeCanvas(client, relativePath);
      if (!canCommitCanvasAsyncRequest(request)) {
        setNotice("读取期间本地草稿或当前画布已变化；已保留，不会回填迟到内容。");
        return false;
      }
      replaceCanvas(nextCanvas);
      replaceCanvasDraft(nextCanvas.document);
      setSelectedNodeId(null);
      setEdgeTargetId("");
      setConflicted(false);
      setNotice(nextCanvas.diagnostics.length ? nextCanvas.diagnostics[0]?.message ?? null : null);
      return true;
    } catch {
      setNotice("这份 JSON Canvas 暂时读不到；没有改写现有草稿。");
      return false;
    } finally {
      setBusy(null);
    }
  }, [beginCanvasAsyncRequest, canCommitCanvasAsyncRequest, canvas, client, conflicted, draft, replaceCanvas, replaceCanvasDraft]);

  const refreshWorkspace = useCallback(async () => {
    const request = beginCanvasAsyncRequest();
    const nextSnapshot = await refreshCatalog();
    if (!canCommitCanvasAsyncRequest(request)) {
      setNotice("刷新期间本地草稿或当前画布已变化；已保留，不会回填迟到内容。");
      return;
    }
    if (!nextSnapshot || !canvas) return;
    setBusy("read");
    try {
      const current = await loadKnowledgeCanvas(client, canvas.relative_path);
      if (!canCommitCanvasAsyncRequest(request)) {
        setNotice("刷新期间本地草稿或当前画布已变化；已保留，不会回填迟到内容。");
        return;
      }
      const externallyChanged = current.mtime_ms !== canvas.mtime_ms || current.content_hash !== canvas.content_hash;
      const disposition = knowledgeWorkspaceDraftRefreshDisposition({
        draftIsDirty: Boolean(draft && (conflicted || canvasDraftDiffers(draft, canvas.document))),
        externallyChanged,
      });
      if (disposition === "conflict") {
        setConflicted(true);
        setNotice(knowledgeCanvasConflictNotice());
        return;
      }
      if (disposition === "preserve") {
        setNotice(conflicted ? knowledgeCanvasConflictNotice() : "已刷新已验证的 Canvas 目录；当前 Canvas 草稿尚未保存，已保留。");
        return;
      }
      replaceCanvas(current);
      replaceCanvasDraft(current.document);
      setConflicted(false);
      setNotice("已刷新已验证的 Canvas 目录和当前画布。");
    } catch {
      setNotice("Canvas 目录已刷新，但当前画布暂时读不到；没有覆盖本地草稿。");
    } finally {
      setBusy(null);
    }
  }, [beginCanvasAsyncRequest, canCommitCanvasAsyncRequest, canvas, client, conflicted, draft, refreshCatalog, replaceCanvas, replaceCanvasDraft]);

  const refreshWorkspaceManually = useCallback(() => {
    void refreshWorkspace();
    onWorkspaceMutation?.();
  }, [onWorkspaceMutation, refreshWorkspace]);

  useEffect(() => {
    void refreshCatalog();
  }, [refreshCatalog]);

  useEffect(() => {
    if (refreshRequestId === 0 || refreshRequestId === lastRefreshRequestId.current) return;
    lastRefreshRequestId.current = refreshRequestId;
    void refreshWorkspace();
  }, [refreshRequestId, refreshWorkspace]);

  const createCanvas = useCallback(async (): Promise<boolean> => {
    const draftIsProtected = Boolean(canvas && draft && (conflicted || canvasDraftDiffers(draft, canvas.document)));
    if (knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: draftIsProtected }) === "preserve") {
      setNotice("当前 Canvas 草稿尚未保存；请先保存或取消编辑后再新建并打开画布。");
      return false;
    }
    setBusy("create");
    try {
      await createKnowledgeCanvas(client, newCanvasPath, EMPTY_CANVAS);
      await refreshCatalog();
      const opened = await openCanvas(newCanvasPath);
      if (opened) setNotice("已新建 JSON Canvas；继续编辑前不会启动任何外部程序。");
      onWorkspaceMutation?.();
      return opened;
    } catch {
      setNotice("没有新建 Canvas；路径、结构或固定 vault 状态未通过受限校验。");
      return false;
    } finally {
      setBusy(null);
    }
  }, [canvas, client, conflicted, draft, newCanvasPath, onWorkspaceMutation, openCanvas, refreshCatalog]);

  const saveDraft = useCallback(async () => {
    if (!canvas || !draft || conflicted) return;
    const request = beginCanvasAsyncRequest();
    setBusy("save");
    try {
      await saveKnowledgeCanvas(client, canvas, draft);
      if (!canCommitCanvasAsyncRequest(request)) {
        if (
          request.currentRelativePath === canvasRelativePathRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setConflicted(true);
        }
        setNotice("保存已提交，但期间本地草稿或当前画布已变化；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      const nextCanvas = await loadKnowledgeCanvas(client, canvas.relative_path);
      if (!canCommitCanvasAsyncRequest(request)) {
        if (
          request.currentRelativePath === canvasRelativePathRef.current
          && request.draftRevision !== draftRevisionRef.current
        ) {
          setConflicted(true);
        }
        setNotice("保存后的读取结果已过期；已保留当前草稿，请重新读取后再保存。");
        return;
      }
      replaceCanvas(nextCanvas);
      replaceCanvasDraft(nextCanvas.document);
      setConflicted(false);
      setNotice("已保存并重新读取 JSON Canvas。");
      void refreshCatalog();
      onWorkspaceMutation?.();
    } catch (error) {
      if (isCanvasConflict(error)) {
        setConflicted(true);
        setNotice(knowledgeCanvasConflictNotice());
      } else {
        setNotice("保存没有完成；本地草稿仍保留，未静默覆盖文件。");
      }
    } finally {
      setBusy(null);
    }
  }, [beginCanvasAsyncRequest, canCommitCanvasAsyncRequest, canvas, client, conflicted, draft, onWorkspaceMutation, refreshCatalog, replaceCanvas, replaceCanvasDraft]);

  const reloadCanvas = useCallback(() => {
    if (!canvas) return;
    void openCanvas(canvas.relative_path, { discardDraft: true }).then((reloaded) => {
      if (reloaded) setNotice("已显式重新读取 JSON Canvas；此前草稿没有被自动写回。");
    });
  }, [canvas, openCanvas]);

  const flow = useMemo(
    () => (draft ? canvasDocumentToFlow(draft, selectedNodeId) : { nodes: [], edges: [] }),
    [draft, selectedNodeId],
  );
  const selectedNode = draft && selectedNodeId ? findCanvasNode(draft, selectedNodeId) : null;
  const canvasEntries = canvasEntriesFromSnapshot(snapshot);
  const draftIsDirty = Boolean(canvas && draft && canvasDraftDiffers(draft, canvas.document));
  const canvasStatus = conflicted
    ? "conflict"
    : busy
      ? busy
      : draftIsDirty
        ? "dirty"
        : canvas
          ? "saved"
          : "empty";
  const canvasStatusLabel = conflicted
    ? "冲突 · 草稿已保留"
    : busy === "refresh"
      ? "正在刷新目录"
      : busy === "read"
        ? "正在读取"
        : busy === "create"
          ? "正在新建"
          : busy === "save"
            ? "正在保存"
            : draftIsDirty
              ? "未保存"
              : canvas
                ? "已保存"
                : "未打开";
  const vaultFilePaths = useMemo(
    () => (snapshot?.entries ?? [])
      .filter((entry) => entry.kind === "attachment" || entry.kind === "markdown" || entry.kind === "canvas")
      .map((entry) => entry.relative_path),
    [snapshot],
  );

  useEffect(() => {
    if (!filePanelOpen) return;
    const frame = window.requestAnimationFrame(() => {
      (filePanelFirstActionRef.current ?? newCanvasPathInputRef.current)?.focus();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [canvasEntries.length, filePanelOpen]);

  const closeFilePanel = useCallback((
    focusPolicy: "cancel" | "selection" = "cancel",
    explicitOpener?: HTMLButtonElement,
  ) => {
    const actualOpener = explicitOpener ?? filePanelOpenerRef.current;
    filePanelOpenerRef.current = null;
    setFilePanelOpen(false);
    window.requestAnimationFrame(() => {
      const focusTarget = focusPolicy === "selection"
        ? canvasFilePanelSelectionFocusTarget(canvasStageRef.current, filePanelTriggerRef.current)
        : canvasFilePanelCancelFocusTarget(actualOpener, filePanelTriggerRef.current, canvasStageRef.current);
      focusTarget?.focus();
    });
  }, []);

  const showFilePanel = useCallback((opener: HTMLButtonElement) => {
    filePanelOpenerRef.current = opener;
    setSelectedNodeId(null);
    setFilePanelOpen(true);
  }, []);

  const toggleFilePanel = useCallback((opener: HTMLButtonElement) => {
    if (filePanelOpen) {
      closeFilePanel("cancel", opener);
      return;
    }
    showFilePanel(opener);
  }, [closeFilePanel, filePanelOpen, showFilePanel]);

  const openCanvasFromPanel = useCallback(async (relativePath: string) => {
    const opened = await openCanvas(relativePath);
    if (opened) closeFilePanel("selection");
  }, [closeFilePanel, openCanvas]);

  const createCanvasFromPanel = useCallback(async () => {
    const created = await createCanvas();
    if (created) closeFilePanel("selection");
  }, [closeFilePanel, createCanvas]);

  const closeInspector = useCallback(() => {
    setSelectedNodeId(null);
    window.requestAnimationFrame(() => canvasStageRef.current?.focus());
  }, []);

  const addNode = useCallback(() => {
    if (!draft) return;
    const next = addKnowledgeCanvasNode(draft, createDefaultCanvasNode(newNodeType, canvasNodes(draft).length));
    replaceCanvasDraft(next);
    setSelectedNodeId(canvasNodes(next).at(-1) ? stringField(canvasNodes(next).at(-1)!, "id") : null);
    setNotice("已加入本地 Canvas 草稿；点击保存才会写入固定 vault。");
  }, [draft, newNodeType, replaceCanvasDraft]);

  const removeSelectedNode = useCallback(() => {
    if (!draft || !selectedNodeId) return;
    replaceCanvasDraft(deleteKnowledgeCanvasNode(draft, selectedNodeId));
    setSelectedNodeId(null);
    setNotice("已从本地草稿移除节点及其连线；点击保存才会写入固定 vault。");
  }, [draft, replaceCanvasDraft, selectedNodeId]);

  const addEdge = useCallback(() => {
    if (!draft || !selectedNodeId || !edgeTargetId || selectedNodeId === edgeTargetId) return;
    const next = addKnowledgeCanvasEdge(draft, {
      id: nextCanvasIdentifier("edge", canvasEdges(draft).map((edge) => stringField(edge, "id"))),
      fromNode: selectedNodeId,
      toNode: edgeTargetId,
      fromEnd: "arrow",
    });
    replaceCanvasDraft(next);
    setNotice("已加入本地连线草稿；点击保存才会写入固定 vault。");
  }, [draft, edgeTargetId, replaceCanvasDraft, selectedNodeId]);

  return (
    <section className="native-knowledge-canvas" aria-label="Syn 原生 JSON Canvas">
      <header className="native-canvas-chrome" data-canvas-chrome="compact">
        <button
          ref={filePanelTriggerRef}
          className="native-canvas-file-trigger"
          type="button"
          data-canvas-file-trigger
          data-canvas-file-opener="chrome"
          aria-expanded={filePanelOpen}
          aria-controls="native-canvas-file-panel"
          onClick={(event) => toggleFilePanel(event.currentTarget)}
        >
          画布
        </button>
        <div className="native-canvas-current">
          <span
            className="native-canvas-current__path"
            title={canvas?.relative_path ?? "尚未打开 Canvas"}
            aria-label={`当前 Canvas：${canvas?.relative_path ?? "尚未打开"}`}
          >
            {canvas?.relative_path ?? "尚未打开 Canvas"}
          </span>
          <span data-canvas-status={canvasStatus}>{canvasStatusLabel}</span>
        </div>
        <div className="native-canvas-chrome__actions">
          <button className="primary-button" type="button" onClick={() => void saveDraft()} disabled={!draft || !canvas || conflicted || busy !== null}>保存</button>
          <button className="text-button" type="button" onClick={reloadCanvas} disabled={!canvas || busy !== null}>重新读取</button>
        </div>
      </header>

      <main className="native-canvas-workspace" aria-label="JSON Canvas 编辑区">
        {filePanelOpen ? (
          <aside
            id="native-canvas-file-panel"
            className="native-canvas-file-panel"
            data-canvas-file-panel
            aria-label="Canvas 文件"
            onKeyDown={(event) => {
              if (event.key !== "Escape") return;
              event.preventDefault();
              event.stopPropagation();
              closeFilePanel();
            }}
          >
            <div className="native-canvas-file-panel__head">
              <div>
                <span className="eyebrow">固定 Syn vault</span>
                <strong>画布文件</strong>
              </div>
              <div>
                <button className="text-button" type="button" onClick={refreshWorkspaceManually} disabled={busy !== null}>刷新</button>
                <button className="text-button" type="button" aria-label="关闭 Canvas 文件面板" onClick={() => closeFilePanel()}>关闭</button>
              </div>
            </div>
            {canvasEntries.length ? (
              <ul className="native-canvas-file-list">
                {canvasEntries.map((entry, index) => (
                  <li key={entry.relative_path}>
                    <button
                      ref={index === 0 ? filePanelFirstActionRef : undefined}
                      className={canvas?.relative_path === entry.relative_path ? "is-active" : ""}
                      type="button"
                      title={entry.relative_path}
                      onClick={() => void openCanvasFromPanel(entry.relative_path)}
                    >
                      {entry.relative_path}
                    </button>
                  </li>
                ))}
              </ul>
            ) : <p className="muted small-note">尚无已验证的 .canvas 文件。</p>}
            <label className="native-canvas-create-field">
              <span>新 Canvas 路径</span>
              <input
                ref={newCanvasPathInputRef}
                aria-label="新 Canvas 路径"
                value={newCanvasPath}
                onChange={(event) => setNewCanvasPath(event.target.value)}
              />
            </label>
            <button className="secondary-button" type="button" onClick={() => void createCanvasFromPanel()} disabled={busy !== null}>新建 JSON Canvas</button>
          </aside>
        ) : null}

        <div
          ref={canvasStageRef}
          className="native-canvas-flow-stage"
          data-canvas-stage="continuous"
          tabIndex={0}
          aria-label={canvas ? `JSON Canvas 图形编辑区：${canvas.relative_path}` : "JSON Canvas 图形编辑区：尚未打开"}
        >
          {draft ? (
            <ReactFlowProvider>
              <ReactFlow
                nodes={flow.nodes}
                edges={flow.edges}
                fitView
                minZoom={0.3}
                maxZoom={1.5}
                aria-label={`JSON Canvas：${canvas?.relative_path ?? "本地草稿"}`}
                onNodeClick={(_, node) => {
                  filePanelOpenerRef.current = null;
                  setFilePanelOpen(false);
                  setSelectedNodeId(node.id);
                }}
                onPaneClick={() => setSelectedNodeId(null)}
                onNodeDragStop={(_, node) => updateCanvasDraft((current) => moveKnowledgeCanvasNode(current, node.id, node.position))}
                onConnect={(connection) => {
                  if (!connection.source || !connection.target) return;
                  updateCanvasDraft((current) => addKnowledgeCanvasEdge(current, {
                    id: nextCanvasIdentifier("edge", canvasEdges(current).map((edge) => stringField(edge, "id"))),
                    fromNode: connection.source,
                    toNode: connection.target,
                    fromEnd: "arrow",
                  }));
                }}
                proOptions={{ hideAttribution: true }}
              >
                <Background variant={BackgroundVariant.Dots} gap={24} size={1} />
                <Controls showInteractive aria-label="Canvas 视口控制" />
              </ReactFlow>
            </ReactFlowProvider>
          ) : (
            <div className="native-canvas-empty">
              <strong>打开一张 Canvas</strong>
              <span>文件和链接节点只保留为固定 vault 引用或文本，不会离开 Syn。</span>
              <button
                className="secondary-button"
                type="button"
                data-canvas-file-opener="empty"
                aria-expanded={filePanelOpen}
                aria-controls="native-canvas-file-panel"
                onClick={(event) => showFilePanel(event.currentTarget)}
              >
                选择画布
              </button>
            </div>
          )}
          <div className="native-canvas-floating-tools" role="toolbar" aria-label="Canvas 节点工具">
            <label>
              <span>节点</span>
              <select aria-label="新增 Canvas 节点类型" value={newNodeType} onChange={(event) => setNewNodeType(event.target.value as CanvasNodeType)}>
                <option value="text">文本</option>
                <option value="file">文件</option>
                <option value="link">链接</option>
                <option value="group">分组</option>
              </select>
            </label>
            <button className="secondary-button" type="button" onClick={addNode} disabled={!draft || busy !== null}>加入</button>
          </div>
        </div>

        {selectedNode && draft ? (
          <aside
            id="native-canvas-node-inspector"
            className="native-canvas-inspector"
            data-canvas-inspector
            aria-label={`Canvas 节点属性：${stringField(selectedNode, "id") ?? "未知节点"}`}
            onKeyDown={(event) => {
              if (event.key !== "Escape") return;
              event.preventDefault();
              event.stopPropagation();
              closeInspector();
            }}
          >
            <div className="native-canvas-inspector__head">
              <div>
                <span className="eyebrow">原始 JSON 局部编辑</span>
                <strong>{stringField(selectedNode, "id")}</strong>
              </div>
              <button className="text-button" type="button" aria-label="关闭节点属性" onClick={closeInspector}>关闭</button>
            </div>
            <CanvasNodeInspector
              node={selectedNode}
              allNodeIds={canvasNodes(draft).map((node) => stringField(node, "id")).filter(isString)}
              vaultFilePaths={vaultFilePaths}
              edgeTargetId={edgeTargetId}
              onEdgeTargetChange={setEdgeTargetId}
              onPatch={(patch) => updateCanvasDraft((current) => editKnowledgeCanvasNode(current, stringField(selectedNode, "id") ?? "", patch))}
              onAddEdge={addEdge}
              onDelete={removeSelectedNode}
            />
          </aside>
        ) : null}

        {notice ? <p className={conflicted ? "state-warning native-canvas-notice" : "muted small-note native-canvas-notice"}>{notice}</p> : null}
      </main>
    </section>
  );
}

function CanvasNodeInspector({
  node,
  allNodeIds,
  vaultFilePaths,
  edgeTargetId,
  onEdgeTargetChange,
  onPatch,
  onAddEdge,
  onDelete,
}: {
  node: JsonCanvasObject;
  allNodeIds: string[];
  vaultFilePaths: string[];
  edgeTargetId: string;
  onEdgeTargetChange: (value: string) => void;
  onPatch: (patch: JsonCanvasObject) => void;
  onAddEdge: () => void;
  onDelete: () => void;
}) {
  const id = stringField(node, "id") ?? "";
  const type = stringField(node, "type") as CanvasNodeType | undefined;
  const fileReference = stringField(node, "file") ?? "";
  const fileReferenceOptions = Array.from(new Set([...vaultFilePaths, ...(fileReference ? [fileReference] : [])]));
  const patchText = (field: string, value: string) => onPatch({ [field]: value });
  return (
    <div className="native-canvas-inspector-fields">
      <strong>{id}</strong>
      <span>{canvasNodeTypeLabel(type)}</span>
      {type === "text" ? <label><span>文本</span><textarea aria-label={`节点 ${id} 文本`} value={stringField(node, "text") ?? ""} onChange={(event) => patchText("text", event.target.value)} /></label> : null}
      {type === "file" ? (
        <label>
          <span>固定 vault 文件</span>
          <select aria-label={`节点 ${id} 固定 vault 文件`} value={fileReference} onChange={(event) => patchText("file", event.target.value)}>
            <option value="">选择已验证的相对引用</option>
            {fileReferenceOptions.map((relativePath) => <option key={relativePath} value={relativePath}>{relativePath}</option>)}
          </select>
        </label>
      ) : null}
      {type === "link" ? <label><span>链接文本</span><input aria-label={`节点 ${id} 链接文本`} value={stringField(node, "url") ?? ""} onChange={(event) => patchText("url", event.target.value)} /></label> : null}
      {type === "group" ? <label><span>分组标签</span><input aria-label={`节点 ${id} 分组标签`} value={stringField(node, "label") ?? ""} onChange={(event) => patchText("label", event.target.value)} /></label> : null}
      <div className="native-canvas-coordinate-grid">
        {(["x", "y", "width", "height"] as const).map((field) => (
          <label key={field}>
            <span>{field}</span>
            <input
              aria-label={`节点 ${id} ${field}`}
              type="number"
              value={numberField(node, field) ?? 0}
              onChange={(event) => onPatch({ [field]: Math.round(Number(event.target.value) || 0) })}
            />
          </label>
        ))}
      </div>
      <label>
        <span>连接到</span>
        <select aria-label={`节点 ${id} 连接到`} value={edgeTargetId} onChange={(event) => onEdgeTargetChange(event.target.value)}>
          <option value="">选择现有节点</option>
          {allNodeIds.filter((candidate) => candidate !== id).map((candidate) => <option key={candidate} value={candidate}>{candidate}</option>)}
        </select>
      </label>
      <div className="native-canvas-inspector-actions">
        <button className="secondary-button" type="button" onClick={onAddEdge} disabled={!edgeTargetId}>加入连线</button>
        <button className="text-button danger" type="button" onClick={onDelete}>删除节点</button>
      </div>
    </div>
  );
}

export function canvasDocumentToFlow(document: JsonCanvasObject, selectedNodeId: string | null = null): CanvasFlow {
  const nodes = canvasNodes(document).flatMap((rawNode) => {
    const id = stringField(rawNode, "id");
    const type = stringField(rawNode, "type");
    if (!id || !isCanvasNodeType(type)) return [];
    return [{
      id,
      position: { x: numberField(rawNode, "x") ?? 0, y: numberField(rawNode, "y") ?? 0 },
      data: {
        id,
        type,
        label: canvasNodeLabel(rawNode, type),
        detail: canvasNodeDetail(rawNode, type),
      },
      className: `native-canvas-flow-node native-canvas-flow-node--${type}`,
      style: { width: numberField(rawNode, "width") ?? 220, height: numberField(rawNode, "height") ?? 96 },
      selected: id === selectedNodeId,
      ariaLabel: id === selectedNodeId
        ? `${canvasNodeLabel(rawNode, type)}，已选择，节点属性已打开`
        : `${canvasNodeLabel(rawNode, type)}，选择以编辑节点属性`,
      domAttributes: id === selectedNodeId
        ? { "aria-controls": "native-canvas-node-inspector", "aria-expanded": true }
        : undefined,
    } satisfies CanvasFlowNode];
  });
  const edges = canvasEdges(document).flatMap((rawEdge) => {
    const id = stringField(rawEdge, "id");
    const source = stringField(rawEdge, "fromNode");
    const target = stringField(rawEdge, "toNode");
    if (!id || !source || !target) return [];
    return [{ id, source, target, type: "straight", className: "native-canvas-flow-edge" } satisfies Edge];
  });
  return { nodes, edges };
}

export function moveKnowledgeCanvasNode(
  document: JsonCanvasObject,
  nodeId: string,
  position: Readonly<{ x: number; y: number }>,
): JsonCanvasObject {
  return patchCanvasNode(document, nodeId, { x: Math.round(position.x), y: Math.round(position.y) });
}

export function editKnowledgeCanvasNode(
  document: JsonCanvasObject,
  nodeId: string,
  patch: JsonCanvasObject,
): JsonCanvasObject {
  return patchCanvasNode(document, nodeId, patch);
}

export function addKnowledgeCanvasNode(document: JsonCanvasObject, node: JsonCanvasObject): JsonCanvasObject {
  const id = stringField(node, "id");
  if (!id || canvasNodes(document).some((candidate) => stringField(candidate, "id") === id)) {
    throw new Error("knowledge_workspace_canvas_duplicate_node_id");
  }
  return { ...document, nodes: [...canvasNodes(document), { ...node }] };
}

export function addKnowledgeCanvasEdge(document: JsonCanvasObject, edge: JsonCanvasObject): JsonCanvasObject {
  const id = stringField(edge, "id");
  const fromNode = stringField(edge, "fromNode");
  const toNode = stringField(edge, "toNode");
  const nodeIds = new Set(canvasNodes(document).map((node) => stringField(node, "id")).filter(isString));
  if (!id || canvasEdges(document).some((candidate) => stringField(candidate, "id") === id)) {
    throw new Error("knowledge_workspace_canvas_duplicate_edge_id");
  }
  if (!fromNode || !toNode || !nodeIds.has(fromNode) || !nodeIds.has(toNode)) {
    throw new Error("knowledge_workspace_canvas_dangling_edge");
  }
  return { ...document, edges: [...canvasEdges(document), { ...edge }] };
}

export function deleteKnowledgeCanvasNode(document: JsonCanvasObject, nodeId: string): JsonCanvasObject {
  return {
    ...document,
    nodes: canvasNodes(document).filter((node) => stringField(node, "id") !== nodeId),
    edges: canvasEdges(document).filter(
      (edge) => stringField(edge, "fromNode") !== nodeId && stringField(edge, "toNode") !== nodeId,
    ),
  };
}

function patchCanvasNode(document: JsonCanvasObject, nodeId: string, patch: JsonCanvasObject): JsonCanvasObject {
  let matched = false;
  const nodes = canvasNodes(document).map((node) => {
    if (stringField(node, "id") !== nodeId) return node;
    matched = true;
    return { ...node, ...patch };
  });
  if (!matched) throw new Error("knowledge_workspace_canvas_node_not_found");
  return { ...document, nodes };
}

function createDefaultCanvasNode(type: CanvasNodeType, index: number): JsonCanvasObject {
  const id = `node-${index + 1}`;
  const common = { id, type, x: 80 + (index % 3) * 260, y: 80 + Math.floor(index / 3) * 144, width: 220, height: 96 };
  switch (type) {
    case "text": return { ...common, text: "新的文字笔记" };
    case "file": return { ...common, file: "" };
    case "link": return { ...common, url: "https://" };
    case "group": return { ...common, label: "新的分组" };
  }
}

function canvasEntriesFromSnapshot(snapshot: KnowledgeWorkspaceSnapshot | null) {
  return (snapshot?.entries ?? []).filter((entry) => entry.kind === "canvas");
}

function canvasNodes(document: JsonCanvasObject): JsonCanvasObject[] {
  return Array.isArray(document.nodes) ? document.nodes.filter(isCanvasObject) : [];
}

function canvasEdges(document: JsonCanvasObject): JsonCanvasObject[] {
  return Array.isArray(document.edges) ? document.edges.filter(isCanvasObject) : [];
}

function findCanvasNode(document: JsonCanvasObject, nodeId: string): JsonCanvasObject | null {
  return canvasNodes(document).find((node) => stringField(node, "id") === nodeId) ?? null;
}

function isCanvasObject(value: JsonCanvasValue | undefined): value is JsonCanvasObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringField(object: JsonCanvasObject, field: string): string | null {
  const value = object[field];
  return typeof value === "string" ? value : null;
}

function numberField(object: JsonCanvasObject, field: string): number | null {
  const value = object[field];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function isCanvasNodeType(value: string | null): value is CanvasNodeType {
  return value === "text" || value === "file" || value === "link" || value === "group";
}

function canvasNodeTypeLabel(type: CanvasNodeType | undefined): string {
  return type === "text" ? "文本" : type === "file" ? "文件" : type === "link" ? "链接" : "分组";
}

function canvasNodeLabel(node: JsonCanvasObject, type: CanvasNodeType): string {
  if (type === "text") return stringField(node, "text") || "空白文字";
  if (type === "file") return "受限文件引用";
  if (type === "link") return "链接文本";
  return stringField(node, "label") || "未命名分组";
}

function canvasNodeDetail(node: JsonCanvasObject, type: CanvasNodeType): string {
  if (type === "file") return stringField(node, "file") || "需要选择固定 vault 文件";
  if (type === "link") return stringField(node, "url") || "链接只保存为文本";
  return canvasNodeTypeLabel(type);
}

function nextCanvasIdentifier(prefix: string, ids: Array<string | null>): string {
  const existing = new Set(ids.filter(isString));
  let index = existing.size + 1;
  let candidate = `${prefix}-${index}`;
  while (existing.has(candidate)) {
    index += 1;
    candidate = `${prefix}-${index}`;
  }
  return candidate;
}

function isString(value: string | null): value is string {
  return typeof value === "string";
}

function isCanvasConflict(error: unknown): boolean {
  return error instanceof Error && error.message.includes("knowledge_vault_conflict");
}

function canvasDraftDiffers(left: JsonCanvasObject, right: JsonCanvasObject): boolean {
  try {
    return JSON.stringify(left) !== JSON.stringify(right);
  } catch {
    // The fixed client only returns JSON, but fail closed if a malformed local
    // value ever reaches this comparison: preserve the draft instead of write.
    return true;
  }
}
