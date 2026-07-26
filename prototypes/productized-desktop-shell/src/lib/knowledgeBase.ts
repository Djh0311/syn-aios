import type {
  CreateMemoryCandidateInput,
  FileCandidate,
  FormalMemoryStoreV1,
  MemoryCaptureEventRecord,
  MemoryCaptureStoreV1,
  MemoryCandidate,
  MemoryCandidateStoreV1,
  MemoryRecord,
  MemorySourceRef,
  ProjectRecord,
  TaskPackage,
  WorkflowStateSnapshot,
} from "./types";

export type KnowledgeSourceAnchor = {
  anchor_label: string;
  path_summary: string;
  source_kind: string;
  warnings: string[];
};

export type KnowledgeMemoryLink = {
  kind: "formal_memory" | "memory_candidate";
  label: string;
  claim: string;
  status: string;
  boundary: string;
};

export type KnowledgeTaskReferenceSummary = {
  reference_count: number;
  task_goals: string[];
  display_text: string;
};

export type KnowledgeMemoryCaptureSummary = {
  label: string;
  summary: string;
  policy_label: string;
  boundary: string;
  created_at: string;
};

export type KnowledgeCandidateDraft = {
  label: "提出记忆候选";
  input: CreateMemoryCandidateInput;
  boundary: "只生成候选，不写正式记忆";
};

export type KnowledgeDocumentReadModel = {
  document_key: string;
  title: string;
  project_name: string;
  project_root: string;
  project_id: string;
  workflow_id?: string | null;
  source_anchor: KnowledgeSourceAnchor;
  formal_memory_links: KnowledgeMemoryLink[];
  candidate_links: KnowledgeMemoryLink[];
  task_reference_summary: KnowledgeTaskReferenceSummary;
  candidate_draft: KnowledgeCandidateDraft;
  boundary: string;
};

export type ObsidianCompatibleBoundarySummary = {
  // Keep this legacy read-model shape stable for the existing page selector.
  // The product boundary is Syn-native; Obsidian is only an optional external
  // Markdown/Canvas compatibility target.
  label: string;
  native_sync_status: string;
  vault_scan_status: string;
  display_text: string;
  forbidden_text: string;
};

export type KnowledgeBaseSummary = {
  source_kind: "frontend_read_model";
  document_count: number;
  formal_memory_link_count: number;
  candidate_link_count: number;
  task_reference_count: number;
  capture_event_count: number;
  recent_capture_events: KnowledgeMemoryCaptureSummary[];
  documents: KnowledgeDocumentReadModel[];
  obsidian_boundary: ObsidianCompatibleBoundarySummary;
  warnings: string[];
};

export function deriveKnowledgeBaseSummary({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
}: {
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
}): KnowledgeBaseSummary {
  const taskPackages = collectTaskPackages(workflowState);
  const formalRecords = formalMemoryStore?.records ?? [];
  const captureEvents = memoryCaptureStore?.events ?? [];
  const candidates = memoryCandidateStore?.candidates ?? [];
  const documents = projects.flatMap((project) =>
    project.authority_files.map((file) =>
      buildKnowledgeDocument({
        project,
        file,
        projectWorkflow: workflowState?.project_workflows.find((item) => item.project_root === project.project_root) ?? null,
        formalRecords,
        candidates,
        taskPackages,
        expectedCandidateStoreRevision: memoryCandidateStore?.revision ?? null,
      }),
    ),
  );
  const warnings = new Set<string>();
  for (const project of projects) {
    for (const warning of project.warnings) warnings.add(warning);
  }
  for (const warning of memoryCaptureStore?.warnings ?? []) warnings.add(warning);
  return {
    source_kind: "frontend_read_model",
    document_count: documents.length,
    formal_memory_link_count: documents.reduce((total, document) => total + document.formal_memory_links.length, 0),
    candidate_link_count: documents.reduce((total, document) => total + document.candidate_links.length, 0),
    task_reference_count: documents.reduce((total, document) => total + document.task_reference_summary.reference_count, 0),
    capture_event_count: captureEvents.length,
    recent_capture_events: captureEvents.slice(0, 4).map(buildKnowledgeMemoryCaptureSummary),
    documents,
    obsidian_boundary: {
      label: "Syn 原生知识工作区",
      native_sync_status: "Syn 原生 Markdown 工作区可独立使用；官方 Obsidian 仅为可选外部打开",
      vault_scan_status: "仅访问 Syn 自管 vault",
      display_text: "Syn 以自管 Markdown vault 为知识真相源；官方 Obsidian 仅可按用户动作打开同一份兼容文件。",
      forbidden_text: "知识库资料和知识命中不能绕过候选、正式记忆、来源、版本、审计和权限治理。",
    },
    warnings: [...warnings],
  };
}

function buildKnowledgeMemoryCaptureSummary(event: MemoryCaptureEventRecord): KnowledgeMemoryCaptureSummary {
  return {
    label: memoryCaptureSourceLabel(event.source_type),
    summary: event.summary,
    policy_label: memoryCapturePolicyLabel(event.candidate_policy),
    boundary: event.candidate_key
      ? "已生成候选；仍需确认后才能成为正式记忆。"
      : "只作为捕获/观察来源；不会直接写正式记忆。",
    created_at: event.created_at,
  };
}

function memoryCaptureSourceLabel(sourceType: MemoryCaptureEventRecord["source_type"]): string {
  const labels: Record<MemoryCaptureEventRecord["source_type"], string> = {
    user_action: "用户操作",
    product_command: "产品命令",
    runtime_log: "运行日志",
    readback: "执行读回",
    worker_report: "工作线汇报",
    operation_control_decision: "操作控制决策",
    process_fact_decision: "过程事实确认",
    final_review: "最终复核",
  };
  return labels[sourceType];
}

