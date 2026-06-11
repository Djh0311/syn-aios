import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { PermissionDialog } from "./components/PermissionDialog";
import { RightDetailPanel } from "./components/RightDetailPanel";
import {
  adoptMemoryCandidateToFormalMemory,
  bootstrapProjectWorkflow,
  bindWorkflowNodeCodexSession,
  correctTaskPackageDispatchFields,
  copyTaskPackagePreview,
  createMemoryCandidateFromObservation,
  createProjectConsultationProposal,
  createTaskDraft,
  createMemoryCandidate,
  generateStageCAcceptanceSummary,
  generateTaskPackageFile,
  initializeWorkflowState,
  inspectAutoDispatchAuthorization,
  inspectTaskPackageDispatchReadiness,
  inspectWorkflowRunCheck,
  loadBlackboardCandidateStore,
  loadCodexSessionTranscript,
  loadFormalMemoryStore,
  loadMemoryCaptureStore,
  loadMemoryCandidateStore,
  loadMemoryEntityRelationStore,
  loadMemoryLintStore,
  loadMemoryPatternStore,
  loadObservationStore,
  loadPlanAuthorizationStore,
  loadProjectConsultationProposalStore,
  loadWorkbenchSnapshot,
  loadWorkflowStateSnapshot,
  prepareAuthorizedAutoDispatch,
  prepareOfflineRoleDispatch,
  previewProjectDirectorTaskPlan,
  previewMemoryEntityRelationCandidates,
  previewMaturePatterns,
  recordProjectDirectorProcessFactDecision,
  recordWorkerStructuredReport,
  recordBlackboardCandidateDecision,
  recordGlobalBoundaryReview,
  recordGlobalFinalResultReview,
  recordWorkflowDispatchDirectorReview,
  recordWorkflowPermissionDecision,
  recordMemoryCandidateDecision,
  recordMemoryEntityAliasDecision,
  recordMemoryEntityMergeDecision,
  recordMemoryRelationCandidateDecision,
  recordMaturePatternDecision,
  recordFormalMemoryLifecycleOperation,
  recordOfflineDirectorReview,
  recordOfflineRoleResultHandoff,
  recordProjectConsultationProposalDecision,
  recordUserResultDecision,
  renderTaskPackagePreview,
  previewFormalMemoryLifecycleOperation,
  previewTaskMemoryPacket,
  runProjectWorkflowAutomationPhaseA,
  runMemoryLint,
  runPathAction,
  updateTaskPackageDraftFields,
  updateWorkItemState,
  unbindWorkflowNodeCodexSession,
} from "./lib/tauri";
import { deriveSecretaryContext } from "./lib/secretaryReadModel";
import type { BlackboardCandidateStoreV1, FormalMemoryStoreV1, MemoryCaptureStoreV1, MemoryCandidateStoreV1, MemoryEntityRelationStoreV1, MemoryLintStoreV1, MemoryPatternStoreV1, ObservationStoreV1, PendingAction, PlanAuthorizationStoreV1, PreviewProjectDirectorTaskPlanInput, ProjectConsultationProposalStoreV1, ProjectDirectorTaskPlan, WorkbenchSnapshot, TaskMemoryPacketBuildInput, TaskMemoryPacketBuildOutput, WorkflowStateSnapshot } from "./lib/types";
import { devNavItems, homeNavItem, primaryNavItems, settingsNavItem, workspaceRailItems } from "./lib/workbenchNavigation";
import type { RightPanelKey, ViewKey } from "./lib/workbenchNavigation";
import { AgentView } from "./views/AgentView";
import { CanvasViewWithProvider } from "./views/CanvasView";
import { HarnessBoardView } from "./views/HarnessBoardView";
import { HomeView } from "./views/HomeView";
import { KnowledgeBaseView } from "./views/KnowledgeBaseView";
import { MemoryCenterView } from "./views/MemoryCenterView";
import { ProjectsView } from "./views/ProjectsView";
import { RunningWorkflowsView } from "./views/RunningWorkflowsView";
import { SettingsView } from "./views/SettingsView";
import { SkillsBoardView } from "./views/SkillsBoardView";

export { RightDetailPanel, workspaceRailItems };

const stageKInitialViewKeys = new Set<ViewKey>([
  homeNavItem.key,
  ...primaryNavItems.map((item) => item.key),
  settingsNavItem.key,
  ...devNavItems.map((item) => item.key),
]);

function stageKInitialView(): ViewKey {
  const requested = import.meta.env.VITE_STAGE_K_INITIAL_VIEW;
  if (!requested) return "home";
  return stageKInitialViewKeys.has(requested as ViewKey) ? (requested as ViewKey) : "home";
}

