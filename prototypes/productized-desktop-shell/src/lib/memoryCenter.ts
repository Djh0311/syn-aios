import {
  memoryLintFindingSeverityLabels,
  memoryLintFindingStatusLabels,
  memoryLintFindingTypeLabels,
  memoryStatusLabels,
  observationStatusLabels,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeObservationStore,
  summarizeTaskPackageMemoryInjection,
} from "./candidateGovernance";
import type {
  FormalMemoryStoreV1,
  MaturePatternCandidate,
  MaturePatternPreviewOutput,
  MemoryClusterReport,
  MemoryAuditEvent,
  MemoryCaptureEventRecord,
  MemoryCaptureStoreV1,
  MemoryCandidate,
  MemoryCandidateStoreV1,
  MemoryEntity,
  MemoryEntityCandidate,
  MemoryEntityMergeCandidate,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryLifecycleStatus,
  MemoryLintFinding,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  MemoryMaintenanceReport,
  MemoryRecord,
  MemoryRelation,
  MemoryRelationCandidate,
  MemoryScope,
  MemorySystemAcceptanceSummary,
  MemorySourceRef,
  MemoryVersion,
  ObservationRecord,
  ObservationStoreV1,
  ProjectRecord,
  TaskPackage,
  WorkflowStateSnapshot,
} from "./types";

type BadgeTone = "neutral" | "candidate" | "warning" | "unknown";

export type MemorySourceSummary = {
  label: string;
  authority_label: string;
  sensitive_label: string;
  captured_at: string;
  boundary: string;
};

export type MemoryVersionSummary = {
  version_label: string;
  change_summary: string;
  changed_by: string;
  created_at: string;
};

export type MemoryAuditSummary = {
  event_label: string;
  actor_label: string;
  status_label: string;
  reason: string;
  created_at: string;
};

export type MemoryTaskEligibilitySummary = {
  label: "可进入任务包" | "被检查阻断" | "被排除" | "待审查材料" | "已有采纳回链";
  reason: string;
  included_in_task_package: boolean;
  review_material: boolean;
  badge_tone: BadgeTone;
};

export type MemoryConflictSummary = {
  label: string;
  open_blocking_count: number;
  open_needs_review_count: number;
  finding_summaries: string[];
};

export type FormalMemoryListItem = {
  kind: "formal_memory";
  kind_label: "正式记忆";
  record: MemoryRecord;
  memory_id: string;
  claim: string;
  body: string;
  status_label: string;
  scope_label: string;
  permission_summary: string;
  model_export_summary: string;
  source_summaries: MemorySourceSummary[];
  version_summary: string;
  versions: MemoryVersionSummary[];
  audit_summary: string;
  audits: MemoryAuditSummary[];
  conflict_summary: string;
  conflicts: MemoryConflictSummary;
  task_eligibility: MemoryTaskEligibilitySummary;
  updated_at: string;
};

export type MemoryCandidateListItem = {
  kind: "memory_candidate";
  kind_label: "候选记忆";
  candidate_key: string;
  claim: string;
  body: string;
  status_label: string;
  scope_label: string;
  source_summaries: MemorySourceSummary[];
  risk_summary: string;
  confirmation_summary: string;
  adoption_summary: string;
  formal_memory_boundary: string;
  task_position: MemoryTaskEligibilitySummary;
  lint_summary: string;
  updated_at: string;
};

export type ObservationSourceListItem = {
  kind: "observation_source";
  kind_label: "观察来源";
  summary: string;
  status_label: string;
  source_summary: string;
  boundary: "观察不是正式记忆";
  candidate_link: string;
  updated_at: string;
};

export type MemoryCaptureListItem = {
  kind: "memory_capture";
  kind_label: "捕获";
  summary: string;
  source_label: string;
  policy_label: string;
  observation_link: string;
  candidate_link: string;
  boundary: "捕获事件不是正式记忆";
  updated_at: string;
};

export type MemoryTaskPackageSummary = {
  snapshot_count: number;
  fresh_snapshot_count: number;
  stale_snapshot_count: number;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  display_text: string;
  referenced_memory_ids: string[];
  stale_reasons: string[];
  warnings: string[];
};

export type ProjectMemorySummary = {
  project_name: string;
  formal_count: number;
  candidate_count: number;
  observation_count: number;
  open_lint_count: number;
  task_package_snapshot_count: number;
  display_text: string;
};

export type MemoryWorkbenchActionItem = {
  action_id: string;
  kind:
    | "review_candidate"
    | "confirm_formalization"
    | "repair_capture_link"
    | "review_task_memory_packet"
    | "resolve_memory_blocker";
  title: string;
  summary: string;
  next_step: string;
  source_ref: string;
  badge_tone: BadgeTone;
  updated_at: string;
};

export type MemoryWorkbenchSummary = {
  formal_count: number;
  capture_count: number;
  observation_count: number;
  candidate_count: number;
  pending_candidate_count: number;
  confirmed_pending_formalization_count: number;
  adopted_candidate_count: number;
  capture_compensation_count: number;
  task_package_snapshot_count: number;
  task_package_included_count: number;
  task_package_excluded_count: number;
  task_package_review_material_count: number;
  blocking_finding_count: number;
  needs_review_finding_count: number;
  action_count: number;
  display_text: string;
  task_memory_packet_text: string;
  boundary_text: string;
  action_items: MemoryWorkbenchActionItem[];
  warnings: string[];
};

