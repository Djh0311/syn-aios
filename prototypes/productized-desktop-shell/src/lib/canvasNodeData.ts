// Free-canvas node authoring data layer (plan A1–A3, 2026-06-21).
// Pure mapping between the persisted CanvasNode and the rich in-app node payload
// the React Flow custom node / right-panel editor work with. Kept free of React
// Flow / DOM so it is unit-testable offline (no SSR of @xyflow needed).

import type { CanvasEdge, CanvasNode, CanvasNodeRole } from "./types";

export type CanvasCustomField = { id: string; key: string; value: string };

// Rich, freely-editable node payload. `kind` is an open string (not limited to
// director/subagent); `role` is kept only so the existing persistence /
// (sealed) run logic that still speaks director|subagent keeps working.
export type CanvasNodeData = {
  name: string;
  kind: string;
  role: CanvasNodeRole;
  status: string;
  prompt: string;
  sandbox: string;
  skill: string | null;
  session_id: string | null;
  fields: CanvasCustomField[];
};

export type CanvasNodeKindPreset = {
  kind: string;
  label: string;
  role: CanvasNodeRole;
  accent: string;
  hint: string;
};

// Starter palette. These are presets, NOT a closed set — `kind` stays a free
// text field in the editor, and "自定义" seeds a renameable custom node.
export const NODE_KIND_PRESETS: CanvasNodeKindPreset[] = [
  { kind: "director", label: "主管", role: "director", accent: "#c8602b", hint: "统筹 / 派发" },
  { kind: "subagent", label: "子 agent", role: "subagent", accent: "#5a6f4a", hint: "执行节点" },
  { kind: "reviewer", label: "审查", role: "subagent", accent: "#3a6a77", hint: "复核 / 验收" },
  { kind: "tool", label: "工具", role: "subagent", accent: "#7a5ea6", hint: "工具 / 脚本" },
  { kind: "note", label: "便签", role: "subagent", accent: "#9a8c5a", hint: "说明 / 备注" },
  { kind: "custom", label: "自定义", role: "subagent", accent: "#8a7f6a", hint: "自定义种类" },
];

export const SANDBOX_PRESETS = ["read-only", "workspace-write", "danger-full-access"] as const;

const STATUS_TONES: Record<string, string> = {
  draft: "#b9b3a6",
  ready: "#5a6f4a",
  running: "#c8602b",
  blocked: "#a14242",
  done: "#3a6a77",
};

export const STATUS_PRESETS = Object.keys(STATUS_TONES);

export function statusTone(status: string): string {
  return STATUS_TONES[status.trim().toLowerCase()] ?? "#b9b3a6";
}

export function kindPreset(kind: string): CanvasNodeKindPreset | undefined {
  return NODE_KIND_PRESETS.find((preset) => preset.kind === kind);
}

export function kindAccent(kind: string): string {
  return kindPreset(kind)?.accent ?? "#8a7f6a";
}

export function kindLabel(kind: string): string {
  return kindPreset(kind)?.label ?? kind;
}

// Seed data for a freshly created node of the given (possibly custom) kind.
export function createNodeData(kind: string): CanvasNodeData {
  const preset = kindPreset(kind);
  return {
    name: preset && preset.kind !== "custom" ? preset.label : "新节点",
    kind,
    role: preset?.role ?? "subagent",
    status: "draft",
    prompt: "",
    sandbox: "read-only",
    skill: (preset?.role ?? "subagent") === "subagent" ? "" : null,
    session_id: null,
    fields: [],
  };
}

function readString(raw: Record<string, unknown>, key: string, fallback: string): string {
  const value = raw[key];
  return typeof value === "string" ? value : fallback;
}

function readCustomFields(raw: Record<string, unknown>): CanvasCustomField[] {
  if (!Array.isArray(raw.fields)) return [];
  return (raw.fields as unknown[]).flatMap((entry, index) => {
    if (!entry || typeof entry !== "object") return [];
    const rec = entry as Record<string, unknown>;
    const key = typeof rec.key === "string" ? rec.key : "";
    const value = typeof rec.value === "string" ? rec.value : "";
    if (key === "" && value === "") return [];
    const id = typeof rec.id === "string" && rec.id ? rec.id : `field-${index}`;
    return [{ id, key, value }];
  });
}

// Persisted node → rich editor payload. A4: the free payload (kind/status/
// prompt/sandbox/custom fields) now round-trips through the persisted CanvasNode
// (`kind` + `data`). Backward compatible: a node saved before A4 has no kind /
// data, so kind falls back to its role and the free fields default empty.
export function canvasNodeToData(node: CanvasNode): CanvasNodeData {
  const raw = node.data && typeof node.data === "object" && !Array.isArray(node.data)
    ? (node.data as Record<string, unknown>)
    : {};
  return {
    name: node.label,
    kind: node.kind && node.kind.trim() ? node.kind : node.role,
    role: node.role,
    status: readString(raw, "status", "draft"),
    prompt: readString(raw, "prompt", ""),
    sandbox: readString(raw, "sandbox", "read-only"),
    skill: node.skill ?? null,
    session_id: node.session_id ?? null,
    fields: readCustomFields(raw),
  };
}

// Rich editor payload → persisted node. A4: the full free payload persists —
// name/kind/skill/session_id are first-class, and status/prompt/sandbox/custom
// fields ride in `data`. Front/back types match (CanvasNode has kind + data),
// so this is a complete contract, not a half one.
export function dataToCanvasNode(
  id: string,
  data: CanvasNodeData,
  position: { x: number; y: number },
  priorWarnings: string[],
): CanvasNode {
  return {
    id,
    role: data.role,
    label: data.name,
    skill: data.skill ?? null,
    session_id: data.session_id ?? null,
    kind: data.kind,
    data: {
      status: data.status,
      prompt: data.prompt,
      sandbox: data.sandbox,
      fields: data.fields.map((field) => ({ id: field.id, key: field.key, value: field.value })),
    },
    position: { x: position.x, y: position.y },
    warnings: priorWarnings,
  };
}

export type InstantiatedNode = {
  id: string;
  data: CanvasNodeData;
  position: { x: number; y: number };
};

export type InstantiatedGraph = {
  nodes: InstantiatedNode[];
  edges: { id: string; from: string; to: string }[];
};

// Plan B3 · instantiate a fresh editable graph from a saved template: every node
// gets a brand-new id (so the instance is independent of the template), edges
// are remapped onto the new ids, and node payloads are carried over verbatim.
// `newNodeId` is injected so callers control id generation (the app uses
// time/random; tests pass a deterministic factory).
export function instantiateTemplateGraph(
  templateNodes: CanvasNode[],
  templateEdges: CanvasEdge[],
  newNodeId: (node: CanvasNode, index: number) => string,
): InstantiatedGraph {
  const idMap = new Map<string, string>();
  const nodes = templateNodes.map((node, index) => {
    const id = newNodeId(node, index);
    idMap.set(node.id, id);
    return { id, data: canvasNodeToData(node), position: { x: node.position.x, y: node.position.y } };
  });
  const edges = templateEdges
    .filter((edge) => idMap.has(edge.from) && idMap.has(edge.to))
    .map((edge, index) => ({
      id: `tmpl-edge-${index}-${idMap.get(edge.from)}`,
      from: idMap.get(edge.from) as string,
      to: idMap.get(edge.to) as string,
    }));
  return { nodes, edges };
}
