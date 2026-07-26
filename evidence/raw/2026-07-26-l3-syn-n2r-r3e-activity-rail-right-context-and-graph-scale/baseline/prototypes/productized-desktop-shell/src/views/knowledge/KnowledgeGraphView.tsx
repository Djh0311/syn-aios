import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import {
  Background,
  BackgroundVariant,
  Controls,
  Handle,
  Position,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import {
  knowledgeWorkspace,
  type KnowledgeWorkspaceGraphNode,
  type KnowledgeWorkspaceGraphOptions,
  type KnowledgeWorkspaceGraphResponse,
} from "../../lib/tauri";

type KnowledgeGraphClient = Readonly<{
  graph: (options: KnowledgeWorkspaceGraphOptions) => Promise<KnowledgeWorkspaceGraphResponse>;
}>;

type KnowledgeGraphRequest = Readonly<{
  scope: "global" | "local";
  focusRelativePath?: string;
  query?: string;
  tag?: string;
}>;

type KnowledgeGraphViewProps = Readonly<{
  client?: KnowledgeGraphClient;
  onOpenMarkdown?: (relativePath: string) => void;
  staticGraph?: KnowledgeWorkspaceGraphResponse | null;
  refreshRequestId?: number;
}>;

type GraphLoadState = "loading" | "ready" | "unavailable";

type KnowledgeGraphNodeData = Readonly<{
  relativePath: string;
  title: string;
  tags: ReadonlyArray<string>;
  isolated: boolean;
  current: boolean;
  onActivate?: () => void;
  onSelect?: () => void;
}>;

type KnowledgeGraphFlowNode = Node<KnowledgeGraphNodeData, "synKnowledgeGraphNode">;

export type KnowledgeGraphOpenRequest = Readonly<{
  relativePath: string;
  sequence: number;
}>;

const knowledgeGraphNodeTypes = {
  synKnowledgeGraphNode: KnowledgeGraphNodeView,
};

// The graph remains a read-only index view. This helper deliberately passes a
// named lower-camel request to the fixed client rather than exposing an invoke
// function, vault root, URL, or any routing surface to the component.
export async function loadKnowledgeGraph(
  client: KnowledgeGraphClient,
  request: KnowledgeGraphRequest,
): Promise<KnowledgeWorkspaceGraphResponse> {
  const query = request.query?.trim();
  const tag = request.tag?.trim();
  return client.graph({
    scope: request.scope,
    ...(request.scope === "local" && request.focusRelativePath ? { focusRelativePath: request.focusRelativePath } : {}),
    ...(query ? { query } : {}),
    ...(tag ? { tag } : {}),
  });
}

// A relationship node can only hand the backend-returned vault-relative path
// to its parent. The parent owns the typed native-document handoff.
export function openKnowledgeGraphNode(
  node: KnowledgeWorkspaceGraphNode,
  onOpenMarkdown?: (relativePath: string) => void,
) {
  onOpenMarkdown?.(node.relative_path);
}

// Repeatedly selecting the same graph node must still request a native read
// after the user has navigated elsewhere. The sequence is UI-only state; the
// path remains the exact graph projection supplied by the fixed client.
export function nextKnowledgeGraphOpenRequest(
  current: KnowledgeGraphOpenRequest | null,
  relativePath: string,
): KnowledgeGraphOpenRequest {
  return { relativePath, sequence: (current?.sequence ?? 0) + 1 };
}

export function KnowledgeGraphView({
  client = knowledgeWorkspace,
  onOpenMarkdown,
  staticGraph = null,
  refreshRequestId = 0,
}: KnowledgeGraphViewProps) {
  // The offline runner renders through React SSR. It must be a meaningful
  // read-only shell and never call the typed client or touch Tauri state.
  if (typeof window === "undefined") return <KnowledgeGraphStaticShell graph={staticGraph} />;
  return <KnowledgeGraphViewBrowser client={client} onOpenMarkdown={onOpenMarkdown} refreshRequestId={refreshRequestId} />;
}

function KnowledgeGraphStaticShell({ graph }: { graph: KnowledgeWorkspaceGraphResponse | null }) {
  const presentation = graph ? buildKnowledgeGraphPresentation(graph) : null;
  return (
    <section className="native-knowledge-graph" aria-label="Syn 原生关系图">
      <div className="native-graph-toolbar native-graph-toolbar--static" aria-label="关系图范围与筛选">
        <div className="native-graph-scope-control" aria-label="关系图范围">
          <span className="sr-only">范围</span>
          <button className="is-active" type="button" aria-pressed="true">全局</button>
          <button type="button" aria-pressed="false">局部</button>
        </div>
        <span className="native-graph-status" aria-live="polite">
          {presentation ? `${presentation.graph.nodes.length} 笔记 · ${presentation.graph.edges.length} 关系` : "只读关系投影"}
        </span>
        <button
          className="native-graph-tool-button"
          type="button"
          aria-expanded="false"
          aria-controls="native-graph-static-filter-panel"
          data-graph-filter-opener
        >
          打开关系图筛选
        </button>
      </div>
      {presentation ? (
        <KnowledgeGraphLedger presentation={presentation} />
      ) : (
        <div className="native-graph-static-stage">
          <strong>关系索引在桌面壳启动后生成</strong>
          <span>只投影固定 Syn vault 中已验证 Markdown 的双链与反链；没有边的笔记也会保留。</span>
        </div>
      )}
    </section>
  );
}

function KnowledgeGraphViewBrowser({
  client,
  onOpenMarkdown,
  refreshRequestId,
}: {
  client: KnowledgeGraphClient;
  onOpenMarkdown?: (relativePath: string) => void;
  refreshRequestId: number;
}) {
  const lastRefreshRequestId = useRef(refreshRequestId);
  const [scope, setScope] = useState<"global" | "local">("global");
  const [focusRelativePath, setFocusRelativePath] = useState("");
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState("");
  const [graph, setGraph] = useState<KnowledgeWorkspaceGraphResponse | null>(null);
  const [loadState, setLoadState] = useState<GraphLoadState>("loading");
  const [notice, setNotice] = useState<string | null>(null);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const filterPanelId = `native-graph-filter-${useId().replace(/:/g, "")}`;
  const filterOpenerRef = useRef<HTMLButtonElement | null>(null);
  const firstFilterRef = useRef<HTMLInputElement | null>(null);
  const stageFallbackRef = useRef<HTMLDivElement | null>(null);

  const requestGraph = useCallback(
    async (request: KnowledgeGraphRequest) => {
      if (request.scope === "local" && !request.focusRelativePath) {
        setNotice("先从当前关系图中选择一条 Markdown 笔记，再查看它的局部邻接关系。");
        return false;
      }
      setLoadState("loading");
      try {
        const nextGraph = await loadKnowledgeGraph(client, request);
        setGraph(nextGraph);
        setLoadState("ready");
        setNotice(null);
        return true;
      } catch {
        setLoadState("unavailable");
        setNotice("关系图暂时读不到；没有改写 Markdown、目录或索引。");
        return false;
      }
    },
    [client],
  );

  useEffect(() => {
    void requestGraph({ scope: "global" });
  }, [requestGraph]);

  const runCurrentRequest = useCallback(() => (
    requestGraph({
      scope,
      ...(scope === "local" && focusRelativePath ? { focusRelativePath } : {}),
      ...(query.trim() ? { query } : {}),
      ...(tag.trim() ? { tag } : {}),
    })
  ), [focusRelativePath, query, requestGraph, scope, tag]);

  useEffect(() => {
    if (refreshRequestId === 0 || refreshRequestId === lastRefreshRequestId.current) return;
    lastRefreshRequestId.current = refreshRequestId;
    void runCurrentRequest();
  }, [refreshRequestId, runCurrentRequest]);

  const presentation = useMemo(() => (graph ? buildKnowledgeGraphPresentation(graph) : null), [graph]);
  const availableFocusNodes = graph?.nodes ?? [];

  const restoreFilterOpener = useCallback(() => {
    window.requestAnimationFrame(() => {
      const opener = filterOpenerRef.current;
      if (opener?.isConnected) {
        opener.focus();
        return;
      }
      stageFallbackRef.current?.focus();
    });
  }, []);

  const closeFilters = useCallback(() => {
    setFiltersOpen(false);
    restoreFilterOpener();
  }, [restoreFilterOpener]);

  const openFilters = useCallback((opener: HTMLButtonElement) => {
    filterOpenerRef.current = opener;
    setFiltersOpen(true);
  }, []);

  useEffect(() => {
    if (!filtersOpen) return;
    const frame = window.requestAnimationFrame(() => firstFilterRef.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, [filtersOpen]);

  const statusText = loadState === "loading"
    ? "正在整理…"
    : presentation
      ? `${presentation.graph.nodes.length} 笔记 · ${presentation.graph.edges.length} 关系`
      : "只读关系投影";

  return (
    <section className="native-knowledge-graph" aria-label="Syn 原生关系图">
      <div className="native-graph-toolbar" aria-label="关系图范围与筛选">
        <div className="native-graph-scope-control" aria-label="关系图范围">
          <span className="sr-only">范围</span>
          <button
            className={scope === "global" ? "is-active" : ""}
            type="button"
            aria-pressed={scope === "global"}
            onClick={() => setScope("global")}
          >
            全局
          </button>
          <button
            className={scope === "local" ? "is-active" : ""}
            type="button"
            aria-pressed={scope === "local"}
            onClick={() => setScope("local")}
          >
            局部
          </button>
        </div>
        <span className="native-graph-status" aria-live="polite">{statusText}</span>
        <button
          className="native-graph-tool-button"
          type="button"
          aria-expanded={filtersOpen}
          aria-controls={filterPanelId}
          data-graph-filter-opener
          onClick={(event) => {
            if (filtersOpen) {
              closeFilters();
              return;
            }
            openFilters(event.currentTarget);
          }}
        >
          筛选
        </button>
        <button
          className="native-graph-tool-button native-graph-refresh"
          type="button"
          aria-label="刷新关系图"
          onClick={() => void runCurrentRequest()}
          disabled={loadState === "loading"}
        >
          刷新
        </button>
      </div>

      {filtersOpen ? (
        <form
          id={filterPanelId}
          className="native-graph-filter-disclosure"
          aria-label="关系图筛选"
          data-graph-filter-panel
          onKeyDown={(event) => {
            if (event.key !== "Escape") return;
            event.preventDefault();
            event.stopPropagation();
            closeFilters();
          }}
          onSubmit={(event) => {
            event.preventDefault();
            void runCurrentRequest().then((completed) => {
              if (completed) closeFilters();
            });
          }}
        >
          <label className="native-graph-filter-field">
            <span>文字</span>
            <input
              ref={firstFilterRef}
              aria-label="关系图文字筛选"
              value={query}
              placeholder="标题、正文或标签"
              onChange={(event) => setQuery(event.target.value)}
            />
          </label>
          <label className="native-graph-filter-field">
            <span>标签</span>
            <input
              aria-label="关系图标签筛选"
              value={tag}
              placeholder="标签"
              onChange={(event) => setTag(event.target.value)}
            />
          </label>
          <label className="native-graph-focus-field">
            <span>局部焦点</span>
            <select
              aria-label="关系图局部焦点"
              value={focusRelativePath}
              onChange={(event) => setFocusRelativePath(event.target.value)}
              disabled={!availableFocusNodes.length}
            >
              <option value="">从当前关系图选择 Markdown</option>
              {availableFocusNodes.map((node) => (
                <option key={node.relative_path} value={node.relative_path}>{node.title} · {node.relative_path}</option>
              ))}
            </select>
          </label>
          <div className="native-graph-filter-actions">
            <button className="native-graph-tool-button" type="submit" disabled={loadState === "loading"}>应用</button>
            <button className="native-graph-tool-button" type="button" onClick={closeFilters}>关闭</button>
          </div>
        </form>
      ) : null}

      <div
        ref={stageFallbackRef}
        className="native-graph-surface"
        tabIndex={-1}
        aria-label="Markdown 关系投影舞台"
      >
        {presentation ? (
          <KnowledgeGraphFlow
            presentation={presentation}
            focusRelativePath={focusRelativePath}
            onOpenMarkdown={onOpenMarkdown}
          />
        ) : null}
        {loadState === "loading" && !presentation ? <p className="muted small-note native-graph-state">正在从已验证 Markdown 关系重建索引…</p> : null}
        {loadState === "unavailable" && !presentation ? <p className="muted small-note native-graph-state">关系索引暂时不可用；不会以其他来源替代。</p> : null}
        {notice ? <p className="state-warning native-graph-notice">{notice}</p> : null}
      </div>
    </section>
  );
}

function KnowledgeGraphFlow({
  presentation,
  focusRelativePath,
  onOpenMarkdown,
}: {
  presentation: KnowledgeGraphPresentation;
  focusRelativePath: string;
  onOpenMarkdown?: (relativePath: string) => void;
}) {
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const flowNodes = useMemo(
    () => graphFlowNodes(presentation, focusRelativePath, selectedNodeId, setSelectedNodeId, onOpenMarkdown),
    [focusRelativePath, onOpenMarkdown, presentation, selectedNodeId],
  );
  const flowEdges = useMemo(() => graphFlowEdges(presentation), [presentation]);
  const projectionKey = `${presentation.graph.scope}:${presentation.graph.focus_relative_path ?? ""}:${presentation.graph.query ?? ""}:${presentation.graph.tag ?? ""}:${presentation.graph.nodes.map((node) => node.id).join("|")}`;
  return (
    <div className="native-graph-flow-stage" aria-label="Markdown 关系投影">
      <ReactFlowProvider>
        <ReactFlow
          key={projectionKey}
          nodes={flowNodes}
          edges={flowEdges}
          nodeTypes={knowledgeGraphNodeTypes}
          nodesDraggable={false}
          nodesConnectable={false}
          nodesFocusable={false}
          edgesFocusable={false}
          elementsSelectable={Boolean(onOpenMarkdown)}
          fitView
          fitViewOptions={{ padding: 0.2, maxZoom: 1 }}
          minZoom={0.35}
          maxZoom={1.4}
          proOptions={{ hideAttribution: true }}
        >
          <Background variant={BackgroundVariant.Dots} gap={28} size={1} />
          <Controls showInteractive={false} />
        </ReactFlow>
      </ReactFlowProvider>
      <KnowledgeGraphDiagnostics presentation={presentation} />
    </div>
  );
}

function KnowledgeGraphLedger({ presentation }: { presentation: KnowledgeGraphPresentation }) {
  return (
    <div className="native-graph-static-ledger" aria-label="关系图静态索引">
      <ul>
        {presentation.graph.nodes.map((node) => (
          <li key={node.relative_path}>
            <strong>{node.title}</strong>
            <span className="sr-only">
              {knowledgeGraphNodeAccessibleLabel(node, presentation.isolatedPaths.has(node.relative_path))}
            </span>
          </li>
        ))}
      </ul>
      <KnowledgeGraphDiagnostics presentation={presentation} />
    </div>
  );
}

function KnowledgeGraphDiagnostics({ presentation }: { presentation: KnowledgeGraphPresentation }) {
  if (!presentation.graph.truncated && !presentation.graph.diagnostics.length) return null;
  return (
    <div className="native-graph-diagnostics" aria-label="关系图范围说明">
      {presentation.graph.truncated ? <strong>已截断：仅显示本阶段上限内的关系。</strong> : null}
      {presentation.graph.diagnostics.map((diagnostic) => <span key={`${diagnostic.code}-${diagnostic.relative_path ?? "global"}`}>{diagnostic.message}</span>)}
    </div>
  );
}

function KnowledgeGraphNodeView({ data }: NodeProps<KnowledgeGraphFlowNode>) {
  return (
    <div
      className={`native-graph-node${data.isolated ? " is-isolated" : ""}${data.current ? " is-current" : ""}`}
      data-graph-relative-path={data.relativePath}
    >
      <Handle className="native-graph-handle" type="target" position={Position.Left} isConnectable={false} />
      <KnowledgeGraphNodeAction
        relativePath={data.relativePath}
        title={data.title}
        tags={data.tags}
        isolated={data.isolated}
        current={data.current}
        onActivate={data.onActivate}
        onSelect={data.onSelect}
      />
      <Handle className="native-graph-handle" type="source" position={Position.Right} isConnectable={false} />
    </div>
  );
}

export function KnowledgeGraphNodeAction({
  relativePath,
  title,
  tags,
  isolated,
  current,
  onActivate,
  onSelect,
}: KnowledgeGraphNodeData) {
  return (
    <button
      className="native-graph-node-button"
      type="button"
      aria-label={knowledgeGraphNodeAccessibleLabel(
        { id: relativePath, relative_path: relativePath, title, tags },
        isolated,
      )}
      aria-current={current ? "page" : undefined}
      data-graph-node-action
      disabled={!onActivate}
      onClick={onActivate}
      onFocus={(event) => {
        if (event.currentTarget.matches(":focus-visible")) onSelect?.();
      }}
    >
      <strong>{title}</strong>
    </button>
  );
}

type KnowledgeGraphPresentation = Readonly<{
  graph: KnowledgeWorkspaceGraphResponse;
  isolatedPaths: ReadonlySet<string>;
  isolatedNodeCount: number;
}>;

function buildKnowledgeGraphPresentation(graph: KnowledgeWorkspaceGraphResponse): KnowledgeGraphPresentation {
  const connectedPaths = new Set<string>();
  for (const edge of graph.edges) {
    connectedPaths.add(edge.source);
    connectedPaths.add(edge.target);
  }
  const isolatedPaths = new Set(graph.nodes.filter((node) => !connectedPaths.has(node.id)).map((node) => node.relative_path));
  return { graph, isolatedPaths, isolatedNodeCount: isolatedPaths.size };
}

export function knowledgeGraphNodeAccessibleLabel(node: KnowledgeWorkspaceGraphNode, isolated: boolean): string {
  const tags = node.tags.length ? ` 标签 ${node.tags.map((tag) => `#${tag}`).join("、")}。` : "";
  const isolatedDescription = isolated ? " 独立笔记。" : "";
  return `打开笔记：${node.title}。路径 ${node.relative_path}。${tags}${isolatedDescription}`.replace(/\s+/g, " ").trim();
}

export function deterministicKnowledgeGraphPosition(index: number, total: number): { x: number; y: number } {
  if (total <= 1) return { x: 0, y: 0 };
  const angle = -Math.PI / 2 + (Math.PI * 2 * index) / total;
  return {
    x: Math.round((Math.cos(angle) + 1) * 110),
    y: Math.round((Math.sin(angle) + 1) * 160),
  };
}

function graphFlowNodes(
  presentation: KnowledgeGraphPresentation,
  focusRelativePath: string,
  selectedNodeId: string | null,
  onSelectNode: (nodeId: string) => void,
  onOpenMarkdown?: (relativePath: string) => void,
): KnowledgeGraphFlowNode[] {
  return presentation.graph.nodes.map((node, index) => ({
    id: node.id,
    type: "synKnowledgeGraphNode",
    selected: node.id === selectedNodeId,
    position: deterministicKnowledgeGraphPosition(index, presentation.graph.nodes.length),
    data: {
      relativePath: node.relative_path,
      title: node.title,
      tags: node.tags,
      isolated: presentation.isolatedPaths.has(node.relative_path),
      current: node.relative_path === focusRelativePath,
      onSelect: () => onSelectNode(node.id),
      ...(onOpenMarkdown ? { onActivate: () => openKnowledgeGraphNode(node, onOpenMarkdown) } : {}),
    },
  }));
}

function graphFlowEdges(presentation: KnowledgeGraphPresentation): Edge[] {
  return presentation.graph.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    type: "straight",
    className: "native-graph-edge",
  }));
}