export type MemoryEntityRelationSummary = {
  sidecar_name: string;
  revision: number;
  entity_count: number;
  entity_candidate_count: number;
  merge_candidate_count: number;
  relation_candidate_count: number;
  confirmed_relation_count: number;
  display_text: string;
  persisted_entities: MemoryEntity[];
  entity_candidates: MemoryEntityCandidate[];
  merge_candidates: MemoryEntityMergeCandidate[];
  relation_candidates: MemoryRelationCandidate[];
  confirmed_relations: MemoryRelation[];
  warnings: string[];
};

export type MemoryMaintenanceSummary = {
  recent_report: MemoryMaintenanceReport | null;
  recent_run_label: string;
  display_text: string;
  index_status_text: string;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  check_summaries: string[];
  recommendation_summaries: string[];
  warnings: string[];
};

export type MaturePatternSummary = {
  sidecar_name: string;
  revision: number;
  mature_pattern_candidate_count: number;
  pending_candidate_count: number;
  confirmed_pattern_count: number;
  rejected_pattern_count: number;
  quarantined_pattern_count: number;
  cluster_report_count: number;
  user_confirmation_required_count: number;
  display_text: string;
  boundary_text: string;
  mature_pattern_candidates: MaturePatternCandidate[];
  cluster_reports: MemoryClusterReport[];
  acceptance_summary: MemorySystemAcceptanceSummary | null;
  warnings: string[];
};

export type MemoryManagementSummary = {
  source_kind: "frontend_read_model";
  boundary: string;
  formal_summary: ReturnType<typeof summarizeFormalMemoryStore>;
  candidate_summary: ReturnType<typeof summarizeMemoryCandidateStore>;
  observation_summary: ReturnType<typeof summarizeObservationStore>;
  lint_summary: ReturnType<typeof summarizeMemoryLintStore>;
  maintenance_summary: MemoryMaintenanceSummary;
  entity_relation_summary: MemoryEntityRelationSummary;
  mature_pattern_summary: MaturePatternSummary;
  memory_workbench_summary: MemoryWorkbenchSummary;
  task_package_summary: MemoryTaskPackageSummary;
  formal_memories: FormalMemoryListItem[];
  candidate_memories: MemoryCandidateListItem[];
  capture_events: MemoryCaptureListItem[];
  observation_sources: ObservationSourceListItem[];
  project_summaries: ProjectMemorySummary[];
  recent_changes: string[];
  warnings: string[];
};

export function deriveMemoryManagementSummary({
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
}: {
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  memoryEntityRelationStore?: MemoryEntityRelationStoreV1 | null;
  memoryEntityRelationPreview?: MemoryEntityRelationPreviewOutput | null;
  memoryPatternStore?: MemoryPatternStoreV1 | null;
  maturePatternPreview?: MaturePatternPreviewOutput | null;
}): MemoryManagementSummary {
  const taskPackages = collectTaskPackages(workflowState);
  const taskPackageSummary = summarizeTaskPackages(taskPackages);
  const lintFindings = memoryLintStore?.findings ?? [];
  const formalRecords = formalMemoryStore?.records ?? [];
  const captureEvents = memoryCaptureStore?.events ?? [];
  const candidates = memoryCandidateStore?.candidates ?? [];
  const observations = observationStore?.observations ?? [];

  const formalMemories = formalRecords.map((record) =>
    buildFormalMemoryItem(record, {
      versions: formalMemoryStore?.versions ?? [],
      auditEvents: formalMemoryStore?.audit_events ?? [],
      lintFindings,
      referencedMemoryIds: new Set(taskPackageSummary.referenced_memory_ids),
    }),
  );

  const candidateMemories = candidates.map((candidate) =>
    buildCandidateItem(candidate, {
      lintFindings,
    }),
  );

  const captureItems = captureEvents.map(buildMemoryCaptureItem);
  const observationSources = observations.map(buildObservationSourceItem);
  const warnings = new Set<string>([
    ...(formalMemoryStore?.warnings ?? []),
    ...(memoryCaptureStore?.warnings ?? []),
    ...(observationStore?.warnings ?? []),
    ...(memoryLintStore?.warnings ?? []),
    ...(memoryEntityRelationStore?.warnings ?? []),
    ...(memoryEntityRelationPreview?.warnings ?? []),
    ...(memoryPatternStore?.warnings ?? []),
    ...(maturePatternPreview?.warnings ?? []),
    ...taskPackageSummary.warnings,
  ]);
  const entityRelationSummary = summarizeEntityRelations(memoryEntityRelationStore ?? null, memoryEntityRelationPreview ?? null);
  const maintenanceSummary = summarizeMaintenance(memoryLintStore ?? null);
  const maturePatternSummary = summarizeMaturePatterns(memoryPatternStore ?? null, maturePatternPreview ?? null);
  const memoryWorkbenchSummary = summarizeMemoryWorkbench({
    formalRecords,
    captureEvents,
    observations,
    candidates,
    lintFindings,
    taskPackageSummary,
  });

  return {
    source_kind: "frontend_read_model",
    boundary: "全局记忆入口展示正式记忆、候选、观察来源、检查、成熟模式候选和任务包冻结快照摘要；正式记忆变更必须走受控生命周期确认。",
    formal_summary: summarizeFormalMemoryStore(formalMemoryStore ?? null),
    candidate_summary: summarizeMemoryCandidateStore(memoryCandidateStore ?? null),
    observation_summary: summarizeObservationStore(observationStore ?? null),
    lint_summary: summarizeMemoryLintStore(memoryLintStore ?? null),
    maintenance_summary: maintenanceSummary,
    entity_relation_summary: entityRelationSummary,
    mature_pattern_summary: maturePatternSummary,
    memory_workbench_summary: memoryWorkbenchSummary,
    task_package_summary: taskPackageSummary,
    formal_memories: formalMemories,
    candidate_memories: candidateMemories,
    capture_events: captureItems,
    observation_sources: observationSources,
    project_summaries: buildProjectSummaries({
      projects,
      taskPackages,
      formalRecords,
      candidates,
      observations,
      lintFindings,
    }),
    recent_changes: recentMemoryChanges({
      versions: formalMemoryStore?.versions ?? [],
      auditEvents: formalMemoryStore?.audit_events ?? [],
      lintFindings,
      maintenanceReports: memoryLintStore?.maintenance_reports ?? [],
      maturePatternCandidates: maturePatternSummary.mature_pattern_candidates,
      candidates,
    }),
    warnings: [...warnings],
  };
}

