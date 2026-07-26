import { useEffect, useState, type ReactNode } from "react";
import ReactDOM from "react-dom/client";
import { mockIPC } from "@tauri-apps/api/mocks";
import {
  createKnowledgeWorkspaceClient,
  type JsonCanvasObject,
  type KnowledgeWorkspaceCanvasDocument,
  type KnowledgeWorkspaceEntry,
  type KnowledgeWorkspaceGraphResponse,
  type KnowledgeWorkspaceInvoke,
  type KnowledgeWorkspaceMarkdownDocument,
  type KnowledgeWorkspaceSearchResponse,
  type KnowledgeWorkspaceSnapshot,
} from "../src/lib/tauri";
import { NativeKnowledgeWorkspace } from "../src/views/knowledge/NativeKnowledgeWorkspace";
import "../src/styles.css";

const fixtureMarkdownPath = "notes/visual-baseline.md";
const fixtureCanvasPath = "canvas/visual-baseline.canvas";
const fixtureMtime = 1_784_995_200_000;
const fixtureHash = "d4b5a9a184919f9ca89db723c82c901beff2aa2e9f7ddd49ac6a0a8ea487497e";
const localStorageEmptyBeforeMount = window.localStorage.length === 0;
const localStorageWrites: Array<Readonly<{ key: string; value: string }>> = [];
const nativeStorageSetItem = Storage.prototype.setItem;
Storage.prototype.setItem = function recordSyntheticPreferenceWrite(key: string, value: string) {
  if (this === window.localStorage) localStorageWrites.push({ key, value });
  nativeStorageSetItem.call(this, key, value);
};

const longSyntheticParagraph = "这是一段只用于隔离浏览器视觉量尺的合成知识内容。它描述工作台布局、焦点和滚动，不包含真实 vault、用户数据或外部路径。";

function syntheticMarkdownDocument(relativePath: string, title: string, index: number): KnowledgeWorkspaceMarkdownDocument {
  return {
    relative_path: relativePath,
    title,
    body: [
      `# ${title}`,
      "这份 Markdown 只在 N2R-R3B 的 fresh browser context 中存在。",
      ...Array.from({ length: 36 }, (_, paragraphIndex) => (
        `## 合成段落 ${paragraphIndex + 1}\n${longSyntheticParagraph} 编号 ${index + 1}-${paragraphIndex + 1}。`
      )),
    ].join("\n\n"),
    tags: ["visual-baseline", "synthetic"],
    aliases: [`基线 ${index + 1}`],
    properties: { source: "synthetic-fixture", ordinal: String(index + 1) },
    outlinks: index % 2 === 0 ? ["notes/layout-convergence.md"] : [fixtureMarkdownPath],
    backlinks: index % 3 === 0 ? ["notes/layout-convergence.md"] : [],
    mtime_ms: fixtureMtime + index,
    content_hash: fixtureHash,
  };
}

const syntheticMarkdownPaths = [
  fixtureMarkdownPath,
  "notes/layout-convergence.md",
  "notes/focus-return.md",
  "research/graph-projection.md",
  "research/scroll-audit.md",
  "research/overlay-safety.md",
  "sources/synthetic-source.md",
  ...Array.from({ length: 34 }, (_, index) => `archive/visual-entry-${String(index + 1).padStart(2, "0")}.md`),
] as const;

const syntheticMarkdownDocuments = Object.freeze(Object.fromEntries(
  syntheticMarkdownPaths.map((relativePath, index) => [
    relativePath,
    syntheticMarkdownDocument(
      relativePath,
      index === 0 ? "视觉基线工作台" : `合成知识条目 ${String(index + 1).padStart(2, "0")}`,
      index,
    ),
  ]),
) as Record<string, KnowledgeWorkspaceMarkdownDocument>);

function syntheticMarkdownEntry(relativePath: string, index: number): KnowledgeWorkspaceEntry {
  const document = syntheticMarkdownDocuments[relativePath]!;
  return {
    relative_path: relativePath,
    parent_path: relativePath.includes("/") ? relativePath.slice(0, relativePath.lastIndexOf("/")) : null,
    kind: "markdown",
    title: document.title,
    tags: document.tags,
    aliases: document.aliases,
    properties: document.properties,
    mtime_ms: document.mtime_ms,
    size_bytes: 1_024 + index * 17,
    outlinks: document.outlinks,
    backlinks: document.backlinks,
  };
}

