import { useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Pill } from "../../components/SpecPrimitives";
import {
  JiaobanBlockedState,
  JiaobanNeedsReworkDisposal,
  JiaobanWaitingDecisionState,
} from "./jiaoban/JiaobanBlockedStates";
import {
  JiaobanRunningState,
  JiaobanSupervisorPilotRunningState,
  isDirectorPlanningPhase,
} from "./jiaoban/JiaobanRunningStates";
import {
  JiaobanDoneState,
  type FactMemoryContext,
} from "./jiaoban/JiaobanDoneStates";
import {
  JiaobanAuthorizeState,
  JiaobanGovernanceView,
  JiaobanHowRunView,
} from "./jiaoban/JiaobanAuthorizeStates";
import {
  JiaobanPlanPreviewCanvas,
  jiaobanRuntimeNodeStates,
  previewCanvasNodesFor,
  previewNodeBinding,
  runCanvasBindingsFor,
  type JiaobanPreviewCanvasNode,
} from "./jiaoban/jiaobanPreviewCanvas";
import {
  JiaobanProposalIndex,
  JiaobanHistoryDetail,
  type HistoryFilter,
} from "./jiaoban/JiaobanHistory";
import {
  JiaobanConversationComposer,
  JiaobanConversationStream,
  artifactNoticesForConversation,
  mergeConversationUserTurns,
  supervisorProcessCanvasView,
  supervisorProcessFocusedNodeId,
  supervisorConversationEntriesForProject,
  userTurnsFromProposalHistory,
  type JiaobanConversationPhaseKind,
} from "./jiaoban/JiaobanConversation";
import { buildJiaobanArtifactCanvasViews, type JiaobanCanvasViewSpec, type JiaobanPhase } from "./jiaoban/JiaobanArtifactViews";
import { useConversationAutoScroll, useJiaobanConversationState } from "./jiaoban/useJiaobanConversationState";
import { useJiaobanRunningReadRefresh } from "./jiaoban/useJiaobanRunningReadRefresh";
import {
  JiaobanRawSessionLink,
  JiaobanSessionPicker,
  NEW_SESSION_CHOICE,
} from "./jiaoban/jiaobanSessionParts";
import { formatProposalTime, proposalAgeDays } from "./jiaoban/jiaobanTime";
import {
  humanizeAuthorizeError,
  humanizePreviewError,
  humanizeProviderUnavailable,
  isAlreadyConfirmedRejection,
} from "../../lib/humanize";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  applyProjectDirectorFailedAction,
  autoAdvanceAuthorizedRoleLoop,
  confirmAndStartAuthorizedRun,
  getProjectWorkflowChainStatus,
  launchSupervisorPilot,
  listProjectRunHistory,
  loadFormalMemoryStore,
  loadSupervisorPilotReadModel,
  previewPendingProposalDirectorPlan,
  recordGlobalBoundaryReview,
  recordProjectConsultationProposalDecision,
  runGlobalSupervisorBoundaryReview,
  runGlobalSupervisorReview,
  stopProjectWorkflowChain,
} from "../../lib/tauri";
import type {
  AutoAdvanceRoleLoopOutcome,
  CreateMemoryCandidateInput,
  DirectorChainOutcome,
  DirectorChainStep,
  GlobalSupervisorBoundaryReviewOutcome,
  GlobalSupervisorReviewOutcome,
  PendingAction,
  PlanAuthorizationStoreV1,
  ProjectConsultationProposal,
  ProjectDirectorPlannedTask,
  ProjectDirectorPreviewNodeSessionBinding,
  ProjectConsultationProposalStoreV1,
  ProjectRecord,
  ProjectWorkflowChainStatus,
  RunHistoryEntry,
  RunHistoryList,
  SessionRecord,
  SupervisorPilotReadModel,
  WorkflowStateSnapshot,
} from "../../lib/types";

// 固定测试项目（自动干只在这真跑；非它则老实标注·跳智能体直连）。与 WorkflowCommandConsoleView 同一常量。
const TEST_PROJECT_ROOT = "/Users/yoyi/codex-workflow-mario-test";
// 站 3b（2026-07-12 拍板）：唯一获批的真实项目只读入口——只开主管编排、零写根死线；
// 经典状态机对它仍锁（后端 S1 闸原样）。与后端 STATION_3B_READONLY_PROJECT_ROOT 同值。
export const STATION_3B_READONLY_PROJECT_ROOT = "/Users/yoyi/Documents/mario test";
const SUPERVISOR_PILOT_REASONING_EFFORT = "medium";

export type JiaobanOrchestrationMode = "classic" | "supervisor_pilot";

export function supervisorPilotUnavailableReason(projectRoot: string, allowedWriteRoots: string[]): string | null {
  if (projectRoot === STATION_3B_READONLY_PROJECT_ROOT) {
    // 3b 只读单照旧；站 4（2026-07-14）：唯一同根写根的写单放行，其余写根形状一律拒
    // ——与后端 station4_write_project_unsealed 同判（根精确 ∧ 恰一条写根 ∧ 写根精确同根）。
    if (allowedWriteRoots.length === 0) return null;
    if (allowedWriteRoots.length === 1 && allowedWriteRoots[0] === STATION_3B_READONLY_PROJECT_ROOT) {
      return null;
    }
    return "该项目写单仅允许唯一同根写根（站 4）。";
  }
  if (projectRoot !== TEST_PROJECT_ROOT) return "主管编排试点仅限固定测试项目。";
  if (allowedWriteRoots.some((root) => root !== TEST_PROJECT_ROOT)) {
    return "主管编排写入试点只允许固定测试项目根。";
  }
  return null;
}

// 站 3b 项目上经典状态机不可用的人话理由（测试项目与其它项目返回 null——其它项目根本进不了交办）。
export function classicModeUnavailableReason(projectRoot: string): string | null {
  if (projectRoot === STATION_3B_READONLY_PROJECT_ROOT) {
    return "站 3b 项目只开通主管编排只读单；经典状态机仍锁固定测试项目。";
  }
  return null;
}

export type ProjectJiaobanPanelProps = {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  projectConsultationProposalStore: ProjectConsultationProposalStoreV1 | null;
  planAuthorizationStore: PlanAuthorizationStoreV1 | null;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
  // fix8：出方案成功后刷新方案店（App 的 reloadCandidateStores 穿下来）→ latestProposal 更新 → 自动进批脸。
  // 可选：mock/gallery callsite 可不传（刷新走 noop，不崩）。
  onProposalStoreRefresh?: () => Promise<void>;
  onWorkflowStateReadRefresh?: () => Promise<void>; // P3-A：只读黑板派生刷新，不连带方案/候选店。
  // M2：外壳提供的「完整工作流」跳转；交办内的五态与命令不碰。
  onOpenWorkflow?: () => void;
  // M2：只把已有历史/五态内容交给 Shell 排版，面板自身仍拥有数据、状态与命令。
  renderLayout?: (content: ProjectJiaobanPanelLayout) => ReactNode;
};

export type { JiaobanCanvasViewSpec, JiaobanPhase } from "./jiaoban/JiaobanArtifactViews";

export function jiaobanStageFromChainOutcome(chain: DirectorChainOutcome | null): string {
  const reason = chain?.stopped_reason?.trim() ?? "";
  if (!reason) return "completed";
  if (reason.startsWith("waiting_decision:")) return "waiting_decision";
  if (reason.startsWith("fail_stop:")) return "failed";
  return "interrupted";
}

export function jiaobanPhaseForOutcome(outcome: AutoAdvanceRoleLoopOutcome): JiaobanPhase {
  // P1-D 人闸收敛:绑定停点已摘除——批准后默认自动新会话直进 prepare,合流命令再也不会
  // 回 task_session_binding_required=true,「binding」相位在此不再有出口(相位机本身零改)。
  if (outcome.stage === "completed") return "done";
  if (outcome.stage === "waiting_decision") return "waiting_decision";
  return "blocked";
}

export type ProjectJiaobanPanelLayout = {
  phase: JiaobanPhase;
  main: ReactNode;
  proposalIndex: ReactNode;
  // M1：批前、运行和终态都在 M2 的右侧画布区域展示同一张纵向工序图。
  previewCanvas?: ReactNode;
  // 右区=信息展开面:多视图时右区顶部出切换 chips(工序图/治理保证/怎么跑…);单视图/缺席=旧行为。
  canvasViews?: JiaobanCanvasViewSpec[];
  activeCanvasView?: string;
  onCanvasViewChange?: (key: string) => void;
};

// 交办面 = 项目默认页。同一容器随状态换脸（说 → 批 → 干 → 交货 / 卡住），不弹窗、永不冻。
// 离线/SSR：本组件含 hooks + 真命令，findButtonByText 会当普通函数平铺调用触发 "Invalid hook call"，
// 故 typeof window 守卫放最前，无 window 直接渲染静态占位（同 DirectorChainRunButton 套路）。
export function ProjectJiaobanPanel(props: ProjectJiaobanPanelProps) {
  if (typeof window === "undefined") {
    return (
      <section className="project-jiaoban" aria-label="交办">
        <p className="muted small-note">交办面需在桌面壳中打开。</p>
      </section>
    );
  }
  return <ProjectJiaobanPanelBrowser {...props} />;
}

// 交办进度（人话化用）。stage 与后端 outcome/链状态解耦——这里只管「说给用户听」。

// 方案a fix：「开个新的」的显式哨兵值。此前用 null 一词两用（"还没定" 与 "用户选了新建"），
// 重挂载/默认效果会把用户的显式选择无声改回「接现有」（真机踩到：明确选了新建却路由到旧对话）。
// export 供离线 DOM 断言测试对齐哨兵值（纯暴露·不改会话选择语义）。

// 阶段3拆巨石:预演画布类型与工具迁 jiaoban/jiaobanPreviewCanvas.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanPlanPreviewCanvas,
  jiaobanRuntimeNodeStates,
} from "./jiaoban/jiaobanPreviewCanvas";
export type {
  JiaobanPreviewCanvasNode,
  JiaobanRuntimeNodeState,
  JiaobanRuntimeNodeStateInfo,
} from "./jiaoban/jiaobanPreviewCanvas";

