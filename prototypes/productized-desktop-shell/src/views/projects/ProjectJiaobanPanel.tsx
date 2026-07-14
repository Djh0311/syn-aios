import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { Badge } from "../../components/Badge";
import {
  JiaobanBlockedState,
  JiaobanNeedsReworkDisposal,
  JiaobanWaitingDecisionState,
} from "./jiaoban/JiaobanBlockedStates";
import { JiaobanHistoryColumn, JiaobanHistoryDetail, type HistoryFilter } from "./jiaoban/JiaobanHistory";
import {
  JiaobanRawSessionLink,
  JiaobanSessionPicker,
  JiaobanTaskSessionBindingState,
  NEW_SESSION_CHOICE,
} from "./jiaoban/jiaobanSessionParts";
import { formatProposalTime, proposalAgeDays } from "./jiaoban/jiaobanTime";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  applyProjectDirectorFailedAction,
  autoAdvanceAuthorizedRoleLoop,
  confirmAndStartAuthorizedRun,
  confirmProjectDirectorTaskSessionBindings,
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
  runProjectConsultation,
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
  ProjectDirectorTaskSessionBinding,
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
  // M2：外壳提供的「完整工作流」跳转；交办内的五态与命令不碰。
  onOpenWorkflow?: () => void;
  // M2：只把已有历史/五态内容交给 Shell 排版，面板自身仍拥有数据、状态与命令。
  renderLayout?: (content: ProjectJiaobanPanelLayout) => ReactNode;
};

export type JiaobanPhase =
  | "say"
  | "authorize"
  | "binding"
  | "running"
  | "done"
  | "waiting_decision"
  | "blocked";

export function jiaobanStageFromChainOutcome(chain: DirectorChainOutcome | null): string {
  const reason = chain?.stopped_reason?.trim() ?? "";
  if (!reason) return "completed";
  if (reason.startsWith("waiting_decision:")) return "waiting_decision";
  if (reason.startsWith("fail_stop:")) return "failed";
  return "interrupted";
}

export function jiaobanPhaseForOutcome(outcome: AutoAdvanceRoleLoopOutcome): JiaobanPhase {
  if (outcome.task_session_binding_required) return "binding";
  if (outcome.stage === "completed") return "done";
  if (outcome.stage === "waiting_decision") return "waiting_decision";
  return "blocked";
}

