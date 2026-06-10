import type {
  BlackboardCandidateState,
  BlackboardCandidateStoreV1,
  BlackboardEntry,
  FormalMemoryStoreV1,
  MemoryAuditEvent,
  MemoryCandidateAdoptionRef,
  MemoryCandidateStoreV1,
  MemoryLifecycleStatus,
  MemoryLintFindingSeverity,
  MemoryLintFindingStatus,
  MemoryLintFindingType,
  MemoryLintStoreV1,
  ObservationStatus,
  ObservationStoreV1,
  TaskPackageMemoryInjectionSummary,
  TaskMemoryPacketBuildOutput,
  TaskMemoryPacketExclusionReason,
} from "./types";

export type BlackboardCandidateOverlay = {
  sidecar_name: "blackboard-candidates.v1.json";
  revision: number;
  status_by_entry_id: Record<string, BlackboardCandidateState>;
  status_by_candidate_key: Record<string, BlackboardCandidateState>;
  confirmed_count: number;
  rejected_count: number;
  deferred_count: number;
  discarded_count: number;
  warnings: string[];
};

export function buildBlackboardCandidateOverlay({
  store,
  entries,
}: {
  store: BlackboardCandidateStoreV1 | null;
  entries: BlackboardEntry[];
}): BlackboardCandidateOverlay {
  const statusByEntryId: Record<string, BlackboardCandidateState> = {};
  const statusByCandidateKey: Record<string, BlackboardCandidateState> = {};
  const warnings = new Set<string>(store?.warnings ?? []);

  for (const record of store?.records ?? []) {
    statusByCandidateKey[record.candidate_key] = record.state;
    if (record.source_entry_id) {
      statusByEntryId[record.source_entry_id] = record.state;
    }
    if (record.target_kind === "formal_memory") {
      warnings.add("formal_memory_target_is_candidate_only");
    }
  }

  for (const entry of entries) {
    if (entry.promotion_decision.status in blackboardStateLabels && !statusByEntryId[entry.entry_id]) {
      statusByEntryId[entry.entry_id] = entry.promotion_decision.status as BlackboardCandidateState;
    }
  }

  const states = Object.values(statusByCandidateKey);
  return {
    sidecar_name: "blackboard-candidates.v1.json",
    revision: store?.revision ?? 0,
    status_by_entry_id: statusByEntryId,
    status_by_candidate_key: statusByCandidateKey,
    confirmed_count: states.filter((state) => state === "candidate_confirmed_for_followup").length,
    rejected_count: states.filter((state) => state === "candidate_rejected").length,
    deferred_count: states.filter((state) => state === "candidate_deferred").length,
    discarded_count: states.filter((state) => state === "candidate_discarded").length,
    warnings: [...warnings],
  };
}

export type MemoryCandidateSummary = {
  sidecar_name: "memory-candidates.v1.json";
  revision: number;
  candidate_count: number;
  needs_review_count: number;
  confirmed_count: number;
  rejected_count: number;
  quarantined_count: number;
  discarded_count: number;
  formal_memory_count: number;
  adopted_count: number;
  first_adoption?: MemoryCandidateAdoptionRef | null;
  display_text: string;
  warnings: string[];
};

export function summarizeMemoryCandidateStore(store: MemoryCandidateStoreV1 | null): MemoryCandidateSummary {
  const candidates = store?.candidates ?? [];
  const statuses = candidates.map((candidate) => candidate.status);
  const formalMemoryCount = statuses.filter((status) => status.startsWith("memory_")).length;
  const adoptions = candidates.map((candidate) => candidate.adoption).filter((adoption): adoption is MemoryCandidateAdoptionRef => Boolean(adoption));
  const warnings = new Set<string>();
  if (formalMemoryCount > 0) {
    warnings.add("formal_memory_status_should_not_appear_in_candidate_store");
  }
  const confirmedCount = countStatus(statuses, "candidate_confirmed");
  const needsReviewCount = countStatus(statuses, "candidate_needs_review") + countStatus(statuses, "candidate_draft");
  const rejectedCount = countStatus(statuses, "candidate_rejected");
  const quarantinedCount = countStatus(statuses, "candidate_quarantined");
  const discardedCount = countStatus(statuses, "candidate_discarded");

  return {
    sidecar_name: "memory-candidates.v1.json",
    revision: store?.revision ?? 0,
    candidate_count: statuses.length,
    needs_review_count: needsReviewCount,
    confirmed_count: confirmedCount,
    rejected_count: rejectedCount,
    quarantined_count: quarantinedCount,
    discarded_count: discardedCount,
    formal_memory_count: formalMemoryCount,
    adopted_count: adoptions.length,
    first_adoption: adoptions[0] ?? null,
    display_text: [
      `记忆候选待审 ${needsReviewCount}`,
      `记忆候选已确认保留 ${confirmedCount}`,
      `记忆候选受控采纳 ${adoptions.length}`,
      `记忆候选已隔离 ${quarantinedCount}`,
      `记忆候选已废弃 ${discardedCount}`,
      `记忆候选已拒绝 ${rejectedCount}`,
    ].join(" / "),
    warnings: [...warnings],
  };
}