const emptySnapshot: WorkbenchSnapshot = {
  summary: {
    generated_at: null,
    project_count: 0,
    session_count: 0,
    skill_count: 0,
    plugin_count: 0,
    task_count: 0,
    warning_count: 0,
  },
  projects: [],
  sessions: [],
  skills: [],
  plugins: [],
  tasks: [],
  agent_adapters: [],
  session_operations: [],
  provider_availability: [],
  session_continuation_previews: [],
  session_continuation_store: {
    schema_version: "session_continuation_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: null,
      sidecar_path: null,
      project_roots: [],
    },
    revision: 0,
    last_write_id: null,
    generated_by: "empty_snapshot",
    created_at: "未读取",
    updated_at: "未读取",
    continuations: [],
    attempts: [],
    audit_events: [],
    warnings: [],
  },
  runtime_session_attention: [],
  session_run_status_summaries: [],
  runtime_log_store: {
    schema_version: "runtime_log_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: null,
      sidecar_path: null,
      project_roots: [],
    },
    revision: 0,
    last_write_id: null,
    generated_by: "empty_snapshot",
    created_at: "未读取",
    updated_at: "未读取",
    boundary: {
      runtime_log_definition: "运行日志记录运行状态、耗时、分类和脱敏摘要。",
      audit_event_definition: "审计事件记录可追责的决定、权限、操作者和状态变化。",
      separation_rule: "运行日志与审计事件不能互相替代；日志只引用审计引用。",
      redaction_rule: "只展示脱敏摘要，不展示正文、凭据或原始会话记录。",
      forbidden_payloads: ["credential_material", "conversation_body", "raw_provider_material"],
    },
    entries: [],
    summaries: [],
    warnings: [],
  },
  worker_protocol: {
    schema_version: "worker_protocol_read_model.v1",
    generated_at: "未读取",
    source_policy: "empty frontend fallback; no worker execution.",
    worker_adapters: [],
    work_threads: [],
    run_units: [],
    credential_requirements: [],
    external_call_risk_envelopes: [],
    project_capability_policies: [],
    run_relations: [],
    worker_lanes: [],
    multi_worker_dispatch_plans: [],
    adapter_contract_checklists: [],
    controlled_api_cli_semantics: [],
    diagnostic_event_schemas: [],
    adapter_health_summaries: [],
    adapter_degraded_modes: [],
    adapter_data_locations: [],
    dispatch_requests: [],
    dispatch_guards: [],
    permission_envelopes: [],
    task_memory_packet_refs: [],
    worker_handoffs: [],
    readback_results: [],
    worker_report_candidates: [],
    warnings: ["empty_worker_protocol_read_model"],
  },
  page_read_model_inventory: {
    schema_version: "workbench_page_read_model_inventory.v1",
    generated_at: "未读取",
    status: "empty_frontend_fallback",
    source_policy: "空 snapshot 只用于前端兜底；真实合同来自后端 WorkbenchSnapshot。",
    contracts: [],
    warnings: ["empty_page_read_model_inventory"],
  },
  diagnostic_summary: {
    status: "degraded_readonly",
    generated_at: "未读取",
    overall_severity: "warning",
    healthy_count: 0,
    warning_count: 1,
    degraded_count: 0,
    blocked_count: 0,
    store_integrity: [],
    degraded_states: [
      {
        state_id: "empty_snapshot",
        kind: "empty_snapshot",
        severity: "warning",
        title: "当前没有真实诊断数据",
        summary: "空 snapshot 只用于前端兜底展示。",
        user_action_required: false,
        blocks_real_execution: false,
        source_refs: ["empty_snapshot"],
        recommended_next_step: "重新读取索引和事实层。",
      },
    ],
    recent_error_summaries: [],
    boundary_notes: ["G2 诊断只读展示，不自动修复、不自动重试。"],
  },
  diagnostics: {
    index_path: "未读取",
    tasks_path: "未读取",
    generated_at: null,
    top_level_warning_count: 0,
    context_warning_count: 0,
    allowed_project_path_count: 0,
    allowed_rollout_path_count: 0,
    release_bundle_enabled: false,
    notes: ["当前没有真实索引数据。"],
  },
};