function syntheticDirectoryEntry(relativePath: string): KnowledgeWorkspaceEntry {
  return {
    relative_path: relativePath,
    parent_path: null,
    kind: "directory",
    title: null,
    tags: [],
    aliases: [],
    properties: {},
    mtime_ms: fixtureMtime,
    size_bytes: 0,
    outlinks: [],
    backlinks: [],
  };
}

function syntheticCanvasEntry(): KnowledgeWorkspaceEntry {
  return {
    relative_path: fixtureCanvasPath,
    parent_path: "canvas",
    kind: "canvas",
    title: "视觉基线画布",
    tags: ["visual-baseline", "synthetic"],
    aliases: [],
    properties: { source: "synthetic-fixture" },
    mtime_ms: fixtureMtime,
    size_bytes: 512,
    outlinks: [],
    backlinks: [],
  };
}

const syntheticSnapshot: KnowledgeWorkspaceSnapshot = Object.freeze({
  entries: Object.freeze([
    syntheticDirectoryEntry("notes"),
    syntheticDirectoryEntry("research"),
    syntheticDirectoryEntry("sources"),
    syntheticDirectoryEntry("archive"),
    syntheticDirectoryEntry("canvas"),
    ...syntheticMarkdownPaths.map(syntheticMarkdownEntry),
    syntheticCanvasEntry(),
  ]),
  tags: Object.freeze([
    { tag: "visual-baseline", note_count: syntheticMarkdownPaths.length },
    { tag: "synthetic", note_count: syntheticMarkdownPaths.length },
  ]),
  diagnostics: Object.freeze([]),
});

const syntheticGraph: KnowledgeWorkspaceGraphResponse = Object.freeze({
  scope: "global",
  focus_relative_path: null,
  query: null,
  tag: null,
  nodes: Object.freeze(syntheticMarkdownPaths.slice(0, 6).map((relativePath) => ({
    id: relativePath,
    relative_path: relativePath,
    title: syntheticMarkdownDocuments[relativePath]!.title,
    tags: ["visual-baseline", "synthetic"],
  }))),
  edges: Object.freeze([
    { id: "visual-layout", source: fixtureMarkdownPath, target: "notes/layout-convergence.md" },
    { id: "layout-focus", source: "notes/layout-convergence.md", target: "notes/focus-return.md" },
    { id: "focus-graph", source: "notes/focus-return.md", target: "research/graph-projection.md" },
    { id: "graph-scroll", source: "research/graph-projection.md", target: "research/scroll-audit.md" },
    { id: "scroll-overlay", source: "research/scroll-audit.md", target: "research/overlay-safety.md" },
  ]),
  diagnostics: Object.freeze([]),
  truncated: false,
});

const syntheticCanvasDocument: JsonCanvasObject = Object.freeze({
  nodes: [
    { id: "baseline", type: "text", text: "合成视觉基线", x: 40, y: 48, width: 190, height: 96 },
    { id: "focus", type: "text", text: "键盘焦点与回焦", x: 320, y: 48, width: 190, height: 96 },
    { id: "scroll", type: "text", text: "内部滚动量尺", x: 180, y: 230, width: 190, height: 96 },
  ],
  edges: [
    { id: "baseline-focus", fromNode: "baseline", toNode: "focus", fromEnd: "arrow" },
    { id: "focus-scroll", fromNode: "focus", toNode: "scroll", fromEnd: "arrow" },
  ],
});

const syntheticCanvas: KnowledgeWorkspaceCanvasDocument = Object.freeze({
  relative_path: fixtureCanvasPath,
  document: syntheticCanvasDocument,
  mtime_ms: fixtureMtime,
  content_hash: fixtureHash,
  diagnostics: Object.freeze([]),
});

const allowedReadCommands = new Set([
  "knowledge_workspace_snapshot",
  "knowledge_workspace_search",
  "knowledge_workspace_graph",
  "knowledge_workspace_read_markdown",
  "knowledge_workspace_read_canvas",
]);

const writeCommands = new Set([
  "knowledge_workspace_create_directory",
  "knowledge_workspace_create_markdown",
  "knowledge_workspace_write_markdown",
  "knowledge_workspace_create_canvas",
  "knowledge_workspace_write_canvas",
  "knowledge_workspace_import_attachment",
  "knowledge_workspace_move_entry",
  "knowledge_workspace_rename_entry",
  "knowledge_workspace_delete_entry",
  "knowledge_workspace_create_recovery_backup",
  "knowledge_workspace_restore_recovery_backup",
]);

