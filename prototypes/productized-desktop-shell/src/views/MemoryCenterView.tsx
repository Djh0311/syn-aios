import { useMemo, useState } from "react";
import { Badge } from "../components/Badge";
import { DetailLine } from "../components/WorkbenchPrimitives";
import { deriveMemoryManagementSummary, type FormalMemoryListItem, type MemoryCandidateListItem } from "../lib/memoryCenter";
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
  MemoryClusterReport,
  MemoryEntityCandidate,
  MemoryEntityMergeCandidate,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryLintRunInput,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  MemoryRecord,
  MemoryRelation,
  MemoryRelationCandidate,
  MemoryScope,
  ObservationStoreV1,
  PendingAction,
  ProjectRecord,
  WorkflowStateSnapshot,
} from "../lib/types";

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
  const primaryFormalMemory = summary.formal_memories[0] ?? null;
  const primaryCandidate = summary.candidate_memories[0] ?? null;
  const pageReadModel = useMemo(
    () => deriveMemoryCenterPageReadModelFromParts({ summary, hasRealSnapshot }),
    [summary, hasRealSnapshot],
  );

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
      <div className="pg-head">
        <div>
          <p className="pg-sub">记忆</p>
          <h1 className="pg-title">记忆</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{pageReadModel.snapshot_status_label}</div>
          <div>{pageReadModel.boundary}</div>
        </div>
      </div>

      <div className="stat-strip memory-center-stats">
        <StatCell label="正式" value={`${pageReadModel.formal_memory.record_count}`} helper="正式记忆" />
        <StatCell label="活跃" value={`${pageReadModel.formal_memory.active_count}`} helper="可评估入包" />
        <StatCell label="候选" value={`${pageReadModel.candidate_memory.candidate_count}`} helper="候选记忆" />
        <StatCell label="观察" value={`${pageReadModel.observation.observation_count}`} helper="观察来源" />
        <StatCell label="检查" value={`${pageReadModel.lint.open_count}`} helper={`阻断 ${pageReadModel.lint.blocking_count}`} warn={pageReadModel.lint.blocking_count > 0} />
        <StatCell label="维护" value={`${pageReadModel.maintenance.blocking_count}`} helper={`复核 ${pageReadModel.maintenance.needs_review_count} / 信息 ${pageReadModel.maintenance.info_count}`} warn={pageReadModel.maintenance.blocking_count > 0} />
        <StatCell label="成熟模式" value={`${pageReadModel.mature_pattern.candidate_count}`} helper={`确认 ${pageReadModel.mature_pattern.user_confirmation_required_count}`} warn={pageReadModel.mature_pattern.user_confirmation_required_count > 0} />
        <StatCell label="任务包" value={`${pageReadModel.task_package.snapshot_count}`} helper="冻结快照" />
      </div>

      <div className="memory-center-grid">
        <section className="memory-center-panel memory-workbench-panel" aria-label="记忆工作台摘要">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">记忆工作台</p>
              <h3>捕获 / 候选 / 任务记忆包</h3>
            </div>
            <Badge tone={pageReadModel.memory_workbench.action_count ? "warning" : "candidate"}>
              {pageReadModel.memory_workbench.action_count} 待处理
            </Badge>
          </div>
          <div className="memory-workbench-strip" aria-label="记忆工作台关键数字">
            <MiniMetric label="捕获" value={`${pageReadModel.memory_workbench.capture_count}`} />
            <MiniMetric label="观察" value={`${pageReadModel.memory_workbench.observation_count}`} />
            <MiniMetric label="候选" value={`${pageReadModel.memory_workbench.candidate_count}`} />
            <MiniMetric label="待正式化" value={`${pageReadModel.memory_workbench.confirmed_pending_formalization_count}`} />
            <MiniMetric label="需补证" value={`${pageReadModel.memory_workbench.capture_compensation_count}`} warn={pageReadModel.memory_workbench.capture_compensation_count > 0} />
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item memory-workbench-lede">
              <strong>记忆链路</strong>
              <span>{summary.memory_workbench_summary.display_text}</span>
              <em>{summary.memory_workbench_summary.boundary_text}</em>
            </div>
            <div className="workflow-compact-item memory-workbench-lede">
              <strong>任务记忆包</strong>
              <span>{summary.memory_workbench_summary.task_memory_packet_text}</span>
              <em>发送前应看清入选、排除和待审材料；候选和观察不会冒充正式记忆。</em>
            </div>
            {summary.memory_workbench_summary.action_items.slice(0, 6).map((item) => (
              <div className="workflow-compact-item memory-workbench-action" key={item.action_id}>
                <div className="memory-item-topline">
                  <strong>{item.title}</strong>
                  <Badge tone={item.badge_tone}>{memoryWorkbenchActionLabel(item.kind)}</Badge>
                </div>
                <span>{item.summary}</span>
                <em>下一步：{item.next_step}</em>
              </div>
            ))}
            {!summary.memory_workbench_summary.action_items.length ? (
              <p className="muted small-note">当前没有候选确认、正式化、补证或任务记忆包待审事项。</p>
            ) : null}
          </div>
        </section>

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
              <FormalMemoryItem item={item} key={item.memory_id} />
            ))}
            {!summary.formal_memories.length ? <p className="muted small-note">暂无正式记忆；不会用候选或观察补编正式记忆。</p> : null}
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
          <div className="workflow-compact-list">
            {summary.candidate_memories.map((item) => (
              <CandidateMemoryItem item={item} key={item.candidate_key} />
            ))}
            {!summary.candidate_memories.length ? <p className="muted small-note">暂无候选记忆；候选不等于正式记忆。</p> : null}
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
          {primaryCandidate ? <CandidateMemoryDetail item={primaryCandidate} /> : null}
          {!primaryFormalMemory && !primaryCandidate ? <p className="muted small-note">暂无可展示详情。</p> : null}
        </section>

        <details className="memory-advanced-details">
          <summary className="memory-advanced-summary">高级治理 / 诊断</summary>

        <section className="memory-center-panel memory-entity-relation-panel" aria-label="实体和关系治理">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">实体 / 关系治理</p>
              <h3>实体候选 / 关系候选 / 已确认关系</h3>
            </div>
            <Badge tone={summary.entity_relation_summary.confirmed_relation_count ? "candidate" : "unknown"}>
              {summary.entity_relation_summary.confirmed_relation_count}
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
            {summary.entity_relation_summary.entity_candidates.slice(0, 3).map((candidate) => (
              <EntityCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestAliasDecision(candidate, "confirm_alias")}
                onReject={() => requestAliasDecision(candidate, "reject_alias")}
              />
            ))}
            {summary.entity_relation_summary.merge_candidates.slice(0, 3).map((candidate) => (
              <MergeCandidateItem
                candidate={candidate}
                key={candidate.merge_candidate_id}
                onConfirm={() => requestMergeDecision(candidate, "confirm_merge")}
                onReject={() => requestMergeDecision(candidate, "reject_merge")}
              />
            ))}
            {summary.entity_relation_summary.relation_candidates.slice(0, 3).map((candidate) => (
              <RelationCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestRelationDecision(candidate, "confirm_relation")}
                onReject={() => requestRelationDecision(candidate, "reject_relation")}
              />
            ))}
            {summary.entity_relation_summary.confirmed_relations.slice(0, 4).map((relation) => (
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
            {summary.task_package_summary.stale_reasons.slice(0, 3).map((reason) => (
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
              {summary.maintenance_summary.blocking_count}
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
            {summary.maintenance_summary.check_summaries.slice(0, 4).map((check) => (
              <div className="workflow-compact-item" key={check}>
                <span>{check}</span>
              </div>
            ))}
            {summary.maintenance_summary.recommendation_summaries.slice(0, 3).map((recommendation) => (
              <p className="state-warning" key={recommendation}>{recommendation}</p>
            ))}
          </div>
        </section>

        <section className="memory-center-panel memory-mature-pattern-panel" aria-label="成熟模式和跨项目主题">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">成熟模式 / 跨项目主题</p>
              <h3>候选 / 报告 / M1-M12 门禁</h3>
            </div>
            <Badge tone={summary.mature_pattern_summary.user_confirmation_required_count ? "warning" : "unknown"}>
              {summary.mature_pattern_summary.mature_pattern_candidate_count}
            </Badge>
          </div>
          <div className="workflow-compact-list">
            <div className="workflow-compact-item">
              <strong>M12 摘要</strong>
              <span>{summary.mature_pattern_summary.display_text}</span>
              <em>{summary.mature_pattern_summary.boundary_text}</em>
              <div className="knowledge-action-row">
                <button type="button" className="secondary-button" onClick={() => void requestMaturePatternPreview()} disabled={maturePatternBusy}>
                  {maturePatternBusy ? "预览中" : "刷新成熟模式候选"}
                </button>
              </div>
              {maturePatternError ? <p className="state-warning">{maturePatternError}</p> : null}
            </div>
            {summary.mature_pattern_summary.mature_pattern_candidates.slice(0, 4).map((candidate) => (
              <MaturePatternCandidateItem
                candidate={candidate}
                key={candidate.candidate_id}
                onConfirm={() => requestMaturePatternDecision(candidate, "confirm_as_formal_memory")}
                onReject={() => requestMaturePatternDecision(candidate, "reject")}
                onQuarantine={() => requestMaturePatternDecision(candidate, "quarantine")}
                onRequestChanges={() => requestMaturePatternDecision(candidate, "request_changes")}
              />
            ))}
            {summary.mature_pattern_summary.cluster_reports.slice(0, 3).map((report) => (
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
            {!summary.capture_events.length ? <p className="muted small-note">暂无捕获事件；J3 不会伪造记忆来源。</p> : null}
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
            {!summary.observation_sources.length ? <p className="muted small-note">暂无观察来源；观察不会冒充正式记忆。</p> : null}
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
                <em>项目页只保留轻量摘要；完整治理后台不在 M7 实现。</em>
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
            {!summary.recent_changes.length ? <p className="muted small-note">暂无版本、审计或检查变化。</p> : null}
          </div>
        </section>
        </details>
      </div>
    </section>
  );
}

function FormalMemoryItem({ item }: { item: FormalMemoryListItem }) {
  return (
    <div className="workflow-compact-item formal-memory-item">
      <div className="memory-item-topline">
        <strong>{item.kind_label} / {item.status_label}</strong>
        <Badge tone={item.task_eligibility.badge_tone}>{item.task_eligibility.label}</Badge>
      </div>
      <span>{item.claim}</span>
      <em>来源：{sourceText(item.source_summaries)}</em>
      <em>{item.version_summary}</em>
      <em>审计 {item.audit_summary}</em>
      <em>{item.scope_label} / {item.permission_summary} / {item.model_export_summary}</em>
      <em>{item.conflict_summary}</em>
      {item.task_eligibility.included_in_task_package ? <em>任务包冻结快照已引用</em> : null}
    </div>
  );
}

function CandidateMemoryItem({ item }: { item: MemoryCandidateListItem }) {
  return (
    <div className="workflow-compact-item candidate-memory-item">
      <div className="memory-item-topline">
        <strong>{item.kind_label} / {item.status_label}</strong>
        <Badge tone={item.task_position.badge_tone}>{item.task_position.label}</Badge>
      </div>
      <span>{item.claim}</span>
      <em>{item.formal_memory_boundary}</em>
      <em>{item.risk_summary}</em>
      <em>{item.confirmation_summary}</em>
      <em>{item.adoption_summary}</em>
      <em>来源：{sourceText(item.source_summaries)}</em>
      <em>{item.lint_summary}</em>
    </div>
  );
}

function EntityCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryEntityCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  return (
    <div className="workflow-compact-item memory-entity-candidate-item">
      <div className="memory-item-topline">
        <strong>实体候选 / {entityKindLabel(candidate.entity_kind)}</strong>
        <Badge tone="warning">{candidate.status}</Badge>
      </div>
      <span>{candidate.display_name}</span>
      <em>{candidate.reason}</em>
      <em>来源 {sourceKindLabel(candidate.source_kind)} / {candidate.confidence_kind}</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm}>登记实体 / 别名</button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

function MergeCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryEntityMergeCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  return (
    <div className="workflow-compact-item memory-merge-candidate-item">
      <div className="memory-item-topline">
        <strong>去重候选</strong>
        <Badge tone={candidate.source_kind === "similarity_hit" ? "warning" : "candidate"}>{sourceKindLabel(candidate.source_kind)}</Badge>
      </div>
      <span>{candidate.left_label} / {candidate.right_label}</span>
      <em>{candidate.reason}</em>
      <em>相似度命中仅作候选；确认动作只登记治理决定。</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm}>确认去重候选</button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

function RelationCandidateItem({
  candidate,
  onConfirm,
  onReject,
}: {
  candidate: MemoryRelationCandidate;
  onConfirm: () => void;
  onReject: () => void;
}) {
  const confirmationBlocked = candidate.source_kind === "llm_inferred" || candidate.source_kind === "similarity_hit";
  return (
    <div className="workflow-compact-item memory-relation-candidate-item">
      <div className="memory-item-topline">
        <strong>关系候选 / {relationKindLabel(candidate.relation_kind)}</strong>
        <Badge tone={candidate.relation_kind === "causal" ? "warning" : "candidate"}>{sourceKindLabel(candidate.source_kind)}</Badge>
      </div>
      <span>{candidate.subject_label} {"->"} {candidate.object_label}</span>
      <em>{candidate.reason}</em>
      <em>{candidate.requires_user_confirmation ? "需要用户确认" : "项目主管或用户可确认"}</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm} disabled={confirmationBlocked}>
          确认关系
        </button>
        <button type="button" className="secondary-button" onClick={onReject}>拒绝候选</button>
      </div>
    </div>
  );
}

function ConfirmedRelationItem({ relation }: { relation: MemoryRelation }) {
  return (
    <div className="workflow-compact-item memory-confirmed-relation-item">
      <div className="memory-item-topline">
        <strong>已确认关系 / {relationKindLabel(relation.relation_kind)}</strong>
        <Badge tone="candidate">已确认</Badge>
      </div>
      <span>{relation.subject_label} {"->"} {relation.object_label}</span>
      <em>{relation.confirmation_role} / {relation.confirmation_reason}</em>
      <em>已确认关系用于解释召回原因。</em>
    </div>
  );
}

function MaturePatternCandidateItem({
  candidate,
  onConfirm,
  onReject,
  onQuarantine,
  onRequestChanges,
}: {
  candidate: MaturePatternCandidate;
  onConfirm: () => void;
  onReject: () => void;
  onQuarantine: () => void;
  onRequestChanges: () => void;
}) {
  const actionable = candidate.status === "candidate";
  return (
    <div className="workflow-compact-item mature-pattern-candidate-item">
      <div className="memory-item-topline">
        <strong>成熟模式候选 / {candidate.pattern_kind}</strong>
        <Badge tone={maturePatternStatusTone(candidate.status)}>{candidate.status}</Badge>
      </div>
      <span>{candidate.title}</span>
      <em>{candidate.claim}</em>
      <em>来源 {candidate.source_refs.length} / 关联成员 {candidate.member_refs.length} / 识别信号 {candidate.signal_refs.length}</em>
      <em>{candidate.requires_user_confirmation ? "需要用户确认" : "仍需显式决定"}；候选未确认，不会进入任务包入选列表。</em>
      <em>{candidate.review_summary}</em>
      <div className="knowledge-action-row">
        <button type="button" className="secondary-button" onClick={onConfirm} disabled={!actionable}>
          用户确认为正式记忆
        </button>
        <button type="button" className="secondary-button" onClick={onReject} disabled={!actionable}>
          拒绝
        </button>
        <button type="button" className="secondary-button" onClick={onQuarantine} disabled={!actionable}>
          隔离
        </button>
        <button type="button" className="secondary-button" onClick={onRequestChanges} disabled={!actionable}>
          要求补来源
        </button>
      </div>
    </div>
  );
}

function MemoryClusterReportItem({ report }: { report: MemoryClusterReport }) {
  return (
    <div className="workflow-compact-item memory-cluster-report-item">
      <div className="memory-item-topline">
        <strong>跨项目主题报告 / {report.report_kind}</strong>
        <Badge tone={report.staleness === "fresh" ? "candidate" : "warning"}>{stalenessLabel(report.staleness)}</Badge>
      </div>
      <span>{report.title}</span>
      <em>{report.display_text}</em>
      <em>项目 {report.project_ids.length} / 成员引用 {report.member_refs.length} / 来源 {report.source_refs.length}</em>
      <em>报告可下钻来源，但不是正式事实，也不会进入任务包入选清单。</em>
    </div>
  );
}

function AcceptanceSummaryItem({ acceptanceSummary }: { acceptanceSummary: MaturePatternPreviewOutput["acceptance_summary"] | null }) {
  if (!acceptanceSummary) {
    return (
      <div className="workflow-compact-item memory-acceptance-summary-item">
        <strong>M1-M12 门禁摘要</strong>
        <span>尚未生成 M12 预览；不能声称完整验收摘要已刷新。</span>
        <em>M12 只覆盖 M1-M12 记忆系统集成摘要，最终权威验收仍在后续阶段。</em>
      </div>
    );
  }

  return (
    <div className="workflow-compact-item memory-acceptance-summary-item">
      <div className="memory-item-topline">
        <strong>M1-M12 门禁摘要</strong>
        <Badge tone={acceptanceSummary.blocked_count ? "warning" : "candidate"}>
          {acceptanceSummary.passed_count}/{acceptanceSummary.gate_count}
        </Badge>
      </div>
      <span>{acceptanceSummary.display_text}</span>
      <em>阻断 {acceptanceSummary.blocked_count} / 后置 {acceptanceSummary.deferred_count} / 范围 {acceptanceSummary.scope_label}</em>
      {acceptanceSummary.gates.slice(0, 6).map((gate) => (
        <em key={gate.gate_id}>
          {gate.label}：{gate.status} / {gate.evidence}
        </em>
      ))}
      <em>M12 摘要不替代后续最终权威验收。</em>
    </div>
  );
}

function FormalMemoryDetail({
  item,
  busyKind,
  error,
  onLifecycleAction,
}: {
  item: FormalMemoryListItem;
  busyKind: FormalMemoryLifecycleOperationKind | null;
  error: string | null;
  onLifecycleAction: (operationKind: FormalMemoryLifecycleOperationKind) => void;
}) {
  const operations: FormalMemoryLifecycleOperationKind[] = [
    "revise",
    "deprecate",
    "freeze",
    "unfreeze",
    "archive",
    "merge",
    "split",
    "promote_to_global",
    "demote_to_project",
  ];

  return (
    <div className="memory-detail-section">
      <h4>正式记忆详情</h4>
      <div className="workflow-draft-grid">
        <DetailLine label="来源" value={sourceText(item.source_summaries)} />
        <DetailLine label="版本摘要" value={item.version_summary} />
        <DetailLine label="审计摘要" value={item.audit_summary} />
        <DetailLine label="冲突 / 检查" value={item.conflict_summary} />
        <DetailLine label="权限 / 外发" value={`${item.permission_summary} / ${item.model_export_summary}`} />
        <DetailLine label="任务包入选状态" value={`${item.task_eligibility.label}：${item.task_eligibility.reason}`} />
      </div>
      {item.conflicts.finding_summaries.slice(0, 3).map((finding) => (
        <p className="state-warning" key={finding}>{finding}</p>
      ))}
      <div className="memory-lifecycle-actions" aria-label="正式记忆生命周期操作">
        <div className="memory-lifecycle-copy">
          <strong>生命周期</strong>
          <span>编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；冻结后不能普通编辑。</span>
        </div>
        <div className="memory-lifecycle-button-row">
          {operations.map((operationKind) => (
            <button
              className="secondary-button"
              disabled={busyKind !== null}
              key={operationKind}
              onClick={() => onLifecycleAction(operationKind)}
              type="button"
            >
              {busyKind === operationKind ? "预览中" : operationLabel(operationKind)}
            </button>
          ))}
        </div>
        <p className="muted small-note">非启用态记忆默认不进任务包；合并和拆分只使用明确可见的当前记录草稿。</p>
        {error ? <p className="state-warning">{error}</p> : null}
      </div>
    </div>
  );
}

function CandidateMemoryDetail({ item }: { item: MemoryCandidateListItem }) {
  return (
    <div className="memory-detail-section">
      <h4>候选详情</h4>
      <div className="workflow-draft-grid">
        <DetailLine label="候选状态" value={item.status_label} />
        <DetailLine label="确认要求" value={item.confirmation_summary} />
        <DetailLine label="采纳回链" value={item.adoption_summary} />
        <DetailLine label="任务包位置" value={`${item.task_position.label}：${item.task_position.reason}`} />
        <DetailLine label="候选边界" value={item.formal_memory_boundary} />
      </div>
    </div>
  );
}

function StatCell({
  label,
  value,
  helper,
  warn = false,
}: {
  label: string;
  value: string;
  helper: string;
  warn?: boolean;
}) {
  return (
    <div className="stat-cell">
      <div className="lbl">{label}</div>
      <div className={`val mono${warn ? " warn" : ""}`}>{value}</div>
      <div className="memory-stat-helper">{helper}</div>
    </div>
  );
}

function MiniMetric({ label, value, warn = false }: { label: string; value: string; warn?: boolean }) {
  return (
    <div className={`memory-mini-metric${warn ? " warn" : ""}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function memoryWorkbenchActionLabel(kind: string): string {
  if (kind === "review_candidate") return "候选待审";
  if (kind === "confirm_formalization") return "正式化";
  if (kind === "repair_capture_link") return "补证";
  if (kind === "review_task_memory_packet") return "任务包";
  if (kind === "resolve_memory_blocker") return "阻断";
  return kind;
}

function sourceText(sources: FormalMemoryListItem["source_summaries"] | MemoryCandidateListItem["source_summaries"]) {
  return sources.map((source) => `${source.label} / ${source.authority_label} / ${source.sensitive_label}`).join("；") || "来源未记录";
}

function buildLifecycleRequest({
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

function confirmationForOperation(
  operationKind: FormalMemoryLifecycleOperationKind,
  preview: FormalMemoryLifecyclePreview,
): { confirmedBy: string; summary: string } {
  const userRequired = preview.required_approval.required_actor_role === "user";
  return {
    confirmedBy: userRequired ? "user" : "project-director-ui",
    summary: `${operationLabel(operationKind)} 已查看影响面：${preview.impact.display_text}`,
  };
}

function lifecycleReason(operationKind: FormalMemoryLifecycleOperationKind, item: FormalMemoryListItem): string {
  return `${operationLabel(operationKind)} 正式记忆：${item.memory_id}；编辑会创建新版本，不覆盖旧版本。`;
}

function operationLabel(operationKind: FormalMemoryLifecycleOperationKind): string {
  const labels: Record<FormalMemoryLifecycleOperationKind, string> = {
    revise: "编辑提案",
    deprecate: "废弃",
    freeze: "冻结",
    unfreeze: "解冻",
    archive: "归档",
    merge: "合并",
    split: "拆分",
    promote_to_global: "上升为全局",
    demote_to_project: "下沉为项目",
  };
  return labels[operationKind];
}

function primaryProjectRoot(projects: ProjectRecord[]): string | null {
  return projects.find((project) => project.active_hint)?.project_root ?? projects[0]?.project_root ?? null;
}

function entityKindLabel(kind: string): string {
  if (kind === "project") return "项目";
  if (kind === "workflow") return "工作流";
  if (kind === "session") return "会话";
  if (kind === "role") return "角色";
  if (kind === "knowledge_doc") return "知识资料";
  if (kind === "tool") return "工具";
  if (kind === "model") return "模型";
  if (kind === "harness") return "运行器";
  if (kind === "proposal") return "建议方案";
  if (kind === "memory_record") return "正式记忆";
  if (kind === "memory_candidate") return "候选记忆";
  return kind;
}

function relationKindLabel(kind: string): string {
  if (kind === "entity") return "实体";
  if (kind === "temporal") return "时间";
  if (kind === "causal") return "因果";
  if (kind === "semantic") return "语义";
  return kind;
}

function sourceKindLabel(kind: string): string {
  if (kind === "manual") return "手动";
  if (kind === "formal_memory") return "正式记忆";
  if (kind === "memory_candidate") return "候选记忆";
  if (kind === "observation") return "观察";
  if (kind === "knowledge_doc") return "知识资料";
  if (kind === "task_package") return "任务包";
  if (kind === "llm_inferred") return "LLM 推断仅作候选";
  if (kind === "similarity_hit") return "相似度命中仅作候选";
  return kind;
}

function stalenessLabel(staleness: string): string {
  if (staleness === "fresh") return "新鲜";
  if (staleness === "stale") return "过期";
  if (staleness === "unknown") return "未知";
  return staleness;
}

function maturePatternStatusTone(status: MaturePatternCandidate["status"]) {
  if (status === "candidate" || status === "changes_requested") return "warning";
  if (status === "confirmed") return "candidate";
  return "unknown";
}

function maturePatternDecisionLabel(decision: MaturePatternDecisionKind): string {
  if (decision === "confirm_as_formal_memory") return "用户确认成熟模式候选";
  if (decision === "reject") return "拒绝成熟模式候选";
  if (decision === "quarantine") return "隔离成熟模式候选";
  return "要求补充成熟模式候选来源";
}

function maturePatternDecisionReason(candidate: MaturePatternCandidate, decision: MaturePatternDecisionKind): string {
  if (decision === "confirm_as_formal_memory") {
    return `用户确认成熟模式候选：${candidate.title}；通过正式记忆受控路径写入。`;
  }
  if (decision === "reject") return `用户拒绝成熟模式候选：${candidate.title}；保留来源材料。`;
  if (decision === "quarantine") return `用户隔离成熟模式候选：${candidate.title}；保留来源材料。`;
  return `用户要求补充成熟模式候选来源：${candidate.title}。`;
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

function projectIdForProject(project: ProjectRecord): string {
  return `project:${stableId(project.project_root)}`;
}

function defaultWorkflowId(projectRoot: string): string {
  return `workflow:${stableId(projectRoot)}:default`;
}

function stableId(value: string): string {
  return sanitize(value).slice(0, 96);
}

function sanitize(value: string): string {
  return value.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase() || "unknown";
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