function memoryCapturePolicyLabel(policy: MemoryCaptureEventRecord["candidate_policy"]): string {
  const labels: Record<MemoryCaptureEventRecord["candidate_policy"], string> = {
    observation_only: "仅观察",
    candidate_allowed: "允许候选",
    audit_only: "仅审计",
    blocked_sensitive: "敏感阻断",
  };
  return labels[policy];
}

function buildKnowledgeDocument({
  project,
  file,
  projectWorkflow,
  formalRecords,
  candidates,
  taskPackages,
  expectedCandidateStoreRevision,
}: {
  project: ProjectRecord;
  file: FileCandidate;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  formalRecords: MemoryRecord[];
  candidates: MemoryCandidate[];
  taskPackages: TaskPackage[];
  expectedCandidateStoreRevision: number | null;
}): KnowledgeDocumentReadModel {
  const projectId = projectWorkflow?.project_id ?? projectIdForProject(project);
  const workflowId = projectWorkflow?.workflow_id ?? null;
  const documentKey = knowledgeDocumentKey(project.project_root, file.path);
  const title = file.name || pathTail(file.path);
  const formalLinks = formalRecords
    .filter((record) => record.source_refs.some((source) => sourceMatchesDocument(source, file, documentKey, title)) || record.scope.document_refs.includes(file.path))
    .map((record) => ({
      kind: "formal_memory" as const,
      label: "正式记忆引用了该知识库来源",
      claim: record.claim,
      status: record.status,
      boundary: "正式记忆仍以 formal store 的来源、版本、审计和权限为准。",
    }));
  const candidateLinks = candidates
    .filter((candidate) => candidate.source_refs.some((source) => sourceMatchesDocument(source, file, documentKey, title)) || candidate.scope.document_refs.includes(file.path))
    .map((candidate) => ({
      kind: "memory_candidate" as const,
      label: "候选引用了该知识库来源",
      claim: candidate.claim,
      status: candidate.status,
      boundary: "候选不是正式记忆；需要确认和受控采纳。",
    }));
  const taskRefs = taskPackages.filter((taskPackage) =>
    taskPackage.available_knowledge_refs.some((ref) => refMatchesDocument(ref, file, documentKey, title)),
  );

  return {
    document_key: documentKey,
    title,
    project_name: project.name,
    project_root: project.project_root,
    project_id: projectId,
    workflow_id: workflowId,
    source_anchor: {
      anchor_label: `知识库来源 / ${title}`,
      path_summary: pathTail(file.path),
      source_kind: file.kind || "authority_file",
      warnings: file.warnings,
    },
    formal_memory_links: formalLinks,
    candidate_links: candidateLinks,
    task_reference_summary: {
      reference_count: taskRefs.length,
      task_goals: taskRefs.map((taskPackage) => taskPackage.task_goal || taskPackage.task_package_id),
      display_text: `任务包知识引用 ${taskRefs.length}`,
    },
    candidate_draft: {
      label: "提出记忆候选",
      input: {
        project_root: project.project_root,
        project_id: projectId,
        workflow_id: workflowId,
        scope: {
          scope_id: `scope:project:${sanitize(projectId)}:knowledge`,
          scope_type: "project",
          project_id: projectId,
          workflow_id: workflowId,
          role_ids: [],
          document_refs: [file.path],
          model_export_policy: "local_only",
          valid_from: new Date().toISOString(),
        },
        memory_type: "project_memory",
        claim: `${title} 可形成待审记忆候选。`,
        body: `来自知识库资料「${title}」的候选。知识库资料本身不是正式记忆，候选仍需确认和受控采纳。`,
        source_refs: [
          {
            source_ref_id: `source:${documentKey}:candidate`,
            source_type: "knowledge_doc",
            source_id: documentKey,
            source_path: file.path,
            source_title: title,
            anchor: `knowledge:${pathTail(file.path)}`,
            captured_at: new Date().toISOString(),
            authority_level: "knowledge_material",
            sensitive_level: "project",
          },
        ],
        generated_by_role: "project_director",
        generated_from: "knowledge_summary",
        risk_level: "low",
        sensitive_level: "project",
        requires_user_confirmation: true,
        review_reason: "从明确知识库资料提出候选；只生成候选，不写正式记忆。",
        expected_store_revision: expectedCandidateStoreRevision,
      },
      boundary: "只生成候选，不写正式记忆",
    },
    boundary: "知识库是材料和笔记空间；正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文。",
  };
}

function collectTaskPackages(workflowState: WorkflowStateSnapshot | null): TaskPackage[] {
  return (workflowState?.project_workflows ?? []).flatMap((projectWorkflow) => projectWorkflow.derived_workflow?.task_packages ?? []);
}

function sourceMatchesDocument(source: MemorySourceRef, file: FileCandidate, documentKey: string, title: string): boolean {
  if (source.source_type !== "knowledge_doc") return false;
  return (
    source.source_path === file.path ||
    source.source_id === documentKey ||
    source.source_title === title ||
    Boolean(source.source_path && source.source_path.endsWith(file.path))
  );
}

function refMatchesDocument(ref: string, file: FileCandidate, documentKey: string, title: string): boolean {
  return ref === file.path || ref === documentKey || ref === title || file.path.endsWith(ref);
}

function knowledgeDocumentKey(projectRoot: string, path: string): string {
  return `knowledge-doc:${sanitize(projectRoot)}:${sanitize(path)}`;
}

function projectIdForProject(project: ProjectRecord): string {
  return `project:${project.project_root.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
}

function pathTail(path: string): string {
  return path.split("/").filter(Boolean).slice(-2).join("/") || path;
}

function sanitize(value: string): string {
  return value.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase() || "unknown";
}