const callsByCommand = new Map<string, number>();
const unknownCommandNames = new Set<string>();
let writeCallCount = 0;
let unrecognizedCallCount = 0;

function recordCommand(command: string) {
  callsByCommand.set(command, (callsByCommand.get(command) ?? 0) + 1);
}

function requestedRelativePath(payload: unknown): string {
  if (typeof payload !== "object" || payload === null) return "";
  const candidate = (payload as { relativePath?: unknown }).relativePath;
  return typeof candidate === "string" ? candidate : "";
}

function requestedSearchQuery(payload: unknown): string {
  if (typeof payload !== "object" || payload === null) return "";
  const candidate = (payload as { query?: unknown }).query;
  return typeof candidate === "string" ? candidate : "";
}

function syntheticSearch(query: string): KnowledgeWorkspaceSearchResponse {
  const normalized = query.trim().toLowerCase();
  const results = Object.values(syntheticMarkdownDocuments)
    .filter((document) => !normalized || `${document.title} ${document.relative_path}`.toLowerCase().includes(normalized))
    .slice(0, 16)
    .map((document) => ({
      relative_path: document.relative_path,
      title: document.title,
      snippet: "合成搜索摘要：仅用于视觉基线与布局量尺。",
      tags: document.tags,
      mtime_ms: document.mtime_ms,
    }));
  return { query, results, diagnostics: [] };
}

function syntheticReply(command: string, payload?: unknown): unknown {
  recordCommand(command);
  if (writeCommands.has(command)) {
    writeCallCount += 1;
    throw new Error(`synthetic_fixture_write_command_rejected:${command}`);
  }
  if (!allowedReadCommands.has(command)) {
    unrecognizedCallCount += 1;
    unknownCommandNames.add(command);
    throw new Error(`synthetic_fixture_unrecognized_command_rejected:${command}`);
  }
  switch (command) {
    case "knowledge_workspace_snapshot":
      return syntheticSnapshot;
    case "knowledge_workspace_search":
      return syntheticSearch(requestedSearchQuery(payload));
    case "knowledge_workspace_graph":
      return syntheticGraph;
    case "knowledge_workspace_read_markdown": {
      const relativePath = requestedRelativePath(payload);
      const document = syntheticMarkdownDocuments[relativePath];
      if (!document) throw new Error(`synthetic_fixture_unknown_markdown:${relativePath}`);
      return document;
    }
    case "knowledge_workspace_read_canvas":
      if (requestedRelativePath(payload) !== fixtureCanvasPath) throw new Error("synthetic_fixture_unknown_canvas");
      return syntheticCanvas;
    default:
      throw new Error(`synthetic_fixture_unreachable_command:${command}`);
  }
}

mockIPC((command, payload) => syntheticReply(command, payload));

const syntheticInvoke: KnowledgeWorkspaceInvoke = async (command, args) => syntheticReply(command, args) as never;
const syntheticClient = createKnowledgeWorkspaceClient(syntheticInvoke);

type Bounds = Readonly<{ x: number; y: number; width: number; height: number }>;
type ScrollMetric = Readonly<{
  clientHeight: number;
  clientWidth: number;
  scrollHeight: number;
  scrollWidth: number;
  canScrollY: boolean;
  movedOnProbe: boolean;
}>;

function roundedBounds(element: Element | null): Bounds | null {
  if (!element) return null;
  const bounds = element.getBoundingClientRect();
  return { x: Math.round(bounds.x), y: Math.round(bounds.y), width: Math.round(bounds.width), height: Math.round(bounds.height) };
}

function scrollMetric(element: HTMLElement | null): ScrollMetric | null {
  if (!element) return null;
  const initialScrollTop = element.scrollTop;
  const canScrollY = element.scrollHeight > element.clientHeight;
  if (canScrollY) element.scrollTop = Math.min(48, element.scrollHeight - element.clientHeight);
  const movedOnProbe = element.scrollTop > initialScrollTop;
  element.scrollTop = initialScrollTop;
  return {
    clientHeight: element.clientHeight,
    clientWidth: element.clientWidth,
    scrollHeight: element.scrollHeight,
    scrollWidth: element.scrollWidth,
    canScrollY,
    movedOnProbe,
  };
}

