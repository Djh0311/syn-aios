import type {
  AgentAdapterDescriptor,
  ProjectRecord,
  SessionRecord,
  SessionOperationDescriptor,
  ProviderAvailabilitySummary,
  WorkbenchSnapshot,
  WorkflowStateSnapshot,
} from "./types";

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

function tail(value: string): string {
  const normalized = value.replace(/\/+$/, "");
  return normalized.split("/").pop() || value;
}