function summarizeMemoryWorkbench({
  formalRecords,
  captureEvents,
  observations,
  candidates,
  lintFindings,
  taskPackageSummary,
}: {
  formalRecords: MemoryRecord[];
  captureEvents: MemoryCaptureEventRecord[];
  observations: ObservationRecord[];
  candidates: MemoryCandidate[];
  lintFindings: MemoryLintFinding[];
  taskPackageSummary: MemoryTaskPackageSummary;
}): MemoryWorkbenchSummary {
  const pendingCandidates = candidates.filter((candidate) => isPendingCandidateStatus(candidate.status));
  const confirmedPendingFormalization = candidates.filter(
    (candidate) => candidate.status === "candidate_confirmed" && !candidate.adoption,
  );
  const adoptedCandidates = candidates.filter((candidate) => candidate.adoption);
  const captureCompensationEvents = captureEvents.filter(captureNeedsCompensation);
  const openBlockingFindings = lintFindings.filter((finding) => finding.status === "open" && finding.severity === "blocking");
  const openNeedsReviewFindings = lintFindings.filter((finding) => finding.status === "open" && finding.severity === "needs_review");
  const actionItems = buildMemoryWorkbenchActions({
    pendingCandidates,
    confirmedPendingFormalization,
    captureCompensationEvents,
    openBlockingFindings,
    taskPackageSummary,
  });

  return {
    formal_count: formalRecords.length,
    capture_count: captureEvents.length,
    observation_count: observations.length,
    candidate_count: candidates.length,
    pending_candidate_count: pendingCandidates.length,
    confirmed_pending_formalization_count: confirmedPendingFormalization.length,
    adopted_candidate_count: adoptedCandidates.length,
    capture_compensation_count: captureCompensationEvents.length,
    task_package_snapshot_count: taskPackageSummary.snapshot_count,
    task_package_included_count: taskPackageSummary.included_count,
    task_package_excluded_count: taskPackageSummary.excluded_count,
    task_package_review_material_count: taskPackageSummary.review_material_count,
    blocking_finding_count: openBlockingFindings.length,
    needs_review_finding_count: openNeedsReviewFindings.length,
    action_count: actionItems.length,
    display_text:
      `捕获 ${captureEvents.length} / 观察 ${observations.length} / 候选 ${candidates.length} / 正式 ${formalRecords.length}；` +
      `待审候选 ${pendingCandidates.length} / 待正式化 ${confirmedPendingFormalization.length} / 需补证 ${captureCompensationEvents.length}。`,
    task_memory_packet_text:
      taskPackageSummary.snapshot_count > 0
        ? `任务记忆包快照 ${taskPackageSummary.snapshot_count} 个：入选 ${taskPackageSummary.included_count} / 排除 ${taskPackageSummary.excluded_count} / 待审材料 ${taskPackageSummary.review_material_count}。`
        : "任务记忆包快照未生成；发送前只能展示资格判断，不能声称已注入。",
    boundary_text: "观察和候选都不是正式记忆；只有确认后的正式记忆才可能进入任务记忆包入选列表。",
    action_items: actionItems,
    warnings: [
      "memory_workbench_summary_is_read_model_only",
      "candidate_and_observation_are_not_formal_memory",
      "task_memory_packet_included_requires_formal_memory",
    ],
  };
}