export function App() {
  const [snapshot, setSnapshot] = useState<WorkbenchSnapshot | null>(null);
  const [activeView, setActiveView] = useState<ViewKey>(() => stageKInitialView());
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState("");
  const [error, setError] = useState(false);
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [workflowState, setWorkflowState] = useState<WorkflowStateSnapshot | null>(null);
  const [workflowStateLoading, setWorkflowStateLoading] = useState(false);
  const [workflowStateError, setWorkflowStateError] = useState<string | null>(null);
  const [blackboardCandidateStore, setBlackboardCandidateStore] = useState<BlackboardCandidateStoreV1 | null>(null);
  const [planAuthorizationStore, setPlanAuthorizationStore] = useState<PlanAuthorizationStoreV1 | null>(null);
  const [projectConsultationProposalStore, setProjectConsultationProposalStore] =
    useState<ProjectConsultationProposalStoreV1 | null>(null);
  const [observationStore, setObservationStore] = useState<ObservationStoreV1 | null>(null);
  const [memoryCaptureStore, setMemoryCaptureStore] = useState<MemoryCaptureStoreV1 | null>(null);
  const [memoryCandidateStore, setMemoryCandidateStore] = useState<MemoryCandidateStoreV1 | null>(null);
  const [formalMemoryStore, setFormalMemoryStore] = useState<FormalMemoryStoreV1 | null>(null);
  const [memoryLintStore, setMemoryLintStore] = useState<MemoryLintStoreV1 | null>(null);
  const [memoryEntityRelationStore, setMemoryEntityRelationStore] = useState<MemoryEntityRelationStoreV1 | null>(null);
  const [memoryPatternStore, setMemoryPatternStore] = useState<MemoryPatternStoreV1 | null>(null);
  const [activeRightPanel, setActiveRightPanel] = useState<RightPanelKey | null>(null);
  const [focusedAgentThreadId, setFocusedAgentThreadId] = useState<string | null>(null);

  useEffect(() => {
    document.getElementById("tauri-boot-visible-probe")?.remove();

    if (!import.meta.env.DEV) return;

    document.documentElement.dataset.appBoot = "mounted";
    void getCurrentWindow()
      .setTitle("Codex 治理工作台 · 首屏已挂载")
      .catch(() => {
        document.documentElement.dataset.appTitleProbe = "unavailable";
      });
  }, []);

  useEffect(() => {
    void reload();
  }, []);

  async function reload() {
    setNotice("正在读取索引。");
    setError(false);
    try {
      const nextSnapshot = await loadWorkbenchSnapshot();
      setSnapshot(nextSnapshot);
      setNotice("");
      void reloadWorkflowState();
    } catch (loadError) {
      setSnapshot(null);
      setError(true);
      setNotice(`读取失败：${messageOf(loadError)}`);
    }
  }

  async function reloadWorkflowState() {
    setWorkflowStateLoading(true);
    setWorkflowStateError(null);
    try {
      const nextWorkflowState = await loadWorkflowStateSnapshot();
      setWorkflowState(nextWorkflowState);
      void reloadCandidateStores();
    } catch (loadError) {
      setWorkflowState(null);
      setWorkflowStateError(messageOf(loadError));
    } finally {
      setWorkflowStateLoading(false);
    }
  }

  async function reloadCandidateStores() {
    try {
      const [
        nextBlackboardStore,
        nextPlanAuthorizationStore,
        nextProjectConsultationProposalStore,
        nextMemoryCaptureStore,
        nextObservationStore,
        nextMemoryStore,
        nextFormalMemoryStore,
        nextMemoryLintStore,
        nextMemoryEntityRelationStore,
        nextMemoryPatternStore,
      ] = await Promise.all([
        loadBlackboardCandidateStore(),
        loadPlanAuthorizationStore(),
        loadProjectConsultationProposalStore(),
        loadMemoryCaptureStore(),
        loadObservationStore(),
        loadMemoryCandidateStore(),
        loadFormalMemoryStore(),
        loadMemoryLintStore(),
        loadMemoryEntityRelationStore(),
        loadMemoryPatternStore(),
      ]);
      setBlackboardCandidateStore(nextBlackboardStore);
      setPlanAuthorizationStore(nextPlanAuthorizationStore);
      setProjectConsultationProposalStore(nextProjectConsultationProposalStore);
      setMemoryCaptureStore(nextMemoryCaptureStore);
      setObservationStore(nextObservationStore);
      setMemoryCandidateStore(nextMemoryStore);
      setFormalMemoryStore(nextFormalMemoryStore);
      setMemoryLintStore(nextMemoryLintStore);
      setMemoryEntityRelationStore(nextMemoryEntityRelationStore);
      setMemoryPatternStore(nextMemoryPatternStore);
    } catch (loadError) {
      setNotice(`记忆治理读取失败：${messageOf(loadError)}`);
      setError(true);
    }
  }

  async function confirmAction() {
    if (!pendingAction) return;
    setActionBusy(true);
    try {
      if (pendingAction.kind === "initialize-workflow-state") {
        const result = await initializeWorkflowState();
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "bootstrap-project-workflow") {
        const result = await bootstrapProjectWorkflow(pendingAction.path);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "create-task-draft") {
        if (!pendingAction.taskDraft) {
          throw new Error("任务包草稿缺少待写入字段");
        }
        const result = await createTaskDraft({
          project_root: pendingAction.taskDraft.projectRoot,
          title: pendingAction.taskDraft.title,
          objective: pendingAction.taskDraft.objective,
          assigned_role: pendingAction.taskDraft.assignedRole,
        });
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "copy-task-preview") {
        if (!pendingAction.taskPreview) {
          throw new Error("任务包预览缺少待复制字段");
        }
        const result = await copyTaskPackagePreview({
          project_root: pendingAction.taskPreview.projectRoot,
          work_item_id: pendingAction.taskPreview.workItemId,
        });
        setNotice(result);
      } else if (pendingAction.kind === "update-task-fields") {
        if (!pendingAction.taskFields) {
          throw new Error("任务包字段缺少待写入内容");
        }
        const result = await updateTaskPackageDraftFields(pendingAction.taskFields);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "correct-dispatch-fields") {
        if (!pendingAction.dispatchFields) {
          throw new Error("派发字段修正缺少待写入内容");
        }
        const result = await correctTaskPackageDispatchFields(pendingAction.dispatchFields);
        setWorkflowState(result.snapshot);
        setNotice(`${result.message} 请重新检查派发准备。`);
      } else if (pendingAction.kind === "generate-task-file") {
        if (!pendingAction.taskFileGeneration) {
          throw new Error("任务包文件生成缺少待写入对象");
        }
        const result = await generateTaskPackageFile(pendingAction.taskFileGeneration);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "advance-work-item-state") {
        if (!pendingAction.workItemStateUpdate) {
          throw new Error("工作项状态推进缺少待写入对象");
        }
        const result = await updateWorkItemState(pendingAction.workItemStateUpdate);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "bind-node-session") {
        if (!pendingAction.nodeSessionBinding) {
          throw new Error("节点会话绑定缺少待写入对象");
        }
        const result = await bindWorkflowNodeCodexSession(pendingAction.nodeSessionBinding);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "unbind-node-session") {
        if (!pendingAction.nodeSessionUnbinding) {
          throw new Error("节点会话解绑缺少待写入对象");
        }
        const result = await unbindWorkflowNodeCodexSession(pendingAction.nodeSessionUnbinding);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "execute-node-dispatch") {
        if (!pendingAction.nodeDispatch) {
          throw new Error("节点派发缺少待执行对象");
        }
        throw new Error(legacyProductCommandBlockedNotice("execute_workflow_node_dispatch"));
      } else if (pendingAction.kind === "record-director-review") {
        if (!pendingAction.directorReview) {
          throw new Error("总指导回收意见缺少待写入对象");
        }
        const result = await recordWorkflowDispatchDirectorReview(pendingAction.directorReview);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "record-permission-decision") {
        if (!pendingAction.permissionDecision) {
          throw new Error("权限结论缺少待写入对象");
        }
        const result = await recordWorkflowPermissionDecision(pendingAction.permissionDecision);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "record-blackboard-candidate-decision") {
        if (!pendingAction.blackboardCandidateDecision) {
          throw new Error("黑板候选处理缺少待写入对象");
        }
        const result = await recordBlackboardCandidateDecision(pendingAction.blackboardCandidateDecision);
        await reloadCandidateStores();
        setNotice(`黑板候选状态已写入候选 sidecar：${result.record.state}；未写正式事实或正式记忆。`);
      } else if (pendingAction.kind === "create-memory-candidate-from-observation") {
        if (!pendingAction.observationCandidateCreation) {
          throw new Error("observation 生成候选缺少待写入对象");
        }
        const result = await createMemoryCandidateFromObservation(pendingAction.observationCandidateCreation);
        await reloadCandidateStores();
        setNotice(`工作流观察已生成记忆候选：${result.candidate.candidate_key}；候选仍需确认 / 采纳。`);
      } else if (pendingAction.kind === "create-memory-candidate") {
        if (!pendingAction.memoryCandidateCreation) {
          throw new Error("知识库资料生成候选缺少待写入对象");
        }
        const result = await createMemoryCandidate(pendingAction.memoryCandidateCreation);
        await reloadCandidateStores();
        setNotice(`知识库资料已提出记忆候选：${result.candidate.candidate_key}；只写候选 sidecar，未写正式记忆。`);
      } else if (pendingAction.kind === "record-memory-candidate-decision") {
        if (!pendingAction.memoryCandidateDecision) {
          throw new Error("记忆候选处理缺少待写入对象");
        }
        const result = await recordMemoryCandidateDecision(pendingAction.memoryCandidateDecision);
        await reloadCandidateStores();
        setNotice(`记忆候选状态已写入候选 sidecar：${result.candidate.status}；未写正式长期记忆。`);
      } else if (pendingAction.kind === "adopt-memory-candidate-to-formal-memory") {
        if (!pendingAction.memoryCandidateAdoption) {
          throw new Error("记忆候选采纳缺少待写入对象");
        }
        const result = await adoptMemoryCandidateToFormalMemory(pendingAction.memoryCandidateAdoption);
        await reloadCandidateStores();
        setNotice(`记忆候选已受控采纳为正式记忆：${result.record.memory_id}；版本 ${result.version.version_id}；审计 ${result.audit_event.audit_event_id}。`);
      } else if (pendingAction.kind === "record-formal-memory-lifecycle-operation") {
        if (!pendingAction.formalMemoryLifecycle) {
          throw new Error("正式记忆生命周期操作缺少待写入对象");
        }
        const result = await recordFormalMemoryLifecycleOperation(pendingAction.formalMemoryLifecycle);
        await reloadCandidateStores();
        setNotice(
          `正式记忆生命周期已记录：${result.operation_id}；版本 ${result.versions.length}；审计 ${result.audit_event.audit_event_id}。`,
        );
      } else if (pendingAction.kind === "record-memory-entity-alias-decision") {
        if (!pendingAction.memoryEntityAliasDecision) {
          throw new Error("实体候选别名决定缺少待写入对象");
        }
        const result = await recordMemoryEntityAliasDecision(pendingAction.memoryEntityAliasDecision);
        await reloadCandidateStores();
        setNotice(`实体候选决定已写入 M10 辅助状态文件：${result.candidate.status}；未写正式记忆。`);
      } else if (pendingAction.kind === "record-memory-entity-merge-decision") {
        if (!pendingAction.memoryEntityMergeDecision) {
          throw new Error("实体去重候选决定缺少待写入对象");
        }
        const result = await recordMemoryEntityMergeDecision(pendingAction.memoryEntityMergeDecision);
        await reloadCandidateStores();
        setNotice(`实体去重候选决定已写入 M10 辅助状态文件：${result.merge_candidate.status}；未改正式记忆。`);
      } else if (pendingAction.kind === "record-memory-relation-candidate-decision") {
        if (!pendingAction.memoryRelationCandidateDecision) {
          throw new Error("关系候选决定缺少待写入对象");
        }
        const result = await recordMemoryRelationCandidateDecision(pendingAction.memoryRelationCandidateDecision);
        await reloadCandidateStores();
        setNotice(`关系候选决定已写入 M10 辅助状态文件：${result.relation_candidate.status}；已确认关系只用于解释召回原因。`);
      } else if (pendingAction.kind === "run-memory-maintenance") {
        if (!pendingAction.memoryMaintenanceRun) {
          throw new Error("记忆维护任务缺少待运行对象");
        }
        const result = await runMemoryLint(pendingAction.memoryMaintenanceRun);
        await reloadCandidateStores();
        setNotice(
          `记忆维护任务已写入检查辅助状态文件：新增发现 ${result.new_findings.length} / 阻断 ${result.blocking_count}；${result.report?.display_text ?? "未生成维护报告"}。`,
        );
      } else if (pendingAction.kind === "record-mature-pattern-decision") {
        if (!pendingAction.maturePatternDecision) {
          throw new Error("成熟模式候选处理缺少待写入对象");
        }
        const result = await recordMaturePatternDecision(pendingAction.maturePatternDecision);
        await reloadCandidateStores();
        if (result.formal_memory_output) {
          setNotice(
          `成熟模式候选已由用户确认并受控写入正式记忆：${result.formal_memory_output.record.memory_id}；版本 ${result.formal_memory_output.version.version_id}；审计 ${result.formal_memory_output.audit_event.audit_event_id}。`,
          );
        } else {
          setNotice(`成熟模式候选决定已写入 M12 sidecar：${result.candidate.status}；未写正式记忆。`);
        }
      } else if (pendingAction.kind === "create-project-consultation-proposal") {
        if (!pendingAction.projectConsultationProposalCreation) {
          throw new Error("项目咨询方案草案缺少待写入对象");
        }
        const result = await createProjectConsultationProposal(pendingAction.projectConsultationProposalCreation);
        await reloadCandidateStores();
        setNotice(`项目咨询方案草案已创建：${result.proposal.status}；等待用户确认，不会启动真实工作者。`);
      } else if (pendingAction.kind === "record-project-consultation-proposal-decision") {
        if (!pendingAction.projectConsultationProposalDecision) {
          throw new Error("项目咨询方案决定缺少待写入对象");
        }
        const result = await recordProjectConsultationProposalDecision(pendingAction.projectConsultationProposalDecision);
        await reloadCandidateStores();
        if (result.plan_authorization) {
          setNotice(`已记录用户确认并创建方案授权：${result.plan_authorization.status}；仍需全局主管复核后才可自动推进。`);
        } else {
          setNotice(`项目咨询方案决定已记录：${result.decision.decision}；未创建方案授权，未启动真实工作者。`);
        }
      } else if (pendingAction.kind === "record-global-boundary-review") {
        if (!pendingAction.globalBoundaryReview) {
          throw new Error("全局边界复核缺少待写入对象");
        }
        const result = await recordGlobalBoundaryReview(pendingAction.globalBoundaryReview);
        await reloadCandidateStores();
        if (result.authorization.status === "active") {
          setNotice(`全局边界复核已通过，授权有效：${result.authorization.authorization_id}；仍未派发工作者。`);
        } else {
          setNotice(`全局边界复核已记录：${result.authorization.status}；不能自动推进。`);
        }
      } else if (pendingAction.kind === "prepare-authorized-auto-dispatch") {
        if (!pendingAction.authorizedAutoDispatch) {
          throw new Error("授权准备派发缺少待写入对象");
        }
        const result = await prepareAuthorizedAutoDispatch(pendingAction.authorizedAutoDispatch);
        setWorkflowState(result.snapshot);
        await reloadCandidateStores();
        setNotice(
          `准备派发已记录；仍未执行工作者。计划 ${result.plan.planned_task_count} / 已准备 ${result.plan.prepared_dispatch_count} / 需绑定 ${result.plan.needs_binding_count} / 阻断 ${result.plan.blocked_count}`,
        );
      } else if (pendingAction.kind === "record-worker-structured-report") {
        if (!pendingAction.workerStructuredReport) {
          throw new Error("工作者结构化汇报缺少待写入对象");
        }
        const result = await recordWorkerStructuredReport(pendingAction.workerStructuredReport);
        setWorkflowState(result.snapshot);
        setNotice(`${result.message} 等待项目主管确认过程事实。`);
      } else if (pendingAction.kind === "record-project-director-process-fact-decision") {
        if (!pendingAction.processFactDecision) {
          throw new Error("过程事实确认缺少待写入对象");
        }
        const result = await recordProjectDirectorProcessFactDecision(pendingAction.processFactDecision);
        setWorkflowState(result.snapshot);
        await reloadCandidateStores();
        setNotice(result.message);
      } else if (pendingAction.kind === "record-global-final-result-review") {
        if (!pendingAction.globalFinalResultReview) {
          throw new Error("全局最终复核缺少待写入对象");
        }
        const result = await recordGlobalFinalResultReview(pendingAction.globalFinalResultReview);
        setWorkflowState(result.snapshot);
        setNotice(`${result.message} 仍需用户查看并作出结果决定。`);
      } else if (pendingAction.kind === "record-user-result-decision") {
        if (!pendingAction.userResultDecision) {
          throw new Error("用户结果决定缺少待写入对象");
        }
        const result = await recordUserResultDecision(pendingAction.userResultDecision);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "generate-stage-c-acceptance-summary") {
        if (!pendingAction.stageCAcceptanceSummary) {
          throw new Error("阶段 C 验收摘要缺少待写入对象");
        }
        const result = await generateStageCAcceptanceSummary(pendingAction.stageCAcceptanceSummary);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "run-project-workflow-automation-phase-a") {
        if (!pendingAction.projectWorkflowAutomation) {
          throw new Error("项目自动编排缺少待写入对象");
        }
        const result = await runProjectWorkflowAutomationPhaseA(pendingAction.projectWorkflowAutomation);
        const nextSnapshot = await loadWorkbenchSnapshot();
        setSnapshot(nextSnapshot);
        await reloadWorkflowState();
        setNotice(
          `项目自动编排 Level A 已记录：${result.plan.run_units.length} 个 run unit；状态 ${result.status}；未发送 prompt、未执行真实 Codex。`,
        );
      } else if (pendingAction.kind === "offline-role-dispatch") {
        if (!pendingAction.offlineRoleDispatch) {
          throw new Error("离线角色派发缺少派发块");
        }
        const result = await prepareOfflineRoleDispatch(pendingAction.offlineRoleDispatch);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "offline-role-result-handoff") {
        if (!pendingAction.offlineRoleResultHandoff) {
          throw new Error("离线角色回传缺少待写入对象");
        }
        const result = await recordOfflineRoleResultHandoff(pendingAction.offlineRoleResultHandoff);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "offline-director-review") {
        if (!pendingAction.offlineDirectorReview) {
          throw new Error("离线总指导回收缺少待写入对象");
        }
        const result = await recordOfflineDirectorReview(pendingAction.offlineDirectorReview);
        setWorkflowState(result.snapshot);
        setNotice(result.message);
      } else if (pendingAction.kind === "run-workflow-machine") {
        if (!pendingAction.workflowMachineRun) {
          throw new Error("工作流机器缺少运行参数");
        }
        throw new Error(legacyProductCommandBlockedNotice("run_workflow_machine"));
      } else {
        const result = await runPathAction(pendingAction);
        setNotice(result);
      }
      setError(false);
      setPendingAction(null);
    } catch (actionError) {
      setNotice(`动作失败：${messageOf(actionError)}`);
      setError(true);
      setPendingAction(null);
    } finally {
      setActionBusy(false);
    }
  }

  const filteredSnapshot = useMemo(() => {
    if (!snapshot || !query.trim()) return snapshot;
    const needle = query.trim().toLowerCase();
    return {
      ...snapshot,
      projects: snapshot.projects.filter((project) =>
        [project.name, project.project_root, ...project.context_warnings].some((value) => value.toLowerCase().includes(needle)),
      ),
      sessions: snapshot.sessions.filter((session) =>
        [session.title, session.thread_id, session.project_root ?? "", session.model ?? ""].some((value) =>
          value.toLowerCase().includes(needle),
        ),
      ),
      skills: snapshot.skills.filter((skill) =>
        [skill.title, skill.skill_id, skill.path, skill.plugin_name ?? "", skill.source_type].some((value) =>
          value.toLowerCase().includes(needle),
        ),
      ),
      plugins: snapshot.plugins.filter((plugin) =>
        [plugin.plugin_name, plugin.plugin_version, plugin.homepage ?? ""].some((value) => value.toLowerCase().includes(needle)),
      ),
    };
  }, [snapshot, query]);

  const rightStats = useMemo(() => {
    const summary = filteredSnapshot?.summary;
    const counts = workflowState?.counts;
    return [
      { label: "项目", value: summary?.project_count ?? filteredSnapshot?.projects.length ?? 0 },
      { label: "会话", value: summary?.session_count ?? filteredSnapshot?.sessions.length ?? 0 },
      { label: "工作流", value: counts?.workflows ?? 0 },
      { label: "工作项", value: counts?.work_items ?? 0 },
    ];
  }, [filteredSnapshot, workflowState]);

  const displaySnapshot = filteredSnapshot ?? emptySnapshot;
  const secretaryContext = useMemo(
    () =>
      deriveSecretaryContext({
        snapshot: displaySnapshot,
        workflowState,
        blackboardCandidateStore,
        memoryCaptureStore,
        memoryCandidateStore,
        workflowStateError,
      }),
    [displaySnapshot, workflowState, blackboardCandidateStore, memoryCaptureStore, memoryCandidateStore, workflowStateError],
  );
  const pendingReviewCount =
    workflowState?.project_workflows.reduce(
      (count, workflow) => count + workflow.task_drafts.filter((task) => task.state === "ready_for_review").length,
      0,
    ) ?? 0;
  const topbarReviewCount = pendingReviewCount + (workflowState?.counts.reviews ?? 0);
  const isDeveloperView = devNavItems.some((item) => item.key === activeView);

  return (
    <div className={`app-shell ${activeRightPanel ? "right-pane-open" : ""}`}>
      <header className="shell-topbar ink-shell">
        <label className="search-box">
          <span aria-hidden="true">⌕</span>
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="跨项目 · 跨智能体 · 跨工作流" aria-label="跨项目、跨智能体、跨工作流检索" />
          <kbd>⌘K</kbd>
        </label>
        <div className="topbar-actions">
          {topbarReviewCount > 0 ? (
            <button className="pending-review-button" type="button" onClick={() => setActiveRightPanel("todos")}>
              {topbarReviewCount} 待审
            </button>
          ) : null}
          <span className="meta-text">{displaySnapshot.summary.project_count} 项目</span>
          <button className="secondary-button icon-button" type="button" onClick={() => void reload()} aria-label="重新读取">↺</button>
          <span className={`top-health-dot ${error ? "error" : ""}`} title={error ? "需处理" : "可用"} aria-label={error ? "需处理" : "可用"} />
        </div>
      </header>

      <aside className="sidebar ink-shell">
        <div className="sidebar-inner">
          <button
            className={`brand brand-home-button ${activeView === "home" ? "active" : ""}`}
            type="button"
            onClick={() => setActiveView("home")}
            title="首页"
          >
            <span className="brand-mark">案</span>
            <span className="brand-name">本地智能工作台</span>
          </button>
          <nav className="sidebar-nav" aria-label="主导航">
            <p className="nav-section-label">工作台</p>
            <div className="nav-list">
              {primaryNavItems.map((item) => (
                <button
                  className={`nav-item ${activeView === item.key ? "active" : ""}`}
                  key={item.key}
                  type="button"
                  onClick={() => setActiveView(item.key)}
                  title={item.label}
                >
                  <span className="nav-glyph" aria-hidden="true">{item.glyph}</span>
                  <span className="nav-label">{item.label}</span>
                </button>
              ))}
            </div>
            <div className="nav-list settings-nav-list" aria-label="设置入口">
              <button
                className={`nav-item ${activeView === settingsNavItem.key || isDeveloperView ? "active" : ""}`}
                type="button"
                onClick={() => setActiveView(settingsNavItem.key)}
                title={settingsNavItem.label}
              >
                <span className="nav-glyph" aria-hidden="true">{settingsNavItem.glyph}</span>
                <span className="nav-label">{settingsNavItem.label}</span>
              </button>
            </div>
          </nav>
        </div>
      </aside>

      <main
        className={`main-panel stage ${activeView === "projects" ? "project-stage" : ""} ${
          activeView === "agents" ? "agent-stage" : ""
        }`}
      >
        {notice || error ? (
          <section className={`notice-panel ${error ? "error" : ""}`} aria-live="polite">
            <strong>{error ? "需要处理" : "状态"}</strong>
            <span>{notice}</span>
          </section>
        ) : null}

        {renderActiveView(
          activeView,
          displaySnapshot,
          setPendingAction,
          setActiveView,
          workflowState,
          workflowStateLoading,
          workflowStateError,
          reloadWorkflowState,
          setNotice,
          Boolean(filteredSnapshot),
          (threadId) => {
            setFocusedAgentThreadId(threadId);
            setActiveView("agents");
          },
          focusedAgentThreadId,
          blackboardCandidateStore,
          planAuthorizationStore,
          projectConsultationProposalStore,
          observationStore,
          memoryCaptureStore,
          memoryCandidateStore,
          formalMemoryStore,
          memoryLintStore,
          memoryEntityRelationStore,
          memoryPatternStore,
          previewTaskMemoryPacket,
          previewProjectDirectorTaskPlan,
          previewFormalMemoryLifecycleOperation,
          previewMemoryEntityRelationCandidates,
          previewMaturePatterns,
        )}
      </main>

      <aside className="status-rail ink-shell" aria-label="工作台入口">
        <div className="right-icon-strip">
          {workspaceRailItems.map((item) => (
            <button
              className={`rail-icon-button ${activeRightPanel === item.key ? "active" : ""}`}
              key={item.key}
              type="button"
              title={item.label}
              aria-label={item.label}
              aria-expanded={activeRightPanel === item.key}
              onClick={() => setActiveRightPanel((current) => (current === item.key ? null : item.key))}
            >
              <span aria-hidden="true">{item.glyph}</span>
            </button>
          ))}
          <div className="rail-mini-stats" aria-label="工作台状态摘要">
            {rightStats.map((stat) => (
              <span key={stat.label} title={`${stat.label} ${stat.value}`}>
                {stat.value}
              </span>
            ))}
          </div>
          <span
            className={`rail-health-dot ${error || workflowStateError ? "error" : workflowStateLoading ? "loading" : ""}`}
            title={workflowStateError ?? (workflowStateLoading ? "读取中" : "状态可用")}
            aria-label={workflowStateError ?? (workflowStateLoading ? "读取中" : "状态可用")}
          />
        </div>
        {activeRightPanel ? (
          <RightDetailPanel
            activePanel={activeRightPanel}
            snapshot={displaySnapshot}
            workflowState={workflowState}
            notice={notice}
            error={error || Boolean(workflowStateError)}
            workflowStateError={workflowStateError}
            memoryCaptureStore={memoryCaptureStore}
            memoryCandidateStore={memoryCandidateStore}
            secretaryContext={secretaryContext}
            onClose={() => setActiveRightPanel(null)}
            onNavigate={setActiveView}
            onReloadWorkflowState={reloadWorkflowState}
          />
        ) : null}
      </aside>

      <footer className="dock ink-shell" aria-label="秘书对话框">
        <button
          className="secretary secretary-dock-trigger"
          type="button"
          onClick={() => setActiveRightPanel("secretary")}
          aria-label="打开秘书对话"
        >
          <span className="secretary-orb" aria-hidden="true" />
          <span>秘 书 · 辅 助</span>
        </button>
        <div className="dock-input-wrap">
          <span className="prompt" aria-hidden="true">›</span>
          <input
            className="dock-input"
            readOnly
            onFocus={() => setActiveRightPanel("secretary")}
            onClick={() => setActiveRightPanel("secretary")}
            placeholder="让秘书解释、整理、提醒或说明影响面（预览）"
            aria-label="秘书对话输入预览，点击打开秘书"
          />
          <div className="dock-chips" aria-label="秘书快捷入口">
            <button className="chip" type="button" onClick={() => setActiveRightPanel("secretary")}>解释</button>
            <button className="chip" type="button" onClick={() => setActiveRightPanel("secretary")}>整理</button>
            <button className="chip" type="button" onClick={() => setActiveRightPanel("secretary")}>提醒</button>
            <button className="chip" type="button" onClick={() => setActiveRightPanel("secretary")}>影响面</button>
            <button className="chip send" type="button" onClick={() => setActiveRightPanel("secretary")}>打开秘书</button>
          </div>
        </div>
      </footer>

      <PermissionDialog
        action={pendingAction}
        busy={actionBusy}
        onCancel={() => setPendingAction(null)}
        onConfirm={() => void confirmAction()}
      />

      <button
        className={`secretary-float ${activeRightPanel === "secretary" ? "active" : ""}`}
        type="button"
        aria-label="打开秘书"
        onClick={() => setActiveRightPanel((current) => (current === "secretary" ? null : "secretary"))}
      >
        <span aria-hidden="true">秘</span>
        {topbarReviewCount > 0 ? <i>{topbarReviewCount}</i> : null}
      </button>
    </div>
  );
}

