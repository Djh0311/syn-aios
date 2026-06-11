import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../src/lib/types";
import {
  deriveAgentsPageReadModel,
  deriveAgentsPageReadModelFromParts,
  deriveProjectsPageReadModel,
  deriveProjectsPageReadModelFromParts,
} from "../src/lib/pageSelectors";

const snapshot = {
  summary: {
    generated_at: "2026-06-11T00:00:00Z",
    project_count: 2,
    session_count: 3,
    skill_count: 0,
    plugin_count: 0,
    task_count: 0,
    warning_count: 0,
  },
  projects: [
    {
      project_root: "/tmp/mario-test",
      name: "mario test",
      active_hint: true,
      thread_count: 2,
      active_thread_count: 1,
      archived_thread_count: 1,
      latest_updated_at_ms: 100,
      authority_files: [{ path: "AUTHORITY.md", warnings: [] }],
      handoff_files: [{ path: "handoff.md", warnings: [] }],
      evidence_files: [{ path: "evidence.md", warnings: [] }],
      harness_candidates: [],
      harness_resources: [],
      context_warnings: [],
      warnings: [],
    },
    {
      project_root: "/tmp/quiet",
      name: "quiet",
      active_hint: false,
      thread_count: 0,
      active_thread_count: 0,
      archived_thread_count: 0,
      latest_updated_at_ms: null,
      authority_files: [],
      handoff_files: [],
      evidence_files: [],
      harness_candidates: [],
      harness_resources: [],
      context_warnings: ["索引近似"],
      warnings: [],
    },
  ],
  sessions: [
    {
      thread_id: "s-readable",
      title: "readable",
      project_root: "/tmp/mario-test",
      updated_at_ms: 100,
      archived: false,
      rollout_exists: true,
      rollout_path: "/tmp/rollout.jsonl",
      model: "codex",
      reasoning_effort: "high",
      thread_source: "codex",
      warnings: [],
    },
    {
      thread_id: "s-missing",
      title: "missing",
      project_root: "/tmp/mario-test",
      updated_at_ms: 90,
      archived: false,
      rollout_exists: false,
      rollout_path: null,
      model: null,
      reasoning_effort: null,
      thread_source: "codex",
      warnings: ["缺回放记录"],
    },
    {
      thread_id: "s-archived",
      title: "archived",
      project_root: "/tmp/archived-only",
      updated_at_ms: 80,
      archived: true,
      rollout_exists: true,
      rollout_path: "/tmp/archived.jsonl",
      model: null,
      reasoning_effort: null,
      thread_source: "codex",
      warnings: [],
    },
  ],
  skills: [],
  plugins: [],
  tasks: [],
  agent_adapters: [
    {
      adapter_id: "codex-local",
      agent_type: "codex",
      agent_id: "codex-local",
      display_name: "Codex Local",
      provider: "OpenAI Codex",
      status: "available",
      permission_level: "user_confirmed_write",
      source_kind: "backend_read_model",
      capabilities: [],
      implemented_action_kinds: [],
      hidden_unimplemented_adapters: [],
      warnings: [],
      execution_status: "available_with_user_confirmation",
      credential_status: "not_read",
      model_access_status: "local_read_model_only",
      permission_boundary: "user confirmed",
      unavailable_reason: null,
      requires_user_setup: false,
    },
    {
      adapter_id: "claude-code-planned",
      agent_type: "claude-code",
      agent_id: "claude-code-planned",
      display_name: "Claude Code",
      provider: "Claude",
      status: "planned",
      permission_level: "read_only",
      source_kind: "backend_read_model",
      capabilities: [],
      implemented_action_kinds: [],
      hidden_unimplemented_adapters: ["claude-code"],
      warnings: ["planned"],
      execution_status: "not_implemented",
      credential_status: "not_configured",
      model_access_status: "not_verified",
      permission_boundary: "planned only",
      unavailable_reason: "planned",
      requires_user_setup: true,
    },
  ],
  session_operations: [
    {
      operation_id: "resume",
      label: "继续",
      category: "execution",
      current_status: "planned",
      risk_level: "high",
      adapter_id: "codex-local",
      agent_type: "codex",
      applies_to_session_state: "existing",
      requires_user_confirmation: true,
      writes_codex_home: true,
      writes_workbench_state: true,
      writes_project_files: false,
      reads_full_transcript: false,
      requires_credential: false,
      requires_model_access: true,
      requires_runtime_handle: true,
      audit_requirement: "required",
      unavailable_reason: "not connected in selector test",
      future_task_hint: "future",
      warnings: [],
    },
  ],
  provider_availability: [
    {
      adapter_id: "claude-code-planned",
      provider_id: "claude",
      provider_label: "Claude",
      provider_kind: "external",
      adapter_status: "planned",
      availability_status: "planned",
      credential_status: "not_configured",
      model_status: "not_verified",
      external_call_status: "requires_future_authorization",
      cost_risk_status: "external_cost_possible",
      user_visible_reason: "计划中",
      safe_to_display: true,
      requires_user_configuration: true,
      requires_future_task: true,
      warnings: [],
    },
  ],
  session_continuation_previews: [],
  session_continuation_store: { continuations: [], attempts: [], audit_events: [], warnings: [] },
  runtime_session_attention: [],
  session_run_status_summaries: [],
  runtime_log_store: { entries: [], warnings: [] },
  worker_protocol: { warnings: [] },
  real_execution_product_commands: null,
  project_workflow_automation: null,
  page_read_model_inventory: {
    schema_version: "workbench_page_read_model_inventory.v1",
    generated_at: "2026-06-11T00:00:00Z",
    status: "contract_only",
    source_policy: "fixture",
    contracts: [],
    warnings: [],
  },
  diagnostic_summary: { degraded_states: [] },
  diagnostics: {},
} as unknown as WorkbenchSnapshot;