function buildMemoryWorkbenchActions({
  pendingCandidates,
  confirmedPendingFormalization,
  captureCompensationEvents,
  openBlockingFindings,
  taskPackageSummary,
}: {
  pendingCandidates: MemoryCandidate[];
  confirmedPendingFormalization: MemoryCandidate[];
  captureCompensationEvents: MemoryCaptureEventRecord[];
  openBlockingFindings: MemoryLintFinding[];
  taskPackageSummary: MemoryTaskPackageSummary;
}): MemoryWorkbenchActionItem[] {
  const actions: MemoryWorkbenchActionItem[] = [
    ...captureCompensationEvents.slice(0, 3).map((event) => ({
      action_id: `repair-capture:${event.capture_event_id}`,
      kind: "repair_capture_link" as const,
      title: "补齐捕获链路",
      summary: `${event.summary}；捕获允许生成候选，但 observation 或 candidate 回链不完整。`,
      next_step: "先补证或人工确认补偿，不自动写正式记忆。",
      source_ref: event.capture_event_id,
      badge_tone: "warning" as const,
      updated_at: event.updated_at,
    })),
    ...pendingCandidates.slice(0, 3).map((candidate) => ({
      action_id: `review-candidate:${candidate.candidate_key}`,
      kind: "review_candidate" as const,
      title: "审查候选记忆",
      summary: `${candidate.claim}；候选仍需确认，不能进入任务包入选列表。`,
      next_step: candidate.requires_user_confirmation ? "由用户确认、拒绝或延后。" : "由授权角色确认、拒绝或延后。",
      source_ref: candidate.candidate_key,
      badge_tone: "warning" as const,
      updated_at: candidate.updated_at,
    })),
    ...confirmedPendingFormalization.slice(0, 3).map((candidate) => ({
      action_id: `formalize-candidate:${candidate.candidate_key}`,
      kind: "confirm_formalization" as const,
      title: "确认正式化",
      summary: `${candidate.claim}；候选已确认保留，但还没有正式记忆采纳回链。`,
      next_step: "继续走正式记忆生命周期、版本和审计确认。",
      source_ref: candidate.candidate_key,
      badge_tone: "candidate" as const,
      updated_at: candidate.updated_at,
    })),
    ...openBlockingFindings.slice(0, 2).map((finding) => ({
      action_id: `resolve-memory-blocker:${finding.finding_id}`,
      kind: "resolve_memory_blocker" as const,
      title: "处理记忆阻断",
      summary: finding.summary,
      next_step: "先处理阻断发现；阻断未关闭前不得进入任务包入选列表。",
      source_ref: finding.finding_id,
      badge_tone: "warning" as const,
      updated_at: finding.updated_at,
    })),
  ];

  if (taskPackageSummary.review_material_count > 0 || taskPackageSummary.stale_snapshot_count > 0) {
    actions.push({
      action_id: "review-task-memory-packet",
      kind: "review_task_memory_packet",
      title: "查看任务记忆包",
      summary: taskPackageSummary.display_text,
      next_step: "确认入选、排除和待审材料；候选 / 观察只能作为待审材料。",
      source_ref: taskPackageSummary.snapshot_count ? "task-memory-packet-snapshots" : "task-memory-packet-eligibility",
      badge_tone: taskPackageSummary.stale_snapshot_count > 0 ? "warning" : "neutral",
      updated_at: "derived",
    });
  }

  return actions.sort((left, right) => right.updated_at.localeCompare(left.updated_at));
}

function isPendingCandidateStatus(status: MemoryLifecycleStatus): boolean {
  return status === "candidate_draft" || status === "candidate_needs_review";
}

function captureNeedsCompensation(event: MemoryCaptureEventRecord): boolean {
  if (event.candidate_policy === "candidate_allowed") {
    return !event.observation_id || !event.candidate_key;
  }
  if (event.candidate_policy === "observation_only") {
    return !event.observation_id;
  }
  return false;
}

function summarizeEntityRelations(
  store: MemoryEntityRelationStoreV1 | null,
  preview: MemoryEntityRelationPreviewOutput | null,
): MemoryEntityRelationSummary {
  const persistedEntities = store?.registry.entities ?? [];
  const persistedEntityCandidates = store?.entity_candidates ?? [];
  const persistedMergeCandidates = store?.merge_candidates ?? [];
  const persistedRelationCandidates = store?.relation_candidates ?? [];
  const confirmedRelations = (store?.relations ?? []).filter((relation) => relation.status === "confirmed");
  const entityCandidates = preview?.entity_candidates ?? persistedEntityCandidates.filter((candidate) => candidate.status === "candidate");
  const mergeCandidates = preview?.merge_candidates ?? persistedMergeCandidates.filter((candidate) => candidate.status === "candidate");
  const relationCandidates = preview?.relation_candidates ?? persistedRelationCandidates.filter((candidate) => candidate.status === "candidate");
  const warnings = new Set<string>([
    ...(store?.warnings ?? []),
    ...(store?.registry.warnings ?? []),
    ...(preview?.warnings ?? []),
    "llm_inferred_relation_candidate_only",
    "similarity_hit_candidate_only",
  ]);

  return {
    sidecar_name: "memory-entity-relations.v1.json",
    revision: preview?.store_revision ?? store?.revision ?? 0,
    entity_count: persistedEntities.length,
    entity_candidate_count: entityCandidates.length,
    merge_candidate_count: mergeCandidates.length,
    relation_candidate_count: relationCandidates.length,
    confirmed_relation_count: confirmedRelations.length,
    display_text: `实体候选 ${entityCandidates.length} / dedupe 候选 ${mergeCandidates.length} / 关系候选 ${relationCandidates.length} / 已确认关系 ${confirmedRelations.length}；LLM 推断仅作候选，相似度命中仅作候选。`,
    persisted_entities: persistedEntities,
    entity_candidates: entityCandidates,
    merge_candidates: mergeCandidates,
    relation_candidates: relationCandidates,
    confirmed_relations: confirmedRelations,
    warnings: [...warnings],
  };
}

