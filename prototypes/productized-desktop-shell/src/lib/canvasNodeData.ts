// Free-canvas node authoring data layer (plan A1–A3, 2026-06-21).
// Pure mapping between the persisted CanvasNode and the rich in-app node payload
// the React Flow custom node / right-panel editor work with. Kept free of React
// Flow / DOM so it is unit-testable offline (no SSR of @xyflow needed).

import type {
  CanvasDefinition,
  CanvasEdge,
  CanvasNode,
  CanvasNodeDispatchRequest,
  ExperimentNodeDispatchRequest,
  CanvasNodeRole,
  CanvasScope,
} from "./types";

export type CanvasCustomField = { id: string; key: string; value: string };

// Canvas surface scope (two-surfaces-one-engine plan, P1/B). The explicit,
// persisted `scope` wins; old canvases that predate the field fall back to
// deriving it from `project_root` (bound → project, unbound → experiment), so
// nothing breaks on read. Takes a Pick so it is trivially offline-testable.
export function canvasScope(canvas: Pick<CanvasDefinition, "scope" | "project_root">): CanvasScope {
  if (canvas.scope === "experiment" || canvas.scope === "project") return canvas.scope;
  return canvas.project_root ? "project" : "experiment";
}

// Session model (plan 2026-06-21 workflow-session-and-scope). The node stores a
// session POLICY, not a resolved session id — definition/template stay free of a
// concrete session; the real session is resolved at run time (P3):
//   { mode: "new" }                  → mint a fresh codex session when run
//   { mode: "resume"; thread_id }    → resume the given existing session
// The two modes are peers (no default bias). thread_id may be "" for a
// "resume-but-not-yet-chosen" node (e.g. fresh from a template — see §8.1).
export type SessionPolicy = { mode: "new" } | { mode: "resume"; thread_id: string };

// Rich, freely-editable node payload. `kind` is an open string (not limited to
// director/subagent); `role` is kept only so the existing persistence /
// (sealed) run logic that still speaks director|subagent keeps working.
// `session` (session model) is the execution-context policy; `work_item_id`
// (C1) binds the node to a workflow-state work item to dispatch.
export type CanvasNodeData = {
  name: string;
  kind: string;
  role: CanvasNodeRole;
  status: string;
  prompt: string;
  sandbox: string;
  skill: string | null;
  session: SessionPolicy;
  work_item_id: string;
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
  { kind: "custom", label: "自定义", role: "subagent", accent: "#8a7f6a", hint: "自定义（也可在节点里直接输入任意 kind）" },
];

export const SANDBOX_PRESETS = ["read-only", "workspace-write", "danger-full-access"] as const;

// Full tone map — still colors every state the system may set (running/blocked
// included), even though the editor only suggests the 3 common ones below.
const STATUS_TONES: Record<string, string> = {
  draft: "#b9b3a6",
  ready: "#5a6f4a",
  running: "#c8602b",
  blocked: "#a14242",
  done: "#3a6a77",
  // 中文状态灯（编辑器默认走中文）；上面的英文键保留，兼容旧画布 / 派生工作流的英文状态。
  草稿: "#b9b3a6",
  就绪: "#5a6f4a",
  进行中: "#c8602b",
  受阻: "#a14242",
  完成: "#3a6a77",
};

// 编辑器只建议常用几个；`status` 仍是自由文本，打「进行中」「受阻」也认、也有配色。
// 中文为主；旧画布存的英文值（STATUS_TONES 保留英文键）仍能正确上色。
export const STATUS_PRESETS = ["草稿", "就绪", "完成"];

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
    status: "草稿",
    prompt: "",
    sandbox: "read-only",
    skill: (preset?.role ?? "subagent") === "subagent" ? "" : null,
    session: { mode: "new" },
    work_item_id: "",
    fields: [],
  };
}

// Read the session policy from a persisted node. Priority: the rich `data.session`
// policy if present; otherwise migrate the legacy top-level `session_id` (a value
// → resume that thread, none → new). Keeps pre-feature canvases working.
function readSessionPolicy(raw: Record<string, unknown>, node: CanvasNode): SessionPolicy {
  const stored = raw.session;
  if (stored && typeof stored === "object" && !Array.isArray(stored)) {
    const mode = (stored as Record<string, unknown>).mode;
    if (mode === "new") return { mode: "new" };
    if (mode === "resume") {
      const threadId = (stored as Record<string, unknown>).thread_id;
      return { mode: "resume", thread_id: typeof threadId === "string" ? threadId : "" };
    }
  }
  if (node.session_id && node.session_id.trim()) {
    return { mode: "resume", thread_id: node.session_id };
  }
  return { mode: "new" };
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
    status: readString(raw, "status", "草稿"),
    prompt: readString(raw, "prompt", ""),
    sandbox: readString(raw, "sandbox", "read-only"),
    skill: node.skill ?? null,
    session: readSessionPolicy(raw, node),
    work_item_id: readString(raw, "work_item_id", ""),
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
  // Top-level session_id stays populated for legacy / sealed logic that still
  // reads it: only the resume policy has a concrete id; a "new" node has none.
  const legacySessionId = data.session.mode === "resume" && data.session.thread_id
    ? data.session.thread_id
    : null;
  return {
    id,
    role: data.role,
    label: data.name,
    skill: data.skill ?? null,
    session_id: legacySessionId,
    kind: data.kind,
    data: {
      status: data.status,
      prompt: data.prompt,
      sandbox: data.sandbox,
      session: data.session,
      work_item_id: data.work_item_id,
      fields: data.fields.map((field) => ({ id: field.id, key: field.key, value: field.value })),
    },
    position: { x: position.x, y: position.y },
    warnings: priorWarnings,
  };
}