const workflowState = {
  project_workflows: [
    {
      project_id: "mario",
      project_root: "/tmp/mario-test",
      workflow_id: "wf-1",
      title: "Mario workflow",
      state: "prepared",
      node_count: 2,
      edge_count: 1,
      task_draft_count: 0,
      task_drafts: [],
      pending_permission_count: 0,
      pending_permissions: [],
      recent_dispatches: [],
      recent_reviews: [],
      recent_execution_attempts: [],
      session_binding_count: 0,
      session_bindings: [],
      open_warnings: [],
      blackboard_entry_count: 0,
      blackboard_candidate_count: 0,
      latest_acceptance_state: null,
      latest_acceptance_review_id: null,
      latest_acceptance_summary: null,
      status: "prepared",
      warnings: [],
    },
  ],
} as unknown as WorkflowStateSnapshot;

const projectsModel = deriveProjectsPageReadModel({ snapshot, workflowState });
const agentsModel = deriveAgentsPageReadModel({ snapshot });
const splitProjectsModel = deriveProjectsPageReadModelFromParts({
  projects: snapshot.projects,
  sessions: snapshot.sessions,
  workflowState,
});
const splitAgentsModel = deriveAgentsPageReadModelFromParts({
  projects: snapshot.projects,
  sessions: snapshot.sessions,
  adapterDescriptors: snapshot.agent_adapters,
  sessionOperationDescriptors: snapshot.session_operations,
  providerAvailabilitySummaries: snapshot.provider_availability,
});

assert(projectsModel.schema_version === "projects_page_read_model.v1", "projects selector schema should be stable");
assert(agentsModel.schema_version === "agents_page_read_model.v1", "agents selector schema should be stable");
assert(
  JSON.stringify(projectsModel) === JSON.stringify(splitProjectsModel),
  "split projects selector should match snapshot wrapper",
);
assert(
  JSON.stringify(agentsModel) === JSON.stringify(splitAgentsModel),
  "split agents selector should match snapshot wrapper",
);
assert(projectsModel.source_boundary.generated_from === "workbench_snapshot_selector", "projects selector source should be explicit");
assert(agentsModel.source_boundary.generated_from === "workbench_snapshot_selector", "agents selector source should be explicit");
assert(projectsModel.source_boundary.workbench_snapshot_active, "projects selector must keep WorkbenchSnapshot active");
assert(agentsModel.source_boundary.workbench_snapshot_active, "agents selector must keep WorkbenchSnapshot active");
assert(!projectsModel.source_boundary.page_ui_migrated, "projects page must not claim UI migration");
assert(!agentsModel.source_boundary.page_ui_migrated, "agents page must not claim UI migration");
assert(!projectsModel.source_boundary.writes_stores, "projects selector must be read-only");
assert(!agentsModel.source_boundary.writes_stores, "agents selector must be read-only");
assert(projectsModel.project_count === 2, "projects selector should count projects");
assert(projectsModel.workflow_summary_count === 1, "projects selector should count workflow summaries");
assert(projectsModel.projects[0]?.name === "mario test", "active project should sort first");
assert(projectsModel.projects[0]?.session_count === 2, "project session count should come from sessions");
assert(
  splitProjectsModel.projects[0]?.authority_count === 1 &&
    splitProjectsModel.projects[0]?.handoff_count === 1 &&
    splitProjectsModel.projects[0]?.evidence_count === 1,
  "split projects selector should provide page card file counts",
);
assert(agentsModel.session_summary.readable_count === 1, "agents selector should count readable sessions");
assert(agentsModel.session_summary.missing_rollout_count === 1, "agents selector should keep missing rollout distinct");
assert(agentsModel.session_summary.archived_count === 1, "agents selector should count archived sessions");
assert(agentsModel.project_options[0]?.label === "archived-only", "agents selector should add session-only project options");
assert(
  agentsModel.project_options.some((project) => project.project_root === "/tmp/mario-test" && project.session_count === 2),
  "agents selector project options should include page selection counts",
);
assert(agentsModel.available_adapter_count === 1, "agents selector should count available adapter");
assert(agentsModel.planned_adapter_count === 1, "agents selector should count planned adapter");
assert(agentsModel.operation_boundary_count === 1, "agents selector should count operation boundaries");
assert(agentsModel.provider_boundary_count === 1, "agents selector should count provider boundaries");
assert(agentsModel.conversation_first, "agents selector should preserve conversation-first page intent");
assert(agentsModel.developer_details_collapsed, "developer details should stay collapsed");

const serialized = JSON.stringify({ projectsModel, agentsModel });
for (const forbidden of ["raw transcript", "full transcript", "secret", "token", "prompt_body"]) {
  assert(!serialized.toLowerCase().includes(forbidden), `selector output should not expose ${forbidden}`);
}
assert(
  projectsModel.source_boundary.warnings.includes("do_not_claim_workbench_snapshot_deprecated"),
  "selector should warn against WorkbenchSnapshot deprecation claims",
);

console.log("r4 page selectors test passed");

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