export type FormalMemorySummary = {
  sidecar_name: "formal-memories.v1.json";
  revision: number;
  record_count: number;
  active_count: number;
  non_active_count: number;
  version_count: number;
  audit_event_count: number;
  recent_audit_event?: MemoryAuditEvent | null;
  display_text: string;
  warnings: string[];
};

export function summarizeFormalMemoryStore(store: FormalMemoryStoreV1 | null): FormalMemorySummary {
  const records = store?.records ?? [];
  const activeCount = records.filter((record) => record.status === "memory_active").length;
  const nonActiveCount = records.length - activeCount;
  const versionCount = store?.versions.length ?? 0;
  const auditEventCount = store?.audit_events.length ?? 0;
  const recentAuditEvent = store?.audit_events.at(-1) ?? null;
  const hasAdoptionEvent = (store?.audit_events ?? []).some((event) => event.event_type === "memory_candidate_adopted_to_formal_memory");
  const warnings = new Set(store?.warnings ?? []);
  const candidateStatusCount = records.filter((record) => record.status.startsWith("candidate_")).length;
  if (candidateStatusCount > 0) {
    warnings.add("candidate_status_should_not_appear_in_formal_memory_store");
  }
  return {
    sidecar_name: "formal-memories.v1.json",
    revision: store?.revision ?? 0,
    record_count: records.length,
    active_count: activeCount,
    non_active_count: nonActiveCount,
    version_count: versionCount,
    audit_event_count: auditEventCount,
    recent_audit_event: recentAuditEvent,
    display_text: [
      `受控正式记忆 ${records.length}`,
      `memory_active ${activeCount}`,
      `version ${versionCount}`,
      `audit ${auditEventCount}`,
      "创建时写入 version 和 audit",
      hasAdoptionEvent ? "候选受控采纳审计已记录" : "候选采纳需走单独受控动作",
      "任务包注入使用冻结只读快照",
    ].join(" / "),
    warnings: [...warnings],
  };
}

export type MemoryLintSummary = {
  sidecar_name: "memory-lint.v1.json";
  revision: number;
  finding_count: number;
  open_count: number;
  blocking_count: number;
  needs_review_count: number;
  info_count: number;
  recent_run?: MemoryLintStoreV1["runs"][number] | null;
  recent_maintenance_report?: NonNullable<MemoryLintStoreV1["maintenance_reports"]>[number] | null;
  display_text: string;
  warnings: string[];
};

export function summarizeMemoryLintStore(store: MemoryLintStoreV1 | null): MemoryLintSummary {
  const findings = store?.findings ?? [];
  const openFindings = findings.filter((finding) => finding.status === "open");
  const blockingCount = openFindings.filter((finding) => finding.severity === "blocking").length;
  const needsReviewCount = openFindings.filter((finding) => finding.severity === "needs_review").length;
  const infoCount = openFindings.filter((finding) => finding.severity === "info").length;
  const recentMaintenanceReport = store?.maintenance_reports?.at(-1) ?? null;
  const warnings = new Set(store?.warnings ?? []);
  if (blockingCount > 0) {
    warnings.add("memory_lint_open_blocking_findings_present");
  }
  return {
    sidecar_name: "memory-lint.v1.json",
    revision: store?.revision ?? 0,
    finding_count: findings.length,
    open_count: openFindings.length,
    blocking_count: blockingCount,
    needs_review_count: needsReviewCount,
    info_count: infoCount,
    recent_run: store?.runs.at(-1) ?? null,
    recent_maintenance_report: recentMaintenanceReport,
    display_text: [
      recentMaintenanceReport ? `维护任务 ${recentMaintenanceReport.open_count} open finding` : `记忆 lint 阻断摘要 open ${openFindings.length}`,
      `blocking ${blockingCount}`,
      `needs_review ${needsReviewCount}`,
      `info ${infoCount}`,
      "blocking finding 会阻止进入任务包",
      "lint 只生成待处理 finding",
      "维护任务只生成 finding",
      "不会自动修改正式记忆",
    ].join(" / "),
    warnings: [...warnings],
  };
}