// A node is run-ready (UI guard, NOT a security gate — the backend double gate is
// authoritative) once its session policy is resolvable. `surface` decides the
// work-item rule:
//   - experiment (A 映射): the backend auto-creates a temp work_item in the fixed
//     test project on run, so NO manual work_item_id is needed (B 过渡态删除).
//   - project (C 映射): the node IS a workflow-state work item, so it must carry
//     its work_item_id.
// "new" session is resolved at run time (currently only wired for the project /
// resume path; experiment "new" is reported clearly by the backend, not blocked here).
export function nodeRunReadiness(
  data: CanvasNodeData,
  surface: "experiment" | "project" = "project",
): { ready: boolean; reason: string | null } {
  if (data.session.mode === "resume" && !data.session.thread_id.trim()) {
    return { ready: false, reason: "续已有会话但未选具体会话" };
  }
  if (surface === "experiment") {
    // 决策（2026-06-22）：实验面 resume-only，「开新会话」未启用（详见后端注释）。
    if (data.session.mode === "new") {
      return { ready: false, reason: "实验面本期只支持续已有会话（开新会话未启用）" };
    }
    if (!data.prompt.trim()) return { ready: false, reason: "未填提示词（节点要做什么）" };
    return { ready: true, reason: null };
  }
  if (!data.work_item_id.trim()) return { ready: false, reason: "未填工作项 ID（workflow-state 绑定）" };
  return { ready: true, reason: null };
}

// P3 实验面真跑（A 映射）· 从节点数据造实验派发请求。纯函数、离线可测。后端会把目标硬锁成
// 固定测试项目、自动建临时 work_item，所以这里不带 project_root / work_item_id，只传会话策略
// + 节点名 + prompt + 沙箱。new 策略后端不启用（resume-only 决策），会回明确错。
export function buildExperimentNodeDispatchRequest(data: CanvasNodeData): ExperimentNodeDispatchRequest {
  return {
    session_mode: data.session.mode,
    thread_id: data.session.mode === "resume" ? data.session.thread_id.trim() : null,
    summary: data.name,
    objective: data.prompt,
    sandbox_mode: data.sandbox,
    timeout_seconds: 600,
  };
}

// Plan C1 · build the dispatch request from node data. Pure (instructionId
// injected) so it is offline-testable. prompt/sandbox come from the A4 payload;
// project_root / node_id / work_item_id locate the workflow-state work item.
// This only SHAPES the request — the backend double gate decides blocked vs run.
export function buildNodeDispatchRequest(input: {
  nodeId: string;
  projectRoot: string;
  data: CanvasNodeData;
  instructionId: string;
}): CanvasNodeDispatchRequest {
  const { nodeId, projectRoot, data, instructionId } = input;
  const writeRoots = data.sandbox === "workspace-write" && projectRoot ? [projectRoot] : [];
  return {
    project_root: projectRoot,
    node_id: nodeId,
    work_item_id: data.work_item_id.trim(),
    prompt_kind: "user_reviewed_instruction",
    user_reviewed_instruction: {
      instruction_id: instructionId,
      summary: data.name,
      objective: data.prompt,
      execution_cwd: projectRoot,
      sandbox_mode: data.sandbox,
      allowed_write_roots: writeRoots,
      allowed_reads: [],
      allowed_writes: writeRoots,
      forbidden_actions: [],
      timeout_seconds: 600,
      max_retries: 0,
      required_return: [],
      prompt_preview: data.prompt || null,
    },
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
    const data = canvasNodeToData(node);
    // §8.1 · a template only stores the "resume" INTENT, not a concrete session:
    // clear the thread_id on instantiation so a new workflow never inherits the
    // template author's old conversation (the reuse bug this model fixes).
    const session: SessionPolicy =
      data.session.mode === "resume" ? { mode: "resume", thread_id: "" } : data.session;
    return { id, data: { ...data, session }, position: { x: node.position.x, y: node.position.y } };
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