export type ProjectJiaobanPanelLayout = {
  phase: JiaobanPhase;
  history: ReactNode;
  main: ReactNode;
  // M1：批前、运行和终态都在 M2 的右侧画布区域展示同一张纵向工序图。
  previewCanvas?: ReactNode;
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

function defaultTaskSessionBindings(
  tasks: ProjectDirectorPlannedTask[],
  topLevelChoice: string | null,
): ProjectDirectorTaskSessionBinding[] {
  const firstExistingSession =
    topLevelChoice && topLevelChoice !== NEW_SESSION_CHOICE ? topLevelChoice : null;
  return tasks.map((task, index) =>
    index === 0 && firstExistingSession
      ? {
          planned_task_id: task.planned_task_id,
          session_choice: "existing",
          session_id: firstExistingSession,
        }
      : { planned_task_id: task.planned_task_id, session_choice: "new" },
  );
}

export type JiaobanPreviewCanvasNode = {
  preview_node_id: string;
  title: string;
  depends_on: string[];
};

export type JiaobanRuntimeNodeState =
  | "pending"
  | "running"
  | "completed"
  | "waiting_decision"
  | "needs_rework"
  | "failed"
  | "skipped"
  | "archived"
  | "unknown";

export type JiaobanRuntimeNodeStateInfo = {
  state: JiaobanRuntimeNodeState;
  detail?: string;
  rawState?: string;
};

const jiaobanRuntimeNodeLabel: Record<JiaobanRuntimeNodeState, string> = {
  pending: "等待",
  running: "正在执行",
  completed: "已完成",
  waiting_decision: "待你决定",
  needs_rework: "需要重做",
  failed: "失败",
  skipped: "没轮到/被跳过",
  archived: "本单已结束",
  unknown: "状态未知",
};

function normalizeJiaobanRuntimeNodeState(
  value: string | null | undefined,
  message: string | null | undefined,
): JiaobanRuntimeNodeStateInfo {
  const rawState = value?.trim() ?? "";
  const detail = message?.trim() ?? "";
  switch (rawState.toLowerCase()) {
    case "pending":
    case "waiting":
      return { state: "pending" };
    case "running":
      return { state: "running" };
    case "completed":
    case "finished":
    case "done":
    case "succeeded":
      return { state: "completed" };
    case "needs_rework":
    case "needs-rework":
      return { state: "needs_rework" };
    case "waiting_decision":
    case "waiting-decision":
      return { state: "waiting_decision" };
    case "failed":
    case "aborted":
    case "stopped":
      return { state: "failed" };
    case "skipped":
      return {
        state: "skipped",
        detail: detail || "skipped；详情看画布",
      };
    case "archived":
      return { state: "archived", detail: detail || undefined };
    default:
      return rawState
        ? {
            state: "unknown",
            rawState,
            detail: `状态未知（${rawState}）；详情看画布`,
          }
        : { state: "pending" };
  }
}

function jiaobanRuntimeNodeStateLabel(state: JiaobanRuntimeNodeStateInfo): string {
  return state.state === "unknown" && state.rawState
    ? `状态未知（${state.rawState}）`
    : jiaobanRuntimeNodeLabel[state.state];
}

// 运行读模型以实时链节点优先；终态若轮询来不及回写，再用本轮 outcome 的步骤兜底。
// 只有单节点简单活才允许把唯一链节点映射给唯一预演节点，避免多节点时猜错归属。
export function jiaobanRuntimeNodeStates(
  nodes: JiaobanPreviewCanvasNode[],
  chainStatus: ProjectWorkflowChainStatus | null,
  chainSteps: DirectorChainStep[] = [],
): Record<string, JiaobanRuntimeNodeStateInfo> {
  return Object.fromEntries(
    nodes.map((node) => {
      const chainNode = chainStatus?.nodes.find((item) => item.node_id === node.preview_node_id);
      const chainStep = chainSteps.find((item) => item.planned_task_id === node.preview_node_id);
      const singleChainNode = nodes.length === 1 ? chainStatus?.nodes[0] : null;
      const singleChainStep = nodes.length === 1 ? chainSteps[0] : null;
      return [
        node.preview_node_id,
        normalizeJiaobanRuntimeNodeState(
          chainNode?.state ?? chainStep?.state ?? singleChainNode?.state ?? singleChainStep?.state,
          chainNode?.message ?? singleChainNode?.message,
        ),
      ];
    }),
  );
}

function previewFallbackNode(proposal: ProjectConsultationProposal): JiaobanPreviewCanvasNode {
  return {
    preview_node_id: `proposal:${proposal.proposal_id}:single`,
    title: proposal.goal_summary || proposal.user_goal || "这项任务",
    depends_on: [],
  };
}

function previewCanvasNodesFor(
  proposal: ProjectConsultationProposal | null,
  previewTasks: ProjectDirectorPlannedTask[] | null,
): JiaobanPreviewCanvasNode[] {
  if (!proposal) return [];
  if (previewTasks?.length) {
    return previewTasks.map((task) => ({
      preview_node_id: task.planned_task_id,
      title: task.title,
      depends_on: task.depends_on,
    }));
  }
  return [previewFallbackNode(proposal)];
}

function previewNodeBinding(
  previewNodeId: string,
  sessionChoice: string | null,
): ProjectDirectorPreviewNodeSessionBinding {
  return sessionChoice && sessionChoice !== NEW_SESSION_CHOICE
    ? { preview_node_id: previewNodeId, session_choice: "existing", session_id: sessionChoice }
    : { preview_node_id: previewNodeId, session_choice: "new" };
}

function runCanvasBindingsFor(
  nodes: JiaobanPreviewCanvasNode[],
  previewBindings: ProjectDirectorPreviewNodeSessionBinding[],
  taskBindings: ProjectDirectorTaskSessionBinding[],
): ProjectDirectorPreviewNodeSessionBinding[] {
  return nodes.map((node) => {
    const taskBinding = taskBindings.find((binding) => binding.planned_task_id === node.preview_node_id);
    if (taskBinding) {
      return taskBinding.session_choice === "existing"
        ? {
            preview_node_id: node.preview_node_id,
            session_choice: "existing",
            session_id: taskBinding.session_id,
          }
        : { preview_node_id: node.preview_node_id, session_choice: "new" };
    }
    return (
      previewBindings.find((binding) => binding.preview_node_id === node.preview_node_id) ??
      previewNodeBinding(node.preview_node_id, NEW_SESSION_CHOICE)
    );
  });
}

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
  // 开工前任务→会话映射：拆任务后才有，切 tab 回来也仍留在绑定面板。
  taskSessionBindings: ProjectDirectorTaskSessionBinding[];
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
    taskSessionBindings: [],
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

// 预拆偶发早退（flaky·后端已自动重试一次仍可能空）→ 人话，优雅降级：不影响批。
function humanizePreviewError(error: unknown): string {
  const raw = error instanceof Error ? error.message : String(error ?? "");
  if (/找不到方案|proposal/i.test(raw)) return "这份方案暂时读不到，右侧预演画布的工序图没画出来（可重试）。";
  return "右侧预演画布的工序图没画出来（可重试）；不影响你批。";
}

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
  onOpenWorkflow,
  renderLayout,
}: ProjectJiaobanPanelProps) {
  const projectWorkflow =
    workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;

  // 本项目最新一份方案（授权卡的数据源）。runProjectConsultation 出方案后 App 会 reload proposal store，
  // 这里随 prop 变化自动拿到新方案 → 从「说」跳到「批」。
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
  const [goal, setGoal] = useState("");
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
  const [amendment, setAmendment] = useState("");
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
  const [taskSessionBindings, setTaskSessionBindings] = useState<ProjectDirectorTaskSessionBinding[]>(
    cached?.taskSessionBindings ?? [],
  );
  const [taskSessionBindingError, setTaskSessionBindingError] = useState<string | null>(null);
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
  // fix8：出方案（说/改要求）直调期间的 loading/失败态 + 防重入。失败人话上脸，绝不静默、目标不清空。
  const [consultLoading, setConsultLoading] = useState(false);
  const [consultError, setConsultError] = useState<string | null>(null);
  const consultingRef = useRef(false);

  // Part①·工作历史（左栏数据·倒序读模型·不轮询·挂载/交货/重拆各拉一次）。失败静默：历史是增益不是闸。
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
    bindings = runCanvasBindingsFor(nodes, previewBindingsForCanvas, taskSessionBindings),
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
    setTaskSessionBindings([]);
    setTaskSessionBindingError(null);
    setAmendment("");
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

  // 干活期间轮询进度（复用现成只读命令）。fix6：改成 phase==running 就轮——
  // 点允许/接着跑那刻（相位先行）即开轮，执行期步骤逐格亮；重挂载后 phase 从缓存恢复=running→照轮→371 兜底翻脸=自愈。
  // （旧实现守最终 outcome.stage，但合流/接着跑是同步跑到底才返回、跑中 outcome 恒 null，重挂载新实例 outcome 也 null → 整个执行期一格不轮、卡「正在干」。）
  useEffect(() => {
    // 用 manualPhase（phase 常量定义在本 effect 之后、直接用会 TDZ；running 只可能来自 manualPhase，二者等价）。
    if (manualPhase !== "running" || !projectWorkflow) return;
    const { project_root: projectRoot, workflow_id: workflowId } = projectWorkflow;
    let active = true;
    const poll = async () => {
      try {
        const status = await getProjectWorkflowChainStatus(projectRoot, workflowId);
        if (active && status) setChainStatus(status);
      } catch {
        // 轮询失败不致命——进度暂缺不影响永不冻。
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [manualPhase, projectWorkflow]);

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

  // fix8：说 → 出方案 = 面板直调 runProjectConsultation（去掉那层通用确认弹层：咨询只读·决策 2026-06-25 豁免·
  // 人闸=[允许并开始]那一下不动）。自管 loading/失败态，失败人话上脸 + 目标不清空；防重入。成功后刷店→自动进批脸。
  async function runConsultation(goal: string) {
    // 永不冻：projectWorkflow 缺失不是用户的错，也绝不许无声——说清楚、可行动。
    if (!projectWorkflow) {
      setConsultError(
        `这个项目的工作流数据没加载出来（快照里 ${workflowState ? `有 ${workflowState.project_workflows.length} 条工作流、但没有匹配 ${projectRoot} 的` : "workflowState 为空——快照读取失败"}）。重开项目试试；还不行把这行话发给主导线。`,
      );
      return;
    }
    if (!goal.trim() || consultingRef.current) return;
    consultingRef.current = true;
    setConsultLoading(true);
    setConsultError(null);
    try {
      await runProjectConsultation({
        project_root: projectRoot,
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        goal,
        actor_id: "user",
      });
      await onProposalStoreRefresh?.(); // 刷方案店 → latestProposal 更新 → phase 推导自动进批脸
    } catch (e) {
      setConsultError(humanizeConsultError(e)); // 失败上脸（供给类专句/后端原话），绝不静默
    } finally {
      consultingRef.current = false;
      setConsultLoading(false);
    }
  }

  function submitGoal(text: string) {
    void runConsultation(text);
  }

  // 按我说的改：原目标 + 这句意见拼成新目标，重出方案（同一直调路径）。
  function submitAmendment() {
    if (!amendment.trim() || !latestProposal) return;
    void runConsultation(`${latestProposal.user_goal}\n\n补充意见：${amendment.trim()}`);
  }

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
  useEffect(() => {
    const proposalId = latestProposal?.proposal_id;
    if (!proposalId || !shouldPreviewWorksmap) return;
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
    // 顶层选择只决定绑定面板首项预填；此时还没有创建任何会话。
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
    setTaskSessionBindings([]);
    setTaskSessionBindingError(null);
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
      const bindings = outcome.task_session_binding_required
        ? outcome.task_session_bindings?.length
          ? outcome.task_session_bindings
          : defaultTaskSessionBindings(outcome.planned_tasks ?? [], sessionChoice)
        : [];
      const nextPhase = jiaobanPhaseForOutcome(outcome);
      setOutcome(outcome);
      setManualPhase(nextPhase);
      setTaskSessionBindings(bindings);
      setTaskSessionBindingError(outcome.task_session_binding_error ?? null);
      writeJiaobanRunCache(projectRoot, {
        manualPhase: nextPhase,
        outcome,
        startError: null,
        taskSessionBindings: bindings,
      });
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

  function updateTaskSessionBinding(plannedTaskId: string, sessionChoice: string | null) {
    const nextChoice =
      sessionChoice && sessionChoice !== NEW_SESSION_CHOICE
        ? { planned_task_id: plannedTaskId, session_choice: "existing" as const, session_id: sessionChoice }
        : { planned_task_id: plannedTaskId, session_choice: "new" as const };
    setTaskSessionBindings((current) => {
      const next = current.map((binding) =>
        binding.planned_task_id === plannedTaskId ? nextChoice : binding,
      );
      writeJiaobanRunCache(projectRoot, { taskSessionBindings: next });
      return next;
    });
  }

  async function startWithTaskSessionBindings() {
    if (!projectWorkflow || !outcome || starting || runningRef.current) return;
    const plannedTasks = outcome.planned_tasks ?? [];
    if (plannedTasks.length === 0) {
      setTaskSessionBindingError("任务清单没有准备好，不能开始跑。请重新出方案或停下。 ");
      return;
    }
    rememberRunCanvas(
      previewCanvasNodes,
      runCanvasBindingsFor(previewCanvasNodes, previewBindingsForCanvas, taskSessionBindings),
    );
    runningRef.current = true;
    const runStartedAt = Date.now();
    const hasNewSession = taskSessionBindings.some((binding) => binding.session_choice === "new");
    setRunStartedAtMs(runStartedAt);
    setRunIsNewSession(hasNewSession);
    setStarting(true);
    setTaskSessionBindingError(null);
    setStartError(null);
    setChainStatus(null);
    setManualPhase("running");
    writeJiaobanRunCache(projectRoot, {
      manualPhase: "running",
      startError: null,
      runStartedAtMs: runStartedAt,
      runIsNewSession: hasNewSession,
      taskSessionBindings,
    });
    try {
      const nextOutcome = await confirmProjectDirectorTaskSessionBindings({
        project_root: projectWorkflow.project_root,
        workflow_id: projectWorkflow.workflow_id,
        planned_tasks: plannedTasks,
        task_session_bindings: taskSessionBindings,
        actor_id: "user",
      });
      const nextPhase = jiaobanPhaseForOutcome(nextOutcome);
      setOutcome(nextOutcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, {
        manualPhase: nextPhase,
        outcome: nextOutcome,
        startError: null,
        taskSessionBindings,
      });
    } catch (error) {
      const humanized = error instanceof Error ? error.message : String(error);
      setTaskSessionBindingError(humanized);
      setManualPhase("binding");
      writeJiaobanRunCache(projectRoot, {
        manualPhase: "binding",
        startError: null,
        taskSessionBindings,
      });
    } finally {
      setStarting(false);
      runningRef.current = false;
    }
  }

  function stopBeforeTaskSessionBinding() {
    setSayHint("这单停在开工前，没有派发任务。 ");
    setTaskSessionBindingError(null);
    setTaskSessionBindings([]);
    setOutcome(null);
    setManualPhase("say");
    clearJiaobanRunCache(projectRoot);
  }

  // 接着跑，不用重批：方案已 user_confirmed（授权还活着）时的重试口——不走合流命令（会被「已确认」拒），
  // 直接调现成 autoAdvanceAuthorizedRoleLoop 从拆任务接着推进。防冻套路同 authorizeAndStart：
  // 先切「正在干」相位再 await、runningRef 防重入、缓存同步；返回走现有 outcome→脸 映射。
  async function continueRun() {
    if (!projectWorkflow || starting || runningRef.current) return;
    if (outcome?.task_session_binding_required) {
      setManualPhase("binding");
      writeJiaobanRunCache(projectRoot, { manualPhase: "binding" });
      return;
    }
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
    if (latestProposal?.user_goal) setGoal(latestProposal.user_goal);
    setManualPhase("say");
    setOutcome(null);
    setStartError(null);
    setContinueHint(false);
    setChainStatus(null);
    setTaskSessionBindings([]);
    setTaskSessionBindingError(null);
    setAmendment("");
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
            <Badge tone="unknown">未开通自动干</Badge>
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
            <Badge tone="warning">缺项目工作流</Badge>
          </div>
          <p className="muted small-note">先在右侧画布建起这个项目的工作流，再回来交办。</p>
        </div>
      </section>
    );
  }

  // 旧方案判定：方案生成不是今天 → 批面出黄条 + 主按钮换「重新说目标」，防再批库存。
  const proposalAge = latestProposal ? proposalAgeDays(latestProposal.created_at_ms) : 0;
  const proposalIsStale = proposalAge >= 1;

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

  const historyContent = (
    <JiaobanHistoryColumn
      entries={history}
      total={historyTotal}
      loading={historyLoading}
      filter={historyFilter}
      onFilterChange={setHistoryFilter}
      selectedId={selectedHistoryId}
      currentProposalId={currentProposalId}
      latestBlockedId={latestBlockedId}
      onSelectEntry={(entry) => setSelectedHistoryId(entry.proposal_id)}
      onBackToCurrent={() => setSelectedHistoryId(null)}
      onNewJiaoban={() => {
        setSelectedHistoryId(null);
        backToSay();
      }}
      onContinueRun={() => void continueRun()}
    />
  );
  const mainContent = (
    <div className="project-jiaoban-main">
      {selectedHistoryEntry ? (
        <JiaobanHistoryDetail entry={selectedHistoryEntry} onBackToCurrent={() => setSelectedHistoryId(null)} />
      ) : (
        <div className="project-jiaoban-col">
          {phase === "say" ? (
            <>
              {memoryCount > 0 ? (
                <p className="jiaoban-recall-hint" role="note" aria-label="记忆召回">
                  出方案会带上 {memoryCount} 条项目记忆
                </p>
              ) : null}
              <JiaobanSayState
                goal={goal}
                onGoalChange={setGoal}
                onSubmit={() => submitGoal(goal)}
                lastStopHint={sayHint}
                loading={consultLoading}
                error={consultError}
                onEditAgain={() => setConsultError(null)}
              />
            </>
          ) : null}

          {phase === "authorize" && latestProposal ? (
            <JiaobanAuthorizeState
              proposal={latestProposal}
              proposalIsStale={proposalIsStale}
              proposalAgeDays={proposalAge}
              sessions={projectSessions}
              sessionChoice={sessionChoice}
              onSessionChoiceChange={setSessionChoice}
              orchestrationMode={orchestrationMode}
              onOrchestrationModeChange={setOrchestrationMode}
              supervisorPilotDisabledReason={supervisorPilotDisabledReason}
              classicDisabledReason={classicModeUnavailableReason(projectRoot)}
              amendment={amendment}
              onAmendmentChange={setAmendment}
              onAmend={submitAmendment}
              onAuthorizeAndStart={() => void authorizeAndStart()}
              onRePlan={backToSay}
              onDecline={backToSay}
              starting={starting}
              consultLoading={consultLoading}
              consultError={consultError}
              worksmapSwitchOn={workflowSwitchOn}
              onToggleWorksmapSwitch={setWorkflowSwitchOn}
              worksmapTasks={previewTasks}
              worksmapLoading={previewLoading}
              worksmapError={previewError}
              boundaryLoading={boundaryLoadingForThisProposal}
              boundaryOutcome={boundaryForThisProposal}
              onBoundaryRetry={() => {
                // [重试]：force 穿透幂等重跑本方案的边界意见。
                if (latestProposal) void requestBoundaryReview(latestProposal.proposal_id, true);
              }}
              onOpenAgentSession={onOpenAgentSession}
            />
          ) : null}

          {phase === "binding" ? (
            <JiaobanTaskSessionBindingState
              tasks={outcome?.planned_tasks ?? []}
              sessions={projectSessions}
              bindings={taskSessionBindings}
              error={taskSessionBindingError}
              starting={starting}
              onBindingChange={updateTaskSessionBinding}
              onStart={() => void startWithTaskSessionBindings()}
              onReplan={backToSay}
              onStop={stopBeforeTaskSessionBinding}
            />
          ) : null}

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

          {phase === "done" ? (
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
              supervisorLoading={supervisorLoading}
              supervisorOutcome={supervisorReview?.outcome ?? null}
              onSupervisorRetry={() => {
                // [重试]/[重新复核]：force 穿透幂等重跑。键优先取已有结果的轮键，兜底本轮链 started_at。
                const key = supervisorReview?.key ?? thisRoundChainStatus?.started_at ?? null;
                if (key) void requestSupervisorReview(key, true);
              }}
              onSupervisorReplan={backToSay}
            />
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
            />
          ) : null}
        </div>
      )}
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
    ((phase === "authorize" || phase === "binding") && Boolean(latestProposal)) ||
    (runtimeCanvasPhase && canvasNodes.length > 0);
  const previewCanvas = showPlanCanvas ? (
    <JiaobanPlanPreviewCanvas
      nodes={canvasNodes}
      bindings={canvasBindings}
      sessions={projectSessions}
      waitingForPreview={!runtimeCanvasPhase && workflowSwitchOn && previewTasks === null && !previewError}
      previewError={runtimeCanvasPhase ? null : previewError}
      previewWarnings={runtimeCanvasPhase ? [] : previewWarnings}
      readOnly={runtimeCanvasPhase}
      runtimeNodeStates={runtimeNodeStates}
      onBindingChange={updatePreviewSessionBinding}
      onRetryPreview={retryPreview}
      onOpenAgentSession={onOpenAgentSession}
    />
  ) : null;

  return (
    <section className="project-jiaoban project-jiaoban--split" aria-label="交办">
      {renderLayout ? renderLayout({ phase, history: historyContent, main: mainContent, previewCanvas }) : <>{historyContent}{mainContent}</>}
    </section>
  );
}

// ============================================================
// Part①·工作历史左栏（纯只读呈现·吃 listProjectRunHistory 读模型；五态主区行为 0-diff）
// ============================================================

// 阶段3拆巨石:工作历史(回顾面)迁 jiaoban/JiaobanHistory.tsx(原样零逻辑改动);此处 re-export 保外部 import 面不变。
export { JiaobanHistoryColumn, JiaobanHistoryDetail } from "./jiaoban/JiaobanHistory";
export type { HistoryFilter } from "./jiaoban/JiaobanHistory";

// ============================================================
// 五态子组件
// ============================================================

// 1. 说
export function JiaobanSayState({
  goal,
  onGoalChange,
  onSubmit,
  lastStopHint,
  loading,
  error,
  onEditAgain,
}: {
  goal: string;
  onGoalChange: (value: string) => void;
  onSubmit: () => void;
  lastStopHint: string | null;
  loading: boolean;
  error: string | null;
  onEditAgain: () => void;
}) {
  return (
    <div className="project-canvas-detail-card jiaoban-say" aria-label="想让 AI 干点啥">
      <div className="panel-heading">
        <div>
          <h3>想让 AI 干点啥？</h3>
        </div>
      </div>
      {lastStopHint ? (
        <div className="jiaoban-say-hint" role="note" aria-label="上次停在哪">
          上次停在：{lastStopHint}——目标已带回来，改一改再出一版新方案。
        </div>
      ) : null}
      {/* fix8：出方案失败上脸——人话（供给类专句/后端原话）+ 目标不清空，绝不静默死。 */}
      {error ? (
        <div className="jiaoban-consult-error" role="alert" aria-label="出方案没成">
          <span aria-hidden="true">⚠</span> {error}
        </div>
      ) : null}
      <label className="proposal-decision-field">
        <span>说一句话，AI 会读你的项目、想个方案给你审。</span>
        <textarea
          value={goal}
          onChange={(event) => onGoalChange(event.target.value)}
          placeholder="例：给这小游戏加个计分板——吃到东西 +1、显示在右上角。"
          rows={4}
          disabled={loading}
        />
      </label>
      <div className="workflow-state-actions">
        {error && !loading ? (
          // 失败态：绝不零按钮——[重试]（重发原目标·目标还在框里）+ [改要求]（回编辑态改了再出）。
          <>
            <button className="primary-button" type="button" disabled={!goal.trim()} onClick={onSubmit}>
              重试
            </button>
            <button className="secondary-button" type="button" onClick={onEditAgain}>
              改要求
            </button>
          </>
        ) : (
          <button className="primary-button" type="button" disabled={loading || !goal.trim()} onClick={onSubmit}>
            {loading ? "AI 正在读项目、想方案…（约 1–2 分钟）" : "出方案"}
          </button>
        )}
      </div>
    </div>
  );
}

// 2. 批（授权卡·定稿字段）
// export 供离线 DOM 断言（fix9 诚实脸两态；renderToStaticMarkup 渲染·不平铺调用）。
export function JiaobanAuthorizeState({
  proposal,
  proposalIsStale,
  proposalAgeDays,
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  orchestrationMode = "classic",
  onOrchestrationModeChange = () => {},
  supervisorPilotDisabledReason = null,
  classicDisabledReason = null,
  amendment,
  onAmendmentChange,
  onAmend,
  onAuthorizeAndStart,
  onRePlan,
  onDecline,
  starting,
  consultLoading,
  consultError,
  worksmapSwitchOn,
  onToggleWorksmapSwitch,
  worksmapTasks,
  worksmapLoading,
  worksmapError,
  boundaryLoading,
  boundaryOutcome,
  onBoundaryRetry,
  onOpenAgentSession,
}: {
  proposal: ProjectConsultationProposal;
  proposalIsStale: boolean;
  proposalAgeDays: number;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  orchestrationMode?: JiaobanOrchestrationMode;
  onOrchestrationModeChange?: (mode: JiaobanOrchestrationMode) => void;
  supervisorPilotDisabledReason?: string | null;
  classicDisabledReason?: string | null;
  amendment: string;
  onAmendmentChange: (value: string) => void;
  onAmend: () => void;
  onAuthorizeAndStart: () => void;
  onRePlan: () => void;
  onDecline: () => void;
  starting: boolean;
  consultLoading: boolean;
  consultError: string | null;
  worksmapSwitchOn: boolean;
  onToggleWorksmapSwitch: (value: boolean) => void;
  worksmapTasks: ProjectDirectorPlannedTask[] | null;
  worksmapLoading: boolean;
  worksmapError: string | null;
  // B2·全局主管批前边界意见（advisory·意见不是闸·async·缺席不挡批）。
  boundaryLoading: boolean;
  boundaryOutcome: GlobalSupervisorBoundaryReviewOutcome | null;
  onBoundaryRetry: () => void;
  // 「看原始对话」桥：批卡收纳行入口（**必填**·透传给 picker）。批卡是任务点名的主入口，
  // 设必填让上游漏传直接 tsc 报错——防「组件接了、上游忘喂、入口静默不显」的假绿（审查线逮到过）。
  onOpenAgentSession: (threadId: string) => void;
}) {
  const targetFiles = extractTargetFiles(proposal.proposed_steps);
  const willWrite = proposal.scope_draft.allowed_write_roots.length > 0;
  const workerAcceptance = proposal.worker_acceptance_criteria ?? [];
  const controlCoreAcceptance = proposal.control_core_acceptance_criteria ?? [];
  const supervisorAcceptance = proposal.supervisor_acceptance_criteria ?? [];
  const hasRoleAcceptance =
    workerAcceptance.length > 0 || controlCoreAcceptance.length > 0 || supervisorAcceptance.length > 0;
  // 按钮旁的状态话只说明右侧预演画布，不再暗示卡内另有一张图。
  const worksmapReady = !worksmapLoading && !worksmapError && !!(worksmapTasks && worksmapTasks.length);
  const worksmapNote =
    !worksmapSwitchOn || worksmapError
      ? null
      : worksmapLoading
        ? "右侧预演画布正在绘制工序图…（可先批，先批就按现场拆）"
        : worksmapReady
          ? "✓ 工序图已在右侧预演画布显示"
          : null;

  return (
    <div className="project-canvas-detail-card jiaoban-authorize" aria-label="方案">

      {/* 旧方案不冒充当前：不是今天生成 → 顶部黄条 + 主按钮换「重新说目标」，防再批库存。 */}
      {proposalIsStale ? (
        <div className="jiaoban-stale-banner" role="note" aria-label="旧方案提醒">
          <span aria-hidden="true">⚠</span>
          {/* 同 advice-only 警条：正文包单 span 防 flex 拆柱（此条现在恰好没内联元素才幸免，统一防）。 */}
          <span className="jiaoban-banner-body">
            这是 {proposalAgeDays} 天前的旧方案，项目可能已变——建议重新说一遍。
          </span>
        </div>
      ) : null}

      {/* 写根空是可执行的只读单：仍走同一人闸，只是不授予写入。 */}
      {!willWrite ? (
        <div className="jiaoban-advice-only-banner" role="note" aria-label="只读单提醒">
          <span aria-hidden="true">⚠</span>
          <span className="jiaoban-banner-body">
            这单是只读的——AI 只看不改，交货是结论不是改动
          </span>
        </div>
      ) : null}

      <div className="role-loop-plain jiaoban-plan-body" aria-label="方案要点（人话）">
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">我来做：</span>
          {proposal.goal_summary || proposal.user_goal}
        </p>
        {targetFiles ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">会改的文件：</span>
            {targetFiles}
          </p>
        ) : null}
        {hasRoleAcceptance ? (
          <>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">执行 Agent 要做到：</span>
              {workerAcceptance.join("；") || "未提供"}
            </p>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">Syn 要保证：</span>
              {controlCoreAcceptance.join("；") || "未提供"}
            </p>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">主管要判断：</span>
              {supervisorAcceptance.join("；") || "未提供"}
            </p>
          </>
        ) : proposal.acceptance_criteria.length ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">改完怎么验：</span>
            {proposal.acceptance_criteria.join("；")}
          </p>
        ) : null}
      </div>

      {/* B2·全局主管批前边界意见：方案要点之后、按钮区之前。async 后填·意见没到也可以先批（不拦事）。 */}
      <JiaobanBoundaryReviewSection
        loading={boundaryLoading}
        outcome={boundaryOutcome}
        onRetry={onBoundaryRetry}
      />

      <JiaobanWorksmap
        suggestWorkflow={proposal.suggest_workflow === true}
        switchOn={worksmapSwitchOn}
        onToggleSwitch={onToggleWorksmapSwitch}
      />

      <JiaobanOrchestrationModePicker
        mode={orchestrationMode}
        disabledReason={supervisorPilotDisabledReason}
        classicDisabledReason={classicDisabledReason}
        disabled={starting || consultLoading}
        onChange={onOrchestrationModeChange}
      />

      <JiaobanSessionPicker
        sessions={sessions}
        sessionChoice={sessionChoice}
        onSessionChoiceChange={onSessionChoiceChange}
        onOpenAgentSession={onOpenAgentSession}
        label="给第一个预演节点预填对话"
      />

      {willWrite ? (
        <div className="jiaoban-grant" role="note">
          <span aria-hidden="true">🔓</span> 需要你允许：改这个测试项目
        </div>
      ) : null}

      <label className="proposal-decision-field jiaoban-amend">
        <input
          type="text"
          aria-label="修改方案"
          value={amendment}
          onChange={(event) => onAmendmentChange(event.target.value)}
          placeholder="例：改成暗色、分数存下来…"
          disabled={consultLoading}
        />
      </label>

      {/* fix8：改要求出新方案期间/失败也上脸——loading 提示 + 失败人话，绝不静默。 */}
      {consultLoading ? (
        <p className="muted small-note">正在出新方案…（约 1–2 分钟）</p>
      ) : consultError ? (
        <div className="jiaoban-consult-error" role="alert" aria-label="出方案没成">
          <span aria-hidden="true">⚠</span> {consultError}
        </div>
      ) : null}

      {worksmapNote ? (
        <p className={`jiaoban-worksmap-cta ${worksmapReady ? "ready" : ""}`}>{worksmapNote}</p>
      ) : null}

      <div className="workflow-state-actions">
        {!willWrite ? (
          // 只读单的唯一开工门仍是 [允许并开始]；重新出方案保留为次操作。
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "允许并开始（只读）"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onRePlan}
            >
              {consultLoading ? "正在出新方案…" : "重新出方案（要动手）"}
            </button>
          </>
        ) : proposalIsStale ? (
          // 旧方案：主按钮 = 重新说目标；[允许并开始] 降为次按钮（防再批库存），但仍可手动点。
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onRePlan}
            >
              {consultLoading ? "正在出新方案…" : "重新说目标出新方案"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "仍要允许并开始（旧方案）"}
            </button>
          </>
        ) : (
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "允许并开始"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading || !amendment.trim()}
              onClick={onAmend}
            >
              {consultLoading ? "正在出新方案…" : "按我说的改"}
            </button>
          </>
        )}
        <button className="secondary-button" type="button" disabled={starting} onClick={onDecline}>
          先不做
        </button>
      </div>
    </div>
  );
}