function regionState(selector: string) {
  const region = document.querySelector<HTMLElement>(selector);
  return {
    bounds: roundedBounds(region),
    ariaHidden: region?.getAttribute("aria-hidden") ?? null,
    inert: region?.hasAttribute("inert") ?? false,
    interactiveChildren: region?.querySelectorAll("button, input, textarea, select, [tabindex]").length ?? 0,
    scroll: scrollMetric(region),
  };
}

function parsedStorageValue(value: string | undefined): unknown {
  if (!value) return null;
  try {
    return JSON.parse(value);
  } catch {
    return "invalid-json";
  }
}

function centralGroupState() {
  return [...document.querySelectorAll<HTMLElement>("[data-knowledge-tab-group]")].map((group) => {
    const tabs = [...group.querySelectorAll<HTMLElement>('[role="tab"]')];
    const activeTab = tabs.find((tab) => tab.getAttribute("aria-selected") === "true") ?? null;
    const panel = group.querySelector<HTMLElement>('[role="tabpanel"]:not([hidden])');
    return {
      id: group.dataset.knowledgeTabGroup ?? null,
      active: group.dataset.activeGroup === "true",
      bounds: roundedBounds(group),
      tablistCount: group.querySelectorAll("[data-knowledge-group-tablist]").length,
      tabCount: tabs.length,
      tabs: tabs.map((tab) => ({
        id: tab.id,
        controls: tab.getAttribute("aria-controls"),
        controlsExists: Boolean(document.getElementById(tab.getAttribute("aria-controls") ?? "")),
        selected: tab.getAttribute("aria-selected"),
        tabIndex: tab.tabIndex,
        ariaLabel: tab.getAttribute("aria-label"),
        title: tab.getAttribute("title"),
        text: tab.textContent?.trim() ?? "",
      })),
      activeTabId: activeTab?.id ?? null,
      panelId: panel?.id ?? null,
      panelLabelledBy: panel?.getAttribute("aria-labelledby") ?? null,
      panelContainsGraph: Boolean(panel?.querySelector(".native-knowledge-graph")),
      panelContainsCanvas: Boolean(panel?.querySelector(".native-knowledge-canvas")),
      scroll: scrollMetric(panel),
    };
  });
}

function containedBy(outer: Bounds | null, inner: Bounds | null): boolean | null {
  if (!outer || !inner) return null;
  return (
    inner.x >= outer.x
    && inner.y >= outer.y
    && inner.x + inner.width <= outer.x + outer.width
    && inner.y + inner.height <= outer.y + outer.height
  );
}

