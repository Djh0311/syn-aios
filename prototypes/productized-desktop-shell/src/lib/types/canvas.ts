// Editable Canvas v1 (A 模式：车间 + Codex 主管)
// 决策来源：decisions/2026-05-31-editable-canvas-codex-as-director-v1.md

export type CanvasNodeRole = "director" | "subagent";

export type CanvasNode = {
  id: string;
  role: CanvasNodeRole;
  label: string;
  skill?: string | null;
  session_id?: string | null;
  position: { x: number; y: number };
  warnings: string[];
};

export type CanvasEdge = {
  id: string;
  from: string;
  to: string;
};

export type CanvasDefinition = {
  schema_version: "canvas-v1";
  canvas_id: string;
  display_name: string;
  project_root?: string | null;
  nodes: CanvasNode[];
  edges: CanvasEdge[];
  created_at: string;
  updated_at: string;
  warnings: string[];
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
