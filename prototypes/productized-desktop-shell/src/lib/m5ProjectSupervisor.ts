import { invoke } from "@tauri-apps/api/core";

export type M5SupervisorOpenRequest = {
  project_id: string;
  role_session_id: string;
};

export type M5SupervisorOpenResponse = {
  binding_id: string;
  project_id: string;
  role_session_id: string;
};

export type M5SupervisorTurnRequest = {
  binding_id: string;
  project_id: string;
  kind: string;
  text: string;
};

export type M5SupervisorTurnResponse = {
  kind: string;
  created_proposal: boolean;
  created_grant: boolean;
  spawned: boolean;
  text: string;
};

export type M5SourceRefRead = {
  source_type: string;
  source_id: string;
  deep_link: string;
  last_updated_ms: number;
};

export type M5ProjectSummaryRead = {
  project_id: string;
  version: number;
  watermark_ms: number;
  fact_count: number;
  unverified_claim_count: number;
  open_run_count: number;
  stale: boolean;
  source_refs: M5SourceRefRead[];
};

export async function openM5ProjectSupervisor(
  projectId: string,
): Promise<M5SupervisorOpenResponse> {
  return invoke("open_m5_project_supervisor", {
    request: { project_id: projectId, role_session_id: "" },
  });
}

export async function submitM5SupervisorTurn(
  request: M5SupervisorTurnRequest,
): Promise<M5SupervisorTurnResponse> {
  return invoke("submit_m5_project_supervisor_turn", { request });
}

export async function recordM5AuthorizationDecision(input: {
  binding_id: string;
  project_id: string;
  proposal_id: string;
  decision: "APPROVED" | "REJECTED";
  allowed_command?: string;
}): Promise<{
  dispatched: boolean;
  grant_id: string | null;
  attempt_id: string | null;
  dispatch_id: string | null;
}> {
  return invoke("record_m5_authorization_decision", { request: input });
}

export type M5IsolatedAcceptanceStatus = {
  isolated: boolean;
  project_id: string;
  launch_ordinal: number;
  scene: string;
};

export async function loadM5IsolatedAcceptanceStatus(): Promise<M5IsolatedAcceptanceStatus> {
  return invoke("load_m5_isolated_acceptance_status");
}

export async function writeM5IsolatedUiReceipt(receipt: {
  phase: string;
  binding_id: string;
  role_session_id: string;
  project_id: string;
  proposal_id?: string | null;
  grant_id?: string | null;
  dispatched: boolean;
  spawned: boolean;
  deep_link?: string | null;
  stale?: boolean | null;
  notes: string[];
}): Promise<string> {
  return invoke("write_m5_isolated_ui_receipt", { receipt });
}

export async function runM5IsolatedAuthorizedFollowthrough(input: {
  binding_id: string;
  project_id: string;
  grant_id: string;
  dispatch_id: string;
}): Promise<{
  claim_id: string;
  duplicate_claim_id: string;
  fact_project_id: string;
  review_id: string;
}> {
  return invoke("run_m5_isolated_authorized_followthrough", { request: input });
}

export async function loadM5GlobalAdviceFixture(projectId: string): Promise<{
  advice_id: string;
  project_id: string;
  summary: string;
  source_ref: string;
  writable: boolean;
}> {
  return invoke("load_m5_global_advice_fixture", { projectId });
}

export async function loadM5ProjectSummary(projectId: string): Promise<M5ProjectSummaryRead> {
  return invoke("load_m5_project_summary", { projectId });
}

export async function rebuildM5ProjectSummary(projectId: string): Promise<M5ProjectSummaryRead> {
  return invoke("rebuild_m5_project_summary", { projectId });
}

export async function openM5SourceDeepLink(
  projectId: string,
  sourceId: string,
): Promise<string> {
  return invoke("open_m5_source_deep_link", { projectId, sourceId });
}