function renderActiveView(
  view: ViewKey,
  snapshot: WorkbenchSnapshot,
  onRequestAction: (action: PendingAction) => void,
  onNavigate: (view: ViewKey) => void,
  workflowState: WorkflowStateSnapshot | null,
  workflowStateLoading: boolean,
  workflowStateError: string | null,
  onReloadWorkflowState: () => void,
  onNotice: (msg: string) => void,
  hasRealSnapshot: boolean,
  onOpenAgentSession: (threadId: string) => void,
  focusedAgentThreadId?: string | null,
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null,
  planAuthorizationStore?: PlanAuthorizationStoreV1 | null,
  projectConsultationProposalStore?: ProjectConsultationProposalStoreV1 | null,
  observationStore?: ObservationStoreV1 | null,
  memoryCaptureStore?: MemoryCaptureStoreV1 | null,
  memoryCandidateStore?: MemoryCandidateStoreV1 | null,
  formalMemoryStore?: FormalMemoryStoreV1 | null,
  memoryLintStore?: MemoryLintStoreV1 | null,
  memoryEntityRelationStore?: MemoryEntityRelationStoreV1 | null,
  memoryPatternStore?: MemoryPatternStoreV1 | null,
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>,
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>,
  onPreviewFormalMemoryLifecycle?: Parameters<typeof MemoryCenterView>[0]["onPreviewFormalMemoryLifecycle"],
  onPreviewMemoryEntityRelationCandidates?: Parameters<typeof MemoryCenterView>[0]["onPreviewMemoryEntityRelationCandidates"],
  onPreviewMaturePatterns?: Parameters<typeof MemoryCenterView>[0]["onPreviewMaturePatterns"],
) {
  if (view === "agents") {
    return (
      <AgentView
        sessions={snapshot.sessions}
        projects={snapshot.projects}
        adapterDescriptors={snapshot.agent_adapters}
        sessionOperationDescriptors={snapshot.session_operations}
        providerAvailabilitySummaries={snapshot.provider_availability}
        sessionContinuationPreviews={snapshot.session_continuation_previews}
        sessionContinuationStore={snapshot.session_continuation_store}
        runtimeSessionAttention={snapshot.runtime_session_attention}
        sessionRunStatusSummaries={snapshot.session_run_status_summaries}
        realExecutionProductCommands={snapshot.real_execution_product_commands}
        projectWorkflowAutomation={snapshot.project_workflow_automation}
        workerProtocol={snapshot.worker_protocol}
        workflowState={workflowState}
        focusedThreadId={focusedAgentThreadId}
        onLoadTranscript={loadCodexSessionTranscript}
        onRequestAction={onRequestAction}
      />
    );
  }

  if (view === "projects") {
    return (
      <ProjectsView
        projects={snapshot.projects}
        sessions={snapshot.sessions}
        workflowState={workflowState}
        blackboardCandidateStore={blackboardCandidateStore}
        planAuthorizationStore={planAuthorizationStore}
        projectConsultationProposalStore={projectConsultationProposalStore}
        observationStore={observationStore}
        memoryCandidateStore={memoryCandidateStore}
        formalMemoryStore={formalMemoryStore}
        memoryLintStore={memoryLintStore}
        runtimeSessionAttention={snapshot.runtime_session_attention}
        realExecutionProductCommands={snapshot.real_execution_product_commands}
        projectWorkflowAutomation={snapshot.project_workflow_automation}
        workflowStateLoading={workflowStateLoading}
        workflowStateError={workflowStateError}
        onReloadWorkflowState={onReloadWorkflowState}
        onRequestAction={onRequestAction}
        onLoadTranscript={loadCodexSessionTranscript}
        onRenderTaskPreview={(projectRoot, workItemId) =>
          renderTaskPackagePreview({ project_root: projectRoot, work_item_id: workItemId })
        }
        onInspectDispatchReadiness={(projectRoot, workItemId) =>
          inspectTaskPackageDispatchReadiness({ project_root: projectRoot, work_item_id: workItemId })
        }
        onInspectWorkflowRunCheck={(projectRoot, workflowId) =>
          inspectWorkflowRunCheck({ project_root: projectRoot, workflow_id: workflowId })
        }
        onInspectAutoDispatchAuthorization={inspectAutoDispatchAuthorization}
        onPreviewTaskMemoryPacket={onPreviewTaskMemoryPacket}
        onPreviewProjectDirectorTaskPlan={onPreviewProjectDirectorTaskPlan}
        onOpenAgentSession={onOpenAgentSession}
      />
    );
  }

  if (view === "skills") {
    return <SkillsBoardView skills={snapshot.skills} plugins={snapshot.plugins} projects={snapshot.projects} />;
  }

  if (view === "harness") {
    return <HarnessBoardView projects={snapshot.projects} />;
  }

  if (view === "runningWorkflows") {
    return (
      <RunningWorkflowsView
        snapshot={snapshot}
        workflowState={workflowState}
        workflowStateLoading={workflowStateLoading}
        workflowStateError={workflowStateError}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        onReloadWorkflowState={onReloadWorkflowState}
        onNavigate={onNavigate}
      />
    );
  }

  if (view === "workflow") {
    return (
      <CanvasViewWithProvider
        canvasId="default"
        sessions={snapshot.sessions}
        onNotice={onNotice}
      />
    );
  }

  if (view === "ideas") {
    return <SourceStylePlaceholder title="想法箱" kicker="想法" hasRealSnapshot={hasRealSnapshot} items={snapshot.tasks.map((task) => `${task.status} · ${task.title}`)} />;
  }

  if (view === "proposal") {
    return <SourceStylePlaceholder title="建议方案" kicker="方案" hasRealSnapshot={hasRealSnapshot} items={snapshot.projects.slice(0, 4).map((project) => `${project.name} · ${project.context_warnings.length + project.warnings.length} 条警告`)} />;
  }

  if (view === "knowledge") {
    return (
      <KnowledgeBaseView
        projects={snapshot.projects}
        workflowState={workflowState}
        formalMemoryStore={formalMemoryStore}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        hasRealSnapshot={hasRealSnapshot}
        onRequestAction={onRequestAction}
      />
    );
  }

  if (view === "memory") {
    return (
      <MemoryCenterView
        projects={snapshot.projects}
        workflowState={workflowState}
        formalMemoryStore={formalMemoryStore}
        memoryCaptureStore={memoryCaptureStore}
        memoryCandidateStore={memoryCandidateStore}
        observationStore={observationStore}
        memoryLintStore={memoryLintStore}
        memoryEntityRelationStore={memoryEntityRelationStore}
        memoryPatternStore={memoryPatternStore}
        hasRealSnapshot={hasRealSnapshot}
        onRequestAction={onRequestAction}
        onPreviewFormalMemoryLifecycle={onPreviewFormalMemoryLifecycle}
        onPreviewMemoryEntityRelationCandidates={onPreviewMemoryEntityRelationCandidates}
        onPreviewMaturePatterns={onPreviewMaturePatterns}
      />
    );
  }

  if (view === "tools") {
    return <SourceStylePlaceholder title="工具" kicker="工具" hasRealSnapshot={hasRealSnapshot} items={snapshot.projects.flatMap((project) => project.harness_resources.map((resource) => `${project.name} · ${resource.display_name ?? resource.root_path}`)).slice(0, 6)} />;
  }

  if (view === "models") {
    return <SourceStylePlaceholder title="模型 / 凭据" kicker="模型" hasRealSnapshot={hasRealSnapshot} items={["未接真实模型池和凭据状态；入口已按源稿保留。"]} />;
  }

  if (view === "settings") {
    return (
      <SettingsView
        snapshot={snapshot}
        workflowState={workflowState}
        workflowStateError={workflowStateError}
        hasRealSnapshot={hasRealSnapshot}
        developerItems={devNavItems}
        onNavigate={onNavigate}
      />
    );
  }

  return <HomeView snapshot={snapshot} workflowState={workflowState} onNavigate={onNavigate} />;
}

