import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../src/lib/types";
import {
  deriveAgentsPageReadModel,
  deriveAgentsPageReadModelFromParts,
  deriveMemoryCenterPageReadModelFromParts,
  deriveProjectsPageReadModel,
  deriveProjectsPageReadModelFromParts,
  deriveRunningWorkflowsPageReadModelFromParts,
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
const runningModel = deriveRunningWorkflowsPageReadModelFromParts({
  workflows: [
    {
      ...workflowState.project_workflows[0],
      state: "running",
      task_drafts: [{ work_item_id: "task-permission", state: "waiting_for_permission", title: "Need permission" }],
    },
  ] as any,
  runtimeAttention: [
    {
      attention_id: "attn-readback",
      status: "readback_unavailable",
      requires_user_action: true,
      blocks_continuation: false,
      readback_boundary: {
        status: "readback_unavailable",
        result_count: null,
        reason: "fixture readback unavailable",
      },
    },
  ] as any,
  runQueue: {
    schema_version: "run_queue_read_model.v1",
    generated_from: "workbench_snapshot",
    run_queue_items: [
      {
        queue_item_id: "queue-readback",
        status: "readback_unavailable",
        readback_status: "readback_unavailable",
        readback_result_count: null,
      },
    ],
    user_confirmation_queue: [
      {
        confirmation_item_id: "memory-confirmation",
        kind: "memory_candidate_confirmation",
      },
    ],
    failure_control_summaries: [{ failure_id: "failure-duplicate", classification: "duplicate_blocked" }],
    operation_control_summary: {
      schema_version: "operation_control_summary.v1",
      retry_proposal_count: 1,
      stop_request_count: 0,
      restart_readiness_count: 0,
      resume_readiness_count: 1,
      readback_issue_count: 1,
      duplicate_blocked_count: 1,
      blocked_by_guard_count: 0,
      stale_cleanup_count: 0,
      manual_review_count: 1,
      confirmation_required_count: 2,
      true_operation_available: false,
      retry_boundary: "retry requires confirmation",
      stop_boundary: "stop is controlled",
      restart_boundary: "restart is future",
      resume_boundary: "resume requires authorization",
      readback_boundary: "unknown remains null",
      stale_cleanup_boundary: "workbench state only",
      user_message: "fixture",
      recommended_next_step: "review",
      warnings: [],
    },
    running_count: 0,
    waiting_user_count: 1,
    blocked_count: 0,
    failed_count: 0,
    readback_issue_count: 1,
    duplicate_blocked_count: 1,
    capture_compensation_count: 0,
    warnings: ["unknown_readback_result_count_must_remain_null"],
  } as any,
  productCommandReadModel: {
    command_count: 2,
    pending_decision_count: 1,
    blocked_attempt_count: 1,
    running_attempt_count: 0,
    failure_stop_retry_summary: { readback_issue_count: 1 },
  } as any,
  automation: {
    run_unit_count: 3,
    waiting_user_count: 1,
    blocked_count: 0,
    readback_unknown_count: 1,
  } as any,
  memoryCaptureStore: { events: [{ capture_event_id: "capture-1" }] } as any,
  memoryCandidateStore: {
    candidates: [
      { candidate_key: "candidate-1", status: "candidate_draft" },
      { candidate_key: "candidate-2", status: "candidate_confirmed", adoption: null },
    ],
  } as any,
});
const memoryModel = deriveMemoryCenterPageReadModelFromParts({
  hasRealSnapshot: true,
  summary: {
    boundary: "记忆中心 fixture boundary",
    formal_summary: { record_count: 1, active_count: 1 },
    candidate_summary: { candidate_count: 2 },
    observation_summary: { observation_count: 3 },
    lint_summary: { open_count: 4, blocking_count: 1 },
    maintenance_summary: { blocking_count: 1, needs_review_count: 2, info_count: 3 },
    mature_pattern_summary: { mature_pattern_candidate_count: 2, user_confirmation_required_count: 1 },
    task_package_summary: { snapshot_count: 5 },
    memory_workbench_summary: {
      action_count: 6,
      capture_count: 7,
      observation_count: 3,
      candidate_count: 2,
      confirmed_pending_formalization_count: 1,
      capture_compensation_count: 1,
    },
    warnings: [],
  } as any,
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
assert(runningModel.schema_version === "running_workflows_page_read_model.v1", "running selector schema should be stable");
assert(memoryModel.schema_version === "memory_center_page_read_model.v1", "memory selector schema should be stable");
assert(runningModel.source_boundary.generated_from === "workbench_snapshot_selector", "running selector source should be explicit");
assert(memoryModel.source_boundary.generated_from === "workbench_snapshot_selector", "memory selector source should be explicit");
assert(runningModel.source_boundary.workbench_snapshot_active, "running selector must keep WorkbenchSnapshot active");
assert(memoryModel.source_boundary.workbench_snapshot_active, "memory selector must keep WorkbenchSnapshot active");
assert(!runningModel.source_boundary.page_ui_migrated, "running page must not claim UI migration");
assert(!memoryModel.source_boundary.page_ui_migrated, "memory page must not claim UI migration");
assert(!runningModel.source_boundary.tauri_command_consumed, "running selector must not claim Tauri command consumption");
assert(!memoryModel.source_boundary.tauri_command_consumed, "memory selector must not claim Tauri command consumption");
assert(!runningModel.source_boundary.writes_stores, "running selector must be read-only");
assert(!memoryModel.source_boundary.writes_stores, "memory selector must be read-only");
assert(runningModel.workflow_count === 1, "running selector should count workflow summaries");
assert(runningModel.workflow_focus_count === 1, "running selector should count focused workflows");
assert(runningModel.waiting_permission_count === 1, "running selector should count waiting permissions");
assert(runningModel.readback_issue_count === 1, "running selector should count readback issues");
assert(runningModel.readback_unknown_result_count === 1, "running selector should keep unknown readback result count distinct");
assert(runningModel.memory_pending.confirmation_count === 1, "running selector should count memory confirmations");
assert(runningModel.memory_pending.pending_candidate_count === 2, "running selector should count pending candidates");
assert(runningModel.product_command.command_count === 2, "running selector should count product commands");
assert(runningModel.automation.run_unit_count === 3, "running selector should count automation units");
assert(memoryModel.formal_memory.record_count === 1, "memory selector should count formal memories");
assert(memoryModel.candidate_memory.candidate_count === 2, "memory selector should keep candidates separate from formal memory");
assert(memoryModel.observation.observation_count === 3, "memory selector should keep observations separate from formal memory");
assert(memoryModel.memory_workbench.action_count === 6, "memory selector should count workbench actions");
assert(memoryModel.snapshot_status_label === "只读读模型", "memory selector should preserve snapshot status label");

const serialized = JSON.stringify({ projectsModel, agentsModel, runningModel, memoryModel });
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