// Station 2：按单选择入口。默认经典；试点不可用时仍展示原因，不能靠前端状态偷开。
export function JiaobanOrchestrationModePicker({
  mode,
  disabledReason,
  classicDisabledReason = null,
  disabled,
  onChange,
}: {
  mode: JiaobanOrchestrationMode;
  disabledReason: string | null;
  classicDisabledReason?: string | null;
  disabled: boolean;
  onChange: (mode: JiaobanOrchestrationMode) => void;
}) {
  const pilotDisabled = disabled || disabledReason !== null;
  const classicDisabled = disabled || classicDisabledReason !== null;
  return (
    <fieldset className="proposal-decision-field" aria-label="执行模式">
      <legend className="jiaoban-field-label">执行模式</legend>
      <label className={classicDisabled ? "muted" : undefined}>
        <input
          type="radio"
          name="jiaoban-orchestration-mode"
          checked={mode === "classic"}
          disabled={classicDisabled}
          onChange={() => onChange("classic")}
        />
        经典状态机（默认）
      </label>
      <label className={pilotDisabled ? "muted" : undefined}>
        <input
          type="radio"
          name="jiaoban-orchestration-mode"
          checked={mode === "supervisor_pilot"}
          disabled={pilotDisabled}
          onChange={() => onChange("supervisor_pilot")}
        />
        主管编排（试点）
      </label>
      {disabledReason ? <p className="muted small-note">{disabledReason}</p> : null}
      {classicDisabledReason ? <p className="muted small-note">{classicDisabledReason}</p> : null}
    </fieldset>
  );
}

