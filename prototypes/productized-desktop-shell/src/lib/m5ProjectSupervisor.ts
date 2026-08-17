import { invoke } from "@tauri-apps/api/core";

export type M5SupervisorOpenRequest = {
  project_id: string;
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
    request: { project_id: projectId },
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
  project_locator: string;
  project_id: string;
  launch_ordinal: number;
  scene: string;
  m1_authority_installed: boolean;
  m3_authority_installed: boolean;
  open_available: boolean;
  composition_gap: string | null;
};

export async function loadM5IsolatedAcceptanceStatus(): Promise<M5IsolatedAcceptanceStatus> {
  return invoke("load_m5_isolated_acceptance_status");
}

export async function writeM5IsolatedUiReceipt(phase: string): Promise<string> {
  return invoke("write_m5_isolated_ui_receipt", { phase });
}

export type M5FormalStepResponse = {
  step: string;
  grant_id: string | null;
  dispatch_id: string | null;
  receipt_id: string | null;
  claim_id: string | null;
  review_id: string | null;
  result_decision_recorded: boolean;
  reviewer_actor_id: string | null;
  worker_actor_id: string | null;
  worker_role_session_id: string | null;
};

export async function runM5AuthorizedRuntime(input: {
  binding_id: string;
  project_id: string;
}): Promise<M5FormalStepResponse> {
  return invoke("run_m5_authorized_runtime", { request: input });
}

export async function recordM5WorkerReport(input: {
  binding_id: string;
  project_id: string;
}): Promise<M5FormalStepResponse> {
  return invoke("record_m5_worker_report", { request: input });
}

export async function recordM5IndependentReview(input: {
  binding_id: string;
  project_id: string;
}): Promise<M5FormalStepResponse> {
  return invoke("record_m5_independent_review", { request: input });
}

export async function recordM5ResultDecision(input: {
  binding_id: string;
  project_id: string;
}): Promise<M5FormalStepResponse> {
  return invoke("record_m5_result_decision", { request: input });
}

export async function loadM5GlobalAdviceFixture(
  bindingId: string,
  projectId: string,
): Promise<{
  advice_id: string;
  project_id: string;
  summary: string;
  source_ref: string;
  writable: boolean;
}> {
  return invoke("load_m5_global_advice_fixture", { bindingId, projectId });
}

export async function loadM5ProjectSummary(
  bindingId: string,
  projectId: string,
): Promise<M5ProjectSummaryRead> {
  return invoke("load_m5_project_summary", { bindingId, projectId });
}

export async function rebuildM5ProjectSummary(
  bindingId: string,
  projectId: string,
): Promise<M5ProjectSummaryRead> {
  return invoke("rebuild_m5_project_summary", { bindingId, projectId });
}

export async function openM5SourceDeepLink(
  bindingId: string,
  projectId: string,
  sourceId: string,
): Promise<string> {
  return invoke("open_m5_source_deep_link", { bindingId, projectId, sourceId });
}
