import { deriveAgentAdapterDescriptors } from "../../src/lib/adapterCapabilities";
import { deriveProviderAvailabilitySummaries } from "../../src/lib/providerAvailability";
import { deriveSessionOperationDescriptors } from "../../src/lib/sessionOperations";
import type {
  AgentAdapterDescriptor,
  PluginRecord,
  ProjectRecord,
  SessionRecord,
  SkillRecord,
  WorkbenchSnapshot,
} from "../../src/lib/types";
import { diagnosticSummaryFixture, runtimeLogStoreFixture } from "./offlineRuntimeDiagnosticFixtures";

export function workbenchBaseFixtures(): {
  backendAgentAdapterDescriptors: AgentAdapterDescriptor[];
  backendProviderAvailabilitySummaries: ReturnType<typeof deriveProviderAvailabilitySummaries>;
  backendSessionOperationDescriptors: ReturnType<typeof deriveSessionOperationDescriptors>;
  emptyProject: ProjectRecord;
  otherProjectSession: SessionRecord;
  plugin: PluginRecord;
  project: ProjectRecord;
  session: SessionRecord;
  skill: SkillRecord;
  snapshot: WorkbenchSnapshot;
  workflowId: string;
  workflowProjectId: string;
} {
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
    runtime_log_store: runtimeLogStoreFixture(project.project_root, session.thread_id),
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
      latest_status: null,
      latest_plan: null,
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
    page_read_model_inventory: {
      schema_version: "workbench_page_read_model_inventory.v1",
      generated_at: "2026-06-11T00:00:00Z",
      status: "contract_only",
      source_policy: "offline fixture",
      contracts: [],
      warnings: [],
    },
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

  return {
    backendAgentAdapterDescriptors,
    backendProviderAvailabilitySummaries,
    backendSessionOperationDescriptors,
    emptyProject,
    otherProjectSession,
    plugin,
    project,
    session,
    skill,
    snapshot,
    workflowId: "workflow:offline-fixture-projects-codex-workbench:default",
    workflowProjectId: "project:offline-fixture-projects-codex-workbench",
  };
}