// 结果防丢：换 tab 会卸载本面板（ProjectWorkspaceShell 条件渲染），本地 state 全丢。
// 故把「一轮开始的结果」按 project_root 缓存在模块级，重挂载时恢复——切走再回来结果还在。
// 只缓存呈现所需的最小集：手动相位 + outcome + 报错 + 上次停因（供重出方案预填）+ 这一轮批的方案 id。
type JiaobanRunCache = {
  manualPhase: JiaobanPhase | null;
  outcome: AutoAdvanceRoleLoopOutcome | null;
  startError: string | null;
  lastStopReason: string | null; // 卡住原因，重新出方案时带回「说」面
  ranProposalId: string | null; // 这一轮真按下[允许并开始]批的方案 id（区分「新方案到达」用）
  // 合流命令对「已确认/旧方案」拒绝时置 true：这条路授权本还活着，卡住脸要给[接着跑,不用重批]而非只给重出方案。
  // 换 tab 回来也要记得给这个口，故进缓存。
  continueHint: boolean | null;
  // fix6-v2：这一轮点击[允许并开始]/[接着跑]的时刻（ms）。判「这轮的链」用（chainStatus.started_at >= 它）；重挂载恢复。
  runStartedAtMs: number | null;
  // 方案a：本轮是不是「开个新的」（new）。运行脸 new 时补「正在新建会话（约 1 分钟）」提示；缓存恢复。
  runIsNewSession: boolean;
  // 方案a fix：会话选择跨重挂载保留（NEW_SESSION_CHOICE / thread_id / null=未定）。
  sessionChoice: string | null;
  // M1：批前预演节点→会话选择。与任务绑定不同，它允许简单活使用虚拟步骤 id。
  previewSessionBindings: ProjectDirectorPreviewNodeSessionBinding[];
  // 运行/终态继续使用本轮已批的图，不能随方案 store 刷新退回旧 ReactFlow 视图。
  runCanvasNodes: JiaobanPreviewCanvasNode[];
  runCanvasBindings: ProjectDirectorPreviewNodeSessionBinding[];
  // B1：本轮全局主管复核结果（key=chain_started_at·结果态缓存；loading 是瞬态不缓存——
  // 重挂载后按幂等键补拉，后端幂等命中秒回不重烧）。
  supervisorReview: { key: string; outcome: GlobalSupervisorReviewOutcome } | null;
  // Station 2：主管试点只保留 run-id；事件流始终重新从 sidecar 只读投影加载。
  supervisorPilotRunId: string | null;
};
const jiaobanRunCacheByProject = new Map<string, JiaobanRunCache>();

function readJiaobanRunCache(projectRoot: string): JiaobanRunCache | null {
  return jiaobanRunCacheByProject.get(projectRoot) ?? null;
}
function writeJiaobanRunCache(projectRoot: string, patch: Partial<JiaobanRunCache>) {
  const prev = jiaobanRunCacheByProject.get(projectRoot) ?? {
    manualPhase: null,
    outcome: null,
    startError: null,
    lastStopReason: null,
    ranProposalId: null,
    continueHint: null,
    runStartedAtMs: null,
    runIsNewSession: false,
    sessionChoice: null,
    previewSessionBindings: [],
    runCanvasNodes: [],
    runCanvasBindings: [],
    supervisorReview: null,
    supervisorPilotRunId: null,
  };
  jiaobanRunCacheByProject.set(projectRoot, { ...prev, ...patch });
}
function clearJiaobanRunCache(projectRoot: string) {
  jiaobanRunCacheByProject.delete(projectRoot);
}

// 刀2「批前看图」预拆结果缓存 by proposal_id：预拆真 LM 1-7 分钟，切 tab/重挂载回来不重拆。只缓存成功结果
// （失败/进行中是瞬态 state，不进缓存——重来时按需重拆）。同一份方案的图批时原样回传给后端，所见即所跑。
type JiaobanPreviewCache = {
  tasks: ProjectDirectorPlannedTask[];
  warnings: string[];
};
const jiaobanPreviewCacheByProposal = new Map<string, JiaobanPreviewCache>();

// 人话工程①(2026-07-20):humanizePreviewError 逐字迁 src/lib/humanize.ts,顶部 import-back。

// 阶段3拆巨石:时间工具迁 jiaoban/jiaobanTime.ts(原样零逻辑改动)。

// B2·批前边界意见缓存 by proposal_id（照 worksmap 先例·重挂载先读缓存不重烧）。后端已按 proposal_id 幂等；
// 前端缓存让切 tab/重挂载回来 0 往返。ready 与 unavailable 都缓存（unavailable 走 [重试] force 才重跑）。
const jiaobanBoundaryReviewCacheByProposal = new Map<string, GlobalSupervisorBoundaryReviewOutcome>();

// B2 触发判据（纯函数·export 供离线断言）：只对「今天生成的 pending 方案」触发——stale 不触发=省额度；
// 纯建议方案（写根空）照常触发（它点破 mismatch 与 fix9 警条互证）。
export function shouldRequestBoundaryReview(
  proposal: { status: string; created_at_ms: number } | null | undefined,
): boolean {
  if (!proposal) return false;
  if (!["draft", "pending_user_confirmation"].includes(proposal.status)) return false;
  return proposalAgeDays(proposal.created_at_ms) < 1; // 今天生成的才触发
}