// B2·全局主管批前边界意见区（纯展示·无 hooks·export 供离线 DOM 断言直接调）。
// 词表死线：「全局主管意见/边界意见」——**不是审批**（意见不是闸·不拦批·按钮区行为一概不变）。
// 四态：loading / 意见到（verdict 人话行 + points 列表·mismatch 告警调）/ 不可用（人话 + [重试]）/
// 没触发（outcome null 且不 loading → 零渲染，如 stale 方案/无方案/意见缺席）。
export function JiaobanBoundaryReviewSection({
  loading,
  outcome,
  onRetry,
}: {
  loading: boolean;
  outcome: GlobalSupervisorBoundaryReviewOutcome | null;
  onRetry: () => void;
}) {
  if (loading) {
    return (
      <div className="jiaoban-boundary" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
        <p className="muted small-note">
          <span className="jiaoban-spinner" aria-hidden="true" /> 全局主管正在看边界…（意见没到也可以先批——它不拦事）
        </p>
      </div>
    );
  }
  if (!outcome) return null;
  const review = outcome.status === "ready" ? (outcome.review ?? null) : null;
  if (!review) {
    // 不可用：人话原因 + [重试]（force）——意见缺席不挡批，但绝不零出路。
    const reason = outcome.reason?.trim() || outcome.review?.unavailable_reason?.trim() || "原因不明";
    return (
      <div className="jiaoban-boundary" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
        <p className="muted small-note">边界意见暂时不可用：{reason}（不影响你批）</p>
        <button className="secondary-button" type="button" onClick={onRetry}>
          重试
        </button>
      </div>
    );
  }
  // verdict 人话行（词表：意见，不是审批）。mismatch/caution 告警调、looks_ok 一行绿。
  const verdictLine =
    review.verdict === "looks_ok"
      ? "✓ 全局主管看过：范围和你的目标对得上"
      : review.verdict === "mismatch"
        ? "⚠ 全局主管意见：这方案好像对不上你的目标"
        : "⚠ 全局主管提醒：有几处要留意一下";
  const verdictTone = review.verdict === "looks_ok" ? "jiaoban-boundary-ok" : "jiaoban-boundary-flag";
  return (
    <div className="jiaoban-boundary" aria-label="全局主管意见">
      <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
      <p className={`jiaoban-boundary-verdict ${verdictTone}`}>{verdictLine}</p>
      {review.summary.trim() ? <p className="jiaoban-boundary-summary">{review.summary}</p> : null}
      {review.points.length > 0 ? (
        <ul className="jiaoban-boundary-points" aria-label="边界意见要点">
          {review.points.map((point, index) => (
            <li key={index}>{point}</li>
          ))}
        </ul>
      ) : null}
      <p className="muted small-note jiaoban-boundary-foot">这只是提醒，批不批还是你说了算。</p>
      <button className="jiaoban-linklike jiaoban-boundary-rerun" type="button" onClick={onRetry}>
        重新看一遍
      </button>
    </div>
  );
}

