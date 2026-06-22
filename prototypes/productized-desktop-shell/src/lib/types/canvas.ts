// Editable Canvas v1 (A 模式：车间 + Codex 主管)
// 决策来源：decisions/2026-05-31-editable-canvas-codex-as-director-v1.md

export type CanvasNodeRole = "director" | "subagent";

export type CanvasNode = {
  id: string;
  role: CanvasNodeRole;
  label: string;
  skill?: string | null;
  session_id?: string | null;
  // Free-canvas authoring (plan A4): `kind` is an open node type beyond
  // director/subagent; `data` carries the free payload (status/prompt/sandbox/
  // custom fields). Both optional + backward compatible with pre-feature nodes.
  kind?: string | null;
  data?: Record<string, unknown> | null;
  position: { x: number; y: number };
  warnings: string[];
};

export type CanvasEdge = {
  id: string;
  from: string;
  to: string;
};

// Canvas surface scope (two-surfaces-one-engine plan, P1/B). EXPLICIT + persisted
// rather than derived from `project_root`, so a "designed but not yet bound"
// project draft (scope=project, project_root=null) is representable. Old canvases
// have no scope and fall back to project-root derivation (see `canvasScope`).
export type CanvasScope = "experiment" | "project";

export type CanvasDefinition = {
  schema_version: "canvas-v1";
  canvas_id: string;
  display_name: string;
  project_root?: string | null;
  scope?: CanvasScope | null;
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  created_at: string;
  updated_at: string;
  warnings: string[];
};

// Plan B · 成熟工作流模式（workflow template）。存图本体 + 元数据，与记忆
// mature_pattern_store 无关。字段与后端 storage::WorkflowTemplate 对齐。
export type WorkflowTemplate = {
  schema_version: "workflow-template-v1";
  template_id: string;
  title: string;
  scope: string; // "project" | "global"
  project_root?: string | null;
  source_canvas_id?: string | null;
  version: number;
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  created_at: string;
  updated_at: string;
  warnings: string[];
};

export type WorkflowTemplateSummary = {
  template_id: string;
  title: string;
  scope: string;
  project_root?: string | null;
  node_count: number;
  edge_count: number;
  created_at: string;
  updated_at: string;
};

// Plan C1 · request shape for the existing double-gated dispatch command
// `execute_workflow_node_dispatch`. Mirrors the backend
// WorkflowNodeDispatchExecuteRequest / UserReviewedInstructionInput. The
// frontend only builds + sends this; the backend gate decides blocked vs run.
export type CanvasNodeDispatchInstruction = {
  instruction_id: string;
  summary: string;
  objective: string;
  execution_cwd: string;
  sandbox_mode: string;
  allowed_write_roots: string[];
  allowed_reads: string[];
  allowed_writes: string[];
  forbidden_actions: string[];
  timeout_seconds: number;
  max_retries: number;
  required_return: string[];
  prompt_preview: string | null;
};

export type CanvasNodeDispatchRequest = {
  project_root: string;
  node_id: string;
  work_item_id: string;
  prompt_kind: string;
  user_reviewed_instruction: CanvasNodeDispatchInstruction;
};

// P3 实验面真跑（架构方案 §9 的 A 映射）。前端不传 project_root / work_item_id：目标恒为
// 固定测试项目（后端硬锁），临时 work_item 后端自动建。只带会话策略 + 节点名 + prompt + 沙箱。
export type ExperimentNodeDispatchRequest = {
  session_mode: "new" | "resume";
  thread_id?: string | null;
  summary: string;
  objective: string;
  sandbox_mode: string;
  timeout_seconds?: number | null;
};

// P3 项目面真跑（架构方案 §9 的 C 映射）。节点=workflow-state work_item 本体，只带定位三元组；
// 派发指令由后端从该 work_item 的任务包构造，会话用节点既有绑定（resume）。
export type ProjectWorkflowNodeRunRequest = {
  project_root: string;
  node_id: string;
  work_item_id: string;
  // 后置C：派发哪个工作流的节点（非默认工作流也能跑）。空则后端从 node_id 解析。
  workflow_id?: string | null;
};

// P3 E · 多工作流底座（架构 §12）。命名避开 types/workflow.ts 既有的 ProjectWorkflowSummary。
export type ProjectWorkflowListItem = {
  workflow_id: string;
  title: string;
  state: string;
  node_count: number;
  is_default: boolean;
};
// 提交草案写回：workflow_id 空=新建一个工作流（不覆盖谁）、非空=更新那一个。nodes/edges 传画布原节点/边。
export type SubmitProjectWorkflowDraftRequest = {
  project_root: string;
  workflow_id?: string | null;
  title: string;
  nodes: unknown[];
  edges: unknown[];
};

export type CanvasRunStatus = "running" | "finished" | "aborted";

export type CanvasRunInbox = {
  node_id: string;
  task: string;
  scope?: string | null;
  dispatched_at: string;
};

export type CanvasRunOutboxPointer = {
  node_id: string;
  outbox_path: string;
  summary: string;
  submitted_at: string;
};

export type CanvasRunState = {
  schema_version: "canvas-run-v1";
  run_id: string;
  canvas_id: string;
  goal: string;
  status: CanvasRunStatus;
  busy_node_id?: string | null;
  inbox?: CanvasRunInbox | null;
  outbox?: CanvasRunOutboxPointer | null;
  finish_summary?: string | null;
  abort_reason?: string | null;
  started_at: string;
  updated_at: string;
};

export type CanvasAuditActor =
  | { kind: "director"; node_id: string }
  | { kind: "subagent"; node_id: string }
  | { kind: "user" }
  | { kind: "system" };

export type CanvasAuditAction =
  | "dispatch"
  | "submit_outbox"
  | "report_blocked"
  | "recycle"
  | "stop"
  | "finish"
  | "abort"
  | "wake_director";

export type CanvasAuditEvent = {
  ts: string;
  actor: CanvasAuditActor;
  action: CanvasAuditAction;
  target_node_id?: string | null;
  payload?: Record<string, unknown> | null;
};

export type DirectorListTeamView = {
  canvas: CanvasDefinition;
  run: CanvasRunState;
  recent_audit: CanvasAuditEvent[];
};

export type DirectorDispatchRequest = {
  run_id: string;
  node_id: string;
  task: string;
  scope?: string | null;
};

export type DirectorRecycleVerdict = "pass" | "changes" | "reject";

export type DirectorRecycleRequest = {
  run_id: string;
  node_id: string;
  verdict: DirectorRecycleVerdict;
  notes: string;
};

export type DirectorFinishRequest = {
  run_id: string;
  summary: string;
};

export type SubagentSubmitOutboxRequest = {
  run_id: string;
  node_id: string;
  content: string;
  summary: string;
};

export type SubagentReportBlockedRequest = {
  run_id: string;
  node_id: string;
  reason: string;
};