function ProjectJiaobanPanelBrowser({
  project,
  sessions,
  workflowState,
  projectConsultationProposalStore,
  planAuthorizationStore,
  onRequestAction,
  onOpenAgentSession,
  onProposalStoreRefresh,
  onWorkflowStateReadRefresh,
  onOpenWorkflow,
  renderLayout,
}: ProjectJiaobanPanelProps) {
  const projectWorkflow =
    workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;

  // 本项目最新一份方案（授权卡的数据源）。主管私有 submit_proposal 落卡后，刷新读模型即切到「批」。
  const proposalSummary = useMemo(
    () =>
      summarizeProjectConsultationProposalStore(
        projectConsultationProposalStore,
        planAuthorizationStore,
        projectWorkflow?.project_id,
        projectWorkflow?.workflow_id,
      ),
    [projectConsultationProposalStore, planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const latestProposal = proposalSummary.latest_proposal;
  const workflowProposals = useMemo(
    () =>
      [...(projectConsultationProposalStore?.proposals ?? [])]
        .filter(
          (proposal) =>
            proposal.project_id === projectWorkflow?.project_id &&
            proposal.workflow_id === projectWorkflow?.workflow_id,
        )
        .sort(
          (left, right) =>
            left.created_at_ms - right.created_at_ms || left.proposal_id.localeCompare(right.proposal_id),
        ),
    [projectConsultationProposalStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const knownProposalIds = useMemo(
    () =>
      projectConsultationProposalStore == null
        ? null
        : new Set(workflowProposals.map((proposal) => proposal.proposal_id)),
    [projectConsultationProposalStore, workflowProposals],
  );

  const isTestProject = project.project_root === TEST_PROJECT_ROOT;
  // 站 3b：唯一获批的真实项目只读入口——进交办流，但只开主管编排（经典模式后端 S1 闸原样拒）。
  const isStation3bProject = project.project_root === STATION_3B_READONLY_PROJECT_ROOT;

  // 「用哪个对话干」：本项目可读会话（现成会话列表数据源）。
  const projectSessions = useMemo(
    () => sessions.filter((session) => session.rollout_exists && !session.archived),
    [sessions],
  );

  const projectRoot = project.project_root;

  // 手动相位覆盖：允许并开始/干完/卡住由本地状态驱动（proposal store 变化只决定「说 ↔ 批」）。
  // 挂载时从模块缓存恢复（换 tab 回来结果不丢）——首帧就带上上一轮的脸/结果。
  const cached = readJiaobanRunCache(projectRoot);
  const [manualPhase, setManualPhase] = useState<JiaobanPhase | null>(cached?.manualPhase ?? null);
  // 卡住态乙型「直接回它一句」的草稿：状态提升到本组件——JiaobanBlockedState 被离线测试平铺裸调，不能有 hooks。
  const [blockedReply, setBlockedReply] = useState("");
  // 刀B·记忆召回计数：说脸预告「出方案会带上 N 条项目记忆」（同后端召回口径：本项目 project_id + 活跃态）。
  // 只读 formal store·失败静默 0（召回是增益不是闸）。
  const [memoryCount, setMemoryCount] = useState(0);
  useEffect(() => {
    let alive = true;
    void loadFormalMemoryStore()
      .then((store) => {
        if (!alive) return;
        setMemoryCount(
          store.records.filter(
            (record) =>
              record.status === "memory_active" &&
              record.scope.project_id === projectWorkflow?.project_id,
          ).length,
        );
      })
      .catch(() => {
        if (alive) setMemoryCount(0);
      });
    return () => {
      alive = false;
    };
  }, [projectWorkflow?.project_id]);
  // 「说」面顶部的上次停因摘要（重新出方案时带过来；空则不显示）。
  const [sayHint, setSayHint] = useState<string | null>(null);
  // 「用哪个对话干」：默认选最近一条现有会话（下面 effect 里补默认；null 只在真无会话时保留）。
  const [sessionChoice, setSessionChoiceState] = useState<string | null>(cached?.sessionChoice ?? null);
  const [previewSessionBindings, setPreviewSessionBindings] = useState<ProjectDirectorPreviewNodeSessionBinding[]>(
    cached?.previewSessionBindings ?? [],
  );
  const [runCanvasNodes, setRunCanvasNodes] = useState<JiaobanPreviewCanvasNode[]>(cached?.runCanvasNodes ?? []);
  const [runCanvasBindings, setRunCanvasBindings] = useState<ProjectDirectorPreviewNodeSessionBinding[]>(
    cached?.runCanvasBindings ?? [],
  );
  // 方案a fix：用户任何显式选择（新建/某条现有）都进缓存 → 重挂载不丢、默认效果不覆盖。
  function setSessionChoice(value: string | null) {
    setSessionChoiceState(value);
    sessionDefaultedRef.current = true;
    writeJiaobanRunCache(projectRoot, { sessionChoice: value });
  }
  const [starting, setStarting] = useState(false);
  const [outcome, setOutcome] = useState<AutoAdvanceRoleLoopOutcome | null>(cached?.outcome ?? null);
  const [startError, setStartError] = useState<string | null>(cached?.startError ?? null);
  const [needsReworkActionError, setNeedsReworkActionError] = useState<string | null>(null);
  // 合流命令拒「方案不是待用户确认」时置 true → 卡住脸给[接着跑,不用重批]。
  const [continueHint, setContinueHint] = useState<boolean>(cached?.continueHint ?? false);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);
  // 站 2：模式按单重置为经典；主管试点的过程只从 sidecar 审计读模型取，不投射为链态。
  const [orchestrationMode, setOrchestrationMode] = useState<JiaobanOrchestrationMode>(() =>
    project.project_root === STATION_3B_READONLY_PROJECT_ROOT ? "supervisor_pilot" : "classic",
  );
  const [supervisorPilotRunId, setSupervisorPilotRunId] = useState<string | null>(
    cached?.supervisorPilotRunId ?? null,
  );
  const [supervisorPilotReadModel, setSupervisorPilotReadModel] = useState<SupervisorPilotReadModel | null>(null);
  const [supervisorPilotLedgerError, setSupervisorPilotLedgerError] = useState<string | null>(null);
  const runningRef = useRef(false);
  const {
    messageBusy,
    messageErrors,
    makeConversationComposer,
    setComposerDraft: setConversationComposerDraft,
  } = useJiaobanConversationState({
    projectWorkflow,
    workflowState,
    projectRoot,
    onProposalStoreRefresh,
  });
  const residentMessageBusyKey = messageBusy ? "resident-message" : null;
  const consultLoading = messageBusy;
  const consultError = messageErrors["resident-message"] ?? null;

  const refreshWorkflowStateReadModelAfterSuccessfulChainAction = useJiaobanRunningReadRefresh({
    manualPhase,
    projectRoot: projectWorkflow?.project_root ?? null,
    workflowId: projectWorkflow?.workflow_id ?? null,
    onChainStatus: setChainStatus,
    onWorkflowStateReadRefresh,
  });

  // Part①·历届方案索引（倒序读模型·不轮询·挂载/交货/重拆各拉一次）。失败静默：索引是增益不是闸。
  const [history, setHistory] = useState<RunHistoryEntry[]>([]);
  const [historyTotal, setHistoryTotal] = useState(0);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyFilter, setHistoryFilter] = useState<HistoryFilter>("all");
  const [selectedHistoryId, setSelectedHistoryId] = useState<string | null>(null);
  const historyLoadingRef = useRef(false);
  // 这一轮真按下[允许并开始]批的方案 id（从缓存恢复）。用来判「有没有来一份新方案」。
  const ranProposalIdRef = useRef<string | null>(cached?.ranProposalId ?? null);
  // fix6-v2：本轮点击时刻（ms，缓存恢复）。判「这轮的链」：chainStatus.started_at >= 它才算本轮（旧链更早、天然排除）。
  const [runStartedAtMs, setRunStartedAtMs] = useState<number | null>(cached?.runStartedAtMs ?? null);
  // 方案a：本轮是不是「开个新的」。运行脸 new 时补「正在新建会话」提示；缓存恢复。
  const [runIsNewSession, setRunIsNewSession] = useState<boolean>(cached?.runIsNewSession ?? false);

  // 刀2「批前看图」：复杂活（proposal.suggest_workflow）或用户开「按工作流来」→ 批前预拆并在右侧画布预演工序图。
  // 结果按 proposal_id 缓存（切 tab 回来不重拆·下面 effect 命中即恢复）；loading/error 是瞬态。
  // 批时 previewTasks 原样回传后端 = 所见即所跑；简单活/预拆失败则 previewTasks 为空 → 不传 → 后端照旧拆。
  const [workflowSwitchOn, setWorkflowSwitchOn] = useState(latestProposal?.suggest_workflow === true);
  // 右区信息展开面当前视图：首帧也按缓存相位落到本态定稿物，随后由呈现 effect 随相位同步。
  const [canvasViewKey, setCanvasViewKey] = useState(() => {
    if (cached?.manualPhase === "done") return "delivery";
    if (cached?.manualPhase && cached.manualPhase !== "say") return "graph";
    return latestProposal && cached?.manualPhase !== "say" ? "proposal" : "graph";
  });
  const [focusedRuntimeNodeId, setFocusedRuntimeNodeId] = useState<string | null>(null);
  // 批卡「怎么跑」入口一行的摘要(真值随右区视图里的控件走)。
  const howRunSummary = `${orchestrationMode === "supervisor_pilot" ? "主管编排(试点)" : "经典状态机"} · ${
    sessionChoice && sessionChoice !== NEW_SESSION_CHOICE ? "接现有对话" : "开个新对话"
  } · 预演图${workflowSwitchOn ? "开" : "关"}`;
  const [previewTasks, setPreviewTasks] = useState<ProjectDirectorPlannedTask[] | null>(null);
  const [previewWarnings, setPreviewWarnings] = useState<string[]>([]);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [previewNonce, setPreviewNonce] = useState(0); // 重试 bump：effect 依赖没变时强制重跑预拆
  const previewLoadingRef = useRef(false);
  const previewCanvasNodes = useMemo(
    () => previewCanvasNodesFor(latestProposal, previewTasks),
    [latestProposal, previewTasks],
  );
  const previewBindingsForCanvas = useMemo(
    () =>
      previewCanvasNodes.map((node, index) => {
        const saved = previewSessionBindings.find(
          (binding) => binding.preview_node_id === node.preview_node_id,
        );
        return saved ?? previewNodeBinding(node.preview_node_id, index === 0 ? sessionChoice : NEW_SESSION_CHOICE);
      }),
    [previewCanvasNodes, previewSessionBindings, sessionChoice],
  );

  function rememberRunCanvas(
    nodes = previewCanvasNodes,
    // P1-D 后逐任务绑定面板退场,任务侧映射恒空——只按 M1 预演选择装配。
    bindings = runCanvasBindingsFor(nodes, previewBindingsForCanvas, []),
  ) {
    setRunCanvasNodes(nodes);
    setRunCanvasBindings(bindings);
    writeJiaobanRunCache(projectRoot, { runCanvasNodes: nodes, runCanvasBindings: bindings });
  }

  function updatePreviewSessionBinding(previewNodeId: string, value: string | null) {
    const nextBinding = previewNodeBinding(previewNodeId, value);
    setPreviewSessionBindings((current) => {
      const existing = current.findIndex((binding) => binding.preview_node_id === previewNodeId);
      const next =
        existing >= 0
          ? current.map((binding, index) => (index === existing ? nextBinding : binding))
          : [...current, nextBinding];
      writeJiaobanRunCache(projectRoot, { previewSessionBindings: next });
      return next;
    });
    if (previewCanvasNodes[0]?.preview_node_id === previewNodeId) {
      setSessionChoice(value);
    }
  }

  // 顶层选择仍保留，但它只是第一个预演节点的预填；改顶层时同步节点，绝不扩散到其余节点。
  useEffect(() => {
    const firstNode = previewCanvasNodes[0];
    if (!firstNode) return;
    const nextBinding = previewNodeBinding(firstNode.preview_node_id, sessionChoice);
    setPreviewSessionBindings((current) => {
      const index = current.findIndex((binding) => binding.preview_node_id === firstNode.preview_node_id);
      if (
        index >= 0 &&
        current[index]?.session_choice === nextBinding.session_choice &&
        current[index]?.session_id === nextBinding.session_id
      ) {
        return current;
      }
      const next =
        index >= 0
          ? current.map((binding, bindingIndex) => (bindingIndex === index ? nextBinding : binding))
          : [...current, nextBinding];
      writeJiaobanRunCache(projectRoot, { previewSessionBindings: next });
      return next;
    });
  }, [previewCanvasNodes, sessionChoice, projectRoot]);

  // 状态防丢的关键判断：**只在「出现一份新的、待用户确认的方案」时**清上一轮开始态、回到「批」。
  // 反过来，下面几种一律不清（旧实现漏了这些，导致结果被抹）：
  //   · 正在跑（runningRef.current === true）——await 期间 store 刷新不能把「正在干」的脸清掉；
  //   · latestProposal 变 null（store 刷新/方案被 supersede）——结果得留着，不是「新方案到了」；
  //   · 方案 id 没变，或变成的是我们这一轮已经批过的那份——不是新方案，别清。
  // 「新方案到了回批面」= 有 latestProposal + 状态是待确认/草案 + id 既不同于上次已批、也不同于上次见过的。
  const seenProposalIdRef = useRef<string | null>(cached?.ranProposalId ?? null);
  useEffect(() => {
    const proposalId = latestProposal?.proposal_id ?? null;
    if (!proposalId || !latestProposal) return; // 变 null 不清
    if (runningRef.current) return; // 正在跑不清
    const isFreshPending =
      ["draft", "pending_user_confirmation"].includes(latestProposal.status) &&
      proposalId !== ranProposalIdRef.current &&
      proposalId !== seenProposalIdRef.current;
    seenProposalIdRef.current = proposalId;
    if (!isFreshPending) return;
    // 确是一份新的待批方案 → 回批面重审，清掉上一轮。
    setManualPhase(null);
    setOutcome(null);
    setStartError(null);
    setContinueHint(false);
    setChainStatus(null);
    setSupervisorPilotRunId(null);
    setSupervisorPilotReadModel(null);
    setSupervisorPilotLedgerError(null);
    ranProposalIdRef.current = null;
    clearJiaobanRunCache(projectRoot);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [latestProposal?.proposal_id, latestProposal?.status]);

  const supervisorPilotDisabledReason = latestProposal
    ? supervisorPilotUnavailableReason(projectRoot, latestProposal.scope_draft.allowed_write_roots)
    : "请先生成方案。";
  useEffect(() => {
    // 3b 项目模式钉死主管编排：即使暂不可用（如方案带写根）也不回落经典——经典对 3b 后端必拒，
    // 回落只会把用户引向一个假门；不可用原因由开始按钮/选择器上脸，fail-closed。
    if (supervisorPilotDisabledReason && !isStation3bProject) setOrchestrationMode("classic");
  }, [supervisorPilotDisabledReason, isStation3bProject]);

  // 默认选最近一条现有会话（会话到齐后补；用户手动选过就不覆盖）。无可用会话保持 null → UI 给人话提示。
  const sessionDefaultedRef = useRef(false);
  useEffect(() => {
    if (sessionDefaultedRef.current) return;
    const latest = [...projectSessions].sort(
      (a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0),
    )[0];
    if (latest) {
      // 方案a fix：函数式「只填空位」——用户已选（含 NEW_SESSION_CHOICE）绝不覆盖（旧写法会把
      // 显式的「开个新的」无声改回最新会话）。
      setSessionChoiceState((prev) => prev ?? latest.thread_id);
      sessionDefaultedRef.current = true;
    }
  }, [projectSessions]);

  // 主管试点进度来自它自己的 sidecar 账本。链读模型照旧可轮询，但不拿它猜主管状态。
  useEffect(() => {
    if (manualPhase !== "running" || !projectWorkflow || !supervisorPilotRunId) return;
    let active = true;
    const poll = async () => {
      try {
        const readModel = await loadSupervisorPilotReadModel({
          project_root: projectWorkflow.project_root,
          workflow_id: projectWorkflow.workflow_id,
          run_id: supervisorPilotRunId,
        });
        if (!active) return;
        setSupervisorPilotReadModel(readModel);
        setSupervisorPilotLedgerError(null);
      } catch (error) {
        if (active) {
          setSupervisorPilotLedgerError(error instanceof Error ? error.message : String(error));
        }
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [manualPhase, projectWorkflow, supervisorPilotRunId]);

  // 当前该显哪张脸：手动相位优先；否则由「有没有方案」决定说/批。
  const phase: JiaobanPhase = manualPhase ?? (latestProposal ? "authorize" : "say");
  // 右区默认看本态的定稿物；同一 authorize 相位换了新方案，也回到新方案本身。
  // 这里只切既有呈现 state，不参与相位/命令/读模型判断。
  useLayoutEffect(() => {
    setCanvasViewKey(phase === "done" ? "delivery" : phase === "authorize" && latestProposal ? "proposal" : "graph");
  }, [phase, latestProposal?.proposal_id]);
  // fix6-v2：只认「这一轮」的链——started_at（后端 ms）>= 本轮点击时刻。旧链时间戳更早、天然排除，
  // 防拆任务期（本轮链还没起、轮询却拿到旧链的绿状态）提前翻交货 / 显旧步骤。链没起 = null → 照显「主管正在拆任务」。
  const thisRoundChainStatus =
    chainStatus && runStartedAtMs != null && Number(chainStatus.started_at) >= runStartedAtMs ? chainStatus : null;
  const isDirectorPlanning = isDirectorPlanningPhase(phase, thisRoundChainStatus);
  const [directorPlanningElapsedMinutes, setDirectorPlanningElapsedMinutes] = useState(0);
  useEffect(() => {
    if (!isDirectorPlanning) {
      setDirectorPlanningElapsedMinutes(0);
      return;
    }
    const startedAt = Date.now();
    const refresh = () => {
      setDirectorPlanningElapsedMinutes(Math.floor((Date.now() - startedAt) / 60_000));
    };
    refresh();
    const timer = window.setInterval(refresh, 1_000);
    return () => window.clearInterval(timer);
  }, [isDirectorPlanning]);

  // ===== B1·全局主管复核（advisory·意见不是闸）=====
  // 交货翻脸 → 自动起复核（fire-and-forget·async 不挡交货·定稿第 3 条）；幂等防重烧（后端同轮
  // 记录直接回、[重新复核]/[重试] 才 force）；结果态缓存进 JiaobanRunCache（重挂载先读缓存、
  // 无缓存按幂等键补拉——后端幂等命中秒回不重烧）。前端只传定位键，复核内容全在后端盘上读。
  const [supervisorReview, setSupervisorReview] = useState<{
    key: string;
    outcome: GlobalSupervisorReviewOutcome;
  } | null>(cached?.supervisorReview ?? null);
  const [supervisorLoading, setSupervisorLoading] = useState(false);
  const supervisorInflightRef = useRef(false);

  async function requestSupervisorReview(chainStartedAt: string, force: boolean) {
    if (!projectWorkflow || supervisorInflightRef.current) return;
    supervisorInflightRef.current = true;
    setSupervisorLoading(true);
    try {
      const outcome = await runGlobalSupervisorReview({
        project_root: projectRoot,
        workflow_id: projectWorkflow.workflow_id,
        chain_started_at: chainStartedAt,
        force,
      });
      setSupervisorReview({ key: chainStartedAt, outcome });
      writeJiaobanRunCache(projectRoot, { supervisorReview: { key: chainStartedAt, outcome } });
    } catch (e) {
      // 复核失败不挡任何事：兜「复核不可用（人话）+ 重试」，绝不零出路、绝不断交货脸。
      const fallback: GlobalSupervisorReviewOutcome = {
        status: "unavailable",
        review: null,
        reason: e instanceof Error ? e.message : String(e),
        warnings: [],
      };
      setSupervisorReview({ key: chainStartedAt, outcome: fallback });
      writeJiaobanRunCache(projectRoot, { supervisorReview: { key: chainStartedAt, outcome: fallback } });
    } finally {
      supervisorInflightRef.current = false;
      setSupervisorLoading(false);
    }
  }

  // 交货翻脸自动触发。定位键 = 本轮链 started_at；ran 直翻 done 时轮询已停、thisRoundChainStatus
  // 可能为 null → 用现成只读命令补拉一次，并按 fix6-v2 同口径校验（started_at >= 本轮起点）防拿旧轮。
  // 拿不到键 → 本次不复核（区块零渲染·复核缺席不挡交货）。
  useEffect(() => {
    if (phase !== "done" || !projectWorkflow) return;
    let cancelled = false;
    const resolveKeyThenRun = async () => {
      let startedAt: string | null = thisRoundChainStatus?.started_at ?? null;
      if (!startedAt) {
        try {
          const status = await getProjectWorkflowChainStatus(projectRoot, projectWorkflow.workflow_id);
          const candidate = status?.started_at ?? null;
          if (candidate && (runStartedAtMs == null || Number(candidate) >= runStartedAtMs)) {
            if (!cancelled) setChainStatus(status);
            startedAt = candidate;
          }
        } catch {
          // 拿不到定位键：本次不复核（不冻、不猜轮次）。
        }
      }
      if (cancelled || !startedAt) return;
      if (supervisorReview?.key === startedAt) return; // 本轮结果已在（含缓存恢复），不重发。
      void requestSupervisorReview(startedAt, false);
    };
    void resolveKeyThenRun();
    return () => {
      cancelled = true;
    };
    // requestSupervisorReview 每渲染新建（组件内 async fn）——依赖只锚触发条件与幂等键，防 effect 空转。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [phase, thisRoundChainStatus?.started_at, projectWorkflow?.workflow_id, supervisorReview?.key]);

  // ===== B2·全局主管·批前边界意见（advisory·意见不是闸）=====
  // 批脸自动触发（async·不挡批·§2.2）：只对「今天生成的 pending 方案」触发（stale 不触发=省额度；纯建议方案
  // 照常触发·点破 mismatch 与 fix9 警条互证）。结果按 proposal_id 缓存（照 worksmap 先例·重挂载先读缓存，
  // 后端 proposal_id 幂等命中秒回不重烧）。前端只传定位键，内容后端盘读。
  const [boundaryReview, setBoundaryReview] = useState<{
    proposalId: string;
    outcome: GlobalSupervisorBoundaryReviewOutcome;
  } | null>(() => {
    const proposalId = latestProposal?.proposal_id;
    if (!proposalId) return null;
    const cached = jiaobanBoundaryReviewCacheByProposal.get(proposalId);
    return cached ? { proposalId, outcome: cached } : null;
  });
  const [boundaryLoading, setBoundaryLoading] = useState(false);
  // 记「正在为哪份方案飞」——改要求换了新方案时不硬锁（新方案照常触发·两条只读 consult 并行属预期）。
  const boundaryInflightRef = useRef<string | null>(null);

  async function requestBoundaryReview(proposalId: string, force: boolean) {
    if (boundaryInflightRef.current === proposalId) return; // 同方案已在飞·不重发
    boundaryInflightRef.current = proposalId;
    setBoundaryLoading(true);
    try {
      const outcome = await runGlobalSupervisorBoundaryReview({
        project_root: projectRoot,
        proposal_id: proposalId,
        force,
      });
      setBoundaryReview({ proposalId, outcome });
      jiaobanBoundaryReviewCacheByProposal.set(proposalId, outcome);
    } catch (e) {
      // 意见失败绝不挡批：兜「不可用（人话）+ 重试」，绝不零出路、绝不断批脸。
      const fallback: GlobalSupervisorBoundaryReviewOutcome = {
        status: "unavailable",
        review: null,
        reason: e instanceof Error ? e.message : String(e),
        warnings: [],
      };
      setBoundaryReview({ proposalId, outcome: fallback });
      jiaobanBoundaryReviewCacheByProposal.set(proposalId, fallback);
    } finally {
      // 只由「当前在飞的那份」收尾（被新方案接管时不误清）。
      if (boundaryInflightRef.current === proposalId) {
        boundaryInflightRef.current = null;
        setBoundaryLoading(false);
      }
    }
  }

  // 批脸挂载/方案切换自动触发。命中缓存直接恢复（重挂载不重烧）；否则对「今天的 pending 方案」invoke。
  // stale/非 pending → 不触发（区块零渲染·意见缺席不挡批）。
  useEffect(() => {
    if (phase !== "authorize") return;
    const proposalId = latestProposal?.proposal_id ?? null;
    if (!proposalId || !shouldRequestBoundaryReview(latestProposal)) return;
    const cached = jiaobanBoundaryReviewCacheByProposal.get(proposalId);
    if (cached) {
      setBoundaryReview({ proposalId, outcome: cached });
      return;
    }
    if (boundaryReview?.proposalId === proposalId) return; // 已有本方案结果（含刚 set），不重发。
    void requestBoundaryReview(proposalId, false);
    // requestBoundaryReview 每渲染新建——依赖只锚触发条件与方案 id，防 effect 空转。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [phase, latestProposal?.proposal_id, latestProposal?.status]);

  // ---- 动作 ----

  // 常驻输入框在所有状态都只调用 submit_supervisor_resident_answer。卡上的“按我说的改”复用
  // 同一草稿和同一提交，不再存在“目标→直接出方案”的第二条路线。
  const conversationComposer = makeConversationComposer({ isTestProject });

  // 刀2「批前看图」触发条件 = 工作流开关开着（AI 建议时开关默认开·见上面 reset effect；用户也可手动开/关）。
  // 简单活默认关 → 不触发、不加一秒等待。
  const shouldPreviewWorksmap = workflowSwitchOn;

  // 换方案（新 proposal_id）→ 清上一份预拆展示态 + 关开关；新方案按需重新触发/命中缓存。
  useEffect(() => {
    setPreviewTasks(null);
    setPreviewWarnings([]);
    setPreviewError(null);
    setWorkflowSwitchOn(latestProposal?.suggest_workflow === true);
    // M1：每份待批方案从「新会话」起，不把上一单的 existing 悄悄继承给任何预演节点。
    setSessionChoiceState(NEW_SESSION_CHOICE);
    sessionDefaultedRef.current = true;
    setPreviewSessionBindings([]);
    setRunCanvasNodes([]);
    setRunCanvasBindings([]);
    writeJiaobanRunCache(projectRoot, {
      sessionChoice: NEW_SESSION_CHOICE,
      previewSessionBindings: [],
      runCanvasNodes: [],
      runCanvasBindings: [],
    });
    previewLoadingRef.current = false;
  }, [latestProposal?.proposal_id, projectRoot]);

  // 预拆触发：命中缓存直接恢复（切 tab 回来不重拆）；否则异步调后端预拆（零写盘·1-7 分钟），持住结果。防重入靠 ref。
  // P2-A：方案自带任务图时不再触发这个旧 LM 预拆调用——previewCanvasNodesFor 直接读 proposal.tasks 秒出；
  // 这条只保留给存量无图方案（fallback）。
  useEffect(() => {
    const proposalId = latestProposal?.proposal_id;
    if (!proposalId || !shouldPreviewWorksmap || (latestProposal?.tasks?.length ?? 0) > 0) return;
    const cachedPreview = jiaobanPreviewCacheByProposal.get(proposalId);
    if (cachedPreview) {
      setPreviewTasks(cachedPreview.tasks);
      setPreviewWarnings(cachedPreview.warnings);
      setPreviewError(null);
      return;
    }
    if (previewLoadingRef.current) return;
    previewLoadingRef.current = true;
    setPreviewLoading(true);
    setPreviewError(null);
    previewPendingProposalDirectorPlan({ project_root: projectRoot, proposal_id: proposalId })
      .then((result) => {
        jiaobanPreviewCacheByProposal.set(proposalId, { tasks: result.planned_tasks, warnings: result.warnings });
        setPreviewTasks(result.planned_tasks);
        setPreviewWarnings(result.warnings);
      })
      .catch((error) => {
        setPreviewError(humanizePreviewError(error));
      })
      .finally(() => {
        previewLoadingRef.current = false;
        setPreviewLoading(false);
      });
  }, [latestProposal?.proposal_id, shouldPreviewWorksmap, projectRoot, previewNonce]);

  // 重试预拆：清该方案缓存 + bump nonce 让上面 effect 重跑。
  function retryPreview() {
    const proposalId = latestProposal?.proposal_id;
    if (!proposalId) return;
    jiaobanPreviewCacheByProposal.delete(proposalId);
    previewLoadingRef.current = false;
    setPreviewError(null);
    setPreviewTasks(null);
    setPreviewNonce((nonce) => nonce + 1);
  }

  // 主管试点只复用既有确认/边界激活两步；它刻意不进入经典链的拆任务、绑定或派发入口。
  async function confirmAndLaunchSupervisorPilot() {
    if (!latestProposal || !projectWorkflow) throw new Error("当前方案或工作流缺失，不能启动主管试点。");
    const isReadOnly = latestProposal.scope_draft.allowed_write_roots.length === 0;
    const confirmed = await recordProjectConsultationProposalDecision({
      project_root: projectRoot,
      proposal_id: latestProposal.proposal_id,
      actor_id: "user",
      decision: "confirm",
      summary: isReadOnly
        ? "用户点[允许并开始]：确认只读单，选择主管编排试点（不进入经典链）。"
        : "用户点[允许并开始]：确认授权写根内的写单，选择主管编排试点（主管只读、worker 按写根执行）。",
      expected_proposal_store_revision: projectConsultationProposalStore?.revision ?? null,
      expected_plan_authorization_store_revision: null,
    });
    const authorization = confirmed.plan_authorization;
    const authorizationRevision = confirmed.plan_authorization_store_revision;
    if (!authorization || authorizationRevision == null) {
      throw new Error("确认方案未产出可激活授权，不能启动主管试点。");
    }
    const activated = await recordGlobalBoundaryReview({
      project_root: projectRoot,
      project_id: confirmed.proposal.project_id,
      workflow_id: confirmed.proposal.workflow_id,
      proposal_id: confirmed.proposal.proposal_id,
      authorization_id: authorization.authorization_id,
      actor_id: "user",
      review_status: "approved",
      summary: isReadOnly
        ? "用户点[允许并开始]：确认主管试点仅在只读沙箱内运行。"
        : "用户点[允许并开始]：确认主管自身只读；worker 仅在方案授权写根内运行。",
      checklist: {
        architecture_boundary_checked: true,
        cross_project_impact_checked: true,
        permission_scope_checked: true,
        read_write_scope_checked: true,
        tool_and_check_scope_checked: true,
        memory_boundary_checked: true,
        stop_conditions_checked: true,
        acceptance_criteria_checked: true,
      },
      findings: [],
      expected_authorization_revision: authorizationRevision,
    });
    if (activated.authorization.status !== "active") {
      throw new Error("主管试点授权未激活，已拒绝发射。");
    }
    return launchSupervisorPilot({
      project_root: projectRoot,
      workflow_id: projectWorkflow.workflow_id,
      authorization_id: activated.authorization.authorization_id,
      reasoning_effort: SUPERVISOR_PILOT_REASONING_EFFORT,
    });
  }

  // 允许并开始 = 方案授权人闸那一下。经典模式走刀1 合流命令 confirm_and_start_authorized_run（见 lib/tauri）。
  async function authorizeAndStart() {
    if (!projectWorkflow || !latestProposal || starting || runningRef.current) return;
    const pilotSelected = orchestrationMode === "supervisor_pilot";
    if (pilotSelected && supervisorPilotDisabledReason) {
      setStartError(supervisorPilotDisabledReason);
      return;
    }
    // 站 3b 皮带扣：任何路径滑到经典模式都在此拦住（后端 S1 闸同样会拒，这里给人话）。
    if (isStation3bProject && !pilotSelected) {
      setStartError("站 3b 项目只开通主管编排只读单。");
      return;
    }
    rememberRunCanvas();
    // ★ 一进来立刻上「正在干」脸——不等 await 回来（await 可能几十秒~几分钟，中间不能无脸看着像冻死）。
    runningRef.current = true;
    ranProposalIdRef.current = latestProposal.proposal_id;
    const runStartedAt = Date.now(); // fix6-v2：本轮起点，判「这轮的链」用
    // 顶层选择只是合流命令的枚举校验位(existing|new)；P1-D 后不再有绑定面板消费它的预填值——
    // 真正的会话选择走 M1 预演画布(previewBindingsForCanvas)或默认自动新会话。
    const topLevelUsesNew = sessionChoice === NEW_SESSION_CHOICE || sessionChoice === null;
    const isNewSession = false;
    setRunStartedAtMs(runStartedAt);
    setRunIsNewSession(isNewSession);
    setStarting(true);
    setStartError(null);
    setOutcome(null);
    setChainStatus(null);
    setSupervisorPilotRunId(null);
    setSupervisorPilotReadModel(null);
    setSupervisorPilotLedgerError(null);
    setSupervisorReview(null); // B1：新一轮开跑，上一轮复核意见随之清（防旧轮意见冒充本轮）。
    setManualPhase("running");
    writeJiaobanRunCache(projectRoot, {
      manualPhase: "running",
      outcome: null,
      startError: null,
      ranProposalId: latestProposal.proposal_id,
      runStartedAtMs: runStartedAt,
      runIsNewSession: isNewSession,
      supervisorReview: null,
      supervisorPilotRunId: null,
    });
    try {
      if (pilotSelected) {
        const receipt = await confirmAndLaunchSupervisorPilot();
        setSupervisorPilotRunId(receipt.run_id);
        setManualPhase("running");
        writeJiaobanRunCache(projectRoot, {
          manualPhase: "running",
          supervisorPilotRunId: receipt.run_id,
        });
        void onProposalStoreRefresh?.().catch(() => {
          // 已发射的主管不能因刷新展示店失败被误报为启动失败。
        });
        return;
      }
      // 用户只点一次人闸：把预演节点的逐项选择随同方案提交；后端拆完按 id/顺序保守映射，复用既有绑定校验。
      const outcome = await confirmAndStartAuthorizedRun({
        project_root: projectRoot,
        proposal_id: latestProposal.proposal_id,
        session_choice: topLevelUsesNew ? "new" : "existing",
        session_id: topLevelUsesNew ? undefined : (sessionChoice ?? undefined),
        actor_id: "user",
        // 刀2 所批即所跑：批前预拆成功过就把那份图原样带回 → 后端照图跑不重拆；简单活/预拆失败 previewTasks 空 → 不传。
        approved_planned_tasks: previewTasks ?? undefined,
        preview_session_bindings: previewBindingsForCanvas,
      });
      const nextPhase = jiaobanPhaseForOutcome(outcome);
      setOutcome(outcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, {
        manualPhase: nextPhase,
        outcome,
        startError: null,
      });
      refreshWorkflowStateReadModelAfterSuccessfulChainAction();
    } catch (e) {
      // record_decision 成功后的后续写（尤其边界批准）仍可能失败。先刷新方案店：否则 props 还停在
      // pending，界面既不再待批、又不知道已确认而不给[接着跑]。刷新失败不覆盖原始错误。
      try {
        await onProposalStoreRefresh?.();
      } catch {
        // 刷新只是把已落库状态带回界面；合流的原始失败才是此刻应展示的主因。
      }
      // 合流命令对「已确认」会干净拒（后端：方案不是待用户确认状态）。这不是坏了——授权本还活着，
      // 直接调 auto_advance 就能从拆任务接着跑。刷新后的 user_confirmed 也会由 planIsConfirmed 放行。
      const alreadyConfirmed = isAlreadyConfirmedRejection(e);
      const humanized = humanizeAuthorizeError(e);
      setStartError(humanized);
      setContinueHint(alreadyConfirmed);
      setManualPhase("blocked");
      writeJiaobanRunCache(projectRoot, {
        manualPhase: "blocked",
        startError: humanized,
        lastStopReason: humanized,
        continueHint: alreadyConfirmed,
      });
    } finally {
      setStarting(false);
      runningRef.current = false;
    }
  }

  // 接着跑，不用重批：方案已 user_confirmed（授权还活着）时的重试口——不走合流命令（会被「已确认」拒），
  // 直接调现成 autoAdvanceAuthorizedRoleLoop 从拆任务接着推进。防冻套路同 authorizeAndStart：
  // 先切「正在干」相位再 await、runningRef 防重入、缓存同步；返回走现有 outcome→脸 映射。
  async function continueRun() {
    if (!projectWorkflow || starting || runningRef.current) return;
    if (runCanvasNodes.length === 0) rememberRunCanvas();
    runningRef.current = true;
    const runStartedAt = Date.now(); // fix6-v2：本轮起点
    setRunStartedAtMs(runStartedAt);
    setRunIsNewSession(false); // 方案a：接着跑用现有链，不是新建会话
    setStarting(true);
    setStartError(null);
    setContinueHint(false);
    setOutcome(null);
    setChainStatus(null);
    setSupervisorReview(null); // B1：新一轮开跑，上一轮复核意见随之清。
    setManualPhase("running");
    writeJiaobanRunCache(projectRoot, {
      manualPhase: "running",
      outcome: null,
      startError: null,
      continueHint: false,
      runStartedAtMs: runStartedAt,
      runIsNewSession: false,
      supervisorReview: null,
    });
    try {
      const nextOutcome = await autoAdvanceAuthorizedRoleLoop({
        project_root: projectWorkflow.project_root,
        workflow_id: projectWorkflow.workflow_id,
        actor_id: "user",
      });
      const nextPhase = jiaobanPhaseForOutcome(nextOutcome);
      setOutcome(nextOutcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, { manualPhase: nextPhase, outcome: nextOutcome, startError: null });
      refreshWorkflowStateReadModelAfterSuccessfulChainAction();
    } catch (e) {
      // 接着跑也可能失败（如授权真过期）→ 翻人话进卡住脸。若仍是「已确认」类，continueHint 留着还给[接着跑]。
      const alreadyConfirmed = isAlreadyConfirmedRejection(e);
      const humanized = humanizeAuthorizeError(e);
      setStartError(humanized);
      setContinueHint(alreadyConfirmed);
      setManualPhase("blocked");
      writeJiaobanRunCache(projectRoot, {
        manualPhase: "blocked",
        startError: humanized,
        lastStopReason: humanized,
        continueHint: alreadyConfirmed,
      });
    } finally {
      setStarting(false);
      runningRef.current = false;
    }
  }

  async function applyDecisionAction(action: "retry" | "change_session" | "rework" | "archive") {
    if (!projectWorkflow || !outcome?.chain_outcome || starting) return;
    const plannedTaskId =
      outcome.chain_outcome.steps.find((step) => step.state === "waiting_decision")?.planned_task_id ??
      outcome.chain_outcome.steps.find((step) => step.state === "needs_rework")?.planned_task_id ??
      thisRoundChainStatus?.nodes.find((node) => node.state === "waiting_decision")?.node_id ??
      thisRoundChainStatus?.nodes.find((node) => node.state === "needs_rework")?.node_id;
    const plannedTask = outcome.planned_tasks?.find((task) => task.planned_task_id === plannedTaskId);
    if (!plannedTaskId || !plannedTask) {
      setNeedsReworkActionError("这单的处置信息还没载入，暂时不能执行该动作。");
      return;
    }

    setStarting(true);
    setNeedsReworkActionError(null);
    try {
      const result = await applyProjectDirectorFailedAction({
        project_root: projectWorkflow.project_root,
        workflow_id: projectWorkflow.workflow_id,
        chain_run_id: outcome.chain_outcome.chain_run_id,
        planned_task_id: plannedTaskId,
        action,
        actor_role: "project_director",
        actor_id: "user",
        explicit_retry_or_reopen: action === "retry" || action === "change_session",
        planned_task: plannedTask,
        max_nodes: 1,
      });
      const nextChainOutcome =
        action === "retry" || action === "change_session"
          ? (result.chain_outcome ?? null)
          : action === "rework"
            ? outcome.chain_outcome
            : null;
      const nextOutcome: AutoAdvanceRoleLoopOutcome = {
        stage:
          action === "retry" || action === "change_session"
            ? jiaobanStageFromChainOutcome(nextChainOutcome)
            : "interrupted",
        planned_task_count: outcome.planned_task_count,
        prepared_count: outcome.prepared_count,
        needs_binding_count: outcome.needs_binding_count,
        blocked_count: outcome.blocked_count,
        message: result.message,
        chain_outcome: nextChainOutcome,
        stop_reason: result.stopped_reason ?? null,
        planned_tasks: outcome.planned_tasks,
        warnings: result.warnings,
      };
      const nextPhase = jiaobanPhaseForOutcome(nextOutcome);
      setOutcome(nextOutcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, { manualPhase: nextPhase, outcome: nextOutcome, startError: null });
      refreshWorkflowStateReadModelAfterSuccessfulChainAction();
      try {
        const status = await getProjectWorkflowChainStatus(projectRoot, projectWorkflow.workflow_id);
        if (status) setChainStatus(status);
      } catch {
        // 处置结果已返回；进度读模型暂缺不阻断主脸。
      }
    } catch (error) {
      setNeedsReworkActionError(error instanceof Error ? error.message : String(error));
    } finally {
      setStarting(false);
    }
  }

  async function stopRun() {
    if (!projectWorkflow) return;
    try {
      await stopProjectWorkflowChain({
        project_root: projectWorkflow.project_root,
        workflow_id: projectWorkflow.workflow_id,
      });
      refreshWorkflowStateReadModelAfterSuccessfulChainAction();
    } catch (e) {
      setStartError(e instanceof Error ? e.message : String(e));
    }
    setManualPhase("blocked");
    writeJiaobanRunCache(projectRoot, { manualPhase: "blocked" });
  }

  // 重新出方案：回「说」面，**带上原目标 + 上次停因**——不再落空白首屏。
  function backToSay() {
    // 停因优先取本轮实况（outcome 停因 / 报错），带到「说」面顶部当摘要。
    const reason =
      outcome?.stop_reason?.trim() || outcome?.message?.trim() || startError?.trim() || null;
    setSayHint(reason);
    // 目标带回常驻框(07-18 唯一对话框):改一改 Enter 就重出。
    if (latestProposal?.user_goal) setConversationComposerDraft(latestProposal.user_goal);
    setManualPhase("say");
    setCanvasViewKey("graph");
    setOutcome(null);
    setStartError(null);
    setContinueHint(false);
    setChainStatus(null);
    ranProposalIdRef.current = null;
    clearJiaobanRunCache(projectRoot);
  }

  // 只按真实链终态翻脸：completed 才交货；失败/中断/待决定各去自己的脸。
  useEffect(() => {
    if (phase !== "running") return;
    const state = thisRoundChainStatus?.state.trim().toLowerCase();
    const nextPhase: JiaobanPhase | null = ["finished", "completed", "done", "succeeded"].includes(state ?? "")
      ? "done"
      : state === "waiting_decision"
        ? "waiting_decision"
        : ["aborted", "stopped", "interrupted", "failed", "archived"].includes(state ?? "")
          ? "blocked"
          : null;
    if (!nextPhase) return;
    setManualPhase(nextPhase);
    writeJiaobanRunCache(projectRoot, { manualPhase: nextPhase });
  }, [phase, thisRoundChainStatus, projectRoot]);

  // 非测试项目：老实标注 + 跳智能体直连，不装能跑。（站 3b 项目例外：进交办流、只开主管只读。）
  if (!isTestProject && !isStation3bProject) {
    const latestSession =
      projectSessions.sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;
    return (
      <section className="project-jiaoban" aria-label="交办">
        <div className="project-canvas-detail-card" aria-label="交办 · 这个项目暂不能自动干">
          <div className="panel-heading">
            <div>
              <h3>这个项目现在用智能体直连</h3>
            </div>
            <Pill tone="unknown">未开通自动干</Pill>
          </div>
          <div className="role-loop-plain" aria-label="老实说明">
            <p className="role-loop-plain-lead">
              自动干目前只在固定测试项目开通；这个项目现在可用智能体直连——你自己一步步来。
            </p>
            <p className="role-loop-plain-note">
              交办 = 说一句、批一次、AI 自动跑一串；智能体 = 手动直连一个对话。这个项目走后者。
            </p>
          </div>
          <div className="workflow-state-actions">
            <button
              className="primary-button"
              type="button"
              disabled={!latestSession}
              onClick={() => latestSession && onOpenAgentSession(latestSession.thread_id)}
            >
              去智能体直连
            </button>
          </div>
          {!latestSession ? <p className="muted small-note">这个项目还没有可打开的对话。</p> : null}
        </div>
      </section>
    );
  }

  // 没有本项目工作流：交办跑不起来（说态也要有工作流才能出方案）。给老实提示 + 跳智能体，永不冻。
  if (!projectWorkflow) {
    return (
      <section className="project-jiaoban" aria-label="交办">
        <div className="project-canvas-detail-card">
          <div className="panel-heading">
            <div>
              <h3>这个项目还没准备好交办</h3>
            </div>
            <Pill tone="warn">缺项目工作流</Pill>
          </div>
          <p className="muted small-note">先在右侧画布建起这个项目的工作流，再回来交办。</p>
        </div>
      </section>
    );
  }

  // B2 展示门控：意见永远跟着当前这份方案（proposalId 对上才显·防旧方案意见冒充新方案）。
  const boundaryForThisProposal =
    boundaryReview && latestProposal && boundaryReview.proposalId === latestProposal.proposal_id
      ? boundaryReview.outcome
      : null;
  const boundaryLoadingForThisProposal =
    boundaryLoading && !boundaryForThisProposal && shouldRequestBoundaryReview(latestProposal);

  // [接着跑] 出现的硬前提（§2.1）：方案已 user_confirmed（授权还活着，本不用重批）。
  // 两条来路都算数：① 当前 latestProposal.status==user_confirmed；② 合流命令刚拒过「已确认」(continueHint)——
  // 后者时 store 里那份就是已确认态，只是 summary 可能还没刷到，故 continueHint 直接放行。
  const planIsConfirmed = latestProposal?.status === "user_confirmed" || continueHint;

  // 「看原始对话」桥·兜底会话：本项目最近一条会话的 thread_id（按更新时间倒序头一条）。
  // 哨兵单（新会话·真 id 前端拿不到）跑中/交货/卡住时用它兜底「看最近对话」。不 mutate 原数组。
  const latestSessionThreadId =
    [...projectSessions].sort(
      (a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0),
    )[0]?.thread_id ?? null;
  const needsReworkStep = outcome?.chain_outcome?.steps.find((step) => step.state === "needs_rework") ?? null;
  const needsReworkNode = thisRoundChainStatus?.nodes.find((node) => node.state === "needs_rework") ?? null;
  const needsReworkTaskId = needsReworkStep?.planned_task_id ?? needsReworkNode?.node_id ?? null;
  const needsRework = needsReworkTaskId
    ? {
        reason:
          needsReworkNode?.message?.trim() || "主管认为这一步还需要重做，请从下面选择怎么处理。",
        actionsReady: outcome?.planned_tasks?.some((task) => task.planned_task_id === needsReworkTaskId) ?? false,
      }
    : null;
  const waitingDecisionStep =
    outcome?.chain_outcome?.steps.find((step) => step.state === "waiting_decision") ?? null;
  const waitingDecisionNode =
    thisRoundChainStatus?.nodes.find((node) => node.state === "waiting_decision") ?? null;
  const waitingDecisionTaskId =
    waitingDecisionStep?.planned_task_id ?? waitingDecisionNode?.node_id ?? null;
  const waitingDecision = waitingDecisionTaskId
    ? {
        reason:
          waitingDecisionNode?.message?.trim() ||
          waitingDecisionStep?.report_summary?.trim() ||
          outcome?.message?.trim() ||
          "worker 停下来向你求助，请选择下一步。",
        actionsReady:
          outcome?.planned_tasks?.some((task) => task.planned_task_id === waitingDecisionTaskId) ?? false,
      }
    : {
        reason: outcome?.message?.trim() || "worker 停下来向你求助，请选择下一步。",
        actionsReady: false,
      };

  // Part①·历史拉取（挂载/交货翻脸/重拆各一次·不轮询）。失败静默：读不到就保留已有、不上错、不挡五态主区。
  async function loadHistory() {
    if (!projectWorkflow || historyLoadingRef.current) return;
    historyLoadingRef.current = true;
    setHistoryLoading(true);
    try {
      const result: RunHistoryList = await listProjectRunHistory({
        project_root: projectRoot,
        workflow_id: projectWorkflow.workflow_id,
        limit: 50,
      });
      setHistory(result.entries);
      setHistoryTotal(result.total);
    } catch {
      // 历史是增益不是闸。
    } finally {
      historyLoadingRef.current = false;
      setHistoryLoading(false);
    }
  }
  useEffect(() => {
    void loadHistory();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectRoot, phase === "done", latestProposal?.proposal_id]);

  // Part①·历史派生：当前单 / 最新卡住单（仅它给行内[接着跑]·倒序头一条 blocked）/ 选中的非当前单（→详情卡）。
  const currentProposalId = latestProposal?.proposal_id ?? null;
  const latestBlockedId = history.find((entry) => entry.state === "blocked")?.proposal_id ?? null;
  const selectedHistoryEntry = selectedHistoryId
    ? history.find((entry) => entry.proposal_id === selectedHistoryId) ?? null
    : null;
  const currentHistoryEntry = currentProposalId
    ? history.find((entry) => entry.proposal_id === currentProposalId) ?? null
    : null;

  const supervisorConversationEntries = useMemo(
    () => supervisorConversationEntriesForProject(workflowState, projectRoot, projectWorkflow?.workflow_id),
    [workflowState, projectRoot, projectWorkflow?.workflow_id],
  );
  const conversationStarted =
    consultLoading ||
    Boolean(consultError) ||
    supervisorConversationEntries.length > 0 ||
    latestProposal != null ||
    phase !== "say";

  function selectProposalIndexEntry(entry: RunHistoryEntry) {
    const isCurrent = entry.proposal_id === currentProposalId;
    setSelectedHistoryId(isCurrent ? null : entry.proposal_id);
    setCanvasViewKey(entry.state === "delivered" ? "delivery" : "proposal");
  }

  function backToCurrentArtifact() {
    setSelectedHistoryId(null);
    const currentView = phase === "done" ? "delivery" : currentProposalId ? "proposal" : "graph";
    setCanvasViewKey(currentView);
  }

  const proposalIndexContent = (
    <JiaobanProposalIndex
      entries={history}
      total={historyTotal}
      loading={historyLoading}
      filter={historyFilter}
      onFilterChange={setHistoryFilter}
      selectedId={selectedHistoryId}
      currentProposalId={currentProposalId}
      latestBlockedId={latestBlockedId}
      onSelectEntry={selectProposalIndexEntry}
      onBackToCurrent={backToCurrentArtifact}
      onNewJiaoban={() => {
        setSelectedHistoryId(null);
        backToSay();
      }}
      onContinueRun={() => void continueRun()}
      knownProposalIds={knownProposalIds}
    />
  );
  const focusedProposalId = selectedHistoryId ?? currentProposalId;
  const focusedProposal = focusedProposalId
    ? workflowProposals.find((proposal) => proposal.proposal_id === focusedProposalId) ?? null
    : latestProposal;
  const focusedProposalIsLatest = focusedProposal?.proposal_id === latestProposal?.proposal_id;
  const proposalInteractive =
    focusedProposalIsLatest &&
    selectedHistoryId == null &&
    phase === "authorize";
  const proposalCard = focusedProposal ? (
    <JiaobanAuthorizeState
      proposal={focusedProposal}
      proposalIsStale={proposalAgeDays(focusedProposal.created_at_ms) >= 1}
      // 卡上修改框退场；[按我说的改]提交的是常驻框当前草稿和唯一消息入口。
      amendment={proposalInteractive ? conversationComposer?.draft ?? "" : ""}
      onAmend={() => conversationComposer?.onSubmit()}
      onAuthorizeAndStart={() => void authorizeAndStart()}
      onRePlan={backToSay}
      onDecline={backToSay}
      starting={starting}
      // 普通对话的发送中/失败态只属于中栏，不得锁住或污染右侧批准卡。
      consultLoading={false}
      consultError={null}
      howRunSummary={howRunSummary}
      onShowGovernance={() => setCanvasViewKey("governance")}
      onShowHowRun={() => setCanvasViewKey("howrun")}
      boundaryLoading={proposalInteractive && boundaryLoadingForThisProposal}
      boundaryOutcome={proposalInteractive ? boundaryForThisProposal : null}
      onBoundaryRetry={() => {
        if (focusedProposalIsLatest) void requestBoundaryReview(focusedProposal.proposal_id, true);
      }}
      readOnly={!proposalInteractive}
    />
  ) : null;
  const artifactNotices = artifactNoticesForConversation({
    proposals: workflowProposals,
    history,
    currentProposalId,
    includeCurrentDelivery: phase === "done",
    currentProposalCreatedAtMs: latestProposal?.created_at_ms ?? null,
    onActivate: (kind, proposalId) => {
      setSelectedHistoryId(proposalId === currentProposalId ? null : proposalId);
      setCanvasViewKey(kind);
    },
  });
  const baseConversationGoal =
    workflowProposals[0]?.user_goal ??
    null;
  const persistedUserTurns = userTurnsFromProposalHistory(workflowProposals);
  const timelineUserTurns = mergeConversationUserTurns(
    baseConversationGoal,
    persistedUserTurns,
    [],
  );
  // A3·视口停最新：当前单可见内容(条数/相位/等待态)变了就滚回底部,逻辑本体在会话 hook 里。
  useConversationAutoScroll(
    `${supervisorConversationEntries.length}:${timelineUserTurns.length}:${artifactNotices.length}:${phase}:${consultLoading}:${residentMessageBusyKey ?? ""}`,
  );
  const currentDeliveryCard = phase === "done" ? (
    <JiaobanDoneState
      outcome={outcome}
      chainStatus={thisRoundChainStatus}
      onContinue={backToSay}
      needsRework={needsRework}
      needsReworkActionError={needsReworkActionError}
      needsReworkActionStarting={starting}
      onNeedsReworkContinue={() => void continueRun()}
      onNeedsReworkAction={(action) => void applyDecisionAction(action)}
      onRequestAction={onRequestAction}
      factCtx={{
        projectRoot: project.project_root,
        projectId: projectWorkflow?.project_id ?? null,
        workflowId: projectWorkflow?.workflow_id ?? null,
      }}
      sessionChoice={sessionChoice}
      latestSessionThreadId={latestSessionThreadId}
      onOpenAgentSession={onOpenAgentSession}
      derivedWorkflow={projectWorkflow?.derived_workflow ?? null}
      supervisorLoading={supervisorLoading}
      supervisorOutcome={supervisorReview?.outcome ?? null}
      onSupervisorRetry={() => {
        // [重试]/[重新复核]：force 穿透幂等重跑。键优先取已有结果的轮键，兜底本轮链 started_at。
        const key = supervisorReview?.key ?? thisRoundChainStatus?.started_at ?? null;
        if (key) void requestSupervisorReview(key, true);
      }}
      onSupervisorReplan={backToSay}
    />
  ) : null;
  const phaseContent = (
    <>
          {phase === "running" ? (
            supervisorPilotRunId ? (
              <JiaobanSupervisorPilotRunningState
                runId={supervisorPilotRunId}
                readModel={supervisorPilotReadModel}
                ledgerError={supervisorPilotLedgerError}
              />
            ) : (
              <JiaobanRunningState
                chainStatus={thisRoundChainStatus}
                directorPlanningElapsedMinutes={directorPlanningElapsedMinutes}
                isNewSession={runIsNewSession}
                onStop={() => void stopRun()}
                sessionChoice={sessionChoice}
                latestSessionThreadId={latestSessionThreadId}
                onOpenAgentSession={onOpenAgentSession}
              />
            )
          ) : null}

          {phase === "waiting_decision" ? (
            <JiaobanWaitingDecisionState
              reason={waitingDecision.reason}
              actionsReady={waitingDecision.actionsReady}
              starting={starting}
              error={needsReworkActionError}
              onContinue={() => void applyDecisionAction("retry")}
              onChangeSession={() => void applyDecisionAction("change_session")}
              onRework={() => void continueRun()}
              onArchive={() => void applyDecisionAction("archive")}
            />
          ) : null}

          {phase === "blocked" ? (
            <JiaobanBlockedState
              outcome={outcome}
              error={startError}
              planIsConfirmed={planIsConfirmed}
              sessions={projectSessions}
              sessionChoice={sessionChoice}
              onSessionChoiceChange={setSessionChoice}
              onContinueRun={() => void continueRun()}
              onRePlan={backToSay}
              starting={starting}
              onOpenWorkflow={onOpenWorkflow ?? null}
              latestSessionThreadId={latestSessionThreadId}
              onOpenAgentSession={onOpenAgentSession}
              replyDraft={blockedReply}
              onReplyDraftChange={setBlockedReply}
              // follow-up 回话通道后端未就绪（后端包 §C 勘察补缺中）→ 形态立住、按钮 disabled 带人话。
              // 通道落地后：这里改成真判据 + 传 onSendFollowUp，卡片形态零改。
              followUpReady={false}
            />
          ) : null}
    </>
  );
  const conversationPhaseKind: JiaobanConversationPhaseKind =
    phase === "say" ? "composer" : phase === "authorize" ? "proposal" : phase === "done" ? "delivery" : phase === "blocked" ? "legacy" : "conversation";
  const mainContent = (
    <div className="project-jiaoban-main">
      <div className="project-jiaoban-col" data-conversation-phase={conversationPhaseKind}>
        <JiaobanConversationStream
          entries={supervisorConversationEntries}
          userGoal={conversationStarted ? baseConversationGoal : null}
          userTurns={timelineUserTurns}
          artifactNotices={artifactNotices}
          proposals={workflowProposals}
          phaseKind={conversationPhaseKind}
          phaseContent={phase === "blocked" ? phaseContent : null}
          consultLoading={consultLoading}
          messageBusyKey={residentMessageBusyKey}
          messageErrors={messageErrors}
          onSupervisorProcessActivate={(entry) => {
            setSelectedHistoryId(null);
            setFocusedRuntimeNodeId(supervisorProcessFocusedNodeId(entry));
            setCanvasViewKey(supervisorProcessCanvasView(entry));
          }}
        />
        {conversationComposer ? (
          <JiaobanConversationComposer {...conversationComposer} />
        ) : null}
      </div>
    </div>
  );
  const runtimeCanvasPhase =
    phase === "running" || phase === "done" || phase === "waiting_decision" || phase === "blocked";
  const canvasNodes = runtimeCanvasPhase && runCanvasNodes.length > 0 ? runCanvasNodes : previewCanvasNodes;
  const canvasBindings = runtimeCanvasPhase && runCanvasBindings.length > 0 ? runCanvasBindings : previewBindingsForCanvas;
  const runtimeNodeStates = runtimeCanvasPhase
    ? jiaobanRuntimeNodeStates(canvasNodes, thisRoundChainStatus, outcome?.chain_outcome?.steps ?? [])
    : null;
  const showPlanCanvas =
    (phase === "authorize" && Boolean(latestProposal)) ||
    (runtimeCanvasPhase && canvasNodes.length > 0);
  const previewCanvas = showPlanCanvas ? (
    <JiaobanPlanPreviewCanvas
      nodes={canvasNodes}
      bindings={canvasBindings}
      sessions={projectSessions}
      waitingForPreview={
        !runtimeCanvasPhase &&
        workflowSwitchOn &&
        previewTasks === null &&
        (latestProposal?.tasks?.length ?? 0) === 0 &&
        !previewError
      }
      previewError={runtimeCanvasPhase ? null : previewError}
      previewWarnings={runtimeCanvasPhase ? [] : previewWarnings}
      readOnly={runtimeCanvasPhase}
      runtimeNodeStates={runtimeNodeStates}
      focusedNodeId={focusedRuntimeNodeId} focusActive={canvasViewKey === "graph"}
      onBindingChange={updatePreviewSessionBinding}
      onRetryPreview={retryPreview}
      onOpenAgentSession={onOpenAgentSession}
    />
  ) : null;

  // 右区是定稿物的家：中栏只说话；方案/交货卡连同原动作回调整体搬到这里。
  const selectedHistoryCard = selectedHistoryEntry
    ? <JiaobanHistoryDetail entry={selectedHistoryEntry} onBackToCurrent={backToCurrentArtifact} />
    : null;
  const currentDeliveryHistoryCard = currentHistoryEntry?.state === "delivered" ? (
    <JiaobanHistoryDetail entry={currentHistoryEntry} onBackToCurrent={backToCurrentArtifact} showBackAction={false} />
  ) : null;
  const proposalViewContent = proposalCard ?? (selectedHistoryEntry?.state !== "delivered" ? selectedHistoryCard : null);
  const deliveryViewContent = selectedHistoryEntry?.state === "delivered"
    ? selectedHistoryCard
    : selectedHistoryId == null
      ? currentDeliveryCard ?? currentDeliveryHistoryCard
      : null;
  const jiaobanCanvasViews = buildJiaobanArtifactCanvasViews({
    phase,
    selectedHistoryId,
    activeViewKey: canvasViewKey,
    proposalInteractive,
    proposalContent: proposalViewContent,
    deliveryContent: deliveryViewContent,
    graphContent: phase === "authorize"
      ? previewCanvas ?? <p className="muted small-note">预演图关着——到「怎么跑」打开;也可以直接批,先批就按现场拆。</p>
      : previewCanvas,
    workStateContent: phaseContent,
    governanceContent: latestProposal ? <JiaobanGovernanceView proposal={latestProposal} /> : null,
    howRunContent: latestProposal ? (
      <JiaobanHowRunView
        suggestWorkflow={latestProposal.suggest_workflow === true}
        worksmapSwitchOn={workflowSwitchOn}
        onToggleWorksmapSwitch={setWorkflowSwitchOn}
        orchestrationMode={orchestrationMode}
        onOrchestrationModeChange={setOrchestrationMode}
        supervisorPilotDisabledReason={supervisorPilotDisabledReason}
        classicDisabledReason={classicModeUnavailableReason(projectRoot)}
        disabled={starting}
        sessions={projectSessions}
        sessionChoice={sessionChoice}
        onSessionChoiceChange={setSessionChoice}
        onOpenAgentSession={onOpenAgentSession}
      />
    ) : null,
  });

  return (
    <section className="project-jiaoban project-jiaoban--split" aria-label="交办">
      {renderLayout ? (
        renderLayout({
          phase,
          main: mainContent,
          proposalIndex: proposalIndexContent,
          previewCanvas,
          canvasViews: jiaobanCanvasViews,
          activeCanvasView: canvasViewKey,
          onCanvasViewChange: setCanvasViewKey,
        })
      ) : (
        <>
          {mainContent}
          {proposalIndexContent}
        </>
      )}
    </section>
  );
}

// ============================================================
// Part①·历届方案索引（纯只读呈现·吃 listProjectRunHistory 读模型；五态主区行为 0-diff）
// ============================================================

export { JiaobanProposalIndex, JiaobanHistoryDetail } from "./jiaoban/JiaobanHistory";
export type { HistoryFilter } from "./jiaoban/JiaobanHistory";

// ============================================================
// 五态子组件
// ============================================================

// 阶段3拆巨石:说态+批态族迁 jiaoban/JiaobanAuthorizeStates.tsx(原样零逻辑改动);re-export 保外部 import 面。
// (JiaobanPlanPreviewCanvas 依赖主文件预演工具群,留待下刀与工具一起迁。)
export {
  JiaobanAuthorizeState,
  JiaobanBoundaryReviewSection,
  JiaobanOrchestrationModePicker,
} from "./jiaoban/JiaobanAuthorizeStates";

// 「看原始对话」桥（定稿承诺补做·2026-07-06）：凡能确定对话的地方给一键钻进智能体页。
// 判据诚实三态：sessionChoice=真 thread_id → 「看原始对话」（就是本单那条）；
// 阶段3拆巨石:会话选择件迁 jiaoban/jiaobanSessionParts.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanRawSessionLink,
  JiaobanSessionPicker,
  NEW_SESSION_CHOICE,
} from "./jiaoban/jiaobanSessionParts";

// 阶段3拆巨石:干态两组件+进度工具迁 jiaoban/JiaobanRunningStates.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanRunningState,
  JiaobanSupervisorPilotRunningState,
  humanizeChainProgress,
  isDirectorPlanningPhase,
} from "./jiaoban/JiaobanRunningStates";

// 阶段3拆巨石:交货族迁 jiaoban/JiaobanDoneStates.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanDoneState,
  JiaobanStepReportList,
  JiaobanSupervisorReviewSection,
  buildFactMemoryCandidate,
  countYellowFlags,
  jiaobanDoneTitle,
  stepReportFlag,
} from "./jiaoban/JiaobanDoneStates";
export type { FactMemoryContext, StepReportFlag } from "./jiaoban/JiaobanDoneStates";

// 阶段3拆巨石:卡住/待决定态迁 jiaoban/JiaobanBlockedStates.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanBlockedState,
  JiaobanNeedsReworkDisposal,
  JiaobanWaitingDecisionState,
  classifyBlocked,
} from "./jiaoban/JiaobanBlockedStates";

// ============================================================
// 接缝：允许并开始 = 方案授权人闸那一下 → 走刀1 合流命令 confirm_and_start_authorized_run
// （lib/tauri.ts 的 confirmAndStartAuthorizedRun）。后端一原子命令做完 确认方案 + 边界复核 +
// 授权生效 + 绑现有会话 + 自动推进；返回同形 outcome，组件按 stage 分支不变。人闸不省。
// ============================================================

// 人话工程①(2026-07-20):isAlreadyConfirmedRejection / humanizeProviderUnavailable /
// humanizeAuthorizeError 三函数逐字迁 src/lib/humanize.ts,顶部 import-back,导入面不变。

// ============================================================
// 人话化辅助（把后端结构翻成给用户看的话；主路径不出现节点 id / 链 id）
// ============================================================

// 从 proposed_steps 里抽「目标文件：…」行的文件部分（后端 consultant_agent 会把 target_files 塞在最前）。
