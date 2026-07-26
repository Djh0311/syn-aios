import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  KnowledgeCanvasView,
  addKnowledgeCanvasEdge,
  addKnowledgeCanvasNode,
  canvasDocumentToFlow,
  canvasFilePanelCancelFocusTarget,
  canvasFilePanelSelectionFocusTarget,
  deleteKnowledgeCanvasNode,
  editKnowledgeCanvasNode,
  knowledgeCanvasConflictNotice,
  loadKnowledgeCanvas,
  moveKnowledgeCanvasNode,
  saveKnowledgeCanvas,
  createKnowledgeCanvas,
} from "../src/views/knowledge/KnowledgeCanvasView";
import {
  knowledgeWorkspaceDraftNavigationDisposition,
  knowledgeWorkspaceDraftRefreshDisposition,
  knowledgeWorkspaceAsyncCommitDisposition,
} from "../src/lib/knowledgeWorkspace";
import type {
  JsonCanvasObject,
  KnowledgeWorkspaceCanvasDocument,
  KnowledgeWorkspaceMutationResult,
  KnowledgeWorkspaceSnapshot,
} from "../src/lib/tauri";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[knowledge-canvas] ${message}`);
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

function occurrenceCount(value: string, needle: string): number {
  return value.split(needle).length - 1;
}

const hashA = "a".repeat(64);
const canvasDocument: JsonCanvasObject = {
  nodes: [
    {
      id: "field-note",
      type: "text",
      text: "田野笔记",
      x: 12,
      y: 24,
      width: 240,
      height: 96,
      future_text_field: { retained: true },
    },
    {
      id: "reference",
      type: "file",
      file: "research/plan.md",
      x: 300,
      y: 24,
      width: 240,
      height: 96,
      future_file_field: ["keep"],
    },
    {
      id: "source-link",
      type: "link",
      url: "https://example.test/reference",
      x: 12,
      y: 160,
      width: 240,
      height: 96,
    },
    {
      id: "context",
      type: "group",
      label: "背景",
      x: -16,
      y: -16,
      width: 600,
      height: 320,
      future_group_field: "retain",
    },
  ],
  edges: [
    {
      id: "field-to-reference",
      fromNode: "field-note",
      toNode: "reference",
      fromEnd: "arrow",
      future_edge_field: "retain",
    },
  ],
  future_root_field: { retained: ["without", "loss"] },
};

const canvasSnapshot: KnowledgeWorkspaceSnapshot = {
  entries: [
    {
      relative_path: "research/field.canvas",
      parent_path: "research",
      kind: "canvas",
      title: null,
      tags: [],
      aliases: [],
      properties: {},
      mtime_ms: 123,
      size_bytes: 400,
      outlinks: [],
      backlinks: [],
    },
    {
      relative_path: "research/plan.md",
      parent_path: "research",
      kind: "markdown",
      title: "计划",
      tags: [],
      aliases: [],
      properties: {},
      mtime_ms: 123,
      size_bytes: 80,
      outlinks: [],
      backlinks: [],
    },
  ],
  tags: [],
  diagnostics: [],
};

const canvasFile: KnowledgeWorkspaceCanvasDocument = {
  relative_path: "research/field.canvas",
  document: canvasDocument,
  mtime_ms: 123,
  content_hash: hashA,
  diagnostics: [],
};

// SSR only receives supplied static data. It must not call the typed client,
// Tauri, a vault, or expose a workflow/external-opening surface.
let staticCalls = 0;
const staticClient = {
  snapshot: async () => {
    staticCalls += 1;
    return canvasSnapshot;
  },
  readCanvas: async () => {
    staticCalls += 1;
    return canvasFile;
  },
  createCanvas: async () => {
    staticCalls += 1;
    return canvasMutation("canvas_created");
  },
  writeCanvas: async () => {
    staticCalls += 1;
    return canvasMutation("canvas_updated");
  },
};
const staticMarkup = renderToStaticMarkup(
  <KnowledgeCanvasView client={staticClient} staticSnapshot={canvasSnapshot} staticCanvas={canvasFile} />,
);

assert(staticCalls === 0, "SSR Canvas 壳不得调用 Tauri 或读取 vault");
assert(staticMarkup.includes("Syn 原生 JSON Canvas"), "静态壳必须明确是 Syn 原生 Canvas");
assert(staticMarkup.includes("research/field.canvas"), "静态壳必须列出固定 vault 内的 Canvas 条目");
assert(staticMarkup.includes("文本") && staticMarkup.includes("文件") && staticMarkup.includes("链接") && staticMarkup.includes("分组"), "静态壳必须说明四类 JSON Canvas 节点");
assert(staticMarkup.includes("本地草稿") && staticMarkup.includes("重新读取"), "静态壳必须说明冲突时保留草稿并显式重读");
assert(!staticMarkup.includes("工作流") && !staticMarkup.includes("外部打开"), "Canvas 不得成为工作流或外部打开入口");

const r3cStaticContracts = [
  ["static shell has exactly one Canvas root", occurrenceCount(staticMarkup, 'class="native-knowledge-canvas"') === 1],
  ["compact chrome replaces the page header", staticMarkup.includes('data-canvas-chrome="compact"')],
  ["page-level Canvas heading is retired", !staticMarkup.includes("<h2")],
  ["legacy static grid is retired", !staticMarkup.includes("native-canvas-static-grid")],
  ["catalog is not permanently mounted", !staticMarkup.includes("native-canvas-catalog")],
  ["continuous Canvas stage is the static focal surface", staticMarkup.includes('data-canvas-stage="continuous"')],
  ["node tools are represented inside the Canvas stage", staticMarkup.includes("native-canvas-floating-tools")],
  ["current Canvas path keeps a full accessible label", staticMarkup.includes('title="research/field.canvas"')],
  ["compact chrome exposes an explicit saved state", staticMarkup.includes('data-canvas-status="saved"')],
  ["closed file panel is absent from the static accessibility tree", !staticMarkup.includes("data-canvas-file-panel")],
  ["unselected inspector is absent from the static accessibility tree", !staticMarkup.includes("data-canvas-inspector")],
  ["legacy static toolbar is retired", !staticMarkup.includes("native-canvas-static-toolbar")],
  ["legacy browser grid is not introduced by SSR", !staticMarkup.includes("native-canvas-browser-grid")],
  ["static stage preserves JSON Canvas roundtrip guidance", staticMarkup.includes("未识别字段")],
] as const;
const r3cStaticFailures = r3cStaticContracts
  .filter(([, passed]) => !passed)
  .map(([name]) => name);
if (r3cStaticFailures.length) {
  throw new Error(
    `[knowledge-canvas-r3c] ${r3cStaticContracts.length} contracts / ${r3cStaticFailures.length} failed: ${r3cStaticFailures.join(" | ")}`,
  );
}

const focusTarget = (name: string, isConnected = true) => ({
  name,
  isConnected,
  focus: () => undefined,
});
const emptyOpener = focusTarget("empty-opener");
const chromeOpener = focusTarget("chrome-opener");
const canvasStage = focusTarget("canvas-stage");
const disconnectedOpener = focusTarget("disconnected-opener", false);
const disconnectedChrome = focusTarget("disconnected-chrome", false);
const r3cR1FocusContracts = [
  [
    "cancel returns to the connected actual opener instead of the chrome fallback",
    canvasFilePanelCancelFocusTarget(emptyOpener, chromeOpener, canvasStage) === emptyOpener,
  ],
  [
    "cancel uses the chrome fallback only when the actual opener is disconnected",
    canvasFilePanelCancelFocusTarget(disconnectedOpener, chromeOpener, canvasStage) === chromeOpener,
  ],
  [
    "cancel reaches the stage when both opener and chrome fallback are disconnected",
    canvasFilePanelCancelFocusTarget(disconnectedOpener, disconnectedChrome, canvasStage) === canvasStage,
  ],
  [
    "successful selection targets the continuous stage before the chrome fallback",
    canvasFilePanelSelectionFocusTarget(canvasStage, chromeOpener) === canvasStage,
  ],
] as const;
const r3cR1FocusFailures = r3cR1FocusContracts
  .filter(([, passed]) => !passed)
  .map(([name]) => name);
if (r3cR1FocusFailures.length) {
  throw new Error(
    `[knowledge-canvas-r3c-r1] ${r3cR1FocusContracts.length} contracts / ${r3cR1FocusFailures.length} failed: ${r3cR1FocusFailures.join(" | ")}`,
  );
}

// The component has no raw invoker. It uses only the fixed client methods and
// their lower-camel payloads; the caller cannot inject a vault root or shell.
const calls: Array<unknown> = [];
const client = {
  snapshot: async () => canvasSnapshot,
  readCanvas: async (relativePath: string) => {
    calls.push({ method: "readCanvas", relativePath });
    return canvasFile;
  },
  createCanvas: async (relativePath: string, document: JsonCanvasObject) => {
    calls.push({ method: "createCanvas", relativePath, document });
    return canvasMutation("canvas_created");
  },
  writeCanvas: async (relativePath: string, document: JsonCanvasObject, expectedMtimeMs: number, expectedContentHash: string) => {
    calls.push({ method: "writeCanvas", relativePath, document, expectedMtimeMs, expectedContentHash });
    return canvasMutation("canvas_updated");
  },
};

await loadKnowledgeCanvas(client, "research/field.canvas");
await createKnowledgeCanvas(client, "research/new.canvas", canvasDocument);
await saveKnowledgeCanvas(client, canvasFile, canvasDocument);
assertDeep(
  calls,
  [
    { method: "readCanvas", relativePath: "research/field.canvas" },
    { method: "createCanvas", relativePath: "research/new.canvas", document: canvasDocument },
    {
      method: "writeCanvas",
      relativePath: "research/field.canvas",
      document: canvasDocument,
      expectedMtimeMs: 123,
      expectedContentHash: hashA,
    },
  ],
  "Canvas 读/新建/保存必须只走受限 lower-camel typed client",
);
assert(!("invoke" in client) && !("root" in client), "Canvas UI 注入面不得暴露原始 command 或 vault root");

// Converting to React Flow is a display projection only. Mutations copy the
// complete raw JSON Canvas document and patch only their named fields, so
// unknown root/node/edge fields survive unchanged.
const flow = canvasDocumentToFlow(canvasDocument);
assert(flow.nodes.length === 4 && flow.edges.length === 1, "四类节点和连线必须投影到 React Flow");
assert(flow.nodes.find((node) => node.id === "field-note")?.position.x === 12, "节点坐标必须从 JSON Canvas 读取");

const moved = moveKnowledgeCanvasNode(canvasDocument, "field-note", { x: 41.8, y: -2.2 });
const movedNode = canvasDocumentToFlow(moved).nodes.find((node) => node.id === "field-note");
assertDeep(movedNode?.position, { x: 42, y: -2 }, "拖动坐标必须只写入整数 JSON Canvas 位置");
assert((moved.future_root_field as JsonCanvasObject).retained !== undefined, "拖动不得删除未知根字段");
assert(((moved.nodes as JsonCanvasObject[])[0].future_text_field as JsonCanvasObject).retained === true, "拖动不得删除未知节点字段");

const edited = editKnowledgeCanvasNode(moved, "field-note", { text: "已整理田野笔记" });
assert((edited.nodes as JsonCanvasObject[])[0].text === "已整理田野笔记", "编辑只补丁目标节点字段");
assert(((edited.nodes as JsonCanvasObject[])[0].future_text_field as JsonCanvasObject).retained === true, "编辑不得删除未知节点字段");

const withNode = addKnowledgeCanvasNode(edited, {
  id: "new-note",
  type: "text",
  text: "新便签",
  x: 620,
  y: 24,
  width: 220,
  height: 96,
  future_new_node_field: "retain",
});
const withEdge = addKnowledgeCanvasEdge(withNode, {
  id: "new-to-field",
  fromNode: "new-note",
  toNode: "field-note",
  fromEnd: "arrow",
  future_new_edge_field: "retain",
});
assert((withEdge.nodes as JsonCanvasObject[]).length === 5 && (withEdge.edges as JsonCanvasObject[]).length === 2, "新增节点和连线必须保留原结构");
assert(((withEdge.edges as JsonCanvasObject[])[1].future_new_edge_field) === "retain", "新增连线的未知字段必须保留");

const deleted = deleteKnowledgeCanvasNode(withEdge, "field-note");
assert(!(deleted.nodes as JsonCanvasObject[]).some((node) => node.id === "field-note"), "删除节点必须移除目标节点");
assert(!(deleted.edges as JsonCanvasObject[]).some((edge) => edge.fromNode === "field-note" || edge.toNode === "field-note"), "删除节点必须同步移除关联连线");
assert((deleted.future_root_field as JsonCanvasObject).retained !== undefined, "删除节点不得删除未知根字段");
assert(knowledgeCanvasConflictNotice().includes("本地草稿") && knowledgeCanvasConflictNotice().includes("重新读取"), "冲突必须保留草稿并要求显式重读，不能静默覆盖");
assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: true, externallyChanged: false }) === "preserve",
  "Canvas 脏草稿在 focus/manual/写后刷新且磁盘未变时必须保留",
);
assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: true, externallyChanged: true }) === "conflict",
  "Canvas 脏草稿与外部版本同时变化时必须进入冲突态",
);
assert(
  knowledgeWorkspaceDraftRefreshDisposition({ draftIsDirty: false, externallyChanged: true }) === "replace",
  "Canvas 只有 clean 草稿才可在 refresh 时自动读取当前版本",
);
assert(
  knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: true }) === "preserve"
  && knowledgeWorkspaceDraftNavigationDisposition({ draftIsDirty: false }) === "open",
  "Canvas 列表切换和新建打开不得覆盖脏草稿，只有显式重读/取消后才可替换",
);
assert(
  knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 2,
    currentDraftRevision: 3,
    requestGeneration: 9,
    currentGeneration: 9,
    requestCurrentRelativePath: "research/field.canvas",
    currentRelativePath: "research/field.canvas",
  }) === "preserve"
  && knowledgeWorkspaceAsyncCommitDisposition({
    requestDraftRevision: 2,
    currentDraftRevision: 2,
    requestGeneration: 9,
    currentGeneration: 10,
    requestCurrentRelativePath: "research/field.canvas",
    currentRelativePath: "research/other.canvas",
  }) === "preserve",
  "Canvas 异步读取或保存回读迟到时必须保留期间编辑与后续选中的画布",
);

console.log(`native knowledge canvas static shell, typed calls, local JSON Canvas patches, R3C ${r3cStaticContracts.length} / 0, and R3C-R1 ${r3cR1FocusContracts.length} / 0 contracts passed`);

function canvasMutation(operation: "canvas_created" | "canvas_updated"): KnowledgeWorkspaceMutationResult {
  return {
    operation,
    relative_path: "research/field.canvas",
    source_relative_path: null,
    mtime_ms: 123,
    content_hash: hashA,
    audit_event_id: `audit-${operation}`,
  };
}