function SourceStylePlaceholder({
  title,
  kicker,
  hasRealSnapshot,
  items,
}: {
  title: string;
  kicker: string;
  hasRealSnapshot: boolean;
  items: string[];
}) {
  return (
    <section className="stage-pad source-placeholder">
      <div className="pg-head">
        <div>
          <p className="pg-sub">{kicker}</p>
          <h1 className="pg-title">{title}</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{hasRealSnapshot ? "索引已读取" : "未接真实数据"}</div>
          <div>只读入口；不足部分不冒充真实完成</div>
        </div>
      </div>
      <div className="cols cols-2-eq">
        <section className="panel">
          <div className="panel-h">
            当前内容
            <span className="count">{items.length}</span>
          </div>
          {items.length ? (
            items.map((item) => (
              <div className="card" key={item}>
                <div className="c-head">
                  <span className="c-title">{item.split(" · ")[0]}</span>
                  <span className="c-meta">只读</span>
                </div>
                <div className="c-body">{item}</div>
              </div>
            ))
          ) : (
            <p className="muted small-note">当前索引没有提供可展示的真实条目。</p>
          )}
        </section>
        <section className="panel">
          <div className="panel-h">
            接入状态
            <span className="count">占位</span>
          </div>
          <div className="card lit">
            <div className="c-head">
              <span className="c-title">未接真实能力时仅展示入口</span>
              <span className="c-meta">边界</span>
            </div>
            <div className="c-body">这里按源稿结构保留页面，不用假数据冒充已经接入。</div>
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

function legacyProductCommandBlockedNotice(commandName: string) {
  return `${commandName} 是旧真实执行入口，已在 K2.5 封存；请改走统一 Product Command，不再从普通 UI 调用 legacy Tauri wrapper。`;
}
