import { useMemo, useState } from "react";
import { Badge } from "../components/Badge";
import { DailyMemoryCandidateInbox } from "../components/DailyMemoryCandidateInbox";
import { deriveMemoryManagementSummary, type FormalMemoryListItem } from "../lib/memoryCenter";
import {
  buildAdoptMemoryCandidateAction,
  buildDailyMemoryCandidateDecisionAction,
  deriveDailyMemoryCandidateInbox,
} from "../lib/memoryDailyLoop";
import { deriveMemoryCenterPageReadModelFromParts } from "../lib/pageSelectors";
import type {
  FormalMemoryLifecycleInput,
  FormalMemoryLifecycleOperationKind,
  FormalMemoryLifecyclePreview,
  FormalMemoryLifecyclePreviewInput,
  FormalMemoryStoreV1,
  MaturePatternCandidate,
  MaturePatternDecisionKind,
  MaturePatternPreviewOutput,
  MemoryCaptureStoreV1,
  MemoryCandidateStoreV1,
  MemoryEntityCandidate,
  MemoryEntityMergeCandidate,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryLintRunInput,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  MemoryRelationCandidate,
  ObservationStoreV1,
  PendingAction,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../lib/types";
import {
  buildLifecycleRequest,
  confirmationForOperation,
  defaultWorkflowId,
  maturePatternDecisionLabel,
  maturePatternDecisionReason,
  primaryProjectRoot,
  projectIdForProject,
} from "./memory/MemoryActionBuilders";
import { CandidateMemoryDetail, FormalMemoryDetail, operationLabel } from "./memory/MemoryDetailPanels";
import {
  AcceptanceSummaryItem,
  CandidateMemoryItem,
  ConfirmedRelationItem,
  EntityCandidateItem,
  FormalMemoryItem,
  MaturePatternCandidateItem,
  MemoryClusterReportItem,
  MergeCandidateItem,
  RelationCandidateItem,
} from "./memory/MemoryListPanels";
import { MemoryCenterStats, MemoryWorkbenchSummary } from "./memory/MemoryWorkbenchSummary";

export function MemoryCenterView({
  projects,
  workflowState,
  formalMemoryStore,
  memoryCaptureStore,
  memoryCandidateStore,
  observationStore,
  memoryLintStore,
  memoryEntityRelationStore,
  memoryPatternStore,
  hasRealSnapshot,
  onRequestAction,
  onPreviewFormalMemoryLifecycle,
  onPreviewMemoryEntityRelationCandidates,
  onPreviewMaturePatterns,
}: {
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryCaptureStore?: MemoryCaptureStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  memoryEntityRelationStore?: MemoryEntityRelationStoreV1 | null;
  memoryPatternStore?: MemoryPatternStoreV1 | null;
  hasRealSnapshot: boolean;
  onRequestAction?: (action: PendingAction) => void;
  onPreviewFormalMemoryLifecycle?: (request: FormalMemoryLifecyclePreviewInput) => Promise<FormalMemoryLifecyclePreview>;
  onPreviewMemoryEntityRelationCandidates?: (request: {
    project_root: string;
    project_id?: string | null;
    workflow_id?: string | null;
  }) => Promise<MemoryEntityRelationPreviewOutput>;
  onPreviewMaturePatterns?: (request: {
    project_root: string;
    project_id?: string | null;
    workflow_id?: string | null;
  }) => Promise<MaturePatternPreviewOutput>;
}) {
  const [previewingKind, setPreviewingKind] = useState<FormalMemoryLifecycleOperationKind | null>(null);
  const [entityRelationPreview, setEntityRelationPreview] = useState<MemoryEntityRelationPreviewOutput | null>(null);
  const [maturePatternPreview, setMaturePatternPreview] = useState<MaturePatternPreviewOutput | null>(null);
  const [entityRelationBusy, setEntityRelationBusy] = useState(false);
  const [maturePatternBusy, setMaturePatternBusy] = useState(false);
  const [lifecycleError, setLifecycleError] = useState<string | null>(null);
  const [entityRelationError, setEntityRelationError] = useState<string | null>(null);
  const [maturePatternError, setMaturePatternError] = useState<string | null>(null);
  const summary = useMemo(
    () =>
      deriveMemoryManagementSummary({
        projects,
        workflowState,
        formalMemoryStore,
        memoryCaptureStore,
        memoryCandidateStore,
        observationStore,
        memoryLintStore,
        memoryEntityRelationStore,
        memoryEntityRelationPreview: entityRelationPreview,
        memoryPatternStore,
        maturePatternPreview,
      }),
    [
      projects,
      workflowState,
      formalMemoryStore,
      memoryCaptureStore,
      memoryCandidateStore,
      observationStore,
      memoryLintStore,
      memoryEntityRelationStore,
      entityRelationPreview,
      memoryPatternStore,
      maturePatternPreview,
    ],
  );
  // 批2·P0修复:详情跟随列表选中(此前死绑数组第一项,其余记录永久点不开)。未选中=默认第一条(兼容旧行为)。
  const [selectedFormalMemoryId, setSelectedFormalMemoryId] = useState<string | null>(null);
  const [selectedCandidateKey, setSelectedCandidateKey] = useState<string | null>(null);
  const [inboxShowAll, setInboxShowAll] = useState(false);
  // 批2·P0修复:高级治理区八处硬截断统一走「显示全部」开关(此前第N+1条起永久不可达且无提示)。
  const [advancedShowAll, setAdvancedShowAll] = useState(false);
  function capped<T>(items: T[], limit: number): T[] {
    return advancedShowAll ? items : items.slice(0, limit);
  }
  const primaryFormalMemory =
    (selectedFormalMemoryId
      ? summary.formal_memories.find((item) => item.memory_id === selectedFormalMemoryId)
      : null) ??
    summary.formal_memories[0] ??
    null;
  const primaryCandidate =
    (selectedCandidateKey
      ? summary.candidate_memories.find((item) => item.candidate_key === selectedCandidateKey)
      : null) ??
    summary.candidate_memories[0] ??
    null;
  const primaryCandidateRecord = primaryCandidate
    ? memoryCandidateStore?.candidates.find((candidate) => candidate.candidate_key === primaryCandidate.candidate_key) ?? null
    : null;
  const dailyMemoryInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const dailyProjectRoot = primaryProjectRoot(projects) ?? "workbench://memory-center";
  const pageReadModel = useMemo(
    () => deriveMemoryCenterPageReadModelFromParts({ summary, hasRealSnapshot }),
    [summary, hasRealSnapshot],
  );

  function requestCandidateConfirmation() {
    if (!primaryCandidateRecord || !onRequestAction) return;
    onRequestAction(
      buildDailyMemoryCandidateDecisionAction({
        candidate: primaryCandidateRecord,
        projectRoot: dailyProjectRoot,
        requestedStatus: "candidate_confirmed",
        reason: `记忆中心确认候选属实：${primaryCandidateRecord.claim}；仍不写正式记忆。`,
        candidateStoreRevision: memoryCandidateStore?.revision ?? null,
      }),
    );
  }

  function requestCandidateAdoption() {
    if (!primaryCandidateRecord || !onRequestAction) return;
    onRequestAction(
      buildAdoptMemoryCandidateAction({
        candidate: primaryCandidateRecord,
        projectRoot: dailyProjectRoot,
        candidateStoreRevision: memoryCandidateStore?.revision ?? null,
        formalStoreRevision: formalMemoryStore?.revision ?? null,
      }),
    );
  }

  async function requestLifecycleAction(item: FormalMemoryListItem, operationKind: FormalMemoryLifecycleOperationKind) {
    if (!onRequestAction || !onPreviewFormalMemoryLifecycle) {
      setLifecycleError("当前运行环境未连接生命周期预览命令。");
      return;
    }
    setPreviewingKind(operationKind);
    setLifecycleError(null);
    try {
      const request = buildLifecycleRequest({
        item,
        operationKind,
        allFormalMemories: summary.formal_memories,
        projects,
        workflowState,
        storeRevision: formalMemoryStore?.revision ?? null,
      });
      const preview = await onPreviewFormalMemoryLifecycle(request);
      const confirmation = confirmationForOperation(operationKind, preview);
      const lifecycleInput: FormalMemoryLifecycleInput = {
        ...request,
        confirmed_by: confirmation.confirmedBy,
        confirmation_summary: confirmation.summary,
      };
      onRequestAction({
        kind: "record-formal-memory-lifecycle-operation",
        label: `正式记忆 ${operationLabel(operationKind)}`,
        path: request.project_root,
        source: "Tauri 应用数据目录",
        boundary: "编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；非活跃记忆默认不进任务包。",
        formalMemoryLifecycle: lifecycleInput,
        formalMemoryLifecyclePreview: preview,
      });
    } catch (error) {
      setLifecycleError(messageOf(error));
    } finally {
      setPreviewingKind(null);
    }
  }

  async function requestEntityRelationPreview() {
    if (!onPreviewMemoryEntityRelationCandidates) {
      setEntityRelationError("当前运行环境未连接实体 / 关系候选 preview 命令。");
      return;
    }
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) {
      setEntityRelationError("当前索引没有可用于实体 / 关系 preview 的项目路径。");
      return;
    }
    setEntityRelationBusy(true);
    setEntityRelationError(null);
    try {
      const preview = await onPreviewMemoryEntityRelationCandidates({
        project_root: projectRoot,
        project_id: null,
        workflow_id: null,
      });
      setEntityRelationPreview(preview);
    } catch (error) {
      setEntityRelationError(messageOf(error));
    } finally {
      setEntityRelationBusy(false);
    }
  }

  async function requestMaturePatternPreview() {
    if (!onPreviewMaturePatterns) {
      setMaturePatternError("当前运行环境未连接成熟模式 preview 命令。");
      return;
    }
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) {
      setMaturePatternError("当前索引没有可用于成熟模式 preview 的项目路径。");
      return;
    }
    setMaturePatternBusy(true);
    setMaturePatternError(null);
    try {
      const preview = await onPreviewMaturePatterns({
        project_root: projectRoot,
        project_id: null,
        workflow_id: null,
      });
      setMaturePatternPreview(preview);
    } catch (error) {
      setMaturePatternError(messageOf(error));
    } finally {
      setMaturePatternBusy(false);
    }
  }

  function requestMaturePatternDecision(candidate: MaturePatternCandidate, decision: MaturePatternDecisionKind) {
    if (!onRequestAction) return;
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) return;
    const isConfirmation = decision === "confirm_as_formal_memory";
    onRequestAction({
      kind: "record-mature-pattern-decision",
      label: maturePatternDecisionLabel(decision),
      path: projectRoot,
      source: "Tauri 应用数据目录",
      boundary:
        isConfirmation
          ? "用户确认后写 memory-patterns.v1.json，并通过正式记忆受控路径写 formal-memories.v1.json；候选和报告未确认不进入任务包。"
          : "只写 memory-patterns.v1.json 的候选决定；不写正式记忆，不改来源材料，不影响任务包入选列表。",
      maturePatternDecision: {
        project_root: projectRoot,
        candidate_id: candidate.candidate_id,
        decision,
        actor_id: "user-memory-center",
        actor_role: "user",
        confirmed_by: isConfirmation ? "user" : null,
        reason: maturePatternDecisionReason(candidate, decision),
        expected_pattern_store_revision: summary.mature_pattern_summary.revision,
        expected_formal_store_revision: isConfirmation ? formalMemoryStore?.revision ?? null : null,
      },
      maturePatternCandidate: candidate,
    });
  }

  function requestAliasDecision(candidate: MemoryEntityCandidate, decision: "confirm_alias" | "reject_alias") {
    if (!onRequestAction) return;
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) return;
    onRequestAction({
      kind: "record-memory-entity-alias-decision",
      label: decision === "confirm_alias" ? "登记实体 / 别名候选" : "拒绝实体候选",
      path: projectRoot,
      source: "Tauri 应用数据目录",
      boundary: "只写 memory-entity-relations.v1.json；不写正式记忆，不改候选记忆，不改工作流状态。",
      memoryEntityAliasDecision: {
        project_root: projectRoot,
        entity_candidate_id: candidate.candidate_id,
        decision,
        actor_id: "project-director-memory-center",
        actor_role: "project_director",
        reason: decision === "confirm_alias" ? "项目主管登记实体 / 别名候选。" : "项目主管拒绝实体候选。",
        expected_store_revision: summary.entity_relation_summary.revision,
      },
      memoryEntityAliasCandidate: candidate,
    });
  }

  function requestMergeDecision(candidate: MemoryEntityMergeCandidate, decision: "confirm_merge" | "reject_merge") {
    if (!onRequestAction) return;
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) return;
    onRequestAction({
      kind: "record-memory-entity-merge-decision",
      label: decision === "confirm_merge" ? "确认实体去重候选" : "拒绝实体去重候选",
      path: projectRoot,
      source: "Tauri 应用数据目录",
      boundary: "只写 memory-entity-relations.v1.json；相似度命中仅作候选，确认也不会改正式记忆。",
      memoryEntityMergeDecision: {
        project_root: projectRoot,
        merge_candidate_id: candidate.merge_candidate_id,
        decision,
        actor_id: "project-director-memory-center",
        actor_role: "project_director",
        confirmed_by: decision === "confirm_merge" ? "project_director" : null,
        reason: decision === "confirm_merge" ? "项目主管确认实体去重候选。" : "项目主管拒绝实体去重候选。",
        expected_store_revision: summary.entity_relation_summary.revision,
      },
      memoryEntityMergeCandidate: candidate,
    });
  }

  function requestRelationDecision(candidate: MemoryRelationCandidate, decision: "confirm_relation" | "reject_relation") {
    if (!onRequestAction) return;
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) return;
    onRequestAction({
      kind: "record-memory-relation-candidate-decision",
      label: decision === "confirm_relation" ? "确认关系候选" : "拒绝关系候选",
      path: projectRoot,
      source: "Tauri 应用数据目录",
      boundary: "只写 memory-entity-relations.v1.json；已确认关系只用于解释召回原因，不改变任务包入选列表。",
      memoryRelationCandidateDecision: {
        project_root: projectRoot,
        relation_candidate_id: candidate.candidate_id,
        decision,
        actor_id: "project-director-memory-center",
        actor_role: "project_director",
        confirmed_by: decision === "confirm_relation" ? "project_director" : null,
        reason: decision === "confirm_relation" ? "项目主管确认关系候选，用于解释召回原因。" : "项目主管拒绝关系候选。",
        expected_store_revision: summary.entity_relation_summary.revision,
      },
      memoryRelationCandidate: candidate,
    });
  }

  function requestMaintenanceRun() {
    if (!onRequestAction) return;
    const projectRoot = primaryProjectRoot(projects);
    if (!projectRoot) {
      setEntityRelationError("当前索引没有可用于维护任务的项目路径。");
      return;
    }
    const projectId = projectIdForProject(projects.find((project) => project.project_root === projectRoot) ?? projects[0]);
    const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_id === projectId) ?? null;
    const request: MemoryLintRunInput = {
      project_root: projectRoot,
      project_id: projectId,
      workflow_id: projectWorkflow?.workflow_id ?? defaultWorkflowId(projectRoot),
      actor_id: "project-director-memory-center",
      actor_role: "project_director",
      lint_intent: "maintenance_run",
      candidate_key: null,
      task_id: "memory-maintenance:m11",
      revoked_source_ids: [],
      expected_formal_store_revision: formalMemoryStore?.revision ?? null,
      expected_candidate_store_revision: memoryCandidateStore?.revision ?? null,
      expected_lint_store_revision: memoryLintStore?.revision ?? null,
      dry_run: false,
    };
    onRequestAction({
      kind: "run-memory-maintenance",
      label: "运行记忆维护任务",
      path: projectRoot,
      source: "Tauri 应用数据目录",
      boundary: "只写 memory-lint.v1.json 的维护运行、发现和报告；不会自动修改正式记忆、候选、观察、实体关系或工作流状态。",
      memoryMaintenanceRun: request,
    });
  }

  return (
    <section className="stage-pad memory-center" aria-label="记忆管理最小入口">
      <div className="sr-only">
        <p>记忆</p>
        <h1>记忆</h1>
        <p>{pageReadModel.snapshot_status_label}；{pageReadModel.boundary}</p>
      </div>

      <MemoryCenterStats pageReadModel={pageReadModel} />

      <div className="memory-center-grid">
        <MemoryWorkbenchSummary pageReadModel={pageReadModel} summary={summary} />

        <section className="memory-center-panel formal-memory-panel" aria-label="正式记忆列表">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">正式记忆</p>
              <h3>来源 / 版本 / 审计</h3>
            </div>
            <Badge tone="candidate">{summary.formal_memories.length}</Badge>
          </div>
          <div className="workflow-compact-list">
            {summary.formal_memories.map((item) => (
              <FormalMemoryItem
                item={item}
                key={item.memory_id}
                selected={primaryFormalMemory?.memory_id === item.memory_id}
                onSelect={() => setSelectedFormalMemoryId(item.memory_id)}
              />
            ))}
            {!summary.formal_memories.length ? <p className="muted small-note">暂无正式记忆；交货时点[属实,沉淀]产生候选，采纳后才会出现在这里。</p> : null}
          </div>
        </section>

        <section className="memory-center-panel candidate-memory-panel" aria-label="候选记忆列表">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">候选记忆</p>
              <h3>风险 / 确认 / 采纳回链</h3>
            </div>
            <Badge tone={summary.candidate_memories.length ? "warning" : "unknown"}>{summary.candidate_memories.length}</Badge>
          </div>
          <DailyMemoryCandidateInbox
            inbox={dailyMemoryInbox}
            projectRoot={dailyProjectRoot}
            candidateStoreRevision={memoryCandidateStore?.revision ?? null}
            formalStoreRevision={formalMemoryStore?.revision ?? null}
            onRequestAction={onRequestAction}
            showAll={inboxShowAll}
            onShowAll={() => setInboxShowAll(true)}
          />
          <div className="workflow-compact-list">
            {summary.candidate_memories.map((item) => (
              <CandidateMemoryItem
                item={item}
                key={item.candidate_key}
                selected={primaryCandidate?.candidate_key === item.candidate_key}
                onSelect={() => setSelectedCandidateKey(item.candidate_key)}
              />
            ))}
            {!summary.candidate_memories.length ? <p className="muted small-note">暂无候选记忆；去项目页交办一单活，交货时点[属实,沉淀]就会出现在这里。</p> : null}
          </div>
        </section>

        <section className="memory-center-panel memory-detail-panel" aria-label="记忆详情">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">详情</p>
              <h3>入包资格 / 冲突 / 边界</h3>
            </div>
            <Badge tone="neutral">受控</Badge>
          </div>
          {primaryFormalMemory ? (
            <FormalMemoryDetail
              item={primaryFormalMemory}
              busyKind={previewingKind}
              error={lifecycleError}
              onLifecycleAction={(operationKind) => void requestLifecycleAction(primaryFormalMemory, operationKind)}
            />
          ) : null}
          {primaryCandidate ? (
            <CandidateMemoryDetail
              item={primaryCandidate}
              canConfirm={primaryCandidateRecord?.status === "candidate_needs_review"}
              canAdopt={primaryCandidateRecord?.status === "candidate_confirmed" && !primaryCandidateRecord.adoption}
              onConfirm={onRequestAction ? requestCandidateConfirmation : undefined}
              onAdopt={onRequestAction ? requestCandidateAdoption : undefined}
            />
          ) : null}
          {!primaryFormalMemory && !primaryCandidate ? <p className="muted small-note">暂无可展示详情；先在左侧列表选一条记忆或候选（列表为空时，去项目页交办一单活攒第一条）。</p> : null}
        </section>

        <details className="memory-advanced-details">
          <summary className="memory-advanced-summary">高级治理 / 诊断</summary>

        {!advancedShowAll ? (
          <button className="jiaoban-linklike memory-advanced-showall" type="button" onClick={() => setAdvancedShowAll(true)}>
            列表默认只显示前几条——点这里显示全部
          </button>
        ) : null}

        <section className="memory-center-panel memory-entity-relation-panel" aria-label="实体和关系治理">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">实体 / 关系治理</p>
              <h3>实体候选 / 关系候选 / 已确认关系</h3>
            </div>
            <Badge tone={summary.entity_relation_summary.confirmed_relation_count ? "candidate" : "unknown"}>
              {summary.entity_relation_summary.entity_candidates.length +
                summary.entity_relation_summary.merge_candidates.length +
                summary.entity_relation_summary.relation_candidates.length}{" 候选 · "}
              {summary.entity_relation_summary.confirmed_relation_count}{" 已确认"}
            </Badge>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>治理摘要</strong>
              <span>{summary.entity_relation_summary.display_text}</span>
                <em>已确认关系用于解释召回原因；关系候选不会影响任务包入选清单。</em>
              <div className="knowledge-action-row">
                <button type="button" className="secondary-button" onClick={() => void requestEntityRelationPreview()} disabled={entityRelationBusy}>
                  {entityRelationBusy ? "预览中" : "刷新实体 / 关系候选"}
                </button>
              </div>
              {entityRelationError ? <p className="state-warning">{entityRelationError}</p> : null}
            </div>
            {capped(summary.entity_relation_summary.entity_candidates, 3).map((candidate) => (
              <EntityCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestAliasDecision(candidate, "confirm_alias")}
                onReject={() => requestAliasDecision(candidate, "reject_alias")}
              />
            ))}
            {capped(summary.entity_relation_summary.merge_candidates, 3).map((candidate) => (
              <MergeCandidateItem
                candidate={candidate}
                key={candidate.merge_candidate_id}
                onConfirm={() => requestMergeDecision(candidate, "confirm_merge")}
                onReject={() => requestMergeDecision(candidate, "reject_merge")}
              />
            ))}
            {capped(summary.entity_relation_summary.relation_candidates, 3).map((candidate) => (
              <RelationCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestRelationDecision(candidate, "confirm_relation")}
                onReject={() => requestRelationDecision(candidate, "reject_relation")}
              />
            ))}
            {capped(summary.entity_relation_summary.confirmed_relations, 4).map((relation) => (
              <ConfirmedRelationItem relation={relation} key={relation.relation_id} />
            ))}
            {!summary.entity_relation_summary.entity_candidates.length &&
            !summary.entity_relation_summary.merge_candidates.length &&
            !summary.entity_relation_summary.relation_candidates.length &&
            !summary.entity_relation_summary.confirmed_relations.length ? (
              <p className="muted small-note">暂无实体 / 关系候选；可先刷新候选预览。</p>
            ) : null}
          </div>
        </section>

        <section className="memory-center-panel" aria-label="任务包冻结快照与检查摘要">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">任务包冻结快照</p>
              <h3>入选 / 排除 / 待审查材料</h3>
            </div>
            <Badge tone={summary.task_package_summary.snapshot_count ? "candidate" : "unknown"}>
              {summary.task_package_summary.snapshot_count}
            </Badge>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>任务包冻结快照</strong>
              <span>{summary.task_package_summary.display_text}</span>
              <em>只有启用态正式记忆可进入入选清单；候选 / 观察只作为待审查材料。</em>
            </div>
            <div className="workflow-compact-item">
              <strong>冲突 / 检查摘要</strong>
              <span>{summary.lint_summary.display_text}</span>
              <em>阻断级发现会阻止进入任务包；检查不会自动修改正式记忆。</em>
            </div>
            {capped(summary.task_package_summary.stale_reasons, 3).map((reason) => (
              <p className="state-warning" key={reason}>{reason}</p>
            ))}
          </div>
        </section>

        <section className="memory-center-panel memory-maintenance-panel" aria-label="维护任务和记忆检查">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">维护任务</p>
              <h3>运行 / 发现 / 报告</h3>
            </div>
            <Badge tone={summary.maintenance_summary.blocking_count ? "warning" : "candidate"}>
              {summary.maintenance_summary.blocking_count}{" 阻断 · "}{summary.maintenance_summary.check_summaries.length}{" 检查"}
            </Badge>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>维护摘要</strong>
              <span>{summary.maintenance_summary.display_text}</span>
              <em>{summary.maintenance_summary.index_status_text}</em>
              <em>{summary.maintenance_summary.recent_run_label}</em>
              <div className="knowledge-action-row">
                <button type="button" className="secondary-button" onClick={requestMaintenanceRun}>
                  运行维护任务
                </button>
              </div>
              <em>维护任务只生成发现；阻断级发现会阻止召回；不会自动修改正式记忆。</em>
            </div>
            {capped(summary.maintenance_summary.check_summaries, 4).map((check) => (
              <div className="workflow-compact-item" key={check}>
                <span>{check}</span>
              </div>
            ))}
            {capped(summary.maintenance_summary.recommendation_summaries, 3).map((recommendation) => (
              <p className="state-warning" key={recommendation}>{recommendation}</p>
            ))}
          </div>
        </section>

        <section className="memory-center-panel memory-mature-pattern-panel" aria-label="成熟模式和跨项目主题">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">成熟模式 / 跨项目主题</p>
              <h3>候选 / 报告 / 验收门禁</h3>
            </div>
            <Badge tone={summary.mature_pattern_summary.user_confirmation_required_count ? "warning" : "unknown"}>
              {summary.mature_pattern_summary.mature_pattern_candidate_count}
            </Badge>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>集成验收摘要</strong>
              <span>{summary.mature_pattern_summary.display_text}</span>
              <em>{summary.mature_pattern_summary.boundary_text}</em>
              <div className="knowledge-action-row">
                <button type="button" className="secondary-button" onClick={() => void requestMaturePatternPreview()} disabled={maturePatternBusy}>
                  {maturePatternBusy ? "预览中" : "刷新成熟模式候选"}
                </button>
              </div>
              {maturePatternError ? <p className="state-warning">{maturePatternError}</p> : null}
            </div>
            {capped(summary.mature_pattern_summary.mature_pattern_candidates, 4).map((candidate) => (
              <MaturePatternCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestMaturePatternDecision(candidate, "confirm_as_formal_memory")}
                onReject={() => requestMaturePatternDecision(candidate, "reject")}
                onQuarantine={() => requestMaturePatternDecision(candidate, "quarantine")}
                onRequestChanges={() => requestMaturePatternDecision(candidate, "request_changes")}
              />
            ))}
            {capped(summary.mature_pattern_summary.cluster_reports, 3).map((report) => (
              <MemoryClusterReportItem report={report} key={report.report_id} />
            ))}
            <AcceptanceSummaryItem acceptanceSummary={summary.mature_pattern_summary.acceptance_summary} />
            {!summary.mature_pattern_summary.mature_pattern_candidates.length && !summary.mature_pattern_summary.cluster_reports.length ? (
              <p className="muted small-note">暂无成熟模式候选或跨项目主题报告；可先刷新预览。候选和报告不会补编任务包。</p>
            ) : null}
          </div>
        </section>

        <section className="memory-center-panel" aria-label="记忆捕获总线">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">记忆捕获</p>
              <h3>操作先进入捕获总线</h3>
            </div>
            <Badge tone="unknown">{summary.capture_events.length}</Badge>
          </div>
          <div className="workflow-compact-list">
            {summary.capture_events.map((item) => (
              <div className="workflow-compact-item memory-capture-item" key={`${item.summary}-${item.updated_at}`}>
                <strong>{item.kind_label} / {item.source_label}</strong>
                <span>{item.summary}</span>
                <em>{item.policy_label}；{item.observation_link}；{item.candidate_link}</em>
                <em>{item.boundary}；正式化仍需确认。</em>
              </div>
            ))}
            {!summary.capture_events.length ? <p className="muted small-note">暂无捕获事件；系统不会伪造记忆来源——去项目页交办一单活，操作会先进捕获总线。</p> : null}
          </div>
        </section>

        <section className="memory-center-panel" aria-label="观察来源">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">观察来源</p>
              <h3>观察不是正式记忆</h3>
            </div>
            <Badge tone="unknown">{summary.observation_sources.length}</Badge>
          </div>
          <div className="workflow-compact-list">
            {summary.observation_sources.map((item) => (
              <div className="workflow-compact-item observation-source-item" key={`${item.summary}-${item.updated_at}`}>
                <strong>{item.kind_label} / {item.status_label}</strong>
                <span>{item.summary}</span>
                <em>{item.source_summary}</em>
                <em>{item.boundary}；{item.candidate_link}</em>
              </div>
            ))}
            {!summary.observation_sources.length ? <p className="muted small-note">暂无观察来源；观察不会冒充正式记忆——观察随真实使用自动累积，无需手动创建。</p> : null}
          </div>
        </section>

        <section className="memory-center-panel" aria-label="项目相关记忆摘要">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">项目相关记忆摘要</p>
              <h3>轻量汇总</h3>
            </div>
            <Badge tone="neutral">{summary.project_summaries.length}</Badge>
          </div>
          <div className="workflow-compact-list">
            {summary.project_summaries.map((item) => (
              <div className="workflow-compact-item project-memory-summary-item" key={item.project_name}>
                <strong>{item.project_name}</strong>
                <span>{item.display_text}</span>
                <em>项目页只保留轻量摘要；完整治理在本记忆中心进行。</em>
              </div>
            ))}
          </div>
        </section>

        <section className="memory-center-panel memory-recent-panel" aria-label="最近变化">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">最近变化</p>
              <h3>版本 / 审计 / 检查</h3>
            </div>
            <Badge tone="neutral">{summary.recent_changes.length}</Badge>
          </div>
          <div className="workflow-compact-list">
            {summary.recent_changes.map((change) => (
              <div className="workflow-compact-item" key={change}>
                <span>{change}</span>
              </div>
            ))}
            {!summary.recent_changes.length ? <p className="muted small-note">暂无版本、审计或检查变化；对记忆做任何生命周期操作后这里会留痕。</p> : null}
          </div>
        </section>
        </details>
      </div>
    </section>
  );
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
