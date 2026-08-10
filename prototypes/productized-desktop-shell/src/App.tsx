import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { renderActiveWorkbenchView } from "./components/ActiveWorkbenchView";
import { RightDetailPanel } from "./components/RightDetailPanel";
import { SecretaryBoardView } from "./components/SecretaryBoardView";
import { WorkbenchShell } from "./components/WorkbenchShell";
import { humanizeNoticeMessage } from "./lib/humanize";
import {
  adoptMemoryCandidateToFormalMemory,
  bootstrapProjectWorkflow,
  bindWorkflowNodeCodexSession,
  correctTaskPackageDispatchFields,
  copyTaskPackagePreview,
  captureMemoryEvent,
  createMemoryCandidateFromObservation,
  createProjectConsultationProposal,
  runProjectConsultation,
  createTaskDraft,
  createMemoryCandidate,
  generateStageCAcceptanceSummary,
  generateTaskPackageFile,
  initializeWorkflowState,
  loadBlackboardCandidateStore,
  loadFormalMemoryStore,
  loadMemoryCaptureStore,
  loadMemoryCandidateStore,
  loadMemoryEntityRelationStore,
  loadGlobalSupervisorReviewStore,
  loadMemoryLintStore,
  loadMemoryPatternStore,
  loadObservationStore,
  loadPlanAuthorizationStore,
  loadSecretaryHomeContext,
  loadProjectConsultationProposalStore,
  loadSystemStatusReadModel,
  loadWorkflowStateSnapshot,
  prepareAuthorizedAutoDispatch,
  prepareOfflineRoleDispatch,
  previewProjectDirectorTaskPlan,
  previewMemoryEntityRelationCandidates,
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
  recordOperationControlDecision,
  recordProjectConsultationProposalDecision,
  recordUserResultDecision,
  queryWorkbenchPageReadModel,
  operateSecretaryCoordination,
  previewFormalMemoryLifecycleOperation,
  previewTaskMemoryPacket,
  runProjectWorkflowAutomationPhaseA,
  runMemoryLint,
  runPathAction,
  knowledgeVaultAiWrite,
  acknowledgeKnowledgeOpenRelayIntent as acknowledgeKnowledgeOpenRelayIntentAtHost,
  updateTaskPackageDraftFields,
  updateWorkItemState,
  unbindWorkflowNodeCodexSession,
} from "./lib/tauri";
import {
  KNOWLEDGE_OPEN_RELAY_EVENT_NAME,
  parseKnowledgeOpenRelayIntent,
  sameKnowledgeOpenRelayIntent,
  type KnowledgeOpenRelayIntent,
  type KnowledgeOpenRelayOutcome,
} from "./lib/knowledgeOpenRelay";
import type { SystemStatusReadModel } from "./lib/tauri";
import {
  browserPreviewSessionPage,
  browserPreviewSnapshot,
  browserPreviewTranscript,
} from "./lib/browserPreviewSnapshot";
import {
  browserPreviewPlanAuthorizationStore,
  browserPreviewProposalStore,
  browserPreviewWorkflowState,
} from "./lib/browserPreviewWorkflowState";
import { emptySnapshot } from "./lib/emptySnapshot";
import { loadWorkbenchSnapshotFromPageQueries } from "./lib/pageReadModelRuntime";
import {
  deriveSecretaryContext,
  deriveSecretaryHomeReadModel,
  mintSecretaryCoordinationIdempotencyKey,
} from "./lib/secretaryReadModel";
import { setTauriWindowTitle } from "./lib/tauriWindow";
import type {
  M4SecretaryCoordinationActionRequestDto,
  M4SecretaryHomeContextEnvelopeDto,
  SecretaryHomeReadModel,
  SecretaryTypedDeepLinkDescriptor,
} from "./lib/types/m4Secretary";
import type { BlackboardCandidateStoreV1, FormalMemoryStoreV1, GlobalSupervisorReviewStoreV1, MemoryCaptureStoreV1, MemoryCandidateStoreV1, MemoryEntityRelationStoreV1, MemoryLintStoreV1, MemoryPatternStoreV1, ObservationStoreV1, PendingAction, PlanAuthorizationStoreV1, ProjectConsultationProposalStoreV1, WorkbenchSnapshot, WorkflowStateSnapshot } from "./lib/types";
import { devNavItems, homeNavItem, primaryNavItems, settingsNavItem, workspaceRailItems } from "./lib/workbenchNavigation";
import type { NavigateHandler, NavigationFocus, RightPanelKey, ViewKey } from "./lib/workbenchNavigation";
import { HomeView, type SecretaryCoordinationIntent, type SecretaryCoordinationViewState } from "./views/HomeView";