export type ObservationSummary = {
  sidecar_name: "observations.v1.json";
  revision: number;
  observation_count: number;
  recorded_count: number;
  candidate_created_count: number;
  ignored_count: number;
  quarantined_count: number;
  recent_audit_event?: ObservationStoreV1["events"][number] | null;
  recent_candidate_key?: string | null;
  display_text: string;
  warnings: string[];
};

export function summarizeObservationStore(store: ObservationStoreV1 | null): ObservationSummary {
  const observations = store?.observations ?? [];
  const statuses = observations.map((observation) => observation.status);
  const recordedCount = countObservationStatus(statuses, "recorded");
  const candidateCreatedCount = countObservationStatus(statuses, "candidate_created");
  const ignoredCount = countObservationStatus(statuses, "ignored");
  const quarantinedCount = countObservationStatus(statuses, "quarantined");
  const warnings = new Set(store?.warnings ?? []);
  if (observations.some((observation) => observation.candidate_key && observation.status !== "candidate_created")) {
    warnings.add("observation_candidate_link_status_mismatch");
  }
  return {
    sidecar_name: "observations.v1.json",
    revision: store?.revision ?? 0,
    observation_count: observations.length,
    recorded_count: recordedCount,
    candidate_created_count: candidateCreatedCount,
    ignored_count: ignoredCount,
    quarantined_count: quarantinedCount,
    recent_audit_event: store?.events.at(-1) ?? null,
    recent_candidate_key: [...observations].reverse().find((observation) => observation.candidate_key)?.candidate_key ?? null,
    display_text: [
      `工作流观察 ${observations.length}`,
      `recorded ${recordedCount}`,
      `candidate_created ${candidateCreatedCount}`,
      `ignored ${ignoredCount}`,
      `quarantined ${quarantinedCount}`,
      "observation 不是正式记忆",
    ].join(" / "),
    warnings: [...warnings],
  };
}

export type TaskMemoryPacketPreviewSummary = {
  sidecar_name: "task_memory_packet_preview";
  packet_id?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  estimated_tokens: number;
  max_estimated_tokens: number;
  reason_counts: Partial<Record<TaskMemoryPacketExclusionReason, number>>;
  display_text: string;
  reason_text: string;
  warnings: string[];
};

export function summarizeTaskMemoryPacketPreview(output: TaskMemoryPacketBuildOutput | null): TaskMemoryPacketPreviewSummary {
  const preview = output?.preview ?? null;
  const reasonCounts: Partial<Record<TaskMemoryPacketExclusionReason, number>> = {};
  for (const item of preview?.excluded_items ?? []) {
    reasonCounts[item.reason] = (reasonCounts[item.reason] ?? 0) + 1;
  }
  const reasonText = Object.entries(reasonCounts)
    .map(([reason, count]) => `${reason} ${count}`)
    .join(" / ");
  const warnings = new Set([...(output?.warnings ?? []), ...(preview?.warnings ?? [])]);
  return {
    sidecar_name: "task_memory_packet_preview",
    packet_id: preview?.packet_id ?? null,
    included_count: preview?.included_memories.length ?? 0,
    excluded_count: preview?.excluded_items.length ?? 0,
    review_material_count: preview?.review_materials.length ?? 0,
    estimated_tokens: preview?.estimated_tokens ?? 0,
    max_estimated_tokens: preview?.max_estimated_tokens ?? 0,
    reason_counts: reasonCounts,
    display_text: preview
      ? [
          `入选 ${preview.included_memories.length}`,
          `排除 ${preview.excluded_items.length}`,
          `待审查材料 ${preview.review_materials.length}`,
          `估算 token ${preview.estimated_tokens}/${preview.max_estimated_tokens}`,
          "预览未注入任务包",
        ].join(" / ")
      : "任务记忆包预览未生成 / 预览未注入任务包",
    reason_text: reasonText || "暂无排除理由",
    warnings: [...warnings],
  };
}

