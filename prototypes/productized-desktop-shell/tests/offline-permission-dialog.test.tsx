import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { AgentSessionCenter, AgentView, ChatTranscript, filterAgentSessions } from "../src/views/AgentView";
import { HomeView } from "../src/views/HomeView";
import { RunningWorkflowsView } from "../src/views/RunningWorkflowsView";
import { SettingsView } from "../src/views/SettingsView";
import { PermissionDialog } from "../src/components/PermissionDialog";
import { WorkflowStatePanel } from "../src/components/WorkflowStatePanel";
import { deriveAgentAdapterDescriptors } from "../src/lib/adapterCapabilities";
import { deriveProviderAvailabilitySummaries } from "../src/lib/providerAvailability";
import {
  deriveH2RealResumeAuthorizationReadiness,
  deriveH2RealResumeExecutionDecisionSurface,
} from "../src/lib/h2RealResumeAuthorization";
import { deriveSessionContinuationPreviews, inspectSessionContinuationGuard } from "../src/lib/sessionContinuation";
import { deriveSessionOperationDescriptors } from "../src/lib/sessionOperations";
import { summarizePlanAuthorizationStore } from "../src/lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../src/lib/projectConsultationProposal";
import {
  buildOfflineRoleDispatchAction,
  buildOfflineStubResult,
  defaultOfflineDispatchBlock,
  OfflineRoleOrchestrationPanel,
  parseOfflineDispatchBlock,
} from "../src/views/OfflineRoleOrchestrationPanel";
import {
  ProjectDetail,
  ProjectDirectorTaskPlanCard,
  ProjectsView,
  TaskDispatchFieldCorrectionEditor,
  TaskDispatchFieldCorrectionShell,
  TaskDispatchReadinessDetails,
  TaskDispatchReadinessController,
  TaskDispatchReadinessShell,
  TaskFieldCorrectionPreview,
  WorkflowRunCheckDetails,
  buildGlobalBoundaryReviewAction,
  buildPrepareAuthorizedAutoDispatchAction,
  buildProjectConsultationProposalDecisionAction,
  missingCorrectionFields,
  TaskFileGenerationController,
  WorkItemOrchestrationCard,
  filterProjectSessionsForProject,
  nextSelectedWorkItemId,
  selectedTaskDraftFor,
} from "../src/views/ProjectsView";
import { SkillsBoardView } from "../src/views/SkillsBoardView";
import { HarnessBoardView } from "../src/views/HarnessBoardView";
import { RightDetailPanel, workspaceRailItems } from "../src/App";
import { devNavItems, primaryNavItems } from "../src/lib/workbenchNavigation";
import {
  canvasBoundaryForbiddenPhrases,
  experimentCanvasBoundary,
  projectWorkflowCanvasBoundary,
} from "../src/lib/canvasSurfaceBoundaries";
import { deriveProjectWorkflowCanvasReadModel, projectCanvasStateExamples } from "../src/lib/projectCanvas";
import { conversationTurns } from "../src/lib/conversationTurns";
import {
  buildBlackboardCandidateOverlay,
  summarizeObservationStore,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeTaskPackageMemoryInjection,
  summarizeTaskMemoryPacketPreview,
} from "../src/lib/candidateGovernance";
import { deriveKnowledgeBaseSummary } from "../src/lib/knowledgeBase";
import { deriveMemoryManagementSummary } from "../src/lib/memoryCenter";
import { deriveRunQueueReadModel } from "../src/lib/runQueue";
import { SecretaryBrief } from "../src/components/SecretaryBrief";
import { deriveSecretaryContext } from "../src/lib/secretaryReadModel";
import { KnowledgeBaseView } from "../src/views/KnowledgeBaseView";
import { MemoryCenterView } from "../src/views/MemoryCenterView";
import type {
  AgentAdapterDescriptor,
  AutoDispatchGuardResult,
  FormalMemoryLifecyclePreview,
  FormalMemoryStoreV1,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryCaptureStoreV1,
  MemoryCandidate,
  MemoryCandidateStoreV1,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  MaturePatternPreviewOutput,
  ObservationStoreV1,
  PendingAction,
  PlanAuthorizationStoreV1,
  PluginRecord,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  ProjectRecord,
  DiagnosticSummary,
  RuntimeSessionAttention,
  RuntimeLogStoreV1,
  SessionRecord,
  SessionContinuationStoreV1,
  SessionRunStatusSummary,
  SkillRecord,
  TaskMemoryPacketBuildOutput,
  TaskPackageFields,
  TaskPackageDispatchReadiness,
  WorkflowRunCheck,
  WorkbenchSnapshot,
  WorkerProtocolReadModel,
  WorkflowStateSnapshot,
} from "../src/lib/types";

type ReactElementLike = React.ReactElement & {
  type: unknown;
  props?: Record<string, unknown>;
};

type Scenario = {
  name: string;
  root: React.ReactNode;
  buttonText: string;
  expectedAction: PendingAction;
};

const project: ProjectRecord = {
  project_root: "/offline-fixture/projects/codex-workbench",
  name: "codex-workbench",
  active_hint: true,
  thread_count: 2,
  active_thread_count: 1,
  archived_thread_count: 1,
  latest_updated_at_ms: 1_764_000_000_000,
  authority_files: [],
  handoff_files: [],
  evidence_files: [],
  harness_candidates: [
    {
      entry_type: "package_script",
      name: "test:offline",
      path: "/offline-fixture/projects/codex-workbench/package.json",
      source: "package.json",
      size_bytes: 512,
      updated_at_ms: 1_764_000_000_000,
      warnings: [],
    },
  ],
  harness_resources: [
    {
      root_path: "/offline-fixture/projects/codex-workbench/harness",
      display_name: "offline folder harness",
      harness_kind: "codex_harness",
      agent_type: "codex",
      adapter_id: "codex-local",
      source_kind: "derived",
      capabilities: ["codex", "harness"],
      manifest_path: null,
      readme_path: null,
      version: null,
      entrypoints: [
        {
          entry_type: "node_script",
          name: "check.js",
          path: "/offline-fixture/projects/codex-workbench/harness/check.js",
          source_kind: "project_file",
          size_bytes: 128,
          updated_at_ms: 1_764_000_000_000,
          warnings: [],
        },
      ],
      permission_level: "read_only",
      size_bytes: 96,
      updated_at_ms: 1_764_000_000_000,
      warnings: ["missing_manifest", "missing_readme", "missing_version"],
    },
    {
      root_path: "/offline-fixture/projects/codex-workbench/weak-harness",
      display_name: "offline weak harness",
      harness_kind: "codex_harness",
      agent_type: "codex",
      adapter_id: "codex-local",
      source_kind: "derived",
      capabilities: [],
      manifest_path: null,
      readme_path: null,
      version: null,
      entrypoints: [],
      permission_level: "read_only",
      size_bytes: 64,
      updated_at_ms: 1_764_000_000_000,
      warnings: ["missing_manifest", "missing_readme", "missing_entrypoints", "missing_version"],
    },
  ],
  context_warnings: [],
  warnings: [],
};

const skill: SkillRecord = {
  skill_id: "offline-skill",
  title: "Offline Skill",
  description: "Fixture skill",
  path: "/offline-fixture/skills/offline",
  source_type: "plugin",
  plugin_name: "offline-plugin",
  plugin_version: "1.0.0",
  warnings: [],
};

const plugin: PluginRecord = {
  plugin_name: "offline-plugin",
  plugin_version: "1.0.0",
  homepage: null,
  skill_count: 1,
  has_apps: false,
  has_mcp_servers: false,
  warnings: [],
};

const session: SessionRecord = {
  thread_id: "offline-thread-001",
  title: "Offline interaction fixture",
  project_root: project.project_root,
  updated_at_ms: 1_764_000_000_000,
  archived: false,
  rollout_exists: true,
  rollout_path: "/offline-fixture/rollouts/offline-thread-001.jsonl",
  model: "offline-model",
  reasoning_effort: "offline",
  thread_source: "offline-fixture",
  warnings: [],
};

const otherProjectSession: SessionRecord = {
  ...session,
  thread_id: "offline-thread-other-project",
  title: "Other project session",
  project_root: "/offline-fixture/projects/other-project",
  rollout_path: "/offline-fixture/rollouts/offline-thread-other-project.jsonl",
};

const workflowProjectId = "project:offline-fixture-projects-codex-workbench";
const workflowId = "workflow:offline-fixture-projects-codex-workbench:default";

const backendAgentAdapterDescriptor: AgentAdapterDescriptor = {
  ...deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: null,
  })[0],
  source_kind: "backend_read_model",
  warnings: [
    "adapter_descriptor_is_backend_read_model_only",
    "does_not_change_codex_execution_semantics",
    "unimplemented_adapters_hidden",
  ],
};

const backendAgentAdapterDescriptors: AgentAdapterDescriptor[] = [
  backendAgentAdapterDescriptor,
  ...deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: null,
  })
    .slice(1)
    .map((descriptor) => ({
      ...descriptor,
      source_kind: "backend_read_model" as const,
    })),
];
const backendSessionOperationDescriptors = deriveSessionOperationDescriptors(backendAgentAdapterDescriptors);
const backendProviderAvailabilitySummaries = deriveProviderAvailabilitySummaries(
  backendAgentAdapterDescriptors,
  backendSessionOperationDescriptors,
);

function workerProtocolFixtureForAdapters(
  descriptors: AgentAdapterDescriptor[],
  operations = backendSessionOperationDescriptors,
): WorkerProtocolReadModel {
  const workerAdapters = descriptors.map((descriptor) => {
    const provider = backendProviderAvailabilitySummaries.find((summary) => summary.adapter_id === descriptor.adapter_id);
    const providerId = provider?.provider_id ?? descriptor.provider;
    return {
      worker_adapter_id: `worker-adapter:${descriptor.adapter_id}`,
      adapter_id: descriptor.adapter_id,
      worker_kind: descriptor.adapter_id === "codex-local" ? "local_cli_agent" : "external_cli_agent_planned",
      display_name: descriptor.display_name,
      provider_id: providerId,
      lifecycle_status: descriptor.status === "available" ? "available_with_guard" : "planned",
      execution_status: descriptor.execution_status,
      credential_status: provider?.credential_status ?? descriptor.credential_status,
      model_status: provider?.model_status ?? descriptor.model_access_status,
      source_policy:
        descriptor.adapter_id === "codex-local"
          ? "codex-local maps into neutral WorkerAdapter; protocol must remain adapter-neutral."
          : "planned descriptor only; no provider call, credential check, or runtime connection.",
      capability_descriptors: descriptor.capabilities.map((capability) => ({
        capability_id: `worker-capability:${descriptor.adapter_id}:${capability.kind}`,
        capability_kind: capability.kind,
        label: capability.label,
        status: capability.status,
        risk_level: capability.kind.includes("dispatch") || capability.kind.includes("run") ? "high" : "medium",
        execution_boundary: capability.boundary,
        provider_id: providerId,
        credential_requirement_id: `credential-requirement:${descriptor.adapter_id}`,
        risk_envelope_id: `external-call-risk:${descriptor.adapter_id}:${capability.kind}`,
        project_policy_status: descriptor.adapter_id === "codex-local" ? "allowed_with_confirmation" : "blocked_planned_adapter",
        source_refs: [{ source_kind: "adapter_capability", source_id: capability.capability_id, label: capability.label }],
        warnings: capability.warnings,
      })),
      source_refs: [{ source_kind: "agent_adapter_descriptor", source_id: descriptor.adapter_id, label: descriptor.display_name }],
      warnings: descriptor.warnings,
    };
  });
  const credentialRequirements = descriptors.map((descriptor) => ({
    requirement_id: `credential-requirement:${descriptor.adapter_id}`,
    adapter_id: descriptor.adapter_id,
    provider_id: descriptor.provider,
    credential_status: descriptor.adapter_id === "codex-local" ? "not_required_by_workbench" : "credential_missing",
    required_for_real_execution: descriptor.adapter_id !== "codex-local",
    read_policy: "never_read_secret_material_in_worker_protocol",
    verification_status: descriptor.adapter_id === "codex-local" ? "workbench_does_not_verify_local_cli_credentials" : "not_verified",
    user_action_required: descriptor.adapter_id !== "codex-local",
    source_refs: [{ source_kind: "provider_availability", source_id: descriptor.adapter_id, label: descriptor.display_name }],
    warnings: ["credential_descriptor_does_not_read_secret"],
  }));
  const externalCallRiskEnvelopes = workerAdapters.flatMap((adapter) =>
    adapter.capability_descriptors.map((capability) => ({
      envelope_id: `external-call-risk:${adapter.adapter_id}:${capability.capability_kind}`,
      adapter_id: adapter.adapter_id,
      provider_id: adapter.provider_id,
      capability_kind: capability.capability_kind,
      external_call_status: adapter.adapter_id === "codex-local" ? "not_needed_for_readonly" : "external_call_blocked",
      data_egress_risk: "prompt_and_project_context_egress_risk",
      cost_risk: adapter.adapter_id === "codex-local" ? "unknown" : "blocked_until_authorized",
      credential_risk: adapter.adapter_id === "codex-local" ? "managed_outside_workbench" : "missing",
      model_risk: adapter.adapter_id === "codex-local" ? "managed_outside_workbench" : "unverified",
      project_policy_status: capability.project_policy_status,
      user_visible_summary: `${adapter.display_name} / ${capability.label} requires policy, permission, audit, and runtime log before real execution.`,
      source_refs: capability.source_refs,
      warnings: ["external_call_risk_envelope_read_model_only"],
    })),
  );
  const adapterContractChecklists = workerAdapters.map((adapter) => {
    const planned = adapter.adapter_id !== "codex-local";
    return {
      checklist_id: `adapter-contract-checklist:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      status: planned ? "blocked_or_reserved_contract" : "ready_for_controlled_adapter_contract",
      protocol_surface_ready: adapter.capability_descriptors.length > 0,
      control_core_required: true,
      permission_required: true,
      audit_required: true,
      runtime_log_required: true,
      credential_boundary_defined: true,
      model_boundary_defined: !planned,
      data_location_defined: !planned,
      missing_items: planned
        ? ["runtime_connection_not_implemented", "model_boundary_or_verification_missing", "data_location_reserved_not_connected"]
        : [],
      source_refs: adapter.source_refs,
      warnings: ["adapter_contract_checklist_read_model_only"],
    };
  });
  return {
    schema_version: "worker_protocol_read_model.v1",
    generated_at: "2026-06-08T00:00:00Z",
    source_policy: "offline fixture; no worker execution.",
    worker_adapters: workerAdapters,
    work_threads: [],
    run_units: [],
    credential_requirements: credentialRequirements,
    external_call_risk_envelopes: externalCallRiskEnvelopes,
    project_capability_policies: [],
    run_relations: [],
    worker_lanes: [],
    multi_worker_dispatch_plans: [],
    adapter_contract_checklists: adapterContractChecklists,
    controlled_api_cli_semantics: workerAdapters.map((adapter) => ({
      semantics_id: `controlled-api-cli-semantics:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      cli_surface: adapter.adapter_id === "codex-local" ? "codex CLI command preview only" : "planned CLI descriptor only",
      api_surface: "Workbench controlled API must call control_core, permission envelope, runtime log, and audit before any real execution.",
      parity_status: adapter.adapter_id === "codex-local" ? "contract_parity_requires_guard" : "reserved_no_runtime_parity",
      control_core_path: "required_before_runner",
      permission_path: "explicit_user_confirmation_required_for_real_execution",
      audit_path: "runtime_log_and_audit_refs_required",
      universal_api_backdoor_blocked: true,
      supported_operation_ids: operations.filter((operation) => operation.adapter_id === adapter.adapter_id).map((operation) => operation.operation_id),
      source_refs: adapter.source_refs,
      warnings: ["cli_parity_read_model_only", "no_universal_app_api_backdoor", "control_core_permission_audit_required"],
    })),
    diagnostic_event_schemas: workerAdapters.map((adapter) => ({
      schema_id: `diagnostic-event-schema:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      event_kinds: ["adapter_health", "dispatch_guard", "permission_decision", "runner_attempt", "readback_boundary", "degraded_mode"],
      severity_levels: ["info", "warning", "degraded", "blocking"],
      required_fields: ["event_id", "adapter_id", "event_kind", "severity", "redacted_summary", "source_refs", "audit_refs", "created_at"],
      redaction_policy: "no_secret_no_raw_transcript_no_provider_payload",
      export_policy: "diagnostic_bundle_requires_separate_authorized_task",
      source_refs: adapter.source_refs,
      warnings: ["diagnostic_event_schema_reserved_read_model_only"],
    })),
    adapter_health_summaries: workerAdapters.map((adapter) => ({
      health_id: `adapter-health:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      status: adapter.adapter_id === "codex-local" ? "available_with_guard" : "planned_unavailable",
      severity: adapter.adapter_id === "codex-local" ? "info" : "warning",
      credential_status: adapter.credential_status,
      model_status: adapter.model_status,
      runtime_status: "no_runtime_attempt",
      degraded_reason: adapter.adapter_id === "codex-local" ? null : "adapter_runtime_not_implemented",
      source_refs: adapter.source_refs,
      warnings: ["adapter_health_read_model_only", "does_not_probe_provider_or_credentials"],
    })),
    adapter_degraded_modes: workerAdapters.map((adapter) => ({
      degraded_mode_id: `adapter-degraded-mode:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      mode: adapter.adapter_id === "codex-local" ? "guarded_execution_possible_only_after_permission" : "readonly_or_blocked",
      blocks_real_execution: adapter.adapter_id !== "codex-local",
      user_visible_summary: `${adapter.display_name} remains guarded or read-only; no universal API backdoor.`,
      allowed_surfaces: ["read_model", "diagnostic_summary", "permission_preview"],
      blocked_surfaces: ["universal_api_backdoor", "provider_probe"],
      recovery_requirement: adapter.adapter_id === "codex-local" ? "execution_point_user_authorization_required" : "separate_adapter_task_with_credentials_model_runtime_and_policy_review",
      source_refs: adapter.source_refs,
      warnings: ["adapter_degraded_mode_read_model_only"],
    })),
    adapter_data_locations: workerAdapters.map((adapter) => ({
      data_location_id: `adapter-data-location:${adapter.adapter_id}`,
      adapter_id: adapter.adapter_id,
      persistence_kind: "descriptor_only",
      workbench_store_refs: ["workbench_snapshot.worker_protocol"],
      adapter_home_policy: adapter.adapter_id === "codex-local" ? "no_codex_home_read_write_without_execution_point_authorization" : "no_adapter_home_known_or_accessed",
      project_write_policy: "project_file_write_requires_permission_envelope_and_allowed_write_roots",
      transcript_policy: "metadata_only_by_default_no_full_transcript",
      secret_policy: "never_read_auth_token_env_keychain_oauth_provider_credentials",
      source_refs: adapter.source_refs,
      warnings: ["adapter_data_location_descriptor_read_model_only"],
    })),
    dispatch_requests: [],
    dispatch_guards: [],
    permission_envelopes: [],
    task_memory_packet_refs: [],
    worker_handoffs: [],
    readback_results: [],
    worker_report_candidates: [],
    warnings: ["worker_protocol_read_model_only", "cli_parity_requires_control_core_permission_audit"],
  };
}

const emptyProject: ProjectRecord = {
  ...project,
  project_root: "/offline-fixture/projects/empty-project",
  name: "empty-project",
  thread_count: 0,
  active_thread_count: 0,
  archived_thread_count: 0,
};

const snapshot: WorkbenchSnapshot = {
  summary: {
    generated_at: "2026-05-28T00:00:00Z",
    project_count: 1,
    session_count: 1,
    skill_count: 1,
    plugin_count: 1,
    task_count: 0,
    warning_count: 0,
  },
  projects: [project],
  sessions: [session],
  skills: [skill],
  plugins: [plugin],
  tasks: [],
  agent_adapters: backendAgentAdapterDescriptors,
  session_operations: backendSessionOperationDescriptors,
  provider_availability: backendProviderAvailabilitySummaries,
  session_continuation_previews: [],
  session_continuation_store: {
    schema_version: "session_continuation_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: "/offline-fixture/workflow-state.v0.json",
      sidecar_path: "/offline-fixture/session-continuations.v1.json",
      project_roots: [],
    },
    revision: 0,
    last_write_id: null,
    generated_by: "offline-test",
    created_at: "2026-06-06T00:00:00Z",
    updated_at: "2026-06-06T00:00:00Z",
    continuations: [],
    attempts: [],
    audit_events: [],
    warnings: [],
  },
  runtime_session_attention: [],
  session_run_status_summaries: [],
  runtime_log_store: runtimeLogStoreFixture(),
  worker_protocol: {
    schema_version: "worker_protocol_read_model.v1",
    generated_at: "2026-05-28T00:00:00Z",
    source_policy: "offline fixture; no worker execution.",
    worker_adapters: [],
    work_threads: [],
    run_units: [],
    credential_requirements: [],
    external_call_risk_envelopes: [],
    project_capability_policies: [],
    run_relations: [],
    worker_lanes: [],
    multi_worker_dispatch_plans: [],
    adapter_contract_checklists: [],
    controlled_api_cli_semantics: [],
    diagnostic_event_schemas: [],
    adapter_health_summaries: [],
    adapter_degraded_modes: [],
    adapter_data_locations: [],
    dispatch_requests: [],
    dispatch_guards: [],
    permission_envelopes: [],
    task_memory_packet_refs: [],
    worker_handoffs: [],
    readback_results: [],
    worker_report_candidates: [],
    warnings: [],
  },
  real_execution_product_commands: {
    schema_version: "real_execution_product_commands.v1",
    sidecar_name: "real-execution-product-commands.v1.json",
    sidecar_path: "/offline-fixture/real-execution-product-commands.v1.json",
    store_available: false,
    store_revision: 0,
    command_count: 0,
    pending_decision_count: 0,
    running_attempt_count: 0,
    blocked_attempt_count: 0,
    last_attempt_status: null,
    failure_stop_retry_summary: {
      schema_version: "real_execution_product_command_failure_stop_retry.v1",
      item_count: 0,
      failure_count: 0,
      blocked_count: 0,
      readback_issue_count: 0,
      manual_stop_requested_count: 0,
      retry_requires_new_user_confirmation: false,
      items: [],
      warnings: ["pcr7_failure_stop_retry_empty_fixture"],
    },
    ordinary_product_entry_status: "readiness_only_pcr1_no_execute",
    legacy_entry_status: "legacy_sealed_blocked_not_product_command",
    runner_entry_status: "internal_runner_blocked_until_unified_execute_and_level_b",
    level_b_authorization_required: true,
    warnings: ["pcr1_read_model_fixture"],
  },
  project_workflow_automation: {
    schema_version: "project_workflow_automation.v1",
    available: false,
    generated_at: "2026-06-09T00:00:00Z",
    latest_automation_id: null,
    latest_status: null, latest_plan: null,
    run_unit_count: 0,
    waiting_user_count: 0,
    blocked_count: 0,
    readback_unknown_count: 0,
    worker_report_count: 0,
    capture_event_count: 0,
    observation_count: 0,
    next_step: null,
    warnings: ["project_workflow_automation_not_recorded"],
  },
  page_read_model_inventory: { schema_version: "workbench_page_read_model_inventory.v1", generated_at: "2026-06-11T00:00:00Z", status: "contract_only", source_policy: "offline fixture", contracts: [], warnings: [] },
  diagnostic_summary: diagnosticSummaryFixture(),
  diagnostics: {
    index_path: "/offline-fixture/index.json",
    tasks_path: "/offline-fixture/tasks.md",
    generated_at: "2026-05-28T00:00:00Z",
    top_level_warning_count: 0,
    context_warning_count: 0,
    allowed_project_path_count: 1,
    allowed_rollout_path_count: 1,
    release_bundle_enabled: false,
    notes: [],
  },
};

const workflowState: WorkflowStateSnapshot = {
  exists: false,
  path: "/offline-fixture/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json",
  schema_version: null,
  workflow_version: null,
  workspace_id: null,
  updated_at: null,
  initialized: false,
  counts: {
    projects: 0,
    agent_adapters: 0,
    workflows: 0,
    nodes: 0,
    edges: 0,
    work_items: 0,
    artifacts: 0,
    reviews: 0,
    audit_events: 0,
    capabilities: 0,
    harness_resources: 0,
  },
  project_workflows: [],
  warnings: ["状态文件不存在；不会自动创建。"],
};

const workflowStateWithProjectWorkflow: WorkflowStateSnapshot = {
  ...workflowState,
  exists: true,
  schema_version: "workflow_state_v0",
  workflow_version: 1,
  initialized: true,
  counts: {
    ...workflowState.counts,
    projects: 1,
    workflows: 1,
    nodes: 7,
    edges: 6,
    audit_events: 2,
  },
  project_workflows: [
    {
      project_id: "project:offline-fixture-projects-codex-workbench",
      project_root: project.project_root,
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      title: "codex-workbench 默认工作流草稿",
      state: "draft",
      node_count: 7,
      edge_count: 6,
      task_draft_count: 2,
      task_drafts: [
        {
          work_item_id: "work-item:offline:001",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          title: "已有任务草稿",
          state: "ready_to_dispatch",
          assigned_role_id: "codex-dev",
          current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
          next_states: ["running", "paused"],
          next_action_label: "下一步：标记执行中",
          artifact_type: "task_package",
          artifact_path: null,
          recent_audit_events: [
            {
              event_id: "audit:offline:001",
              event_type: "task_draft_created",
              before_state: "missing_task_draft",
              after_state: "draft",
              created_at: "2026-05-29T00:00:00Z",
              reason: "离线夹具登记工作项",
            },
          ],
        },
        {
          work_item_id: "work-item:offline:002",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          title: "第二个任务草稿",
          state: "ready_for_review",
          assigned_role_id: "desktop-app",
          current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
          next_states: ["accepted", "needs_changes", "paused"],
          next_action_label: "下一步：接受或要求修改",
          artifact_type: "task_package",
          artifact_path: null,
          recent_audit_events: [],
        },
      ],
      director_reviews: [],
      node_session_bindings: [
        {
          binding_id: "binding:offline:codex-dev",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          work_item_id: "work-item:offline:001",
          agent_type: "codex",
          adapter_id: "codex-local",
          native_thread_id: session.thread_id,
          native_rollout_path: session.rollout_path,
          session_title: session.title,
          session_updated_at_ms: session.updated_at_ms,
          rollout_exists: session.rollout_exists,
          project_binding_source: "index_inferred",
          binding_source: "workflow_bound",
          binding_mode: "select_existing_session",
          lifecycle: "active",
          created_at_ms: 1_764_000_000_000,
          updated_at_ms: 1_764_000_000_000,
          warnings: [],
        },
      ],
      node_dispatches: [
        {
          dispatch_id: "dispatch:offline:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          work_item_id: "work-item:offline:001",
          binding_id: "binding:offline:codex-dev",
          native_thread_id: session.thread_id,
          prompt_preview: "请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
          prompt_kind: "safe_probe",
          state: "completed",
          started_at_ms: 1_764_000_000_000,
          ended_at_ms: 1_764_000_001_000,
          exit_code: 0,
          last_message_path: "/tmp/codex-workflow-node-dispatch-v1/offline-last-message.txt",
          last_message_summary: "WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
          transcript_event_count: 12,
          transcript_target_hits: 1,
          warnings: ["session_cwd_differs_from_project_root"],
        },
        {
          dispatch_id: "dispatch:offline:002",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          work_item_id: "work-item:offline:002",
          binding_id: "binding:offline:codex-dev",
          native_thread_id: session.thread_id,
          prompt_preview: "请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
          prompt_kind: "safe_probe",
          state: "completed",
          started_at_ms: 1_764_000_002_000,
          ended_at_ms: 1_764_000_003_000,
          exit_code: 0,
          last_message_path: "/tmp/codex-workflow-node-dispatch-v1/offline-last-message-002.txt",
          last_message_summary: "WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
          transcript_event_count: 32,
          transcript_target_hits: 4,
          warnings: ["session_cwd_differs_from_project_root", "transcript_warning_count:3"],
        },
      ],
      execution_controls: [
        {
          control_id: "control:offline:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          control_state: "waiting_for_permission",
          long_task_state: "waiting_for_permission",
          retry_count: 1,
          max_retries: 0,
          timeout_seconds: 900,
          cancel_requested_at: null,
          failure_reason: "需要用户确认写入边界。",
          user_reviewed_instruction: {
            instruction_id: "instruction:offline:001",
            summary: "用户审核业务指令夹具",
            objective: "验证协议预览，不执行真实业务任务。",
            execution_cwd: "/Users/yoyi",
            sandbox_mode: "workspace-write",
            allowed_write_roots: ["/Users/yoyi/codex-workflow-mario-test"],
            allowed_reads: [project.project_root],
            allowed_writes: ["/Users/yoyi/codex-workflow-mario-test/index.html"],
            forbidden_actions: ["不读取 auth.json、.env、密钥、token 或授权文件", "不读取完整会话记录", "不运行运行器"],
            required_return: ["薄弱点", "验证命令和结果"],
            timeout_seconds: 900,
            max_retries: 0,
            approval_state: "reviewed",
            preview_markdown:
              "# 用户审核业务指令预览\n\n## 摘要\n用户审核业务指令夹具\n\n## 目标\n验证协议预览，不执行真实业务任务。\n\n## 执行目录\n/Users/yoyi\n\n## 沙箱模式\nworkspace-write",
          },
          audit_event_types: [
            "workflow_execution_control_defined",
            "workflow_permission_requested",
            "workflow_permission_decision_recorded",
            "workflow_execution_retry_scheduled",
            "workflow_execution_timeout_recorded",
            "workflow_execution_cancel_requested",
          ],
          warnings: ["protocol_only_not_business_execution"],
        },
      ],
      permission_requests: [
        {
          request_id: "permission:offline:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          dispatch_id: "dispatch:offline:001",
          permission_kind: "write_workflow_state",
          reason: "需要用户确认是否允许写协议字段。",
          status: "pending",
          requested_at: "2026-05-30T00:00:00Z",
          decided_at: null,
          decision: null,
          warnings: [],
        },
      ],
      execution_attempts: [
        {
          attempt_id: "attempt:offline:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          dispatch_id: "dispatch:offline:001",
          attempt_no: 1,
          state: "failed",
          started_at: "2026-05-30T00:00:00Z",
          ended_at: "2026-05-30T00:01:00Z",
          failure_reason: "离线夹具失败原因。",
          retry_scheduled_at: "2026-05-30T00:02:00Z",
          timed_out_at: null,
          cancel_requested_at: null,
          warnings: ["retry_pending_fixture"],
        },
        {
          attempt_id: "attempt:offline:002",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          dispatch_id: "dispatch:offline:001",
          attempt_no: 2,
          state: "timed_out",
          started_at: "2026-05-30T00:02:00Z",
          ended_at: "2026-05-30T00:17:00Z",
          failure_reason: "离线夹具超时。",
          retry_scheduled_at: null,
          timed_out_at: "2026-05-30T00:17:00Z",
          cancel_requested_at: "2026-05-30T00:18:00Z",
          warnings: ["cancel_requested_fixture"],
        },
      ],
    },
  ],
  warnings: ["状态文件已读取；只展示元数据，不展示正文。"],
};

const blockedWorkflowRunCheck: WorkflowRunCheck = {
  project_root: project.project_root,
  workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
  status: "blocked",
  evidence_completeness: "missing",
  blocked_reasons: ["缺模型；系统不会自动选择模型。", "没有读范围；不能运行。", "会写文件但没有写范围；不能运行。"],
  warnings: ["节点没有声明工具；工具白名单为空。", "节点未要求 harness；harness 要求为空。"],
  checks: [
    {
      check_id: "missing_model",
      label: "模型",
      status: "blocked",
      severity: "blocked",
      reason: "缺模型；系统不会自动选择模型。",
      source_ref: "work-item:offline:001",
    },
    {
      check_id: "missing_read_scope",
      label: "读取范围",
      status: "blocked",
      severity: "blocked",
      reason: "没有读范围；不能运行。",
      source_ref: "work-item:offline:001",
    },
    {
      check_id: "missing_tool_whitelist",
      label: "工具白名单",
      status: "warning",
      severity: "warning",
      reason: "节点没有声明工具；工具白名单为空。",
      source_ref: "work-item:offline:001",
    },
  ],
};

const blockedAutoDispatchGuardResult: AutoDispatchGuardResult = {
  status: "blocked",
  authorization_id: "plan-auth:offline:active",
  reasons: ["写入范围超出方案授权"],
  required_user_confirmation: false,
  required_global_review: false,
  checked_at_ms: 1_764_000_004_000,
};

const planAuthorizationStore: PlanAuthorizationStoreV1 = {
  schema_version: "plan_authorization_store.v1",
  revision: 4,
  authorizations: [
    {
      authorization_id: "plan-auth:offline:active",
      schema_version: "plan_authorization.v1",
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      source_proposal_id: "proposal:offline:001",
      title: "离线方案授权",
      goal_summary: "只允许离线夹具范围内的任务包检查。",
      status: "active",
      scope: {
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        allowed_role_ids: ["codex-dev"],
        allowed_agent_ids: [session.thread_id],
        allowed_read_roots: [project.project_root],
        allowed_write_roots: ["/offline-fixture/projects/codex-workbench/src"],
        allowed_tools: ["read_file"],
        allowed_checks: [],
        allowed_task_package_kinds: ["task_package"],
        max_worker_dispatches: 1,
        max_runtime_minutes: 30,
        stop_conditions: [
          {
            condition_id: "requires-user-confirmation",
            kind: "requires_user_confirmation",
            summary: "需要用户确认时停止。",
            requires_user_confirmation: true,
          },
        ],
      },
      user_confirmation: {
        confirmed_by: "user",
        confirmed_at_ms: 1_764_000_001_000,
        confirmation_summary: "用户确认离线 fixture 方案授权范围。",
      },
      global_boundary_review: {
        reviewed_by: "global_director",
        reviewed_at_ms: 1_764_000_002_000,
        status: "approved",
        summary: "全局主管复核通过离线 fixture 边界。",
      },
      audit_refs: ["audit:plan-auth:offline:created", "audit:auto-dispatch-scope-checked:offline"],
      created_at_ms: 1_764_000_000_000,
      updated_at_ms: 1_764_000_002_000,
      expires_at_ms: null,
    },
  ],
  audit_events: [
    {
      audit_event_id: "audit:auto-dispatch-scope-checked:offline",
      event_type: "auto_dispatch_scope_checked",
      actor_id: "control_core",
      actor_role: "control_core",
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      authorization_id: "plan-auth:offline:active",
      work_item_id: "work-item:offline:001",
      before_status: null,
      after_status: null,
      reason: "写入范围超出方案授权",
      guard_result: blockedAutoDispatchGuardResult,
      created_at_ms: 1_764_000_004_000,
    },
  ],
  updated_at_ms: 1_764_000_004_000,
  warnings: [],
};

const projectConsultationProposalStore: ProjectConsultationProposalStoreV1 = {
  schema_version: "project_consultation_proposal_store.v1",
  revision: 1,
  proposals: [
    {
      proposal_id: "proposal:offline:c2:pending",
      schema_version: "project_consultation_proposal.v1",
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      title: "离线项目咨询方案草案",
      user_goal: "让用户先确认工作流自动推进的方案范围。",
      goal_summary: "确认任务包、角色、读写范围、工具和停止条件后，再进入全局复核。",
      proposed_steps: [
        "整理项目目标和当前任务包。",
        "确认允许角色、agent、读写范围和工具。",
        "用户确认后等待全局主管边界复核。",
      ],
      scope_draft: {
        allowed_role_ids: ["codex-dev", "project_director"],
        allowed_agent_ids: [session.thread_id],
        allowed_read_roots: [project.project_root],
        allowed_write_roots: ["/offline-fixture/projects/codex-workbench/src"],
        allowed_tools: ["read_file"],
        allowed_checks: ["npm run typecheck"],
        allowed_task_package_kinds: ["task_package"],
        stop_conditions: ["超出读写范围或需要权限升级时必须停下。"],
        max_worker_dispatches: 3,
        max_runtime_minutes: 60,
      },
      risks: [
        {
          risk_id: "risk:offline:c2",
          severity: "warning",
          summary: "用户确认后仍不能自动派发。",
          mitigation: "等待全局主管复核。",
        },
      ],
      acceptance_criteria: ["确认后授权仍停在待全局复核。"],
      status: "pending_user_confirmation",
      plan_authorization_id: null,
      created_by_role: "project_consultant",
      created_at_ms: 1_764_000_005_000,
      updated_at_ms: 1_764_000_005_000,
    },
  ],
  decisions: [],
  audit_events: [
    {
      audit_event_id: "audit:project-consultation-proposal-created:offline",
      event_type: "project_consultation_proposal_created",
      actor_id: "project-consultation-fixture",
      actor_role: "project_consultant",
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      proposal_id: "proposal:offline:c2:pending",
      plan_authorization_id: null,
      before_status: null,
      after_status: "pending_user_confirmation",
      reason: "创建离线项目咨询方案草案。",
      created_at_ms: 1_764_000_005_000,
    },
  ],
  updated_at_ms: 1_764_000_005_000,
  warnings: [],
};

const pendingGlobalBoundaryReviewAuthorization = {
  ...planAuthorizationStore.authorizations[0],
  authorization_id: "plan-auth:offline:pending-global",
  source_proposal_id: "proposal:offline:c3:confirmed",
  status: "pending_global_boundary_review" as const,
  user_confirmation: {
    confirmed_by: "user",
    confirmed_at_ms: 1_764_000_006_000,
    confirmation_summary: "用户确认离线 C3 方案范围。",
  },
  global_boundary_review: null,
  audit_refs: ["audit:plan-auth:offline:c3:created", "audit:plan-auth:offline:c3:user-confirmed"],
  updated_at_ms: 1_764_000_006_000,
};

const planAuthorizationStorePendingGlobal: PlanAuthorizationStoreV1 = {
  ...planAuthorizationStore,
  revision: 5,
  authorizations: [pendingGlobalBoundaryReviewAuthorization],
  audit_events: [],
  updated_at_ms: 1_764_000_006_000,
};

const projectConsultationProposalStoreConfirmed: ProjectConsultationProposalStoreV1 = {
  ...projectConsultationProposalStore,
  revision: 2,
  proposals: [
    {
      ...projectConsultationProposalStore.proposals[0],
      proposal_id: "proposal:offline:c3:confirmed",
      status: "user_confirmed",
      plan_authorization_id: "plan-auth:offline:pending-global",
      updated_at_ms: 1_764_000_006_000,
    },
  ],
  decisions: [
    {
      decision_id: "decision:offline:c3:user-confirmed",
      proposal_id: "proposal:offline:c3:confirmed",
      decided_by: "user",
      decision: "confirm",
      summary: "用户确认离线 C3 方案范围。",
      created_at_ms: 1_764_000_006_000,
    },
  ],
  updated_at_ms: 1_764_000_006_000,
};

const projectConsultationProposalStoreActive: ProjectConsultationProposalStoreV1 = {
  ...projectConsultationProposalStore,
  revision: 6,
  proposals: [
    {
      ...projectConsultationProposalStore.proposals[0],
      proposal_id: "proposal:offline:001",
      status: "user_confirmed",
      plan_authorization_id: "plan-auth:offline:active",
      updated_at_ms: 1_764_000_008_000,
    },
  ],
  decisions: [
    {
      decision_id: "decision:offline:c6:user-confirmed",
      proposal_id: "proposal:offline:001",
      decided_by: "user",
      decision: "confirm",
      summary: "用户确认离线 C6 方案范围。",
      created_at_ms: 1_764_000_008_000,
    },
  ],
  updated_at_ms: 1_764_000_008_000,
};

const planAuthorizationSummary = summarizePlanAuthorizationStore(planAuthorizationStore, workflowProjectId, workflowId);
const projectConsultationProposalSummary = summarizeProjectConsultationProposalStore(
  projectConsultationProposalStoreActive,
  planAuthorizationStore,
  workflowProjectId,
  workflowId,
);

const projectDirectorTaskPlan: ProjectDirectorTaskPlan = {
  project_root: project.project_root,
  project_id: "project:offline-fixture-projects-codex-workbench",
  workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
  proposal_id: "proposal:offline:c4:confirmed",
  authorization_id: "plan-auth:offline:active",
  actor_id: "project_director",
  planned_tasks: [
    {
      planned_task_id: "project-director-planned-task:offline:c4",
      title: "C4 准备态子任务",
      objective: "在授权范围内完成离线夹具检查。",
      scope: {
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        target_role: "codex-dev",
        task_package_kind: "task_package",
        allowed_read_scope: [project.project_root],
        allowed_write_scope: ["/offline-fixture/projects/codex-workbench/src"],
        callable_tool_capabilities: ["read_file"],
        required_checks: ["npm run typecheck"],
        stop_conditions: ["超出读写范围或需要权限升级时必须停下。"],
      },
      depends_on: [],
      acceptance_criteria: ["确认 prepared dispatch 仍然只是准备态。"],
      report_format: ["验证结果", "风险和下一步建议"],
      status: "authorized",
      guard_result: {
        status: "authorized",
        authorization_id: "plan-auth:offline:active",
        reasons: [],
        required_user_confirmation: false,
        required_global_review: false,
        checked_at_ms: 1_764_000_007_000,
      },
      work_item_id: "work-item:offline:c4",
      workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
      task_package_id: "artifact:offline:task-package:c4",
      memory_packet_snapshot_id: "task-package-memory-packet-snapshot:v1:offline:c4",
      prepared_dispatch_id: null,
      blocked_reasons: [],
    },
  ],
  planned_task_count: 1,
  authorized_task_count: 1,
  prepared_dispatch_count: 0,
  blocked_count: 0,
  needs_binding_count: 0,
  blocked_reasons: [],
  memory_snapshot_summary: {
    snapshot_id: "task-package-memory-packet-snapshot:v1:offline:c4",
    included_count: 1,
    excluded_count: 0,
    review_material_count: 0,
    stale: false,
    stale_reasons: [],
    display_text: "任务包记忆快照：1 个 snapshot；使用了 1 条正式记忆；排除了 0 条候选 / 观察 / lint 阻断项；0 条待审查材料。",
    warnings: [],
  },
  display_text: "项目主管拆任务：计划 1 / 已授权 1 / 已准备 0 / 需绑定 0 / 阻断 0；准备派发仍未执行工作者。",
  warnings: ["prepared_dispatch_is_not_worker_execution"],
};

const runnableWorkflowRunCheck: WorkflowRunCheck = {
  project_root: project.project_root,
  workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
  status: "runnable",
  evidence_completeness: "complete",
  blocked_reasons: [],
  warnings: [],
  checks: [
    {
      check_id: "missing_model",
      label: "模型",
      status: "pass",
      severity: "info",
      reason: "任务包已显式指定模型。",
      source_ref: "work-item:offline:001",
    },
    {
      check_id: "missing_memory_refs",
      label: "记忆引用",
      status: "pass",
      severity: "info",
      reason: "任务包没有声明需要记忆引用。",
      source_ref: "work-item:offline:001",
    },
  ],
};

const pendingWorkflowResultSummary = {
  project_id: workflowProjectId,
  workflow_id: workflowId,
  final_review_status: "pending",
  final_review_id: null,
  user_decision_status: "pending",
  user_decision_id: null,
  stage_c_acceptance: {
    project_id: workflowProjectId,
    workflow_id: workflowId,
    gates: [
      {
        gate_id: "c6-global-final-review",
        label: "C6 全局最终复核",
        status: "missing_evidence",
        reason: "全局主管尚未记录最终复核。",
        evidence_refs: [],
      },
      {
        gate_id: "stage-c-deferred-real-worker",
        label: "后置：真实工作者 / Codex 执行",
        status: "deferred",
        reason: "C6 默认不执行真实工作者、codex exec 或 codex exec resume。",
        evidence_refs: [],
      },
    ],
    final_review_status: "pending",
    user_decision_status: "pending",
    accepted_as_stage_c_complete: false,
    deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。"],
    open_blockers: ["C6 全局最终复核：全局主管尚未记录最终复核。"],
    warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
  },
  open_issues: [],
  deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。"],
  warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
};

const workflowStateWithDerivedWorkflow: WorkflowStateSnapshot = {
  ...workflowStateWithProjectWorkflow,
  project_blackboards: [
    {
      project_id: "project:offline-fixture-projects-codex-workbench",
      project_root: project.project_root,
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      entries: [
        {
          entry_id: "blackboard:offline:report:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: null,
          workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          kind: "subagent_report",
          title: "子智能体汇报 / codex-dev",
          summary: "离线桩结果：已接收任务，没有执行真实 Codex 会话。",
          status: "candidate",
          source_status: "reported_not_completed",
          source_refs: [{ source_kind: "subagent_report", source_id: "report:dispatch:offline:001", label: "子智能体汇报" }],
          created_at: null,
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:report:001",
            status: "candidate_pending_control_core",
            target_kind: "workflow_fact",
            decided_by_role: null,
            decided_at: null,
            reason: "必须经控制核心确认后才能升级为正式事实。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["blackboard_entry_is_candidate_only"],
        },
        {
          entry_id: "blackboard:offline:risk:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: null,
          workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          kind: "risk",
          title: "方向风险候选",
          summary: "direction_risk_fixture",
          status: "candidate",
          source_status: "reported_not_completed",
          source_refs: [{ source_kind: "subagent_report", source_id: "report:dispatch:offline:001", label: "风险来源" }],
          created_at: null,
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:risk:001",
            status: "candidate_pending_control_core",
            target_kind: "workflow_risk",
            decided_by_role: null,
            decided_at: null,
            reason: "风险只作为黑板候选，不直接推进状态。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["risk_candidate_not_workflow_state_transition"],
        },
        {
          entry_id: "blackboard:offline:permission:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          workflow_node_id: null,
          kind: "permission_request",
          title: "权限请求 / write_workflow_state",
          summary: "需要用户确认是否允许写协议字段。",
          status: "candidate",
          source_status: "pending",
          source_refs: [{ source_kind: "permission_request", source_id: "permission:offline:001", label: "权限请求" }],
          created_at: "2026-05-30T00:00:00Z",
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:permission:001",
            status: "candidate_pending_control_core",
            target_kind: "permission_decision",
            decided_by_role: null,
            decided_at: null,
            reason: "权限请求不能由黑板直接批准或推进状态。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["permission_request_requires_control_core_decision"],
        },
        {
          entry_id: "blackboard:offline:tool:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: "work-item:offline:001",
          workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          kind: "tool_summary",
          title: "工具摘要候选",
          summary: "工具调用只保留摘要和引用，不保留完整输出。",
          status: "candidate",
          source_status: null,
          source_refs: [{ source_kind: "tool_call", source_id: "tool-call:offline:001", label: "工具调用引用" }],
          created_at: "2026-05-30T00:02:00Z",
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:tool:001",
            status: "candidate_pending_control_core",
            target_kind: "audit_event",
            decided_by_role: null,
            decided_at: null,
            reason: "工具摘要不会把工具全文直接升级为审计事件或事实。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["tool_summary_is_not_full_tool_output"],
        },
        {
          entry_id: "blackboard:offline:memory:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: null,
          workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
          kind: "memory_candidate",
          title: "记忆候选",
          summary: "memory:candidate:offline:001 只是候选，不写正式记忆。",
          status: "candidate",
          source_status: "fresh_task_package",
          source_refs: [{ source_kind: "memory_candidate", source_id: "memory:candidate:offline:001", label: "记忆候选" }],
          created_at: null,
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:memory:001",
            status: "candidate_pending_control_core",
            target_kind: "formal_memory",
            decided_by_role: null,
            decided_at: null,
            reason: "记忆候选不能由黑板直接写正式记忆。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["memory_candidate_not_formal_memory"],
        },
        {
          entry_id: "blackboard:offline:knowledge:001",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          work_item_id: null,
          workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
          kind: "knowledge_ref",
          title: "知识引用",
          summary: "knowledge:offline:001 是资料来源，不会被当作记忆写入。",
          status: "candidate",
          source_status: "fresh_task_package",
          source_refs: [{ source_kind: "knowledge_ref", source_id: "knowledge:offline:001", label: "知识引用" }],
          created_at: null,
          promotion_decision: {
            decision_id: "promotion:blackboard:offline:knowledge:001",
            status: "candidate_pending_control_core",
            target_kind: "knowledge_reference",
            decided_by_role: null,
            decided_at: null,
            reason: "知识引用不能被黑板直接升级为正式记忆。",
            audit_refs: [],
            warnings: ["not_promoted_without_control_core_confirmation"],
          },
          warnings: ["knowledge_ref_is_not_memory"],
        },
      ],
      warnings: [
        "project_blackboard_is_read_model_only",
        "blackboard_promotion_requires_control_core_confirmation",
      ],
    },
  ],
  project_workflows: [
    {
      ...workflowStateWithProjectWorkflow.project_workflows[0],
      derived_workflow: {
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        project_id: "project:offline-fixture-projects-codex-workbench",
        title: "codex-workbench 默认工作流草稿",
        source_proposal_id: null,
        status: "draft",
        view_mode: null,
        created_by_role: "project_director",
        owner_role: "director",
        current_stage: "draft",
        run_check_status: "blocked",
        risk_level: null,
        result_summary: pendingWorkflowResultSummary,
        created_at: "2026-05-29T00:00:00Z",
        updated_at: "2026-05-29T00:00:00Z",
        nodes: [
          {
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            node_type: "agent",
            title: "Codex 开发线",
            assigned_role: "codex-dev",
            assigned_session_id: session.thread_id,
            status: "draft",
            task_package_id: "artifact:offline:task-package:001",
            depends_on: ["workflow:offline-fixture-projects-codex-workbench:default:node:director"],
            harness_requirements: [],
            review_requirements: [],
            acceptance_criteria: [],
            created_at: null,
            updated_at: null,
            missing_fields: ["acceptance_criteria"],
            warnings: [],
          },
        ],
        task_packages: [
          {
            task_package_id: "artifact:offline:task-package:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
            project_id: "project:offline-fixture-projects-codex-workbench",
            target_session_id: session.thread_id,
            target_role: "codex-dev",
            task_goal: "已有任务草稿",
            allowed_read_scope: [project.project_root],
            allowed_write_scope: [],
            available_skills: ["offline-skill"],
            available_knowledge_refs: [],
            available_memory_refs: ["mem:formal:offline:included"],
            callable_tool_capabilities: [],
            model_id: null,
            harness_requirements: [],
            forbidden_actions: ["不读取 auth/token/.env/完整会话记录"],
            acceptance_criteria: [],
            report_format: [],
            timeout_policy: "未登记",
            failure_policy: "未登记",
            version: 2,
            stale: true,
            stale_reasons: ["manual_edit_requires_recheck"],
            missing_fields: ["allowed_write_scope", "model_id", "acceptance_criteria", "report_format"],
            export_includes_internal_audit: false,
            memory_injection_summary: {
              snapshot_id: "task-package-memory-packet-snapshot:v1:offline:001",
              included_count: 1,
              excluded_count: 2,
              review_material_count: 2,
              stale: false,
              stale_reasons: [],
              display_text:
                "任务包记忆注入摘要：入选 1 / 排除 2 / 待审查材料 2；快照新鲜。仅活跃正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。",
              warnings: ["candidate_and_observation_review_materials_only"],
            },
            warnings: [],
          },
        ],
        ledger_entries: [
          {
            ledger_entry_id: "audit:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
            entry_type: "task_package_created",
            actor_role: "project_director",
            actor_session_id: null,
            summary: "离线夹具登记工作项",
            source_refs: ["work-item:offline:001"],
            tool_call_refs: [],
            audit_refs: ["audit:offline:001"],
            risk_flags: [],
            created_at: "2026-05-29T00:00:00Z",
          },
          {
            ledger_entry_id: "ledger:dispatch:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            entry_type: "subagent_started",
            actor_role: "project_director",
            actor_session_id: session.thread_id,
            summary: "请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
            source_refs: ["work-item:offline:001"],
            tool_call_refs: [],
            audit_refs: [],
            risk_flags: ["session_cwd_differs_from_project_root"],
            created_at: "1764000000000",
          },
          {
            ledger_entry_id: "ledger:permission:permission:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: null,
            entry_type: "permission_requested",
            actor_role: "subagent_or_project_director",
            actor_session_id: null,
            summary: "需要用户确认是否允许写协议字段。",
            source_refs: ["work-item:offline:001"],
            tool_call_refs: [],
            audit_refs: [],
            risk_flags: [],
            created_at: "2026-05-30T00:00:00Z",
          },
          {
            ledger_entry_id: "ledger:tool:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            entry_type: "tool_call_summary",
            actor_role: "codex-dev",
            actor_session_id: session.thread_id,
            summary: "工具调用只保留摘要和引用，不保留完整输出。",
            source_refs: ["work-item:offline:001"],
            tool_call_refs: ["tool-call:offline:001"],
            audit_refs: ["audit:offline:tool:001"],
            risk_flags: [],
            created_at: "2026-05-30T00:02:00Z",
          },
          {
            ledger_entry_id: "ledger:review:review:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
            entry_type: "review_result",
            actor_role: "review",
            actor_session_id: null,
            summary: "审查通过，但节点仍需项目主管确认。",
            source_refs: ["work-item:offline:001"],
            tool_call_refs: [],
            audit_refs: ["audit:offline:review:001"],
            risk_flags: [],
            created_at: "2026-05-30T00:03:00Z",
          },
        ],
        subagent_reports: [
          {
            report_id: "report:dispatch:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            actor_role: "codex-dev",
            executed_what: "执行离线 safe probe。",
            changed_what: "没有写真实业务项目。",
            summary: "离线桩结果：已接收任务，没有执行真实 Codex 会话。",
            evidence_refs: ["evidence:offline:subagent:001"],
            open_issues: ["需要项目主管判断方向风险。"],
            permission_requests: ["permission:offline:001"],
            direction_risks: ["direction_risk_fixture"],
            follow_up_suggestions: ["由项目主管发起 proposal / decision request。"],
            acceptance_status: "reported_not_completed",
            warnings: ["subagent_cannot_complete_node"],
          },
        ],
        review_results: [
          {
            review_id: "review:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
            result: "passed",
            summary: "审查通过，但 passed 不等于 completed。",
            evidence_refs: ["evidence:offline:review:001"],
            accepted_fact_ids: [],
            observation_ids: [],
            requires_director_confirmation: true,
            can_complete_node: false,
            warnings: ["review_passed_but_director_still_confirms_node_completion"],
          },
        ],
        exceptions: [
          {
            exception_id: "exception:permission:permission:offline:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: null,
            exception_type: "long_permission_wait",
            summary: "权限请求仍在等待。",
            status: "open",
            warnings: [],
          },
          {
            exception_id: "exception:direction:artifact:offline:task-package:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            exception_type: "unresolved_direction_risk",
            summary: "存在未解决方向风险；进入 waiting_decision，不自动继续。",
            status: "waiting_decision",
            warnings: ["direction_risk_fixture"],
          },
          {
            exception_id: "exception:harness:artifact:offline:task-package:001",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
            exception_type: "harness_blocked",
            summary: "harness 阻断完成判定。",
            status: "open",
            warnings: ["harness_fixture"],
          },
        ],
        interface_boundaries: {
          proposal_interface: {
            interface_id: "proposal_interface",
            status: "conservative_stub",
            allowed: ["explicit_proposal_refs", "director_decision_request"],
            blocked: ["subagent_direct_user_decision", "implicit_direction_change"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          memory_candidate_interface: {
            interface_id: "memory_candidate_interface",
            status: "conservative_stub",
            allowed: ["confirmed_memory_refs", "memory_candidates_after_director_summary"],
            blocked: ["auto_write_formal_memory"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          knowledge_refs_interface: {
            interface_id: "knowledge_refs_interface",
            status: "conservative_stub",
            allowed: ["explicit_material_refs"],
            blocked: ["auto_scan_knowledge_base", "obsidian_native_without_design"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          tool_capability_registry: {
            interface_id: "tool_capability_registry",
            status: "conservative_stub",
            allowed: ["static_whitelist", "registered_tool_capabilities"],
            blocked: ["tool_without_whitelist", "tool_output_fulltext_in_ledger"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          model_pool_selector: {
            interface_id: "model_pool_selector",
            status: "conservative_stub",
            allowed: ["explicit_model_id"],
            blocked: ["silent_auto_model_selection"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          harness_requirement_provider: {
            interface_id: "harness_requirement_provider",
            status: "conservative_stub",
            allowed: ["run_check", "task_package_template", "completion_gate"],
            blocked: ["ordinary_workflow_node", "auto_run_harness"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          audit_refs_interface: {
            interface_id: "audit_refs_interface",
            status: "conservative_stub",
            allowed: ["summary_refs", "evidence_refs", "handoff_refs"],
            blocked: ["full_tool_output_in_workflow_ledger"],
            source_refs: ["workflow-task-package-design-v1-confirmed-boundary"],
            warnings: [],
          },
          warnings: [
            "multiple_harness_conflict_policy_open",
            "harness_failure_policy_open",
            "harness_output_ui_detail_open",
            "tool_call_summary_retention_granularity_open",
          ],
        },
        state_machine: {
          workflow_allowed_transitions: [
            "draft->ready",
            "ready->running",
            "running->paused",
            "paused->running",
            "running->waiting_decision",
            "waiting_decision->running",
            "running->completed",
            "running->failed",
            "completed->archived",
            "failed->archived",
          ],
          workflow_rejected_transitions: [
            "draft->running",
            "waiting_decision->completed",
            "failed->running_without_retry_or_reopen",
          ],
          node_allowed_transitions: [
            "not_started->waiting",
            "waiting->running",
            "running->waiting_permission",
            "waiting_permission->running",
            "running->waiting_decision",
            "waiting_decision->running",
            "running->reviewing",
            "reviewing->passed",
            "reviewing->returned",
            "returned->running",
            "running->failed",
            "running->paused",
            "paused->running",
            "waiting->skipped",
          ],
          node_rejected_transitions: [
            "subagent->passed",
            "waiting_decision->running_without_director",
            "failed->running_without_retry_or_reopen",
          ],
          completion_gate: {
            can_complete: false,
            required: [
              "task_goal_completed",
              "acceptance_criteria_met",
              "evidence_refs_exist",
              "review_or_harness_passed_when_required",
              "no_unresolved_risk",
              "memory_candidate_step_recorded",
              "final_user_report_need_recorded",
            ],
            missing: [
              "acceptance_criteria_met",
              "final_user_report_need_recorded",
              "memory_candidate_step_recorded",
              "no_unresolved_risk",
            ],
            warnings: ["only_project_director_can_mark_complete", "passed_not_equal_completed"],
          },
          warnings: [
            "passed_not_equal_completed",
            "subagent_and_review_agent_cannot_complete_node",
            "waiting_decision_requires_manual_confirmation",
          ],
        },
        acceptance_scenarios: [
          {
            scenario_id: "10.1",
            title: "子智能体发现方向风险",
            status: "covered_by_fixture",
            expected: [
              "subagent_report_writes_direction_risk",
              "node_enters_waiting_decision",
              "subagent_does_not_ask_user_directly",
            ],
            evidence_refs: ["exception:direction:artifact:offline:task-package:001"],
            warnings: [],
          },
          {
            scenario_id: "10.2",
            title: "任务包限制上下文",
            status: "blocked_until_package_complete",
            expected: [
              "explicit_memory_refs",
              "explicit_knowledge_refs",
              "explicit_tool_capabilities",
              "missing_scope_means_not_allowed",
            ],
            evidence_refs: ["artifact:offline:task-package:001"],
            warnings: [],
          },
          {
            scenario_id: "10.3",
            title: "子智能体完成并汇报",
            status: "covered_by_fixture",
            expected: [
              "report_enters_workflow_ledger",
              "memory_candidate_can_be_generated",
              "formal_memory_not_written_automatically",
            ],
            evidence_refs: ["report:dispatch:offline:001"],
            warnings: [],
          },
          {
            scenario_id: "10.4",
            title: "审查智能体通过",
            status: "covered_by_fixture",
            expected: ["review_result_stored", "project_director_still_marks_node_completion"],
            evidence_refs: ["review:offline:001"],
            warnings: [],
          },
          {
            scenario_id: "10.5",
            title: "harness 不是普通节点",
            status: "covered_by_rules",
            expected: [
              "harness_affects_run_check",
              "harness_affects_task_package_template",
              "harness_affects_completion_gate",
              "harness_not_main_workflow_node",
            ],
            evidence_refs: ["workflow_interface_boundaries.harness_requirement_provider"],
            warnings: [],
          },
        ],
        warnings: ["derived_from_workflow_state_v0_missing_fields_are_not_guessed"],
      },
    },
  ],
};

const c6WorkflowResultSummary = {
  ...pendingWorkflowResultSummary,
  final_review_status: "accepted",
  final_review_id: "global-final-review:offline:001",
  user_decision_status: "accept_result",
  user_decision_id: "user-result-decision:offline:001",
  stage_c_acceptance: {
    project_id: workflowProjectId,
    workflow_id: workflowId,
    gates: [
      {
        gate_id: "c1-plan-authorization",
        label: "C1 方案授权",
        status: "passed",
        reason: "authorization plan-auth:offline:active / status active",
        evidence_refs: ["plan-auth:offline:active"],
      },
      {
        gate_id: "c2-user-confirmed-proposal",
        label: "C2 用户确认方案",
        status: "passed",
        reason: "proposal proposal:offline:001 / status user_confirmed",
        evidence_refs: ["proposal:offline:001"],
      },
      {
        gate_id: "c3-global-boundary-review",
        label: "C3 全局边界复核",
        status: "passed",
        reason: "authorization active 且 global boundary review 为 approved。",
        evidence_refs: ["plan-auth:offline:active"],
      },
      {
        gate_id: "c4-prepared-dispatch",
        label: "C4 项目主管拆任务 / prepared dispatch",
        status: "passed",
        reason: "task package artifact 和 prepared dispatch 记录存在。",
        evidence_refs: ["dispatch:offline:001", "artifact:offline:task-package:001"],
      },
      {
        gate_id: "c5-worker-report-process-fact",
        label: "C5 工作者汇报 / 过程事实确认",
        status: "passed",
        reason: "工作者汇报已由项目主管确认过程事实；观察仍不是正式记忆。",
        evidence_refs: ["report:dispatch:offline:001", "review:process-fact:offline:001"],
      },
      {
        gate_id: "c6-global-final-review",
        label: "C6 全局最终复核",
        status: "passed",
        reason: "全局主管最终复核不能代表用户已接受。",
        evidence_refs: ["global-final-review:offline:001"],
      },
      {
        gate_id: "c6-user-result-decision",
        label: "C6 用户结果决定",
        status: "passed",
        reason: "用户决定只适用于本次结果，不代表未来任务默认接受。",
        evidence_refs: ["user-result-decision:offline:001"],
      },
      {
        gate_id: "stage-c-deferred-real-worker",
        label: "后置：真实工作者 / Codex 执行",
        status: "deferred",
        reason: "C6 默认不执行真实工作者、codex exec 或 codex exec resume。",
        evidence_refs: [],
      },
    ],
    final_review_status: "accepted",
    user_decision_status: "accept_result",
    accepted_as_stage_c_complete: true,
    deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。", "真实 Tauri 全面截图验收仍是后置项。"],
    open_blockers: [],
    warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
  },
  open_issues: [],
  deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。", "真实 Tauri 全面截图验收仍是后置项。"],
  warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
};

const workflowStateWithC6ResultSummary: WorkflowStateSnapshot = {
  ...workflowStateWithDerivedWorkflow,
  project_workflows: [
    {
      ...workflowStateWithDerivedWorkflow.project_workflows[0],
      derived_workflow: {
        ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!,
        review_results: [
          ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!.review_results,
          {
            review_id: "review:process-fact:offline:001",
            workflow_id: workflowId,
            workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
            reviewer_role: "project_director",
            report_id: "report:dispatch:offline:001",
            accepted_fact_ids: ["process-fact:offline:001"],
            observation_ids: ["observation:process-fact:offline:001"],
            result: "process_fact_confirmed",
            summary: "项目主管确认 C5 过程事实；observation 仍不是正式记忆。",
            evidence_refs: ["evidence:offline:process-fact:001"],
            requires_director_confirmation: false,
            can_complete_node: false,
            warnings: ["observation_is_not_formal_memory"],
          },
        ],
        result_summary: c6WorkflowResultSummary,
      },
    },
  ],
};

const workflowStateReadyForReview: WorkflowStateSnapshot = {
  ...workflowStateWithProjectWorkflow,
  counts: {
    ...workflowStateWithProjectWorkflow.counts,
    reviews: 0,
  },
  project_workflows: [
    {
      ...workflowStateWithProjectWorkflow.project_workflows[0],
      task_drafts: [
        {
          ...workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[0],
          state: "ready_for_review",
          current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
          next_states: ["accepted", "needs_changes", "paused"],
          next_action_label: "下一步：接受或要求修改",
        },
        workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[1],
      ],
    },
  ],
};

const workflowStateWithPreparedOfflineDispatch: WorkflowStateSnapshot = {
  ...workflowStateWithProjectWorkflow,
  project_workflows: [
    {
      ...workflowStateWithProjectWorkflow.project_workflows[0],
      node_dispatches: [
        {
          dispatch_id: "offline-dispatch:fixture:prepared",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          work_item_id: "work-item:offline:001",
          binding_id: "offline-role-binding:codex-dev",
          native_thread_id: "offline-role:codex-dev",
          prompt_preview: "派发给：开发线\n任务名：已落账离线派发\n目标：验证回传使用已落账派发块。",
          prompt_kind: "offline_role_dispatch",
          offline_role_dispatch: {
            project_root: project.project_root,
            work_item_id: "work-item:offline:001",
            target_role_id: "codex-dev",
            target_role_label: "开发线",
            task_title: "已落账离线派发",
            objective: "验证回传使用已落账派发块。",
            execution_cwd: project.project_root,
            allowed_reads: [project.project_root],
            allowed_writes: [`${project.project_root}/README.md`],
            forbidden_actions: ["不执行 codex exec resume"],
            acceptance_criteria: ["角色回传摘要包含已落账任务名"],
            timeout_seconds: 600,
            required_return: ["薄弱点", "验证结果"],
            raw_block: "派发给：开发线\n任务名：已落账离线派发\n目标：验证回传使用已落账派发块。",
          },
          state: "prepared",
          started_at_ms: null,
          ended_at_ms: null,
          exit_code: null,
          last_message_path: null,
          last_message_summary: null,
          transcript_event_count: null,
          transcript_target_hits: null,
          warnings: ["offline_only_no_codex_resume"],
        },
        ...workflowStateWithProjectWorkflow.project_workflows[0].node_dispatches,
      ],
    },
  ],
};

const workflowStateWithCompletedOfflineDispatch: WorkflowStateSnapshot = {
  ...workflowStateWithProjectWorkflow,
  project_workflows: [
    {
      ...workflowStateWithProjectWorkflow.project_workflows[0],
      task_drafts: [
        {
          ...workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[0],
          state: "ready_for_review",
          current_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:review",
          next_states: ["accepted", "needs_changes", "paused"],
          next_action_label: "下一步：接受或要求修改",
        },
        workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[1],
      ],
      node_dispatches: [
        {
          dispatch_id: "offline-dispatch:fixture:completed",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
          work_item_id: "work-item:offline:001",
          binding_id: "offline-role-binding:codex-dev",
          native_thread_id: "offline-role:codex-dev",
          prompt_preview: "派发给：开发线\n任务名：已完成离线派发\n目标：验证总指导回收。",
          prompt_kind: "offline_role_dispatch",
          offline_role_dispatch: {
            project_root: project.project_root,
            work_item_id: "work-item:offline:001",
            target_role_id: "codex-dev",
            target_role_label: "开发线",
            task_title: "已完成离线派发",
            objective: "验证总指导回收。",
            execution_cwd: project.project_root,
            allowed_reads: [project.project_root],
            allowed_writes: [`${project.project_root}/README.md`],
            forbidden_actions: ["不执行 codex exec resume"],
            acceptance_criteria: ["总指导回收按钮可用"],
            timeout_seconds: 600,
            required_return: ["薄弱点", "验证结果"],
            raw_block: "派发给：开发线\n任务名：已完成离线派发\n目标：验证总指导回收。",
          },
          state: "completed",
          started_at_ms: null,
          ended_at_ms: 1_764_000_004_000,
          exit_code: 0,
          last_message_path: null,
          last_message_summary: "离线桩结果：已接收任务，没有执行真实 Codex 会话。",
          transcript_event_count: 0,
          transcript_target_hits: 0,
          warnings: ["offline_only_no_codex_resume"],
        },
        ...workflowStateWithProjectWorkflow.project_workflows[0].node_dispatches,
      ],
    },
  ],
};

const workflowStateWithGeneratedTaskFile: WorkflowStateSnapshot = {
  ...workflowStateWithProjectWorkflow,
  project_workflows: [
    {
      ...workflowStateWithProjectWorkflow.project_workflows[0],
      task_drafts: [
        {
          ...workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[0],
          artifact_path: "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
        },
        workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[1],
      ],
    },
  ],
};

const notReadyDispatchReadiness: TaskPackageDispatchReadiness = {
  project_root: project.project_root,
  workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
  work_item_id: "work-item:offline:001",
  artifact_id: "artifact:offline:001",
  artifact_path: "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
  status: "not_ready",
  blocking_reasons: [
    "任务名为空、待补充或仍像测试草稿。",
    "禁止事项仍包含和当前生成行为冲突的历史禁令。",
  ],
  warnings: [],
  can_generate_next_version: false,
  memory_injection_summary: {
    snapshot_id: null,
    included_count: 0,
    excluded_count: 0,
    review_material_count: 0,
    stale: true,
    stale_reasons: ["task_memory_packet_snapshot_missing"],
    display_text:
      "任务包记忆注入摘要：尚未生成任务包记忆快照。仅活跃正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。",
    warnings: ["task_memory_packet_snapshot_missing"],
  },
};

const scenarios: Scenario[] = [];

let capturedAction: PendingAction | null = null;

function captureAction(action: PendingAction) {
  capturedAction = action;
}

function main() {
  runShellScenario();
  runProjectCanvasReadModelScenario();
  runAdapterCapabilityScenario();
  runSessionOperationBoundaryScenario();
  runProviderAvailabilityBoundaryScenario();
  runAdapterSdkCliDiagnosticsBoundaryScenario();
  runRealExecutionProductCommandBoundaryScenario();
  runStageJRunQueueScenario();
  runSessionContinuationPreviewScenario();
  runControlledSessionContinuationLevelAScenario();
  runH2RealResumeAuthorizationReadinessScenario();
  runRuntimeSessionAttentionScenario();
  runRuntimeLogBoundaryScenario();
  runCandidateGovernanceScenario();
  runMemoryManagementCenterScenario();
  runKnowledgeBaseBoundaryScenario();
  runSecretaryReadModelScenario();
  runRightRailSecretarySurfaceScenario();
  runOfflineRoleOrchestrationScenario();
  runTranscriptCleaningScenario();
  runSessionCenterHardeningScenario();
  for (const scenario of scenarios) {
    runScenario(scenario);
  }
  console.log(`offline interaction tests passed: ${scenarios.length + 14}`);
}

function runRealExecutionProductCommandBoundaryScenario() {
  const readModel = snapshot.real_execution_product_commands;
  assert(readModel, "PCR1 snapshot 应包含真实执行 product command 只读摘要");
  assert(readModel.schema_version === "real_execution_product_commands.v1", "PCR1 read model schema 不匹配");
  assert(readModel.store_available === false, "PCR1 离线 fixture 不应声明真实 sidecar 可用");
  assert(readModel.command_count === 0, "PCR1 离线 fixture 不应包含真实 product command");
  assert(
    readModel.ordinary_product_entry_status === "readiness_only_pcr1_no_execute",
    "PCR1 普通入口只能是 readiness-only",
  );
  assert(
    readModel.legacy_entry_status === "legacy_sealed_blocked_not_product_command",
    "PCR1 旧入口必须保持 legacy / sealed / blocked",
  );
  assert(
    readModel.runner_entry_status === "internal_runner_blocked_until_unified_execute_and_level_b",
    "PCR1 runner 入口必须保持内部阻断直到 Level B",
  );
  assert(readModel.level_b_authorization_required, "PCR1 必须保留 Level B 授权要求");
  assert(readModel.failure_stop_retry_summary.item_count === 0, "PCR7 默认 fixture 不应造失败/停止/重试状态");
  assert(!readModel.failure_stop_retry_summary.retry_requires_new_user_confirmation, "PCR7 默认 fixture 不应要求重新确认");

  const pcr7FailureStopRetryItems = [
    ["user_rejected", "用户已拒绝", "用户拒绝或要求修改，当前不能继续执行。", "high", true, ["decision:decision:pcr7:user_rejected"], ["decision:rejected"]],
    ["blocked_by_guard", "被安全边界阻断", "安全边界或准备状态阻断了统一执行链路。", "high", false, ["preview:preview:pcr7:guard"], ["guard_policy_blocked"]],
    ["blocked_by_diagnostics", "被诊断阻断", "诊断降级或阻断状态要求先查看诊断。", "high", false, ["preview:preview:pcr7:diagnostics"], ["diagnostics:blocking_fixture"]],
    ["duplicate_blocked", "重复执行已阻断", "已有重复命令或运行记录，不能并行继续。", "medium", false, ["preview:preview:pcr7:duplicate"], ["duplicate_active"]],
    ["blocked_stale_memory", "记忆包缺失或过期", "任务记忆包缺失或过期，重新确认前需要先检查。", "medium", false, ["preview:preview:pcr7:memory"], ["memory_packet_stale"]],
    ["timed_out", "执行超时", "执行或读回超时，不能解释为已经完成停止。", "high", true, ["attempt:attempt:pcr7:timed_out"], ["readback_timed_out"]],
    ["readback_unavailable", "读回不可用", "没有可用读回来源，结果数未知。", "medium", false, ["attempt:attempt:pcr7:readback_unavailable"], ["unknown_readback_result_count_must_remain_null"]],
    ["readback_failed", "读回失败", "读回尝试失败或不可信，结果数未知。", "high", true, ["attempt:attempt:pcr7:readback_failed"], ["readback_parser_failed"]],
    ["runner_failed", "运行失败", "运行记录失败，不能自动重新执行。", "high", true, ["attempt:attempt:pcr7:runner_failed"], ["failure_reason:runner_failed_fixture"]],
    ["manual_stop_requested", "停止请求需受控处理", "用户请求停止仅作为产品状态，本任务不会停止真实进程。", "medium", false, ["decision:decision:pcr7:manual_stop"], ["manual_stop_requested_from_decision_reason"]],
    ["retry_requires_new_user_confirmation", "需要重新确认", "再次执行前需要新的用户确认；不会自动重试。", "high", true, ["product_command_retry_boundary"], ["pcr7_no_auto_retry_requires_new_user_confirmation"]],
  ].map(([kind, title, summary, severity, requiresNewUserConfirmation, sourceRefs, warnings]) => ({
    kind: kind as string,
    title: title as string,
    summary: summary as string,
    count: 1,
    severity: severity as string,
    requires_new_user_confirmation: requiresNewUserConfirmation as boolean,
    result_count: null,
    source_refs: sourceRefs as string[],
    warnings: warnings as string[],
  }));

  const activeReadModel = {
    ...readModel,
    store_available: true,
    command_count: 2,
    pending_decision_count: 1,
    running_attempt_count: 1,
    blocked_attempt_count: 1,
    last_attempt_status: "succeeded_stub",
    failure_stop_retry_summary: {
      schema_version: "real_execution_product_command_failure_stop_retry.v1",
      item_count: pcr7FailureStopRetryItems.length,
      failure_count: 4,
      blocked_count: 4,
      readback_issue_count: 3,
      manual_stop_requested_count: 1,
      retry_requires_new_user_confirmation: true,
      items: pcr7FailureStopRetryItems,
      warnings: ["pcr7_failure_stop_retry_summary_is_read_model_only", "retry_requires_new_user_confirmation_no_auto_retry"],
    },
    warnings: ["readback_unavailable_is_not_zero_results"],
  };
  const workflowId = workflowStateWithProjectWorkflow.project_workflows[0]?.workflow_id ?? "workflow:k3:fixture";
  const activeAutomation = {
    schema_version: "project_workflow_automation.v1",
    available: true,
    generated_at: "2026-06-09T00:00:00Z",
    latest_automation_id: "project-workflow-automation:offline",
    latest_status: "phase_a_closed_loop_recorded",
    latest_plan: {
      schema_version: "project_workflow_automation.v1",
      automation_id: "project-workflow-automation:offline",
      project_id: "project:offline",
      project_root: project.project_root,
      workflow_id: workflowId,
      user_goal: "离线验证 K3 Level A 项目自动编排摘要。",
      current_phase: "collector_summary",
      next_step: "等待主管复核 K3 Level A evidence / handoff。",
      run_units: ["director_plan", "developer_execution", "verifier_check", "collector_summary", "director_final_review"].map((kind, index) => ({
        run_unit_id: `run-unit:k3:${kind}`,
        run_unit_kind: kind,
        role: kind === "developer_execution" ? "developer_execution" : kind,
        status: kind === "director_final_review" ? "needs_review" : kind === "developer_execution" ? "readback_unavailable" : "completed",
        project_id: "project:offline",
        project_root: project.project_root,
        workflow_id: workflowId,
        workflow_node_id: `${workflowId}:node:codex-dev`,
        work_item_id: "work-item:k3:offline",
        task_package_ref: "task-package:k3:offline",
        memory_packet_ref: "memory-packet:k3:offline",
        product_command_preview_ref: `preview:k3:${kind}`,
        product_command_ref: index === 1 ? "product-command:k3:developer" : null,
        runtime_log_refs: index === 1 ? ["runtime-log:k3:phase-a"] : [],
        audit_refs: [`audit:k3:${kind}`],
        readback_ref: index === 1 ? "readback:k3:phase-a" : null,
        readback_status: "readback_unavailable",
        readback_result_count: null,
        worker_report_ref: index === 1 || kind === "collector_summary" ? "worker-report:k3:offline" : null,
        capture_event_refs: ["memory-capture:k3:offline"],
        observation_refs: kind === "collector_summary" || kind === "director_final_review" ? ["observation:k3:process-fact"] : [],
        memory_candidate_refs: [],
        runner_call_allowed: false,
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        summary: `${kind} 摘要`,
        next_step: kind === "director_final_review" ? "等待主管复核。" : "进入下一阶段。",
        blocked_reasons: [],
        warnings: ["k3_level_a_no_real_codex_execution"],
      })),
      blocked_reasons: [],
      warnings: ["k3_level_a_phase_a_only"],
    },
    run_unit_count: 5,
    waiting_user_count: 0,
    blocked_count: 0,
    readback_unknown_count: 5,
    worker_report_count: 2,
    capture_event_count: 1,
    observation_count: 2,
    next_step: "等待主管复核 K3 Level A evidence / handoff。",
    warnings: ["k3_level_a_read_model_from_workflow_audit_event"],
  };
  const snapshotWithProductCommands = {
    ...snapshot,
    real_execution_product_commands: activeReadModel,
    project_workflow_automation: activeAutomation,
  };

  const agentNode = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={snapshot.agent_adapters}
      workflowState={workflowStateWithProjectWorkflow}
      realExecutionProductCommands={activeReadModel}
      projectWorkflowAutomation={activeAutomation}
      onRequestAction={captureAction}
    />
  );
  const agentText = visibleText(agentNode);
  const agentMarkup = renderToStaticMarkup(agentNode);
  for (const expectedText of ["项目", "对话", "可以开始对话", "任务输入", "生成发送预览"]) {
    assert(agentText.includes(expectedText), `J5 Agent 对话工作区缺少 ${expectedText}`);
  }
  assert(agentMarkup.includes("agent-conversation-bar"), "J5 Agent 普通区应有项目 / 对话选择条");
  assert(agentMarkup.includes("agent-chat-composer"), "J5 Agent 普通区应有任务输入框");
  assert(
    agentMarkup.indexOf("agent-conversation-bar") < agentMarkup.indexOf("agent-boundary-details"),
    "J5 Agent 普通对话区必须排在开发者详情前面",
  );
  for (const expectedText of ["统一执行链路", "2 条统一命令", "等待确认：1", "受控记录：1", "阻断：1", "读回边界：未知 / 不可用"]) {
    assert(agentText.includes(expectedText), `PCR6 Agent 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["自动编排：Level A 闭环已记录", "编排 run units：5", "编排读回未知：5", "编排捕获来源：1", "worker report 已回收"]) {
    assert(agentText.includes(expectedText), `K3 Agent 自动编排摘要缺少 ${expectedText}`);
  }
  for (const expectedText of [
    "Codex 控制",
    "J1-A · 产品命令入口 · 非真实执行",
    "生成预览",
    "写入准备",
    "用户确认",
    "记录 Phase A（不真实执行）",
    "任务正文保存策略",
    "观察 / 候选来源",
    "不会自动写正式记忆",
    "临时运行绑定",
  ]) {
    assert(agentText.includes(expectedText), `J1-A Agent Codex 控制入口缺少 ${expectedText}`);
  }
  for (const expectedText of [
    "需要重新确认",
    "用户已拒绝",
    "被安全边界阻断",
    "被诊断阻断",
    "重复执行已阻断",
    "记忆包缺失或过期",
    "读回不可用",
    "读回失败",
    "执行超时",
    "运行失败",
    "停止请求需受控处理",
  ]) {
    assert(agentText.includes(expectedText), `PCR7 Agent 统一执行链路缺少 ${expectedText}`);
  }
  assert(agentText.includes("结果数：未知/不可用"), "PCR7 Agent readback null 应显示未知 / 不可用");
  assert(!agentText.includes("H6 真实执行状态"), "PCR6 Agent 普通 UI 不应继续显示 H6 阶段标题");

  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshotWithProductCommands}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );
  for (const expectedText of ["统一执行命令", "2", "1 等确认", "最近状态", "受控记录已写入", "不等于真实 Codex 自由运行", "未知 / 不可用不显示成 0"]) {
    assert(runningText.includes(expectedText), `PCR6 Running 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["自动编排", "5", "0 等确认", "5 读回未知", "worker report", "捕获来源", "过程观察", "主管复核"]) {
    assert(runningText.includes(expectedText), `K3 Running 自动编排摘要缺少 ${expectedText}`);
  }
  for (const expectedText of ["失败", "读回异常", "停止请求", "需要重新确认", "读回结果：未知 / 不可用"]) {
    assert(runningText.includes(expectedText), `PCR7 Running 统一执行链路缺少 ${expectedText}`);
  }

  const projectDetailText = visibleText(
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      realExecutionProductCommands={activeReadModel}
      projectWorkflowAutomation={activeAutomation}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "统一执行链路",
    "统一命令状态",
    "运行关注",
    "旧派发记录",
    "历史派发记录可见，不是统一产品命令",
    "读回边界",
    "未知 / 不可用",
    "开发者详情：统一命令读模型",
    "失败 / 阻断 / 读回",
    "重新确认",
    "停止请求",
    "读回结果：未知 / 不可用",
    "自动编排",
    "自动编排阶段",
    "编排捕获",
    "编排读回",
    "项目自动编排目标",
    "生成 Level A 编排记录",
    "确认后只写工作台记录、捕获来源和 observation",
    "主管复核",
  ]) {
    assert(projectDetailText.includes(expectedText), `PCR6 Projects 统一执行链路缺少 ${expectedText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: snapshotWithProductCommands,
    workflowState: workflowStateWithProjectWorkflow,
    workflowStateError: null,
  });
  const secretaryText = visibleText(<SecretaryBrief context={secretaryContext} />);
  assert(secretaryText.includes("查看统一执行链路"), "PCR6 秘书应提供统一执行链路查看建议");
  const secretaryProductCommandText = [
    ...secretaryContext.risk_signals.map((risk) => risk.summary),
    ...secretaryContext.suggestions.map((suggestion) => suggestion.summary),
  ].join("\n");
  assert(secretaryProductCommandText.includes("用户已拒绝"), "PCR7 秘书 read model 应解释 product command 失败/停止/重试状态");
  assert(secretaryProductCommandText.includes("需要重新确认"), "PCR7 秘书 read model 应提示重新确认边界");
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "real_execution_product_command_boundary"), "PCR6 秘书风险应包含 product command 边界");
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "project_workflow_automation_boundary"), "K3 秘书风险应包含自动编排边界");
  assert(
    secretaryContext.risk_signals.some((risk) => risk.summary.includes("不能批准、派发、恢复、重试、停止或重启")),
    "PCR6 秘书风险摘要应声明不生成执行类建议",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_real_execution_product_commands"),
    "PCR6 秘书建议应包含统一执行链路查看建议",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_project_workflow_automation"),
    "K3 秘书建议应包含自动编排查看建议",
  );
  assert(
    secretaryContext.suggestions.every((suggestion) =>
      !["approve", "dispatch", "retry", "stop", "restart", "resume", "send"].includes(suggestion.kind),
    ),
    "PCR7 秘书 suggestion kind 不应变成执行类动作",
  );
  for (const forbiddenProposalText of ["批准", "派发", "重试", "stop", "resume", "停止", "恢复"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.toLowerCase().includes(forbiddenProposalText.toLowerCase())),
      `PCR6 秘书 action proposal 不应生成执行动作：${forbiddenProposalText}`,
    );
  }

  const rightPanelText = visibleText(
    <RightDetailPanel
      activePanel="running"
      snapshot={snapshotWithProductCommands}
      workflowState={workflowStateWithProjectWorkflow}
      notice="offline notice"
      error={false}
      workflowStateError={null}
      secretaryContext={secretaryContext}
      onClose={() => {}}
      onNavigate={() => {}}
      onReloadWorkflowState={() => {}}
    />,
  );
  for (const expectedText of ["统一执行链路", "统一执行命令状态", "最近状态：受控记录已写入", "读回未知 / 不可用不能显示成 0"]) {
    assert(rightPanelText.includes(expectedText), `PCR6 Right rail 统一执行链路缺少 ${expectedText}`);
  }
  for (const expectedText of ["失败", "读回异常", "需确认", "停止请求", "停止请求需受控处理", "读回结果：未知 / 不可用"]) {
    assert(rightPanelText.includes(expectedText), `PCR7 Right rail 统一执行链路缺少 ${expectedText}`);
  }

  const combinedMarkup = renderToStaticMarkup(
    <>
      <AgentView sessions={[session]} realExecutionProductCommands={activeReadModel} onRequestAction={captureAction} />
      <RunningWorkflowsView
        snapshot={snapshotWithProductCommands}
        workflowState={workflowStateWithProjectWorkflow}
        workflowStateLoading={false}
        workflowStateError={null}
        onReloadWorkflowState={() => {}}
        onNavigate={() => {}}
      />
    </>,
  );
  for (const forbiddenText of [
    "H5 命令",
    "H6 真实执行状态",
    "允许一次",
    "结果数：0",
    "runRealExecutionProductCommandPhaseA",
    "runRealExecutionProductCommandPhaseB",
    "run_real_execution_product_command_phase_b",
    "confirmRealExecutionProductCommand",
    "recordRealExecutionProductCommandDecision",
    "prepareRealExecutionProductCommand",
  ]) {
    assert(!combinedMarkup.includes(forbiddenText), `PCR6 UI 不应暴露 ${forbiddenText}`);
  }
}

function runStageJRunQueueScenario() {
  const j4FailureItems = [
    ["retry_requires_new_user_confirmation", "需要重新确认", "再次执行前需要新的用户确认；不会自动重试。", "high", true, ["product_command_retry_boundary"], ["pcr7_no_auto_retry_requires_new_user_confirmation"]],
    ["manual_stop_requested", "停止请求需受控处理", "用户请求停止仅作为产品状态，本任务不会停止真实进程。", "medium", false, ["decision:j4:manual_stop"], ["manual_stop_requested_from_decision_reason"]],
    ["duplicate_blocked", "重复执行已阻断", "已有重复命令或运行记录，不能并行继续。", "medium", false, ["preview:j4:duplicate"], ["duplicate_active"]],
    ["readback_failed", "读回失败", "读回尝试失败或不可信，结果数未知。", "high", true, ["attempt:j4:readback_failed"], ["readback_parser_failed"]],
    ["timed_out", "执行超时", "执行或读回超时，不能解释为已经完成停止。", "high", true, ["attempt:j4:timed_out"], ["readback_timed_out"]],
  ].map(([kind, title, summary, severity, requiresNewUserConfirmation, sourceRefs, warnings]) => ({
    kind: kind as string,
    title: title as string,
    summary: summary as string,
    count: 1,
    severity: severity as string,
    requires_new_user_confirmation: requiresNewUserConfirmation as boolean,
    result_count: null,
    source_refs: sourceRefs as string[],
    warnings: warnings as string[],
  }));
  const activeReadModel = {
    ...snapshot.real_execution_product_commands!,
    store_available: true,
    command_count: 3,
    pending_decision_count: 1,
    running_attempt_count: 1,
    blocked_attempt_count: 1,
    last_attempt_status: "readback_failed",
    failure_stop_retry_summary: {
      schema_version: "real_execution_product_command_failure_stop_retry.v1",
      item_count: j4FailureItems.length,
      failure_count: 2,
      blocked_count: 1,
      readback_issue_count: 2,
      manual_stop_requested_count: 1,
      retry_requires_new_user_confirmation: true,
      items: j4FailureItems,
      warnings: ["j4_failure_control_fixture"],
    },
    warnings: ["j4_readback_null_fixture"],
  };
  const workflowId = workflowStateWithProjectWorkflow.project_workflows[0]?.workflow_id ?? "workflow:j4:fixture";
  const activeAutomation = {
    schema_version: "project_workflow_automation.v1",
    available: true,
    generated_at: "2026-06-09T12:00:00Z",
    latest_automation_id: "j4-automation:offline",
    latest_status: "phase_a_closed_loop_recorded",
    latest_plan: {
      schema_version: "project_workflow_automation.v1",
      automation_id: "j4-automation:offline",
      project_id: "project:offline",
      project_root: project.project_root,
      workflow_id: workflowId,
      user_goal: "离线验证 J4 运行队列。",
      current_phase: "director_final_review",
      next_step: "等待用户处理 J4 待确认队列。",
      run_units: [
        {
          run_unit_id: "run-unit:j4:developer",
          run_unit_kind: "developer_execution",
          role: "developer_execution",
          status: "readback_unavailable",
          project_id: "project:offline",
          project_root: project.project_root,
          workflow_id: workflowId,
          workflow_node_id: `${workflowId}:node:codex-dev`,
          work_item_id: "work-item:j4:developer",
          task_package_ref: "task-package:j4",
          memory_packet_ref: "memory-packet:j4",
          product_command_preview_ref: "preview:j4:developer",
          product_command_ref: "product-command:j4:developer",
          runtime_log_refs: ["runtime-log:j4:developer"],
          audit_refs: ["audit:j4:developer"],
          readback_ref: "readback:j4:developer",
          readback_status: "readback_unavailable",
          readback_result_count: null,
          worker_report_ref: null,
          capture_event_refs: ["capture:j4:compensation"],
          observation_refs: [],
          memory_candidate_refs: [],
          runner_call_allowed: false,
          prompt_sent: false,
          real_codex_executed: false,
          writes_codex_home: false,
          writes_project_files: false,
          summary: "开发线读回不可用，结果数未知。",
          next_step: "等待人工查看读回边界。",
          blocked_reasons: [],
          warnings: ["unknown_readback_result_count_must_remain_null"],
        },
        {
          run_unit_id: "run-unit:j4:collector",
          run_unit_kind: "collector_summary",
          role: "collector_summary",
          status: "completed",
          project_id: "project:offline",
          project_root: project.project_root,
          workflow_id: workflowId,
          workflow_node_id: `${workflowId}:node:collector`,
          work_item_id: "work-item:j4:collector",
          task_package_ref: "task-package:j4",
          memory_packet_ref: "memory-packet:j4",
          product_command_preview_ref: null,
          product_command_ref: null,
          runtime_log_refs: ["runtime-log:j4:collector"],
          audit_refs: ["audit:j4:collector"],
          readback_ref: null,
          readback_status: "unknown",
          readback_result_count: null,
          worker_report_ref: "worker-report:j4",
          capture_event_refs: ["capture:j4:candidate"],
          observation_refs: ["observation:j4:collector"],
          memory_candidate_refs: [],
          runner_call_allowed: false,
          prompt_sent: false,
          real_codex_executed: false,
          writes_codex_home: false,
          writes_project_files: false,
          summary: "回收线已形成观察。",
          next_step: "等待主管复核。",
          blocked_reasons: [],
          warnings: [],
        },
        {
          run_unit_id: "run-unit:j4:review",
          run_unit_kind: "director_final_review",
          role: "global_director",
          status: "needs_review",
          project_id: "project:offline",
          project_root: project.project_root,
          workflow_id: workflowId,
          workflow_node_id: `${workflowId}:node:review`,
          work_item_id: "work-item:j4:review",
          task_package_ref: "task-package:j4",
          memory_packet_ref: "memory-packet:j4",
          product_command_preview_ref: null,
          product_command_ref: null,
          runtime_log_refs: [],
          audit_refs: ["audit:j4:review"],
          readback_ref: null,
          readback_status: "unknown",
          readback_result_count: null,
          worker_report_ref: "worker-report:j4",
          capture_event_refs: [],
          observation_refs: [],
          memory_candidate_refs: [],
          runner_call_allowed: false,
          prompt_sent: false,
          real_codex_executed: false,
          writes_codex_home: false,
          writes_project_files: false,
          summary: "主管复核等待确认。",
          next_step: "确认过程事实。",
          blocked_reasons: [],
          warnings: [],
        },
      ],
      blocked_reasons: [],
      warnings: ["j4_queue_fixture"],
    },
    run_unit_count: 3,
    waiting_user_count: 1,
    blocked_count: 0,
    readback_unknown_count: 3,
    worker_report_count: 2,
    capture_event_count: 2,
    observation_count: 1,
    next_step: "等待用户处理 J4 待确认队列。",
    warnings: ["j4_run_queue_fixture"],
  };
  const memoryCaptureStore: MemoryCaptureStoreV1 = {
    store_version: "memory_capture_store.v1",
    project_id: "project:offline",
    workflow_id: workflowId,
    revision: 2,
    events: [
      {
        capture_event_id: "capture:j4:candidate",
        event_key: "j4:candidate",
        schema_version: "memory_capture_event.v1",
        source_type: "worker_report",
        source_ref_id: "worker-report:j4",
        project_id: "project:offline",
        workflow_id: workflowId,
        workflow_node_id: `${workflowId}:node:collector`,
        run_unit_id: "run-unit:j4:collector",
        product_command_id: null,
        product_attempt_id: null,
        runtime_log_ref: "runtime-log:j4:collector",
        audit_refs: ["audit:j4:collector"],
        readback_ref: null,
        task_package_ref: "task-package:j4",
        memory_packet_ref: "memory-packet:j4",
        summary: "J4 运行事实可以形成候选。",
        evidence_summary: "离线 fixture。",
        sensitivity: "internal",
        candidate_policy: "candidate_allowed",
        blocked_reason: null,
        observation_id: "observation:j4:collector",
        candidate_key: "memory-candidate:j4:collector",
        created_by: "project_director",
        created_at: "2026-06-09T12:00:00Z",
        updated_at: "2026-06-09T12:00:00Z",
      },
      {
        capture_event_id: "capture:j4:compensation",
        event_key: "j4:compensation",
        schema_version: "memory_capture_event.v1",
        source_type: "readback",
        source_ref_id: "readback:j4:developer",
        project_id: "project:offline",
        workflow_id: workflowId,
        workflow_node_id: `${workflowId}:node:codex-dev`,
        run_unit_id: "run-unit:j4:developer",
        product_command_id: "product-command:j4:developer",
        product_attempt_id: "attempt:j4:developer",
        runtime_log_ref: "runtime-log:j4:developer",
        audit_refs: ["audit:j4:developer"],
        readback_ref: "readback:j4:developer",
        task_package_ref: "task-package:j4",
        memory_packet_ref: "memory-packet:j4",
        summary: "候选链路半完成。",
        evidence_summary: "离线 fixture。",
        sensitivity: "internal",
        candidate_policy: "candidate_allowed",
        blocked_reason: null,
        observation_id: null,
        candidate_key: null,
        created_by: "project_director",
        created_at: "2026-06-09T12:01:00Z",
        updated_at: "2026-06-09T12:01:00Z",
      },
    ],
    updated_at: "2026-06-09T12:01:00Z",
    warnings: [],
  };
  const memoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    project_id: "project:offline",
    workflow_id: workflowId,
    revision: 1,
    candidates: [
      {
        candidate_id: "memcand:j4:formalization",
        candidate_key: "memcand:v1:j4-formalization",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:j4",
          scope_type: "project",
          project_id: "project:offline",
          workflow_id: workflowId,
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-09T12:00:00Z",
        },
        memory_type: "workflow_summary",
        claim: "J4 运行队列需要保留候选正式化边界。",
        body: "已确认候选仍不是正式记忆，需要继续走正式化生命周期。",
        source_refs: [
          {
            source_ref_id: "source:j4:formalization",
            source_type: "observation_ref",
            source_id: "capture:j4:candidate",
            source_title: "J4 capture candidate",
            captured_at: "2026-06-09T12:00:00Z",
            authority_level: "derived_summary",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "project_director",
        generated_from: "observation:observation:j4:collector",
        status: "candidate_confirmed",
        risk_level: "medium",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "J4 正式化确认 fixture。",
        conflicts: [],
        audit_refs: [],
        adoption: null,
        created_at: "2026-06-09T12:00:00Z",
        updated_at: "2026-06-09T12:00:00Z",
      },
    ],
    events: [],
    updated_at: "2026-06-09T12:00:00Z",
  };
  const j4RuntimeAttention: RuntimeSessionAttention[] = [
    {
      attention_id: "attention:j4:readback-failed",
      project_id: "project:offline",
      workflow_id: workflowId,
      node_id: `${workflowId}:node:codex-dev`,
      session_id: "session:j4",
      adapter_id: "codex-local",
      source_refs: [{ source_kind: "runtime_log", source_id: "runtime-log:j4:readback", label: "读回日志" }],
      kind: "readback_failed",
      severity: "blocking",
      status: "readback_failed",
      title: "读回失败，等待人工处理",
      user_message: "读回失败不能当成 0 条结果。",
      technical_summary: "offline fixture",
      recommended_next_step: "查看运行队列和失败控制。",
      requires_user_action: true,
      blocks_continuation: true,
      readback_boundary: {
        status: "readback_failed",
        reason: "readback_parser_failed",
        attempted: true,
        real_readback_performed: false,
        result_count: null,
        user_message: "读回失败，结果数未知。",
        technical_summary: "offline fixture",
        source_refs: [{ source_kind: "runtime_log", source_id: "runtime-log:j4:readback", label: "读回日志" }],
        warnings: ["unknown_readback_result_count_must_remain_null"],
      },
      created_at: "2026-06-09T12:02:00Z",
      updated_at: "2026-06-09T12:02:00Z",
      warnings: [],
    },
  ];
  const j4Snapshot: WorkbenchSnapshot = {
    ...snapshot,
    runtime_session_attention: j4RuntimeAttention,
    real_execution_product_commands: activeReadModel,
    project_workflow_automation: activeAutomation,
  };

  const readModel = deriveRunQueueReadModel({
    snapshot: j4Snapshot,
    workflowState: workflowStateWithProjectWorkflow,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  assert(readModel.schema_version === "run_queue_read_model.v1", "J4 run queue schema 不匹配");
  assert(readModel.operation_control_summary.schema_version === "operation_control_summary.v1", "K5 operation control schema 不匹配");
  assert(readModel.operation_control_summary.true_operation_available === false, "K5 操作控制摘要不应声明真实操作可用");
  assert(readModel.operation_control_summary.retry_proposal_count >= 1, "K5 应汇总重试确认或重试提案");
  assert(readModel.operation_control_summary.stop_request_count >= 1, "K5 应汇总停止 / 取消确认");
  assert(readModel.operation_control_summary.restart_readiness_count === 0, "K5 不应声明真实重启准备完成");
  assert(readModel.operation_control_summary.resume_readiness_count === 0, "K5 不应声明真实恢复准备完成");
  assert(readModel.operation_control_summary.readback_issue_count >= 2, "K5 应汇总读回异常");
  assert(readModel.operation_control_summary.duplicate_blocked_count >= 1, "K5 应汇总重复阻断");
  assert(readModel.operation_control_summary.manual_review_count >= 1, "K5 应汇总人工复核事项");
  assert(
    readModel.operation_control_summary.warnings.includes("no_auto_retry_stop_restart_resume"),
    "K5 操作控制必须声明不会自动重试/停止/重启/恢复",
  );
  assert(readModel.run_queue_items.some((item) => item.status === "readback_unavailable" && item.readback_result_count === null), "J4 readback_unavailable 应保持 result_count=null");
  assert(readModel.run_queue_items.some((item) => item.status === "readback_failed" && item.readback_result_count === null), "J4 readback_failed 应保持 result_count=null");
  assert(readModel.failure_control_summaries.some((item) => item.status === "timed_out" && item.readback_result_count === null), "J4 timed_out 应保持 result_count=null");
  assert(readModel.failure_control_summaries.some((item) => item.classification === "duplicate_blocked"), "J4 duplicate blocked 应进入失败控制");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "execute_confirmation"), "J4 应包含执行确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "retry_confirmation"), "J4 应包含重试确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "stop_cancel_confirmation"), "J4 应包含停止 / 取消确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "result_confirmation"), "J4 应包含结果确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "process_fact_confirmation"), "J4 应包含过程事实确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "memory_candidate_confirmation"), "J4 应包含记忆候选确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "memory_formalization_confirmation"), "J4 应包含正式化确认");
  assert(readModel.user_confirmation_queue.some((item) => item.kind === "capture_compensation_confirmation"), "J4 应包含 capture 补偿确认");
  assert(readModel.capture_compensation_count === 1, "J4 capture 半完成状态应进入补偿摘要");
  assert(
    readModel.user_confirmation_queue.every((item) => item.confirmation_command_kind !== "runner_call"),
    "J4 确认队列不应直接调用 runner",
  );
  assert(
    readModel.user_confirmation_queue.every((item) => !item.writes_codex_home),
    "J4 默认确认队列不应声明写 .codex",
  );

  const runningText = visibleText(
    <RunningWorkflowsView
      snapshot={j4Snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      onReloadWorkflowState={() => {}}
      onNavigate={() => {}}
    />,
  );
  for (const expectedText of [
    "运行队列",
    "待确认",
    "失败控制",
    "操作控制 / 恢复建议",
    "重试提案",
    "停止请求",
    "重启准备",
    "恢复准备",
    "只读建议",
    "后续任务",
    "单独授权",
    "不执行真实恢复命令",
    "不清理真实 Codex 本地状态",
    "重试确认",
    "停止 / 取消确认",
    "过程事实确认",
    "记忆候选确认",
    "正式化确认",
    "捕获补偿确认",
    "重复执行已阻断",
    "候选不是正式记忆",
    "结果数：未知 / 不可用",
    "不会自动调用 runner",
  ]) {
    assert(runningText.includes(expectedText), `K5 Running UI 缺少 ${expectedText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: j4Snapshot,
    workflowState: workflowStateWithProjectWorkflow,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  assert(secretaryContext.risk_signals.some((risk) => risk.kind === "run_queue_boundary"), "J4 秘书风险应包含运行队列边界");
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "run_queue_boundary" && risk.summary.includes("捕获补偿 1")),
    "J4 秘书风险应包含 capture compensation 计数",
  );
  assert(secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_run_queue"), "J4 秘书建议应包含查看运行队列");
  assert(
    secretaryContext.action_proposals.every((proposal) => !["retry", "stop", "restart", "resume", "send"].includes(proposal.kind)),
    "J4 秘书 action proposal 不应变成执行动作",
  );

  const rightPanelText = visibleText(
    <RightDetailPanel
      activePanel="running"
      snapshot={j4Snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      notice="offline notice"
      error={false}
      workflowStateError={null}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      secretaryContext={secretaryContext}
      onClose={() => {}}
      onNavigate={() => {}}
      onReloadWorkflowState={() => {}}
    />,
  );
  for (const expectedText of ["运行队列", "待确认", "失败控制", "捕获补偿", "不自动执行", "记忆候选确认", "正式化确认", "捕获补偿确认"]) {
    assert(rightPanelText.includes(expectedText), `J4 Right rail 缺少 ${expectedText}`);
  }

  const combinedMarkup = renderToStaticMarkup(
    <>
      <RunningWorkflowsView
        snapshot={j4Snapshot}
        workflowState={workflowStateWithProjectWorkflow}
        workflowStateLoading={false}
        workflowStateError={null}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        onReloadWorkflowState={() => {}}
        onNavigate={() => {}}
      />
      <RightDetailPanel
        activePanel="running"
        snapshot={j4Snapshot}
        workflowState={workflowStateWithProjectWorkflow}
        notice="offline notice"
        error={false}
        workflowStateError={null}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        secretaryContext={secretaryContext}
        onClose={() => {}}
        onNavigate={() => {}}
        onReloadWorkflowState={() => {}}
      />
    </>,
  );
  for (const forbiddenText of ["自动重试中", "已自动修复", "已写正式记忆", "结果数：0", "runner_call", "codex exec resume", "已停止", "已重启", "已恢复", "已 resume"]) {
    assert(!combinedMarkup.includes(forbiddenText), `K5 UI 不应出现误导文案：${forbiddenText}`);
  }
}

function runProjectCanvasReadModelScenario() {
  const boundaryText = JSON.stringify([experimentCanvasBoundary, projectWorkflowCanvasBoundary]);
  assert(experimentCanvasBoundary.context_kind === "experiment_canvas", "F4 一级画布边界应声明 experiment canvas 语境");
  assert(projectWorkflowCanvasBoundary.context_kind === "project_workflow_canvas", "F4 项目画布边界应声明 project workflow canvas 语境");
  for (const expectedText of [
    "实验 / 模板画布",
    "experiment / template / canvas library",
    "不会写项目事实",
    "不会写正式记忆",
    "不会写项目工作流状态",
    "不是项目 workflow 事实源",
    "MCP canvas run 非默认项目工作流",
    "项目工作流画布",
    "工作流状态派生读模型",
    "方案授权 / 控制核心 / 权限 / 审计",
    "React Flow 仅负责渲染",
    "实验画布不会写入本项目事实",
  ]) {
    assert(boundaryText.includes(expectedText), `F4 画布边界声明缺少 ${expectedText}`);
  }
  for (const forbiddenText of canvasBoundaryForbiddenPhrases) {
    assert(!boundaryText.includes(forbiddenText), `F4 画布边界声明不应出现误导文案 ${forbiddenText}`);
  }

  const projectWorkflow = workflowStateWithDerivedWorkflow.project_workflows[0];
  const selectedTask = projectWorkflow.task_drafts[0];
  const model = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow,
    projectBlackboard: workflowStateWithDerivedWorkflow.project_blackboards?.[0] ?? null,
    selectedTask,
    workflowStatePath: workflowStateWithDerivedWorkflow.path,
    runtimeSessionAttention: [
      {
        ...runtimeAttentionFixture("canvas-readback-unavailable", "readback_unavailable", "warning", "readback_unavailable", "not_attempted_stub", true, false),
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
      },
    ],
  });

  assert(model.schema_version === "project_workflow_canvas.v1", "项目画布读模型 schema 不匹配");
  assert(model.source.source_kind === "workflow_state_read_model", "项目画布必须声明来自 workflow state 读模型");
  assert(model.status === "waiting_for_permission", "pending 权限请求应让画布进入等待权限状态");
  assert(model.status_reason.label === "等待权限", "画布缺少状态原因标签");
  assert(model.attention_items.some((item) => item.kind === "waiting_for_permission"), "画布 attention 缺少权限待处理");
  assert(model.attention_items.some((item) => item.kind === "readback_unavailable"), "画布 attention 缺少 readback unavailable");
  assert(model.nodes.some((node) => node.node_type === "project_goal"), "项目画布缺少项目目标节点");
  assert(model.nodes.some((node) => node.node_type === "director"), "项目画布缺少总指导节点");
  assert(model.nodes.some((node) => node.node_type === "dev_line"), "项目画布缺少开发线节点");
  assert(model.nodes.some((node) => node.node_type === "validation_line"), "项目画布缺少验证线节点");
  assert(model.nodes.some((node) => node.node_type === "review_line"), "项目画布缺少回收线节点");
  assert(model.nodes.some((node) => node.node_type === "permission_request"), "项目画布缺少权限请求 sidecar 节点");
  assert(model.nodes.some((node) => node.node_type === "blackboard_candidate"), "项目画布缺少黑板候选 sidecar 节点");
  assert(model.edges.some((edge) => edge.edge_type === "responsibility_flow"), "项目画布缺少责任流转边");
  assert(model.edges.some((edge) => edge.edge_type === "blocking_relation"), "项目画布缺少阻塞关系边");
  assert(model.viewport_hint.selected_node_id.includes(":canvas:codex-dev"), "默认选中节点应落在当前派发角色");
  assert(model.edit_boundary.source_kind === "frontend_read_model", "F3 编辑边界应来自前端只读模型");
  assert(model.edit_boundary.layout_boundary.react_flow_source_of_truth === false, "画布渲染层不应成为 workflow authority");
  assert(model.edit_boundary.layout_boundary.writes_workflow_state === false, "布局不应写 workflow state");
  assert(model.edit_boundary.layout_boundary.persists_layout === false, "F3 不应持久化布局");
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "local_layout_preview" && capability.status === "allowed" && !capability.changes_workflow_facts),
    "F3 应允许本地视图布局预览且不改事实",
  );
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "personal_layout_preference" && capability.status === "requires_future_task"),
    "F3 不应实现个人布局持久化",
  );
  for (const mutationKind of ["workflow_node_mutation", "workflow_edge_mutation"] as const) {
    assert(
      model.edit_boundary.capabilities.some(
        (capability) =>
          capability.kind === mutationKind &&
          capability.status === "preview_only" &&
          capability.changes_workflow_facts &&
          capability.requires_proposal &&
          capability.requires_control_core &&
          capability.requires_audit,
      ),
      `F3 ${mutationKind} 应只允许 proposal preview`,
    );
  }
  assert(
    model.edit_boundary.capabilities.some(
      (capability) =>
        capability.kind === "permission_or_model_mutation" &&
        capability.status === "blocked" &&
        capability.requires_confirmation &&
        capability.requires_control_core &&
        capability.requires_audit,
    ),
    "F3 高风险权限 / 模型变更必须被阻断并要求确认、控制核心和审计",
  );
  assert(
    model.edit_boundary.capabilities.some((capability) => capability.kind === "execution_mutation" && capability.status === "blocked"),
    "F3 不应允许执行变更",
  );
  assert(
    model.edit_boundary.proposal_previews.some(
      (preview) => preview.change_kind === "workflow_node_mutation" && preview.status === "preview_only" && preview.requires_proposal,
    ),
    "F3 节点变更缺少 proposal preview",
  );

  const selectedNode = model.nodes.find((node) => node.node_id === model.viewport_hint.selected_node_id);
  assert(selectedNode, "默认选中节点不存在");
  const detail = selectedNode ? model.detail_panels[selectedNode.detail_panel_id] : null;
  assert(detail, "默认选中节点缺少详情面板");
  const detailKinds = detail?.sections.map((section) => section.kind) ?? [];
  const detailLayers = detail?.sections.map((section) => section.layer) ?? [];
  for (const expectedLayer of ["user_summary", "project_director", "technical_details"]) {
    assert(detailLayers.includes(expectedLayer as never), `节点详情缺少 ${expectedLayer} 层`);
  }
  for (const expectedKind of ["summary", "task_package", "memory_packet", "session_binding", "dispatch", "readback", "permission_requests", "blackboard_entries", "completion_gate", "audit_refs"]) {
    assert(detailKinds.includes(expectedKind as never), `节点详情缺少 ${expectedKind}`);
  }
  const userSummary = detail?.sections.find((section) => section.layer === "user_summary");
  for (const expectedLabel of ["当前节点", "当前状态", "为什么停下", "谁能处理", "下一步"]) {
    assert(userSummary?.items.some((item) => item.label === expectedLabel), `用户摘要缺少 ${expectedLabel}`);
  }
  assert(
    detail?.sections.some((section) => section.kind === "source_refs" && section.layer === "technical_details"),
    "技术详情缺少 source refs 摘要",
  );
  assert(
    detail?.allowed_actions.some((action) => action.action_kind === "record_permission_decision" && action.enabled),
    "待权限节点详情应暴露权限结论动作说明",
  );
  assert(!model.source.derived_from.some((source) => source.kind === "audit_event" && source.id.includes("transcript")), "画布读模型不应引用完整 transcript");
  assert(
    detail?.sections.some((section) =>
      section.kind === "memory_packet" &&
      section.items.some((item) => item.value.includes("候选和观察不会当作正式记忆注入") || item.item_id === "memory-snapshot"),
    ),
    "节点详情缺少任务记忆包摘要边界",
  );
  assert(
    detail?.sections.some((section) =>
      section.kind === "readback" &&
      section.items.some((item) => item.value.includes("0 条") || item.value.includes("有摘要") || item.value.includes("events")),
    ),
    "节点详情缺少读回摘要",
  );

  const examples = projectCanvasStateExamples();
  assertDeepEqual(
    examples.map((example) => example.example_id),
    ["empty", "four_roles", "prepared", "running", "needs_review", "waiting_permission", "blocked", "failed", "timed_out", "readback_unavailable", "reviewing", "accepted"],
    "画布组件状态样例清单不匹配",
  );
  assert(examples.some((example) => example.permission_queue === "pending"), "状态样例缺少权限队列 pending 态");
  assert(examples.some((example) => example.detail_sections.includes("blackboard_entries") || example.description.includes("候选")), "状态样例缺少黑板候选基准");
  assert(examples.some((example) => example.status === "prepared"), "状态样例缺少 prepared 态");
  assert(examples.some((example) => example.status === "readback_unavailable"), "状态样例缺少 readback unavailable 态");

  const preparedWorkflow = {
    ...projectWorkflow,
    permission_requests: [],
    execution_attempts: [],
    task_drafts: [{ ...selectedTask, state: "prepared" }],
    node_dispatches: [
      {
        ...projectWorkflow.node_dispatches[0],
        state: "prepared",
        last_message_summary: null,
        transcript_event_count: null,
        transcript_target_hits: null,
        warnings: ["prepared_dispatch_is_not_worker_execution"],
      },
    ],
  };
  const preparedModel = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow: preparedWorkflow,
    projectBlackboard: null,
    selectedTask: preparedWorkflow.task_drafts[0],
  });
  assert(preparedModel.status === "prepared", "prepared dispatch 应进入准备派发状态");
  assert(preparedModel.attention_items.some((item) => item.summary.includes("仍未启动工作者")), "准备态关注项不应暗示工作者已执行");

  const emptyModel = deriveProjectWorkflowCanvasReadModel({
    project,
    projectWorkflow: null,
    projectBlackboard: null,
    selectedTask: null,
  });
  assert(emptyModel.status === "empty", "缺 workflow 应进入空态");
  assert(emptyModel.attention_items.some((item) => item.summary.includes("不补编任务")), "空态不应补编工作项");
}

function runAdapterCapabilityScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(descriptors.length === 5, "E1 应返回 Codex 和四个计划中的 adapter descriptor");
  const codex = descriptors[0];
  assert(codex.adapter_id === "codex-local", "Codex adapter id 不匹配");
  assert(codex.agent_type === "codex", "Codex adapter agent_type 不匹配");
  assert(codex.status === "available", "有 Codex 会话和绑定时 adapter 应 available");
  assert(codex.source_kind === "frontend_read_model", "适配器能力声明应是前端读模型");
  assert(codex.execution_status === "available_with_user_confirmation", "Codex 执行状态应要求用户确认");
  assert(codex.credential_status === "not_read", "Codex descriptor 不应读取凭据");
  assert(codex.model_access_status === "local_read_model_only", "Codex 模型状态只能是本地读模型摘要");
  assert(codex.warnings.includes("adapter_descriptor_frontend_fallback_used"), "前端派生 helper 应声明 fallback 警告");
  assert(codex.hidden_unimplemented_adapters.includes("openclaw"), "未实现 OpenClaw 应隐藏");
  assert(codex.hidden_unimplemented_adapters.includes("claude-code"), "未实现 Claude Code 应隐藏");
  assert(codex.hidden_unimplemented_adapters.includes("opencode-like"), "OpenCode-like 应进入未实现清单");
  assert(codex.implemented_action_kinds.includes("bind-node-session"), "Codex adapter 缺少节点绑定动作声明");
  assert(codex.implemented_action_kinds.includes("execute-node-dispatch"), "Codex adapter 缺少派发动作声明");
  const plannedAdapters = descriptors.filter((descriptor) => descriptor.adapter_id !== "codex-local");
  assert(plannedAdapters.length === 4, "应包含四个计划中的 adapter descriptor");
  for (const planned of plannedAdapters) {
    assert(planned.status === "planned", `${planned.adapter_id} 必须是计划中状态`);
    assert(planned.execution_status === "not_implemented", `${planned.adapter_id} 不能有真实执行能力`);
    assert(planned.credential_status === "not_configured", `${planned.adapter_id} 凭据状态必须是未配置`);
    assert(planned.model_access_status === "not_verified", `${planned.adapter_id} 模型访问状态必须是未验证`);
    assert(planned.implemented_action_kinds.length === 0, `${planned.adapter_id} 不能声明已实现动作`);
    assert(planned.capabilities.every((capability) => capability.status !== "available"), `${planned.adapter_id} 不能有 available 能力`);
    assert(planned.warnings.includes("no_execution_button"), `${planned.adapter_id} 必须声明无执行按钮边界`);
  }
  for (const expectedCapability of [
    "session_index_read",
    "session_transcript_read",
    "workflow_node_binding",
    "safe_probe_dispatch",
    "user_reviewed_dispatch",
    "workflow_machine_run",
    "permission_decision_record",
    "harness_resource_index",
  ]) {
    assert(
      codex.capabilities.some((capability) => capability.kind === expectedCapability),
      `Codex adapter 缺少能力声明 ${expectedCapability}`,
    );
  }
  assert(
    codex.capabilities
      .filter((capability) => capability.status === "requires_confirmation")
      .every((capability) => capability.boundary.includes("本轮只声明能力") || capability.boundary.includes("控制核心") || capability.boundary.includes("工作流状态")),
    "需确认能力必须标明声明边界或控制核心边界",
  );
}

function runSessionOperationBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  assert(operations.length === 40, "H3.1 后应为 5 个 adapter 派生 40 条会话操作边界");

  const expectedOperationIds = ["new_session", "send_message", "stop", "restart", "resume", "export", "delete", "favorite"] as const;
  for (const operationId of expectedOperationIds) {
    assert(
      operations.filter((operation) => operation.operation_id === operationId).length === 5,
      `E2 缺少 ${operationId} per-adapter 边界`,
    );
  }
  assert(
    operations.every((operation) => !["available", "available_to_execute", "executable"].includes(operation.current_status)),
    "E2 不允许任何会话操作进入可执行状态",
  );
  assert(
    operations.every((operation) => operation.warnings.includes("session_operation_boundary_read_model_only")),
    "会话操作必须声明只读边界",
  );
  assert(
    operations.every((operation) => operation.warnings.includes("no_session_operation_execution_in_e2")),
    "会话操作必须声明 E2 不执行",
  );

  const codexNewSession = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "new_session");
  assert(codexNewSession?.current_status === "requires_future_task", "Codex 新会话必须需要后续任务");
  assert(codexNewSession?.writes_codex_home, "新会话真实实现前应显式声明 Codex home 写入影响");
  assert(codexNewSession?.requires_user_confirmation, "新会话真实实现前必须要求用户确认");
  assert(
    codexNewSession?.warnings.includes("h3_1_new_session_noop_only"),
    "H3.1 新会话必须声明 no-op only",
  );

  const codexSend = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "send_message");
  assert(codexSend?.current_status === "requires_future_task", "Codex 发消息必须需要后续任务");
  assert(codexSend?.writes_codex_home, "发消息真实实现前应显式声明 Codex home 写入影响");
  assert(codexSend?.requires_user_confirmation, "发消息真实实现前必须要求用户确认");

  const codexResume = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "resume");
  assert(codexResume?.current_status === "requires_future_task", "Codex resume 必须需要后续任务");
  assert(
    codexResume?.warnings.includes("workflow_dispatch_is_not_session_center_resume"),
    "workflow dispatch resume 不能被等同为会话中心 resume",
  );

  const deleteOperations = operations.filter((operation) => operation.operation_id === "delete");
  assert(
    deleteOperations.every((operation) => operation.current_status === "blocked_destructive" && operation.risk_level === "destructive"),
    "删除必须全部是破坏性阻断",
  );
  assert(
    deleteOperations.every((operation) => operation.warnings.includes("destructive_operation_blocked")),
    "删除必须包含破坏性阻断 warning",
  );

  const plannedOperations = operations.filter((operation) => operation.adapter_id !== "codex-local");
  assert(
    plannedOperations.every((operation) => operation.warnings.includes("planned_adapter_operation_not_available")),
    "planned adapter 会话操作必须保持不可用",
  );
  assert(
    plannedOperations.every((operation) => operation.applies_to_session_state === "planned_adapter_without_session_source"),
    "planned adapter 不应伪造会话事实源",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={backendAgentAdapterDescriptors}
      sessionOperationDescriptors={operations}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "会话操作边界",
    "只读历史浏览器",
    "新会话预览",
    "H3.1 只实现新会话 request",
    "发消息",
    "需要后续任务",
    "停止",
    "当前不可执行",
    "resume",
    "会话中心通用 resume",
    "导出",
    "计划中",
    "删除",
    "破坏性阻断",
    "收藏",
    "计划中不可执行",
    "不执行新建会话、发消息、停止、重启、恢复、导出、删除或收藏",
  ]) {
    assert(agentViewText.includes(expectedText), `会话操作边界 UI 缺少 ${expectedText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["新建会话", "新会话预览", "发消息", "停止", "重启", "resume", "导出", "删除", "收藏"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `会话操作边界不应渲染可点击按钮：${forbiddenButtonText}`);
  }
  for (const forbiddenText of ["真实新会话已创建", "已创建真实会话", "已发送", "已停止", "已重启", "已 resume", "已导出", "已删除", "已收藏"]) {
    assert(!agentViewText.includes(forbiddenText), `会话操作边界不应出现误导文案：${forbiddenText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      session_operations: operations,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "session_operation_boundary"),
    "秘书风险应包含会话操作边界提醒",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_session_operation_boundary"),
    "秘书建议应包含查看会话操作边界",
  );
  for (const forbiddenProposalText of ["新建会话", "发消息", "停止", "重启", "resume", "导出", "删除", "收藏"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成会话操作：${forbiddenProposalText}`,
    );
  }
}

function runProviderAvailabilityBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  assert(summaries.length === 5, "E3 应为 5 个 adapter 派生 provider availability 摘要");
  assert(summaries.every((summary) => summary.safe_to_display), "E3 摘要必须可安全展示");
  assert(
    summaries.every((summary) => summary.warnings.includes("provider_availability_read_model_only")),
    "E3 摘要必须声明只读 provider availability",
  );
  assert(
    summaries.every((summary) => summary.warnings.includes("credential_secret_not_read")),
    "E3 摘要必须声明不读取 secret",
  );
  assert(
    summaries.every((summary) => summary.warnings.includes("provider_availability_not_project_authorization")),
    "E3 摘要必须声明不等于项目授权",
  );

  const codex = summaries.find((summary) => summary.adapter_id === "codex-local");
  assert(codex, "E3 缺少 codex-local provider 摘要");
  assert(codex.provider_kind === "local_cli", "codex-local provider kind 应是 local_cli");
  assert(codex.availability_status === "available_readonly", "codex-local 只能是只读可见");
  assert(codex.credential_status === "not_required_by_workbench", "codex-local 不应要求工作台读取凭据");
  assert(codex.model_status === "local_cli_managed", "codex-local 模型状态应由本地 CLI 管理");
  assert(codex.external_call_status === "not_needed_for_readonly", "codex-local 只读摘要不需要外发调用");
  assert(codex.cost_risk_status === "unknown", "codex-local 成本风险第一版应保持未知");

  const plannedSummaries = summaries.filter((summary) => summary.adapter_id !== "codex-local");
  assert(plannedSummaries.length === 4, "E3 planned provider 数量不匹配");
  assert(
    plannedSummaries.every(
      (summary) =>
        summary.availability_status === "planned" &&
        summary.credential_status === "credential_missing" &&
        summary.model_status === "model_unverified" &&
        summary.external_call_status === "external_call_blocked" &&
        summary.cost_risk_status === "blocked_until_authorized",
    ),
    "planned adapters 必须保持未配置、未验证和外发阻断",
  );
  assert(
    plannedSummaries.every((summary) => summary.requires_user_configuration && summary.requires_future_task),
    "planned provider 必须需要后续任务或用户设置",
  );

  const serializedSummaries = JSON.stringify(summaries);
  for (const forbiddenFragment of ["api_key", "oauth", "keychain", ".env", "available_to_execute", "provider_verified"]) {
    assert(!serializedSummaries.toLowerCase().includes(forbiddenFragment), `E3 摘要不应包含 ${forbiddenFragment}`);
  }

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "供应方 / 模型 / 凭据边界",
    "只读供应方可用性",
    "不等于项目授权",
    "Codex 本地 CLI",
    "本地 CLI 管理",
    "工作台不读取",
    "模型未验证",
    "外发调用已阻断",
    "授权前阻断",
    "planned_adapter_not_connected",
    "provider_availability_not_project_authorization",
    "no_external_provider_call_in_e3",
  ]) {
    assert(agentViewText.includes(expectedText), `Provider availability UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已配置凭据",
    "模型已验证",
    "外部模型已可用",
    "Claude Code 已接入",
    "OpenClaw 已接入",
    "OpenCode 已接入",
    "provider 已验证",
    "测试调用成功",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `Provider availability UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["配置凭据", "验证模型", "测试 provider", "调用模型", "dispatch"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `Provider availability 不应渲染可点击按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "provider_availability_boundary"),
    "秘书风险应包含 provider availability 边界提醒",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_provider_availability_boundary"),
    "秘书建议应包含查看模型与凭据边界",
  );
  for (const forbiddenProposalText of ["配置凭据", "验证模型", "调用模型", "provider"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 provider/model/credential 动作：${forbiddenProposalText}`,
    );
  }
}

function runAdapterSdkCliDiagnosticsBoundaryScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const workerProtocol = workerProtocolFixtureForAdapters(descriptors, operations);
  assert(
    workerProtocol.adapter_contract_checklists.length === descriptors.length,
    "I5 每个 adapter 都应有 contract checklist",
  );
  assert(
    workerProtocol.adapter_contract_checklists.every((checklist) => checklist.control_core_required && checklist.permission_required && checklist.audit_required && checklist.runtime_log_required),
    "I5 checklist 必须要求 control core / permission / audit / runtime log",
  );
  const plannedChecklists = workerProtocol.adapter_contract_checklists.filter((checklist) => checklist.adapter_id !== "codex-local");
  assert(
    plannedChecklists.every((checklist) => checklist.status === "blocked_or_reserved_contract"),
    "planned adapter contract 必须保持阻断或预留",
  );
  assert(
    plannedChecklists.every((checklist) => checklist.missing_items.includes("runtime_connection_not_implemented")),
    "planned adapter contract 必须明确缺 runtime connection",
  );
  assert(
    workerProtocol.controlled_api_cli_semantics.every((semantics) => semantics.universal_api_backdoor_blocked),
    "CLI parity 必须阻断 universal API backdoor",
  );
  assert(
    workerProtocol.diagnostic_event_schemas.every((schema) => schema.redaction_policy === "no_secret_no_raw_transcript_no_provider_payload"),
    "diagnostic schema 必须脱敏，不允许 secret/raw transcript/provider payload",
  );
  assert(
    workerProtocol.adapter_health_summaries
      .filter((summary) => summary.adapter_id !== "codex-local")
      .every((summary) => summary.status === "planned_unavailable"),
    "planned adapter health 必须保持 unavailable",
  );
  assert(
    workerProtocol.adapter_degraded_modes
      .filter((mode) => mode.adapter_id !== "codex-local")
      .every((mode) => mode.blocks_real_execution),
    "planned adapter degraded mode 必须阻断真实执行",
  );
  assert(
    workerProtocol.adapter_data_locations.every((location) => location.secret_policy === "never_read_auth_token_env_keychain_oauth_provider_credentials"),
    "data location descriptor 不能允许读取 secret",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      workerProtocol={workerProtocol}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "适配器 SDK / 命令行 / 诊断预留",
    "只定义未来适配器接入的契约",
    "不提供通用执行接口",
    "不绕过控制核心",
    "运行日志",
    "审计",
    "阻断或预留",
    "阻断通用 API 后门",
    "诊断结构",
    "数据位置",
    "契约材料齐备",
    "阻断或预留",
    "runtime_connection_not_implemented",
    "model_boundary_or_verification_missing",
    "data_location_reserved_not_connected",
    "contract_parity_requires_guard",
    "reserved_no_runtime_parity",
    "required_before_runner",
    "explicit_user_confirmation_required_for_real_execution",
    "runtime_log_and_audit_refs_required",
    "后门阻断：是",
    "adapter_health_read_model_only",
    "adapter_data_location_descriptor_read_model_only",
  ]) {
    assert(agentViewText.includes(expectedText), `I5 Adapter SDK / CLI diagnostics UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "SDK 已接入",
    "CLI 已可执行",
    "通用真实 send/resume 已完成",
    "provider 已验证",
    "凭据已配置",
    "模型已验证",
    "外部 adapter 已接入",
    "自动派发已开始",
    "worker 执行中",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `I5 UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["配置 SDK", "执行 CLI", "验证 provider", "配置凭据", "测试模型", "send", "resume", "dispatch", "重试"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `I5 不应渲染可执行按钮：${forbiddenButtonText}`);
  }
}

function runSessionContinuationPreviewScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(previews.length === 15, "H3.1 后应为 5 个 adapter 的 new_session / send_message / resume 派生预览");
  assert(
    previews.every((preview) => preview.user_visible_warnings.includes("session_continuation_preview_only")),
    "E4 preview 必须声明只预览",
  );
  assert(
    previews.every((preview) => preview.audit_impact.impact_kind === "preview_only_no_execution"),
    "E4 preview 不能写 attempt / dispatch / readback",
  );
  assert(
    previews.every((preview) => !preview.audit_impact.writes_attempt_in_e4 && !preview.audit_impact.writes_dispatch_in_e4 && !preview.audit_impact.writes_readback_in_e4),
    "E4 audit impact 必须保持不写执行态",
  );

  const codexPreviews = previews.filter((preview) => preview.adapter_id === "codex-local");
  assert(codexPreviews.length === 3, "codex-local 应有 new_session / send_message / resume 三条预览");
  assert(
    codexPreviews.every((preview) => preview.guard_result.status === "needs_user_confirmation"),
    "完整绑定的 codex-local 预览应停在需要用户确认",
  );
  const codexNewSessionPreview = codexPreviews.find((preview) => preview.operation_id === "new_session");
  assert(codexNewSessionPreview, "codex-local 应有 new_session 预览");
  assert(codexNewSessionPreview.target_session_id === null, "new_session 预览不应要求已有 target session");
  assert(codexNewSessionPreview.work_item_id === "work-item:offline:001", "new_session 预览必须绑定 work item");
  assert(
    codexNewSessionPreview.request.prompt_source_kind === "h3_new_session_task_package",
    "new_session prompt source 应独立于 send/resume",
  );
  assert(
    codexNewSessionPreview.guard_result.warnings.includes("new_session_does_not_require_existing_session"),
    "new_session guard 应声明不要求已有 session",
  );
  assert(
    codexNewSessionPreview.readback_expectation.expected_sources.includes("future_h3_new_session_last_message"),
    "new_session readback 应指向未来 H3 last-message，而不是现有 session rollout",
  );
  const codexSendResumePreviews = codexPreviews.filter((preview) => preview.operation_id !== "new_session");
  assert(
    codexSendResumePreviews.every((preview) => preview.target_session_id === session.thread_id && preview.project_root === project.project_root),
    "codex-local send/resume preview 应携带 target session 和 project root",
  );
  assert(
    codexPreviews.every((preview) => preview.readback_expectation.strategy === "required"),
    "codex-local continuation preview 必须声明 readback required",
  );

  const plannedPreviews = previews.filter((preview) => preview.adapter_id !== "codex-local");
  assert(
    plannedPreviews.every((preview) => preview.guard_result.status === "blocked" || preview.guard_result.status === "requires_future_task"),
    "planned adapters 必须保持阻断或后续任务状态",
  );
  assert(
    plannedPreviews.every((preview) => preview.guard_result.reasons.some((reason) => reason.includes("planned_adapter_blocked"))),
    "planned adapter continuation preview 必须包含 planned_adapter_blocked 原因",
  );

  const codexOperation = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "send_message");
  assert(codexOperation, "缺少 codex send_message operation");
  const codexNewSessionOperation = operations.find((operation) => operation.adapter_id === "codex-local" && operation.operation_id === "new_session");
  assert(codexNewSessionOperation, "缺少 codex new_session operation");
  const codexAdapter = descriptors.find((descriptor) => descriptor.adapter_id === "codex-local");
  assert(codexAdapter, "缺少 codex adapter");
  const codexProvider = summaries.find((summary) => summary.adapter_id === "codex-local");
  const safeRequest = codexPreviews.find((preview) => preview.operation_id === "send_message")?.request;
  assert(safeRequest, "缺少 send_message safe request");
  const confirmedGuard = inspectSessionContinuationGuard(
    { ...safeRequest, user_confirmation_state: "confirmed" },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(confirmedGuard.status === "allowed_preview", "用户确认后仍只能进入 allowed_preview");
  assert(confirmedGuard.blocks_execution, "allowed_preview 也必须阻断 E4 执行");

  const outOfScopeGuard = inspectSessionContinuationGuard(
    { ...safeRequest, target_cwd: "/offline-fixture/outside", allowed_write_roots: [project.project_root] },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(outOfScopeGuard.status === "blocked", "cwd 越界必须阻断");
  assert(outOfScopeGuard.reasons.includes("cwd_out_of_scope_blocked"), "cwd 越界应有明确 reason");

  const sensitiveGuard = inspectSessionContinuationGuard(
    { ...safeRequest, target_cwd: `${project.project_root}/.env`, allowed_write_roots: [project.project_root] },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(sensitiveGuard.status === "blocked", "敏感路径必须阻断");
  assert(
    sensitiveGuard.reasons.some((reason) => reason.startsWith("sensitive_path_blocked")),
    "敏感路径应有明确 reason",
  );

  const noReadbackGuard = inspectSessionContinuationGuard(
    { ...safeRequest, readback_strategy: "not_defined" },
    codexAdapter,
    codexOperation,
    codexProvider,
  );
  assert(noReadbackGuard.status === "blocked", "缺 readback strategy 必须阻断");
  assert(noReadbackGuard.reasons.includes("readback_strategy_required"), "缺 readback 应有明确 reason");

  const newSessionConfirmedGuard = inspectSessionContinuationGuard(
    { ...codexNewSessionPreview.request, user_confirmation_state: "confirmed" },
    codexAdapter,
    codexNewSessionOperation,
    codexProvider,
  );
  assert(newSessionConfirmedGuard.status === "allowed_preview", "new_session 用户确认后仍只能进入 allowed_preview");
  assert(newSessionConfirmedGuard.blocks_execution, "new_session allowed_preview 仍必须阻断真实执行");

  const newSessionMissingWorkItemGuard = inspectSessionContinuationGuard(
    { ...codexNewSessionPreview.request, work_item_id: null },
    codexAdapter,
    codexNewSessionOperation,
    codexProvider,
  );
  assert(newSessionMissingWorkItemGuard.status === "blocked", "new_session 缺 work item 必须阻断");
  assert(
    newSessionMissingWorkItemGuard.reasons.includes("missing_work_item_binding"),
    "new_session 缺 work item 应有明确 reason",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "会话继续预览 / 权限预览",
    "E4 / H3.1 预览协议",
    "新会话预览",
    "不会创建真实新会话",
    "不会发送提示词",
    "不会执行恢复",
    "不会写 Codex 原生状态",
    "工作项：work-item:offline:001",
    "执行边界摘要：工作目录",
    "运行器：H3.1 空操作",
    "提示词发送状态：否",
    "真实 Codex 执行状态：否",
    "写入 Codex 主目录：否",
    "需要用户确认",
    "读回：必需",
    "审计影响：仅预览不执行",
    "供应方：只读可见",
    "planned_adapter_blocked",
    "h3_1_no_real_new_session",
    "no_prompt_sent_in_e4",
    "no_codex_home_write_in_e4",
  ]) {
    assert(agentViewText.includes(expectedText), `会话继续预览 UI 缺少 ${expectedText}`);
  }
  assert(!agentViewText.includes("命令计划：codex exec -C"), "会话继续普通 UI 不应暴露裸 codex exec 命令");
  for (const forbiddenText of [
    "真实新会话已创建",
    "已创建真实会话",
    "已发送",
    "已 resume",
    "Codex 已收到任务",
    "自动派发已开始",
    "worker 执行中",
    "readback 已完成",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `会话继续预览 UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["新建会话", "发消息", "发送", "resume", "申请确认", "执行", "重试"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `会话继续预览不应渲染可点击按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "session_continuation_boundary"),
    "秘书风险应包含会话继续预览边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_session_continuation_preview"),
    "秘书建议应包含查看会话继续预览",
  );
  for (const forbiddenProposalText of ["新建会话", "发送", "发消息", "resume", "批准", "确认预览", "重试"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 continuation 执行动作：${forbiddenProposalText}`,
    );
  }
}

function runControlledSessionContinuationLevelAScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  const preview = previews.find((item) => item.adapter_id === "codex-local" && item.operation_id === "resume");
  assert(preview, "E5 场景缺少 codex-local resume preview");
  const continuationId = "session-continuation:v1:offline";
  const attemptId = "session-continuation-attempt:offline";
  const store: SessionContinuationStoreV1 = {
    schema_version: "session_continuation_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: "/offline-fixture/workflow-state.v0.json",
      sidecar_path: "/offline-fixture/session-continuations.v1.json",
      project_roots: [project.project_root],
    },
    revision: 2,
    last_write_id: "write-offline-stub",
    generated_by: "control_core",
    created_at: "2026-06-06T00:00:00Z",
    updated_at: "2026-06-06T00:01:00Z",
    continuations: [
      {
        record_version: 1,
        continuation_id: continuationId,
        preview_id: preview.preview_id,
        adapter_id: "codex-local",
        operation_id: "resume",
        project_id: preview.project_id ?? project.project_root,
        project_root: preview.project_root ?? project.project_root,
        workflow_id: preview.workflow_id ?? "workflow:offline-fixture-projects-codex-workbench:default",
        node_id: preview.node_id ?? "node:offline-dev",
        session_id: preview.target_session_id ?? session.thread_id,
        target_cwd: preview.target_cwd ?? project.project_root,
        allowed_write_roots: preview.allowed_write_roots_summary,
        sandbox: preview.sandbox_summary,
        prompt_source_kind: preview.prompt_source_kind,
        prompt_summary: preview.prompt_summary,
        command_preview: "Level A preview only: codex exec resume <session>",
        readback_strategy: "required",
        status: "succeeded_stub",
        execution_level: "level_a_stub_only",
        runner_kind: "stub",
        user_confirmation_state: "confirmed",
        guard_status: "needs_user_confirmation",
        requested_by: "workbench_e4_preview",
        confirmed_by: "user",
        confirmation_reason: "离线 Level A stub 验收",
        created_at: "2026-06-06T00:00:00Z",
        updated_at: "2026-06-06T00:01:00Z",
        audit_refs: ["audit:session-continuation-confirmed:offline", "audit:session-continuation-stub-completed:offline"],
        warnings: ["level_a_stub_only", "real_codex_executed_false", "writes_codex_home_false"],
      },
    ],
    attempts: [
      {
        attempt_version: 1,
        attempt_id: attemptId,
        continuation_id: continuationId,
        runner_kind: "stub",
        execution_level: "level_a_stub_only",
        status: "succeeded_stub",
        started_at: "2026-06-06T00:01:00Z",
        finished_at: "2026-06-06T00:01:00Z",
        timeout_ms: 30000,
        command_preview: "Level A preview only: codex exec resume <session>",
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: {
          status: "readback_unavailable",
          source_kind: "stub_no_transcript_read",
          result_count: null,
          unavailable_reason: "Level A stub 不读取真实 transcript；unavailable 不等于空读回结果。",
          warnings: ["readback_unavailable_is_not_zero_results", "no_real_transcript_read_in_level_a"],
        },
        failure_reason: null,
        audit_refs: ["audit:session-continuation-stub-started:offline", "audit:session-continuation-stub-completed:offline"],
        warnings: [
          "stub_runner_only",
          "prompt_not_sent",
          "real_codex_execution_not_authorized",
          "codex_home_not_touched",
          "readback_unavailable_is_not_zero_results",
        ],
      },
    ],
    audit_events: [
      {
        event_version: 1,
        event_id: "audit:session-continuation-confirmed:offline",
        event_type: "session_continuation_preview_confirmed",
        continuation_id: continuationId,
        attempt_id: null,
        preview_id: preview.preview_id,
        actor_role: "user",
        before_status: null,
        after_status: "preview_confirmed",
        store_revision: 1,
        reason: "用户确认 Level A stub",
        created_at: "2026-06-06T00:00:00Z",
        warnings: ["level_a_stub_only"],
      },
    ],
    warnings: [],
  };

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      sessionContinuationStore={store}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "受控会话继续 / E5 Level A",
    "桩验收",
    "真实执行未授权",
    "读回不可用",
    "readback_unavailable_is_not_zero_results",
    "提示词发送状态：否",
    "真实 Codex 执行状态：否",
    "写入 Codex 主目录：否",
    "session-continuations.v1.json",
  ]) {
    assert(agentViewText.includes(expectedText), `E5 Level A UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已发送",
    "已 resume",
    "Codex 已收到任务",
    "真实 Codex 已执行",
    "worker 执行中",
    "readback 已完成",
    "0 条读回",
    "0 条结果",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `E5 Level A UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["发消息", "发送", "resume", "执行", "重试", "stub 验收"]) {
    assert(!buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText), `E5 Level A 不应渲染可执行按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
      session_continuation_store: store,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "controlled_session_continuation_boundary"),
    "秘书风险应包含 E5 controlled continuation 边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_controlled_session_continuation"),
    "秘书建议应包含查看 E5 controlled continuation",
  );
  for (const forbiddenProposalText of ["发送", "发消息", "resume", "批准", "确认", "重试", "stub"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 E5 continuation 执行动作：${forbiddenProposalText}`,
    );
  }
}

function runH2RealResumeAuthorizationReadinessScenario() {
  const descriptors = deriveAgentAdapterDescriptors({
    sessions: [session, otherProjectSession],
    projects: [project],
    workflowState: workflowStateWithProjectWorkflow,
  });
  const operations = deriveSessionOperationDescriptors(descriptors);
  const summaries = deriveProviderAvailabilitySummaries(descriptors, operations);
  const previews = deriveSessionContinuationPreviews({
    adapterDescriptors: descriptors,
    sessionOperationDescriptors: operations,
    providerAvailabilitySummaries: summaries,
    workflowState: workflowStateWithProjectWorkflow,
  });
  const readiness = deriveH2RealResumeAuthorizationReadiness({
    previews,
    store: snapshot.session_continuation_store,
  });
  const decisionSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews,
    store: snapshot.session_continuation_store,
  });
  assert(readiness.status === "blocked_waiting_authorization", "H2.2 readiness 默认必须等待授权矩阵");
  assert(readiness.missing_count > 0, "H2.2 readiness 必须暴露缺失授权项");
  assert(decisionSurface.status.startsWith("blocked_"), "H2.8 decision surface 默认必须保持阻断态");
  assert(!decisionSurface.final_approval_allowed, "H2.8 decision surface 不得允许 final approval");
  assert(
    decisionSurface.decision_checks.some((check) => check.check_id === "codex_home_scope" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 .codex 最小范围列为阻断",
  );
  assert(
    decisionSurface.decision_checks.some((check) => check.check_id === "rollback" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 rollback 缺失列为阻断",
  );
  assert(
    decisionSurface.readback_boundary.result_count === null &&
      decisionSurface.readback_boundary.warnings.includes("readback_not_attempted_is_not_zero_results"),
    "H2.8 decision surface 必须说明未读回结果数未知",
  );
  assert(
    decisionSurface.permission_preview.denied_paths.some((path) => path.includes("auth/token")),
    "H2.8 permission preview 必须提示 secret / token 禁止展示",
  );
  const missingSessionSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews: previews.map((preview) =>
      preview.adapter_id === "codex-local" && preview.operation_id === "resume"
        ? {
            ...preview,
            target_session_id: null,
            target_session_title: null,
            request: {
              ...preview.request,
              session_id: null,
            },
          }
        : preview,
    ),
    store: snapshot.session_continuation_store,
  });
  assert(
    missingSessionSurface.status === "blocked_waiting_target_session",
    "H2.8 decision surface 缺 target session 时必须明确阻断",
  );
  assert(
    missingSessionSurface.decision_checks.some((check) => check.check_id === "target_session" && check.blocks_final_approval),
    "H2.8 decision surface 必须把 target session 缺失列为阻断",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "prompt_hash_ref" && item.status === "missing"),
    "H2.2 readiness 必须缺少 prompt hash/ref",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "codex_home_scope" && item.status === "missing"),
    "H2.2 readiness 必须缺少 .codex 最小范围",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "user_confirmation" && item.status === "missing"),
    "H2.2 readiness 必须缺少用户确认",
  );
  assert(
    readiness.readiness_items.some((item) => item.item_id === "global_supervisor_confirmation" && item.status === "missing"),
    "H2.2 readiness 必须缺少全局主管确认",
  );

  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={descriptors}
      sessionOperationDescriptors={operations}
      providerAvailabilitySummaries={summaries}
      sessionContinuationPreviews={previews}
      sessionContinuationStore={snapshot.session_continuation_store}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentView);
  for (const expectedText of [
    "H2 真实恢复授权准备",
    "H2.8 最终批准决策面",
    "当前不可批准",
    "权限弹层预览",
    "审计 / 运行日志 / 读回预览",
    "未尝试读回",
    "结果数：未知/不可用",
    "permission_preview_is_not_approval",
    "h2_phase_b_final_approval_not_granted",
    "等待授权矩阵",
    "不会发送提示词",
    "不会执行 codex exec resume",
    "不会读写 /Users/yoyi/.codex",
    "目标会话",
    ".codex 最小范围",
    "提示词引用 / 哈希",
    "回滚：",
    "h2_readiness_is_not_execution_authorization",
  ]) {
    assert(agentViewText.includes(expectedText), `H2.2 readiness UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "Codex 已收到任务",
    "真实 Codex 已执行",
    "prompt 已发送",
    ".codex 已读写",
    "H2 已完成",
    "H3 可开始",
    "readback 0 条",
    "0 条读回",
    "final approval 已批准",
  ]) {
    assert(!agentViewText.includes(forbiddenText), `H2.2 readiness UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentViewMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["执行", "resume", "发送", "确认", "授权", "重试"]) {
    assert(
      !buttonTextsInMarkup(agentViewMarkup).includes(forbiddenButtonText),
      `H2.2 readiness 不应渲染执行或授权按钮：${forbiddenButtonText}`,
    );
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: {
      ...snapshot,
      agent_adapters: descriptors,
      session_operations: operations,
      provider_availability: summaries,
      session_continuation_previews: previews,
    },
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "h2_real_resume_decision_boundary"),
    "秘书风险应包含 H2.8 final approval 决策面边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_h2_real_resume_decision_surface"),
    "秘书建议应包含查看 H2.8 final approval 决策面",
  );
  for (const forbiddenProposalText of ["发送", "发消息", "resume", "批准", "确认", "重试"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 H2.8 执行动作：${forbiddenProposalText}`,
    );
  }

  const confirmedPreview = previews.find((preview) => preview.adapter_id === "codex-local" && preview.operation_id === "resume");
  assert(confirmedPreview, "H2.8 duplicate guard fixture 缺少 codex-local resume preview");
  const duplicateStore: SessionContinuationStoreV1 = {
    ...snapshot.session_continuation_store,
    continuations: [
      {
        record_version: 1,
        continuation_id: "session-continuation:v1:h2-8-duplicate",
        preview_id: confirmedPreview.preview_id,
        adapter_id: "codex-local",
        operation_id: "resume",
        project_id: confirmedPreview.project_id ?? project.project_root,
        project_root: confirmedPreview.project_root ?? project.project_root,
        workflow_id: confirmedPreview.workflow_id ?? "workflow:offline",
        node_id: confirmedPreview.node_id ?? "node:offline",
        session_id: confirmedPreview.target_session_id ?? session.thread_id,
        target_cwd: confirmedPreview.target_cwd ?? project.project_root,
        allowed_write_roots: confirmedPreview.allowed_write_roots_summary,
        sandbox: confirmedPreview.sandbox_summary,
        prompt_source_kind: confirmedPreview.prompt_source_kind,
        prompt_summary: confirmedPreview.prompt_summary,
        command_preview: "Level B preview only: codex exec resume <session>",
        readback_strategy: "required",
        status: "queued",
        execution_level: "level_b_real_user_approved",
        runner_kind: "codex_local_real",
        user_confirmation_state: "confirmed",
        guard_status: "needs_user_confirmation",
        requested_by: "h2_8_duplicate_fixture",
        confirmed_by: "user",
        confirmation_reason: "duplicate guard fixture",
        created_at: "2026-06-08T00:00:00Z",
        updated_at: "2026-06-08T00:01:00Z",
        audit_refs: ["audit:h2-8-duplicate"],
        warnings: ["duplicate_guard_fixture_only"],
      },
    ],
    attempts: [
      {
        attempt_version: 1,
        attempt_id: "session-continuation-attempt:h2-8-duplicate",
        continuation_id: "session-continuation:v1:h2-8-duplicate",
        runner_kind: "codex_local_real",
        execution_level: "level_b_real_user_approved",
        status: "queued",
        started_at: "2026-06-08T00:01:00Z",
        finished_at: null,
        timeout_ms: 120000,
        command_preview: "Level B queued preview: codex exec resume <session>",
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: {
          status: "readback_unavailable",
          source_kind: "queued_no_readback",
          result_count: null,
          unavailable_reason: "Queued attempt has no readback; unavailable is not zero results.",
          warnings: ["readback_unavailable_is_not_zero_results"],
        },
        failure_reason: null,
        audit_refs: ["audit:h2-8-duplicate"],
        warnings: ["duplicate_guard_fixture_only"],
      },
    ],
  };
  const duplicateSurface = deriveH2RealResumeExecutionDecisionSurface({
    previews,
    store: duplicateStore,
  });
  assert(
    duplicateSurface.status === "blocked_by_duplicate_attempt",
    "H2.8 decision surface 必须优先阻断 queued/running duplicate attempt",
  );
  assert(duplicateSurface.duplicate_attempt_blocked, "H2.8 duplicate attempt 必须阻断 final approval");
  assert(
    duplicateSurface.readback_boundary.result_count === null &&
      duplicateSurface.readback_boundary.warnings.includes("readback_unavailable_is_not_zero_results"),
    "H2.8 duplicate/readback unavailable 必须保持结果数未知",
  );
}

function runRuntimeSessionAttentionScenario() {
  const attention = runtimeAttentionFixtures();
  const summaries: SessionRunStatusSummary[] = [
    {
      session_id: session.thread_id,
      adapter_id: "codex-local",
      project_id: "project:offline",
      workflow_id: "workflow:offline",
      node_id: "node:offline",
      current_status: "blocked_by_guard",
      current_status_label: "边界保护阻断",
      attention_count: attention.length,
      blocking_count: 2,
      needs_user_count: 3,
      readback_status: "readback_unavailable",
      latest_attention_ids: attention.slice(0, 4).map((item) => item.attention_id),
      source_refs: attention.flatMap((item) => item.source_refs).slice(0, 4),
      warnings: [],
    },
  ];
  const runtimeSnapshot: WorkbenchSnapshot = {
    ...snapshot,
    runtime_session_attention: attention,
    session_run_status_summaries: summaries,
  };
  const agentView = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={backendAgentAdapterDescriptors}
      sessionOperationDescriptors={backendSessionOperationDescriptors}
      providerAvailabilitySummaries={backendProviderAvailabilitySummaries}
      sessionContinuationPreviews={[]}
      sessionContinuationStore={snapshot.session_continuation_store}
      runtimeSessionAttention={attention}
      sessionRunStatusSummaries={summaries}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentText = visibleText(agentView);
  for (const expectedText of [
    "运行关注 / E6",
    "等待确认",
    "边界保护阻断",
    "读回不可用",
    "读回失败",
    "结果数：未知/不可用",
    "真实读回：否",
  ]) {
    assert(agentText.includes(expectedText), `E6 runtime attention UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已自动重试",
    "已停止 agent",
    "已重启 agent",
    "真实派发已完成",
    "真实 prompt 已发送",
    "Codex 已收到任务",
    "真实 readback 已完成",
    "readback 0 条",
    "失败已自动恢复",
    "Claude Code 已接管",
    "OpenClaw 已运行",
    "OpenCode 已 resume",
  ]) {
    assert(!agentText.includes(forbiddenText), `E6 runtime attention UI 不应出现误导文案：${forbiddenText}`);
  }
  const agentMarkup = renderToStaticMarkup(agentView);
  for (const forbiddenButtonText of ["发送", "resume", "重试", "停止", "重启"]) {
    assert(!buttonTextsInMarkup(agentMarkup).includes(forbiddenButtonText), `E6 不应渲染执行类按钮：${forbiddenButtonText}`);
  }

  const secretaryContext = deriveSecretaryContext({
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
  });
  assert(
    secretaryContext.risk_signals.some((risk) => risk.kind === "runtime_session_attention_boundary"),
    "秘书风险应包含 E6 runtime session attention 边界",
  );
  assert(
    secretaryContext.suggestions.some((suggestion) => suggestion.kind === "inspect_runtime_session_attention"),
    "秘书建议应包含查看 E6 runtime attention",
  );
  for (const forbiddenProposalText of ["发送", "resume", "批准", "确认", "重试", "停止", "重启"]) {
    assert(
      !secretaryContext.action_proposals.some((proposal) => proposal.title.includes(forbiddenProposalText)),
      `秘书 action proposal 不应变成 E6 执行动作：${forbiddenProposalText}`,
    );
  }

  const commonProps = {
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
    notice: "offline notice",
    error: false,
    workflowStateError: null,
    secretaryContext,
    onClose: () => {},
    onNavigate: () => {},
    onReloadWorkflowState: () => {},
  };
  const runningPanelText = visibleText(<RightDetailPanel activePanel="running" {...commonProps} />);
  assert(runningPanelText.includes("运行中摘要"), "运行中入口应使用职责化摘要标题");
  assert(runningPanelText.includes("不停止、恢复、重试或启动真实执行"), "运行中入口应声明只汇总不执行");
  assert(runningPanelText.includes("边界保护阻断"), "运行中入口应显示 E6 session summary");
  assert(runningPanelText.includes("读回不可用"), "运行中入口应显示读回边界");
  const todoPanelText = visibleText(<RightDetailPanel activePanel="todos" {...commonProps} />);
  assert(todoPanelText.includes("待处理事项"), "待办入口应使用职责化摘要标题");
  assert(todoPanelText.includes("不替用户批准、派发或写入状态"), "待办入口不应暗示自动处理");
  assert(todoPanelText.includes("查看 E6"), "待办入口应显示需要用户查看的 runtime attention");
  const notificationPanelText = visibleText(<RightDetailPanel activePanel="notifications" {...commonProps} />);
  assert(notificationPanelText.includes("通知摘要"), "通知入口应使用职责化摘要标题");
  assert(notificationPanelText.includes("读取状态"), "通知入口不应再暴露索引读取状态措辞");
  assert(!notificationPanelText.includes("索引读取状态"), "通知入口不应把普通提示写成索引状态面板");
  assert(notificationPanelText.includes("读回失败"), "通知入口应显示读回失败摘要");
}

function runRuntimeLogBoundaryScenario() {
  const runtimeStore = runtimeLogStoreFixture();
  const runtimeSnapshot: WorkbenchSnapshot = {
    ...snapshot,
    runtime_log_store: runtimeStore,
  };
  const commonProps = {
    snapshot: runtimeSnapshot,
    workflowState: workflowStateWithProjectWorkflow,
    notice: "offline notice",
    error: false,
    workflowStateError: null,
    secretaryContext: deriveSecretaryContext({
      snapshot: runtimeSnapshot,
      workflowState: workflowStateWithProjectWorkflow,
    }),
    onClose: () => {},
    onNavigate: () => {},
    onReloadWorkflowState: () => {},
  };
  const managementPanelText = visibleText(<RightDetailPanel activePanel="audit" {...commonProps} />);

  for (const expectedText of [
    "管理",
    "管理摘要",
    "原始材料仍在开发者区或详情中查看",
    "健康 / 诊断边界",
    "degraded_readonly",
    "工作流事实层",
    "读回不可用不是 0 条结果",
    "不自动修复 store",
    "诊断 bundle",
    "日志 / 审计边界",
    "运行日志与审计事件不能互相替代",
    "应用会话",
    "工作流运行",
    "派发尝试",
    "读回",
    "权限等待",
    "诊断事件",
    "审计引用 1",
  ]) {
    assert(managementPanelText.includes(expectedText), `G1 管理入口运行日志摘要缺少 ${expectedText}`);
  }

  const railManagement = workspaceRailItems.find((item) => item.key === "audit");
  assert(railManagement?.label === "管理", "审计和日志应收进右侧管理入口，不新增散开的日志入口");
  assert(!workspaceRailItems.some((item) => item.label.includes("日志") && item.key !== "audit"), "不应新增右侧日志顶级入口");
  assert(!workspaceRailItems.some((item) => item.label.includes("诊断") && item.key !== "audit"), "不应新增右侧诊断顶级入口");

  const serialized = JSON.stringify(runtimeStore);
  for (const forbiddenText of [
    "sk-test-secret",
    "raw provider credential",
    "完整 transcript",
    "full transcript",
    "OAuth",
    "auth.json",
    ".env",
    "keychain",
    "provider credential",
  ]) {
    assert(!serialized.includes(forbiddenText), `G1 runtime log store 不应包含敏感内容：${forbiddenText}`);
    assert(!managementPanelText.includes(forbiddenText), `G1 管理入口不应显示敏感内容：${forbiddenText}`);
  }

  assert(
    runtimeStore.entries.every((entry) => entry.redaction_status === "redacted_safe_summary"),
    "G1 runtime log entries 必须是脱敏摘要",
  );
  assert(
    runtimeStore.entries.some((entry) => entry.audit_refs.length === 1),
    "G1 runtime log 应只保留 audit_refs 引用",
  );
}

function diagnosticSummaryFixture(): DiagnosticSummary {
  return {
    status: "degraded_readonly",
    generated_at: "2026-06-07T00:00:00Z",
    overall_severity: "degraded",
    healthy_count: 2,
    warning_count: 2,
    degraded_count: 1,
    blocked_count: 1,
    store_integrity: [
      {
        store_id: "workflow_state",
        label: "工作流事实层",
        status: "ok",
        severity: "info",
        path: "/offline-fixture/workflow-state.v0.json",
        schema_version: "workflow_state.v0",
        revision: 3,
        item_count: 8,
        warning_count: 0,
        error: null,
        summary: "工作流事实层可读取。",
        boundary: "只读解析 workflow-state.v0.json；G2 不修改状态枚举或顶层结构。",
      },
      {
        store_id: "runtime_log",
        label: "运行日志 sidecar",
        status: "warning",
        severity: "warning",
        path: "/offline-fixture/runtime-logs.v1.json",
        schema_version: "runtime_log_store.v1",
        revision: 1,
        item_count: 6,
        warning_count: 1,
        error: null,
        summary: "运行日志 sidecar 有 1 条 warning，G2 只解释不修复。",
        boundary: "运行日志只记录脱敏运行摘要；不能替代审计事件。",
      },
    ],
    degraded_states: [
      {
        state_id: "diagnostic:runtime_attention",
        kind: "runtime_attention",
        severity: "warning",
        title: "运行关注存在阻断或读回边界",
        summary: "1 条运行关注需要解释；读回不可用不是 0 条结果。",
        user_action_required: true,
        blocks_real_execution: true,
        source_refs: ["workbench_snapshot.runtime_session_attention"],
        recommended_next_step: "查看运行中入口和管理入口摘要；G2 不自动恢复、重试或修复。",
      },
      {
        state_id: "diagnostic:bundle_reference",
        kind: "diagnostic_bundle_reference",
        severity: "info",
        title: "诊断 bundle 为只读引用",
        summary: "G2 在 WorkbenchSnapshot.diagnostic_summary 中提供可引用诊断 bundle；不导出 secret、不生成新文件。",
        user_action_required: false,
        blocks_real_execution: false,
        source_refs: ["workbench_snapshot.diagnostic_summary"],
        recommended_next_step: "如需落盘导出 bundle，必须另拆任务并定义脱敏规则。",
      },
    ],
    recent_error_summaries: ["readback · readback failed safe summary"],
    boundary_notes: [
      "G2 是只读诊断，不自动修复 store、不自动重试、不调用 provider。",
      "读回不可用表示无法读回，不能显示成 0 条结果。",
      "真实 Tauri 截图验收仍属于 G3，不由 G2 冒领。",
    ],
  };
}

function runtimeLogStoreFixture(): RuntimeLogStoreV1 {
  const categories = [
    "app_session",
    "workflow_run",
    "dispatch_attempt",
    "readback",
    "permission_wait",
    "diagnostic_event",
  ];
  const entries = categories.map((category, index) => ({
    entry_version: 1,
    entry_id: `runtime-log:${category}:offline`,
    category,
    status: index === 3 ? "readback_unavailable" : "observed",
    severity: index === 3 || index === 4 ? "warning" : "info",
    started_at: "2026-06-07T00:00:00Z",
    finished_at: index === 4 ? null : "2026-06-07T00:00:01Z",
    duration_ms: index === 2 ? 1000 : null,
    project_id: "project:offline",
    workflow_id: "workflow:offline",
    node_id: "node:offline",
    session_id: session.thread_id,
    adapter_id: "codex-local",
    summary: `${category} redacted runtime summary`,
    detail: "只展示脱敏运行摘要；不展示正文、凭据或原始会话记录。",
    source_refs: [
      {
        source_kind: category === "dispatch_attempt" ? "session_continuation_attempt" : "workbench_runtime",
        source_id: `source:${category}:offline`,
        label: category,
      },
    ],
    audit_refs: category === "dispatch_attempt" ? ["audit:runtime-log:offline"] : [],
    redaction_status: "redacted_safe_summary",
    sensitive_omissions: ["conversation_body", "credential_material"],
    user_visible: true,
    warnings: category === "readback" ? ["readback_unavailable_is_not_zero_results"] : [],
  }));

  return {
    schema_version: "runtime_log_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: "/offline-fixture/workflow-state.v0.json",
      sidecar_path: "/offline-fixture/runtime-logs.v1.json",
      project_roots: [project.project_root],
    },
    revision: 1,
    last_write_id: null,
    generated_by: "offline-test",
    created_at: "2026-06-07T00:00:00Z",
    updated_at: "2026-06-07T00:00:01Z",
    boundary: {
      runtime_log_definition: "运行日志记录运行状态、耗时、分类、来源引用和脱敏摘要。",
      audit_event_definition: "审计事件记录可追责的操作者、权限、状态变化和原因。",
      separation_rule: "运行日志与审计事件不能互相替代；日志只引用审计引用，不内嵌审计事件本体。",
      redaction_rule: "日志展示必须脱敏；授权材料、环境材料、会话正文和供应方原始材料只记录为省略类别。",
      forbidden_payloads: ["credential_material", "conversation_body", "raw_provider_material"],
    },
    entries,
    summaries: categories.map((category) => ({
      category,
      status: category === "readback" ? "readback_unavailable" : "observed",
      severity: category === "readback" || category === "permission_wait" ? "warning" : "info",
      entry_count: 1,
      latest_entry_ids: [`runtime-log:${category}:offline`],
      warnings: [],
    })),
    warnings: ["runtime_log_does_not_replace_audit_event", "audit_event_does_not_replace_runtime_log"],
  };
}

function runtimeAttentionFixtures(): RuntimeSessionAttention[] {
  return [
    runtimeAttentionFixture("waiting-permission", "waiting_permission", "needs_user", "readback_unavailable", "level_b_not_authorized", true, false),
    runtimeAttentionFixture("guard-blocked", "blocked_by_guard", "blocking", "readback_unavailable", "guard_blocked", false, true),
    runtimeAttentionFixture("readback-unavailable", "readback_unavailable", "warning", "readback_unavailable", "not_attempted_stub", true, false),
    runtimeAttentionFixture("readback-failed", "readback_failed", "needs_user", "readback_failed", "readback_parser_failed", true, true),
  ];
}

function runtimeAttentionFixture(
  id: string,
  status: RuntimeSessionAttention["status"],
  severity: RuntimeSessionAttention["severity"],
  readbackStatus: RuntimeSessionAttention["readback_boundary"]["status"],
  reason: RuntimeSessionAttention["readback_boundary"]["reason"],
  requiresUserAction: boolean,
  blocksContinuation: boolean,
): RuntimeSessionAttention {
  return {
    attention_id: `runtime-attention:${id}`,
    project_id: "project:offline",
    workflow_id: "workflow:offline",
    node_id: "node:offline",
    session_id: session.thread_id,
    adapter_id: "codex-local",
    source_refs: [
      {
        source_kind: "session_continuation_attempt",
        source_id: `source:${id}`,
        label: id,
      },
    ],
    kind: status,
    severity,
    status,
    title: `查看 E6 ${status}`,
    user_message:
      readbackStatus === "readback_failed"
        ? "读回失败表示读回失败或不可信，不能显示成空读回。"
        : "读回不可用表示没有真实读回来源，不能显示成空读回。",
    technical_summary: `status=${status} readback=${readbackStatus}`,
    recommended_next_step: "查看运行关注边界；不要自动重试、停止、恢复或批准权限。",
    requires_user_action: requiresUserAction,
    blocks_continuation: blocksContinuation,
    readback_boundary: {
      status: readbackStatus,
      reason,
      attempted: false,
      real_readback_performed: false,
      result_count: null,
      user_message: "unavailable / failed 都不是空读回结果。",
      technical_summary: `reason=${reason}`,
      source_refs: [
        {
          source_kind: "session_continuation_attempt",
          source_id: `source:${id}`,
          label: id,
        },
      ],
      warnings: [],
    },
    created_at: "2026-06-06T00:00:00Z",
    updated_at: "2026-06-06T00:00:00Z",
    warnings: [],
  };
}

function runCandidateGovernanceScenario() {
  const blackboardOverlay = buildBlackboardCandidateOverlay({
    store: {
      schema_version: "blackboard_candidate_persistence.v1",
      store_version: 1,
      storage_kind: "sidecar_json_v0",
      revision: 3,
      records: [
        {
          candidate_key: "bbcand:v1:offline-report",
          project_id: project.project_root,
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          source_entry_id: "blackboard:offline:report:001",
          entry_kind: "subagent_report",
          target_kind: "workflow_fact",
          state: "candidate_confirmed_for_followup",
          title_snapshot: "离线子汇报",
          summary_snapshot: "只确认后续处理，不写正式事实。",
          source_refs: [{ source_kind: "subagent_report", source_id: "report:offline:001", label: "子智能体汇报" }],
          updated_at: "2026-06-03T00:00:00Z",
          warnings: [],
        },
      ],
      audit_events: [],
      updated_at: "2026-06-03T00:00:00Z",
      warnings: [],
    },
    entries: workflowStateWithDerivedWorkflow.project_blackboards?.[0].entries ?? [],
  });
  assert(blackboardOverlay.status_by_entry_id["blackboard:offline:report:001"] === "candidate_confirmed_for_followup", "黑板候选 overlay 应能按 source_entry_id 映射确认状态");
  assert(blackboardOverlay.sidecar_name === "blackboard-candidates.v1.json", "黑板候选 sidecar 文件名不匹配");
  assert(!blackboardOverlay.warnings.includes("writes_formal_memory"), "黑板 overlay 不应写正式记忆 warning");

  const confirmedMemoryCandidate: MemoryCandidate = {
    candidate_id: "memcand:offline:001",
    candidate_key: "memcand:v1:offline-preference",
    schema_version: "memory_governance.v1",
    scope: {
      scope_id: "scope:user:yoyi",
      scope_type: "user_preference",
      role_ids: [],
      document_refs: [],
      model_export_policy: "local_only",
      valid_from: "2026-06-03T00:00:00Z",
    },
    memory_type: "user_preference",
    claim: "用户要求先指出风险。",
    body: "候选已确认保留，但不是正式长期记忆。",
    source_refs: [
      {
        source_ref_id: "source:user-confirmed:001",
        source_type: "user_confirmed_proposal",
        source_id: "task:offline",
        source_title: "离线确认",
        captured_at: "2026-06-03T00:00:00Z",
        authority_level: "user_confirmed",
        sensitive_level: "private",
      },
    ],
    generated_by_role: "user",
    generated_from: "explicit_user_confirmation",
    status: "candidate_confirmed",
    risk_level: "low",
    sensitive_level: "private",
    requires_user_confirmation: true,
    review_reason: "离线候选治理测试",
    conflicts: [],
    audit_refs: [],
    adoption: null,
    created_at: "2026-06-03T00:00:00Z",
    updated_at: "2026-06-03T00:00:00Z",
  };

  const memorySummary = summarizeMemoryCandidateStore({
    store_version: "memory_candidate_store.v1",
    revision: 2,
    candidates: [confirmedMemoryCandidate],
    events: [],
    updated_at: "2026-06-03T00:00:00Z",
  });
  assert(memorySummary.sidecar_name === "memory-candidates.v1.json", "记忆候选 sidecar 文件名不匹配");
  assert(memorySummary.confirmed_count === 1, "记忆候选确认保留计数不匹配");
  assert(memorySummary.formal_memory_count === 0, "候选确认不应生成正式记忆");
  assert(memorySummary.adopted_count === 0, "普通 candidate_confirmed 不应显示为已采纳");
  assert(!memorySummary.display_text.includes("已记住"), "记忆候选 UI 文案不能说已记住");
  assert(!memorySummary.display_text.includes("正式记忆已写入"), "记忆候选 UI 文案不能说正式记忆已写入");

  const adoptedMemorySummary = summarizeMemoryCandidateStore({
    store_version: "memory_candidate_store.v1",
    revision: 3,
    candidates: [
      {
        ...confirmedMemoryCandidate,
        candidate_id: "memcand:offline:002",
        candidate_key: "memcand:v1:offline-project",
        adoption: {
          adopted_memory_id: "mem:formal:offline:002",
          adopted_version_id: "memver:formal:offline:002",
          adopted_audit_event_id: "audit:formal:offline:002",
          adopted_at: "2026-06-03T00:00:02Z",
          adopted_by_role: "project_director",
          adoption_reason: "项目主管采纳低风险本项目记忆候选。",
        },
      },
    ],
    events: [],
    updated_at: "2026-06-03T00:00:02Z",
  });
  assert(adoptedMemorySummary.adopted_count === 1, "已采纳候选计数不匹配");
  assert(adoptedMemorySummary.formal_memory_count === 0, "候选 sidecar 不应把采纳候选改成正式状态");
  assert(adoptedMemorySummary.first_adoption?.adopted_memory_id === "mem:formal:offline:002", "采纳摘要缺少 adopted_memory_id");
  assert(adoptedMemorySummary.first_adoption?.adopted_version_id === "memver:formal:offline:002", "采纳摘要缺少 adopted_version_id");
  assert(adoptedMemorySummary.first_adoption?.adopted_audit_event_id === "audit:formal:offline:002", "采纳摘要缺少 adopted_audit_event_id");

  const observationStore = {
    store_version: "observation_store.v1" as const,
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 2,
    observations: [
      {
        observation_id: "obs:v1:offline:001",
        observation_key: "obs:v1:offline-recorded",
        schema_version: "memory_observation.v1" as const,
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project" as const,
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only" as const,
          valid_from: "2026-06-04T00:00:00Z",
        },
        observation_type: "worker_report" as const,
        summary: "开发线汇报：观察入口已经写入 sidecar。",
        source_refs: [
          {
            source_ref_id: "obs-source:offline:001",
            source_kind: "worker_report" as const,
            source_id: "worker-report:offline:001",
            project_id: "project:offline-fixture-projects-codex-workbench",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            summary: "工作者汇报摘要，不复制完整会话记录。",
            sensitive_level: "internal" as const,
            created_at: "2026-06-04T00:00:00Z",
          },
        ],
        status: "recorded" as const,
        generated_by_role: "worker",
        actor_id: "codex-dev",
        risk_level: "low" as const,
        sensitive_level: "internal" as const,
        candidate_key: null,
        audit_refs: [],
        created_at: "2026-06-04T00:00:00Z",
        updated_at: "2026-06-04T00:00:00Z",
      },
      {
        observation_id: "obs:v1:offline:002",
        observation_key: "obs:v1:offline-candidate-created",
        schema_version: "memory_observation.v1" as const,
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project" as const,
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only" as const,
          valid_from: "2026-06-04T00:00:00Z",
        },
        observation_type: "project_director_confirmation" as const,
        summary: "项目主管确认 observation 可生成候选。",
        source_refs: [
          {
            source_ref_id: "obs-source:offline:002",
            source_kind: "director_review" as const,
            source_id: "director-review:offline:002",
            project_id: "project:offline-fixture-projects-codex-workbench",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            summary: "项目主管确认摘要。",
            sensitive_level: "internal" as const,
            created_at: "2026-06-04T00:00:02Z",
          },
        ],
        status: "candidate_created" as const,
        generated_by_role: "project_director",
        actor_id: "project-director-offline",
        risk_level: "low" as const,
        sensitive_level: "internal" as const,
        candidate_key: "memcand:v1:from-observation",
        audit_refs: [],
        created_at: "2026-06-04T00:00:02Z",
        updated_at: "2026-06-04T00:00:03Z",
      },
    ],
    events: [
      {
        audit_ref_id: "audit:observation-candidate-created:offline",
        event_type: "observation_candidate_created" as const,
        actor_id: "project-director-offline",
        actor_role: "project_director",
        target_kind: "observation" as const,
        target_id: "obs:v1:offline:002",
        before_status: "recorded" as const,
        after_status: "candidate_created" as const,
        reason: "项目主管确认生成候选。",
        created_at: "2026-06-04T00:00:03Z",
      },
    ],
    updated_at: "2026-06-04T00:00:03Z",
    warnings: [],
  };
  const observationSummary = summarizeObservationStore(observationStore);
  assert(observationSummary.sidecar_name === "observations.v1.json", "observation sidecar 文件名不匹配");
  assert(observationSummary.recorded_count === 1, "recorded observation 计数不匹配");
  assert(observationSummary.candidate_created_count === 1, "candidate_created observation 计数不匹配");
  assert(observationSummary.recent_candidate_key === "memcand:v1:from-observation", "observation 摘要应显示最近 candidate_key");
  assert(observationSummary.display_text.includes("observation 不是正式记忆"), "observation 摘要必须说明不是正式记忆");

  capturedAction = null;
  const observationWorkflowProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      observationStore={observationStore}
      memoryCandidateStore={{
        store_version: "memory_candidate_store.v1",
        revision: 0,
        candidates: [],
        events: [],
        updated_at: "2026-06-04T00:00:00Z",
      }}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const observationWorkflowText = visibleText(observationWorkflowProject);
  for (const expectedText of [
    "工作流观察",
    "observations.v1.json",
    "recorded 1",
    "candidate_created 1",
    "observation_candidate_created",
    "memcand:v1:from-observation",
    "观察可生成候选",
    "从工作流观察生成候选",
    "候选仍需确认 / 采纳",
    "observation 不是正式记忆",
  ]) {
    assert(observationWorkflowText.includes(expectedText), `工作流观察 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["系统已记住", "自动学习完成", "observation 已成为正式记忆", "已注入任务包"]) {
    assert(!observationWorkflowText.includes(forbiddenText), `工作流观察 UI 不应出现越界文案：${forbiddenText}`);
  }

  const formalMemorySummary = summarizeFormalMemoryStore({
    store_version: "formal_memory_store.v1",
    project_id: "project:offline",
    workflow_id: "workflow:offline:default",
    revision: 4,
    records: [
      {
        memory_id: "mem:formal:offline:001",
        schema_version: "memory_governance.v1",
        record_version: 1,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline",
          workflow_id: "workflow:offline:default",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-03T00:00:00Z",
        },
        memory_type: "project_memory",
        claim: "正式记忆创建必须写 version 和 audit。",
        body: "M1 只验证正式记忆骨架，不做任务包注入。",
        source_refs: [
          {
            source_ref_id: "source:offline:formal:001",
            source_type: "stage_report",
            source_id: "stage:offline",
            source_title: "离线正式记忆测试",
            captured_at: "2026-06-03T00:00:00Z",
            authority_level: "evidence",
            sensitive_level: "project",
          },
        ],
        status: "memory_active",
        supersedes_memory_id: null,
        superseded_by_memory_id: null,
        conflict_refs: [],
        audit_refs: [],
        created_at: "2026-06-03T00:00:00Z",
        updated_at: "2026-06-03T00:00:00Z",
      },
    ],
    versions: [
      {
        version_id: "memver:formal:offline:001",
        memory_id: "mem:formal:offline:001",
        version_number: 1,
        change_type: "created",
        change_summary: "创建正式记忆第一版。",
        record_snapshot: {
          memory_id: "mem:formal:offline:001",
          schema_version: "memory_governance.v1",
          record_version: 1,
          scope: {
            scope_id: "scope:project:offline",
            scope_type: "project",
            project_id: "project:offline",
            workflow_id: "workflow:offline:default",
            role_ids: [],
            document_refs: [],
            model_export_policy: "local_only",
            valid_from: "2026-06-03T00:00:00Z",
          },
          memory_type: "project_memory",
          claim: "正式记忆创建必须写 version 和 audit。",
          body: "M1 只验证正式记忆骨架，不做任务包注入。",
          source_refs: [],
          status: "memory_active",
          supersedes_memory_id: null,
          superseded_by_memory_id: null,
          conflict_refs: [],
          audit_refs: [],
          created_at: "2026-06-03T00:00:00Z",
          updated_at: "2026-06-03T00:00:00Z",
        },
        source_refs: [],
        changed_by_role: "project_director",
        reviewed_by: null,
        created_at: "2026-06-03T00:00:00Z",
      },
    ],
    audit_events: [
      {
        audit_event_id: "audit:formal:offline:001",
        event_type: "memory_record_created",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        project_id: "project:offline",
        workflow_id: "workflow:offline:default",
        session_id: null,
        target_kind: "memory_record",
        target_id: "mem:formal:offline:001",
        before_state: null,
        after_state: "memory_active",
        reason: "离线正式记忆测试",
        source_refs: [],
        status: "succeeded",
        created_at: "2026-06-03T00:00:00Z",
      },
    ],
    updated_at: "2026-06-03T00:00:00Z",
    warnings: [],
  });
  assert(formalMemorySummary.sidecar_name === "formal-memories.v1.json", "正式记忆 sidecar 文件名不匹配");
  assert(formalMemorySummary.record_count === 1, "正式记忆 record 计数不匹配");
  assert(formalMemorySummary.version_count === 1, "正式记忆 version 计数不匹配");
  assert(formalMemorySummary.audit_event_count === 1, "正式记忆 audit 计数不匹配");
  assert(formalMemorySummary.active_count === 1, "正式记忆 active 计数不匹配");
  assert(formalMemorySummary.recent_audit_event?.event_type === "memory_record_created", "正式记忆最近审计事件不匹配");
  assert(formalMemorySummary.display_text.includes("创建时写入 version 和 audit"), "正式记忆摘要应说明 version/audit 骨架");
  for (const forbiddenText of ["AI 自动记住", "候选已记住", "秘书已批准", "worker 已写入正式记忆", "完整记忆层完成", "系统已学习", "任务包注入已完成", "正式记忆完整完成"]) {
    assert(!formalMemorySummary.display_text.includes(forbiddenText), `正式记忆摘要不应出现越界文案：${forbiddenText}`);
  }

  const adoptedFormalMemorySummary = summarizeFormalMemoryStore({
    store_version: "formal_memory_store.v1",
    project_id: "project:offline",
    workflow_id: "workflow:offline:default",
    revision: 5,
    records: [],
    versions: [],
    audit_events: [
      {
        audit_event_id: "audit:formal:offline:002",
        event_type: "memory_candidate_adopted_to_formal_memory",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        project_id: "project:offline",
        workflow_id: "workflow:offline:default",
        session_id: null,
        target_kind: "memory_record",
        target_id: "mem:formal:offline:002",
        before_state: null,
        after_state: "memory_active",
        reason: "项目主管采纳低风险本项目记忆候选。",
        source_refs: [],
        status: "succeeded",
        created_at: "2026-06-03T00:00:02Z",
      },
    ],
    updated_at: "2026-06-03T00:00:02Z",
    warnings: [],
  });
  assert(adoptedFormalMemorySummary.recent_audit_event?.event_type === "memory_candidate_adopted_to_formal_memory", "正式记忆摘要应识别候选采纳审计");
  assert(adoptedFormalMemorySummary.display_text.includes("候选受控采纳审计已记录"), "正式记忆摘要应显示候选受控采纳审计");
  assert(!adoptedFormalMemorySummary.display_text.includes("M1 不包含候选采纳"), "采纳事件出现后不应继续显示 M1 候选采纳缺口文案");

  const memoryLintStore: MemoryLintStoreV1 = {
    store_version: "memory_lint_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 2,
    findings: [
      {
        finding_id: "memlint:v1:offline:blocking",
        schema_version: "memory_governance.v1",
        finding_type: "source_permission_revoked",
        severity: "blocking",
        status: "open",
        source_kind: "memory_record",
        source_id: "mem:formal:offline:revoked",
        target_memory_id: "mem:formal:offline:revoked",
        target_candidate_key: null,
        scope_type: "project",
        memory_type: "project_memory",
        claim: "撤回来源不能进入任务记忆包。",
        summary: "来源权限撤回产生 open blocking finding。",
        recommended_action: "exclude_from_task_packet",
        evidence_refs: [],
        audit_event_id: null,
        created_at: "2026-06-04T02:00:00Z",
        updated_at: "2026-06-04T02:00:00Z",
      },
      {
        finding_id: "memlint:v1:offline:review",
        schema_version: "memory_governance.v1",
        finding_type: "duplicate_claim",
        severity: "needs_review",
        status: "open",
        source_kind: "memory_record",
        source_id: "mem:formal:offline:duplicate",
        target_memory_id: "mem:formal:offline:duplicate",
        target_candidate_key: null,
        scope_type: "project",
        memory_type: "project_memory",
        claim: "重复 claim 需要人工复核。",
        summary: "重复 claim 只生成 needs_review finding。",
        recommended_action: "review_and_deprecate",
        evidence_refs: [],
        audit_event_id: null,
        created_at: "2026-06-04T02:00:01Z",
        updated_at: "2026-06-04T02:00:01Z",
      },
      {
        finding_id: "memlint:v1:offline:resolved",
        schema_version: "memory_governance.v1",
        finding_type: "candidate_conflicts_with_active_memory",
        severity: "blocking",
        status: "resolved",
        source_kind: "memory_candidate",
        source_id: "memcand:v1:offline-resolved",
        target_memory_id: "mem:formal:offline:old",
        target_candidate_key: "memcand:v1:offline-resolved",
        scope_type: "project",
        memory_type: "project_memory",
        claim: "已解决 finding 不应计入 open blocking。",
        summary: "resolved blocking finding 不计入 open blocking 摘要。",
        recommended_action: "block_adoption",
        evidence_refs: [],
        audit_event_id: null,
        created_at: "2026-06-04T02:00:02Z",
        updated_at: "2026-06-04T02:00:02Z",
      },
    ],
    runs: [
      {
        run_id: "memlint-run:v1:offline",
        lint_intent: "candidate_adoption_guard",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        finding_ids: ["memlint:v1:offline:blocking"],
        blocking_count: 1,
        status: "blocked",
        reason: "candidate_adoption_guard blocked by 1 blocking finding",
        created_at: "2026-06-04T02:00:03Z",
      },
    ],
    updated_at: "2026-06-04T02:00:03Z",
    warnings: ["memory_lint_findings_only_no_formal_memory_mutation"],
  };
  const memoryLintSummary = summarizeMemoryLintStore(memoryLintStore);
  assert(memoryLintSummary.sidecar_name === "memory-lint.v1.json", "记忆 lint sidecar 文件名不匹配");
  assert(memoryLintSummary.finding_count === 3, "记忆 lint finding 计数不匹配");
  assert(memoryLintSummary.open_count === 2, "记忆 lint open 计数不匹配");
  assert(memoryLintSummary.blocking_count === 1, "记忆 lint blocking 计数不匹配");
  assert(memoryLintSummary.needs_review_count === 1, "记忆 lint needs_review 计数不匹配");
  assert(memoryLintSummary.recent_run?.status === "blocked", "记忆 lint 最近 run 状态不匹配");
  for (const expectedText of [
    "记忆 lint 阻断摘要",
    "blocking finding 会阻止进入任务包",
    "lint 只生成待处理 finding",
    "不会自动修改正式记忆",
  ]) {
    assert(memoryLintSummary.display_text.includes(expectedText), `记忆 lint 摘要缺少 ${expectedText}`);
  }

  const taskMemoryPacketPreview: TaskMemoryPacketBuildOutput = {
    preview: {
      packet_id: "task-memory-packet-preview:v1:offline",
      schema_version: "task_memory_packet.v1",
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      task_id: "work-item:offline:001",
      role_id: "codex-dev",
      retrieval_intent: "worker_task",
      included_memories: [
        {
          memory_id: "mem:formal:offline:included",
          memory_type: "project_memory",
          scope_type: "project",
          claim: "接口验收必须保留控制核心边界。",
    body: "活跃正式记忆可以进入任务记忆包预览。",
          source_refs: [],
          retrieval_reason: "active formal memory matched task goal by 接口；scope=project",
          estimated_tokens: 28,
          model_export_policy: "local_only",
        },
      ],
      excluded_items: [
        {
          source_kind: "memory_candidate",
          source_id: "memcand:v1:offline-packet",
          claim: "候选还没有受控采纳。",
          reason: "candidate_unconfirmed",
          detail: "记忆候选不是正式记忆；不会进入任务记忆包入选列表",
        },
        {
          source_kind: "observation",
          source_id: "obs:v1:offline-packet",
          claim: "观察只作为待审查材料。",
          reason: "observation_not_formal_memory",
          detail: "观察不是正式记忆；不会进入任务记忆包入选列表",
        },
      ],
      review_materials: [
        {
          source_kind: "memory_candidate",
          source_id: "memcand:v1:offline-packet",
          title: "候选还没有受控采纳。",
          reason: "candidate_unconfirmed",
        },
        {
          source_kind: "observation",
          source_id: "obs:v1:offline-packet",
          title: "观察只作为待审查材料。",
          reason: "observation_not_formal_memory",
        },
      ],
      estimated_tokens: 28,
      max_estimated_tokens: 8000,
      generated_at: "2026-06-04T01:00:00Z",
      warnings: [
        "preview_only_not_injected",
        "worker_has_not_received_memory_packet",
        "candidate_and_observation_review_materials_only",
      ],
    },
    formal_store_revision: 4,
    candidate_store_revision: 3,
    observation_store_revision: 2,
    lint_store_revision: 5,
    warnings: ["preview_only_not_injected"],
  };
  const taskMemoryPacketSummary = summarizeTaskMemoryPacketPreview(taskMemoryPacketPreview);
  assert(taskMemoryPacketSummary.included_count === 1, "任务记忆包预览 included 计数不匹配");
  assert(taskMemoryPacketSummary.excluded_count === 2, "任务记忆包预览 excluded 计数不匹配");
  assert(taskMemoryPacketSummary.review_material_count === 2, "任务记忆包预览待审查材料计数不匹配");
  assert(taskMemoryPacketSummary.reason_counts.candidate_unconfirmed === 1, "任务记忆包预览缺少 candidate_unconfirmed 计数");
  assert(taskMemoryPacketSummary.reason_counts.observation_not_formal_memory === 1, "任务记忆包预览缺少 observation_not_formal_memory 计数");
  assert(taskMemoryPacketSummary.display_text.includes("预览未注入任务包"), "任务记忆包预览摘要必须说明未注入");
  assert(taskMemoryPacketSummary.reason_text.includes("candidate_unconfirmed"), "任务记忆包预览 reason 摘要缺少 candidate_unconfirmed");
  assert(taskMemoryPacketSummary.reason_text.includes("observation_not_formal_memory"), "任务记忆包预览 reason 摘要缺少 observation_not_formal_memory");
  const taskPackageMemoryInjectionSummary = summarizeTaskPackageMemoryInjection(
    workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow?.task_packages[0].memory_injection_summary,
  );
  assert(taskPackageMemoryInjectionSummary.included_count === 1, "任务包记忆注入 included 计数不匹配");
  assert(taskPackageMemoryInjectionSummary.excluded_count === 2, "任务包记忆注入 excluded 计数不匹配");
  assert(taskPackageMemoryInjectionSummary.review_material_count === 2, "任务包记忆注入 review materials 计数不匹配");
  assert(!taskPackageMemoryInjectionSummary.stale, "任务包记忆注入 fixture 应为 fresh");
  for (const expectedText of [
    "任务包记忆注入摘要",
    "仅活跃正式记忆可进入任务包",
    "候选 / 观察仅作为待审查材料",
    "任务包内容不会回灌成正式记忆",
  ]) {
    assert(taskPackageMemoryInjectionSummary.display_text.includes(expectedText), `任务包记忆注入摘要缺少 ${expectedText}`);
  }

  const taskMemoryPacketWorkflowText = visibleText(
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      selectedTool="workflow"
      taskMemoryPacketPreview={taskMemoryPacketPreview}
      memoryLintStore={memoryLintStore}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "任务记忆包预览",
    "入选 1",
    "排除 2",
    "待审查材料 2",
    "估算 token 28/8000",
    "candidate_unconfirmed",
    "observation_not_formal_memory",
    "仅启用态正式记忆可入选",
    "候选 / 观察仅作为待审查材料",
    "不进入正式记忆列表",
    "preview_only_not_injected",
    "记忆 lint sidecar",
    "memory-lint.v1.json",
    "记忆 lint 阻断摘要",
    "任务包记忆注入摘要",
    "task-package-memory-packet-snapshot:v1:offline:001",
    "入选正式记忆",
    "快照状态",
    "新鲜",
    "仅启用态正式记忆可进入任务包",
    "任务包内容不会回灌成正式记忆",
    "open 2",
    "blocking 1",
    "needs_review 1",
    "最近检查运行",
    "来源权限撤回",
    "blocking finding 会阻止进入任务包",
    "lint 只生成待处理 finding",
    "不会自动修改正式记忆",
  ]) {
    assert(taskMemoryPacketWorkflowText.includes(expectedText), `任务记忆包预览 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "系统已记住",
    "自动学习完成",
    "候选已进入任务包",
    "observation 已注入任务包",
    "worker 已收到记忆包",
    "真实 worker 已执行",
    "系统已自动记住",
    "任务包内容已写入正式记忆",
    "任务包注入已完成",
    "中间版本记忆层完成",
    "AI 已自动解决冲突",
    "系统已废弃旧记忆",
    "旧记忆已自动更新",
    "正式记忆生命周期完成",
  ]) {
    assert(!taskMemoryPacketWorkflowText.includes(forbiddenText), `任务记忆包预览 UI 不应出现越界文案：${forbiddenText}`);
  }
}

function runMemoryManagementCenterScenario() {
  const formalMemoryStore: FormalMemoryStoreV1 = {
    store_version: "formal_memory_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 7,
    records: [
      {
        memory_id: "mem:formal:offline:included",
        schema_version: "memory_governance.v1",
        record_version: 1,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T00:00:00Z",
        },
        memory_type: "project_memory",
        claim: "接口验收必须保留控制核心边界。",
        body: "active 正式记忆可以进入任务记忆包预览。",
        source_refs: [
          {
            source_ref_id: "source:memory-center:formal:001",
            source_type: "evidence",
            source_id: "evidence:m7",
            source_title: "M7 离线证据",
            captured_at: "2026-06-05T00:00:00Z",
            authority_level: "evidence",
            sensitive_level: "project",
          },
        ],
        status: "memory_active",
        supersedes_memory_id: null,
        superseded_by_memory_id: null,
        conflict_refs: [],
        audit_refs: [],
        created_at: "2026-06-05T00:00:00Z",
        updated_at: "2026-06-05T00:00:00Z",
      },
      {
        memory_id: "mem:formal:offline:blocked",
        schema_version: "memory_governance.v1",
        record_version: 1,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          role_ids: ["codex-dev"],
          document_refs: [],
          model_export_policy: "blocked",
          valid_from: "2026-06-05T00:00:00Z",
        },
        memory_type: "project_memory",
        claim: "撤回来源不能进入任务记忆包。",
        body: "open blocking finding 必须阻断任务包入选。",
        source_refs: [
          {
            source_ref_id: "source:memory-center:formal:002",
            source_type: "director_review",
            source_id: "review:m7",
            source_title: "项目主管复核",
            captured_at: "2026-06-05T00:00:01Z",
            authority_level: "audit",
            sensitive_level: "private",
          },
        ],
        status: "memory_active",
        supersedes_memory_id: null,
        superseded_by_memory_id: null,
        conflict_refs: ["conflict:source-permission"],
        audit_refs: [],
        created_at: "2026-06-05T00:00:01Z",
        updated_at: "2026-06-05T00:00:01Z",
      },
    ],
    versions: [
      {
        version_id: "memver:memory-center:formal:001",
        memory_id: "mem:formal:offline:included",
        version_number: 1,
        change_type: "created",
        change_summary: "创建正式记忆第一版。",
        record_snapshot: {} as FormalMemoryStoreV1["records"][number],
        source_refs: [],
        changed_by_role: "project_director",
        reviewed_by: null,
        created_at: "2026-06-05T00:00:00Z",
      },
      {
        version_id: "memver:memory-center:formal:002",
        memory_id: "mem:formal:offline:blocked",
        version_number: 1,
        change_type: "created",
        change_summary: "创建正式记忆第一版。",
        record_snapshot: {} as FormalMemoryStoreV1["records"][number],
        source_refs: [],
        changed_by_role: "project_director",
        reviewed_by: null,
        created_at: "2026-06-05T00:00:01Z",
      },
    ],
    audit_events: [
      {
        audit_event_id: "audit:memory-center:formal:001",
        event_type: "memory_record_created",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        session_id: null,
        target_kind: "memory_record",
        target_id: "mem:formal:offline:included",
        before_state: null,
        after_state: "memory_active",
        reason: "M7 记忆中心离线测试。",
        source_refs: [],
        status: "succeeded",
        created_at: "2026-06-05T00:00:00Z",
      },
      {
        audit_event_id: "audit:memory-center:formal:002",
        event_type: "memory_record_created",
        actor_id: "project-director-offline",
        actor_role: "project_director",
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        session_id: null,
        target_kind: "memory_record",
        target_id: "mem:formal:offline:blocked",
        before_state: null,
        after_state: "memory_active",
        reason: "M7 记忆中心 lint 阻断离线测试。",
        source_refs: [],
        status: "succeeded",
        created_at: "2026-06-05T00:00:01Z",
      },
    ],
    updated_at: "2026-06-05T00:00:02Z",
    warnings: [],
  };
  formalMemoryStore.versions[0].record_snapshot = formalMemoryStore.records[0];
  formalMemoryStore.versions[1].record_snapshot = formalMemoryStore.records[1];

  const memoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 8,
    candidates: [
      {
        candidate_id: "memcand:memory-center:001",
        candidate_key: "memcand:v1:memory-center-review",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T00:00:02Z",
        },
        memory_type: "project_memory",
        claim: "候选需要确认要求和风险提示。",
        body: "candidate_confirmed 只表示候选保留，不代表正式记忆。",
        source_refs: [
          {
            source_ref_id: "source:memory-center:candidate:001",
            source_type: "observation_ref",
            source_id: "obs:v1:memory-center:001",
            source_title: "观察来源",
            captured_at: "2026-06-05T00:00:02Z",
            authority_level: "derived_summary",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "project_director",
        generated_from: "observation:obs:v1:memory-center:001",
        status: "candidate_confirmed",
        risk_level: "medium",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "M7 记忆中心候选边界测试。",
        conflicts: [],
        audit_refs: [],
        adoption: null,
        created_at: "2026-06-05T00:00:02Z",
        updated_at: "2026-06-05T00:00:02Z",
      },
      {
        candidate_id: "memcand:memory-center:002",
        candidate_key: "memcand:v1:memory-center-adopted",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T00:00:03Z",
        },
        memory_type: "project_memory",
        claim: "采纳回链必须可见但仍保留候选身份。",
        body: "候选列表显示采纳回链，不把候选行显示成正式记忆。",
        source_refs: [],
        generated_by_role: "project_director",
        generated_from: "stage_handoff",
        status: "candidate_confirmed",
        risk_level: "low",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "M7 记忆中心采纳回链测试。",
        conflicts: [],
        audit_refs: [],
        adoption: {
          adopted_memory_id: "mem:formal:offline:included",
          adopted_version_id: "memver:memory-center:formal:001",
          adopted_audit_event_id: "audit:memory-center:formal:001",
          adopted_at: "2026-06-05T00:00:03Z",
          adopted_by_role: "project_director",
          adoption_reason: "项目主管受控采纳。",
        },
        created_at: "2026-06-05T00:00:03Z",
        updated_at: "2026-06-05T00:00:03Z",
      },
    ],
    events: [],
    updated_at: "2026-06-05T00:00:03Z",
  };

  const observationStore: ObservationStoreV1 = {
    store_version: "observation_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 4,
    observations: [
      {
        observation_id: "obs:v1:memory-center:001",
        observation_key: "obs:v1:memory-center-source",
        schema_version: "memory_observation.v1",
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T00:00:02Z",
        },
        observation_type: "worker_report",
        summary: "开发线汇报形成观察来源。",
        source_refs: [
          {
            source_ref_id: "obs-source:memory-center:001",
            source_kind: "worker_report",
            source_id: "worker-report:m7",
            project_id: "project:offline-fixture-projects-codex-workbench",
            workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
            summary: "worker 报告摘要。",
            sensitive_level: "internal",
            created_at: "2026-06-05T00:00:02Z",
          },
        ],
        status: "candidate_created",
        generated_by_role: "worker",
        actor_id: "codex-dev",
        risk_level: "low",
        sensitive_level: "internal",
        candidate_key: "memcand:v1:memory-center-review",
        audit_refs: [],
        created_at: "2026-06-05T00:00:02Z",
        updated_at: "2026-06-05T00:00:02Z",
      },
    ],
    events: [],
    updated_at: "2026-06-05T00:00:02Z",
    warnings: [],
  };

  const memoryCaptureStore: MemoryCaptureStoreV1 = {
    store_version: "memory_capture_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 3,
    events: [
      {
        capture_event_id: "capture:memory-center:candidate",
        event_key: "memory-center:candidate",
        schema_version: "memory_capture_event.v1",
        source_type: "worker_report",
        source_ref_id: "worker-report:m7",
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:developer",
        run_unit_id: "run-unit:memory-center:developer",
        product_command_id: null,
        product_attempt_id: null,
        runtime_log_ref: "runtime-log:memory-center:candidate",
        audit_refs: ["audit:memory-center:candidate"],
        readback_ref: null,
        task_package_ref: "task-package:memory-center",
        memory_packet_ref: "memory-packet:memory-center",
        summary: "开发线汇报形成候选来源。",
        evidence_summary: "离线记忆中心 fixture。",
        sensitivity: "internal",
        candidate_policy: "candidate_allowed",
        blocked_reason: null,
        observation_id: "obs:v1:memory-center:001",
        candidate_key: "memcand:v1:memory-center-review",
        created_by: "project_director",
        created_at: "2026-06-05T00:00:02Z",
        updated_at: "2026-06-05T00:00:02Z",
      },
      {
        capture_event_id: "capture:memory-center:needs-proof",
        event_key: "memory-center:needs-proof",
        schema_version: "memory_capture_event.v1",
        source_type: "readback",
        source_ref_id: "readback:memory-center",
        project_id: "project:offline-fixture-projects-codex-workbench",
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:developer",
        run_unit_id: "run-unit:memory-center:developer",
        product_command_id: "product-command:memory-center",
        product_attempt_id: "attempt:memory-center",
        runtime_log_ref: "runtime-log:memory-center:needs-proof",
        audit_refs: ["audit:memory-center:needs-proof"],
        readback_ref: "readback:memory-center",
        task_package_ref: "task-package:memory-center",
        memory_packet_ref: "memory-packet:memory-center",
        summary: "捕获事件声明允许候选但缺少回链。",
        evidence_summary: "离线记忆中心补证 fixture。",
        sensitivity: "internal",
        candidate_policy: "candidate_allowed",
        blocked_reason: null,
        observation_id: null,
        candidate_key: null,
        created_by: "project_director",
        created_at: "2026-06-05T00:00:07Z",
        updated_at: "2026-06-05T00:00:07Z",
      },
    ],
    updated_at: "2026-06-05T00:00:07Z",
    warnings: [],
  };

  const memoryLintStore: MemoryLintStoreV1 = {
    store_version: "memory_lint_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 6,
    findings: [
      {
        finding_id: "memlint:memory-center:blocking",
        schema_version: "memory_governance.v1",
        finding_type: "source_permission_revoked",
        severity: "blocking",
        status: "open",
        source_kind: "memory_record",
        source_id: "mem:formal:offline:blocked",
        target_memory_id: "mem:formal:offline:blocked",
        target_candidate_key: null,
        scope_type: "project",
        memory_type: "project_memory",
        claim: "撤回来源不能进入任务记忆包。",
        summary: "来源权限撤回产生 open blocking finding。",
        recommended_action: "exclude_from_task_packet",
        evidence_refs: [],
        audit_event_id: null,
        created_at: "2026-06-05T00:00:04Z",
        updated_at: "2026-06-05T00:00:04Z",
      },
      {
        finding_id: "memlint:memory-center:review",
        schema_version: "memory_governance.v1",
        finding_type: "candidate_conflicts_with_active_memory",
        severity: "needs_review",
        status: "open",
        source_kind: "memory_candidate",
        source_id: "memcand:v1:memory-center-review",
        target_memory_id: "mem:formal:offline:included",
        target_candidate_key: "memcand:v1:memory-center-review",
        scope_type: "project",
        memory_type: "project_memory",
        claim: "候选需要确认要求和风险提示。",
        summary: "候选与 active 记忆需要人工复核。",
        recommended_action: "block_adoption",
        evidence_refs: [],
        audit_event_id: null,
        created_at: "2026-06-05T00:00:05Z",
        updated_at: "2026-06-05T00:00:05Z",
      },
    ],
    runs: [
      {
        run_id: "memlint-run:memory-center",
        lint_intent: "task_packet_guard",
        actor_id: "system",
        actor_role: "system",
        finding_ids: ["memlint:memory-center:blocking"],
        blocking_count: 1,
        status: "blocked",
        reason: "task packet guard blocked by revoked source",
        report_id: null,
        created_at: "2026-06-05T00:00:06Z",
      },
      {
        run_id: "memlint-run:memory-center:maintenance",
        lint_intent: "maintenance_run",
        actor_id: "project-director-memory-center",
        actor_role: "project_director",
        finding_ids: ["memlint:memory-center:blocking", "memlint:memory-center:review"],
        blocking_count: 1,
        status: "blocked",
        reason: "maintenance_run found 1 blocking finding(s)",
        report_id: "memory-maintenance-report:v1:offline",
        created_at: "2026-06-05T00:00:07Z",
      },
    ],
    maintenance_reports: [
      {
        report_id: "memory-maintenance-report:v1:offline",
        run_id: "memlint-run:memory-center:maintenance",
        checked_memory_count: 2,
        checked_candidate_count: 2,
        checked_observation_count: 1,
        checked_relation_count: 1,
        open_count: 2,
        blocking_count: 1,
        needs_review_count: 1,
        info_count: 0,
        check_summaries: [
          {
            check_kind: "permission_revocation",
            checked_count: 2,
            finding_count: 1,
            blocking_count: 1,
            needs_review_count: 0,
            info_count: 0,
            display_text: "权限撤回：checked 2 / finding 1 / blocking 1 / needs_review 0 / info 0",
          },
          {
            check_kind: "entity_relation_drift",
            checked_count: 2,
            finding_count: 1,
            blocking_count: 0,
            needs_review_count: 1,
            info_count: 0,
            display_text: "实体 / 关系漂移：checked 2 / finding 1 / blocking 0 / needs_review 1 / info 0",
          },
        ],
        recommendations: [
          {
            recommendation_id: "memory-maintenance-recommendation:v1:offline",
            severity: "blocking",
            target_kind: "memory_record",
            target_id: "mem:formal:offline:blocked",
            action_label: "review_source_permission",
            display_text: "source_permission_revoked：来源权限撤回产生 open blocking finding；建议人工复核，不会自动执行 lifecycle",
          },
        ],
        index_status: {
          status: "stale",
          formal_store_revision: 4,
          lint_store_revision: 6,
          entity_relation_store_revision: 2,
          checked_at: "2026-06-05T00:00:07Z",
          display_text: "索引状态 stale：formal rev 4 / entity-relation rev 2；状态 finding 不会重建索引或改变事实",
          warnings: ["derived_index_status_requires_review"],
        },
        display_text: "维护任务摘要：检查正式记忆 2 / 候选 2 / observation 1 / relation 1；open 2 / blocking 1 / needs_review 1 / info 0。维护任务只生成 finding，不会自动修改正式记忆。",
        warnings: ["memory_maintenance_findings_only"],
        created_at: "2026-06-05T00:00:07Z",
      },
    ],
    updated_at: "2026-06-05T00:00:06Z",
    warnings: [],
  };
  const memoryEntityRelationStore: MemoryEntityRelationStoreV1 = {
    store_version: "memory_entity_relations.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 2,
    registry: {
      entities: [
        {
          entity_id: "entity:v1:tool:codex",
          entity_kind: "tool",
          canonical_key: "tool:codex",
          display_name: "Codex CLI",
          aliases: [
            {
              alias_id: "entity-alias:v1:codex",
              alias: "codex tool",
              source_kind: "formal_memory",
              source_id: "mem:formal:offline:included",
              created_at: "2026-06-05T00:00:07Z",
            },
          ],
          source_refs: [
            {
              source_kind: "formal_memory",
              source_id: "mem:formal:offline:included",
              source_path: null,
              source_title: "接口验收必须保留控制核心边界。",
              authority_level: "formal_memory",
              sensitive_level: "project",
            },
          ],
          status: "registered",
          created_at: "2026-06-05T00:00:07Z",
          updated_at: "2026-06-05T00:00:07Z",
          warnings: [],
        },
      ],
      updated_at: "2026-06-05T00:00:07Z",
      warnings: ["memory_entity_registry_minimal"],
    },
    entity_candidates: [],
    merge_candidates: [],
    relation_candidates: [],
    relations: [
      {
        relation_id: "relation:v1:memory-center:001",
        relation_kind: "causal",
        subject_entity_id: "entity:v1:memory_record:included",
        object_entity_id: "entity:v1:knowledge_doc:contract",
        subject_label: "接口验收必须保留控制核心边界。",
        object_label: "接口契约资料",
        predicate: "causal_candidate",
        source_kind: "manual",
        source_refs: [
          {
            source_kind: "manual",
            source_id: "manual:contract-change",
            source_path: null,
            source_title: "接口契约资料",
            authority_level: "evidence",
            sensitive_level: "project",
          },
        ],
        status: "confirmed",
        confirmed_by: "project_director",
        confirmation_role: "project_director",
        confirmation_reason: "项目主管确认低风险本项目因果关系。",
        created_at: "2026-06-05T00:00:08Z",
        updated_at: "2026-06-05T00:00:08Z",
        warnings: ["confirmed_relation_explains_retrieval_only"],
      },
    ],
    audit_events: [
      {
        audit_event_id: "audit:memory-relation:v1:offline",
        event_type: "memory_relation_candidate_decision_recorded",
        actor_id: "project-director-memory-center",
        actor_role: "project_director",
        target_kind: "memory_relation_candidate",
        target_id: "relation-candidate:v1:offline",
        before_status: "candidate",
        after_status: "confirmed",
        reason: "确认关系候选。",
        created_at: "2026-06-05T00:00:08Z",
        warnings: [],
      },
    ],
    updated_at: "2026-06-05T00:00:08Z",
    warnings: ["memory_entity_relation_store_m10_minimal_sidecar"],
  };
  const memoryEntityRelationPreview: MemoryEntityRelationPreviewOutput = {
    store_revision: memoryEntityRelationStore.revision,
    entity_candidates: [
      {
        candidate_id: "entity-candidate:v1:codex-cli",
        entity_kind: "tool",
        display_name: "Codex CLI",
        normalized_key: "tool:codex",
        source_kind: "formal_memory",
        source_id: "mem:formal:offline:included",
        source_path: null,
        source_title: "Codex CLI",
        source_refs: [],
        confidence_kind: "source_ref",
        status: "candidate",
        reason: "来源引用派生实体候选。",
        created_at: "2026-06-05T00:00:09Z",
        warnings: [],
      },
    ],
    merge_candidates: [
      {
        merge_candidate_id: "entity-merge-candidate:v1:codex",
        left_entity_candidate_id: "entity-candidate:v1:codex-cli",
        right_entity_candidate_id: "entity-candidate:v1:codex-tool",
        left_label: "Codex CLI",
        right_label: "codex tool",
        normalized_key: "tool:codex",
        source_kind: "similarity_hit",
        status: "candidate",
        requires_user_confirmation: false,
        reason: "相似度命中仅作候选，需人工确认后才会登记实体合并决定。",
        created_at: "2026-06-05T00:00:09Z",
        warnings: [],
      },
    ],
    relation_candidates: [
      {
        candidate_id: "relation-candidate:v1:contract",
        relation_kind: "causal",
        subject_entity_id: "entity:v1:memory_record:included",
        object_entity_id: "entity:v1:knowledge_doc:contract",
        subject_label: "接口验收必须保留控制核心边界。",
        object_label: "接口契约资料",
        predicate: "causal_candidate",
        source_kind: "manual",
        source_refs: [],
        confidence_kind: "deterministic_source_ref",
        status: "candidate",
        requires_user_confirmation: false,
        reason: "待确认因果关系；确认后才可用于解释召回原因。",
        created_at: "2026-06-05T00:00:09Z",
        warnings: [],
      },
      {
        candidate_id: "relation-candidate:v1:llm",
        relation_kind: "causal",
        subject_entity_id: "entity:v1:memory_record:included",
        object_entity_id: "entity:v1:proposal:llm",
        subject_label: "接口验收必须保留控制核心边界。",
        object_label: "LLM 因果推断",
        predicate: "llm_inferred_candidate",
        source_kind: "llm_inferred",
        source_refs: [],
        confidence_kind: "llm_inferred_candidate_only",
        status: "candidate",
        requires_user_confirmation: true,
        reason: "LLM 推断仅作候选，不能直接进入已确认关系。",
        created_at: "2026-06-05T00:00:09Z",
        warnings: [],
      },
    ],
    summary: {
      sidecar_name: "memory-entity-relations.v1.json",
      revision: memoryEntityRelationStore.revision,
      entity_count: 1,
      entity_candidate_count: 1,
      merge_candidate_count: 1,
      relation_candidate_count: 2,
      confirmed_relation_count: 1,
      display_text: "实体 / 关系治理 fixture。",
      warnings: [],
    },
    warnings: ["llm_inferred_relation_candidate_only", "similarity_hit_candidate_only"],
  };
  const maturePatternScope = {
    scope_id: "scope:global:mature-pattern:memory-center",
    scope_type: "global" as const,
    project_id: null,
    workflow_id: null,
    session_id: null,
    role_ids: [],
    document_refs: [],
    permission_policy_ref: "memory_policy:global:user_confirmed",
    model_export_policy: "local_only" as const,
    valid_from: "2026-06-05T00:00:10Z",
  };
  const maturePatternSourceRef = {
    source_ref_id: "source:mature-pattern:v1:maintenance",
    source_type: "evidence" as const,
    source_id: "memory-maintenance-report:v1:offline",
    source_path: "/offline-fixture/evidence/m11.md",
    source_title: "M11 mature pattern signal",
    anchor: "重复控制核心边界",
    captured_at: "2026-06-05T00:00:10Z",
    authority_level: "evidence" as const,
    sensitive_level: "project" as const,
  };
  const memoryPatternStore: MemoryPatternStoreV1 = {
    store_version: "memory_patterns.v1",
    project_id: null,
    workflow_id: null,
    revision: 12,
    mature_pattern_candidates: [
      {
        candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
        pattern_kind: "repeated_review_boundary",
        scope: maturePatternScope,
        title: "跨项目重复边界：控制核心写入必须走确认",
        claim: "跨项目重复出现控制核心写入边界，成熟模式候选需要用户确认后才能成为正式记忆。",
        body: "该候选来自维护 signal、正式记忆和观察来源的重复主题；候选未确认，不会进入任务包。",
        source_refs: [maturePatternSourceRef],
        member_refs: [
          {
            member_ref_id: "cluster-member:v1:formal-memory",
            member_kind: "formal_memory",
            member_id: "mem:formal:offline:included",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "接口验收必须保留控制核心边界。",
            source_refs: [maturePatternSourceRef],
          },
          {
            member_ref_id: "cluster-member:v1:observation",
            member_kind: "observation",
            member_id: "observation:v1:memory-center",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "观察来源说明候选不是正式记忆。",
            source_refs: [maturePatternSourceRef],
          },
        ],
        signal_refs: ["mature_pattern_signal:v1:maintenance"],
        status: "candidate",
        requires_user_confirmation: true,
        review_summary: "秘书或全局主管只能汇总；用户确认前不写正式记忆。",
        created_at: "2026-06-05T00:00:10Z",
        updated_at: "2026-06-05T00:00:10Z",
        warnings: ["mature_pattern_candidate_requires_user_confirmation"],
      },
    ],
    cluster_reports: [
      {
        report_id: "memory-cluster-report:v1:control-core-boundary",
        report_kind: "cross_project_theme",
        scope_type: "global",
        title: "跨项目主题报告：控制核心边界",
        project_ids: ["project:offline-fixture-projects-codex-workbench", "project:offline-fixture-projects-other"],
        member_refs: [
          {
            member_ref_id: "cluster-member:v1:formal-memory",
            member_kind: "formal_memory",
            member_id: "mem:formal:offline:included",
            project_id: "project:offline-fixture-projects-codex-workbench",
            title: "接口验收必须保留控制核心边界。",
            source_refs: [maturePatternSourceRef],
          },
        ],
        source_refs: [maturePatternSourceRef],
        status: "derived_report",
        staleness: "fresh",
        display_text: "跨项目主题报告只解释重复主题和来源下钻，不是正式事实。",
        created_at: "2026-06-05T00:00:10Z",
        warnings: ["memory_cluster_report_not_formal_memory"],
      },
    ],
    audit_events: [],
    updated_at: "2026-06-05T00:00:10Z",
    warnings: ["memory_pattern_store_m12_minimal_sidecar"],
  };
  const maturePatternPreview: MaturePatternPreviewOutput = {
    store_revision: memoryPatternStore.revision,
    mature_pattern_candidates: memoryPatternStore.mature_pattern_candidates,
    cluster_reports: memoryPatternStore.cluster_reports,
    acceptance_summary: {
      summary_id: "memory-acceptance-summary:v1:m12",
      scope_label: "M1-M12 memory layer",
      gate_count: 4,
      passed_count: 3,
      blocked_count: 0,
      deferred_count: 1,
      gates: [
        {
          gate_id: "gate:m1-formal-store",
          label: "M1 formal memory store",
          status: "passed",
          evidence: "formal records include source, version and audit refs",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m4-task-packet",
          label: "M4 task packet recall",
          status: "passed",
          evidence: "活跃正式记忆可以进入入选列表",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m11-maintenance",
          label: "M11 maintenance finding boundary",
          status: "passed",
          evidence: "maintenance report creates findings only",
          blocking_reason: null,
        },
        {
          gate_id: "gate:m13-final-freeze",
          label: "M13 final authority freeze",
          status: "deferred",
          evidence: "outside M12 scope",
          blocking_reason: null,
        },
      ],
      display_text: "M1-M12 门禁摘要：通过 3 / 阻断 0 / 后置 1。",
      warnings: ["m12_is_not_m13_final_acceptance"],
      created_at: "2026-06-05T00:00:10Z",
    },
    summary: {
      sidecar_name: "memory-patterns.v1.json",
      revision: memoryPatternStore.revision,
      mature_pattern_candidate_count: 1,
      cluster_report_count: 1,
      confirmed_pattern_count: 0,
      display_text: "成熟模式候选 1 / 跨项目主题报告 1 / 待用户确认 1。",
      warnings: [],
    },
    warnings: ["memory_cluster_report_not_formal_memory"],
  };

  const summary = deriveMemoryManagementSummary({
    projects: [project],
    workflowState: workflowStateWithDerivedWorkflow,
    formalMemoryStore,
    memoryCaptureStore,
    memoryCandidateStore,
    observationStore,
    memoryLintStore,
    memoryEntityRelationStore,
    memoryEntityRelationPreview,
    memoryPatternStore,
    maturePatternPreview,
  });

  assert(summary.source_kind === "frontend_read_model", "记忆中心必须声明前端只读读模型");
  assert(summary.formal_memories.length === 2, "记忆中心正式记忆数量不匹配");
  assert(summary.candidate_memories.length === 2, "记忆中心候选数量不匹配");
  assert(summary.observation_sources.length === 1, "记忆中心观察来源数量不匹配");
  assert(summary.memory_workbench_summary.capture_count === 2, "记忆工作台捕获数量不匹配");
  assert(summary.memory_workbench_summary.capture_compensation_count === 1, "记忆工作台补证数量不匹配");
  assert(summary.memory_workbench_summary.confirmed_pending_formalization_count === 1, "记忆工作台待正式化数量不匹配");
  assert(summary.memory_workbench_summary.task_package_included_count === 1, "记忆工作台任务包入选数量不匹配");
  assert(summary.memory_workbench_summary.task_package_review_material_count === 2, "记忆工作台待审材料数量不匹配");
  assert(summary.memory_workbench_summary.action_items.some((item) => item.kind === "repair_capture_link"), "记忆工作台缺补证行动项");
  assert(summary.memory_workbench_summary.action_items.some((item) => item.kind === "confirm_formalization"), "记忆工作台缺正式化行动项");
  assert(summary.memory_workbench_summary.boundary_text.includes("观察和候选都不是正式记忆"), "记忆工作台缺候选 / 观察边界");
  assert(summary.task_package_summary.included_count === 1, "记忆中心任务包 included 摘要不匹配");
  assert(summary.task_package_summary.review_material_count === 2, "记忆中心任务包待审材料摘要不匹配");
  assert(summary.entity_relation_summary.entity_candidate_count === 1, "实体候选摘要不匹配");
  assert(summary.entity_relation_summary.merge_candidate_count === 1, "实体 dedupe 候选摘要不匹配");
  assert(summary.entity_relation_summary.relation_candidate_count === 2, "关系候选摘要不匹配");
  assert(summary.entity_relation_summary.confirmed_relation_count === 1, "已确认关系摘要不匹配");
  assert(summary.entity_relation_summary.display_text.includes("LLM 推断仅作候选"), "实体关系摘要缺 LLM 候选边界");
  assert(summary.entity_relation_summary.display_text.includes("相似度命中仅作候选"), "实体关系摘要缺相似度候选边界");
  assert(summary.mature_pattern_summary.mature_pattern_candidate_count === 1, "成熟模式候选摘要不匹配");
  assert(summary.mature_pattern_summary.cluster_report_count === 1, "跨项目主题报告摘要不匹配");
  assert(summary.mature_pattern_summary.user_confirmation_required_count === 1, "成熟模式用户确认计数不匹配");
  assert(summary.mature_pattern_summary.acceptance_summary?.passed_count === 3, "M1-M12 gate 摘要不匹配");
  assert(summary.mature_pattern_summary.boundary_text.includes("候选未确认，不会进入任务包"), "成熟模式摘要缺未确认边界");
  assert(summary.project_summaries.some((item) => item.project_name === "codex-workbench"), "记忆中心缺项目相关记忆摘要");

  const includedMemory = summary.formal_memories.find((item) => item.claim === "接口验收必须保留控制核心边界。");
  assert(includedMemory, "记忆中心缺少可入选正式记忆");
  assert(includedMemory.kind_label === "正式记忆", "正式记忆条目必须标识正式记忆");
  assert(includedMemory.task_eligibility.label === "可进入任务包", "active 正式记忆应显示可进入任务包");
  assert(includedMemory.task_eligibility.included_in_task_package, "任务包 available_memory_refs 命中的正式记忆应显示已被快照引用");
  assert(includedMemory.version_summary.includes("v1"), "正式记忆条目缺少版本摘要");
  assert(includedMemory.audit_summary.includes("memory_record_created"), "正式记忆条目缺少审计摘要");

  const blockedMemory = summary.formal_memories.find((item) => item.claim === "撤回来源不能进入任务记忆包。");
  assert(blockedMemory, "记忆中心缺少 lint 阻断正式记忆");
  assert(blockedMemory.task_eligibility.label === "被检查阻断", "未关闭阻断发现应阻断任务包入选");
  assert(blockedMemory.conflict_summary.includes("未关闭阻断"), "检查阻断正式记忆应显示冲突摘要");

  const candidate = summary.candidate_memories.find((item) => item.claim === "候选需要确认要求和风险提示。");
  assert(candidate, "记忆中心缺候选条目");
  assert(candidate.kind_label === "候选记忆", "候选条目必须标识候选记忆");
  assert(candidate.formal_memory_boundary.includes("不是正式记忆"), "候选条目必须说明不是正式记忆");
  assert(candidate.task_position.label === "待审查材料", "未采纳候选应显示待审查材料");
  assert(candidate.confirmation_summary.includes("需要用户确认"), "候选条目缺确认要求");

  const adoptedCandidate = summary.candidate_memories.find((item) => item.claim === "采纳回链必须可见但仍保留候选身份。");
  assert(adoptedCandidate, "记忆中心缺受控采纳候选");
  assert(adoptedCandidate.adoption_summary.includes("候选已被受控采纳"), "候选采纳回链应用允许文案说明");
  assert(adoptedCandidate.kind_label === "候选记忆", "已采纳候选在候选列表仍应显示候选记忆");

  const memoryCenterText = visibleText(
    <MemoryCenterView
      projects={[project]}
      workflowState={workflowStateWithDerivedWorkflow}
      formalMemoryStore={formalMemoryStore}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      observationStore={observationStore}
      memoryLintStore={memoryLintStore}
      memoryEntityRelationStore={memoryEntityRelationStore}
      memoryPatternStore={memoryPatternStore}
      onPreviewMemoryEntityRelationCandidates={() => Promise.resolve(memoryEntityRelationPreview)}
      onPreviewMaturePatterns={() => Promise.resolve(maturePatternPreview)}
      hasRealSnapshot
    />,
  );
  for (const expectedText of [
    "正式记忆",
    "记忆工作台",
    "捕获 / 候选 / 任务记忆包",
    "记忆链路",
    "捕获 2 / 观察 1 / 候选 2 / 正式 2",
    "待正式化 1",
    "需补证 1",
    "任务记忆包快照 1 个",
    "确认正式化",
    "补齐捕获链路",
    "候选和观察不会冒充正式记忆",
    "候选记忆",
    "来源",
    "版本 v1",
    "审计 memory_record_created",
    "权限策略未记录",
    "外发 local_only",
    "外发 blocked",
    "可进入任务包",
    "任务包冻结快照已引用",
    "被检查阻断",
    "未关闭阻断",
    "待审查材料",
    "候选已被受控采纳",
    "观察来源",
    "观察不是正式记忆",
    "任务包冻结快照",
    "入选 1",
    "排除 2",
    "待审材料 2",
    "项目相关记忆摘要",
    "codex-workbench",
    "生命周期",
    "编辑提案",
    "废弃",
    "冻结",
    "解冻",
    "归档",
    "合并",
    "拆分",
    "上升为全局",
    "下沉为项目",
    "编辑会创建新版本，不覆盖旧版本",
    "实体候选",
    "关系候选",
    "已确认关系",
    "刷新实体 / 关系候选",
    "相似度命中仅作候选",
    "LLM 推断仅作候选",
    "已确认关系用于解释召回原因",
    "关系候选不会影响任务包入选清单",
    "维护任务",
    "维护任务摘要",
    "运行维护任务",
    "索引状态 stale",
    "维护任务只生成发现",
    "阻断级发现会阻止召回",
    "成熟模式 / 跨项目主题",
    "刷新成熟模式候选",
    "成熟模式候选",
    "跨项目重复边界：控制核心写入必须走确认",
    "需要用户确认",
    "候选未确认，不会进入任务包",
    "用户确认为正式记忆",
    "隔离",
    "要求补来源",
    "跨项目主题报告",
    "报告可下钻来源，但不是正式事实",
    "M1-M12 门禁摘要",
    "尚未生成 M12 预览",
    "最终权威验收仍在后续阶段",
  ]) {
    assert(memoryCenterText.includes(expectedText), `记忆中心 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已记住",
    "系统已长期记住",
    "候选已成为正式记忆",
    "观察已成为正式记忆",
    "worker 已收到记忆包",
    "中间版本记忆层已完成",
    "编辑正式记忆",
    "删除正式记忆",
    "归档正式记忆",
    "自动合并实体",
    "自动确认关系",
    "图谱已证明",
    "LLM 已确认关系",
    "相似度已合并实体",
    "GraphRAG 已接入",
    "关系候选已成为事实",
    "自动清理记忆",
    "自动修复记忆",
    "维护任务已改正式记忆",
    "成熟模式已自动成为规则",
    "自动成为技能",
    "自动成为全局规则",
    "自动写入全局记忆",
    "跨项目摘要已注入任务包",
    "聚类报告就是事实",
    "成熟模式已生效",
    "M13 已完成",
    "中间版本记忆系统最终验收完成",
  ]) {
    assert(!memoryCenterText.includes(forbiddenText), `记忆中心 UI 不应出现越界文案：${forbiddenText}`);
  }

  const memoryCenterMarkup = renderToStaticMarkup(
    <MemoryCenterView
      projects={[project]}
      workflowState={workflowStateWithDerivedWorkflow}
      formalMemoryStore={formalMemoryStore}
      memoryCaptureStore={memoryCaptureStore}
      memoryCandidateStore={memoryCandidateStore}
      observationStore={observationStore}
      memoryLintStore={memoryLintStore}
      memoryEntityRelationStore={memoryEntityRelationStore}
      memoryPatternStore={memoryPatternStore}
      hasRealSnapshot
    />,
  );
  for (const expectedClass of ["memory-center", "memory-workbench-panel", "formal-memory-item", "candidate-memory-item", "memory-detail-panel", "memory-entity-relation-panel", "memory-maintenance-panel", "memory-mature-pattern-panel"]) {
    assert(memoryCenterMarkup.includes(expectedClass), `记忆中心布局缺少 class ${expectedClass}`);
  }

  const lifecyclePreview: FormalMemoryLifecyclePreview = {
    preview_id: "formal-memory-lifecycle-preview:test",
    operation_kind: "deprecate",
    store_revision: formalMemoryStore.revision,
    target_memory_ids: [includedMemory.memory_id],
    impact: {
      affected_memory_ids: [includedMemory.memory_id],
      created_memory_ids: [],
      status_changes: [
        {
          memory_id: includedMemory.memory_id,
          before_status: "memory_active",
          after_status: "memory_deprecated",
        },
      ],
      created_memory_count: 0,
      new_version_count: 1,
      task_packet_eligibility_change: "非活跃记忆默认不进任务包入选列表。",
      source_ref_count: includedMemory.record.source_refs.length,
      display_text: "影响 1 条正式记忆，新增 1 个版本；非活跃记忆默认不进任务包入选列表。",
      warnings: ["formal_memory_lifecycle_versions_and_audit_recorded"],
    },
    required_approval: {
      required: true,
      approval_kind: "project_director_or_user_confirmation",
      required_actor_role: "project_director_or_user",
      reason: "项目内正式记忆 lifecycle 需要项目主管或用户确认。",
    },
    before_records: [includedMemory.record],
    proposed_records: [
      {
        ...includedMemory.record,
        record_version: includedMemory.record.record_version + 1,
        status: "memory_deprecated",
      },
    ],
    display_text: "废弃预览：影响 1 条 / 新版本 1 个",
    warnings: ["formal_memory_lifecycle_versions_and_audit_recorded"],
  };
  const lifecycleAction: PendingAction = {
    kind: "record-formal-memory-lifecycle-operation",
    label: "正式记忆 废弃",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary: "编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；非活跃记忆默认不进任务包。",
    formalMemoryLifecycle: {
      project_root: project.project_root,
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      operation_kind: "deprecate",
      memory_id: includedMemory.memory_id,
      memory_ids: [],
      revise: null,
      merge: null,
      split: null,
      scope_change: null,
      actor_id: "project-director-ui",
      actor_role: "project_director",
      reason: "废弃正式记忆测试。",
      expected_store_revision: formalMemoryStore.revision,
      expected_record_versions: {
        [includedMemory.memory_id]: includedMemory.record.record_version,
      },
      confirmed_by: "project-director-ui",
      confirmation_summary: "已查看影响面。",
    },
    formalMemoryLifecyclePreview: lifecyclePreview,
  };
  const lifecycleDialogText = visibleText(
    <PermissionDialog action={lifecycleAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "正式记忆 废弃",
    "formal-memories.v1.json",
    "确认权",
    "project_director_or_user_confirmation",
    "影响 1 条正式记忆",
    "非活跃记忆默认不进任务包",
    "原版本 v1 / 新版本 v2",
    "会新增版本和审计",
  ]) {
    assert(lifecycleDialogText.includes(expectedText), `正式记忆 lifecycle 确认弹层缺少 ${expectedText}`);
  }
  const relationAction: PendingAction = {
    kind: "record-memory-relation-candidate-decision",
    label: "确认关系候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-entity-relations.v1.json；已确认关系只用于解释召回原因，不改变任务包入选列表。",
    memoryRelationCandidateDecision: {
      project_root: project.project_root,
      relation_candidate_id: "relation-candidate:v1:contract",
      decision: "confirm_relation",
      actor_id: "project-director-memory-center",
      actor_role: "project_director",
      confirmed_by: "project_director",
      reason: "项目主管确认关系候选。",
      expected_store_revision: memoryEntityRelationStore.revision,
    },
    memoryRelationCandidate: memoryEntityRelationPreview.relation_candidates[0],
  };
  const relationDialogText = visibleText(
    <PermissionDialog action={relationAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "确认关系候选",
    "memory-entity-relations.v1.json",
    "接口验收必须保留控制核心边界。",
    "接口契约资料",
    "已确认关系用于解释召回原因",
    "关系候选不会作为正式事实影响工作者",
  ]) {
    assert(relationDialogText.includes(expectedText), `关系候选确认弹层缺少 ${expectedText}`);
  }

  const maintenanceAction: PendingAction = {
    kind: "run-memory-maintenance",
    label: "运行记忆维护任务",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-lint.v1.json 的维护运行 / 发现项 / 报告；不会自动修改正式记忆、候选、观察、实体关系或工作流状态。",
    memoryMaintenanceRun: {
      project_root: project.project_root,
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      actor_id: "project-director-memory-center",
      actor_role: "project_director",
      lint_intent: "maintenance_run",
      candidate_key: null,
      task_id: "memory-maintenance:m11",
      revoked_source_ids: [],
      expected_formal_store_revision: formalMemoryStore.revision,
      expected_candidate_store_revision: memoryCandidateStore.revision,
      expected_lint_store_revision: memoryLintStore.revision,
      dry_run: false,
    },
  };
  const maintenanceDialogText = visibleText(
    <PermissionDialog action={maintenanceAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "运行记忆维护任务",
    "memory-lint.v1.json",
    "维护运行",
    "维护任务只生成发现 / 报告",
    "阻断级发现会阻止召回",
    "不会自动修改正式记忆",
  ]) {
    assert(maintenanceDialogText.includes(expectedText), `维护任务确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["自动清理记忆", "自动修复记忆", "自动合并重复记忆", "维护任务已改正式记忆"]) {
    assert(!maintenanceDialogText.includes(forbiddenText), `维护任务确认弹层不应出现越界文案：${forbiddenText}`);
  }

  const maturePatternAction: PendingAction = {
    kind: "record-mature-pattern-decision",
    label: "用户确认成熟模式候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary: "用户确认后写 memory-patterns.v1.json，并通过正式记忆受控路径写 formal-memories.v1.json；候选和报告未确认不进入任务包。",
    maturePatternDecision: {
      project_root: project.project_root,
      candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
      decision: "confirm_as_formal_memory",
      actor_id: "user-memory-center",
      actor_role: "user",
      confirmed_by: "user",
      reason: "用户确认成熟模式候选。",
      expected_pattern_store_revision: memoryPatternStore.revision,
      expected_formal_store_revision: formalMemoryStore.revision,
    },
    maturePatternCandidate: memoryPatternStore.mature_pattern_candidates[0],
  };
  assert(maturePatternAction?.kind === "record-mature-pattern-decision", "成熟模式确认按钮应生成 record-mature-pattern-decision action");
  assert(maturePatternAction.maturePatternDecision?.decision === "confirm_as_formal_memory", "成熟模式确认 action 决定类型不匹配");
  assert(maturePatternAction.maturePatternDecision.actor_role === "user", "成熟模式正式化必须由用户角色确认");
  assert(maturePatternAction.maturePatternDecision.confirmed_by === "user", "成熟模式正式化必须 confirmed_by user");
  assert(maturePatternAction.maturePatternDecision.expected_pattern_store_revision === memoryPatternStore.revision, "成熟模式确认 action 缺 M12 revision guard");
  assert(maturePatternAction.maturePatternDecision.expected_formal_store_revision === formalMemoryStore.revision, "成熟模式确认 action 缺 formal memory revision guard");

  const maturePatternDialogText = visibleText(
    <PermissionDialog action={maturePatternAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "用户确认成熟模式候选",
    "memory-patterns.v1.json / formal-memories.v1.json",
    "跨项目重复边界：控制核心写入必须走确认",
    "用户确认写入正式记忆",
    "候选和跨项目主题报告未确认不进入任务包",
    "写版本、审计和来源引用",
    "只有用户确认正式化时才会联动 formal-memories.v1.json",
  ]) {
    assert(maturePatternDialogText.includes(expectedText), `成熟模式确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["自动成为技能", "自动成为全局规则", "自动写入全局记忆", "跨项目摘要已注入任务包", "成熟模式已生效"]) {
    assert(!maturePatternDialogText.includes(forbiddenText), `成熟模式确认弹层不应出现越界文案：${forbiddenText}`);
  }

  const quarantineMaturePatternAction: PendingAction = {
    kind: "record-mature-pattern-decision",
    label: "隔离成熟模式候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary: "只写 memory-patterns.v1.json 的候选决定；不写正式记忆，不改来源材料，不影响任务包入选列表。",
    maturePatternDecision: {
      project_root: project.project_root,
      candidate_id: "mature-pattern-candidate:v1:control-core-boundary",
      decision: "quarantine",
      actor_id: "user-memory-center",
      actor_role: "user",
      confirmed_by: null,
      reason: "用户隔离成熟模式候选。",
      expected_pattern_store_revision: memoryPatternStore.revision,
      expected_formal_store_revision: null,
    },
    maturePatternCandidate: memoryPatternStore.mature_pattern_candidates[0],
  };
  const quarantineDialogText = visibleText(
    <PermissionDialog action={quarantineMaturePatternAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["隔离成熟模式候选", "memory-patterns.v1.json", "隔离候选", "未确认正式化", "候选和跨项目主题报告未确认不进入任务包"]) {
    assert(quarantineDialogText.includes(expectedText), `成熟模式隔离弹层缺少 ${expectedText}`);
  }
  assert(!quarantineDialogText.includes("memory-patterns.v1.json / formal-memories.v1.json"), "隔离动作不应声明写 formal store");
}

function runKnowledgeBaseBoundaryScenario() {
  const knowledgePath = "/offline-fixture/projects/codex-workbench/docs/interface-contract.md";
  const projectWithKnowledge: ProjectRecord = {
    ...project,
    authority_files: [
      {
        kind: "knowledge_doc",
        name: "接口契约资料",
        path: knowledgePath,
        warnings: [],
      },
    ],
  };
  const formalMemoryStore: FormalMemoryStoreV1 = {
    store_version: "formal_memory_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 9,
    records: [
      {
        memory_id: "mem:formal:knowledge:001",
        schema_version: "memory_governance.v1",
        record_version: 1,
        scope: {
          scope_id: "scope:project:knowledge",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          role_ids: [],
          document_refs: [knowledgePath],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T01:00:00Z",
        },
        memory_type: "project_memory",
        claim: "接口契约资料可以作为正式记忆来源。",
        body: "正式记忆引用 knowledge_doc 来源，但资料本身不是正式记忆。",
        source_refs: [
          {
            source_ref_id: "source:knowledge:formal:001",
            source_type: "knowledge_doc",
            source_id: "knowledge-doc:interface-contract",
            source_path: knowledgePath,
            source_title: "接口契约资料",
            anchor: "接口边界",
            captured_at: "2026-06-05T01:00:00Z",
            authority_level: "knowledge_material",
            sensitive_level: "project",
          },
        ],
        status: "memory_active",
        supersedes_memory_id: null,
        superseded_by_memory_id: null,
        conflict_refs: [],
        audit_refs: [],
        created_at: "2026-06-05T01:00:00Z",
        updated_at: "2026-06-05T01:00:00Z",
      },
    ],
    versions: [],
    audit_events: [],
    updated_at: "2026-06-05T01:00:00Z",
    warnings: [],
  };
  const memoryCandidateStore: MemoryCandidateStoreV1 = {
    store_version: "memory_candidate_store.v1",
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    revision: 10,
    candidates: [
      {
        candidate_id: "memcand:knowledge:001",
        candidate_key: "memcand:v1:knowledge-interface",
        schema_version: "memory_governance.v1",
        scope: {
          scope_id: "scope:project:knowledge",
          scope_type: "project",
          project_id: "project:offline-fixture-projects-codex-workbench",
          workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
          role_ids: [],
          document_refs: [knowledgePath],
          model_export_policy: "local_only",
          valid_from: "2026-06-05T01:00:01Z",
        },
        memory_type: "project_memory",
        claim: "知识库资料可提出候选。",
        body: "候选仍需确认和受控采纳，不能直接进入正式记忆。",
        source_refs: [
          {
            source_ref_id: "source:knowledge:candidate:001",
            source_type: "knowledge_doc",
            source_id: "knowledge-doc:interface-contract",
            source_path: knowledgePath,
            source_title: "接口契约资料",
            anchor: "候选锚点",
            captured_at: "2026-06-05T01:00:01Z",
            authority_level: "knowledge_material",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "project_director",
        generated_from: "knowledge_summary",
        status: "candidate_needs_review",
        risk_level: "low",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "从明确知识库资料提出候选。",
        conflicts: [],
        audit_refs: [],
        adoption: null,
        created_at: "2026-06-05T01:00:01Z",
        updated_at: "2026-06-05T01:00:01Z",
      },
    ],
    events: [],
    updated_at: "2026-06-05T01:00:01Z",
  };
  const knowledgeWorkflowState: WorkflowStateSnapshot = {
    ...workflowStateWithDerivedWorkflow,
    project_workflows: [
      {
        ...workflowStateWithDerivedWorkflow.project_workflows[0],
        derived_workflow: {
          ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!,
          task_packages: [
            {
              ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!.task_packages[0],
              available_knowledge_refs: [knowledgePath],
            },
          ],
        },
      },
    ],
  };

  const summary = deriveKnowledgeBaseSummary({
    projects: [projectWithKnowledge],
    workflowState: knowledgeWorkflowState,
    formalMemoryStore,
    memoryCandidateStore,
  });
  assert(summary.source_kind === "frontend_read_model", "知识库读模型必须声明前端只读来源");
  assert(summary.documents.length === 1, "知识库读模型应包含 authority file 文档");
  assert(summary.documents[0].formal_memory_links.length === 1, "知识库文档应反向链接正式记忆");
  assert(summary.documents[0].candidate_links.length === 1, "知识库文档应反向链接候选记忆");
  assert(summary.documents[0].task_reference_summary.reference_count === 1, "知识库文档应统计任务包知识引用");
  assert(summary.documents[0].candidate_draft.input.source_refs[0].source_type === "knowledge_doc", "候选草案来源必须是 knowledge_doc");
  assert(summary.documents[0].candidate_draft.input.generated_from === "knowledge_summary", "知识资料候选必须走 knowledge_summary 来源类型");
  assert(summary.obsidian_boundary.native_sync_status === "未执行 Obsidian 原生同步", "M8 只能显示 Obsidian-compatible 占位");

  capturedAction = null;
  const knowledgeView = (
    <KnowledgeBaseView
      projects={[projectWithKnowledge]}
      workflowState={knowledgeWorkflowState}
      formalMemoryStore={formalMemoryStore}
      memoryCandidateStore={memoryCandidateStore}
      hasRealSnapshot
      onRequestAction={captureAction}
    />
  );
  const knowledgeText = visibleText(knowledgeView);
  for (const expectedText of [
    "知识库资料",
    "Obsidian-compatible 占位",
    "未执行 Obsidian 原生同步",
    "知识库是材料和笔记空间",
    "正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文",
    "接口契约资料",
    "关联正式记忆 1",
    "关联候选 1",
    "任务包知识引用 1",
    "正式记忆引用了该知识库来源",
    "提出记忆候选",
    "只生成候选，不写正式记忆",
  ]) {
    assert(knowledgeText.includes(expectedText), `知识库 UI 缺少 ${expectedText}`);
  }
  for (const forbiddenText of [
    "已接入 Obsidian 原生同步",
    "vault 已自动扫描",
    "知识库已自动记住",
    "文档已成为正式记忆",
    "知识命中已成为正式记忆",
    "知识命中已注入任务包",
    "中间版本记忆层已完成",
  ]) {
    assert(!knowledgeText.includes(forbiddenText), `知识库 UI 不应出现越界文案：${forbiddenText}`);
  }

  const candidateButton = findElement(
    knowledgeView,
    (element) => element.type === "button" && visibleText(element).includes("提出记忆候选"),
  );
  assert(candidateButton, "知识库 UI 缺少提出记忆候选按钮");
  const clickCandidate = candidateButton.props?.onClick;
  assert(typeof clickCandidate === "function", "提出记忆候选按钮缺少 onClick");
  clickCandidate();
  const knowledgeCandidateAction = capturedAction as PendingAction | null;
  assert(knowledgeCandidateAction?.kind === "create-memory-candidate", "知识库候选按钮应生成 create-memory-candidate action");
  assert(knowledgeCandidateAction.memoryCandidateCreation?.source_refs[0].source_type === "knowledge_doc", "候选 action 必须保留 knowledge_doc source_ref");
  assert(knowledgeCandidateAction.memoryCandidateCreation?.generated_from === "knowledge_summary", "候选 action 必须使用 knowledge_summary generated_from");
  assert(knowledgeCandidateAction.boundary?.includes("只生成候选，不写正式记忆"), "候选 action 必须说明只生成候选");
  assert(!knowledgeCandidateAction.boundary?.includes("formal-memories.v1.json"), "候选 action 不应声明写正式记忆 store");

  const actionDialogText = visibleText(
    <PermissionDialog action={knowledgeCandidateAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["提出记忆候选", "memory-candidates.v1.json", "只生成候选，不写正式记忆", "knowledge_doc", "接口契约资料"]) {
    assert(actionDialogText.includes(expectedText), `知识库候选确认弹层缺少 ${expectedText}`);
  }
}

function runSecretaryReadModelScenario() {
  const secretarySnapshot: WorkbenchSnapshot = {
    ...snapshot,
    summary: {
      ...snapshot.summary,
      warning_count: 2,
    },
    diagnostics: {
      ...snapshot.diagnostics,
      top_level_warning_count: 1,
      notes: ["offline diagnostic warning"],
    },
  };
  const blackboardCandidateStore = {
    schema_version: "blackboard_candidate_persistence.v1" as const,
    store_version: 1,
    storage_kind: "sidecar_json_v0" as const,
    revision: 4,
    records: [
      {
        candidate_key: "bbcand:v1:offline-pending",
        project_id: "project:offline-fixture-projects-codex-workbench",
        project_root: project.project_root,
        workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
        source_entry_id: "blackboard:offline:risk:001",
        entry_kind: "risk" as const,
        target_kind: "workflow_risk" as const,
        state: "candidate_pending_control_core" as const,
        title_snapshot: "方向风险候选",
        summary_snapshot: "direction_risk_fixture",
        source_refs: [{ source_kind: "blackboard_entry", source_id: "blackboard:offline:risk:001", label: "方向风险候选" }],
        updated_at: "2026-06-03T00:00:00Z",
        warnings: [],
      },
    ],
    audit_events: [],
    updated_at: "2026-06-03T00:00:00Z",
    warnings: [],
  };
  const memoryCandidateStore = {
    store_version: "memory_candidate_store.v1" as const,
    revision: 5,
    candidates: [
      {
        candidate_id: "memcand:offline:secretary:001",
        candidate_key: "memcand:v1:offline-secretary",
        schema_version: "memory_governance.v1" as const,
        scope: {
          scope_id: "scope:project:offline",
          scope_type: "project" as const,
          project_id: "project:offline-fixture-projects-codex-workbench",
          role_ids: [],
          document_refs: [],
          model_export_policy: "local_only" as const,
          valid_from: "2026-06-03T00:00:00Z",
        },
        memory_type: "project_memory" as const,
        claim: "项目需要保留候选治理边界。",
        body: "候选不是正式记忆，必须等待控制核心和用户确认。",
        source_refs: [
          {
            source_ref_id: "source:offline:secretary:001",
            source_type: "stage_report" as const,
            source_id: "stage:offline",
            source_title: "离线秘书测试",
            captured_at: "2026-06-03T00:00:00Z",
            authority_level: "derived_summary" as const,
            sensitive_level: "project" as const,
          },
        ],
        generated_by_role: "secretary",
        generated_from: "secretary_suggestion" as const,
        status: "candidate_needs_review" as const,
        risk_level: "medium" as const,
        sensitive_level: "project" as const,
        requires_user_confirmation: true,
        review_reason: "离线秘书只读模型测试",
        conflicts: [],
        audit_refs: [],
        created_at: "2026-06-03T00:00:00Z",
        updated_at: "2026-06-03T00:00:00Z",
      },
    ],
    events: [],
    updated_at: "2026-06-03T00:00:00Z",
  };
  const context = deriveSecretaryContext({
    snapshot: secretarySnapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    blackboardCandidateStore,
    memoryCandidateStore,
    workflowStateError: "离线工作流状态错误",
  });

  assert(context.source_kind === "derived_read_model", "秘书上下文必须声明 derived_read_model");
  assert(context.warnings.includes("secretary_context_is_read_only"), "秘书上下文必须声明只读边界");
  assert(context.risk_signals.some((risk) => risk.kind === "workflow_state_error"), "秘书风险缺少 workflowStateError");
  assert(context.risk_signals.some((risk) => risk.kind === "diagnostic_warning"), "秘书风险缺少 diagnostics warning");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_permission"), "秘书风险缺少待处理权限");
  assert(context.risk_signals.some((risk) => risk.kind === "failed_execution_attempt"), "秘书风险缺少 failed attempt");
  assert(context.risk_signals.some((risk) => risk.kind === "timed_out_execution_attempt"), "秘书风险缺少 timed_out attempt");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_blackboard_candidate"), "秘书风险缺少 pending 黑板候选");
  assert(context.risk_signals.some((risk) => risk.kind === "pending_memory_candidate"), "秘书风险缺少 pending 记忆候选");
  assert(context.risk_signals.some((risk) => risk.kind === "adapter_warning"), "秘书风险缺少 adapter warning");
  assert(context.risk_signals.some((risk) => risk.kind === "session_operation_boundary"), "秘书风险缺少会话操作边界提醒");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_permission"), "秘书建议缺少权限确认");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "inspect_failed_workflow"), "秘书建议缺少失败/超时查看");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_candidate"), "秘书建议缺少黑板候选治理");
  assert(context.suggestions.some((suggestion) => suggestion.kind === "review_memory_candidate"), "秘书建议缺少记忆候选审查");
  assert(context.suggestions.every((suggestion) => suggestion.requires_user_confirmation), "秘书建议都必须需要用户确认");
  assert(context.suggestions.every((suggestion) => !suggestion.is_fact_change), "秘书建议不能是事实变更");
  assert(context.action_proposals.every((proposal) => !proposal.executable_now), "秘书 action proposal 不能立即执行");
  assert(!context.action_proposals.some((proposal) => proposal.title.includes("adapter")), "计划中 adapter 不能变成秘书可执行 action proposal");
  assert(context.action_proposals.every((proposal) => proposal.requires_user_confirmation), "秘书 action proposal 必须需要确认");
  assert(context.action_proposals.every((proposal) => proposal.blocked_reason.length > 0), "秘书 action proposal 必须说明阻塞原因");
  assert(context.memory_candidates.some((candidate) => candidate.boundary === "候选不等于工作台已经长期记住。"), "秘书记忆候选必须显示候选边界");

  const secretaryText = visibleText(<SecretaryBrief context={context} />);
  for (const expectedText of ["秘书只读摘要", "需要你确认", "候选，不是正式记忆", "建议，不是事实变更"]) {
    assert(secretaryText.includes(expectedText), `秘书摘要缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["秘书已处理", "秘书已执行", "已记住", "正式事实已写入"]) {
    assert(!secretaryText.includes(forbiddenText), `秘书摘要不应出现越界文案：${forbiddenText}`);
  }
}

function runRightRailSecretarySurfaceScenario() {
  const secretaryContext = deriveSecretaryContext({
    snapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    blackboardCandidateStore: null,
    memoryCandidateStore: null,
    workflowStateError: null,
  });
  const secretaryRailItem = workspaceRailItems.find((item) => item.key === "secretary");
  assert(secretaryRailItem, "右侧竖栏缺少秘书独立入口");
  assert(secretaryRailItem.label === "秘书", "秘书入口 label 应保持为独立秘书入口");

  const commonProps = {
    snapshot,
    workflowState: workflowStateWithDerivedWorkflow,
    notice: "offline notice",
    error: false,
    workflowStateError: null,
    secretaryContext,
    onClose: () => {},
    onNavigate: () => {},
    onReloadWorkflowState: () => {},
  };
  const panelSummaryTitles = {
    notifications: "通知摘要",
    todos: "待处理事项",
    audit: "管理摘要",
    running: "运行中摘要",
  };
  for (const activePanel of ["notifications", "todos", "audit", "running"] as const) {
    const panelText = visibleText(<RightDetailPanel activePanel={activePanel} {...commonProps} />);
    assert(panelText.includes(panelSummaryTitles[activePanel]), `${activePanel} 详情应保留自己的职责摘要列表`);
    assert(!panelText.includes("动态"), `${activePanel} 详情不应再使用泛化动态标题`);
    assert(!panelText.includes("秘书只读摘要"), `${activePanel} 详情不应渲染秘书只读摘要`);
    assert(!panelText.includes("建议，不是事实变更"), `${activePanel} 详情不应渲染秘书边界文案`);
    assert(!panelText.includes("候选，不是正式记忆"), `${activePanel} 详情不应渲染秘书记忆边界`);
  }

  const secretaryPanel = <RightDetailPanel activePanel="secretary" {...commonProps} />;
  const secretaryText = visibleText(secretaryPanel);
  for (const expectedText of ["秘书只读摘要", "建议，不是事实变更", "候选，不是正式记忆", "秘书模型只读"]) {
    assert(secretaryText.includes(expectedText), `秘书独立入口缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["动态", "确认执行", "重新读取事实层"]) {
    assert(!secretaryText.includes(forbiddenText), `秘书独立入口不应出现写入或其他中心操作：${forbiddenText}`);
  }
  const secretaryActionButton = findElement(
    secretaryPanel,
    (element) => element.type === "button" && visibleText(element).trim() !== "×",
  );
  assert(!secretaryActionButton, "秘书独立入口除关闭按钮外不应出现任何操作按钮");
}

function runTranscriptCleaningScenario() {
  // A codex rollout carries the same turns twice: the clean event_msg stream and
  // the raw response_item stream that injects the system prompt / environment
  // context as a fake user turn. conversationTurns must keep only event_msg.
  const events = [
    {
      event_id: "e1",
      event_type: "user_message",
      text: "<environment_context>cwd=/x system prompt 注入</environment_context>",
      metadata: { raw_type: "response_item" },
      warnings: [],
    },
    {
      event_id: "e2",
      event_type: "user_message",
      text: "帮我修复登录 bug",
      metadata: { raw_type: "event_msg" },
      warnings: [],
    },
    {
      event_id: "e3",
      event_type: "assistant_message",
      text: "好的，我先看一下代码",
      metadata: { raw_type: "event_msg" },
      warnings: [],
    },
    {
      event_id: "e4",
      event_type: "assistant_message",
      text: "好的，我先看一下代码",
      metadata: { raw_type: "response_item" },
      warnings: [],
    },
    {
      event_id: "e5",
      event_type: "tool_call",
      text: "",
      metadata: { raw_type: "response_item" },
      warnings: [],
    },
    {
      event_id: "e6",
      event_type: "assistant_message",
      text: "   ",
      metadata: { raw_type: "event_msg" },
      warnings: [],
    },
  ] as unknown as Parameters<typeof conversationTurns>[0];

  const turns = conversationTurns(events);
  const ids = turns.map((event) => event.event_id);
  assertDeepEqual(ids, ["e2", "e3"], "对话清洗应只保留 event_msg 的非空人/Agent消息");
  assert(
    !turns.some((event) => (event.text ?? "").includes("environment_context")),
    "对话清洗不应带出系统提示词/环境上下文注入",
  );

  const mixedStream = [
    { event_id: "m1", event_type: "user_message", text: "用户消息在 event_msg", metadata: { raw_type: "event_msg" }, warnings: [] },
    { event_id: "m2", event_type: "assistant_message", text: "Agent 回复在 response_item", metadata: { raw_type: "response_item" }, warnings: [] },
    { event_id: "m3", event_type: "tool_call", text: "tool", metadata: { raw_type: "response_item" }, warnings: [] },
  ] as unknown as Parameters<typeof conversationTurns>[0];
  assertDeepEqual(
    conversationTurns(mixedStream).map((event) => event.event_id),
    ["m1", "m2"],
    "event_msg 不完整时应补 response_item 中缺失的人/Agent轮次",
  );

  // Fallback: a rollout with no event_msg stream still shows its response_item turns.
  const onlyResponseItems = [
    { event_id: "r1", event_type: "user_message", text: "只有 response_item 的会话", metadata: { raw_type: "response_item" }, warnings: [] },
    { event_id: "r2", event_type: "assistant_message", text: "回复", metadata: { raw_type: "response_item" }, warnings: [] },
  ] as unknown as Parameters<typeof conversationTurns>[0];
  assertDeepEqual(
    conversationTurns(onlyResponseItems).map((event) => event.event_id),
    ["r1", "r2"],
    "没有 event_msg 流时应回退到 response_item 对话",
  );

  const noisyFallback = [
    { event_id: "n1", event_type: "user_message", text: "<environment_context>cwd=/tmp</environment_context>", metadata: { raw_type: "response_item" }, warnings: [] },
    { event_id: "n2", event_type: "assistant_message", text: "thinking hidden", metadata: { raw_type: "response_item", payload_type: "reasoning" }, warnings: [] },
    { event_id: "n3", event_type: "tool_call", text: "tool", metadata: { raw_type: "response_item" }, warnings: [] },
    { event_id: "n4", event_type: "user_message", text: "真实旧会话用户消息", metadata: { raw_type: "response_item" }, warnings: [] },
    { event_id: "n5", event_type: "assistant_message", text: "真实旧会话回复", metadata: { raw_type: "response_item" }, warnings: [] },
  ] as unknown as Parameters<typeof conversationTurns>[0];
  assertDeepEqual(
    conversationTurns(noisyFallback).map((event) => event.event_id),
    ["n4", "n5"],
    "response_item 回退也应过滤 thinking、system 注入和工具事件",
  );
}

function runSessionCenterHardeningScenario() {
  const missingSession: SessionRecord = {
    ...session,
    thread_id: "offline-thread-missing",
    title: "Missing rollout fixture",
    rollout_exists: false,
    rollout_path: null,
    warnings: ["rollout_missing_on_disk"],
  };
  const archivedSession: SessionRecord = {
    ...session,
    thread_id: "offline-thread-archived",
    title: "Archived fixture",
    archived: true,
    rollout_path: "/offline-fixture/rollouts/offline-thread-archived.jsonl",
  };
  const sessions = [session, otherProjectSession, missingSession, archivedSession];

  assertDeepEqual(
    filterAgentSessions(sessions, "readable", "Offline interaction").map((item) => item.thread_id),
    [session.thread_id],
    "搜索标题应缩小到匹配会话",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "all", "other-project").map((item) => item.thread_id),
    [otherProjectSession.thread_id],
    "搜索项目路径末段应缩小到匹配项目",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "missing", "").map((item) => item.thread_id),
    [missingSession.thread_id],
    "缺回放记录过滤应只显示缺失会话",
  );
  assertDeepEqual(
    filterAgentSessions(sessions, "archived", "").map((item) => item.thread_id),
    [archivedSession.thread_id],
    "已归档过滤应只显示归档会话",
  );
  const center = (
    <AgentSessionCenter
      sessions={sessions}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError="rollout_outside_allowed_dirs:/tmp/outside.jsonl"
      projectSessionCount={sessions.length}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />
  );
  const centerText = visibleText(center);
  for (const expectedText of ["搜索会话", "可读取", "缺回放记录", "已归档", "路径被安全边界拒绝", "rollout_outside_allowed_dirs"]) {
    assert(centerText.includes(expectedText), `会话中心硬化缺少 ${expectedText}`);
  }
  const centerMarkup = renderToStaticMarkup(center);
  for (const expectedClass of ["agent-session-shell", "agent-session-list", "agent-transcript-panel", "session-state-filter"]) {
    assert(centerMarkup.includes(expectedClass), `会话中心固定布局缺少 class ${expectedClass}`);
  }
  assert(centerMarkup.includes("<button") && centerMarkup.includes("session-card"), "会话卡必须是可键盘聚焦的 button");

  const longMessage = Array.from({ length: 14 }, (_, index) => `line ${index + 1}`).join("\n");
  const transcript = {
    thread_id: session.thread_id,
    rollout_path: "/offline-fixture/rollouts/transcript-hardening.jsonl",
    project_path: project.project_root,
    title: "Transcript hardening",
    created_at_ms: null,
    updated_at_ms: null,
    viewer_boundary: {
      view_kind: "session_history_viewer",
      reads_session_history: true,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "not_execution_readback",
      warnings: ["test_fixture_session_history_is_not_readback"],
    },
    events: [
      ...Array.from({ length: 13 }, (_, index) => ({
        event_id: `old-${index}`,
        event_type: index % 2 === 0 ? "user_message" : "assistant_message",
        actor: index % 2 === 0 ? "user" : "assistant",
        role: index % 2 === 0 ? "user" : "assistant",
        text: `较早消息 ${index}`,
        metadata: { raw_type: "event_msg" },
        warnings: [],
      })),
      {
        event_id: "long",
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: longMessage,
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "code",
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: "```ts\nconst ok = true;\n```",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "tool",
        event_type: "tool_call",
        actor: "assistant",
        text: "should be internal",
        metadata: { raw_type: "response_item" },
        warnings: [],
      },
    ],
    summary: {
      total_events: 16,
      event_type_counts: {},
      unknown_event_count: 0,
      warning_count: 0,
      encrypted_content_event_count: 0,
      sensitive_like_event_count: 0,
    },
    warnings: [],
    source_stats: {},
  };
  const transcriptText = visibleText(<ChatTranscript transcript={transcript} />);
  for (const expectedText of ["已收纳较早 3 条消息", "展开全部", "展开", "开发者详情：过程事件", "复制", "const ok = true;"]) {
    assert(transcriptText.includes(expectedText), `Transcript 展示硬化缺少 ${expectedText}`);
  }
  assert(!transcriptText.includes("should be internal"), "工具事件不应默认进入主对话流");

  const transcriptCenterText = visibleText(
    <AgentSessionCenter
      sessions={sessions}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={sessions.length}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(
    transcriptCenterText.includes("会话来源：只读历史查看，不是执行结果回收。"),
    "transcript viewer 的执行边界说明应收进开发者会话来源详情",
  );

  assert(centerMarkup.includes("session-search") && centerMarkup.includes("session-card"), "会话中心应渲染搜索框和 button 会话卡");
}

function runOfflineRoleOrchestrationScenario() {
  const parsed = parseOfflineDispatchBlock(defaultOfflineDispatchBlock, project.project_root);
  assert(parsed.ok, "默认离线派发块应能解析");
  assert(parsed.proposal.target_role_id === "codex-dev", "开发线应映射到 codex-dev");
  assert(parsed.proposal.task_title === "README 极小修改验证", "默认派发块任务名解析不匹配");
  assert(parsed.proposal.required_return.includes("验证结果"), "默认派发块缺少验证结果回传要求");

  const missing = parseOfflineDispatchBlock("派发给：开发线\n任务名：缺字段测试", project.project_root);
  assert(!missing.ok, "缺字段派发块不应解析成功");
  for (const expectedMissing of ["目标", "执行目录", "允许读取", "允许写入", "禁止事项", "验收标准", "超时", "回传要求"]) {
    assert(missing.missing.includes(expectedMissing), `缺字段派发块没有提示 ${expectedMissing}`);
  }

  const action = buildOfflineRoleDispatchAction(project.project_root, "work-item:offline:001", parsed.proposal);
  assertDeepEqual(
    action,
    {
      kind: "offline-role-dispatch",
      label: "离线派发给开发线",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只把角色派发块写入工作台自己的 workflow-state.v0.json；不启动 Codex、不执行 codex exec resume、不发送消息、不写 /Users/yoyi/.codex、不运行运行器。",
      offlineRoleDispatch: {
        ...parsed.proposal,
        work_item_id: "work-item:offline:001",
      },
    },
    "离线角色派发 action 不匹配",
  );
  const actionDialogText = visibleText(
    <PermissionDialog action={action} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "离线派发给开发线",
    "目标角色",
    "开发线",
    "任务名",
    "README 极小修改验证",
    "必须回传",
    "验证结果",
    "不启动 Codex",
    "不执行 codex exec resume",
    "不写 /Users/yoyi/.codex",
    "工作流状态",
  ]) {
    assert(actionDialogText.includes(expectedText), `离线派发确认弹层缺少 ${expectedText}`);
  }

  const stubResult = buildOfflineStubResult(parsed.proposal);
  assert(stubResult.role_label === "开发线", "桩结果角色不匹配");
  assert(stubResult.summary.includes("没有执行真实 Codex 会话"), "桩结果必须说明没有真实执行");
  assert(stubResult.returned_to_director.includes("请总指导回收"), "桩结果应回传总指导");

  const handoffActions: PendingAction[] = [];
  const rolePanelWithPreparedDispatch = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithPreparedOfflineDispatch.project_workflows[0]}
      sessions={[session]}
      onRequestAction={(action) => {
        handoffActions.push(action);
      }}
    />
  );
  const handoffButton = findElement(
    rolePanelWithPreparedDispatch,
    (element) => element.type === "button" && element.props?.type === "button" && visibleText(element).includes("写入角色回传"),
  );
  assert(handoffButton, "离线角色编排区缺少角色回传按钮");
  const clickHandoff = handoffButton.props?.onClick;
  assert(typeof clickHandoff === "function", "离线角色回传按钮没有 onClick");
  clickHandoff();
  const capturedHandoffAction = handoffActions[0];
  assert(capturedHandoffAction, "角色回传按钮没有捕获 action");
  assert(capturedHandoffAction?.kind === "offline-role-result-handoff", "角色回传按钮应生成离线回传 action");
  assert(
    capturedHandoffAction.offlineRoleResultHandoff?.dispatch_id === "offline-dispatch:fixture:prepared",
    "角色回传应绑定 prepared 离线派发",
  );
  assert(
    capturedHandoffAction.offlineRoleResultHandoff?.summary.includes("已落账离线派发"),
    "角色回传应使用已落账派发块生成摘要",
  );

  const reviewActions: PendingAction[] = [];
  const rolePanelWithCompletedDispatch = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithCompletedOfflineDispatch.project_workflows[0]}
      sessions={[session]}
      onRequestAction={(action) => {
        reviewActions.push(action);
      }}
    />
  );
  const reviewPanelText = visibleText(rolePanelWithCompletedDispatch);
  assert(reviewPanelText.includes("ready_for_review"), "完成回传后应保留 ready_for_review 工作项作为账本锚点");
  assert(reviewPanelText.includes("offline-dispatch:fixture:completed"), "完成回传后应显示 completed 离线派发");
  const reviewButton = findElement(
    rolePanelWithCompletedDispatch,
    (element) => element.type === "button" && element.props?.type === "button" && visibleText(element).includes("写入总指导回收"),
  );
  assert(reviewButton, "离线角色编排区缺少总指导回收按钮");
  assert(reviewButton.props?.disabled !== true, "完成回传后总指导回收按钮不应禁用");
  const clickReview = reviewButton.props?.onClick;
  assert(typeof clickReview === "function", "离线总指导回收按钮没有 onClick");
  clickReview();
  const capturedReviewAction = reviewActions[0];
  assert(capturedReviewAction?.kind === "offline-director-review", "总指导回收按钮应生成离线回收 action");
  assert(
    capturedReviewAction.offlineDirectorReview?.dispatch_id === "offline-dispatch:fixture:completed",
    "总指导回收应绑定 completed 离线派发",
  );

  capturedAction = null;
  const rolePanel = (
    <OfflineRoleOrchestrationPanel
      project={project}
      projectWorkflow={workflowStateWithProjectWorkflow.project_workflows[0]}
      sessions={[session]}
      onRequestAction={captureAction}
    />
  );
  const rolePanelText = visibleText(rolePanel);
  for (const expectedText of [
    "Codex 角色编排",
    "总指导派发闭环",
    "总指导",
    "开发线",
    "验证线",
    "回收线",
    "总指导回复里的派发块",
    "写入离线派发",
    "写入角色回传",
    "写入总指导回收",
    "账本锚点",
    "已有任务草稿",
    "派发预览",
    "角色回传",
    "回传总指导",
    "不启动 Codex",
    "不写 /Users/yoyi/.codex",
    "离线编排账本",
    "预览来自默认示例",
  ]) {
    assert(rolePanelText.includes(expectedText), `离线角色编排区缺少 ${expectedText}`);
  }

  const previewOnlyPanelText = visibleText(
    <OfflineRoleOrchestrationPanel project={project} sessions={[session]} onRequestAction={captureAction} />,
  );
  assert(previewOnlyPanelText.includes("离线编排只能预览"), "没有 ready_to_dispatch 工作项时应只允许预览");

  const offlineForm = findElement(rolePanel, (element) => element.type === "form" && element.props?.className === "offline-role-orchestration-panel");
  assert(offlineForm, "离线角色编排区缺少表单");
  const submitOfflineDispatch = offlineForm.props?.onSubmit;
  assert(typeof submitOfflineDispatch === "function", "离线角色派发表单没有 onSubmit");
  const originalFormData = globalThis.FormData;
  globalThis.FormData = class {
    get(name: string) {
      if (name === "dispatch-block") return defaultOfflineDispatchBlock;
      if (name === "director-request") return "请总指导拆给开发线。";
      return null;
    }
  } as unknown as typeof FormData;
  try {
    submitOfflineDispatch({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assertDeepEqual(capturedAction, action, "离线角色派发表单提交 action 不匹配");

  capturedAction = null;
  globalThis.FormData = class {
    get(name: string) {
      if (name === "dispatch-block") return "派发给：开发线\n任务名：缺字段测试";
      return null;
    }
  } as unknown as typeof FormData;
  try {
    submitOfflineDispatch({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assert(capturedAction === null, "缺字段派发块不应生成离线派发 action");
}

function runShellScenario() {
  const visited: string[] = [];
  const home = <HomeView snapshot={snapshot} onNavigate={(view) => visited.push(view)} />;
  const homeText = visibleText(home);
  for (const expectedText of ["项目", "智能体", "Skill", "Harness", "运行中工作流", "不是真实使用事件"]) {
    assert(homeText.includes(expectedText), `首页缺少 ${expectedText}`);
  }
  assertDeepEqual(
    primaryNavItems.map((item) => item.label),
    ["项目", "智能体", "想法箱", "知识库", "记忆层", "Skill", "Harness", "运行中工作流"],
    "普通主导航应暴露产品级工作对象和素材/记忆入口",
  );
  assertDeepEqual(
    primaryNavItems.map((item) => [item.key, item.glyph]),
    [
      ["projects", "▤"],
      ["agents", "◍"],
      ["ideas", "✎"],
      ["knowledge", "▢"],
      ["memory", "◐"],
      ["skills", "✦"],
      ["harness", "⬡"],
      ["runningWorkflows", "≋"],
    ],
    "左侧主导航应沿用 inkwash-full.html 的水墨 rail 图标语言",
  );
  for (const internalLabel of devNavItems.map((item) => item.label)) {
    assert(!primaryNavItems.some((item) => item.label === internalLabel), `普通主导航不应暴露开发者入口：${internalLabel}`);
  }
  for (const forbiddenText of ["系统", "Skills 1", "Plugins 1"]) {
    assert(!homeText.includes(forbiddenText), `首页不应显示数量：${forbiddenText}`);
  }

  const settingsText = visibleText(
    <SettingsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateError={null}
      hasRealSnapshot={true}
      developerItems={devNavItems}
      onNavigate={(view) => visited.push(view)}
    />,
  );
  for (const expectedText of [
    "开发者",
    "建议方案",
    "实验画布",
    "模型/凭据",
    "适配器",
    "供应方",
    "边车文件",
    "原始状态",
    "诊断",
    "不读取凭据",
    "不从设置页触发",
  ]) {
    assert(settingsText.includes(expectedText), `设置开发者区缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["执行 codex", "恢复会话", "密钥值", "令牌值", "读取并展示"]) {
    assert(!settingsText.includes(forbiddenText), `设置开发者区不应出现执行或凭据读取文案：${forbiddenText}`);
  }

  const runningWorkflowsText = visibleText(
    <RunningWorkflowsView
      snapshot={snapshot}
      workflowState={workflowStateWithProjectWorkflow}
      workflowStateLoading={false}
      workflowStateError={null}
      onReloadWorkflowState={() => {}}
      onNavigate={(view) => visited.push(view)}
    />,
  );
  for (const expectedText of ["运行中工作流", "只显示运行、等待、复核、重试和读回异常摘要", "读回异常", "未知 / 不可用不显示成 0 条结果"]) {
    assert(runningWorkflowsText.includes(expectedText), `运行中工作流页缺少 ${expectedText}`);
  }

  const agentButton = findButtonByText(home, "打开智能体");
  assert(agentButton, "首页找不到智能体入口按钮");
  const openAgent = agentButton.props?.onClick;
  assert(typeof openAgent === "function", "智能体入口没有 onClick");
  openAgent({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(visited, ["agents"], "智能体入口导航不匹配");

  const agentText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "Codex 会话中心",
    "会 话 层",
    "当前会话",
    "Offline interaction fixture",
    "重新读取",
    "定位回放记录",
    "OpenClaw",
  ]) {
    assert(agentText.includes(expectedText), `Agent 页缺少 ${expectedText}`);
  }
  assert(!agentText.includes("启动 OpenClaw"), "未接入 agent 不应出现操作能力");

  const agentViewNode = (
    <AgentView
      sessions={[session]}
      projects={[project]}
      adapterDescriptors={snapshot.agent_adapters}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />
  );
  const agentViewText = visibleText(agentViewNode);
  const agentViewMarkup = renderToStaticMarkup(agentViewNode);
  for (const expectedText of [
    "智能体",
    "项目",
    "对话",
    "可以开始对话",
    "新建对话",
    "只生成预览，不直接创建",
    "任务输入",
    "生成发送预览",
    "Offline interaction fixture",
    "offline-model",
    "codex-workbench",
    "开发者详情",
    "适配器能力",
    "Codex",
    "codex-local",
    "会话索引读取",
    "会话正文只读",
    "工作流节点绑定",
    "安全测试派发",
    "用户审核业务派发",
    "四角色工作流机器",
    "权限结论记录",
    "运行器资源索引",
    "未实现适配器清单",
    "openclaw",
    "claude-code",
    "OpenCode-like",
    "计划中",
    "当前不可执行",
    "凭据：未配置",
    "模型：未验证",
    "已实现动作：无",
    "planned_adapter_not_connected",
    "no_execution_button",
    "backend_read_model",
    "adapter_descriptor_is_backend_read_model_only",
    "does_not_change_codex_execution_semantics",
  ]) {
    assert(agentViewText.includes(expectedText), `AgentView 新方向缺少 ${expectedText}`);
  }
  assert(agentViewMarkup.includes("agent-conversation-bar"), "AgentView 新方向应有项目 / 对话选择条");
  assert(agentViewMarkup.includes("agent-chat-composer"), "AgentView 新方向应有任务输入框");
  assert(
    agentViewMarkup.indexOf("agent-session-shell") < agentViewMarkup.indexOf("agent-boundary-details"),
    "AgentView 新方向应先展示对话界面，再展示开发者详情",
  );
  const fallbackAgentViewText = visibleText(
    <AgentView
      sessions={[session]}
      projects={[project]}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
    />,
  );
  assert(fallbackAgentViewText.includes("adapter_descriptor_frontend_fallback_used"), "AgentView 没有后端 descriptor 时应保留前端 fallback");
  assert(!agentViewText.includes("请进入对应项目查看具体会话与正文"), "智能体页不应再把会话正文导回项目页");
  assert(!agentViewText.includes("选 择 智 能 体"), "智能体页不应再强制先选软件层");
  assert(!agentViewText.includes("启动 OpenClaw"), "未实现 OpenClaw 不应出现启动按钮");
  assert(!agentViewText.includes("绑定 Claude"), "未实现 Claude Code 不应出现绑定按钮");
  assert(!agentViewText.includes("凭据已配置"), "未实现 adapter 不能显示凭据已配置");

  const projectText = visibleText(<ProjectDetail project={project} sessions={[session]} onRequestAction={captureAction} />);
  for (const expectedText of ["总览", "工作流", "交接", "资源", "设置", "项目概览", "智能体入口", "会话列表和对话界面已放到智能体页", "缺少项目默认 workflow"]) {
    assert(projectText.includes(expectedText), `项目工作流缺少 ${expectedText}`);
  }
  assert(!projectText.includes("任务包"), "项目工作台主导航不应出现任务包");
  assert(!projectText.includes("Codex 角色编排"), "工作流页不应混入离线角色编排面板");
  assert(!projectText.includes("任务包 Markdown 预览"), "没有 workflow 时任务包字段区不应占据主流程中心");

  const projectAgentButton = findButtonByText(
    <ProjectDetail project={project} sessions={[session]} onRequestAction={captureAction} />,
    "在智能体中打开",
  );
  assert(projectAgentButton, "项目总览缺少智能体会话入口");
  const filteredProjectSessions = filterProjectSessionsForProject([session, otherProjectSession], project);
  assertDeepEqual(filteredProjectSessions, [session], "项目 Agent 会话应只保留 project_root 等于当前项目的会话");

  const projectAgentSessionText = visibleText(
    <AgentSessionCenter
      scope="project"
      eyebrow="项目 Agent"
      title="项目内 Agent 会话"
      description={`只显示 project_root 等于当前项目的 Codex 会话；项目归属来源为索引推断。当前项目：${project.name}`}
      emptyTitle="没有索引推断关联的 Codex 会话"
      emptyMessage="当前项目没有索引推断关联的 Codex 会话。"
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of [
    "项目内 Agent 会话",
    "project_root 等于当前项目",
    "索引推断",
    "Offline interaction fixture",
    "codex-workbench",
    "重新读取",
  ]) {
    assert(projectAgentSessionText.includes(expectedText), `项目 Agent 会话面板缺少 ${expectedText}`);
  }
  assert(!projectAgentSessionText.includes(otherProjectSession.title), "项目 Agent 会话面板不应显示其他项目会话");
  for (const forbiddenText of ["发送消息", "新建会话", "codex resume", "删除会话", "移动会话"]) {
    assert(!projectAgentSessionText.includes(forbiddenText), `项目 Agent 会话面板不应出现危险入口：${forbiddenText}`);
  }

  const emptyProjectAgentSessionText = visibleText(
    <AgentSessionCenter
      scope="project"
      eyebrow="项目 Agent"
      title="项目内 Agent 会话"
      description={`只显示 project_root 等于当前项目的 Codex 会话；项目归属来源为索引推断。当前项目：${emptyProject.name}`}
      emptyTitle="没有索引推断关联的 Codex 会话"
      emptyMessage="当前项目没有索引推断关联的 Codex 会话。"
      sessions={[]}
      selectedThreadId={null}
      selectedSession={null}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={0}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  for (const expectedText of ["没有索引推断关联的 Codex 会话", "当前项目没有索引推断关联的 Codex 会话。"]) {
    assert(emptyProjectAgentSessionText.includes(expectedText), `空项目 Agent 会话面板缺少 ${expectedText}`);
  }

  capturedAction = null;
  const workflowProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowState}
      selectedTool="task-packages"
      onRequestAction={captureAction}
    />
  );
  const workflowProjectText = visibleText(workflowProject);
  for (const expectedText of [
    "项目工作流草稿",
    "当前项目还没有本地工作流草稿",
    "创建默认工作流草稿",
    "请先创建默认工作流草稿，再登记任务包草稿",
    "不会派发给真实 Codex 会话",
  ]) {
    assert(workflowProjectText.includes(expectedText), `项目工作流草稿区缺少 ${expectedText}`);
  }
  const bootstrapButton = findButtonByText(workflowProject, "创建默认工作流草稿");
  assert(bootstrapButton, "项目页缺少创建默认工作流按钮");
  const bootstrap = bootstrapButton.props?.onClick;
  assert(typeof bootstrap === "function", "创建默认工作流按钮没有 onClick");
  bootstrap({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "bootstrap-project-workflow",
      label: "创建项目默认工作流草稿",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "给工作台自己的 workflow-state.v0.json 写入项目、workflow、默认节点、默认边和 audit；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
    },
    "创建默认工作流待确认动作不匹配",
  );
  const bootstrapDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        throw new Error("离线测试不应确认创建默认工作流");
      }}
    />
  );
  const bootstrapDialogText = visibleText(bootstrapDialog);
  for (const expectedText of ["目标路径", project.project_root, "写入边界", "默认节点", "默认边", "不写 Codex 状态库"]) {
    assert(bootstrapDialogText.includes(expectedText), `创建默认工作流确认弹层缺少 ${expectedText}`);
  }
  const cancelBootstrap = findButtonByText(bootstrapDialog, "取消");
  assert(cancelBootstrap, "创建默认工作流确认弹层缺少取消按钮");
  const cancelBootstrapClick = cancelBootstrap.props?.onClick;
  assert(typeof cancelBootstrapClick === "function", "创建默认工作流取消按钮没有 onClick");
  cancelBootstrapClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消创建默认工作流不应保留待确认动作");

  capturedAction = null;
  const workflowProjectWithDraft = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      selectedTool="task-packages"
      onRequestAction={captureAction}
    />
  );
  const workflowProjectWithDraftText = visibleText(workflowProjectWithDraft);
  for (const expectedText of [
    "当前项目已有本地工作流草稿",
    "任务草稿",
    "2 个",
    "创建任务包草稿",
    "Codex 开发线",
    "已有任务草稿",
    "第二个任务草稿",
    "task_package",
    "当前选中",
    "选择",
    "任务包 Markdown 预览",
    "预览，不是已派发任务包",
    "有任务草稿时可以点“预览 Markdown”查看只读文本",
    "编辑字段表单会绑定当前选中的任务草稿",
  ]) {
    assert(workflowProjectWithDraftText.includes(expectedText), `任务草稿区缺少 ${expectedText}`);
  }

  const workflowCanvasWithDraft = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      planAuthorizationStore={planAuthorizationStore}
      projectConsultationProposalStore={projectConsultationProposalStore}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowCanvasWithDraftText = visibleText(workflowCanvasWithDraft);
  for (const expectedText of [
    "项目工作流主入口",
    "waiting_for_permission",
    "6 nodes",
    "1 pending",
    "项目目标",
    "总指导",
    "开发线",
    "验证线",
    "回收线",
    "权限",
    "节点详情",
    "节点状态",
    "当前任务",
    "会话绑定",
    "派发摘要",
    "权限请求",
    "最近审计",
    "当前工作项",
    "负责角色",
    "当前位置",
    "派发位置",
    "Codex 开发线",
    "下一步：标记执行中",
    "节点会话绑定",
    "派发位置已有绑定",
    "Offline interaction fixture",
    "项目归属来源：index_inferred",
    "读取状态：可读取",
    "打开会话",
    "解除绑定",
    "派发指令",
    "旧安全派发已封存",
    "请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
    "旧入口已封存",
    "执行目录：/Users/yoyi",
    "沙箱模式：workspace-write",
    "允许写入根目录：/Users/yoyi/codex-workflow-mario-test",
    "旧业务派发已封存",
    "dispatch:offline:001",
    "事件：12 / 命中：1",
    "可控执行协议",
    "重试",
    "1/0",
    "超时",
    "900 秒",
    "用户审核业务指令",
    "用户审核业务指令夹具",
    "验证协议预览，不执行真实业务任务。",
    "确认指令边界",
    "权限请求队列",
    "待确认 / write_workflow_state",
    "需要用户确认是否允许写协议字段。",
    "项目咨询方案草案",
    "待用户确认；步骤 3 / 风险 1 / 停止条件 1",
    "确认任务包、角色、读写范围、工具和停止条件后，再进入全局复核。",
    "确认方案范围",
    "要求修改",
    "拒绝方案",
    "C2 只记录方案草案和用户决定",
    "方案授权摘要",
    "授权有效；角色 1 / agent 1 / 读 1 / 写 1 / 工具 1 / 检查 0 / 停止条件 1",
    "plan-authorizations.v1.json / rev 4",
    "plan-auth:offline:active",
    "blocked / 写入范围超出方案授权",
    "audit:auto-dispatch-scope-checked:offline",
    "本摘要只读",
    "本轮未执行真实工作者",
    "批准",
    "拒绝",
    "失败 / 重试 / 超时 / 取消",
    "第 1 次 / 失败",
    "离线夹具失败原因。",
    "第 2 次 / 已超时",
    "离线夹具超时。",
    "最近审计事件",
    "task_draft_created",
  ]) {
    assert(workflowCanvasWithDraftText.includes(expectedText), `工作流画布缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["组件状态样例", "后续画布开发基准", "空画布", "四角色", "工作者已执行", "工作者已启动", "已自动执行"]) {
    assert(!workflowCanvasWithDraftText.includes(forbiddenText), `项目工作流页不应显示开发样例文案：${forbiddenText}`);
  }
  const proposalAction = buildProjectConsultationProposalDecisionAction({
    project,
    proposal: projectConsultationProposalStore.proposals[0],
    decision: "confirm",
    summary: "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。",
    proposalStoreRevision: projectConsultationProposalStore.revision,
    planAuthorizationRevision: planAuthorizationStore.revision,
  });
  assert(proposalAction.kind === "record-project-consultation-proposal-decision", "确认方案范围应生成 proposal decision action");
  assert(
    proposalAction.projectConsultationProposalDecision?.decision === "confirm",
    "确认方案范围 action 应记录 confirm 决定",
  );
  assertDeepEqual(
    proposalAction.projectConsultationProposalDecision,
    {
      project_root: project.project_root,
      proposal_id: "proposal:offline:c2:pending",
      actor_id: "user",
      decision: "confirm",
      summary: "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。",
      expected_proposal_store_revision: 1,
      expected_plan_authorization_store_revision: 4,
    },
    "确认方案范围 action payload 不匹配",
  );
  const proposalDialogText = visibleText(
    <PermissionDialog action={proposalAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "确认方案范围",
    "目标摘要",
    "允许读取",
    project.project_root,
    "允许写入",
    "/offline-fixture/projects/codex-workbench/src",
    "工具 / 检查",
    "read_file / npm run typecheck",
    "停止条件",
    "超出读写范围或需要权限升级时必须停下。",
    "待全局复核",
    "不会启动真实工作者",
    "不写 /Users/yoyi/.codex",
  ]) {
    assert(proposalDialogText.includes(expectedText), `项目咨询方案确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const workflowCanvasWithConfirmedProposal = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      planAuthorizationStore={planAuthorizationStorePendingGlobal}
      projectConsultationProposalStore={projectConsultationProposalStoreConfirmed}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowCanvasWithConfirmedProposalText = visibleText(workflowCanvasWithConfirmedProposal);
  for (const expectedText of [
    "全局边界复核",
    "方案已由用户确认；等待全局主管复核",
    "待全局复核",
    "plan-auth:offline:pending-global",
    "批准并生效",
    "要求修改",
    "阻断方案",
    "C3 只记录全局边界复核和授权状态；不会启动工作者",
  ]) {
    assert(workflowCanvasWithConfirmedProposalText.includes(expectedText), `全局边界复核卡片缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["工作者已执行", "自动派发已开始"]) {
    assert(!workflowCanvasWithConfirmedProposalText.includes(forbiddenText), `全局边界复核卡片不应显示 ${forbiddenText}`);
  }
  const globalReviewAction = buildGlobalBoundaryReviewAction({
    project,
    proposal: projectConsultationProposalStoreConfirmed.proposals[0],
    authorization: planAuthorizationStorePendingGlobal.authorizations[0],
    reviewStatus: "approved",
    summary: "全局主管复核通过方案边界；授权有效，仍未派发工作者。",
    authorizationRevision: planAuthorizationStorePendingGlobal.revision,
  });
  assert(globalReviewAction.kind === "record-global-boundary-review", "批准并生效应生成全局边界复核 action");
  assertDeepEqual(
    globalReviewAction.globalBoundaryReview,
    {
      project_root: project.project_root,
      project_id: "project:offline-fixture-projects-codex-workbench",
      workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
      proposal_id: "proposal:offline:c3:confirmed",
      authorization_id: "plan-auth:offline:pending-global",
      actor_id: "global_director",
      review_status: "approved",
      summary: "全局主管复核通过方案边界；授权有效，仍未派发工作者。",
      checklist: {
        architecture_boundary_checked: true,
        cross_project_impact_checked: true,
        permission_scope_checked: true,
        read_write_scope_checked: true,
        tool_and_check_scope_checked: true,
        memory_boundary_checked: true,
        stop_conditions_checked: true,
        acceptance_criteria_checked: true,
      },
      findings: [],
      expected_authorization_revision: 5,
    },
    "批准并生效 action payload 不匹配",
  );
  const globalReviewDialogText = visibleText(
    <PermissionDialog action={globalReviewAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "批准并生效",
    "复核结论",
    "复核摘要",
    "授权对象",
    "方案标题",
    "目标摘要",
    "读写范围",
    "工具 / 检查",
    "停止条件",
    "无阻断发现",
    "只让授权有效",
    "仍未派发工作者",
    "不写 /Users/yoyi/.codex",
  ]) {
    assert(globalReviewDialogText.includes(expectedText), `全局边界复核确认弹层缺少 ${expectedText}`);
  }

  const projectDirectorTaskPlanRequest = {
    project_root: project.project_root,
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    proposal_id: "proposal:offline:c4:confirmed",
    authorization_id: "plan-auth:offline:active",
    actor_id: "project_director",
    expected_authorization_revision: planAuthorizationStore.revision,
  };
  capturedAction = null;
  const projectDirectorTaskPlanCard = (
    <ProjectDirectorTaskPlanCard
      project={project}
      request={projectDirectorTaskPlanRequest}
      plan={projectDirectorTaskPlan}
      loading={false}
      error={null}
      workflowRevision={workflowStateWithProjectWorkflow.workflow_version ?? null}
      onPreview={() => {}}
      onRequestAction={captureAction}
    />
  );
  const projectDirectorTaskPlanCardText = visibleText(projectDirectorTaskPlanCard);
  for (const expectedText of [
    "项目主管拆任务",
    "授权范围内可准备",
    "planned",
    "prepared",
    "needs_binding",
    "阻断",
    "任务包记忆快照",
    "C4 准备态子任务",
    "授权检查通过",
    "生成拆任务草案",
    "准备授权范围内派发",
    "只创建准备记录",
    "不启动工作者",
    "不执行 codex exec resume",
    "不写 /Users/yoyi/.codex",
  ]) {
    assert(projectDirectorTaskPlanCardText.includes(expectedText), `C4 项目主管卡片缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["工作者已执行", "自动派发已开始", "Codex 已收到任务"]) {
    assert(!projectDirectorTaskPlanCardText.includes(forbiddenText), `C4 项目主管卡片不应出现 ${forbiddenText}`);
  }
  const prepareAuthorizedButton = findButtonByText(projectDirectorTaskPlanCard, "准备授权范围内派发");
  assert(prepareAuthorizedButton, "C4 项目主管卡片缺少准备派发按钮");
  const prepareAuthorizedClick = prepareAuthorizedButton.props?.onClick;
  assert(typeof prepareAuthorizedClick === "function", "C4 准备派发按钮没有 onClick");
  prepareAuthorizedClick({ preventDefault() {}, stopPropagation() {} });
  const expectedPrepareAuthorizedAction = buildPrepareAuthorizedAutoDispatchAction({
    project,
    request: projectDirectorTaskPlanRequest,
    plan: projectDirectorTaskPlan,
    workflowRevision: workflowStateWithProjectWorkflow.workflow_version ?? null,
  });
  assertDeepEqual(capturedAction, expectedPrepareAuthorizedAction, "C4 准备派发 action payload 不匹配");
  const prepareAuthorizedDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "准备授权范围内派发",
    "授权对象",
    "plan-auth:offline:active",
    "方案对象",
    "proposal:offline:c4:confirmed",
    "计划摘要",
    "任务计数",
    "planned 1 / prepared 0 / blocked 0 / needs_binding 0",
    "记忆快照",
    "只创建准备记录",
    "不启动工作者",
    "不执行 codex exec resume",
    "不写 /Users/yoyi/.codex",
    "仍未执行工作者",
  ]) {
    assert(prepareAuthorizedDialogText.includes(expectedText), `C4 准备派发确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["工作者已执行", "自动派发已开始", "Codex 已收到任务"]) {
    assert(!prepareAuthorizedDialogText.includes(forbiddenText), `C4 准备派发确认弹层不应出现 ${forbiddenText}`);
  }

  const workflowProjectWithDerived = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateWithDerivedWorkflow}
      selectedTool="workflow"
      onRequestAction={captureAction}
      onInspectWorkflowRunCheck={async () => blockedWorkflowRunCheck}
    />
  );
  const workflowProjectWithDerivedText = visibleText(workflowProjectWithDerived);
  for (const expectedText of [
    "项目工作流主入口",
    "candidates",
    "黑板候选",
    "工作流详情摘要",
    "任务包",
    "账本",
    "子汇报",
    "审查",
    "异常",
    "完成闸门",
    "任务包、账本、状态机、子汇报和黑板候选只在详情侧展示",
    "画布状态原因",
    "attention",
    "用户摘要",
    "项目主管信息",
    "技术详情",
    "为什么停下",
    "谁能处理",
    "下一步",
    "来源引用、审计、证据、交接",
    "需确认弹层",
    "项目工作流画布",
    "工作流状态派生读模型",
    "方案授权 / 控制核心 / 权限 / 审计",
    "实验画布不会写入本项目事实",
    "编辑 / 布局边界",
    "仅视图布局",
    "未保存为事实",
    "React Flow 仅负责渲染",
    "需要生成提案",
    "需要确认弹层",
    "需要控制核心",
    "需要审计",
    "读回摘要",
    "任务记忆包摘要",
    "节点详情",
    "权限请求",
    "candidate_pending_control_core",
    "运行前检查",
    "只阻止运行，不阻止查看草稿",
    "当前运行器",
    "只读展示；不会自动运行运行器",
    "模型",
    "missing: model_id",
    "允许读取",
    project.project_root,
    "允许写入",
    "missing: allowed_write_scope",
    "验收标准",
    "missing: acceptance_criteria",
    "acceptance_criteria_met",
    "derived_from_workflow_state_v0_missing_fields_are_not_guessed",
  ]) {
    assert(workflowProjectWithDerivedText.includes(expectedText), `派生工作流展示缺少 ${expectedText}`);
  }
  const workflowProjectWithDerivedMarkup = renderToStaticMarkup(workflowProjectWithDerived);
  assert(
    !workflowProjectWithDerivedMarkup.includes('class="project-candidate-governance"'),
    "项目工作流主区域不应再把候选治理作为独立 strip",
  );
  assert(
    workflowProjectWithDerivedMarkup.includes("project-candidate-governance-card"),
    "候选治理仍应保留为项目画布侧栏详情卡",
  );
  for (const forbiddenText of [
    "拖拽已保存",
    "连线已保存",
    "节点已删除",
    "已修改 workflow 事实",
    "画布编辑器已完成",
    ...canvasBoundaryForbiddenPhrases,
  ]) {
    assert(!workflowProjectWithDerivedText.includes(forbiddenText), `F3/F4 项目画布不应出现误导文案 ${forbiddenText}`);
  }

  const blockedRunCheckText = visibleText(<WorkflowRunCheckDetails runCheck={blockedWorkflowRunCheck} />);
  for (const expectedText of [
    "缺模型；系统不会自动选择模型。",
    "没有读范围；不能运行。",
    "会写文件但没有写范围；不能运行。",
    "节点没有声明工具；工具白名单为空。",
    "模型",
    "阻断",
    "读取范围",
    "工具白名单",
    "警告",
  ]) {
    assert(blockedRunCheckText.includes(expectedText), `blocked 运行前检查展示缺少 ${expectedText}`);
  }

  const runnableRunCheckText = visibleText(<WorkflowRunCheckDetails runCheck={runnableWorkflowRunCheck} />);
  for (const expectedText of ["模型", "通过", "任务包已显式指定模型。", "记忆引用", "任务包没有声明需要记忆引用。"]) {
    assert(runnableRunCheckText.includes(expectedText), `runnable 运行前检查展示缺少 ${expectedText}`);
  }
  assert(!runnableRunCheckText.includes("自动选择模型"), "runnable 检查不应出现自动补模型文案");

  const projectsViewText = visibleText(
    <ProjectsView
      projects={[project]}
      sessions={[session]}
      workflowState={workflowStateWithProjectWorkflow}
      onRequestAction={captureAction}
      onLoadTranscript={async () => {
        throw new Error("主路径静态渲染不应读取 transcript");
      }}
    />,
  );
  for (const expectedText of [
    "项 目 入 口",
    "方块入口",
    "codex-workbench",
    "最近更新",
    "会话",
    "工作流",
    "文件",
    "警告",
  ]) {
    assert(projectsViewText.includes(expectedText), `ProjectsView 项目入口缺少 ${expectedText}`);
  }
  assert(!projectsViewText.includes("节点会话绑定"), "ProjectsView 默认入口不应直接进入项目工作台");
  assert(!projectsViewText.includes("任务包"), "ProjectsView 默认入口不应把任务包作为主模块展示");
  assert(!projectsViewText.includes("项目工作流草稿"), "ProjectsView 默认工作流页不应混入任务包页内容");

  const selectedAfterMissing = nextSelectedWorkItemId(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, "work-item:missing");
  assert(selectedAfterMissing === "work-item:offline:001", "缺失选择态应回到第一个草稿");
  const selectedAfterSwitch = nextSelectedWorkItemId(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, "work-item:offline:002");
  assert(selectedAfterSwitch === "work-item:offline:002", "切换选择态后应保留第二个草稿");
  const selectedSecondDraft = selectedTaskDraftFor(workflowStateWithProjectWorkflow.project_workflows[0].task_drafts, selectedAfterSwitch);
  assert(selectedSecondDraft?.work_item_id === "work-item:offline:002", "选择态解析应返回第二个草稿");

  const workflowControlCardWithDraft = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateWithDerivedWorkflow.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateWithDerivedWorkflow.project_workflows[0].node_session_bindings}
      dispatches={workflowStateWithDerivedWorkflow.project_workflows[0].node_dispatches}
      directorReviews={workflowStateWithDerivedWorkflow.project_workflows[0].director_reviews}
      executionControls={workflowStateWithDerivedWorkflow.project_workflows[0].execution_controls}
      permissionRequests={workflowStateWithDerivedWorkflow.project_workflows[0].permission_requests}
      executionAttempts={workflowStateWithDerivedWorkflow.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateWithDerivedWorkflow.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateWithDerivedWorkflow.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );

  capturedAction = null;
  const instructionBoundaryButton = findButtonByText(workflowControlCardWithDraft, "确认指令边界");
  assert(instructionBoundaryButton, "可控执行协议区缺少确认指令边界按钮");
  const previewInstruction = instructionBoundaryButton.props?.onClick;
  assert(typeof previewInstruction === "function", "确认指令边界按钮没有 onClick");
  previewInstruction({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "preview-user-reviewed-instruction",
      label: "确认用户审核业务指令边界",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只确认用户审核业务指令的结构化预览和边界；本版本不执行 codex exec resume、不发送 Codex 消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
      userReviewedInstruction: workflowStateWithProjectWorkflow.project_workflows[0].execution_controls[0].user_reviewed_instruction,
    },
    "用户审核业务指令边界动作不匹配",
  );
  const instructionDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "确认用户审核业务指令边界",
    "指令摘要",
    "用户审核业务指令夹具",
    "审核状态",
    "reviewed",
    "不执行真实业务任务",
    "不启动 Codex",
    "不恢复会话",
    "不发送消息",
    "不写 /Users/yoyi/.codex",
  ]) {
    assert(instructionDialogText.includes(expectedText), `用户审核业务指令确认弹层缺少 ${expectedText}`);
  }

  const c5PanelText = visibleText(workflowControlCardWithDraft);
  for (const expectedText of [
    "C5 工作者汇报 / 过程事实",
    "待主管确认",
    "汇报数量",
    "待确认事实",
    "已确认事实",
    "读回",
    "读取成功",
    "权限",
    "等待权限",
    "失败",
    "超时",
    "离线桩结果：已接收任务，没有执行真实 Codex 会话。",
    "方向风险",
    "记录汇报",
    "确认为过程事实",
    "要求返工",
    "阻断并上报",
  ]) {
    assert(c5PanelText.includes(expectedText), `C5 过程事实面板缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["工作者汇报已成为正式事实", "系统已记住", "最终结果已通过", "自动化工作流已完成"]) {
    assert(!c5PanelText.includes(forbiddenText), `C5 面板不应显示 ${forbiddenText}`);
  }

  const workflowC6ControlCard = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateWithC6ResultSummary.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateWithC6ResultSummary.project_workflows[0].node_session_bindings}
      dispatches={workflowStateWithC6ResultSummary.project_workflows[0].node_dispatches}
      directorReviews={workflowStateWithC6ResultSummary.project_workflows[0].director_reviews}
      executionControls={workflowStateWithC6ResultSummary.project_workflows[0].execution_controls}
      permissionRequests={workflowStateWithC6ResultSummary.project_workflows[0].permission_requests}
      executionAttempts={workflowStateWithC6ResultSummary.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateWithC6ResultSummary.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateWithC6ResultSummary.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateWithC6ResultSummary.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );
  const c6PanelText = visibleText(workflowC6ControlCard);
  for (const expectedText of [
    "C6 结果 / 阶段验收",
    "阶段 C 验收门禁已通过",
    "最终复核",
    "最终复核通过",
    "用户决定",
    "用户已接受",
    "阶段门禁",
    "过程事实",
    "全局主管已完成最终复核",
    "用户已查看结果并作出决定",
    "阶段 C 验收门禁已通过",
    "仍不代表中间版本整体完成",
    "真实工作者 / Codex 执行仍需单独授权任务包。",
    "记录最终复核通过",
    "记录用户接受",
    "生成验收摘要",
  ]) {
    assert(c6PanelText.includes(expectedText), `C6 结果摘要面板缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["中间版本已完成", "完整记忆系统已完成", "工作者汇报已成为正式事实", "系统已记住", "真实工作者已执行"]) {
    assert(!c6PanelText.includes(forbiddenText), `C6 面板不应显示越界文案：${forbiddenText}`);
  }

  capturedAction = null;
  const globalFinalReviewButton = findButtonByText(workflowC6ControlCard, "记录最终复核通过");
  assert(globalFinalReviewButton, "C6 面板缺少全局最终复核按钮");
  const recordGlobalFinalReview = globalFinalReviewButton.props?.onClick;
  assert(typeof recordGlobalFinalReview === "function", "C6 全局最终复核按钮没有 onClick");
  recordGlobalFinalReview({ preventDefault() {}, stopPropagation() {} });
  const globalFinalReviewAction = capturedAction as unknown as PendingAction;
  assert(globalFinalReviewAction.kind === "record-global-final-result-review", "C6 全局最终复核 action kind 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.actor_role === "global_director", "C6 全局最终复核 actor_role 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.decision === "accepted", "C6 全局最终复核 decision 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.proposal_id === "proposal:offline:001", "C6 全局最终复核 proposal 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.authorization_id === "plan-auth:offline:active", "C6 全局最终复核 authorization 不匹配");
  assert(globalFinalReviewAction.globalFinalResultReview?.accepted_process_fact_ids.includes("process-fact:offline:001"), "C6 全局最终复核缺少 process fact");
  assert(globalFinalReviewAction.boundary?.includes("不代表用户已接受"), "C6 全局最终复核 action 边界缺少用户接受限制");
  const globalFinalReviewDialogText = visibleText(
    <PermissionDialog action={globalFinalReviewAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["记录全局最终复核", "复核结论", "最终复核通过", "过程事实", "不代表用户已接受", "不写正式记忆", "不代表中间版本整体完成"]) {
    assert(globalFinalReviewDialogText.includes(expectedText), `C6 全局最终复核确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const userDecisionButton = findButtonByText(workflowC6ControlCard, "记录用户接受");
  assert(userDecisionButton, "C6 面板缺少用户结果决定按钮");
  const recordUserDecision = userDecisionButton.props?.onClick;
  assert(typeof recordUserDecision === "function", "C6 用户结果决定按钮没有 onClick");
  recordUserDecision({ preventDefault() {}, stopPropagation() {} });
  const userDecisionAction = capturedAction as unknown as PendingAction;
  assert(userDecisionAction.kind === "record-user-result-decision", "C6 用户结果决定 action kind 不匹配");
  assert(userDecisionAction.userResultDecision?.actor_role === "user", "C6 用户结果决定 actor_role 不匹配");
  assert(userDecisionAction.userResultDecision?.decision === "accept_result", "C6 用户结果决定 decision 不匹配");
  assert(userDecisionAction.userResultDecision?.accepted_review_id === "global-final-review:offline:001", "C6 用户结果决定 review id 不匹配");
  assert(userDecisionAction.boundary?.includes("不代表未来任务默认接受"), "C6 用户结果决定 action 边界缺少未来任务限制");
  const userDecisionDialogText = visibleText(
    <PermissionDialog action={userDecisionAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["记录用户结果决定", "用户决定", "用户已接受", "关联复核", "只记录本次结果决定", "不代表未来任务默认接受", "不写正式记忆"]) {
    assert(userDecisionDialogText.includes(expectedText), `C6 用户结果决定确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const stageSummaryButton = findButtonByText(workflowC6ControlCard, "生成验收摘要");
  assert(stageSummaryButton, "C6 面板缺少阶段 C 验收摘要按钮");
  const generateStageSummary = stageSummaryButton.props?.onClick;
  assert(typeof generateStageSummary === "function", "C6 阶段 C 验收摘要按钮没有 onClick");
  generateStageSummary({ preventDefault() {}, stopPropagation() {} });
  const stageSummaryAction = capturedAction as unknown as PendingAction;
  assert(stageSummaryAction.kind === "generate-stage-c-acceptance-summary", "C6 阶段 C 验收摘要 action kind 不匹配");
  assert(stageSummaryAction.stageCAcceptanceSummary?.project_id === "project:offline-fixture-projects-codex-workbench", "C6 阶段 C 验收摘要 project_id 不匹配");
  assert(stageSummaryAction.stageCAcceptanceSummary?.workflow_id === "workflow:offline-fixture-projects-codex-workbench:default", "C6 阶段 C 验收摘要 workflow_id 不匹配");
  assert(stageSummaryAction.boundary?.includes("不执行真实 Codex"), "C6 阶段 C 验收摘要 action 边界缺少真实 Codex 限制");
  const stageSummaryDialogText = visibleText(
    <PermissionDialog action={stageSummaryAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["生成阶段 C 验收摘要", "产物", "审计事件", "生成门禁摘要和后置项", "不执行真实工作者", "不写正式记忆", "不代表中间版本整体完成"]) {
    assert(stageSummaryDialogText.includes(expectedText), `C6 阶段 C 验收摘要确认弹层缺少 ${expectedText}`);
  }
  for (const forbiddenText of ["中间版本已完成", "完整记忆系统已完成", "工作者汇报已成为正式事实", "系统已记住", "真实工作者已执行"]) {
    assert(!globalFinalReviewDialogText.includes(forbiddenText), `C6 全局最终复核弹层不应显示越界文案：${forbiddenText}`);
    assert(!userDecisionDialogText.includes(forbiddenText), `C6 用户结果决定弹层不应显示越界文案：${forbiddenText}`);
    assert(!stageSummaryDialogText.includes(forbiddenText), `C6 阶段 C 验收摘要弹层不应显示越界文案：${forbiddenText}`);
  }

  capturedAction = null;
  const recordWorkerReportButton = findButtonByText(workflowControlCardWithDraft, "记录汇报");
  assert(recordWorkerReportButton, "C5 面板缺少记录汇报按钮");
  const recordWorkerReport = recordWorkerReportButton.props?.onClick;
  assert(typeof recordWorkerReport === "function", "C5 记录汇报按钮没有 onClick");
  recordWorkerReport({ preventDefault() {}, stopPropagation() {} });
  const workerReportAction = capturedAction as unknown as PendingAction;
  assert(workerReportAction.kind === "record-worker-structured-report", "C5 记录汇报 action kind 不匹配");
  assert(workerReportAction.workerStructuredReport?.project_id === "project:offline-fixture-projects-codex-workbench", "C5 汇报 project_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.workflow_id === "workflow:offline-fixture-projects-codex-workbench:default", "C5 汇报 workflow_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.dispatch_id === "dispatch:offline:001", "C5 汇报 dispatch_id 不匹配");
  assert(workerReportAction.workerStructuredReport?.evidence_refs[0] === "/tmp/codex-workflow-node-dispatch-v1/offline-last-message.txt", "C5 汇报 evidence_refs 不匹配");
  assert(workerReportAction.workerStructuredReport?.source_refs[0].source_kind === "workflow_event", "C5 汇报 source kind 不匹配");
  assert(workerReportAction.boundary?.includes("不把汇报写成正式事实或正式记忆"), "C5 汇报 action 边界缺少正式事实 / 正式记忆限制");
  const workerReportDialogText = visibleText(
    <PermissionDialog action={workerReportAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["记录工作者结构化汇报", "汇报摘要", "证据", "只记录工作者汇报", "不把汇报写成正式事实或正式记忆", "不启动 Codex"]) {
    assert(workerReportDialogText.includes(expectedText), `C5 工作者汇报确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const confirmProcessFactButton = findButtonByText(workflowControlCardWithDraft, "确认为过程事实");
  assert(confirmProcessFactButton, "C5 面板缺少确认为过程事实按钮");
  const confirmProcessFact = confirmProcessFactButton.props?.onClick;
  assert(typeof confirmProcessFact === "function", "C5 确认为过程事实按钮没有 onClick");
  confirmProcessFact({ preventDefault() {}, stopPropagation() {} });
  const processFactAction = capturedAction as unknown as PendingAction;
  assert(processFactAction.kind === "record-project-director-process-fact-decision", "C5 过程事实 action kind 不匹配");
  assert(processFactAction.processFactDecision?.actor_role === "project_director", "C5 过程事实确认必须由项目主管发起");
  assert(processFactAction.processFactDecision?.decision === "confirm_process_fact", "C5 过程事实 decision 不匹配");
  assert(processFactAction.processFactDecision?.accepted_facts[0].proposed_observation_type === "process_fact", "C5 过程事实 observation type 不匹配");
  assert(processFactAction.processFactDecision?.accepted_facts[0].scope.project_id === "project:offline-fixture-projects-codex-workbench", "C5 过程事实 scope project_id 不匹配");
  assert(processFactAction.processFactDecision?.expected_observation_store_revision === 0, "C5 过程事实 observation revision 不匹配");
  assert(processFactAction.boundary?.includes("不写正式记忆"), "C5 过程事实 action 边界缺少正式记忆限制");
  const processFactDialogText = visibleText(
    <PermissionDialog action={processFactAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["确认为过程事实", "确认事实", "确认后只记录过程事实观察", "不写正式记忆", "不完成最终验收"]) {
    assert(processFactDialogText.includes(expectedText), `C5 过程事实确认弹层缺少 ${expectedText}`);
  }

  capturedAction = null;
  const approvePermissionButton = findButtonByText(workflowControlCardWithDraft, "批准");
  assert(approvePermissionButton, "权限队列缺少批准按钮");
  const approvePermission = approvePermissionButton.props?.onClick;
  assert(typeof approvePermission === "function", "权限批准按钮没有 onClick");
  approvePermission({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "record-permission-decision",
      label: "记录权限结论：批准",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只在用户确认后通过控制核心记录权限请求结论并追加审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex。",
      permissionDecision: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
        request_id: "permission:offline:001",
        decision: "approved",
      },
    },
    "权限结论动作不匹配",
  );
  const permissionDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "记录权限结论：批准",
    "权限请求",
    "permission:offline:001",
    "权限结论",
    "批准",
    "控制核心",
    "写入工作台自己的工作流状态",
    "审计事件",
    "不启动 Codex",
    "不恢复会话",
    "不发送消息",
    "不写 /Users/yoyi/.codex",
  ]) {
    assert(permissionDialogText.includes(expectedText), `权限结论确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;

  capturedAction = null;
  const workflowReviewProject = (
    <ProjectDetail
      project={project}
      sessions={[session]}
      workflowState={workflowStateReadyForReview}
      selectedTool="workflow"
      onRequestAction={captureAction}
    />
  );
  const workflowReviewProjectText = visibleText(workflowReviewProject);
  for (const expectedText of [
    "总指导回收",
    "记录派发结果判断",
    "待回收",
    "WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
    "派发：dispatch:offline:001",
    "事件：12",
    "命中：1",
    "警告：session_cwd_differs_from_project_root",
    "接受",
    "需要修改",
    "暂停",
    "废弃",
  ]) {
    assert(workflowReviewProjectText.includes(expectedText), `总指导回收区缺少 ${expectedText}`);
  }
  const workflowReviewControlCard = (
    <WorkItemOrchestrationCard
      project={project}
      projectId={workflowStateReadyForReview.project_workflows[0].project_id}
      sessions={[session]}
      bindings={workflowStateReadyForReview.project_workflows[0].node_session_bindings}
      dispatches={workflowStateReadyForReview.project_workflows[0].node_dispatches}
      directorReviews={workflowStateReadyForReview.project_workflows[0].director_reviews}
      executionControls={workflowStateReadyForReview.project_workflows[0].execution_controls}
      permissionRequests={workflowStateReadyForReview.project_workflows[0].permission_requests}
      executionAttempts={workflowStateReadyForReview.project_workflows[0].execution_attempts}
      derivedWorkflow={workflowStateReadyForReview.project_workflows[0].derived_workflow ?? null}
      projectConsultationProposalSummary={projectConsultationProposalSummary}
      planAuthorizationSummary={planAuthorizationSummary}
      workflowRevision={workflowStateReadyForReview.workflow_version ?? null}
      observationStoreRevision={0}
      workItem={workflowStateReadyForReview.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onOpenAgentSession={() => {}}
    />
  );
  const directorAcceptButton = findButtonByText(workflowReviewControlCard, "接受");
  assert(directorAcceptButton, "总指导回收区缺少接受按钮");
  const requestDirectorAccept = directorAcceptButton.props?.onClick;
  assert(typeof requestDirectorAccept === "function", "总指导回收接受按钮没有 onClick");
  requestDirectorAccept({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "record-director-review",
      label: "记录总指导回收：接受",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只写真实 workflow-state.v0.json 的复核记录和审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
      directorReview: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
        dispatch_id: "dispatch:offline:001",
        decision: "accepted",
        summary: "总指导回收：接受；派发结果：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
      },
    },
    "总指导回收待确认动作不匹配",
  );
  const directorDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "记录总指导回收：接受",
    "派发记录",
    "dispatch:offline:001",
    "回收结论",
    "接受",
    "复核记录",
    "审计事件",
    "不启动 Codex",
    "不恢复会话",
    "不发送消息",
    "不写 /Users/yoyi/.codex",
    "不读取完整会话记录",
  ]) {
    assert(directorDialogText.includes(expectedText), `总指导回收确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;

  const bindCandidate = findButtonContainingText(workflowControlCardWithDraft, ["Offline interaction fixture", "项目归属来源：索引推断"]);
  assert(bindCandidate, "工作流编排区缺少候选会话绑定按钮");
  const bindSession = bindCandidate.props?.onClick;
  assert(typeof bindSession === "function", "候选会话绑定按钮没有 onClick");
  bindSession({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "bind-node-session",
      label: "绑定节点 Codex 会话",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只把已有索引 Codex 会话绑定到工作台自己的 workflow-state.v0.json；不启动 Codex、不发送消息、不恢复会话、不读取完整会话正文、不写 Codex 状态库。",
      nodeSessionBinding: {
        project_root: project.project_root,
        node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:codex-dev",
        work_item_id: "work-item:offline:001",
        thread_id: "offline-thread-001",
      },
    },
    "绑定节点会话待确认动作不匹配",
  );
  const bindDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["绑定节点 Codex 会话", "Codex 会话", "offline-thread-001", "不启动 Codex", "不发送消息", "不读取完整会话正文"]) {
    assert(bindDialogText.includes(expectedText), `绑定节点会话确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const unbindButton = findButtonByText(workflowControlCardWithDraft, "解除绑定");
  assert(unbindButton, "工作流编排区缺少解除绑定按钮");
  const unbind = unbindButton.props?.onClick;
  assert(typeof unbind === "function", "解除绑定按钮没有 onClick");
  unbind({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "unbind-node-session",
      label: "解除节点会话绑定",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只解除工作台自己的 workflow-state.v0.json 绑定并追加审计事件；不删除、不移动、不归档 Codex 原始会话；不写 .codex 或 Codex 状态库。",
      nodeSessionUnbinding: {
        project_root: project.project_root,
        binding_id: "binding:offline:codex-dev",
      },
    },
    "解除节点会话绑定待确认动作不匹配",
  );
  const unbindDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["解除节点会话绑定", "绑定对象", "binding:offline:codex-dev", "不删除", "不移动", "不归档"]) {
    assert(unbindDialogText.includes(expectedText), `解除节点会话确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const dispatchButton = findButtonByText(workflowControlCardWithDraft, "旧安全派发已封存");
  assert(dispatchButton, "工作流编排区缺少旧安全派发封存按钮");
  assert(dispatchButton.props?.disabled, "旧安全派发按钮应保持禁用");
  assert(!dispatchButton.props?.onClick, "旧安全派发按钮不应再触发 pending action");
  const businessDispatchButton = findButtonByText(workflowControlCardWithDraft, "旧业务派发已封存");
  assert(businessDispatchButton, "工作流编排区缺少旧业务派发封存按钮");
  assert(businessDispatchButton.props?.disabled, "旧业务派发按钮应保持禁用");
  assert(!businessDispatchButton.props?.onClick, "旧业务派发按钮不应再触发 pending action");
  assert(capturedAction === null, "封存的旧派发按钮不应生成待确认动作");
  capturedAction = null;
  const advanceButton = findButtonByText(workflowControlCardWithDraft, "标记执行中");
  assert(advanceButton, "工作流编排区缺少推进到执行中按钮");
  const advance = advanceButton.props?.onClick;
  assert(typeof advance === "function", "推进状态按钮没有 onClick");
  advance({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "advance-work-item-state",
      label: "推进工作项到执行中",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只写工作台自己的 workflow-state.v0.json；追加审计事件；不启动 Codex 命令行、不恢复会话、不派发真实 Codex 会话、不运行运行器、不写 .codex 或 Codex 状态库。",
      workItemStateUpdate: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
        next_state: "running",
      },
    },
    "推进工作项状态待确认动作不匹配",
  );
  const advanceDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of [
    "推进工作项到执行中",
    "目标状态",
    "执行中",
    "推进工作台自己的工作项状态并追加审计事件",
    "不启动 Codex 命令行",
    "不恢复会话",
    "不运行运行器",
  ]) {
    assert(advanceDialogText.includes(expectedText), `推进状态确认弹层缺少 ${expectedText}`);
  }
  capturedAction = null;
  const taskDraftForm = findElement(workflowProjectWithDraft, (element) => element.type === "form" && element.props?.className === "task-draft-form");
  assert(taskDraftForm, "任务草稿区缺少创建表单");
  const createTask = taskDraftForm.props?.onSubmit;
  assert(typeof createTask === "function", "创建任务包草稿表单没有 onSubmit");
  const formValues = new Map<string, string>([
    ["task-title", "登记任务包草稿"],
    ["task-objective", "写入 work_items 和 artifacts"],
  ]);
  const originalFormData = globalThis.FormData;
  globalThis.FormData = class {
    get(name: string) {
      return formValues.get(name) ?? null;
    }
  } as unknown as typeof FormData;
  try {
    createTask({
      preventDefault() {},
      currentTarget: {},
    });
  } finally {
    globalThis.FormData = originalFormData;
  }
  assertDeepEqual(
    capturedAction,
    {
      kind: "create-task-draft",
      label: "创建任务包草稿",
      path: project.project_root,
      source: "索引内项目路径",
      boundary: "只登记到工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行。",
      taskDraft: {
        projectRoot: project.project_root,
        title: "登记任务包草稿",
        objective: "写入 work_items 和 artifacts",
        assignedRole: "codex-dev",
      },
    },
    "创建任务包草稿待确认动作不匹配",
  );
  let taskCreateConfirmed = false;
  const taskDraftDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        taskCreateConfirmed = true;
      }}
    />
  );
  const taskDraftDialogText = visibleText(taskDraftDialog);
  for (const expectedText of [
    "不生成真实任务包文件、不派发真实 Codex 会话",
    "任务标题",
    "登记任务包草稿",
    "目标说明",
    "写入 work_items 和 artifacts",
    "默认指派",
    "codex-dev",
  ]) {
    assert(taskDraftDialogText.includes(expectedText), `创建任务包草稿确认弹层缺少 ${expectedText}`);
  }
  const cancelTaskDraft = findButtonByText(taskDraftDialog, "取消");
  assert(cancelTaskDraft, "创建任务包草稿确认弹层缺少取消按钮");
  const cancelTaskDraftClick = cancelTaskDraft.props?.onClick;
  assert(typeof cancelTaskDraftClick === "function", "创建任务包草稿取消按钮没有 onClick");
  cancelTaskDraftClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消创建任务包草稿不应保留待确认动作");
  assert(!taskCreateConfirmed, "取消确认不应调用创建动作");

  capturedAction = {
    kind: "copy-task-preview",
    label: "复制任务包 Markdown 预览",
    path: project.project_root,
    source: "索引内项目路径",
    boundary: "只复制预览文本到剪贴板；不写真实任务文件、不派发真实 Codex 会话。",
    taskPreview: {
      projectRoot: project.project_root,
      workItemId: selectedSecondDraft.work_item_id,
    },
  };
  let copyPreviewConfirmed = false;
  const copyPreviewDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        copyPreviewConfirmed = true;
      }}
    />
  );
  const copyPreviewDialogText = visibleText(copyPreviewDialog);
  for (const expectedText of [
    "复制任务包 Markdown 预览",
    "复制对象",
    "work-item:offline:002",
    "只复制预览文本",
    "不写真实任务文件、不派发真实 Codex 会话",
  ]) {
    assert(copyPreviewDialogText.includes(expectedText), `复制预览确认弹层缺少 ${expectedText}`);
  }
  const cancelCopyPreview = findButtonByText(copyPreviewDialog, "取消");
  assert(cancelCopyPreview, "复制预览确认弹层缺少取消按钮");
  const cancelCopyPreviewClick = cancelCopyPreview.props?.onClick;
  assert(typeof cancelCopyPreviewClick === "function", "复制预览取消按钮没有 onClick");
  cancelCopyPreviewClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消复制预览不应保留待确认动作");
  assert(!copyPreviewConfirmed, "取消复制预览不应执行复制");

  capturedAction = null;
  const taskFileGenerationPanel = (
    <TaskFileGenerationController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithProjectWorkflow.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
    />
  );
  const taskFileGenerationText = visibleText(taskFileGenerationPanel);
  for (const expectedText of ["真实任务包文件", "从当前草稿生成文件", "生成任务包文件"]) {
    assert(taskFileGenerationText.includes(expectedText), `任务文件生成区缺少 ${expectedText}`);
  }
  const generateButton = findButtonByText(taskFileGenerationPanel, "生成任务包文件");
  assert(generateButton, "任务草稿区缺少生成任务包文件按钮");
  const generateTaskFile = generateButton.props?.onClick;
  assert(typeof generateTaskFile === "function", "生成任务包文件按钮没有 onClick");
  generateTaskFile({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "generate-task-file",
      label: "生成任务包文件",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "写入 /Users/yoyi/workspace/product-line/tasks/ 下的新 Markdown 文件，并更新工作台自己的 workflow-state.v0.json；不覆盖已有任务包、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
      taskFileGeneration: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
      },
    },
    "生成任务包文件待确认动作不匹配",
  );
  let generateConfirmed = false;
  const generateDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        generateConfirmed = true;
      }}
    />
  );
  const generateDialogText = visibleText(generateDialog);
  for (const expectedText of [
    "生成任务包文件",
    "生成对象",
    "work-item:offline:001",
    "写入目录",
    "/Users/yoyi/workspace/product-line/tasks/",
    "不派发真实 Codex 会话",
    "不运行运行器",
    "不写 /Users/yoyi/.codex 或 Codex 状态库",
  ]) {
    assert(generateDialogText.includes(expectedText), `生成任务包文件确认弹层缺少 ${expectedText}`);
  }
  const cancelGenerate = findButtonByText(generateDialog, "取消");
  assert(cancelGenerate, "生成任务包文件确认弹层缺少取消按钮");
  const cancelGenerateClick = cancelGenerate.props?.onClick;
  assert(typeof cancelGenerateClick === "function", "生成任务包文件取消按钮没有 onClick");
  cancelGenerateClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消生成任务包文件不应保留待确认动作");
  assert(!generateConfirmed, "取消生成任务包文件不应调用生成动作");

  const generatedTaskFilePanel = (
    <TaskFileGenerationController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
    />
  );
  const generatedTaskFileText = visibleText(generatedTaskFilePanel);
  for (const expectedText of [
    "该草稿已有生成文件",
    "已生成",
    "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
  ]) {
    assert(generatedTaskFileText.includes(expectedText), `已有 path 时 UI 缺少 ${expectedText}`);
  }
  const generatedButton = findButtonByText(generatedTaskFilePanel, "已生成");
  assert(generatedButton, "已有 path 时缺少已生成按钮");
  assert(generatedButton.props?.disabled === true, "已有 path 时生成按钮应禁用");

  const dispatchReadinessPanel = (
    <TaskDispatchReadinessController
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      onRequestAction={captureAction}
      onInspectDispatchReadiness={async () => notReadyDispatchReadiness}
    />
  );
  const dispatchReadinessElement = dispatchReadinessPanel as ReactElementLike;
  assert(dispatchReadinessElement.props?.selectedTaskDraft === workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0], "派发准备区应绑定选中草稿");
  assert(typeof dispatchReadinessElement.props?.onInspectDispatchReadiness === "function", "派发准备区缺少检查入口");
  const notReadyShell = (
    <TaskDispatchReadinessShell
      readiness={notReadyDispatchReadiness}
      loading={false}
      error={null}
      onInspect={() => {}}
      onGenerateReadyFile={() => {
        throw new Error("not_ready should keep generation disabled");
      }}
    />
  );
  const notReadyShellText = visibleText(notReadyShell);
  for (const expectedText of ["派发准备", "任务包还不能派发", "not_ready", "检查派发准备", "生成可派发版本"]) {
    assert(notReadyShellText.includes(expectedText), `派发准备展示缺少 ${expectedText}`);
  }
  const readyFileButton = findButtonByText(notReadyShell, "生成可派发版本");
  assert(readyFileButton, "派发准备区缺少生成可派发版本按钮");
  assert(readyFileButton.props?.disabled === true, "not_ready 时生成可派发版本按钮应禁用");

  const renderedNotReady = visibleText(<TaskDispatchReadinessDetails readiness={notReadyDispatchReadiness} />);
  for (const expectedText of [
    "任务名为空、待补充或仍像测试草稿。",
    "禁止事项仍包含和当前生成行为冲突的历史禁令。",
    "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md",
  ]) {
    assert(renderedNotReady.includes(expectedText), `not_ready 原因展示缺少 ${expectedText}`);
  }

  const correctionFields: TaskPackageFields = {
    task_name: "派发准备字段修正",
    assigned_line: "桌面应用线",
    background: ["用户提供真实背景。"],
    goals: ["用户提供真实目标。"],
    allowed_read: [project.project_root],
    allowed_write: ["product-line/prototypes/productized-desktop-shell/src/"],
    forbidden_actions: ["不派发真实 Codex 会话。", "不运行运行器。"],
    acceptance_criteria: ["字段保存后可复检 readiness。"],
    required_return: ["做了什么", "改了哪些文件", "验证命令和结果", "风险和下一步建议"],
    review_focus: ["确认没有编造业务目标。"],
  };
  const correctionPreviewText = visibleText(<TaskFieldCorrectionPreview fields={correctionFields} />);
  for (const expectedText of ["字段级预览", "字段已填写，可复检 readiness", "派发准备字段修正", "用户提供真实目标。"]) {
    assert(correctionPreviewText.includes(expectedText), `字段修正预览缺少 ${expectedText}`);
  }
  assertDeepEqual(missingCorrectionFields(correctionFields), [], "完整字段不应有缺失提示");
  const missingPreviewFields = { ...correctionFields, goals: [], allowed_write: [] };
  const missingPreviewText = visibleText(<TaskFieldCorrectionPreview fields={missingPreviewFields} />);
  for (const expectedText of ["仍有字段缺失", "目标缺失", "允许写入缺失"]) {
    assert(missingPreviewText.includes(expectedText), `缺字段预览缺少 ${expectedText}`);
  }

  const correctionEditor = (
    <TaskDispatchFieldCorrectionShell
      projectRoot={project.project_root}
      selectedTaskDraft={workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0]}
      previewFields={correctionFields}
      onPreviewFieldsChange={() => {}}
      onRequestAction={captureAction}
    />
  );
  const correctionEditorText = visibleText(correctionEditor);
  for (const expectedText of ["修正任务字段", "保存前先看字段预览", "不自动补编", "保存派发字段修正"]) {
    assert(correctionEditorText.includes(expectedText), `字段修正入口缺少 ${expectedText}`);
  }

  capturedAction = buildCorrectDispatchFieldsAction(
    project.project_root,
    workflowStateWithGeneratedTaskFile.project_workflows[0].task_drafts[0].work_item_id,
    correctionFields,
  );
  assertDeepEqual(
    capturedAction,
    {
      kind: "correct-dispatch-fields",
      label: "保存派发字段修正",
      path: project.project_root,
      source: "索引内项目路径",
      boundary:
        "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
      dispatchFields: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:001",
        fields: correctionFields,
      },
    },
    "派发字段修正待确认动作不匹配",
  );
  let correctionConfirmed = false;
  const correctionDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        correctionConfirmed = true;
      }}
    />
  );
  const correctionDialogText = visibleText(correctionDialog);
  for (const expectedText of [
    "保存派发字段修正",
    "修正对象",
    "work-item:offline:001",
    "不生成真实任务包文件",
    "不派发真实 Codex 会话",
    "不运行运行器",
  ]) {
    assert(correctionDialogText.includes(expectedText), `派发字段修正确认弹层缺少 ${expectedText}`);
  }
  const cancelCorrection = findButtonByText(correctionDialog, "取消");
  assert(cancelCorrection, "派发字段修正确认弹层缺少取消按钮");
  const cancelCorrectionClick = cancelCorrection.props?.onClick;
  assert(typeof cancelCorrectionClick === "function", "派发字段修正取消按钮没有 onClick");
  cancelCorrectionClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消派发字段修正不应保留待确认动作");
  assert(!correctionConfirmed, "取消派发字段修正不应执行保存");

  capturedAction = null;
  const fieldValues = new Map<string, string>([
    ["task_name", "字段编辑任务"],
    ["assigned_line", "桌面应用线"],
    ["background", "来自结构化字段。"],
    ["goals", "完成字段编辑。"],
    ["allowed_read", "/tmp/indexed-project"],
    ["allowed_write", "工作台状态文件"],
    ["forbidden_actions", "不生成真实任务文件。"],
    ["acceptance_criteria", "预览使用新字段。"],
    ["required_return", "做了什么"],
    ["review_focus", "确认结构化字段。"],
  ]);
  capturedAction = buildUpdateTaskFieldsAction(project.project_root, selectedSecondDraft.work_item_id, fieldValues);
  assertDeepEqual(
    capturedAction,
    {
      kind: "update-task-fields",
      label: "保存任务包字段",
      path: project.project_root,
      source: "索引内项目路径",
      boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
      taskFields: {
        project_root: project.project_root,
        work_item_id: "work-item:offline:002",
        fields: {
          task_name: "字段编辑任务",
          assigned_line: "桌面应用线",
          background: ["来自结构化字段。"],
          goals: ["完成字段编辑。"],
          allowed_read: ["/tmp/indexed-project"],
          allowed_write: ["工作台状态文件"],
          forbidden_actions: ["不生成真实任务文件。"],
          acceptance_criteria: ["预览使用新字段。"],
          required_return: ["做了什么"],
          review_focus: ["确认结构化字段。"],
        },
      },
    },
    "保存任务字段待确认动作不匹配",
  );
  let saveFieldsConfirmed = false;
  const saveFieldsDialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        capturedAction = null;
      }}
      onConfirm={() => {
        saveFieldsConfirmed = true;
      }}
    />
  );
  const saveFieldsDialogText = visibleText(saveFieldsDialog);
  for (const expectedText of [
    "保存任务包字段",
    "更新对象",
    "work-item:offline:002",
    "字段编辑任务",
    "不生成真实任务文件、不派发真实 Codex 会话",
  ]) {
    assert(saveFieldsDialogText.includes(expectedText), `保存字段确认弹层缺少 ${expectedText}`);
  }
  const cancelSaveFields = findButtonByText(saveFieldsDialog, "取消");
  assert(cancelSaveFields, "保存字段确认弹层缺少取消按钮");
  const cancelSaveFieldsClick = cancelSaveFields.props?.onClick;
  assert(typeof cancelSaveFieldsClick === "function", "保存字段取消按钮没有 onClick");
  cancelSaveFieldsClick({ preventDefault() {}, stopPropagation() {} });
  assert(capturedAction === null, "取消保存字段不应保留待确认动作");
  assert(!saveFieldsConfirmed, "取消保存字段不应执行保存");

  capturedAction = null;
  let reloadRequested = false;
  const statePanel = (
    <WorkflowStatePanel
      workflowState={workflowState}
      loading={false}
      error={null}
      onReload={() => {
        reloadRequested = true;
      }}
      onRequestAction={captureAction}
    />
  );
  const stateText = visibleText(statePanel);
  for (const expectedText of [
    "本地事实层 v0",
    "存在状态",
    "不存在",
    "结构版本",
    "工作流版本",
    "工作流",
    "节点",
    "连线",
    "复核",
    "审计事件",
    "状态文件不存在；不会自动创建。",
  ]) {
    assert(stateText.includes(expectedText), `事实层面板缺少 ${expectedText}`);
  }
  const reloadButton = findButtonByText(statePanel, "重新读取事实层");
  assert(reloadButton, "事实层面板缺少重新读取按钮");
  const reload = reloadButton.props?.onClick;
  assert(typeof reload === "function", "重新读取按钮没有 onClick");
  reload({ preventDefault() {}, stopPropagation() {} });
  assert(reloadRequested, "重新读取按钮没有触发回调");

  const initButton = findButtonByText(statePanel, "初始化工作流事实层");
  assert(initButton, "事实层面板缺少初始化按钮");
  const init = initButton.props?.onClick;
  assert(typeof init === "function", "初始化按钮没有 onClick");
  init({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(
    capturedAction,
    {
      kind: "initialize-workflow-state",
      label: "初始化工作流事实层",
      path: workflowState.path,
      source: "Tauri 应用数据目录",
      boundary: "只写 workflow-state.v0.json 和同目录备份；不写 .codex、不写 Codex 状态库、不写项目业务目录。",
    },
    "初始化待确认动作不匹配",
  );

  const initDialogText = visibleText(
    <PermissionDialog action={capturedAction} busy={false} onCancel={() => {}} onConfirm={() => {}} />,
  );
  for (const expectedText of ["写入边界", "workflow-state.v0.json", "备份", "不写 .codex", "追加审计事件", "原子替换"]) {
    assert(initDialogText.includes(expectedText), `初始化确认弹层缺少 ${expectedText}`);
  }

  const skillText = visibleText(<SkillsBoardView skills={[skill]} plugins={[plugin]} projects={[project]} />);
  for (const expectedText of ["Skill 能力库", "可复用能力", "适用场景", "最近使用", "当前可用性", "开发者详情：来源和字段缺口"]) {
    assert(skillText.includes(expectedText), `Skill 看板缺少 ${expectedText}`);
  }

  const harnessText = visibleText(<HarnessBoardView projects={[project]} />);
  for (const expectedText of [
    "Harness 能力库",
    "运行器能力",
    "可运行范围",
    "最近运行",
    "等待配置 / 不可用原因",
    "开发者详情：资源字段和候选入口",
    "文件夹级运行器资源",
    "文件级运行器候选",
    "显示名",
    "根路径",
    "运行器类型",
    "智能体类型",
    "适配器编号",
    "来源类型",
    "能力",
    "清单路径",
    "说明路径",
    "版本",
    "入口",
    "node_script:check.js",
    "权限级别",
    "缺清单",
    "缺说明",
    "缺入口",
    "缺版本",
    "不新增运行按钮",
    "不自动运行运行器",
    "不代表可运行或已验证",
  ]) {
    assert(harnessText.includes(expectedText), `Harness 看板缺少 ${expectedText}`);
  }
}

function runScenario(scenario: Scenario) {
  capturedAction = null;
  const button = findButtonByText(scenario.root, scenario.buttonText);
  assert(button, `${scenario.name}: 找不到按钮 ${scenario.buttonText}`);
  const onClick = button.props?.onClick;
  assert(typeof onClick === "function", `${scenario.name}: 按钮没有 onClick`);

  onClick({ preventDefault() {}, stopPropagation() {} });
  assertDeepEqual(capturedAction, scenario.expectedAction, `${scenario.name}: 待确认动作不匹配`);

  let canceled = false;
  let confirmed = false;
  const dialog = (
    <PermissionDialog
      action={capturedAction}
      busy={false}
      onCancel={() => {
        canceled = true;
      }}
      onConfirm={() => {
        confirmed = true;
      }}
    />
  );

  const text = visibleText(dialog);
  for (const expectedText of [
    "本机动作确认",
    scenario.expectedAction.label,
    "目标路径",
    scenario.expectedAction.path,
    "路径来源",
    scenario.expectedAction.source,
    "取消",
    expectedDialogConfirmLabel(scenario.expectedAction.kind),
  ]) {
    assert(text.includes(expectedText), `${scenario.name}: 弹层缺少文本 ${expectedText}`);
  }

  const cancelButton = findButtonByText(dialog, "取消");
  assert(cancelButton, `${scenario.name}: 找不到取消按钮`);
  const cancel = cancelButton.props?.onClick;
  assert(typeof cancel === "function", `${scenario.name}: 取消按钮没有 onClick`);
  cancel({ preventDefault() {}, stopPropagation() {} });
  assert(canceled, `${scenario.name}: 取消按钮没有触发关闭回调`);
  assert(!confirmed, `${scenario.name}: 测试不应触发确认执行`);
}

function expectedDialogConfirmLabel(kind: PendingAction["kind"]) {
  if (kind === "run-workflow-machine") return "确认启动多轮真实执行";
  if (kind === "execute-node-dispatch") return "确认真实派发";
  if (kind === "copy-task-preview") return "确认复制";
  if (
    kind === "initialize-workflow-state" ||
    kind === "bootstrap-project-workflow" ||
    kind === "update-task-fields" ||
    kind === "correct-dispatch-fields" ||
    kind === "advance-work-item-state" ||
    kind === "bind-node-session" ||
    kind === "unbind-node-session"
  ) {
    return "确认写入状态";
  }
  if (
    kind === "record-director-review" ||
    kind === "record-permission-decision" ||
    kind === "record-worker-structured-report" ||
    kind === "record-project-director-process-fact-decision" ||
    kind === "record-global-final-result-review" ||
    kind === "generate-stage-c-acceptance-summary" ||
    kind === "offline-role-dispatch" ||
    kind === "offline-role-result-handoff" ||
    kind === "offline-director-review"
  ) {
    return "确认记录";
  }
  if (
    kind === "record-blackboard-candidate-decision" ||
    kind === "record-memory-candidate-decision" ||
    kind === "record-memory-entity-alias-decision" ||
    kind === "record-memory-entity-merge-decision" ||
    kind === "record-memory-relation-candidate-decision" ||
    kind === "record-mature-pattern-decision" ||
    kind === "record-project-consultation-proposal-decision" ||
    kind === "record-global-boundary-review" ||
    kind === "record-user-result-decision"
  ) {
    return "确认提交决定";
  }
  if (kind === "create-task-draft") return "确认创建草稿";
  if (
    kind === "create-memory-candidate" ||
    kind === "create-memory-candidate-from-observation" ||
    kind === "adopt-memory-candidate-to-formal-memory"
  ) {
    return "确认创建候选";
  }
  if (
    kind === "generate-task-file" ||
    kind === "record-formal-memory-lifecycle-operation" ||
    kind === "run-memory-maintenance" ||
    kind === "create-project-consultation-proposal" ||
    kind === "prepare-authorized-auto-dispatch"
  ) {
    return "确认创建记录";
  }
  if (kind === "preview-user-reviewed-instruction") return "确认边界预览";
  return "确认继续";
}

function findButtonByText(root: React.ReactNode, text: string): ReactElementLike | null {
  return findElement(root, (element) => element.type === "button" && visibleText(element).trim() === text);
}

function buildUpdateTaskFieldsAction(projectRoot: string, workItemId: string, values: Map<string, string>): PendingAction {
  return {
    kind: "update-task-fields",
    label: "保存任务包字段",
    path: projectRoot,
    source: "索引内项目路径",
    boundary: "写入工作台自己的 workflow-state.v0.json；不生成真实任务文件、不派发真实 Codex 会话。",
    taskFields: {
      project_root: projectRoot,
      work_item_id: workItemId,
      fields: {
        task_name: values.get("task_name") ?? "",
        assigned_line: values.get("assigned_line") ?? "",
        background: listValue(values.get("background")),
        goals: listValue(values.get("goals")),
        allowed_read: listValue(values.get("allowed_read")),
        allowed_write: listValue(values.get("allowed_write")),
        forbidden_actions: listValue(values.get("forbidden_actions")),
        acceptance_criteria: listValue(values.get("acceptance_criteria")),
        required_return: listValue(values.get("required_return")),
        review_focus: listValue(values.get("review_focus")),
      },
    },
  };
}

function buildCorrectDispatchFieldsAction(projectRoot: string, workItemId: string, fields: TaskPackageFields): PendingAction {
  return {
    kind: "correct-dispatch-fields",
    label: "保存派发字段修正",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只写工作台自己的 workflow-state.v0.json；不生成真实任务包文件、不派发真实 Codex 会话、不启动 Codex 命令行、不运行运行器、不写 .codex 或 Codex 状态库。",
    dispatchFields: {
      project_root: projectRoot,
      work_item_id: workItemId,
      fields,
    },
  };
}

function listValue(value: string | undefined): string[] {
  return (value ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function findButtonContainingText(root: React.ReactNode, textParts: string[]): ReactElementLike | null {
  return findElement(
    root,
    (element) => element.type === "button" && textParts.every((textPart) => visibleText(element).includes(textPart)),
  );
}

function findElement(root: React.ReactNode, predicate: (element: ReactElementLike) => boolean): ReactElementLike | null {
  if (!React.isValidElement(root)) return null;
  const element = root as ReactElementLike;
  if (predicate(element)) return element;

  const rendered = renderComposite(element);
  if (rendered !== element) {
    const match = findElement(rendered, predicate);
    if (match) return match;
  }

  const children = element.props?.children;
  const childArray = React.Children.toArray(children as React.ReactNode);
  for (const child of childArray) {
    const match = findElement(child, predicate);
    if (match) return match;
  }
  return null;
}

function visibleText(root: React.ReactNode): string {
  if (root === null || root === undefined || typeof root === "boolean") return "";
  if (typeof root === "string" || typeof root === "number") return String(root);
  if (Array.isArray(root)) return root.map(visibleText).join("");
  if (!React.isValidElement(root)) return "";

  return renderToStaticMarkup(root)
    .replace(/<[^>]*>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#x27;/g, "'")
    .replace(/&quot;/g, '"');
}

function buttonTextsInMarkup(markup: string): string[] {
  return (markup.match(/<button\b[\s\S]*?<\/button>/g) ?? []).map((button) =>
    button
      .replace(/<[^>]*>/g, "")
      .replace(/&nbsp;/g, " ")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&#x27;/g, "'")
      .replace(/&quot;/g, '"')
      .trim(),
  );
}

function renderComposite(element: ReactElementLike): React.ReactNode {
  if (typeof element.type !== "function") return element;
  const Component = element.type as (props: Record<string, unknown>) => React.ReactNode;
  return Component(element.props ?? {});
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown, message: string) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}

main();