// M1·合一页右侧纵向工序图。批前节点可选对话；运行/终态复用同一张图，只读显示真实链状态。
export function JiaobanPlanPreviewCanvas({
  nodes,
  bindings,
  sessions,
  waitingForPreview,
  previewError,
  previewWarnings,
  readOnly = false,
  runtimeNodeStates = null,
  onBindingChange,
  onRetryPreview,
  onOpenAgentSession,
}: {
  nodes: JiaobanPreviewCanvasNode[];
  bindings: ProjectDirectorPreviewNodeSessionBinding[];
  sessions: SessionRecord[];
  waitingForPreview: boolean;
  previewError: string | null;
  previewWarnings: string[];
  readOnly?: boolean;
  runtimeNodeStates?: Record<string, JiaobanRuntimeNodeStateInfo> | null;
  onBindingChange: (previewNodeId: string, value: string | null) => void;
  onRetryPreview: () => void;
  onOpenAgentSession: (threadId: string) => void;
}) {
  if (waitingForPreview) {
    return (
      <div className="jiaoban-plan-preview-state" role="status" aria-label="预演工序图绘制中">
        <strong>正在绘制预演工序图…</strong>
        <span>大约 1–7 分钟；你可以照常允许并开始，未完成时会按现场拆分。</span>
      </div>
    );
  }
  if (previewError) {
    return (
      <div className="jiaoban-plan-preview-state is-error" role="note" aria-label="预演工序图暂不可用">
        <strong>预演工序图暂不可用</strong>
        <span>{previewError}。不影响你批准这份方案。</span>
        <button className="secondary-button" type="button" onClick={onRetryPreview}>
          重试画图
        </button>
      </div>
    );
  }
  return (
    <section className="jiaoban-plan-preview" aria-label={readOnly ? "运行工序图" : "方案预演工序图"}>
      <div className="jiaoban-plan-preview-graph" role="list" aria-label={readOnly ? "运行任务与依赖" : "预演任务与依赖"}>
        {nodes.map((node, index) => {
          const binding =
            bindings.find((item) => item.preview_node_id === node.preview_node_id) ??
            previewNodeBinding(node.preview_node_id, NEW_SESSION_CHOICE);
          const session =
            binding.session_choice === "existing"
              ? sessions.find((item) => item.thread_id === binding.session_id) ?? null
              : null;
          const sessionLabel =
            binding.session_choice === "existing"
              ? `接现有 · ${session?.title || binding.session_id || "已选对话"}`
              : "新会话";
          const dependencies = node.depends_on.filter(Boolean);
          const runtimeNodeState = readOnly
            ? runtimeNodeStates?.[node.preview_node_id] ?? { state: "pending" }
            : null;
          const nodeLabel = runtimeNodeState ? jiaobanRuntimeNodeStateLabel(runtimeNodeState) : "预演";
          return (
            <div className="jiaoban-plan-preview-node-wrap" key={node.preview_node_id} role="listitem">
              {index > 0 ? (
                <span className="jiaoban-plan-preview-edge" aria-label={`步骤 ${index + 1} 的前置关系`}>
                  ↓
                </span>
              ) : null}
              <details
                className={`jiaoban-plan-preview-node${runtimeNodeState ? ` is-runtime-node is-${runtimeNodeState.state}` : ""}`}
              >
                <summary className="project-canvas-static-node task preflight">
                  <span>任务 · {nodeLabel}</span>
                  <strong>{node.title}</strong>
                  {dependencies.length ? <em>依赖：{dependencies.join("、")}</em> : <em>可从这里开始</em>}
                  {runtimeNodeState?.detail ? <em>{runtimeNodeState.detail}</em> : null}
                  <small className={binding.session_choice === "existing" ? "is-existing" : ""}>{sessionLabel}</small>
                </summary>
                {readOnly ? (
                  <div className="jiaoban-plan-preview-picker jiaoban-plan-preview-picker--readonly">
                    <p className="muted small-note">
                      {binding.session_choice === "existing"
                        ? `已绑定：${session?.title || binding.session_id || "现有对话"}`
                        : "这一步使用新会话。"}
                    </p>
                    <JiaobanRawSessionLink
                      sessionChoice={binding.session_choice === "existing" ? binding.session_id ?? null : NEW_SESSION_CHOICE}
                      onOpenAgentSession={onOpenAgentSession}
                    />
                  </div>
                ) : (
                  <div className="jiaoban-plan-preview-picker">
                    <JiaobanSessionPicker
                      sessions={sessions}
                      sessionChoice={binding.session_choice === "existing" ? binding.session_id ?? null : NEW_SESSION_CHOICE}
                      onSessionChoiceChange={(value) => onBindingChange(node.preview_node_id, value)}
                      onOpenAgentSession={onOpenAgentSession}
                      label={`给「${node.title}」选择对话`}
                      inputName={`jiaoban-preview-session-${index}`}
                    />
                  </div>
                )}
              </details>
            </div>
          );
        })}
      </div>
      {previewWarnings.length ? (
        <ul className="jiaoban-plan-preview-warnings" aria-label="预演提醒">
          {previewWarnings.map((warning, index) => (
            <li key={index}>{warning}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}

// 刀2「批前看图」的开关仍留在授权卡：它控制预拆，不再在卡内重复画图。
function JiaobanWorksmap({
  suggestWorkflow,
  switchOn,
  onToggleSwitch,
}: {
  suggestWorkflow: boolean;
  switchOn: boolean;
  onToggleSwitch: (value: boolean) => void;
}) {
  return (
    <div className="jiaoban-worksmap" aria-label="工作流预演开关">
      <label className="jiaoban-worksmap-toggle">
        <input type="checkbox" checked={switchOn} onChange={(event) => onToggleSwitch(event.target.checked)} />
        <span>按工作流来（在右侧预演画布看工序图）</span>
        {suggestWorkflow ? <span className="jiaoban-worksmap-suggest">AI 建议：这活值得先预演</span> : null}
      </label>
    </div>
  );
}

// 「看原始对话」桥（定稿承诺补做·2026-07-06）：凡能确定对话的地方给一键钻进智能体页。
// 判据诚实三态：sessionChoice=真 thread_id → 「看原始对话」（就是本单那条）；
// 阶段3拆巨石:会话选择件迁 jiaoban/jiaobanSessionParts.tsx(原样零逻辑改动);re-export 保外部 import 面。
export {
  JiaobanRawSessionLink,
  JiaobanSessionPicker,
  JiaobanTaskSessionBindingState,
  NEW_SESSION_CHOICE,
} from "./jiaoban/jiaobanSessionParts";

// 3. 干（人话进度）
export function JiaobanRunningState({
  chainStatus,
  directorPlanningElapsedMinutes,
  isNewSession,
  onStop,
  sessionChoice,
  latestSessionThreadId,
  onOpenAgentSession,
}: {
  chainStatus: ProjectWorkflowChainStatus | null;
  directorPlanningElapsedMinutes: number;
  isNewSession: boolean;
  onStop: () => void;
  // 「看原始对话」桥：existing 单跑中→看原始对话（能看实时进度）；哨兵单→latestSession 兜底看最近对话。
  sessionChoice: string | null;
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
}) {
  const isDirectorPlanning = !chainStatus || chainStatus.nodes.length === 0;
  const progress = humanizeChainProgress(chainStatus, directorPlanningElapsedMinutes);
  return (
    <div className="project-canvas-detail-card jiaoban-running" aria-label="正在干">
      <div className="panel-heading">
        <div>
          <h3>正在干…</h3>
        </div>
        <Badge tone="candidate">进行中</Badge>
      </div>
      <div className="role-loop-plain" aria-label="进度（人话）">
        <p className="role-loop-plain-lead">
          <span className="jiaoban-spinner" aria-hidden="true" /> {progress}
        </p>
        {isDirectorPlanning && directorPlanningElapsedMinutes >= 2 ? (
          <p className="muted small-note">模型在长考;若超时会自动停下重试,不用干等</p>
        ) : null}
        {isNewSession ? (
          <p className="muted small-note">正在为需要新会话的任务逐一新建会话（约 1 分钟）…</p>
        ) : null}
      </div>
      <JiaobanRawSessionLink
        sessionChoice={sessionChoice}
        latestSessionThreadId={latestSessionThreadId}
        onOpenAgentSession={onOpenAgentSession}
      />
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onStop}>
          停下
        </button>
      </div>
      <p className="muted small-note">想看每一步的过程，看右侧画布。</p>
    </div>
  );
}

// Station 2：主管试点只消费已存在的 sidecar 审计投影；不把任何事件写回链态。
export function JiaobanSupervisorPilotRunningState({
  runId,
  readModel,
  ledgerError,
}: {
  runId: string;
  readModel: SupervisorPilotReadModel | null;
  ledgerError: string | null;
}) {
  const status = readModel?.launch_status ?? "starting";
  const isActive = status === "starting" || status === "running";
  const waitingReason = readModel?.termination_reason.trim();
  const statusText =
    status === "running"
      ? "主管正在编排"
      : status === "waiting_user"
        ? waitingReason || "主管等待用户决定"
      : status === "exited"
        ? "主管进程已结束，业务状态以权威回程和验收账本为准"
        : status === "failed"
          ? "主管会话异常结束"
          : "主管正在启动";
  return (
    <div
      className="project-canvas-detail-card jiaoban-running"
      aria-label={isActive ? "主管进行中" : status === "waiting_user" ? "主管等待用户决定" : "主管已结束"}
    >
      <div className="panel-heading">
        <div>
          <h3>{isActive ? "主管进行中…" : status === "waiting_user" ? "主管等待用户决定" : "主管进程已结束"}</h3>
        </div>
        <Badge tone={status === "failed" || status === "waiting_user" ? "warning" : "candidate"}>{statusText}</Badge>
      </div>
      <div className="role-loop-plain" aria-label="主管账本事件流">
        <p className="role-loop-plain-lead">
          {isActive ? <span className="jiaoban-spinner" aria-hidden="true" /> : null} {statusText}
        </p>
        <p className="muted small-note">本单主管运行编号：{runId}</p>
        {ledgerError ? <p className="state-warning">主管账本暂时不可读：{ledgerError}</p> : null}
        {readModel?.audit_events.length ? (
          <ul className="jiaoban-boundary-points" aria-label="主管账本事件">
            {readModel.audit_events.map((event) => (
              <li key={event.event_id}>
                {event.tool}：{event.result_summary || event.result_status}
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted small-note">账本事件正在到达…</p>
        )}
      </div>
    </div>
  );
}

// 刀A·口供上脸：单步徽章判定。执行态(failed/skipped)优先于自述；completed 才看 worker 口供。
export type StepReportFlag = {
  kind: "ok" | "yellow" | "fail" | "skip";
  badge: string;
  tone: "green" | "yellow" | "red" | "gray";
};

export function stepReportFlag(step: DirectorChainStep): StepReportFlag {
  if (step.state === "failed") {
    return { kind: "fail", badge: "失败", tone: "red" };
  }
  if (step.state === "skipped") {
    return { kind: "skip", badge: "跳过", tone: "gray" };
  }
  // 到这里视为已完成——看 worker 自述，自报没干完的不许装全绿。
  if (step.report_warning) {
    return { kind: "yellow", badge: step.report_warning, tone: "yellow" };
  }
  if (!step.report_summary) {
    return { kind: "yellow", badge: "没交汇报", tone: "yellow" };
  }
  const status = step.report_status ?? "";
  if (status === "done") {
    return { kind: "ok", badge: "自述：做好了", tone: "green" };
  }
  if (status === "partial") {
    return { kind: "yellow", badge: "自述：没干完", tone: "yellow" };
  }
  if (status === "failed") {
    return { kind: "yellow", badge: "自述：失败", tone: "yellow" };
  }
  return { kind: "yellow", badge: "自述：状态不明", tone: "yellow" };
}

// 黄牌数：只数「完成但自述有问题」的 yellow（failed/skipped 是执行态红/灰，不计入黄牌 N）。
export function countYellowFlags(steps: DirectorChainStep[]): number {
  return steps.filter((step) => stepReportFlag(step).kind === "yellow").length;
}

// 标题联动：有黄牌 → 「✓ 做好了（有 N 项要看一眼）」；全绿 → 「✓ 做好了」。自报没干完的不许装全绿。
export function jiaobanDoneTitle(steps: DirectorChainStep[]): string {
  const yellow = countYellowFlags(steps);
  return yellow > 0 ? `✓ 做好了（有 ${yellow} 项要看一眼）` : "✓ 做好了";
}

// 交货脸/失败脸每任务一行：任务标题 + 自述一句 + 人话徽章。无 steps → 不渲染（零回退）。
// 刀B·事实确认上下文（构造记忆候选需要的项目锚）。
export type FactMemoryContext = {
  projectRoot: string;
  projectId?: string | null;
  workflowId?: string | null;
};

// 刀B·事实确认：把「绿✓且有自述」的任务行构造成记忆**候选**入参（候选≠正式·待治理转正·治理一字不动）。
// claim=自述、body=标题+确认语；memory_type/risk/sensitive 取核查后合法最保守档（workflow_summary/low/project）。
export function buildFactMemoryCandidate(
  step: DirectorChainStep,
  ctx: FactMemoryContext,
): CreateMemoryCandidateInput {
  const nowIso = new Date().toISOString();
  const claim = (step.report_summary ?? step.title).trim();
  const scopeId = ctx.projectId ? `scope:${ctx.projectId}` : `scope:project:${ctx.projectRoot}`;
  return {
    project_root: ctx.projectRoot,
    project_id: ctx.projectId ?? null,
    workflow_id: ctx.workflowId ?? null,
    scope: {
      scope_id: scopeId,
      scope_type: "project",
      user_id: null,
      project_id: ctx.projectId ?? null,
      workflow_id: null,
      session_id: null,
      role_ids: [],
      document_refs: [],
      permission_policy_ref: null,
      model_export_policy: "local_only",
      valid_from: nowIso,
      valid_until: null,
    },
    memory_type: "workflow_summary",
    claim,
    body: `任务「${step.title}」经用户在交货脸确认属实。自述：${step.report_summary ?? "（无）"}`,
    source_refs: [
      {
        source_ref_id: `worker-report:${step.planned_task_id}`,
        source_type: "workflow_summary",
        source_id: ctx.workflowId ?? null,
        source_title: step.title,
        anchor: step.planned_task_id,
        captured_at: nowIso,
        authority_level: "user_confirmed",
        sensitive_level: "project",
      },
    ],
    generated_by_role: "user",
    generated_from: "explicit_user_confirmation",
    risk_level: "low",
    sensitive_level: "project",
    requires_user_confirmation: true,
    review_reason: "用户在交货脸确认任务属实，沉淀为项目记忆候选（候选≠正式，待治理转正）。",
    expected_store_revision: null,
  };
}

export function JiaobanStepReportList({
  steps,
  onConfirmFact,
  confirmedTaskIds,
}: {
  steps: DirectorChainStep[];
  onConfirmFact?: (step: DirectorChainStep) => void;
  confirmedTaskIds?: ReadonlySet<string>;
}) {
  if (!steps || steps.length === 0) {
    return null;
  }
  return (
    <ul className="jiaoban-step-report" aria-label="每一步的自述">
      {steps.map((step) => {
        const flag = stepReportFlag(step);
        return (
          <li key={step.planned_task_id} className={`jiaoban-step-row tone-${flag.tone}`}>
            <span className="jiaoban-step-title">{step.title}</span>
            {step.report_summary ? (
              <span className="jiaoban-step-say">{step.report_summary}</span>
            ) : null}
            <span className={`jiaoban-step-badge tone-${flag.tone}`}>
              {flag.tone === "yellow" ? "⚠ " : ""}
              {flag.badge}
            </span>
            {flag.kind === "ok" && step.report_summary && onConfirmFact ? (
              confirmedTaskIds?.has(step.planned_task_id) ? (
                <span className="jiaoban-fact-done">已沉淀 ✓</span>
              ) : (
                <button
                  type="button"
                  className="jiaoban-fact-btn"
                  onClick={() => onConfirmFact(step)}
                >
                  属实，沉淀
                </button>
              )
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}

// B1·全局主管复核区（纯展示·无 hooks·export 供离线 DOM 断言直接调）。
// 词表死线：「全局主管意见/复核意见」——**不是审批**（意见不是闸，按钮全走现成用户动作）。
// 四态：loading / 意见到（总判 + 每任务点评 + 建议动作按钮）/ 不可用（人话 + [重试]·绝不零出路）/
// 没起（outcome null 且不 loading → 零渲染，如无本轮链、旧数据）。
export function JiaobanSupervisorReviewSection({
  loading,
  outcome,
  onRetry,
  onReplan,
}: {
  loading: boolean;
  outcome: GlobalSupervisorReviewOutcome | null;
  onRetry: () => void;
  onReplan: () => void;
}) {
  if (loading) {
    return (
      <div className="jiaoban-supervisor" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
        <p className="muted small-note">
          <span className="jiaoban-spinner" aria-hidden="true" /> 全局主管复核中…（约 2-7 分钟，不影响交货）
        </p>
      </div>
    );
  }
  if (!outcome) return null;
  const review = outcome.status === "ready" ? (outcome.review ?? null) : null;
  if (!review) {
    // 不可用：人话原因 + [重试]（force）——复核缺席不挡任何事，但绝不零出路。
    const reason = outcome.reason?.trim() || outcome.review?.unavailable_reason?.trim() || "原因不明";
    return (
      <div className="jiaoban-supervisor" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
        <p className="muted small-note">复核不可用：{reason}</p>
        <button className="secondary-button" type="button" onClick={onRetry}>
          重试复核
        </button>
      </div>
    );
  }
  // 总判一行（词表：意见，不是审批）。
  const overallLine =
    review.overall === "pass"
      ? "✓ 全局主管看过：这轮没发现问题"
      : review.overall === "needs_rework"
        ? "⚠ 全局主管意见：建议打回重拆"
        : "⚠ 全局主管意见：建议你亲自核验";
  const overallTone = review.overall === "pass" ? "jiaoban-supervisor-pass" : "jiaoban-supervisor-flag";
  return (
    <div className="jiaoban-supervisor" aria-label="全局主管意见">
      <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
      <p className={`jiaoban-supervisor-overall ${overallTone}`}>{overallLine}</p>
      {review.summary.trim() ? <p className="jiaoban-supervisor-summary">{review.summary}</p> : null}
      {review.tasks.length > 0 ? (
        <ul className="jiaoban-supervisor-tasks" aria-label="每任务点评">
          {review.tasks.map((task, index) => (
            <li key={index} className={task.verdict === "issue" ? "jiaoban-supervisor-issue" : undefined}>
              {task.verdict === "issue" ? "⚠ " : ""}
              {task.title ? `${task.title}：` : ""}
              {task.comment}
            </li>
          ))}
        </ul>
      ) : null}
      {review.suggested_action === "replan" ? (
        <div className="workflow-state-actions">
          <button className="secondary-button" type="button" onClick={onReplan}>
            按建议打回重拆
          </button>
        </div>
      ) : null}
      {review.suggested_action === "human_verify" ? (
        <p className="jiaoban-supervisor-note">
          建议你亲验：{review.human_note.trim() || "亲自核验这轮结果。"}
        </p>
      ) : null}
      <button className="jiaoban-linklike jiaoban-supervisor-rerun" type="button" onClick={onRetry}>
        重新复核
      </button>
    </div>
  );
}

// 4. 交货
export function JiaobanDoneState({
  outcome,
  chainStatus,
  onContinue,
  needsRework,
  needsReworkActionError,
  needsReworkActionStarting,
  onNeedsReworkContinue,
  onNeedsReworkAction,
  onRequestAction,
  factCtx,
  sessionChoice,
  latestSessionThreadId,
  onOpenAgentSession,
  supervisorLoading,
  supervisorOutcome,
  onSupervisorRetry,
  onSupervisorReplan,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  chainStatus: ProjectWorkflowChainStatus | null;
  onContinue: () => void;
  needsRework: { reason: string; actionsReady: boolean } | null;
  needsReworkActionError: string | null;
  needsReworkActionStarting: boolean;
  onNeedsReworkContinue: () => void;
  onNeedsReworkAction: (action: "change_session" | "rework" | "archive") => void;
  onRequestAction: (action: PendingAction) => void;
  factCtx: FactMemoryContext | null;
  // 「看原始对话」桥：existing 单→看原始对话（就是干这单的那条）；哨兵单→latestSession 兜底看最近对话。
  sessionChoice: string | null;
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
  // B1·全局主管复核区（advisory）：意见 + 建议动作按钮（按钮走现成用户动作·意见不是闸）。
  supervisorLoading: boolean;
  supervisorOutcome: GlobalSupervisorReviewOutcome | null;
  onSupervisorRetry: () => void;
  onSupervisorReplan: () => void;
}) {
  const chain = outcome?.chain_outcome ?? null;
  const isCompleted =
    outcome?.stage === "completed" ||
    ["finished", "completed", "done", "succeeded"].includes(chainStatus?.state.trim().toLowerCase() ?? "");
  // 刀B·事实确认本地态（防重复点·经现成 create-memory-candidate PendingAction 走确认弹层）。
  const [confirmedTaskIds, setConfirmedTaskIds] = useState<ReadonlySet<string>>(() => new Set());
  const onConfirmFact = factCtx
    ? (step: DirectorChainStep) => {
        onRequestAction({
          kind: "create-memory-candidate",
          label: `沉淀记忆候选：${step.title}`,
          path: factCtx.projectRoot,
          source: "Tauri 应用数据目录",
          boundary: "只产候选、不是正式记忆；候选待治理转正才进正式记忆库。",
          memoryCandidateCreation: buildFactMemoryCandidate(step, factCtx),
        });
        setConfirmedTaskIds((prev) => new Set(prev).add(step.planned_task_id));
      }
    : undefined;
  const resultLine = chain
    ? `完成 ${chain.completed} 步${chain.stopped_reason ? `；中途停了：${chain.stopped_reason}` : ""}。`
    : outcome?.message || (isCompleted ? "做完了。" : "这单没有完整交货。");
  const isReadOnlyRun =
    (outcome?.planned_tasks ?? []).length > 0 &&
    (outcome?.planned_tasks ?? []).every((task) => task.scope.allowed_write_scope.length === 0);
  // fix3 后端新 warnings（如「角色已按 codex-dev 执行」「已接续上次中断的运行」）→ 小字列出，不挡主路径。
  const warnings = chain?.warnings ?? [];

  return (
    <div className="project-canvas-detail-card jiaoban-done" aria-label={isCompleted ? "做好了" : "未完整交货"}>
      <div className="panel-heading">
        <div>
          <h3 className="jiaoban-done-title">
            {isCompleted
              ? jiaobanDoneTitle(chain?.steps ?? [])
              : needsRework
                ? "这一步需要重做"
                : "这单没有完整交货"}
          </h3>
        </div>
        <Badge tone={isCompleted ? "candidate" : "warning"}>{isCompleted ? "已交货" : "未交货"}</Badge>
      </div>
      <div className="role-loop-plain" aria-label="结果（人话）">
        <p className="role-loop-plain-lead">{resultLine}</p>
        {isReadOnlyRun ? <p className="role-loop-plain-note">只读单·未改文件</p> : null}
      </div>
      <JiaobanStepReportList
        steps={chain?.steps ?? []}
        onConfirmFact={onConfirmFact}
        confirmedTaskIds={confirmedTaskIds}
      />
      <JiaobanRawSessionLink
        sessionChoice={sessionChoice}
        latestSessionThreadId={latestSessionThreadId}
        onOpenAgentSession={onOpenAgentSession}
      />
      {needsRework ? (
        <JiaobanNeedsReworkDisposal
          reason={needsRework.reason}
          actionsReady={needsRework.actionsReady}
          starting={needsReworkActionStarting}
          error={needsReworkActionError}
          onContinue={onNeedsReworkContinue}
          onAction={onNeedsReworkAction}
        />
      ) : null}
      {/* B1：全局主管复核区——交货后 async 后填（loading/意见/不可用+重试），不挡上面交货内容。 */}
      <JiaobanSupervisorReviewSection
        loading={supervisorLoading}
        outcome={supervisorOutcome}
        onRetry={onSupervisorRetry}
        onReplan={onSupervisorReplan}
      />
      {warnings.length > 0 ? (
        <ul className="jiaoban-warnings muted small-note" aria-label="附带说明">
          {warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      ) : null}
      {!needsRework ? (
        <div className="workflow-state-actions">
          <button className="primary-button" type="button" onClick={onContinue}>
            继续弄别的
          </button>
        </div>
      ) : null}
    </div>
  );
}

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

// 判是不是合流命令对「已确认」方案的那一类干净拒（方案不是待用户确认状态）。
// 命中 → 授权本还活着、不用重批，卡住脸该给[接着跑,不用重批]（而非引导重新出方案）。
function isAlreadyConfirmedRejection(e: unknown): boolean {
  const raw = e instanceof Error ? e.message : String(e);
  return (
    raw.includes("待用户确认") ||
    raw.includes("PendingUserConfirmation") ||
    raw.includes("不是「待") ||
    raw.includes("方案不是待")
  );
}

// fix8：供给类错误识别（codex 额度 / 订阅 / 登录 / 服务不可用）。姊妹后端包会带 codex_provider_unavailable:
// 前缀 + 人话，直接取其人话；后端未落地前用兜底关键词匹配。返回 null = 不是供给类（交给别的 humanize）。
function humanizeProviderUnavailable(e: unknown): string | null {
  const raw = e instanceof Error ? e.message : String(e ?? "");
  const marker = "codex_provider_unavailable";
  if (raw.includes(marker)) {
    const after = raw.split(marker)[1]?.replace(/^["'\s]*[:：]?["'\s]*/, "").trim();
    return after && after.length > 0 ? after : "codex 额度 / 订阅 / 登录不可用——处理后点重试。";
  }
  if (/\b403\b|SUBSCRIPTION|quota|usage limit|\b401\b|unauthorized|consult_last_message_read_failed/i.test(raw)) {
    return "codex 服务不可用（常见：额度用完 / 订阅过期 / 登录失效）——处理后点重试；若是网络抽风，重试一次通常就过。";
  }
  return null;
}

// fix8：出方案失败的人话。先认供给类；否则显后端原话（有）；再兜底一句「点重试或改要求」。绝不静默。
function humanizeConsultError(e: unknown): string {
  const provider = humanizeProviderUnavailable(e);
  if (provider) return provider;
  const raw = e instanceof Error ? e.message : String(e ?? "");
  return raw && raw.trim().length > 0 ? raw : "出方案没成——可以点重试，或改一下要求再来一版。";
}

// 合流命令的报错翻人话。最要紧的一类：对「已确认」方案后端会拒（方案不是待用户确认状态）——
// 那不是系统坏了，是这份方案已经批过、授权还活着，引导用户点[接着跑,不用重批]而非重批。
function humanizeAuthorizeError(e: unknown): string {
  // fix8：合流 / 接着跑撞供给死时，同用供给类人话（否则裸抛英文栈让人以为系统坏了）。
  const provider = humanizeProviderUnavailable(e);
  if (provider) return provider;
  const raw = e instanceof Error ? e.message : String(e);
  // 后端拒词：ProjectConsultationProposalStatus 不是 PendingUserConfirmation 时的那句。
  // 这份已批过 → 不裸抛原始错误，翻成「已经批过了，点下面接着跑」。
  if (isAlreadyConfirmedRejection(e)) {
    return "这份方案已经批过了——不用重批，点下面「接着跑」，会从拆任务接着往下推进。";
  }
  if (raw.includes("找不到方案")) {
    return "找不到这份方案了（可能已被新方案取代）。点「重新出方案」重新说一遍目标。";
  }
  return raw;
}

// ============================================================
// 人话化辅助（把后端结构翻成给用户看的话；主路径不出现节点 id / 链 id）
// ============================================================

// 从 proposed_steps 里抽「目标文件：…」行的文件部分（后端 consultant_agent 会把 target_files 塞在最前）。
function extractTargetFiles(proposedSteps: string[]): string | null {
  const line = proposedSteps.find((step) => step.startsWith("目标文件："));
  if (!line) return null;
  const files = line.replace(/^目标文件：/, "").trim();
  return files || null;
}

export function isDirectorPlanningPhase(
  phase: JiaobanPhase,
  chainStatus: ProjectWorkflowChainStatus | null,
): boolean {
  return phase === "running" && (!chainStatus || chainStatus.nodes.length === 0);
}

// 链状态 → 「正在…第 x/y 步」。链事件还没出现的阶段（拿不到节点）= 主管还在拆任务，据实说清。
export function humanizeChainProgress(
  chainStatus: ProjectWorkflowChainStatus | null,
  directorPlanningElapsedMinutes: number,
): string {
  if (!chainStatus || chainStatus.nodes.length === 0) {
    return `主管正在拆任务 · 已 ${Math.max(0, directorPlanningElapsedMinutes)} 分钟`;
  }
  const total = chainStatus.nodes.length;
  const done = countDoneNodes(chainStatus);
  const current = Math.min(done + 1, total);
  return `正在做第 ${current}/${total} 步…`;
}

function countDoneNodes(chainStatus: ProjectWorkflowChainStatus | null): number {
  if (!chainStatus) return 0;
  return chainStatus.nodes.filter((node) => /(finished|completed|done|succeeded|accepted)/i.test(node.state)).length;
}