function canvasState() {
  const root = document.querySelector<HTMLElement>(".native-knowledge-canvas");
  const stage = root?.querySelector<HTMLElement>('[data-canvas-stage="continuous"]') ?? null;
  const chrome = root?.querySelector<HTMLElement>('[data-canvas-chrome="compact"]') ?? null;
  const fileTrigger = root?.querySelector<HTMLButtonElement>("[data-canvas-file-trigger]") ?? null;
  const filePanel = root?.querySelector<HTMLElement>("[data-canvas-file-panel]") ?? null;
  const inspector = root?.querySelector<HTMLElement>("[data-canvas-inspector]") ?? null;
  const floatingTools = stage?.querySelector<HTMLElement>(".native-canvas-floating-tools") ?? null;
  const selectedNode = stage?.querySelector<HTMLElement>('.react-flow__node[aria-controls="native-canvas-node-inspector"]') ?? null;
  const rootBounds = roundedBounds(root);
  const stageBounds = roundedBounds(stage);
  const filePanelBounds = roundedBounds(filePanel);
  const inspectorBounds = roundedBounds(inspector);
  const floatingToolsBounds = roundedBounds(floatingTools);
  return {
    rootCount: document.querySelectorAll(".native-knowledge-canvas").length,
    reactFlowCount: root?.querySelectorAll(".react-flow").length ?? 0,
    chrome: {
      count: root?.querySelectorAll('[data-canvas-chrome="compact"]').length ?? 0,
      bounds: roundedBounds(chrome),
      currentPathLabel: root?.querySelector(".native-canvas-current__path")?.getAttribute("aria-label") ?? null,
      status: root?.querySelector("[data-canvas-status]")?.getAttribute("data-canvas-status") ?? null,
    },
    root: {
      bounds: rootBounds,
      horizontalOverflow: root ? root.scrollWidth > root.clientWidth : null,
      scroll: scrollMetric(root),
    },
    stage: {
      bounds: stageBounds,
      heightRatio: rootBounds && stageBounds && rootBounds.height > 0 ? stageBounds.height / rootBounds.height : null,
      horizontalOverflow: stage ? stage.scrollWidth > stage.clientWidth : null,
      ariaLabel: stage?.getAttribute("aria-label") ?? null,
    },
    fileTrigger: {
      expanded: fileTrigger?.getAttribute("aria-expanded") ?? null,
      controls: fileTrigger?.getAttribute("aria-controls") ?? null,
      controlsExists: Boolean(fileTrigger?.getAttribute("aria-controls") && document.getElementById(fileTrigger.getAttribute("aria-controls")!)),
    },
    filePanel: {
      count: root?.querySelectorAll("[data-canvas-file-panel]").length ?? 0,
      bounds: filePanelBounds,
      withinRoot: containedBy(rootBounds, filePanelBounds),
      position: filePanel ? window.getComputedStyle(filePanel).position : null,
      interactiveChildren: filePanel?.querySelectorAll("button, input, textarea, select, [tabindex]").length ?? 0,
    },
    inspector: {
      count: root?.querySelectorAll("[data-canvas-inspector]").length ?? 0,
      bounds: inspectorBounds,
      withinRoot: containedBy(rootBounds, inspectorBounds),
      position: inspector ? window.getComputedStyle(inspector).position : null,
      interactiveChildren: inspector?.querySelectorAll("button, input, textarea, select, [tabindex]").length ?? 0,
    },
    selectedNode: {
      id: selectedNode?.dataset.id ?? null,
      controls: selectedNode?.getAttribute("aria-controls") ?? null,
      controlsExists: Boolean(selectedNode?.getAttribute("aria-controls") && document.getElementById(selectedNode.getAttribute("aria-controls")!)),
      expanded: selectedNode?.getAttribute("aria-expanded") ?? null,
    },
    floatingTools: {
      count: stage?.querySelectorAll(".native-canvas-floating-tools").length ?? 0,
      bounds: floatingToolsBounds,
      withinStage: containedBy(stageBounds, floatingToolsBounds),
      ariaLabel: floatingTools?.getAttribute("aria-label") ?? null,
    },
  };
}

