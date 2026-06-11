import type {
  AgentAdapterDescriptor,
  FormalMemoryStoreV1,
  MaturePatternPreviewOutput,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  ObservationStoreV1,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  ProjectWorkflowSummary,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  SessionOperationDescriptor,
  ProviderAvailabilitySummary,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "./types";
import { deriveKnowledgeBaseSummary, type KnowledgeBaseSummary } from "./knowledgeBase";
import { deriveMemoryManagementSummary, type MemoryManagementSummary } from "./memoryCenter";
import { deriveRunQueueReadModel, type RunQueueReadModel } from "./runQueue";
import type { WorkbenchNavItem } from "./workbenchNavigation";

export type PageSelectorSourceBoundary = {
  generated_from: "workbench_snapshot_selector";
  workbench_snapshot_active: boolean;
  page_ui_migrated: boolean;
  tauri_command_consumed: boolean;
  writes_stores: boolean;
  warnings: string[];
};

export type ProjectListItemReadModel = {
  project_root: string;
  name: string;
  active_hint: boolean;
  session_count: number;
  active_session_count: number;
  archived_session_count: number;
  workflow_count: number;
  evidence_count: number;
  handoff_count: number;
  authority_count: number;
  warning_count: number;
  latest_updated_at_ms?: number | null;
};

export type ProjectsPageReadModel = {
  schema_version: "projects_page_read_model.v1";
  selector_id: "projects_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  project_count: number;
  active_project_count: number;
  total_session_count: number;
  workflow_summary_count: number;
  projects: ProjectListItemReadModel[];
  user_facing_summary: string;
  developer_details_collapsed: true;
  warnings: string[];
};

export type AgentProjectOptionReadModel = {
  project_root: string;
  label: string;
  session_count: number;
  active_session_count: number;
};

export type AgentSessionSummaryReadModel = {
  readable_count: number;
  missing_rollout_count: number;
  archived_count: number;
  total_count: number;
};

export type AgentsPageReadModel = {
  schema_version: "agents_page_read_model.v1";
  selector_id: "agents_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  project_options: AgentProjectOptionReadModel[];
  session_summary: AgentSessionSummaryReadModel;
  adapter_count: number;
  available_adapter_count: number;
  planned_adapter_count: number;
  operation_boundary_count: number;
  provider_boundary_count: number;
  conversation_first: true;
  developer_details_collapsed: true;
  user_facing_summary: string;
  warnings: string[];
};

export type RunningWorkflowsPageReadModel = {
  schema_version: "running_workflows_page_read_model.v1";
  selector_id: "running_workflows_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  workflow_count: number;
  workflow_focus_count: number;
  running_attention_count: number;
  runtime_attention_count: number;
  waiting_permission_count: number;
  readback_issue_count: number;
  readback_unknown_result_count: number;
  run_queue: {
    item_count: number;
    running_count: number;
    waiting_user_count: number;
    blocked_count: number;
    failed_count: number;
    failure_control_count: number;
    duplicate_blocked_count: number;
    capture_compensation_count: number;
  };
  operation_control: {
    confirmation_required_count: number;
    retry_proposal_count: number;
    stop_request_count: number;
    restart_readiness_count: number;
    resume_readiness_count: number;
    readback_issue_count: number;
    manual_review_count: number;
    blocked_by_guard_count: number;
    duplicate_blocked_count: number;
    stale_cleanup_count: number;
  };
  memory_pending: {
    confirmation_count: number;
    capture_count: number;
    pending_candidate_count: number;
  };
  product_command: {
    command_count: number;
    pending_decision_count: number;
    blocked_attempt_count: number;
    running_attempt_count: number;
    readback_issue_count: number;
  };
  automation: {
    run_unit_count: number;
    waiting_user_count: number;
    blocked_count: number;
    readback_unknown_count: number;
  };
  user_facing_summary: string;
  developer_details_collapsed: true;
  warnings: string[];
};

export type MemoryCenterPageReadModel = {
  schema_version: "memory_center_page_read_model.v1";
  selector_id: "memory_center_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  snapshot_status_label: string;
  boundary: string;
  formal_memory: {
    record_count: number;
    active_count: number;
  };
  candidate_memory: {
    candidate_count: number;
  };
  observation: {
    observation_count: number;
  };
  lint: {
    open_count: number;
    blocking_count: number;
  };
  maintenance: {
    blocking_count: number;
    needs_review_count: number;
    info_count: number;
  };
  mature_pattern: {
    candidate_count: number;
    user_confirmation_required_count: number;
  };
  task_package: {
    snapshot_count: number;
  };
  memory_workbench: {
    action_count: number;
    capture_count: number;
    observation_count: number;
    candidate_count: number;
    confirmed_pending_formalization_count: number;
    capture_compensation_count: number;
  };
  user_facing_summary: string;
  developer_details_collapsed: true;
  warnings: string[];
};

export type KnowledgeBasePageReadModel = {
  schema_version: "knowledge_base_page_read_model.v1";
  selector_id: "knowledge_base_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  snapshot_status_label: string;
  boundary_text: string;
  document_count: number;
  formal_memory_link_count: number;
  candidate_link_count: number;
  task_reference_count: number;
  capture_event_count: number;
  obsidian_boundary: {
    label: string;
    native_sync_status: string;
    vault_scan_status: string;
    forbidden_text: string;
  };
  user_facing_summary: string;
  developer_details_collapsed: true;
  warnings: string[];
};

export type SettingsPageReadModel = {
  schema_version: "settings_page_read_model.v1";
  selector_id: "settings_page_selector_v1";
  source_boundary: PageSelectorSourceBoundary;
  snapshot_status_label: string;
  boundary_text: string;
  general: {
    project_count: number;
    session_count: number;
    skill_count: number;
    workflow_count: number;
  };
  developer_boundary: {
    developer_item_count: number;
    adapter_count: number;
    provider_count: number;
    diagnostic_count: number;
    runtime_log_count: number;
    page_contract_count: number;
    credential_display_allowed: false;
    execution_from_settings_allowed: false;
  };
  page_contract: {
    count: number;
    status: string;
    source_policy: string;
  };
  user_facing_summary: string;
  developer_details_collapsed: true;
  warnings: string[];
};

export function deriveProjectsPageReadModel({
  snapshot,
  workflowState,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
}): ProjectsPageReadModel {
  return deriveProjectsPageReadModelFromParts({
    projects: snapshot.projects,
    sessions: snapshot.sessions,
    workflowState,
  });
}

export function deriveProjectsPageReadModelFromParts({
  projects: sourceProjects,
  sessions,
  workflowState,
}: {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  workflowState?: WorkflowStateSnapshot | null;
}): ProjectsPageReadModel {
  const sessionsByProject = groupSessionsByProject(sessions);
  const workflowsByProject = groupWorkflowsByProject(workflowState);
  const projects = sourceProjects.map((project) => {
    const sessions = sessionsByProject.get(project.project_root) ?? [];
    const workflowCount = workflowsByProject.get(project.project_root) ?? 0;
    return {
      project_root: project.project_root,
      name: project.name,
      active_hint: project.active_hint,
      session_count: sessions.length || project.thread_count,
      active_session_count: sessions.length ? sessions.filter((session) => !session.archived).length : project.active_thread_count,
      archived_session_count: sessions.length ? sessions.filter((session) => session.archived).length : project.archived_thread_count,
      workflow_count: workflowCount,
      evidence_count: project.evidence_files.length,
      handoff_count: project.handoff_files.length,
      authority_count: project.authority_files.length,
      warning_count: project.context_warnings.length + project.warnings.length,
      latest_updated_at_ms: project.latest_updated_at_ms ?? null,
    };
  });

  projects.sort((a, b) => {
    if (a.active_hint !== b.active_hint) return a.active_hint ? -1 : 1;
    return a.name.localeCompare(b.name);
  });

  return {
    schema_version: "projects_page_read_model.v1",
    selector_id: "projects_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    project_count: projects.length,
    active_project_count: projects.filter((project) => project.active_hint).length,
    total_session_count: sessions.length,
    workflow_summary_count: workflowState?.project_workflows.length ?? 0,
    projects,
    user_facing_summary: projects.length
      ? `${projects.length} 个项目，${sessions.length} 个会话，${workflowState?.project_workflows.length ?? 0} 条工作流摘要`
      : "暂无项目；页面仍等待工作台索引提供真实数据",
    developer_details_collapsed: true,
    warnings: [
      "r4_a3_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      ...sourceProjects.flatMap((project) => project.warnings).slice(0, 5),
    ],
  };
}

export function deriveAgentsPageReadModel({
  snapshot,
}: {
  snapshot: WorkbenchSnapshot;
}): AgentsPageReadModel {
  return deriveAgentsPageReadModelFromParts({
    projects: snapshot.projects,
    sessions: snapshot.sessions,
    adapterDescriptors: snapshot.agent_adapters,
    sessionOperationDescriptors: snapshot.session_operations,
    providerAvailabilitySummaries: snapshot.provider_availability,
  });
}

export function deriveAgentsPageReadModelFromParts({
  projects,
  sessions,
  adapterDescriptors,
  sessionOperationDescriptors,
  providerAvailabilitySummaries,
}: {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  adapterDescriptors: AgentAdapterDescriptor[];
  sessionOperationDescriptors: SessionOperationDescriptor[];
  providerAvailabilitySummaries: ProviderAvailabilitySummary[];
}): AgentsPageReadModel {
  const projectOptions = deriveAgentProjectOptions(projects, sessions);
  const sessionSummary = summarizeSessions(sessions);
  const adapterSummary = summarizeAdapters(adapterDescriptors);

  return {
    schema_version: "agents_page_read_model.v1",
    selector_id: "agents_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    project_options: projectOptions,
    session_summary: sessionSummary,
    adapter_count: adapterDescriptors.length,
    available_adapter_count: adapterSummary.available,
    planned_adapter_count: adapterSummary.planned,
    operation_boundary_count: countBoundaries(sessionOperationDescriptors),
    provider_boundary_count: countProviderBoundaries(providerAvailabilitySummaries),
    conversation_first: true,
    developer_details_collapsed: true,
    user_facing_summary: `${sessionSummary.total_count} 个会话，${sessionSummary.readable_count} 个可读取，${adapterSummary.available} 个可用 adapter`,
    warnings: [
      "r4_a3_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      "developer_boundary_data_must_stay_collapsed",
      ...sessions.flatMap((session) => session.warnings).slice(0, 5),
    ],
  };
}

export function deriveRunningWorkflowsPageReadModel({
  snapshot,
  workflowState,
  memoryCaptureStore,
  memoryCandidateStore,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
}): RunningWorkflowsPageReadModel {
  const runQueue = deriveRunQueueReadModel({ snapshot, workflowState, memoryCaptureStore, memoryCandidateStore });
  return deriveRunningWorkflowsPageReadModelFromParts({
    workflows: workflowState?.project_workflows ?? [],
    runtimeAttention: snapshot.runtime_session_attention,
    runQueue,
    productCommandReadModel: snapshot.real_execution_product_commands ?? null,
    automation: snapshot.project_workflow_automation ?? null,
    memoryCaptureStore,
    memoryCandidateStore,
  });
}

export function deriveRunningWorkflowsPageReadModelFromParts({
  workflows,
  runtimeAttention,
  runQueue,
  productCommandReadModel,
  automation,
  memoryCaptureStore,
  memoryCandidateStore,
}: {
  workflows: ProjectWorkflowSummary[];
  runtimeAttention: RuntimeSessionAttention[];
  runQueue: RunQueueReadModel;
  productCommandReadModel?: RealExecutionProductCommandReadModel | null;
  automation?: ProjectWorkflowAutomationReadModel | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
}): RunningWorkflowsPageReadModel {
  const workflowFocusCount = workflows.filter(isWorkflowInRunningFocus).length;
  const waitingPermissionCount = workflows.reduce(
    (count, workflow) => count + workflow.task_drafts.filter((task) => task.state === "waiting_for_permission").length,
    0,
  );
  const runtimeAttentionInFocus = runtimeAttention.filter(
    (item) => item.requires_user_action || item.blocks_continuation || runningFocusStates.has(item.status),
  );
  const readbackIssueCount = runtimeAttention.filter((item) =>
    item.readback_boundary.status === "readback_unavailable" || item.readback_boundary.status === "readback_failed",
  ).length;
  const readbackUnknownResultCount = runQueue.run_queue_items.filter((item) =>
    (item.readback_status === "readback_unavailable" ||
      item.readback_status === "readback_failed" ||
      item.readback_status === "timed_out" ||
      item.readback_status === "unknown") &&
    item.readback_result_count == null,
  ).length;
  const operationControl = runQueue.operation_control_summary;
  const memoryConfirmationCount = runQueue.user_confirmation_queue.filter((item) =>
    item.kind === "memory_candidate_confirmation" ||
    item.kind === "memory_formalization_confirmation" ||
    item.kind === "capture_compensation_confirmation",
  ).length;
  const pendingMemoryCandidateCount = (memoryCandidateStore?.candidates ?? []).filter((candidate) =>
    candidate.status === "candidate_draft" || candidate.status === "candidate_needs_review" ||
    (candidate.status === "candidate_confirmed" && !candidate.adoption),
  ).length;
  const failureStopRetry = productCommandReadModel?.failure_stop_retry_summary ?? null;

  return {
    schema_version: "running_workflows_page_read_model.v1",
    selector_id: "running_workflows_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    workflow_count: workflows.length,
    workflow_focus_count: workflowFocusCount,
    running_attention_count: workflowFocusCount + runtimeAttentionInFocus.length,
    runtime_attention_count: runtimeAttentionInFocus.length,
    waiting_permission_count: waitingPermissionCount,
    readback_issue_count: readbackIssueCount,
    readback_unknown_result_count: readbackUnknownResultCount,
    run_queue: {
      item_count: runQueue.run_queue_items.length,
      running_count: runQueue.running_count,
      waiting_user_count: runQueue.waiting_user_count,
      blocked_count: runQueue.blocked_count,
      failed_count: runQueue.failed_count,
      failure_control_count: runQueue.failure_control_summaries.length,
      duplicate_blocked_count: runQueue.duplicate_blocked_count,
      capture_compensation_count: runQueue.capture_compensation_count,
    },
    operation_control: {
      confirmation_required_count: operationControl.confirmation_required_count,
      retry_proposal_count: operationControl.retry_proposal_count,
      stop_request_count: operationControl.stop_request_count,
      restart_readiness_count: operationControl.restart_readiness_count,
      resume_readiness_count: operationControl.resume_readiness_count,
      readback_issue_count: operationControl.readback_issue_count,
      manual_review_count: operationControl.manual_review_count,
      blocked_by_guard_count: operationControl.blocked_by_guard_count,
      duplicate_blocked_count: operationControl.duplicate_blocked_count,
      stale_cleanup_count: operationControl.stale_cleanup_count,
    },
    memory_pending: {
      confirmation_count: memoryConfirmationCount,
      capture_count: memoryCaptureStore?.events.length ?? 0,
      pending_candidate_count: pendingMemoryCandidateCount,
    },
    product_command: {
      command_count: productCommandReadModel?.command_count ?? 0,
      pending_decision_count: productCommandReadModel?.pending_decision_count ?? 0,
      blocked_attempt_count: productCommandReadModel?.blocked_attempt_count ?? 0,
      running_attempt_count: productCommandReadModel?.running_attempt_count ?? 0,
      readback_issue_count: failureStopRetry?.readback_issue_count ?? 0,
    },
    automation: {
      run_unit_count: automation?.run_unit_count ?? 0,
      waiting_user_count: automation?.waiting_user_count ?? 0,
      blocked_count: automation?.blocked_count ?? 0,
      readback_unknown_count: automation?.readback_unknown_count ?? 0,
    },
    user_facing_summary: `${workflows.length} 条工作流，${workflowFocusCount + runtimeAttentionInFocus.length} 条运行关注，${waitingPermissionCount} 条等待权限`,
    developer_details_collapsed: true,
    warnings: [
      "r4_a5_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      "readback_unavailable_must_not_render_as_zero",
      ...runQueue.warnings.slice(0, 5),
    ],
  };
}

export function deriveMemoryCenterPageReadModel({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
  observationStore,
  memoryLintStore,
  memoryEntityRelationStore,
  memoryEntityRelationPreview,
  memoryPatternStore,
  maturePatternPreview,
  hasRealSnapshot,
}: {
  projects: ProjectRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  memoryEntityRelationStore?: MemoryEntityRelationStoreV1 | null;
  memoryEntityRelationPreview?: MemoryEntityRelationPreviewOutput | null;
  memoryPatternStore?: MemoryPatternStoreV1 | null;
  maturePatternPreview?: MaturePatternPreviewOutput | null;
  hasRealSnapshot: boolean;
}): MemoryCenterPageReadModel {
  const summary = deriveMemoryManagementSummary({
    projects,
    workflowState: workflowState ?? null,
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
  return deriveMemoryCenterPageReadModelFromParts({ summary, hasRealSnapshot });
}

export function deriveMemoryCenterPageReadModelFromParts({
  summary,
  hasRealSnapshot,
}: {
  summary: MemoryManagementSummary;
  hasRealSnapshot: boolean;
}): MemoryCenterPageReadModel {
  return {
    schema_version: "memory_center_page_read_model.v1",
    selector_id: "memory_center_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    snapshot_status_label: hasRealSnapshot ? "只读读模型" : "未读取真实索引",
    boundary: summary.boundary,
    formal_memory: {
      record_count: summary.formal_summary.record_count,
      active_count: summary.formal_summary.active_count,
    },
    candidate_memory: {
      candidate_count: summary.candidate_summary.candidate_count,
    },
    observation: {
      observation_count: summary.observation_summary.observation_count,
    },
    lint: {
      open_count: summary.lint_summary.open_count,
      blocking_count: summary.lint_summary.blocking_count,
    },
    maintenance: {
      blocking_count: summary.maintenance_summary.blocking_count,
      needs_review_count: summary.maintenance_summary.needs_review_count,
      info_count: summary.maintenance_summary.info_count,
    },
    mature_pattern: {
      candidate_count: summary.mature_pattern_summary.mature_pattern_candidate_count,
      user_confirmation_required_count: summary.mature_pattern_summary.user_confirmation_required_count,
    },
    task_package: {
      snapshot_count: summary.task_package_summary.snapshot_count,
    },
    memory_workbench: {
      action_count: summary.memory_workbench_summary.action_count,
      capture_count: summary.memory_workbench_summary.capture_count,
      observation_count: summary.memory_workbench_summary.observation_count,
      candidate_count: summary.memory_workbench_summary.candidate_count,
      confirmed_pending_formalization_count: summary.memory_workbench_summary.confirmed_pending_formalization_count,
      capture_compensation_count: summary.memory_workbench_summary.capture_compensation_count,
    },
    user_facing_summary:
      `正式 ${summary.formal_summary.record_count}，候选 ${summary.candidate_summary.candidate_count}，` +
      `观察 ${summary.observation_summary.observation_count}，待处理 ${summary.memory_workbench_summary.action_count}`,
    developer_details_collapsed: true,
    warnings: [
      "r4_a5_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      "candidate_and_observation_are_not_formal_memory",
      ...summary.warnings.slice(0, 5),
    ],
  };
}

export function deriveKnowledgeBasePageReadModel({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
  hasRealSnapshot,
}: {
  projects: ProjectRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  hasRealSnapshot: boolean;
}): KnowledgeBasePageReadModel {
  const summary = deriveKnowledgeBaseSummary({
    projects,
    workflowState: workflowState ?? null,
    formalMemoryStore,
    memoryCaptureStore,
    memoryCandidateStore,
  });
  return deriveKnowledgeBasePageReadModelFromParts({ summary, hasRealSnapshot });
}

export function deriveKnowledgeBasePageReadModelFromParts({
  summary,
  hasRealSnapshot,
}: {
  summary: KnowledgeBaseSummary;
  hasRealSnapshot: boolean;
}): KnowledgeBasePageReadModel {
  return {
    schema_version: "knowledge_base_page_read_model.v1",
    selector_id: "knowledge_base_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    snapshot_status_label: hasRealSnapshot ? "权威文件读模型" : "未读取真实索引",
    boundary_text: summary.obsidian_boundary.display_text,
    document_count: summary.document_count,
    formal_memory_link_count: summary.formal_memory_link_count,
    candidate_link_count: summary.candidate_link_count,
    task_reference_count: summary.task_reference_count,
    capture_event_count: summary.capture_event_count,
    obsidian_boundary: {
      label: summary.obsidian_boundary.label,
      native_sync_status: summary.obsidian_boundary.native_sync_status,
      vault_scan_status: summary.obsidian_boundary.vault_scan_status,
      forbidden_text: summary.obsidian_boundary.forbidden_text,
    },
    user_facing_summary:
      `资料 ${summary.document_count}，正式记忆关联 ${summary.formal_memory_link_count}，` +
      `候选关联 ${summary.candidate_link_count}，捕获 ${summary.capture_event_count}`,
    developer_details_collapsed: true,
    warnings: [
      "r4_a6_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      "knowledge_hit_and_candidate_are_not_formal_memory",
      ...summary.warnings.slice(0, 5),
    ],
  };
}

export function deriveSettingsPageReadModel({
  snapshot,
  workflowState,
  workflowStateError,
  hasRealSnapshot,
  developerItems,
}: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  workflowStateError?: string | null;
  hasRealSnapshot: boolean;
  developerItems: WorkbenchNavItem[];
}): SettingsPageReadModel {
  return deriveSettingsPageReadModelFromParts({
    summary: snapshot.summary,
    workflowCount: workflowState?.counts.workflows ?? 0,
    workflowStateError,
    hasRealSnapshot,
    developerItems,
    adapterCount: snapshot.agent_adapters.length,
    providerCount: snapshot.provider_availability.length,
    diagnosticCount: snapshot.diagnostic_summary.degraded_states.length,
    runtimeLogCount: snapshot.runtime_log_store.entries.length,
    pageReadModelInventory: snapshot.page_read_model_inventory,
  });
}

export function deriveSettingsPageReadModelFromParts({
  summary,
  workflowCount,
  workflowStateError,
  hasRealSnapshot,
  developerItems,
  adapterCount,
  providerCount,
  diagnosticCount,
  runtimeLogCount,
  pageReadModelInventory,
}: {
  summary: WorkbenchSnapshot["summary"];
  workflowCount: number;
  workflowStateError?: string | null;
  hasRealSnapshot: boolean;
  developerItems: WorkbenchNavItem[];
  adapterCount: number;
  providerCount: number;
  diagnosticCount: number;
  runtimeLogCount: number;
  pageReadModelInventory: WorkbenchSnapshot["page_read_model_inventory"];
}): SettingsPageReadModel {
  return {
    schema_version: "settings_page_read_model.v1",
    selector_id: "settings_page_selector_v1",
    source_boundary: selectorSourceBoundary(),
    snapshot_status_label: hasRealSnapshot ? "索引已读取" : "未接真实数据",
    boundary_text: "设置页只整理入口和边界，不读取凭据、不触发执行。",
    general: {
      project_count: summary.project_count,
      session_count: summary.session_count,
      skill_count: summary.skill_count,
      workflow_count: workflowCount,
    },
    developer_boundary: {
      developer_item_count: developerItems.length,
      adapter_count: adapterCount,
      provider_count: providerCount,
      diagnostic_count: diagnosticCount,
      runtime_log_count: runtimeLogCount,
      page_contract_count: pageReadModelInventory.contracts.length,
      credential_display_allowed: false,
      execution_from_settings_allowed: false,
    },
    page_contract: {
      count: pageReadModelInventory.contracts.length,
      status: pageReadModelInventory.status,
      source_policy: pageReadModelInventory.source_policy,
    },
    user_facing_summary:
      `${summary.project_count} 个项目，${summary.session_count} 个会话，` +
      `${pageReadModelInventory.contracts.length} 个页面合同，${diagnosticCount} 条诊断状态`,
    developer_details_collapsed: true,
    warnings: [
      "r4_a6_selector_only_page_ui_not_migrated",
      "workbench_snapshot_still_active",
      "settings_page_must_not_read_credentials_or_trigger_execution",
      ...(workflowStateError ? ["workflow_state_read_error_visible"] : []),
    ],
  };
}

function selectorSourceBoundary(): PageSelectorSourceBoundary {
  return {
    generated_from: "workbench_snapshot_selector",
    workbench_snapshot_active: true,
    page_ui_migrated: false,
    tauri_command_consumed: false,
    writes_stores: false,
    warnings: [
      "selector_is_frontend_pure_function",
      "page_consumption_not_migrated",
      "do_not_claim_workbench_snapshot_deprecated",
    ],
  };
}

function groupSessionsByProject(sessions: SessionRecord[]): Map<string, SessionRecord[]> {
  const map = new Map<string, SessionRecord[]>();
  for (const session of sessions) {
    if (!session.project_root) continue;
    const bucket = map.get(session.project_root) ?? [];
    bucket.push(session);
    map.set(session.project_root, bucket);
  }
  return map;
}

function groupWorkflowsByProject(workflowState?: WorkflowStateSnapshot | null): Map<string, number> {
  const map = new Map<string, number>();
  for (const workflow of workflowState?.project_workflows ?? []) {
    const current = map.get(workflow.project_root) ?? 0;
    map.set(workflow.project_root, current + 1);
  }
  return map;
}

function deriveAgentProjectOptions(
  projects: ProjectRecord[],
  sessions: SessionRecord[],
): AgentProjectOptionReadModel[] {
  const sessionsByProject = groupSessionsByProject(sessions);
  const knownRoots = new Set(projects.map((project) => project.project_root));
  const options: AgentProjectOptionReadModel[] = projects.map((project) => {
    const projectSessions = sessionsByProject.get(project.project_root) ?? [];
    return {
      project_root: project.project_root,
      label: project.name,
      session_count: projectSessions.length || project.thread_count,
      active_session_count: projectSessions.length
        ? projectSessions.filter((session) => !session.archived).length
        : project.active_thread_count,
    };
  });

  for (const [projectRoot, projectSessions] of sessionsByProject.entries()) {
    if (knownRoots.has(projectRoot)) continue;
    options.push({
      project_root: projectRoot,
      label: tail(projectRoot),
      session_count: projectSessions.length,
      active_session_count: projectSessions.filter((session) => !session.archived).length,
    });
  }

  return options.sort((a, b) => a.label.localeCompare(b.label));
}

function summarizeSessions(sessions: SessionRecord[]): AgentSessionSummaryReadModel {
  let readable = 0;
  let missing = 0;
  let archived = 0;
  for (const session of sessions) {
    if (session.archived) archived += 1;
    if (!session.rollout_exists || !session.rollout_path) missing += 1;
    if (!session.archived && session.rollout_exists && !!session.rollout_path) readable += 1;
  }
  return {
    readable_count: readable,
    missing_rollout_count: missing,
    archived_count: archived,
    total_count: sessions.length,
  };
}

function summarizeAdapters(adapters: AgentAdapterDescriptor[]): { available: number; planned: number } {
  return adapters.reduce(
    (summary, adapter) => {
      if (adapter.status === "available" || adapter.execution_status === "available_with_user_confirmation") {
        summary.available += 1;
      }
      if (adapter.status === "planned" || adapter.execution_status === "not_implemented") {
        summary.planned += 1;
      }
      return summary;
    },
    { available: 0, planned: 0 },
  );
}

function countBoundaries(operations: SessionOperationDescriptor[]): number {
  return operations.filter((operation) => operation.current_status !== "readonly_available").length;
}

function countProviderBoundaries(providers: ProviderAvailabilitySummary[]): number {
  return providers.filter((provider) => provider.requires_future_task || provider.requires_user_configuration).length;
}

const runningFocusStates = new Set([
  "running",
  "waiting_for_permission",
  "ready_to_dispatch",
  "ready_for_review",
  "retry_pending",
  "blocked_by_guard",
  "readback_unavailable",
  "readback_failed",
  "timed_out",
]);

function isWorkflowInRunningFocus(workflow: ProjectWorkflowSummary): boolean {
  return runningFocusStates.has(workflow.state) || workflow.task_drafts.some((task) => runningFocusStates.has(task.state));
}

function tail(value: string): string {
  const normalized = value.replace(/\/+$/, "");
  return normalized.split("/").pop() || value;
}
