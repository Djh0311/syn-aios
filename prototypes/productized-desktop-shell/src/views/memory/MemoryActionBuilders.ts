import type { FormalMemoryListItem } from "../../lib/memoryCenter";
import type {
  FormalMemoryLifecycleOperationKind,
  FormalMemoryLifecyclePreview,
  FormalMemoryLifecyclePreviewInput,
  MemoryRecord,
  MemoryScope,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { operationLabel } from "./MemoryDetailPanels";

export function buildLifecycleRequest({
  item,
  operationKind,
  allFormalMemories,
  projects,
  workflowState,
  storeRevision,
}: {
  item: FormalMemoryListItem;
  operationKind: FormalMemoryLifecycleOperationKind;
  allFormalMemories: FormalMemoryListItem[];
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  storeRevision: number | null;
}): FormalMemoryLifecyclePreviewInput {
  const context = lifecycleContextForRecord(item.record, projects, workflowState);
  const expectedRecordVersions: Record<string, number> = {
    [item.memory_id]: item.record.record_version,
  };
  const base: FormalMemoryLifecyclePreviewInput = {
    project_root: context.projectRoot,
    project_id: context.projectId,
    workflow_id: context.workflowId,
    operation_kind: operationKind,
    memory_id: item.memory_id,
    memory_ids: [],
    revise: null,
    merge: null,
    split: null,
    scope_change: null,
    actor_id: "project-director-ui",
    actor_role: "project_director",
    reason: lifecycleReason(operationKind, item),
    expected_store_revision: storeRevision,
    expected_record_versions: expectedRecordVersions,
  };

  if (operationKind === "revise") {
    return {
      ...base,
      revise: {
        claim: item.record.claim,
        body: item.record.body,
        source_refs: item.record.source_refs,
      },
    };
  }

  if (operationKind === "merge") {
    const mergePeer = allFormalMemories.find((candidate) => candidate.memory_id !== item.memory_id && candidate.record.status === "memory_active");
    if (!mergePeer) {
      throw new Error("合并需要另一条活跃正式记忆作为明确选择对象。");
    }
    expectedRecordVersions[mergePeer.memory_id] = mergePeer.record.record_version;
    return {
      ...base,
      memory_id: null,
      memory_ids: [item.memory_id, mergePeer.memory_id],
      expected_record_versions: expectedRecordVersions,
      merge: {
        source_memory_ids: [item.memory_id, mergePeer.memory_id],
        target_memory_id: null,
        merged_claim: `合并记录：${item.record.claim}`,
        merged_body: `${item.record.body}\n\n来源记录：${mergePeer.record.claim}\n${mergePeer.record.body}`,
        memory_type: item.record.memory_type,
        scope: item.record.scope,
        source_refs: uniqueSourceRefs([...item.record.source_refs, ...mergePeer.record.source_refs]),
      },
    };
  }

  if (operationKind === "split") {
    return {
      ...base,
      split: {
        source_memory_id: item.memory_id,
        split_records: [
          {
            claim: `${item.record.claim} / A`,
            body: `${item.record.body}\n\n拆分草稿 A。`,
            memory_type: item.record.memory_type,
            scope: item.record.scope,
            source_refs: item.record.source_refs,
          },
          {
            claim: `${item.record.claim} / B`,
            body: `${item.record.body}\n\n拆分草稿 B。`,
            memory_type: item.record.memory_type,
            scope: item.record.scope,
            source_refs: item.record.source_refs,
          },
        ],
      },
    };
  }

  if (operationKind === "promote_to_global" || operationKind === "demote_to_project") {
    return {
      ...base,
      scope_change: {
        target_scope:
          operationKind === "promote_to_global"
            ? globalScopeForRecord(item.record)
            : projectScopeForContext(item.record, context),
        applicability:
          operationKind === "promote_to_global"
            ? "用户确认后适用于全局范围；跨项目成熟模式仍留待后续阶段。"
            : "用户确认后下沉回当前项目范围。",
      },
    };
  }

  return base;
}

export function confirmationForOperation(
  operationKind: FormalMemoryLifecycleOperationKind,
  preview: FormalMemoryLifecyclePreview,
): { confirmedBy: string; summary: string } {
  const userRequired = preview.required_approval.required_actor_role === "user";
  return {
    confirmedBy: userRequired ? "user" : "project-director-ui",
    summary: `${operationLabel(operationKind)} 已查看影响面：${preview.impact.display_text}`,
  };
}

export function primaryProjectRoot(projects: ProjectRecord[]): string | null {
  return projects.find((project) => project.active_hint)?.project_root ?? projects[0]?.project_root ?? null;
}

export function maturePatternDecisionLabel(decision: string): string {
  if (decision === "confirm_as_formal_memory") return "用户确认成熟模式候选";
  if (decision === "reject") return "拒绝成熟模式候选";
  if (decision === "quarantine") return "隔离成熟模式候选";
  return "要求补充成熟模式候选来源";
}

export function maturePatternDecisionReason(candidate: { title: string }, decision: string): string {
  if (decision === "confirm_as_formal_memory") {
    return `用户确认成熟模式候选：${candidate.title}；通过正式记忆受控路径写入。`;
  }
  if (decision === "reject") return `用户拒绝成熟模式候选：${candidate.title}；保留来源材料。`;
  if (decision === "quarantine") return `用户隔离成熟模式候选：${candidate.title}；保留来源材料。`;
  return `用户要求补充成熟模式候选来源：${candidate.title}。`;
}

export function projectIdForProject(project: ProjectRecord): string {
  return `project:${stableId(project.project_root)}`;
}

export function defaultWorkflowId(projectRoot: string): string {
  return `workflow:${stableId(projectRoot)}:default`;
}

function lifecycleContextForRecord(
  record: MemoryRecord,
  projects: ProjectRecord[],
  workflowState: WorkflowStateSnapshot | null,
): { projectRoot: string; projectId: string; workflowId: string } {
  const project =
    projects.find((candidate) => projectIdForProject(candidate) === record.scope.project_id) ??
    projects[0];
  if (!project) {
    throw new Error("缺少可用于生命周期上下文绑定的项目。");
  }
  const projectId = projectIdForProject(project);
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_id === projectId) ?? null;
  return {
    projectRoot: project.project_root,
    projectId,
    workflowId: projectWorkflow?.workflow_id ?? defaultWorkflowId(project.project_root),
  };
}