function buildFormalMemoryItem(
  record: MemoryRecord,
  {
    versions,
    auditEvents,
    lintFindings,
    referencedMemoryIds,
  }: {
    versions: MemoryVersion[];
    auditEvents: MemoryAuditEvent[];
    lintFindings: MemoryLintFinding[];
    referencedMemoryIds: Set<string>;
  },
): FormalMemoryListItem {
  const versionsForRecord = versions
    .filter((version) => version.memory_id === record.memory_id)
    .sort((left, right) => right.version_number - left.version_number);
  const auditsForRecord = auditEvents
    .filter((event) => event.target_id === record.memory_id)
    .sort((left, right) => right.created_at.localeCompare(left.created_at));
  const findings = lintFindings.filter((finding) => finding.target_memory_id === record.memory_id || finding.source_id === record.memory_id);
  const conflictSummary = summarizeConflicts(record.conflict_refs, findings);
  const latestVersion = versionsForRecord[0] ?? null;
  const latestAudit = auditsForRecord[0] ?? null;

  return {
    kind: "formal_memory",
    kind_label: "正式记忆",
    record,
    memory_id: record.memory_id,
    claim: record.claim,
    body: record.body,
    status_label: formalStatusLabel(record.status),
    scope_label: scopeLabel(record.scope),
    permission_summary: permissionSummary(record.scope),
    model_export_summary: `外发 ${record.scope.model_export_policy}`,
    source_summaries: record.source_refs.map(sourceSummary),
    version_summary: latestVersion ? `版本 v${latestVersion.version_number} / ${latestVersion.change_type} / ${latestVersion.change_summary}` : "版本未记录",
    versions: versionsForRecord.map((version) => ({
      version_label: `版本 v${version.version_number}`,
      change_summary: `${version.change_type} / ${version.change_summary}`,
      changed_by: version.changed_by_role,
      created_at: version.created_at,
    })),
    audit_summary: latestAudit ? `${latestAudit.event_type} / ${latestAudit.actor_role} / ${latestAudit.status}` : "审计未记录",
    audits: auditsForRecord.map((event) => ({
      event_label: event.event_type,
      actor_label: event.actor_role,
      status_label: event.status,
      reason: event.reason,
      created_at: event.created_at,
    })),
    conflict_summary: conflictSummary.label,
    conflicts: conflictSummary,
    task_eligibility: formalTaskEligibility({
      record,
      conflictSummary,
      includedInTaskPackage: referencedMemoryIds.has(record.memory_id),
    }),
    updated_at: record.updated_at,
  };
}

function buildCandidateItem(
  candidate: MemoryCandidate,
  {
    lintFindings,
  }: {
    lintFindings: MemoryLintFinding[];
  },
): MemoryCandidateListItem {
  const findings = lintFindings.filter(
    (finding) => finding.target_candidate_key === candidate.candidate_key || finding.source_id === candidate.candidate_key,
  );
  const openFindings = findings.filter((finding) => finding.status === "open");

  return {
    kind: "memory_candidate",
    kind_label: "候选记忆",
    candidate_key: candidate.candidate_key,
    claim: candidate.claim,
    body: candidate.body,
    status_label: candidateStatusLabel(candidate.status),
    scope_label: scopeLabel(candidate.scope),
    source_summaries: candidate.source_refs.map(sourceSummary),
    risk_summary: `风险 ${candidate.risk_level} / 敏感 ${candidate.sensitive_level}`,
    confirmation_summary: candidate.requires_user_confirmation ? `需要用户确认：${candidate.review_reason}` : `无需额外用户确认：${candidate.review_reason}`,
    adoption_summary: candidate.adoption
      ? `候选已被受控采纳；采纳角色 ${candidate.adoption.adopted_by_role}；${candidate.adoption.adoption_reason}`
      : "尚无采纳回链",
    formal_memory_boundary: `${candidateStatusLabel(candidate.status)}；不是正式记忆。`,
    task_position: candidate.adoption
      ? {
          label: "已有采纳回链",
          reason: "候选行保留候选身份；正式条目以 formal store 为准。",
          included_in_task_package: false,
          review_material: false,
          badge_tone: "candidate",
        }
      : {
          label: "待审查材料",
          reason: "候选不是正式记忆；不会进入任务包入选列表。",
          included_in_task_package: false,
          review_material: true,
          badge_tone: "warning",
        },
    lint_summary: openFindings.length
      ? openFindings.map((finding) => `${memoryLintFindingSeverityLabels[finding.severity]} ${finding.summary}`).join("；")
      : "暂无未关闭检查发现",
    updated_at: candidate.updated_at,
  };
}

function buildMemoryCaptureItem(event: MemoryCaptureEventRecord): MemoryCaptureListItem {
  return {
    kind: "memory_capture",
    kind_label: "捕获",
    summary: event.summary,
    source_label: memoryCaptureSourceLabel(event.source_type),
    policy_label: memoryCapturePolicyLabel(event.candidate_policy),
    observation_link: event.observation_id ? "已形成观察" : "未形成观察",
    candidate_link: event.candidate_key ? "已形成候选；候选仍需确认" : "未形成候选",
    boundary: "捕获事件不是正式记忆",
    updated_at: event.updated_at,
  };
}

function buildObservationSourceItem(observation: ObservationRecord): ObservationSourceListItem {
  return {
    kind: "observation_source",
    kind_label: "观察来源",
    summary: observation.summary,
    status_label: observationStatusLabels[observation.status] ?? observation.status,
    source_summary: observation.source_refs
      .map((source) => `${source.source_kind} / ${source.summary} / ${source.sensitive_level}`)
      .join("；") || "来源未记录",
    boundary: "观察不是正式记忆",
    candidate_link: observation.candidate_key ? `已形成候选引用；候选仍需确认 / 采纳` : "未形成候选引用",
    updated_at: observation.updated_at,
  };
}