function FixtureMetrics() {
  const [metrics, setMetrics] = useState<Record<string, unknown> | null>(null);

  useEffect(() => {
    const collect = () => {
      const shell = document.querySelector<HTMLElement>('[data-knowledge-shell="syn-workbench"]');
      const active = document.activeElement as HTMLElement | null;
      const latestLocalStorageWrite = localStorageWrites.at(-1);
      const separator = document.querySelector<HTMLElement>('[role="separator"][aria-orientation="vertical"]');
      setMetrics({
        fixture: {
          dataOrigin: "synthetic-only",
          localStorageEmptyBeforeMount,
          scenario: document.documentElement.dataset.fixtureScenario ?? "initial",
        },
        bounds: {
          activity: roundedBounds(document.querySelector('[data-knowledge-region="activity"]')),
          left: roundedBounds(document.querySelector('[data-knowledge-region="left"]')),
          central: roundedBounds(document.querySelector('[data-knowledge-region="central"]')),
          right: roundedBounds(document.querySelector('[data-knowledge-region="right"]')),
          status: roundedBounds(document.querySelector('[data-knowledge-region="status"]')),
        },
        regions: {
          activity: regionState('[data-knowledge-region="activity"]'),
          left: regionState('[data-knowledge-region="left"]'),
          central: regionState('[data-knowledge-region="central"]'),
          right: regionState('[data-knowledge-region="right"]'),
          status: regionState('[data-knowledge-region="status"]'),
        },
        overflow: {
          documentElement: scrollMetric(document.documentElement),
          body: scrollMetric(document.body),
          shell: scrollMetric(shell),
          centralSurface: scrollMetric(document.querySelector<HTMLElement>(".knowledge-workbench-central")),
          activeGroupPanel: scrollMetric(document.querySelector<HTMLElement>('[data-active-group="true"] [data-knowledge-group-panel="active"]')),
          overlay: scrollMetric(document.querySelector<HTMLElement>(".syn-knowledge-overlay")),
        },
        central: {
          groupCount: document.querySelectorAll("[data-knowledge-tab-group]").length,
          tablistCount: document.querySelectorAll("[data-knowledge-group-tablist]").length,
          textareaCount: document.querySelectorAll(".knowledge-workbench-central textarea").length,
          saveActionCount: [...document.querySelectorAll<HTMLButtonElement>(".knowledge-workbench-central button")]
            .filter((button) => button.textContent?.trim() === "保存 Markdown").length,
          groups: centralGroupState(),
          separator: separator ? {
            bounds: roundedBounds(separator),
            min: separator.getAttribute("aria-valuemin"),
            max: separator.getAttribute("aria-valuemax"),
            now: separator.getAttribute("aria-valuenow"),
            orientation: separator.getAttribute("aria-orientation"),
          } : null,
        },
        canvas: canvasState(),
        focus: {
          tag: active?.tagName.toLowerCase() ?? null,
          id: active?.id ?? null,
          ariaLabel: active?.getAttribute("aria-label") ?? null,
          role: active?.getAttribute("role") ?? null,
          region: active?.closest("[data-knowledge-region]")?.getAttribute("data-knowledge-region") ?? null,
        },
        localStorage: {
          emptyBeforeMount: localStorageEmptyBeforeMount,
          writeCount: localStorageWrites.length,
          keys: [...new Set(localStorageWrites.map((entry) => entry.key))],
          latestKey: latestLocalStorageWrite?.key ?? null,
          latestNormalizedContent: parsedStorageValue(latestLocalStorageWrite?.value),
        },
        mock: {
          allowedReadCommands: [...allowedReadCommands].sort(),
          callsByCommand: Object.fromEntries([...callsByCommand.entries()].sort(([left], [right]) => left.localeCompare(right))),
          writeCallCount,
          unrecognizedCallCount,
          unrecognizedCommandNames: [...unknownCommandNames].sort(),
        },
      });
      document.documentElement.dataset.fixtureReady = "true";
    };
    const frame = window.requestAnimationFrame(collect);
    const interval = window.setInterval(collect, 150);
    window.addEventListener("resize", collect);
    window.addEventListener("n2r-r3b-capture", collect);
    window.addEventListener("n2r-r3c-capture", collect);
    return () => {
      window.cancelAnimationFrame(frame);
      window.clearInterval(interval);
      window.removeEventListener("resize", collect);
      window.removeEventListener("n2r-r3b-capture", collect);
      window.removeEventListener("n2r-r3c-capture", collect);
    };
  }, []);

  return <pre hidden id="knowledge-workbench-visual-metrics">{metrics ? JSON.stringify(metrics) : "pending"}</pre>;
}

function SyntheticSourceSidebar(): ReactNode {
  return (
    <section className="syn-knowledge-source-panel" aria-label="合成来源侧栏">
      <p className="eyebrow">合成来源</p>
      <strong>仅量尺，不连接真实来源</strong>
      <div className="knowledge-document-detail">
        {Array.from({ length: 24 }, (_, index) => <span key={index}>合成来源条目 {String(index + 1).padStart(2, "0")}</span>)}
      </div>
    </section>
  );
}

function SyntheticSourceContext({ selectedRelativePath }: { selectedRelativePath: string | null }): ReactNode {
  return (
    <section className="syn-knowledge-source-panel" aria-label="合成来源上下文">
      <p className="eyebrow">合成来源上下文</p>
      <strong>{selectedRelativePath ?? "尚未选择合成笔记"}</strong>
      <div className="knowledge-document-detail">
        {Array.from({ length: 28 }, (_, index) => <span key={index}>上下文量尺 {String(index + 1).padStart(2, "0")}</span>)}
      </div>
    </section>
  );
}

const root = document.getElementById("root");
if (!root) throw new Error("knowledge workbench visual fixture root is missing");

ReactDOM.createRoot(root).render(
  <div id="fixture-shell">
    <main id="fixture-stage">
      <NativeKnowledgeWorkspace
        client={syntheticClient}
        sourceSidebar={<SyntheticSourceSidebar />}
        sourceContext={(selectedRelativePath) => <SyntheticSourceContext selectedRelativePath={selectedRelativePath} />}
        statusContent={<span>isolated synthetic browser baseline</span>}
      />
    </main>
    <FixtureMetrics />
  </div>,
);
