import { useMemo } from "react";
import { Badge } from "../../components/Badge";
import {
  blackboardStateLabels,
  buildBlackboardCandidateOverlay,
  memoryStatusLabels,
  memoryLintFindingSeverityLabels,
  memoryLintFindingStatusLabels,
  memoryLintFindingTypeLabels,
  observationStatusLabels,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeObservationStore,
  summarizeTaskPackageMemoryInjection,
  summarizeTaskMemoryPacketPreview,
  taskMemoryPacketReasonLabels,
} from "../../lib/candidateGovernance";
import type {
  BlackboardCandidateState,
  MemoryCandidateStoreV1,
  MemoryLifecycleStatus,
  MemoryLintStoreV1,
  ObservationStoreV1,
  PendingAction,
  ProjectBlackboard,
  ProjectRecord,
  TaskMemoryPacketBuildOutput,
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { DetailLine } from "./projectWorkflowLabels";

export function ProjectBlackboardPanel({ blackboard }: { blackboard: ProjectBlackboard | null }) {
  const entries = blackboard?.entries ?? [];
  return (
    <section className="workflow-ledger-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目黑板</p>
          <h3>中间态 / 候选</h3>
        </div>
        <Badge tone={entries.length ? "warning" : "candidate"}>{entries.length}</Badge>
      </div>
      <div className="workflow-compact-list">
        {entries.slice(0, 8).map((entry) => (
          <div className="workflow-compact-item" key={entry.entry_id}>
            <strong>{blackboardKindLabel(entry.kind)}</strong>
            <span>{entry.title}：{entry.summary}</span>
          </div>
        ))}
      </div>
      {blackboard?.warnings.map((warning) => (
        <p className="muted small-note" key={warning}>{warning}</p>
      ))}
      {!entries.length ? <p className="muted small-note">暂无黑板候选；黑板不会补编正式事实。</p> : null}
    </section>
  );
}

export function CandidateGovernanceStrip({
  project,
  projectWorkflow,
  selectedTaskPackage,
  blackboard,
  blackboardOverlay,
  observationSummary,
  observationStoreRevision,
  observations,
  memorySummary,
  formalSummary,
  memoryLintSummary,
  memoryLintFindings,
  blackboardStoreRevision,
  memoryStoreRevision,
  memoryCandidates,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  selectedTaskPackage: TaskPackage | null;
  blackboard: ProjectBlackboard | null;
  blackboardOverlay: ReturnType<typeof buildBlackboardCandidateOverlay>;
  observationSummary: ReturnType<typeof summarizeObservationStore>;
  observationStoreRevision: number;
  observations: ObservationStoreV1["observations"];
  memorySummary: ReturnType<typeof summarizeMemoryCandidateStore>;
  formalSummary: ReturnType<typeof summarizeFormalMemoryStore>;
  memoryLintSummary: ReturnType<typeof summarizeMemoryLintStore>;
  memoryLintFindings: MemoryLintStoreV1["findings"];
  blackboardStoreRevision: number;
  memoryStoreRevision: number;
  memoryCandidates: MemoryCandidateStoreV1["candidates"];
  taskMemoryPacketPreview: TaskMemoryPacketBuildOutput | null;
  taskMemoryPacketLoading: boolean;
  taskMemoryPacketError: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const entries = blackboard?.entries ?? [];
  const firstPendingEntry = entries.find((entry) => entry.promotion_decision.status === "candidate_pending_control_core") ?? entries[0] ?? null;
  const firstRecordedObservation = observations.find((observation) => observation.status === "recorded") ?? null;
  const firstMemoryCandidate = memoryCandidates.find((candidate) => candidate.status === "candidate_needs_review") ?? memoryCandidates[0] ?? null;
  const firstAdoptableMemoryCandidate = memoryCandidates.find((candidate) => candidate.status === "candidate_confirmed" && !candidate.adoption) ?? null;
  const taskMemoryPacketSummary = useMemo(
    () => summarizeTaskMemoryPacketPreview(taskMemoryPacketPreview),
    [taskMemoryPacketPreview],
  );
  const taskPackageMemorySummary = useMemo(
    () => summarizeTaskPackageMemoryInjection(selectedTaskPackage?.memory_injection_summary),
    [selectedTaskPackage?.memory_injection_summary],
  );
  const visibleLintFindings = memoryLintFindings.slice(0, 4);
  return (
    <section className="project-canvas-detail-card project-candidate-governance-card" aria-label="候选治理详情">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">候选治理</p>
          <h3>黑板候选 / 记忆候选</h3>
        </div>
        <Badge tone={blackboardOverlay.confirmed_count || memorySummary.confirmed_count ? "candidate" : "unknown"}>
          {blackboardOverlay.revision}/{memorySummary.revision}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="黑板状态" value={`待处理 ${entries.length} / 已确认后续 ${blackboardOverlay.confirmed_count} / 已拒绝 ${blackboardOverlay.rejected_count}`} />
        <DetailLine label="工作流观察" value={observationSummary.display_text} />
        <DetailLine label="记忆候选" value={memorySummary.display_text} />
        <DetailLine label="正式记忆骨架" value={formalSummary.display_text} />
        <DetailLine label="记忆 lint 阻断摘要" value={memoryLintSummary.display_text} />
        <DetailLine label="任务包记忆注入摘要" value={taskPackageMemorySummary.display_text} />
        <DetailLine label="任务记忆包预览" value={taskMemoryPacketSummary.display_text} />
        <DetailLine label="预览排除理由" value={taskMemoryPacketSummary.reason_text} />
      </div>
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="workflow-draft-grid">
          <DetailLine label="黑板 sidecar" value={blackboardOverlay.sidecar_name} />
          <DetailLine label="观察辅助状态文件" value={observationSummary.sidecar_name} />
          <DetailLine label="最近观察审计" value={observationSummary.recent_audit_event?.event_type ?? "暂无"} />
          <DetailLine label="最近观察候选" value={observationSummary.recent_candidate_key ?? "暂无"} />
          <DetailLine label="记忆 sidecar" value={memorySummary.sidecar_name} />
          <DetailLine label="adopted_memory_id" value={memorySummary.first_adoption?.adopted_memory_id ?? "暂无"} />
          <DetailLine label="adopted_version_id" value={memorySummary.first_adoption?.adopted_version_id ?? "暂无"} />
          <DetailLine label="adopted_audit_event_id" value={memorySummary.first_adoption?.adopted_audit_event_id ?? "暂无"} />
          <DetailLine label="正式记忆 sidecar" value={formalSummary.sidecar_name} />
          <DetailLine label="最近正式记忆审计" value={formalSummary.recent_audit_event?.event_type ?? "暂无"} />
          <DetailLine label="记忆 lint sidecar" value={memoryLintSummary.sidecar_name} />
          <DetailLine label="最近检查运行" value={memoryLintSummary.recent_run ? `${memoryLintSummary.recent_run.status} / ${memoryLintSummary.recent_run.reason}` : "暂无"} />
          <DetailLine label="任务包记忆快照" value={taskPackageMemorySummary.snapshot_id ? `${taskPackageMemorySummary.snapshot_id} / ${taskPackageMemorySummary.stale ? "过期" : "新鲜"}` : "未生成"} />
        </div>
      </details>
      {/* C·折：黑板摘要 / 观察 / memory lint / 注入摘要 / 任务包记忆预览 等次要块默认收起。
          KEEP 的「记忆候选 + 正式记忆骨架」在上方 grid 常驻；这些块文案被离线断言可见，折不删。 */}
      <details className="candidate-governance-secondary-fold">
        <summary>
          <span>记忆治理·更多</span>
          <em>注入摘要 / memory lint / 任务记忆包预览（默认收起）</em>
        </summary>
      <div className="workflow-compact-list" aria-label="任务包记忆注入摘要">
        <div className="workflow-compact-item">
          <strong>任务包记忆注入摘要</strong>
          <span>{taskPackageMemorySummary.display_text}</span>
          <em>仅启用态正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。</em>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="入选正式记忆" value={String(taskPackageMemorySummary.included_count)} />
          <DetailLine label="排除项" value={String(taskPackageMemorySummary.excluded_count)} />
          <DetailLine label="待审查材料" value={String(taskPackageMemorySummary.review_material_count)} />
          <DetailLine label="快照状态" value={taskPackageMemorySummary.stale ? "过期" : "新鲜"} />
        </div>
        {taskPackageMemorySummary.stale_reasons.slice(0, 3).map((reason) => (
          <p className="state-warning" key={reason}>{reason}</p>
        ))}
        {taskPackageMemorySummary.warnings.slice(0, 3).map((warning) => (
          <p className="muted small-note" key={warning}>{warning}</p>
        ))}
      </div>
      <div className="workflow-compact-list" aria-label="记忆检查发现摘要">
        <div className="workflow-compact-item">
          <strong>记忆 lint 阻断摘要 / rev {memoryLintSummary.revision}</strong>
          <span>{memoryLintSummary.display_text}</span>
          <em>阻断级发现会阻止进入任务包；检查只生成待处理发现；不会自动修改正式记忆。</em>
        </div>
        {visibleLintFindings.map((finding) => (
          <div className="workflow-compact-item" key={finding.finding_id}>
            <strong>{memoryLintFindingTypeLabels[finding.finding_type] ?? finding.finding_type}</strong>
            <span>
              {memoryLintFindingSeverityLabels[finding.severity] ?? finding.severity} / {memoryLintFindingStatusLabels[finding.status] ?? finding.status}
            </span>
            <em>{finding.summary}</em>
          </div>
        ))}
        {!visibleLintFindings.length ? <p className="muted small-note">暂无检查发现；阻断级发现会阻止进入任务包。</p> : null}
      </div>
      <TaskMemoryPacketPreviewPanel
        output={taskMemoryPacketPreview}
        summary={taskMemoryPacketSummary}
        loading={taskMemoryPacketLoading}
        error={taskMemoryPacketError}
      />
      </details>
      <div className="workflow-state-actions">
        {firstPendingEntry ? (
          <>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_confirmed_for_followup", blackboardStoreRevision))}
            >
              确认黑板候选后续处理
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_rejected", blackboardStoreRevision))}
            >
              拒绝黑板候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_deferred", blackboardStoreRevision))}
            >
              暂缓黑板候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(blackboardDecisionAction(project, projectWorkflow, firstPendingEntry, "candidate_discarded", blackboardStoreRevision))}
            >
              废弃黑板候选
            </button>
          </>
        ) : null}
        {firstRecordedObservation ? (
          <button
            className="secondary-button"
            type="button"
            onClick={() =>
              onRequestAction(
                observationCandidateAction(
                  project,
                  firstRecordedObservation,
                  observationStoreRevision,
                  memoryStoreRevision,
                ),
              )
            }
          >
            从工作流观察生成候选
          </button>
        ) : null}
        {firstMemoryCandidate ? (
          <>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_confirmed", memoryStoreRevision))}
            >
              确认记忆候选保留
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_quarantined", memoryStoreRevision))}
            >
              隔离记忆候选
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() => onRequestAction(memoryDecisionAction(project, firstMemoryCandidate.candidate_key, "candidate_discarded", memoryStoreRevision))}
            >
              废弃记忆候选
            </button>
          </>
        ) : null}
        {firstAdoptableMemoryCandidate ? (
          <button
            className="secondary-button"
            type="button"
            onClick={() => onRequestAction(memoryAdoptionAction(project, firstAdoptableMemoryCandidate.candidate_key, memoryStoreRevision, formalSummary.revision))}
          >
            受控采纳为正式记忆
          </button>
        ) : null}
      </div>
      <p className="muted small-note">工作流观察只记录明确事件和来源；观察可生成候选，候选仍需确认 / 采纳；观察不是正式记忆。</p>
      <p className="muted small-note">候选确认只写候选辅助状态文件；不写正式事实、不写正式长期记忆、不推进工作流状态。</p>
      <p className="muted small-note">受控正式记忆读取 formal-memories.v1.json；创建时写入版本和审计；候选采纳需走受控动作；任务包记忆注入使用生成时冻结快照。</p>
      <p className="muted small-note">检查只生成待处理发现；阻断级发现会阻止进入任务包；不会自动修改正式记忆。</p>
      {observationSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      {formalSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
      {memoryLintSummary.warnings.slice(0, 3).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </section>
  );
}

function TaskMemoryPacketPreviewPanel({
  output,
  summary,
  loading,
  error,
}: {
  output: TaskMemoryPacketBuildOutput | null;
  summary: ReturnType<typeof summarizeTaskMemoryPacketPreview>;
  loading: boolean;
  error: string | null;
}) {
  const preview = output?.preview ?? null;
  const excludedItems = preview?.excluded_items.slice(0, 5) ?? [];
  const reviewMaterials = preview?.review_materials.slice(0, 5) ?? [];
  return (
    <div className="workflow-compact-list task-memory-packet-preview" aria-label="任务记忆包预览">
      <div className="workflow-compact-item">
        <strong>任务记忆包预览</strong>
        <span>{summary.display_text}</span>
        <em>预览未注入任务包；仅启用态正式记忆可入选；候选 / 观察仅作为待审查材料。</em>
        <details className="agent-boundary-details">
          <summary className="agent-boundary-summary">开发者详情</summary>
          <em>packet_id：{summary.packet_id ?? "未生成"}</em>
        </details>
      </div>
      {loading ? (
        <p className="muted small-note">正在生成任务记忆包预览。</p>
      ) : null}
      {error ? (
        <p className="state-warning">任务记忆包预览读取失败：{error}</p>
      ) : null}
      {preview ? (
        <>
          <div className="workflow-draft-grid">
            <DetailLine label="入选正式记忆" value={String(summary.included_count)} />
            <DetailLine label="排除项" value={String(summary.excluded_count)} />
            <DetailLine label="待审查材料" value={String(summary.review_material_count)} />
            <DetailLine label="估算 token" value={`${summary.estimated_tokens}/${summary.max_estimated_tokens}`} />
          </div>
          {excludedItems.map((item) => (
            <div className="workflow-compact-item" key={`${item.source_kind}:${item.source_id}:${item.reason}`}>
              <strong>{taskMemoryPacketReasonLabels[item.reason]}</strong>
              <span>{item.claim ?? item.source_id}</span>
              <em>{item.detail}</em>
              <details className="agent-boundary-details">
                <summary className="agent-boundary-summary">开发者详情</summary>
                <em>source_kind：{item.source_kind} / reason：{item.reason} / source_id：{item.source_id}</em>
              </details>
            </div>
          ))}
          {reviewMaterials.map((item) => (
            <div className="workflow-compact-item" key={`${item.source_kind}:${item.source_id}:${item.reason}`}>
              <strong>待审查材料</strong>
              <span>{item.title}</span>
              <em>{taskMemoryPacketReasonLabels[item.reason]}；不进入正式记忆列表。</em>
              <details className="agent-boundary-details">
                <summary className="agent-boundary-summary">开发者详情</summary>
                <em>source_kind：{item.source_kind} / reason：{item.reason} / source_id：{item.source_id}</em>
              </details>
            </div>
          ))}
          {summary.warnings.slice(0, 4).map((warning) => (
            <p className="muted small-note" key={warning}>{warning}</p>
          ))}
        </>
      ) : (
        <p className="muted small-note">没有后端预览结果时只显示空摘要；不会用前端模拟数据伪装后端能力。</p>
      )}
    </div>
  );
}

function blackboardKindLabel(kind: ProjectBlackboard["entries"][number]["kind"]) {
  if (kind === "subagent_report") return "子智能体汇报";
  if (kind === "risk") return "风险";
  if (kind === "permission_request") return "权限请求";
  if (kind === "tool_summary") return "工具摘要";
  if (kind === "memory_candidate") return "记忆候选";
  if (kind === "knowledge_ref") return "知识引用";
  return kind;
}

function blackboardDecisionAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null,
  entry: ProjectBlackboard["entries"][number],
  requestedState: BlackboardCandidateState,
  expectedStoreRevision: number,
): PendingAction {
  const labelByState: Record<BlackboardCandidateState, string> = {
    candidate_pending_control_core: "重新打开黑板候选",
    candidate_confirmed_for_followup: "确认黑板候选后续处理",
    candidate_rejected: "拒绝黑板候选",
    candidate_deferred: "暂缓黑板候选",
    candidate_discarded: "废弃黑板候选",
  };
  return {
    kind: "record-blackboard-candidate-decision",
    label: labelByState[requestedState],
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只写 blackboard-candidates.v1.json 候选辅助状态文件；不写正式事实、不写正式记忆、不批准权限、不推进工作流状态。",
    blackboardCandidateDecision: {
      project_id: entry.project_id,
      project_root: project.project_root,
      workflow_id: entry.workflow_id,
      source_entry_id: entry.entry_id,
      entry_kind: entry.kind,
      target_kind: blackboardTargetKind(entry.promotion_decision.target_kind),
      requested_state: requestedState,
      reason: `${blackboardStateLabels[requestedState]}：候选层处理，不做正式晋升。`,
      actor_role: "project_director",
      actor_session_id: null,
      source_refs: entry.source_refs,
      expected_store_revision: expectedStoreRevision,
      title_snapshot: entry.title,
      summary_snapshot: entry.summary,
      source_status: entry.source_status ?? entry.status,
      work_item_id: entry.work_item_id ?? projectWorkflow?.task_drafts[0]?.work_item_id ?? null,
      workflow_node_id: entry.workflow_node_id ?? null,
    },
  };
}

function observationCandidateAction(
  project: ProjectRecord,
  observation: ObservationStoreV1["observations"][number],
  expectedObservationStoreRevision: number,
  expectedCandidateStoreRevision: number,
): PendingAction {
  const memoryType =
    observation.scope.scope_type === "session"
      ? "session_summary"
      : observation.scope.scope_type === "workflow"
        ? "workflow_summary"
        : "project_memory";
  return {
    kind: "create-memory-candidate-from-observation",
    label: "从工作流观察生成记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只从已记录观察生成 memory-candidates.v1.json 待审候选，并在 observations.v1.json 回链 candidate_key；不写正式记忆、不推进工作流状态、不注入任务包。",
    observationCandidateCreation: {
      project_root: project.project_root,
      observation_key: observation.observation_key,
      actor_id: "project_director",
      actor_role: "project_director",
      memory_type: memoryType,
      claim: `观察结论：${observation.summary}`,
      body: observation.summary,
      review_reason: `${observationStatusLabels[observation.status]}；项目主管确认观察可生成候选，候选仍需确认 / 采纳。`,
      requires_user_confirmation: observation.risk_level === "high" || observation.sensitive_level === "secret",
      expected_observation_store_revision: expectedObservationStoreRevision,
      expected_candidate_store_revision: expectedCandidateStoreRevision,
    },
  };
}

function memoryDecisionAction(
  project: ProjectRecord,
  candidateKey: string,
  requestedStatus: Extract<MemoryLifecycleStatus, "candidate_confirmed" | "candidate_rejected" | "candidate_quarantined" | "candidate_discarded">,
  expectedStoreRevision: number,
): PendingAction {
  return {
    kind: "record-memory-candidate-decision",
    label: memoryStatusLabels[requestedStatus] ?? "处理记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只写 memory-candidates.v1.json 候选 sidecar；candidate_confirmed 只表示确认保留候选，不写正式长期记忆。",
    memoryCandidateDecision: {
      project_root: project.project_root,
      candidate_key: candidateKey,
      requested_status: requestedStatus,
      reason: `${memoryStatusLabels[requestedStatus] ?? requestedStatus}；不写正式长期记忆。`,
      actor_id: "project_director",
      actor_role: "project_director",
      expected_store_revision: expectedStoreRevision,
    },
  };
}

function memoryAdoptionAction(
  project: ProjectRecord,
  candidateKey: string,
  expectedCandidateStoreRevision: number,
  expectedFormalStoreRevision: number,
): PendingAction {
  return {
    kind: "adopt-memory-candidate-to-formal-memory",
    label: "受控采纳记忆候选",
    path: project.project_root,
    source: "Tauri 应用数据目录",
    boundary:
      "只允许已确认候选经控制核心采纳；写 formal-memories.v1.json，并在 memory-candidates.v1.json 保留采纳回链；不推进工作流状态、不做任务包注入。",
    memoryCandidateAdoption: {
      project_root: project.project_root,
      candidate_key: candidateKey,
      actor_id: "project_director",
      actor_role: "project_director",
      adoption_reason: "项目主管采纳低风险本项目记忆候选。",
      expected_candidate_store_revision: expectedCandidateStoreRevision,
      expected_formal_store_revision: expectedFormalStoreRevision,
    },
  };
}

function blackboardTargetKind(targetKind?: string | null) {
  if (targetKind === "workflow_fact") return "workflow_fact";
  if (targetKind === "workflow_risk") return "workflow_risk";
  if (targetKind === "permission_decision") return "permission_decision";
  if (targetKind === "audit_event") return "audit_event";
  if (targetKind === "formal_memory") return "formal_memory";
  if (targetKind === "knowledge_reference") return "knowledge_reference";
  return "no_promotion";
}