function formalTaskEligibility({
  record,
  conflictSummary,
  includedInTaskPackage,
}: {
  record: MemoryRecord;
  conflictSummary: MemoryConflictSummary;
  includedInTaskPackage: boolean;
}): MemoryTaskEligibilitySummary {
  if (conflictSummary.open_blocking_count > 0) {
    return {
      label: "被检查阻断",
      reason: `未关闭阻断发现 ${conflictSummary.open_blocking_count} 条；不会进入任务包入选列表。`,
      included_in_task_package: includedInTaskPackage,
      review_material: false,
      badge_tone: "warning",
    };
  }

  if (record.status !== "memory_active") {
    return {
      label: "被排除",
      reason: `${formalStatusLabel(record.status)} 不是活跃正式记忆。`,
      included_in_task_package: includedInTaskPackage,
      review_material: false,
      badge_tone: "unknown",
    };
  }

  if (record.scope.model_export_policy === "blocked") {
    return {
      label: "被排除",
      reason: "外发策略 blocked；任务包不得把它交给模型上下文。",
      included_in_task_package: includedInTaskPackage,
      review_material: false,
      badge_tone: "warning",
    };
  }

  return {
    label: "可进入任务包",
    reason: includedInTaskPackage
      ? "任务包冻结快照已引用；活跃正式记忆且没有未关闭阻断发现。"
      : "活跃正式记忆且没有未关闭阻断发现；当前快照未显示逐条引用。",
    included_in_task_package: includedInTaskPackage,
    review_material: false,
    badge_tone: "candidate",
  };
}

function summarizeConflicts(conflictRefs: string[], findings: MemoryLintFinding[]): MemoryConflictSummary {
  const openFindings = findings.filter((finding) => finding.status === "open");
  const openBlocking = openFindings.filter((finding) => finding.severity === "blocking");
  const openNeedsReview = openFindings.filter((finding) => finding.severity === "needs_review");
  const findingSummaries = openFindings.map(
    (finding) =>
      `${memoryLintFindingTypeLabels[finding.finding_type] ?? finding.finding_type} / ${memoryLintFindingSeverityLabels[finding.severity]} / ${memoryLintFindingStatusLabels[finding.status]} / ${finding.summary}`,
  );

  if (openBlocking.length || openNeedsReview.length) {
    return {
      label: `未关闭阻断 ${openBlocking.length} / 待复核 ${openNeedsReview.length}`,
      open_blocking_count: openBlocking.length,
      open_needs_review_count: openNeedsReview.length,
      finding_summaries: findingSummaries,
    };
  }

  if (conflictRefs.length) {
    return {
      label: `冲突引用 ${conflictRefs.length} 条；无未关闭阻断发现`,
      open_blocking_count: 0,
      open_needs_review_count: 0,
      finding_summaries: [],
    };
  }

  return {
    label: "暂无未关闭检查冲突",
    open_blocking_count: 0,
    open_needs_review_count: 0,
    finding_summaries: [],
  };
}

function summarizeMaintenance(store: MemoryLintStoreV1 | null): MemoryMaintenanceSummary {
  const reports = store?.maintenance_reports ?? [];
  const recentReport = reports.at(-1) ?? null;
  const recentRun = store?.runs.at(-1) ?? null;
  const openFindings = (store?.findings ?? []).filter((finding) => finding.status === "open");
  const blockingCount = recentReport?.blocking_count ?? openFindings.filter((finding) => finding.severity === "blocking").length;
  const needsReviewCount = recentReport?.needs_review_count ?? openFindings.filter((finding) => finding.severity === "needs_review").length;
  const infoCount = recentReport?.info_count ?? openFindings.filter((finding) => finding.severity === "info").length;
  const warnings = new Set<string>([
    ...(store?.warnings ?? []),
    ...(recentReport?.warnings ?? []),
    ...(recentReport?.index_status.warnings ?? []),
  ]);

  return {
    recent_report: recentReport,
    recent_run_label: recentRun ? `${recentRun.lint_intent} / ${recentRun.status} / ${recentRun.created_at}` : "尚未运行维护任务",
    display_text:
      recentReport?.display_text ??
      `维护任务尚未生成报告；当前 open finding ${openFindings.length} / blocking ${blockingCount} / needs_review ${needsReviewCount} / info ${infoCount}。维护任务只生成 finding，不会自动修改正式记忆。`,
    index_status_text: recentReport?.index_status.display_text ?? "索引状态尚未由维护任务检查；不会伪装为健康。",
    blocking_count: blockingCount,
    needs_review_count: needsReviewCount,
    info_count: infoCount,
    check_summaries: recentReport?.check_summaries.map((check) => check.display_text) ?? [],
    recommendation_summaries: recentReport?.recommendations.map((recommendation) => recommendation.display_text) ?? [],
    warnings: [...warnings],
  };
}