export { RightDetailPanel, workspaceRailItems };

const viteEnv = import.meta.env ?? {};
const browserPreviewEnabled = viteEnv.DEV === true && !("__TAURI_INTERNALS__" in window);

type SecretaryHomeTransportState = "loading" | "loaded" | "error";

const stageKInitialViewKeys = new Set<ViewKey>([
  homeNavItem.key,
  ...primaryNavItems.map((item) => item.key),
  settingsNavItem.key,
  ...devNavItems.map((item) => item.key),
]);

function stageKInitialView(): ViewKey {
  const requested = viteEnv.VITE_STAGE_K_INITIAL_VIEW;
  if (!requested) return "home";
  return stageKInitialViewKeys.has(requested as ViewKey) ? (requested as ViewKey) : "home";
}

export function App() {
  const [snapshot, setSnapshot] = useState<WorkbenchSnapshot | null>(null);
  const [activeView, setActiveView] = useState<ViewKey>(() => stageKInitialView());
  const [knowledgeOpenIntent, setKnowledgeOpenIntent] = useState<KnowledgeOpenRelayIntent | null>(null);
  // ④「点击带上下文直达」:导航焦点与 activeView 同一次更新落地——跳哪一页 + 落在哪一条。
  // 不带 focus 的导航把它清空,免得旧焦点粘在下一页上。
  const [navigationFocus, setNavigationFocus] = useState<NavigationFocus | null>(null);
  const navigate = useCallback<NavigateHandler>((view, focus) => {
    setActiveView(view);
    setNavigationFocus(focus ?? null);
  }, []);
  const [query, setQuery] = useState("");
  const [notice, setNotice] = useState("");
  const [error, setError] = useState(false);
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [workflowState, setWorkflowState] = useState<WorkflowStateSnapshot | null>(null);
  const [workflowStateLoading, setWorkflowStateLoading] = useState(false);
  const [workflowStateError, setWorkflowStateError] = useState<string | null>(null);
  const [systemStatus, setSystemStatus] = useState<SystemStatusReadModel | null>(null);
  // The envelope is an authoritative transport result only.  No local role,
  // scope, context, source owner or attention record is reconstructed here.
  const [secretaryHomeEnvelope, setSecretaryHomeEnvelope] = useState<M4SecretaryHomeContextEnvelopeDto | null>(null);
  const [secretaryHomeTransport, setSecretaryHomeTransport] = useState<SecretaryHomeTransportState>("loading");
  const [secretaryHomeErrorCode, setSecretaryHomeErrorCode] = useState<string | null>(null);
  const [secretaryCoordinationStates, setSecretaryCoordinationStates] = useState<
    Readonly<Record<string, SecretaryCoordinationViewState>>
  >({});
  // Exact retry material only. It is never an identity/source cache and is
  // discarded after a repository receipt. Keeping the full request makes an
  // ambiguous transport retry reuse both its idempotency key and snooze time.
  const secretaryCoordinationAttempts = useRef(
    new Map<string, Promise<M4SecretaryCoordinationActionRequestDto>>(),
  );
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
  // B3·秘书「待你拍板」数据源之一（主管两类意见·只读整店）。
  const [supervisorReviewStore, setSupervisorReviewStore] = useState<GlobalSupervisorReviewStoreV1 | null>(null);
  const [activeRightPanel, setActiveRightPanel] = useState<RightPanelKey | null>(null);
  const [focusedAgentThreadId, setFocusedAgentThreadId] = useState<string | null>(null);

  useEffect(() => {
    document.getElementById("tauri-boot-visible-probe")?.remove();

    if (viteEnv.DEV !== true) return;

    document.documentElement.dataset.appBoot = "mounted";
    void setTauriWindowTitle("Codex 治理工作台 · 首屏已挂载").then((available) => {
      if (!available) {
        document.documentElement.dataset.appTitleProbe = "unavailable";
      }
    });
  }, []);

  // R2: the desktop host owns the relay listener and emits only a validated
  // Markdown intent.  Browser preview never subscribes, and this handler
  // cannot select a route, command, root, or arbitrary file target.
  useEffect(() => {
    if (browserPreviewEnabled || !("__TAURI_INTERNALS__" in window)) return;
    let disposed = false;
    let stopListening: (() => void) | null = null;
    void listen<unknown>(KNOWLEDGE_OPEN_RELAY_EVENT_NAME, (event) => {
      const intent = parseKnowledgeOpenRelayIntent(event.payload);
      if (!intent) return;
      navigate("knowledge");
      setKnowledgeOpenIntent(intent);
    })
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          stopListening = stop;
        }
      })
      .catch(() => {
        // A listener failure leaves the host request without an acknowledgement;
        // it must time out fail-closed instead of being replaced by a local path.
      });
    return () => {
      disposed = true;
      stopListening?.();
    };
  }, [navigate]);

  const acknowledgeKnowledgeOpenIntent = useCallback(
    async (intent: KnowledgeOpenRelayIntent, outcome: KnowledgeOpenRelayOutcome): Promise<boolean> => {
      try {
        await acknowledgeKnowledgeOpenRelayIntentAtHost(intent, outcome);
        setKnowledgeOpenIntent((current) => (sameKnowledgeOpenRelayIntent(current, intent) ? null : current));
        return true;
      } catch {
        // Keep the matching intent in memory until the host acknowledges it.
        // This prevents a local failure from looking like a completed open.
        return false;
      }
    },
    [],
  );

  useEffect(() => {
    void reload();
  }, []);

  async function reload() {
    setNotice("正在读取索引。");
    setError(false);
    setSystemStatus(null);
    void reloadSecretaryHome();
    if (browserPreviewEnabled) {
      setSnapshot(browserPreviewSnapshot);
      setWorkflowState(browserPreviewWorkflowState);
      setPlanAuthorizationStore(browserPreviewPlanAuthorizationStore);
      setProjectConsultationProposalStore(browserPreviewProposalStore);
      setNotice("浏览器预览模式：使用示例会话数据；真实读取和发送请用 Tauri 桌面壳。");
      return;
    }
    try {
      const { snapshot: nextSnapshot } = await loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel);
      setSnapshot(nextSnapshot);
      setNotice("");
      void reloadSystemStatus();
      void reloadWorkflowState();
    } catch (loadError) {
      setSnapshot(null);
      setError(true);
      setNotice(`读取失败：${messageOf(loadError)}`);
    }
  }

  async function reloadSystemStatus() {
    if (browserPreviewEnabled) return;
    try {
      setSystemStatus(await loadSystemStatusReadModel());
    } catch {
      // 系统状态读模型是首页/顶栏的补充读面，读失败不能打断已有索引。
      setSystemStatus(null);
    }
  }

  async function reloadSecretaryHome() {
    setSecretaryHomeTransport("loading");
    setSecretaryHomeEnvelope(null);
    setSecretaryHomeErrorCode(null);
    if (browserPreviewEnabled) {
      setSecretaryHomeTransport("error");
      setSecretaryHomeErrorCode("M4_SECRETARY_HOME_REQUIRES_TAURI");
      return;
    }
    try {
      const envelope = await loadSecretaryHomeContext();
      setSecretaryHomeEnvelope(envelope);
      setSecretaryHomeTransport("loaded");
    } catch (loadError) {
      setSecretaryHomeTransport("error");
      setSecretaryHomeErrorCode(secretaryHomeSafeErrorCode(loadError));
    }
  }

  async function reloadWorkflowStateCore(refreshCandidateStores: boolean) {
    if (browserPreviewEnabled) {
      setWorkflowState(browserPreviewWorkflowState);
      setPlanAuthorizationStore(browserPreviewPlanAuthorizationStore);
      setProjectConsultationProposalStore(browserPreviewProposalStore);
      setWorkflowStateError(null);
      setWorkflowStateLoading(false);
      return;
    }
    setWorkflowStateLoading(true);
    setWorkflowStateError(null);
    try {
      const nextWorkflowState = await loadWorkflowStateSnapshot();
      setWorkflowState(nextWorkflowState);
      if (refreshCandidateStores) await reloadCandidateStores();
    } catch (loadError) {
      setWorkflowState(null);
      setWorkflowStateError(messageOf(loadError));
    } finally {
      setWorkflowStateLoading(false);
    }
  }

  async function reloadWorkflowState() {
    await reloadWorkflowStateCore(true);
  }

  // P3-A：过程消息只依赖 workflow snapshot 内的黑板派生；别借此轮询候选/方案/记忆店。
  async function reloadWorkflowStateReadOnly() {
    await reloadWorkflowStateCore(false);
  }

  async function reloadProposalAndWorkflowState() {
    await Promise.all([reloadCandidateStores(), reloadWorkflowStateCore(false)]);
  }

  // Resident 对话提交后的投影只重读它实际会改变的 canonical workflow 和 proposal
  // store。保留错误给调用方，让中栏能诚实区分“已送达、但还没刷新”。
  async function reloadJiaobanConversationProjection() {
    if (browserPreviewEnabled) {
      setWorkflowState(browserPreviewWorkflowState);
      setProjectConsultationProposalStore(browserPreviewProposalStore);
      return;
    }
    const [nextWorkflowState, nextProjectConsultationProposalStore] = await Promise.all([
      loadWorkflowStateSnapshot(),
      loadProjectConsultationProposalStore(),
    ]);
    setWorkflowState(nextWorkflowState);
    setProjectConsultationProposalStore(nextProjectConsultationProposalStore);
  }

  async function reloadCandidateStores() {
    if (browserPreviewEnabled) return;
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
        nextSupervisorReviewStore,
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
        loadGlobalSupervisorReviewStore(),
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
      setSupervisorReviewStore(nextSupervisorReviewStore);
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
      } else if (pendingAction.kind === "adopt-memory-candidates-to-formal-memory-batch") {
        if (!pendingAction.memoryCandidateBatchAdoptions?.length) {
          throw new Error("批量记忆候选采纳缺少待写入对象");
        }
        const results = [];
        for (const adoption of pendingAction.memoryCandidateBatchAdoptions) {
          results.push(await adoptMemoryCandidateToFormalMemory(adoption));
        }
        await reloadCandidateStores();
        setNotice(
          `批量采纳已逐条走 M2 门：${results.length} 条候选受控写入正式记忆；未自动采纳其他候选。`,
        );
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
      } else if (pendingAction.kind === "run-project-consultation") {
        if (!pendingAction.runProjectConsultation) {
          throw new Error("AI 出方案缺少目标请求对象");
        }
        // 真 codex 只读咨询（异步·可能耗时数分钟）→ 写一份待确认方案；不自动确认（人闸守住）。
        const proposal = await runProjectConsultation(pendingAction.runProjectConsultation);
        await reloadCandidateStores();
        setNotice(`AI 已出方案：${proposal.status}；请在方案授权卡里审阅、确认或要求修改，本轮未启动真实工作者。`);
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
        const { snapshot: nextSnapshot } = await loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel);
        setSnapshot(nextSnapshot);
        await reloadWorkflowState();
        setNotice(
          `项目自动编排 Level A 已记录：${result.plan.run_units.length} 个 run unit；状态 ${result.status}；未发送提示词、未执行真实 Codex。`,
        );
      } else if (pendingAction.kind === "record-k3-b1-manual-recovery-submission") {
        setNotice("K3-B1 手动回交路径已进入待主管线复核提示；L1 不执行真实 Codex、不发送提示词、不自动接受成功。");
      } else if (pendingAction.kind === "request-k3-b1-renewed-risk-approval") {
        setNotice("K3-B1 重新授权申请只进入待安全审查提示；L1 不继承旧授权、不启动 retry、不解锁 K3-B2。");
      } else if (pendingAction.kind === "record-operation-control-decision") {
        const operation = pendingAction.operationControlAction;
        if (!operation) {
          throw new Error("L3 操作控制缺少待记录对象");
        }
        const result = await recordOperationControlDecision(operation);
        let memoryCaptureNotice = "";
        if (pendingAction.memoryCaptureEvent) {
          const captureOutput = await captureMemoryEvent(pendingAction.memoryCaptureEvent);
          await reloadCandidateStores();
          memoryCaptureNotice = captureOutput.candidate
            ? ` 已生成待确认记忆候选：${captureOutput.candidate.candidate_key}；候选不是正式记忆。`
            : " 已记录记忆捕获来源；未写正式记忆。";
        }
        const { snapshot: nextSnapshot } = await loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel);
        setSnapshot(nextSnapshot);
        setWorkflowState(result.snapshot);
        setNotice(
          `${result.message} 审计 ${result.audit_event_id}；L3 不调用 runner、不停止/重启真实进程、不解锁 K3-B2。${memoryCaptureNotice}`,
        );
      } else if (pendingAction.kind === "knowledge-vault-ai-write") {
        // L3 知识库第一片：AI 提议写 vault——弹窗「允许写入」那一下走到这里才落盘（无常驻授权）。
        const write = pendingAction.knowledgeVaultWrite;
        if (!write) {
          throw new Error("知识库写入缺少笔记内容");
        }
        const result = await knowledgeVaultAiWrite(write);
        window.dispatchEvent(new Event("syn-knowledge-vault-saved"));
        setNotice(`已写入知识库笔记「${result.title}」（AI 提议、你允许才落盘；已审计 ${result.audit_event_id}）。`);
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
  const secretaryHome = useMemo<SecretaryHomeReadModel>(() => {
    if (secretaryHomeTransport === "loading") return deriveSecretaryHomeReadModel({ phase: "loading" });
    if (secretaryHomeEnvelope) return deriveSecretaryHomeReadModel({ home_context: secretaryHomeEnvelope });
    return deriveSecretaryHomeReadModel({ phase: "error", error_code: secretaryHomeErrorCode });
  }, [secretaryHomeEnvelope, secretaryHomeErrorCode, secretaryHomeTransport]);
  const secretaryHomePresentationState = secretaryHomeTransport === "loading"
    ? "loading"
    : secretaryHomeTransport === "error"
      ? "error"
      : null;

  const openSecretaryDeepLink = useCallback((descriptor: SecretaryTypedDeepLinkDescriptor) => {
    // Route by the sealed descriptor kind only. The renderer never reads a
    // path/URL or executes a payload; typed object fields only provide focus
    // inside the fixed source-owner view.
    if (descriptor.kind === "M4_SOURCE_ROUTE") {
      navigate("projects", {
        kind: descriptor.source_object_type,
        id: descriptor.source_object_ref,
      });
      setNotice("已转到来源负责模块；事实详情只在来源模块读取。");
      return;
    }
    setNotice("当前是降级摘要，尚未恢复可打开的来源负责模块。");
  }, [navigate]);

  const operateSecretaryAction = useCallback(async (intent: SecretaryCoordinationIntent) => {
    const revision = intent.item.coordination_revision;
    if (intent.item.source_authority !== "M4_COORDINATION" || revision === null) return;
    const attemptRef = `${intent.item.item_ref}|${revision}|${intent.action}`;
    let requestPromise = secretaryCoordinationAttempts.current.get(attemptRef);
    if (!requestPromise) {
      requestPromise = mintSecretaryCoordinationIdempotencyKey().then((idempotencyKey) => ({
        action: intent.action,
        item_ref: intent.item.item_ref,
        expected_revision: revision,
        idempotency_key: idempotencyKey,
        ...(intent.snoozed_until_utc ? { snoozed_until_utc: intent.snoozed_until_utc } : {}),
      }));
      secretaryCoordinationAttempts.current.set(attemptRef, requestPromise);
    }
    setSecretaryCoordinationStates((current) => ({
      ...current,
      [intent.item.item_ref]: { phase: "pending", action: intent.action },
    }));
    let requestReady = false;
    try {
      const request = await requestPromise;
      requestReady = true;
      const receipt = await operateSecretaryCoordination(request);
      secretaryCoordinationAttempts.current.delete(attemptRef);
      setSecretaryCoordinationStates((current) => ({
        ...current,
        [intent.item.item_ref]: {
          phase: "succeeded",
          action: intent.action,
          command_receipt_ref: receipt.command_receipt_ref,
          outcome_code: receipt.outcome_code,
        },
      }));
      await reloadSecretaryHome();
    } catch (operationError) {
      if (!requestReady) secretaryCoordinationAttempts.current.delete(attemptRef);
      setSecretaryCoordinationStates((current) => ({
        ...current,
        [intent.item.item_ref]: {
          phase: "failed",
          action: intent.action,
          error_code: secretaryHomeSafeErrorCode(operationError),
        },
      }));
    }
  }, []);

  const secretaryContext = useMemo(
    () =>
      deriveSecretaryContext({
        snapshot: displaySnapshot,
        workflowState,
        blackboardCandidateStore,
        memoryCaptureStore,
        memoryCandidateStore,
        workflowStateError,
        proposalStore: projectConsultationProposalStore,
        supervisorReviewStore,
      }),
    [displaySnapshot, workflowState, blackboardCandidateStore, memoryCaptureStore, memoryCandidateStore, workflowStateError, projectConsultationProposalStore, supervisorReviewStore],
  );
  const pendingReviewCount =
    workflowState?.project_workflows.reduce(
      (count, workflow) => count + workflow.task_drafts.filter((task) => task.state === "ready_for_review").length,
      0,
    ) ?? 0;
  const topbarReviewCount = pendingReviewCount + (workflowState?.counts.reviews ?? 0);
  const isDeveloperView = devNavItems.some((item) => item.key === activeView);

  return (
    <WorkbenchShell
      activeRightPanel={activeRightPanel}
      activeView={activeView}
      actionBusy={actionBusy}
      displaySnapshot={displaySnapshot}
      error={error}
      isDeveloperView={isDeveloperView}
      memoryCandidateStore={memoryCandidateStore}
      memoryCaptureStore={memoryCaptureStore}
      notice={notice}
      pendingAction={pendingAction}
      query={query}
      rightStats={rightStats}
      secretaryContext={secretaryContext}
      secretaryHome={secretaryHome}
      secretaryHomePresentationState={secretaryHomePresentationState}
      systemStatus={systemStatus}
      topbarReviewCount={topbarReviewCount}
      workflowState={workflowState}
      workflowStateError={workflowStateError}
      workflowStateLoading={workflowStateLoading}
      onActiveRightPanelChange={setActiveRightPanel}
      onActiveViewChange={navigate}
      onCancelAction={() => setPendingAction(null)}
      onConfirmAction={confirmAction}
      onQueryChange={setQuery}
      onReload={reload}
      onReloadSecretaryHome={reloadSecretaryHome}
      onReloadWorkflowState={reloadWorkflowState}
      onOpenSecretaryDeepLink={openSecretaryDeepLink}
    >
        {activeView === "home" ? (
          <HomeView
            secretaryHome={secretaryHome}
            presentationState={secretaryHomePresentationState}
            coordinationStates={secretaryCoordinationStates}
            onOperateCoordination={(intent) => void operateSecretaryAction(intent)}
            onOpenDeepLink={openSecretaryDeepLink}
            onReloadSecretaryHome={() => void reloadSecretaryHome()}
          />
        ) : activeView === "secretary_board" ? (
          <SecretaryBoardView
            home={secretaryHome}
            presentationState={secretaryHomePresentationState}
            onOpenDeepLink={openSecretaryDeepLink}
            onReloadSecretaryHome={() => void reloadSecretaryHome()}
          />
        ) : renderActiveWorkbenchView({
          view: activeView,
          snapshot: displaySnapshot,
          systemStatus,
          onRequestAction: setPendingAction,
          onNavigate: navigate,
          knowledgeOpenIntent,
          onKnowledgeOpenIntentOutcome: acknowledgeKnowledgeOpenIntent,
          navigationFocus,
          secretaryContext,
          workflowState,
          workflowStateLoading,
          workflowStateError,
          onReloadWorkflowState: reloadWorkflowState,
          onWorkflowStateReadRefresh: reloadWorkflowStateReadOnly,
          onNotice: setNotice,
          onProposalStoreRefresh: reloadJiaobanConversationProjection,
          hasRealSnapshot: Boolean(filteredSnapshot),
          onOpenAgentSession: (threadId) => {
            setFocusedAgentThreadId(threadId);
            // 走 navigate 而不是裸 setActiveView：顺带清掉上一页的焦点，免得旧 focus 粘过来。
            navigate("agents");
          },
          browserPreviewData: browserPreviewEnabled
            ? {
                loadSessionPage: (request) => Promise.resolve(browserPreviewSessionPage(request)),
                loadTranscript: (threadId) => Promise.resolve(browserPreviewTranscript(threadId)),
                loadTranscriptPage: (request) => Promise.resolve(browserPreviewTranscript(request.thread_id)),
              }
            : undefined,
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
          // 浏览器预览（无 Tauri）下，这些预览回调底层会同步抛 ensureTauriRuntime 错误；
          // 画布视图在挂载时会自动触发任务包 / 拆任务预览，会把首屏炸到错误边界。
          // 预览模式改成返回 rejected promise，调用方 .catch 会把它当成「预览不可用」的内联提示，不崩。
          onPreviewTaskMemoryPacket: browserPreviewEnabled
            ? () => Promise.reject(new Error("浏览器预览模式：任务包记忆预览需用 Tauri 桌面壳。"))
            : previewTaskMemoryPacket,
          onPreviewProjectDirectorTaskPlan: browserPreviewEnabled
            ? () => Promise.reject(new Error("浏览器预览模式：拆任务预览需用 Tauri 桌面壳。"))
            : previewProjectDirectorTaskPlan,
          onPreviewFormalMemoryLifecycle: browserPreviewEnabled
            ? () => Promise.reject(new Error("浏览器预览模式：记忆生命周期预览需用 Tauri 桌面壳。"))
            : previewFormalMemoryLifecycleOperation,
          onPreviewMemoryEntityRelationCandidates: browserPreviewEnabled
            ? () => Promise.reject(new Error("浏览器预览模式：记忆实体关系预览需用 Tauri 桌面壳。"))
            : previewMemoryEntityRelationCandidates,
        })}
    </WorkbenchShell>
  );
}

// 人话工程①②(2026-07-20):notice 错误串接 src/lib/humanize.ts 薄委托——命中已知族出人话,
// 未命中原文逐字回退(显示串零变化)。覆盖 :175/:206/:281 及同族第四调用点 :630。
function messageOf(error: unknown): string {
  if (error instanceof Error) return humanizeNoticeMessage(error.message);
  return humanizeNoticeMessage(String(error));
}

function legacyProductCommandBlockedNotice(commandName: string) {
  return `${commandName} 是旧真实执行入口，已在 K2.5 封存；请改走统一 Product Command，不再从普通 UI 调用 legacy Tauri wrapper。`;
}

function secretaryHomeSafeErrorCode(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return /^[A-Za-z0-9_:-]{1,128}$/.test(message) ? message : "M4_SECRETARY_HOME_READ_FAILED";
}
