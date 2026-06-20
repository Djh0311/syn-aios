// Free-canvas node authoring data layer (plan A1–A3, 2026-06-21).
// Pure mapping between the persisted CanvasNode and the rich in-app node payload
// the React Flow custom node / right-panel editor work with. Kept free of React
// Flow / DOM so it is unit-testable offline (no SSR of @xyflow needed).

import type { CanvasNode, CanvasNodeRole } from "./types";

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

// Persisted node → rich editor payload. The persisted CanvasNode only carries
// the A1-era fields (role/label/skill/session_id); the free authoring payload
// (kind/status/prompt/sandbox/custom fields) is NOT persisted in A1–A3, so it
// is seeded fresh here — kind falls back to role, the rest to defaults. (Real
// persistence of the free payload is A4; keeping it session-only here is the
// intended A1–A3 behaviour, so a reload drops in-session free edits.)
export function canvasNodeToData(node: CanvasNode): CanvasNodeData {
  return {
    name: node.label,
    kind: node.role,
    role: node.role,
    status: "draft",
    prompt: "",
    sandbox: "read-only",
    skill: node.skill ?? null,
    session_id: node.session_id ?? null,
    fields: [],
  };
}

// Rich editor payload → persisted node. Only the backend-supported subset is
// emitted (id/role/label/skill/session_id/position/warnings) — the free payload
// (kind/status/prompt/sandbox/custom fields) is deliberately dropped so save
// sends NOTHING the store schema lacks (no half-contract). A4 will extend the
// store to persist it properly.
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
    position: { x: position.x, y: position.y },
    warnings: priorWarnings,
  };
}