function summarizeMaturePatterns(
  store: MemoryPatternStoreV1 | null,
  preview: MaturePatternPreviewOutput | null,
): MaturePatternSummary {
  const candidates = preview?.mature_pattern_candidates ?? store?.mature_pattern_candidates ?? [];
  const clusterReports = preview?.cluster_reports ?? store?.cluster_reports ?? [];
  const pendingCandidates = candidates.filter((candidate) => candidate.status === "candidate");
  const confirmedCandidates = candidates.filter((candidate) => candidate.status === "confirmed");
  const rejectedCandidates = candidates.filter((candidate) => candidate.status === "rejected");
  const quarantinedCandidates = candidates.filter((candidate) => candidate.status === "quarantined");
  const userConfirmationRequired = candidates.filter(
    (candidate) => candidate.requires_user_confirmation && candidate.status === "candidate",
  );
  const warnings = new Set<string>([
    ...(store?.warnings ?? []),
    ...(preview?.warnings ?? []),
    ...candidates.flatMap((candidate) => candidate.warnings),
    ...clusterReports.flatMap((report) => report.warnings),
  ]);
  const previewSummary =
    preview?.summary.display_text ??
    `成熟模式候选 ${candidates.length} / 待用户确认 ${userConfirmationRequired.length} / 已确认 ${confirmedCandidates.length} / 跨项目主题报告 ${clusterReports.length}。`;

  return {
    sidecar_name: preview?.summary.sidecar_name ?? "memory-patterns.v1.json",
    revision: preview?.store_revision ?? store?.revision ?? 0,
    mature_pattern_candidate_count: candidates.length,
    pending_candidate_count: pendingCandidates.length,
    confirmed_pattern_count: confirmedCandidates.length,
    rejected_pattern_count: rejectedCandidates.length,
    quarantined_pattern_count: quarantinedCandidates.length,
    cluster_report_count: clusterReports.length,
    user_confirmation_required_count: userConfirmationRequired.length,
    display_text: previewSummary,
    boundary_text:
      "成熟模式候选未确认，不会进入任务包；跨项目主题报告不是正式事实；用户确认后才可通过正式记忆受控写入。",
    mature_pattern_candidates: candidates,
    cluster_reports: clusterReports,
    acceptance_summary: preview?.acceptance_summary ?? null,
    warnings: [...warnings],
  };
}

function summarizeTaskPackages(taskPackages: TaskPackage[]): MemoryTaskPackageSummary {
  const summaries = taskPackages
    .map((taskPackage) => summarizeTaskPackageMemoryInjection(taskPackage.memory_injection_summary))
    .filter((summary) => summary.snapshot_id || summary.included_count || summary.excluded_count || summary.review_material_count);
  const referencedMemoryIds = new Set<string>();
  const staleReasons = new Set<string>();
  const warnings = new Set<string>();
  let includedCount = 0;
  let excludedCount = 0;
  let reviewMaterialCount = 0;
  let freshSnapshotCount = 0;
  let staleSnapshotCount = 0;

  for (const taskPackage of taskPackages) {
    for (const memoryId of taskPackage.available_memory_refs) {
      referencedMemoryIds.add(memoryId);
    }
  }

  for (const summary of summaries) {
    includedCount += summary.included_count;
    excludedCount += summary.excluded_count;
    reviewMaterialCount += summary.review_material_count;
    if (summary.stale) {
      staleSnapshotCount += 1;
    } else {
      freshSnapshotCount += 1;
    }
    for (const reason of summary.stale_reasons) staleReasons.add(reason);
    for (const warning of summary.warnings) warnings.add(warning);
  }

  return {
    snapshot_count: summaries.length,
    fresh_snapshot_count: freshSnapshotCount,
    stale_snapshot_count: staleSnapshotCount,
    included_count: includedCount,
    excluded_count: excludedCount,
    review_material_count: reviewMaterialCount,
    display_text:
      summaries.length > 0
        ? `任务包冻结快照 ${summaries.length} 个 / 新鲜 ${freshSnapshotCount} / 过期 ${staleSnapshotCount} / 入选 ${includedCount} / 排除 ${excludedCount} / 待审材料 ${reviewMaterialCount}`
        : "任务包冻结快照未生成；当前只能显示资格判断，不能声称已注入。",
    referenced_memory_ids: [...referencedMemoryIds],
    stale_reasons: [...staleReasons],
    warnings: [...warnings],
  };
}

function buildProjectSummaries({
  projects,
  taskPackages,
  formalRecords,
  candidates,
  observations,
  lintFindings,
}: {
  projects: ProjectRecord[];
  taskPackages: TaskPackage[];
  formalRecords: MemoryRecord[];
  candidates: MemoryCandidate[];
  observations: ObservationRecord[];
  lintFindings: MemoryLintFinding[];
}): ProjectMemorySummary[] {
  return projects.slice(0, 8).map((project) => {
    const projectId = projectIdForProject(project);
    const formalCount = formalRecords.filter((record) => belongsToProject(record.scope, projectId)).length;
    const candidateCount = candidates.filter((candidate) => belongsToProject(candidate.scope, projectId)).length;
    const observationCount = observations.filter((observation) => belongsToProject(observation.scope, projectId)).length;
    const taskPackageSnapshotCount = taskPackages.filter((taskPackage) => taskPackage.project_id === projectId && taskPackage.memory_injection_summary).length;
    const openLintCount = lintFindings.filter((finding) => finding.status === "open" && (!finding.scope_type || finding.scope_type === "project")).length;

    return {
      project_name: project.name,
      formal_count: formalCount,
      candidate_count: candidateCount,
      observation_count: observationCount,
      open_lint_count: openLintCount,
      task_package_snapshot_count: taskPackageSnapshotCount,
      display_text: `正式 ${formalCount} / 候选 ${candidateCount} / 观察来源 ${observationCount} / 未关闭检查 ${openLintCount} / 任务包冻结快照 ${taskPackageSnapshotCount}`,
    };
  });
}