export function summarizeTaskPackageMemoryInjection(
  summary: TaskPackageMemoryInjectionSummary | null | undefined,
): TaskPackageMemoryInjectionSummary {
  if (summary) {
    return {
      ...summary,
      snapshot_id: summary.snapshot_id ?? null,
      stale_reasons: summary.stale_reasons ?? [],
      warnings: summary.warnings ?? [],
      display_text:
        summary.display_text ||
        [
          `任务包记忆注入摘要：入选 ${summary.included_count}`,
          `排除 ${summary.excluded_count}`,
          `待审查材料 ${summary.review_material_count}`,
          summary.stale ? "快照过期" : "快照新鲜",
          "仅活跃正式记忆可进入任务包",
          "候选 / 观察仅作为待审查材料",
          "任务包内容不会回灌成正式记忆",
        ].join(" / "),
    };
  }
  return {
    snapshot_id: null,
    included_count: 0,
    excluded_count: 0,
    review_material_count: 0,
    stale: true,
    stale_reasons: ["task_memory_packet_snapshot_missing"],
    display_text:
      "任务包记忆注入摘要：尚未生成任务包记忆快照。仅活跃正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。",
    warnings: ["task_memory_packet_snapshot_missing"],
  };
}

export const blackboardStateLabels: Record<BlackboardCandidateState, string> = {
  candidate_pending_control_core: "黑板候选待处理",
  candidate_confirmed_for_followup: "黑板候选已确认后续处理",
  candidate_rejected: "黑板候选已拒绝",
  candidate_deferred: "黑板候选已暂缓",
  candidate_discarded: "黑板候选已废弃",
};

export const observationStatusLabels: Record<ObservationStatus, string> = {
  recorded: "观察可生成候选",
  candidate_created: "观察已生成候选",
  ignored: "观察已忽略",
  quarantined: "观察已隔离",
};

export const memoryStatusLabels: Partial<Record<MemoryLifecycleStatus, string>> = {
  candidate_draft: "记忆候选草稿",
  candidate_needs_review: "记忆候选待审",
  candidate_confirmed: "记忆候选已确认保留",
  candidate_rejected: "记忆候选已拒绝",
  candidate_quarantined: "记忆候选已隔离",
  candidate_superseded: "记忆候选已替代",
  candidate_discarded: "记忆候选已废弃",
};

export const memoryLintFindingTypeLabels: Record<MemoryLintFindingType, string> = {
  duplicate_claim: "重复 claim",
  claim_conflict: "claim 冲突",
  source_permission_revoked: "来源权限撤回",
  authority_superseded: "权威来源替代",
  stale_memory: "疑似过期记忆",
  missing_source: "缺少来源",
  candidate_conflicts_with_active_memory: "候选与 active 记忆冲突",
  entity_drift: "实体漂移",
  relation_source_revoked: "关系来源撤回",
  sensitive_export_risk: "私密外发风险",
  private_source_risk: "私密来源风险",
  derived_index_stale: "派生索引状态",
  mature_pattern_signal: "成熟模式信号",
};

export const memoryLintFindingSeverityLabels: Record<MemoryLintFindingSeverity, string> = {
  blocking: "blocking",
  needs_review: "needs_review",
  info: "info",
};

export const memoryLintFindingStatusLabels: Record<MemoryLintFindingStatus, string> = {
  open: "open",
  acknowledged: "acknowledged",
  resolved: "resolved",
  dismissed: "dismissed",
};

export const taskMemoryPacketReasonLabels: Record<TaskMemoryPacketExclusionReason, string> = {
  candidate_unconfirmed: "候选未确认",
  permission_blocked: "权限或上下文不匹配",
  conflicted: "冲突记忆",
  stale: "过期或冻结归档",
  model_export_blocked: "模型外发受阻",
  token_limit: "token 预算超限",
  not_relevant: "与任务目标无确定性命中",
  status_not_active: "正式记忆状态不是 active",
  observation_not_formal_memory: "观察不是正式记忆",
  knowledge_hit_not_formal_memory: "知识命中不是正式记忆",
  llm_summary_not_formal_memory: "LLM 摘要不是正式记忆",
};

function countStatus(statuses: MemoryLifecycleStatus[], status: MemoryLifecycleStatus) {
  return statuses.filter((item) => item === status).length;
}

function countObservationStatus(statuses: ObservationStatus[], status: ObservationStatus) {
  return statuses.filter((item) => item === status).length;
}