function globalScopeForRecord(record: MemoryRecord): MemoryScope {
  return {
    ...record.scope,
    scope_id: `scope:global:${sanitize(record.memory_id)}`,
    scope_type: "global",
    project_id: null,
    workflow_id: null,
    session_id: null,
    valid_from: new Date().toISOString(),
  };
}

function projectScopeForContext(
  record: MemoryRecord,
  context: { projectRoot: string; projectId: string; workflowId: string },
): MemoryScope {
  return {
    ...record.scope,
    scope_id: `scope:project:${sanitize(context.projectId)}:${sanitize(record.memory_id)}`,
    scope_type: "project",
    project_id: context.projectId,
    workflow_id: null,
    session_id: null,
    valid_from: new Date().toISOString(),
  };
}

function lifecycleReason(operationKind: FormalMemoryLifecycleOperationKind, item: FormalMemoryListItem): string {
  return `${operationLabel(operationKind)} 正式记忆：${item.memory_id}；编辑会创建新版本，不覆盖旧版本。`;
}

function uniqueSourceRefs(sources: MemoryRecord["source_refs"]) {
  const seen = new Set<string>();
  return sources.filter((source) => {
    const key = `${source.source_ref_id}:${source.source_type}:${source.source_id ?? ""}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function stableId(value: string): string {
  return sanitize(value).slice(0, 96);
}

function sanitize(value: string): string {
  return value.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase() || "unknown";
}