function recentMemoryChanges({
  versions,
  auditEvents,
  lintFindings,
  maintenanceReports,
  maturePatternCandidates,
  candidates,
}: {
  versions: MemoryVersion[];
  auditEvents: MemoryAuditEvent[];
  lintFindings: MemoryLintFinding[];
  maintenanceReports: MemoryMaintenanceReport[];
  maturePatternCandidates: MaturePatternCandidate[];
  candidates: MemoryCandidate[];
}): string[] {
  const changes = [
    ...versions.map((version) => ({ at: version.created_at, text: `版本 v${version.version_number}：${version.change_summary}` })),
    ...auditEvents.map((event) => ({ at: event.created_at, text: `审计 ${event.event_type}：${event.reason}` })),
    ...lintFindings.map((finding) => ({ at: finding.updated_at, text: `检查 ${memoryLintFindingTypeLabels[finding.finding_type] ?? finding.finding_type}：${finding.summary}` })),
    ...maintenanceReports.map((report) => ({ at: report.created_at, text: `维护报告：${report.display_text}` })),
    ...maturePatternCandidates.map((candidate) => ({ at: candidate.updated_at, text: `成熟模式候选 ${candidate.status}：${candidate.title}` })),
    ...candidates.map((candidate) => ({ at: candidate.updated_at, text: `候选 ${candidateStatusLabel(candidate.status)}：${candidate.claim}` })),
  ];
  return changes
    .sort((left, right) => right.at.localeCompare(left.at))
    .slice(0, 6)
    .map((item) => item.text);
}

function collectTaskPackages(workflowState: WorkflowStateSnapshot | null): TaskPackage[] {
  return (workflowState?.project_workflows ?? []).flatMap((projectWorkflow) => projectWorkflow.derived_workflow?.task_packages ?? []);
}

function sourceSummary(source: MemorySourceRef): MemorySourceSummary {
  if (source.source_type === "knowledge_doc") {
    return {
      label: `来自知识库资料：${source.source_title || source.source_id || "未命名资料"}`,
      authority_label: source.authority_level,
      sensitive_label: source.sensitive_level,
      captured_at: source.captured_at,
      boundary: "知识库资料来源已记录；资料本身不是正式记忆。",
    };
  }

  return {
    label: source.source_title || source.source_type,
    authority_label: source.authority_level,
    sensitive_label: source.sensitive_level,
    captured_at: source.captured_at,
    boundary: source.source_path ? "路径来源已记录；普通 UI 不展示原始路径。" : "来源摘要来自 store 字段。",
  };
}

function scopeLabel(scope: MemoryScope): string {
  const rolePart = scope.role_ids.length ? ` / role ${scope.role_ids.length}` : "";
  const docPart = scope.document_refs.length ? ` / 文档限定 ${scope.document_refs.length}` : "";
  if (scope.scope_type === "project") return `项目范围${rolePart}${docPart}`;
  if (scope.scope_type === "workflow") return `工作流范围${rolePart}${docPart}`;
  if (scope.scope_type === "session") return `会话范围${rolePart}${docPart}`;
  if (scope.scope_type === "user_preference") return `用户偏好范围${rolePart}${docPart}`;
  if (scope.scope_type === "global") return `全局范围${rolePart}${docPart}`;
  return `${scope.scope_type}${rolePart}${docPart}`;
}

function permissionSummary(scope: MemoryScope): string {
  return scope.permission_policy_ref ? `权限 ${scope.permission_policy_ref}` : "权限策略未记录";
}

function formalStatusLabel(status: MemoryLifecycleStatus): string {
  if (status === "memory_active") return "活跃正式记忆";
  if (status === "memory_conflicted") return "正式记忆状态 conflicted";
  if (status === "memory_deprecated") return "正式记忆状态 deprecated";
  if (status === "memory_frozen") return "正式记忆状态 frozen";
  if (status === "memory_archived") return "正式记忆状态 archived";
  return memoryStatusLabels[status] ?? status;
}

function candidateStatusLabel(status: MemoryLifecycleStatus): string {
  return memoryStatusLabels[status] ?? status;
}

function memoryCaptureSourceLabel(sourceType: MemoryCaptureEventRecord["source_type"]): string {
  if (sourceType === "user_action") return "用户操作";
  if (sourceType === "product_command") return "产品命令";
  if (sourceType === "runtime_log") return "运行日志";
  if (sourceType === "readback") return "读回";
  if (sourceType === "worker_report") return "工作者汇报";
  if (sourceType === "process_fact_decision") return "过程事实确认";
  if (sourceType === "final_review") return "最终复核";
  return sourceType;
}

function memoryCapturePolicyLabel(policy: MemoryCaptureEventRecord["candidate_policy"]): string {
  if (policy === "observation_only") return "只形成观察";
  if (policy === "candidate_allowed") return "允许生成候选";
  if (policy === "audit_only") return "仅审计";
  if (policy === "blocked_sensitive") return "敏感阻断";
  return policy;
}

function belongsToProject(scope: MemoryScope, projectId: string): boolean {
  return scope.project_id === projectId || scope.scope_type === "global" || scope.scope_type === "user_preference";
}

function projectIdForProject(project: ProjectRecord): string {
  return `project:${project.project_root.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
}
