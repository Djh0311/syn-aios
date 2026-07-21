import { useMemo, useState } from "react";
import { EmptyState, Pill } from "../components/SpecPrimitives";
import { deriveMemoryManagementSummary, type FormalMemoryListItem } from "../lib/memoryCenter";
import {
  buildAdoptMemoryCandidateAction,
  buildBatchAdoptMemoryCandidatesAction,
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
import { CandidateMemoryDetail, FormalMemoryDetail, MemoryLintFindingDetail, operationLabel } from "./memory/MemoryDetailPanels";
import {
  AcceptanceSummaryItem,
  ConfirmedRelationItem,
  EntityCandidateItem,
  MaturePatternCandidateItem,
  MemoryClusterReportItem,
  MergeCandidateItem,
  RelationCandidateItem,
} from "./memory/MemoryListPanels";
import { MemoryWorkbenchSummary } from "./memory/MemoryWorkbenchSummary";

type MemorySelection =
  | { kind: "candidate"; id: string }
  | { kind: "lint"; id: string }
  | { kind: "formal"; id: string }
  | { kind: "governance" };

type DetailMemorySelection = Exclude<MemorySelection, { kind: "governance" }>;
type MemoryScopeFilter = "all" | "global" | string;

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
  // 记忆中心重排：所有右栏状态必须由同一选择来源决定，避免候选和正式详情同时堆叠。
  const [memorySelection, setMemorySelection] = useState<MemorySelection | null>(null);
  const [governanceReturnSelection, setGovernanceReturnSelection] = useState<DetailMemorySelection | null>(null);
  const [memoryQuery, setMemoryQuery] = useState("");
  const [memoryScopeFilter, setMemoryScopeFilter] = useState<MemoryScopeFilter>("all");
  // 批2·P0修复:高级治理区八处硬截断统一走「显示全部」开关(此前第N+1条起永久不可达且无提示)。
  const [advancedShowAll, setAdvancedShowAll] = useState(false);
  function capped<T>(items: T[], limit: number): T[] {
    return advancedShowAll ? items : items.slice(0, limit);
  }
  const dailyMemoryInbox = deriveDailyMemoryCandidateInbox({ memoryCandidateStore });
  const pageReadModel = useMemo(
    () => deriveMemoryCenterPageReadModelFromParts({ summary, hasRealSnapshot }),
    [summary, hasRealSnapshot],
  );
  const allOpenLintFindings = (memoryLintStore?.findings ?? []).filter((finding) => finding.status === "open");
  const openLintFindings = allOpenLintFindings.filter(
    (finding) => finding.severity === "blocking" || finding.severity === "needs_review",
  );
  const matchesScopeFilter = (scope: { scope_type: string; project_id?: string | null }) => {
    if (memoryScopeFilter === "all") return true;
    if (memoryScopeFilter === "global") return scope.scope_type === "global";
    return scope.project_id === memoryScopeFilter;
  };
  const formalById = new Map(summary.formal_memories.map((item) => [item.memory_id, item]));
  const candidateByKey = new Map((memoryCandidateStore?.candidates ?? []).map((item) => [item.candidate_key, item]));
  const formalForFinding = (finding: (typeof allOpenLintFindings)[number]) => {
    const memoryId = finding.target_memory_id ?? (finding.source_kind === "memory_record" ? finding.source_id : null);
    return memoryId ? formalById.get(memoryId) ?? null : null;
  };
  const candidateForFinding = (finding: (typeof allOpenLintFindings)[number]) => {
    const candidateKey =
      finding.target_candidate_key ?? (finding.source_kind === "memory_candidate" ? finding.source_id : null);
    return candidateKey ? candidateByKey.get(candidateKey) ?? null : null;
  };
  const findingMatchesScopeFilter = (finding: (typeof openLintFindings)[number]) => {
    const formal = formalForFinding(finding);
    if (formal) return matchesScopeFilter(formal.record.scope);
    const candidate = candidateForFinding(finding);
    if (candidate) return matchesScopeFilter(candidate.scope);
    return memoryScopeFilter === "all";
  };
  const scopedCandidates = dailyMemoryInbox.items.filter((item) => matchesScopeFilter(item.candidate.scope));
  const scopedLintFindings = openLintFindings.filter(findingMatchesScopeFilter);
  const scopedProjectFormalMemories = summary.formal_memories.filter(
    (item) => item.record.scope.scope_type !== "global" && matchesScopeFilter(item.record.scope),
  );
  const scopedGlobalFormalMemories = summary.formal_memories.filter(
    (item) => item.record.scope.scope_type === "global" && matchesScopeFilter(item.record.scope),
  );
  const projectScopeOptions = projects
    .map((project) => ({ project, id: projectIdForProject(project) }))
    .filter(({ id }) =>
      dailyMemoryInbox.items.some((item) => item.candidate.scope.project_id === id) ||
      summary.formal_memories.some((item) => item.record.scope.project_id === id) ||
      openLintFindings.some((finding) => {
        const formal = formalForFinding(finding);
        if (formal?.record.scope.project_id === id) return true;
        return candidateForFinding(finding)?.scope.project_id === id;
      }),
    );
  const hasGlobalScope =
    dailyMemoryInbox.items.some((item) => item.candidate.scope.scope_type === "global") ||
    summary.formal_memories.some((item) => item.record.scope.scope_type === "global");
  const query = memoryQuery.trim().toLocaleLowerCase();
  const matchesQuery = (...values: Array<string | null | undefined>) =>
    !query || values.some((value) => value?.toLocaleLowerCase().includes(query));
  const visibleCandidates = scopedCandidates.filter((item) => matchesQuery(item.claim, item.source_label));
  const visibleLintFindings = scopedLintFindings.filter((item) => matchesQuery(item.summary, item.claim));
  const visibleProjectFormalMemories = scopedProjectFormalMemories.filter((item) => matchesQuery(item.claim, item.status_label));
  const visibleGlobalFormalMemories = scopedGlobalFormalMemories.filter((item) => matchesQuery(item.claim, item.status_label));
  const fallbackSelection: MemorySelection | null =
    scopedCandidates[0]
      ? { kind: "candidate", id: scopedCandidates[0].candidate_key }
      : scopedLintFindings[0]
        ? { kind: "lint", id: scopedLintFindings[0].finding_id }
        : scopedProjectFormalMemories[0]
          ? { kind: "formal", id: scopedProjectFormalMemories[0].memory_id }
          : scopedGlobalFormalMemories[0]
            ? { kind: "formal", id: scopedGlobalFormalMemories[0].memory_id }
            : null;
  const selectionExists = (selection: MemorySelection) => {
    if (selection.kind === "governance") return true;
    if (selection.kind === "candidate") return dailyMemoryInbox.items.some((item) => item.candidate_key === selection.id);
    if (selection.kind === "lint") return openLintFindings.some((item) => item.finding_id === selection.id);
    return summary.formal_memories.some((item) => item.memory_id === selection.id);
  };
  const activeSelection = memorySelection && selectionExists(memorySelection) ? memorySelection : fallbackSelection;
  const primaryFormalMemory =
    activeSelection?.kind === "formal"
      ? summary.formal_memories.find((item) => item.memory_id === activeSelection.id) ?? null
      : null;
  const primaryCandidate =
    activeSelection?.kind === "candidate"
      ? summary.candidate_memories.find((item) => item.candidate_key === activeSelection.id) ?? null
      : null;
  const primaryCandidateRecord = primaryCandidate
    ? memoryCandidateStore?.candidates.find((candidate) => candidate.candidate_key === primaryCandidate.candidate_key) ?? null
    : null;
  const primaryLintFinding =
    activeSelection?.kind === "lint"
      ? openLintFindings.find((item) => item.finding_id === activeSelection.id) ?? null
      : null;
  const lintTargetFormalMemory = primaryLintFinding ? formalForFinding(primaryLintFinding) : null;
  const primaryCandidateHasOpenLint = Boolean(
    primaryCandidate && allOpenLintFindings.some((finding) => candidateForFinding(finding)?.candidate_key === primaryCandidate.candidate_key),
  );
  const candidateProjectRoot = primaryCandidateRecord?.scope.project_id
    ? projects.find((project) => projectIdForProject(project) === primaryCandidateRecord.scope.project_id)?.project_root ?? null
    : null;
  const candidateActionsAvailable = Boolean(onRequestAction && candidateProjectRoot);
  const scopedAdoptableCandidates = scopedCandidates.filter((item) => item.can_adopt);
  const batchCandidateProjectId = scopedAdoptableCandidates[0]?.candidate.scope.project_id ?? null;
  const batchCandidateProjectRoot =
    batchCandidateProjectId &&
    scopedAdoptableCandidates.every((item) => item.candidate.scope.project_id === batchCandidateProjectId)
      ? projects.find((project) => projectIdForProject(project) === batchCandidateProjectId)?.project_root ?? null
      : null;
  const batchAdoptableCandidates = batchCandidateProjectRoot ? scopedAdoptableCandidates : [];
  const recalledFormalMemory = summary.task_package_summary.referenced_memory_ids
    .map((memoryId) => formalById.get(memoryId) ?? null)
    .find((item): item is FormalMemoryListItem => item !== null) ?? null;

  function requestCandidateConfirmation() {
    if (!primaryCandidateRecord || !candidateProjectRoot || !onRequestAction) return;
    onRequestAction(
      buildDailyMemoryCandidateDecisionAction({
        candidate: primaryCandidateRecord,
        projectRoot: candidateProjectRoot,
        requestedStatus: "candidate_confirmed",
        reason: `记忆中心确认候选属实：${primaryCandidateRecord.claim}；仍不写正式记忆。`,
        candidateStoreRevision: memoryCandidateStore?.revision ?? null,
      }),
    );
  }

  function requestCandidateAdoption() {
    if (!primaryCandidateRecord || !candidateProjectRoot || !onRequestAction) return;
    onRequestAction(
      buildAdoptMemoryCandidateAction({
        candidate: primaryCandidateRecord,
        projectRoot: candidateProjectRoot,
        candidateStoreRevision: memoryCandidateStore?.revision ?? null,
        formalStoreRevision: formalMemoryStore?.revision ?? null,
      }),
    );
  }

  function requestCandidateDecision(requestedStatus: "candidate_discarded" | "candidate_rejected") {
    if (!primaryCandidateRecord || !candidateProjectRoot || !onRequestAction) return;
    const label = requestedStatus === "candidate_rejected" ? "拒绝候选" : "暂不处理候选";
    onRequestAction(
      buildDailyMemoryCandidateDecisionAction({
        candidate: primaryCandidateRecord,
        projectRoot: candidateProjectRoot,
        requestedStatus,
        reason: `记忆中心${label}：${primaryCandidateRecord.claim}；不写正式记忆。`,
        candidateStoreRevision: memoryCandidateStore?.revision ?? null,
      }),
    );
  }

  function requestBatchCandidateAdoption() {
    if (!onRequestAction) return;
    const candidates = batchAdoptableCandidates.map((item) => item.candidate);
    if (!candidates.length || !batchCandidateProjectRoot) return;
    onRequestAction(
      buildBatchAdoptMemoryCandidatesAction({
        candidates,
        projectRoot: batchCandidateProjectRoot,
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
      <div className="memwrap">
        <aside className="mlist" aria-label="记忆列表">
          <header className="memory-list-heading">
            <h1>记忆层</h1>
            <p>AI 记着什么，你说了算</p>
          </header>
          <div className="memory-list-tools">
            <input
              type="search"
              className="jiaoban-session-search"
              placeholder="搜记忆…"
              value={memoryQuery}
              onChange={(event) => setMemoryQuery(event.target.value)}
              aria-label="搜索记忆"
            />
            <div className="memory-project-filters" role="group" aria-label="按项目筛选记忆">
              <button
                aria-pressed={memoryScopeFilter === "all"}
                className={`jiaoban-chip ${memoryScopeFilter === "all" ? "on" : ""}`}
                type="button"
                onClick={() => {
                  setMemoryScopeFilter("all");
                  setMemorySelection(null);
                }}
              >
                全部
              </button>
              {projectScopeOptions.map(({ project, id }) => (
                <button
                  aria-pressed={memoryScopeFilter === id}
                  className={`jiaoban-chip ${memoryScopeFilter === id ? "on" : ""}`}
                  key={id}
                  type="button"
                  onClick={() => {
                    setMemoryScopeFilter(id);
                    setMemorySelection(null);
                  }}
                >
                  {project.name}
                </button>
              ))}
              {hasGlobalScope ? (
                <button
                  aria-pressed={memoryScopeFilter === "global"}
                  className={`jiaoban-chip ${memoryScopeFilter === "global" ? "on" : ""}`}
                  type="button"
                  onClick={() => {
                    setMemoryScopeFilter("global");
                    setMemorySelection(null);
                  }}
                >
                  全局
                </button>
              ) : null}
            </div>
          </div>

          <section className="memory-list-group" data-memory-group="candidate">
            <h2 className="mgroup">候选 · 等你确认 <span className="n">{scopedCandidates.length}</span></h2>
            {visibleCandidates.map((item) => (
              <button
                aria-pressed={activeSelection?.kind === "candidate" && activeSelection.id === item.candidate_key}
                className={`mrow${activeSelection?.kind === "candidate" && activeSelection.id === item.candidate_key ? " sel" : ""}`}
                data-memory-row-id={item.candidate_key}
                data-memory-row-kind="candidate"
                key={item.candidate_key}
                onClick={() => setMemorySelection({ kind: "candidate", id: item.candidate_key })}
                type="button"
              >
                <span className="t">{item.claim}</span>
                <span className="m">来自：{item.source_label} · {memoryRowDate(item.updated_at)}</span>
              </button>
            ))}
            {batchAdoptableCandidates.length ? (
              <button className="memory-group-action" onClick={requestBatchCandidateAdoption} type="button">
                批量记住 {batchAdoptableCandidates.length} 条
              </button>
            ) : null}
          </section>

          <section className="memory-list-group" data-memory-group="lint">
            <h2 className="mgroup">要你看 <span className="n">{scopedLintFindings.length}</span></h2>
            {visibleLintFindings.map((finding) => (
              <button
                aria-pressed={activeSelection?.kind === "lint" && activeSelection.id === finding.finding_id}
                className={`mrow${activeSelection?.kind === "lint" && activeSelection.id === finding.finding_id ? " sel" : ""}`}
                data-memory-row-id={finding.finding_id}
                data-memory-row-kind="lint"
                key={finding.finding_id}
                onClick={() => setMemorySelection({ kind: "lint", id: finding.finding_id })}
                type="button"
              >
                <span className="t">{finding.summary}</span>
                <span className="m">{finding.severity === "blocking" ? "阻断级检查" : "需要复核"} · {memoryRowDate(finding.updated_at)}</span>
              </button>
            ))}
          </section>

          <section className="memory-list-group" data-memory-group="formal-project">
            <h2 className="mgroup">正式 · 按项目 <span className="n">{scopedProjectFormalMemories.length}</span></h2>
            {visibleProjectFormalMemories.map((item) => (
              <button
                aria-pressed={activeSelection?.kind === "formal" && activeSelection.id === item.memory_id}
                className={`mrow${activeSelection?.kind === "formal" && activeSelection.id === item.memory_id ? " sel" : ""}`}
                data-memory-row-id={item.memory_id}
                data-memory-row-kind="formal"
                key={item.memory_id}
                onClick={() => setMemorySelection({ kind: "formal", id: item.memory_id })}
                type="button"
              >
                <span className="t">{item.claim}</span>
                <span className="m">{item.scope_label} · {item.status_label} · {memoryRowDate(item.updated_at)}</span>
              </button>
            ))}
          </section>

          <section className="memory-list-group" data-memory-group="formal-global">
            <h2 className="mgroup">正式 · 全局 <span className="n">{scopedGlobalFormalMemories.length}</span></h2>
            {visibleGlobalFormalMemories.map((item) => (
              <button
                aria-pressed={activeSelection?.kind === "formal" && activeSelection.id === item.memory_id}
                className={`mrow${activeSelection?.kind === "formal" && activeSelection.id === item.memory_id ? " sel" : ""}`}
                data-memory-row-id={item.memory_id}
                data-memory-row-kind="formal"
                key={item.memory_id}
                onClick={() => setMemorySelection({ kind: "formal", id: item.memory_id })}
                type="button"
              >
                <span className="t">{item.claim}</span>
                <span className="m">{item.scope_label} · {item.status_label} · {memoryRowDate(item.updated_at)}</span>
              </button>
            ))}
          </section>

          {!visibleCandidates.length && !visibleLintFindings.length && !visibleProjectFormalMemories.length && !visibleGlobalFormalMemories.length ? (
            <EmptyState
              what={memoryQuery.trim() ? "没有匹配的记忆" : "暂无记忆"}
              next={memoryQuery.trim() ? "换个词试试" : "去项目页交办一单活，交货时点[属实，沉淀]攒第一条"}
            />
          ) : null}

          <button
            aria-pressed={activeSelection?.kind === "governance"}
            className="memory-governance-trigger"
            onClick={() => {
              if (activeSelection?.kind === "governance") {
                setMemorySelection(governanceReturnSelection);
                return;
              }
              setGovernanceReturnSelection(activeSelection);
              setMemorySelection({ kind: "governance" });
            }}
            type="button"
          >
            更多治理 ▸
          </button>
        </aside>

        <section className="mmain" aria-label="记忆详情">
          <div className="mmain-col">
            <div className="mhead-strip" aria-label="记忆体检">
              <button
                className="mstat"
                disabled={!scopedCandidates.length}
                onClick={() => scopedCandidates[0] && setMemorySelection({ kind: "candidate", id: scopedCandidates[0].candidate_key })}
                type="button"
              >
                <b>{scopedCandidates.length}</b> 条候选等确认
              </button>
              <button
                className="mstat"
                disabled={!scopedLintFindings.length}
                onClick={() => scopedLintFindings[0] && setMemorySelection({ kind: "lint", id: scopedLintFindings[0].finding_id })}
                type="button"
              >
                <b>{scopedLintFindings.length}</b> 条要你看
              </button>
              {summary.task_package_summary.snapshot_count > 0 ? (
                <button
                  className="mstat"
                  disabled={!recalledFormalMemory}
                  onClick={() => {
                    if (!recalledFormalMemory) return;
                    setMemoryScopeFilter(
                      recalledFormalMemory.record.scope.scope_type === "global"
                        ? "global"
                        : recalledFormalMemory.record.scope.project_id ?? "all",
                    );
                    setMemorySelection({ kind: "formal", id: recalledFormalMemory.memory_id });
                  }}
                  type="button"
                >
                  出方案会带上 <b>{summary.task_package_summary.included_count}</b> 条
                </button>
              ) : null}
            </div>

            {activeSelection?.kind === "formal" && primaryFormalMemory ? (
              <FormalMemoryDetail
                item={primaryFormalMemory}
                busyKind={previewingKind}
                error={lifecycleError}
                onLifecycleAction={(operationKind) => void requestLifecycleAction(primaryFormalMemory, operationKind)}
              />
            ) : null}
            {activeSelection?.kind === "candidate" && primaryCandidate ? (
              <CandidateMemoryDetail
                item={primaryCandidate}
                canConfirm={candidateActionsAvailable && primaryCandidateRecord?.status === "candidate_needs_review"}
                canAdopt={candidateActionsAvailable && primaryCandidateRecord?.status === "candidate_confirmed" && !primaryCandidateRecord.adoption}
                canDiscard={candidateActionsAvailable && (primaryCandidateRecord?.status === "candidate_needs_review" || primaryCandidateRecord?.status === "candidate_confirmed")}
                canReject={candidateActionsAvailable && primaryCandidateRecord?.status === "candidate_needs_review"}
                hasOpenLintFinding={primaryCandidateHasOpenLint}
                sourceRefs={primaryCandidateRecord?.source_refs}
                onConfirm={onRequestAction && candidateProjectRoot ? requestCandidateConfirmation : undefined}
                onAdopt={onRequestAction && candidateProjectRoot ? requestCandidateAdoption : undefined}
                onDiscard={onRequestAction && candidateProjectRoot ? () => requestCandidateDecision("candidate_discarded") : undefined}
                onReject={onRequestAction && candidateProjectRoot ? () => requestCandidateDecision("candidate_rejected") : undefined}
                onRequestAction={onRequestAction}
              />
            ) : null}
            {activeSelection?.kind === "lint" && primaryLintFinding ? (
              <MemoryLintFindingDetail
                busyKind={previewingKind}
                error={lifecycleError}
                finding={primaryLintFinding}
                targetMemory={lintTargetFormalMemory}
                onLifecycleAction={(operationKind) => {
                  if (lintTargetFormalMemory) void requestLifecycleAction(lintTargetFormalMemory, operationKind);
                }}
              />
            ) : null}
            {!activeSelection ? (
              <EmptyState what="暂无可展示详情" next="先在左侧列表选一条记忆，或去项目页交办一单活。" />
            ) : null}

            {activeSelection?.kind === "governance" ? (
              <section className="memory-governance-view" data-memory-detail-kind="governance" aria-label="更多治理">
                <header className="memory-governance-heading">
                  <div>
                    <p className="memory-detail-kicker">更多治理</p>
                    <h2>治理面</h2>
                  </div>
                  <button className="memory-governance-back" onClick={() => setMemorySelection(governanceReturnSelection)} type="button">返回详情</button>
                </header>
                <MemoryWorkbenchSummary pageReadModel={pageReadModel} summary={summary} />
                {!advancedShowAll ? (
                  <button className="jiaoban-linklike memory-advanced-showall" type="button" onClick={() => setAdvancedShowAll(true)}>
                    列表默认只显示前几条——点这里显示全部
                  </button>
                ) : null}
                <div className="memory-governance-panels">

        <section className="memory-center-panel memory-entity-relation-panel" aria-label="实体和关系治理">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">实体 / 关系治理</p>
              <h3>实体候选 / 关系候选 / 已确认关系</h3>
            </div>
            <Pill tone={summary.entity_relation_summary.confirmed_relation_count ? "candidate" : "unknown"}>
              {summary.entity_relation_summary.entity_candidates.length +
                summary.entity_relation_summary.merge_candidates.length +
                summary.entity_relation_summary.relation_candidates.length}{" 候选 · "}
              {summary.entity_relation_summary.confirmed_relation_count}{" 已确认"}
            </Pill>
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
            <Pill tone={summary.task_package_summary.snapshot_count ? "candidate" : "unknown"}>
              {summary.task_package_summary.snapshot_count}
            </Pill>
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
            <Pill tone={summary.maintenance_summary.blocking_count ? "warn" : "candidate"}>
              {summary.maintenance_summary.blocking_count}{" 阻断 · "}{summary.maintenance_summary.check_summaries.length}{" 检查"}
            </Pill>
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
            <Pill tone={summary.mature_pattern_summary.user_confirmation_required_count ? "warn" : "unknown"}>
              {summary.mature_pattern_summary.mature_pattern_candidate_count}
            </Pill>
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
            <Pill tone="unknown">{summary.capture_events.length}</Pill>
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
            <Pill tone="unknown">{summary.observation_sources.length}</Pill>
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
            <Pill tone="plain">{summary.project_summaries.length}</Pill>
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
            <Pill tone="plain">{summary.recent_changes.length}</Pill>
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
                </div>
              </section>
            ) : null}
          </div>
        </section>
      </div>
    </section>
  );
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function memoryRowDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}
