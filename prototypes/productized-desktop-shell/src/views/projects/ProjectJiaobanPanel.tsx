import { useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "../../components/Badge";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  applyProjectDirectorFailedAction,
  autoAdvanceAuthorizedRoleLoop,
  confirmAndStartAuthorizedRun,
  getProjectWorkflowChainStatus,
  listProjectRunHistory,
  loadFormalMemoryStore,
  previewPendingProposalDirectorPlan,
  runGlobalSupervisorBoundaryReview,
  runGlobalSupervisorReview,
  runProjectConsultation,
  stopProjectWorkflowChain,
} from "../../lib/tauri";
import type {
  AutoAdvanceRoleLoopOutcome,
  CreateMemoryCandidateInput,
  DirectorChainStep,
  GlobalSupervisorBoundaryReviewOutcome,
  GlobalSupervisorReviewOutcome,
  PendingAction,
  PlanAuthorizationStoreV1,
  ProjectConsultationProposal,
  ProjectDirectorPlannedTask,
  ProjectConsultationProposalStoreV1,
  ProjectRecord,
  ProjectWorkflowChainStatus,
  RunHistoryEntry,
  RunHistoryList,
  SessionRecord,
  WorkflowStateSnapshot,
} from "../../lib/types";

// 固定测试项目（自动干只在这真跑；非它则老实标注·跳智能体直连）。与 WorkflowCommandConsoleView 同一常量。
const TEST_PROJECT_ROOT = "/Users/yoyi/codex-workflow-mario-test";

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
type JiaobanPhase = "say" | "authorize" | "running" | "done" | "blocked";

// 方案a fix：「开个新的」的显式哨兵值。此前用 null 一词两用（"还没定" 与 "用户选了新建"），
// 重挂载/默认效果会把用户的显式选择无声改回「接现有」（真机踩到：明确选了新建却路由到旧对话）。
// export 供离线 DOM 断言测试对齐哨兵值（纯暴露·不改会话选择语义）。
export const NEW_SESSION_CHOICE = "__new_session__";

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
  // B1：本轮全局主管复核结果（key=chain_started_at·结果态缓存；loading 是瞬态不缓存——
  // 重挂载后按幂等键补拉，后端幂等命中秒回不重烧）。
  supervisorReview: { key: string; outcome: GlobalSupervisorReviewOutcome } | null;
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
    supervisorReview: null,
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
  if (/找不到方案|proposal/i.test(raw)) return "这份方案暂时读不到，工序图没画出来（可重试）。";
  return "工序图没画出来（可重试）；不影响你批。";
}

const DAY_MS = 24 * 60 * 60 * 1000;

// 方案生成时间 → 「今天/几天前」。用日历日判「不是今天」（避免刚过午夜的边界误判）。
function proposalAgeDays(createdAtMs: number): number {
  const created = new Date(createdAtMs);
  const now = new Date();
  const createdDay = new Date(created.getFullYear(), created.getMonth(), created.getDate()).getTime();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  return Math.max(0, Math.round((today - createdDay) / DAY_MS));
}

function formatProposalTime(createdAtMs: number): string {
  const d = new Date(createdAtMs);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

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

  // 刀2「批前看图」：复杂活（proposal.suggest_workflow）或用户开「按工作流来」→ 批前预拆工序图给用户看。
  // 结果按 proposal_id 缓存（切 tab 回来不重拆·下面 effect 命中即恢复）；loading/error 是瞬态。
  // 批时 previewTasks 原样回传后端 = 所见即所跑；简单活/预拆失败则 previewTasks 为空 → 不传 → 后端照旧拆。
  const [workflowSwitchOn, setWorkflowSwitchOn] = useState(latestProposal?.suggest_workflow === true);
  const [previewTasks, setPreviewTasks] = useState<ProjectDirectorPlannedTask[] | null>(null);
  const [previewWarnings, setPreviewWarnings] = useState<string[]>([]);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [previewNonce, setPreviewNonce] = useState(0); // 重试 bump：effect 依赖没变时强制重跑预拆
  const previewLoadingRef = useRef(false);

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
    setAmendment("");
    ranProposalIdRef.current = null;
    clearJiaobanRunCache(projectRoot);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [latestProposal?.proposal_id, latestProposal?.status]);

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
  // （原守 outcome.stage==ran，但合流/接着跑是同步跑到底才返回、跑中 outcome 恒 null，重挂载新实例 outcome 也 null → 整个执行期一格不轮、卡「正在干」。）
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

  // 当前该显哪张脸：手动相位优先；否则由「有没有方案」决定说/批。
  const phase: JiaobanPhase = manualPhase ?? (latestProposal ? "authorize" : "say");
  // fix6-v2：只认「这一轮」的链——started_at（后端 ms）>= 本轮点击时刻。旧链时间戳更早、天然排除，
  // 防拆任务期（本轮链还没起、轮询却拿到旧链的绿状态）提前翻交货 / 显旧步骤。链没起 = null → 照显「主管正在拆任务」。
  const thisRoundChainStatus =
    chainStatus && runStartedAtMs != null && Number(chainStatus.started_at) >= runStartedAtMs ? chainStatus : null;

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
        goal: goal.trim(),
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
    previewLoadingRef.current = false;
  }, [latestProposal?.proposal_id]);

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

  // 允许并开始 = 方案授权人闸那一下。走刀1 合流命令 confirm_and_start_authorized_run（见 lib/tauri）。
  async function authorizeAndStart() {
    if (!projectWorkflow || !latestProposal || starting || runningRef.current) return;
    // ★ 一进来立刻上「正在干」脸——不等 await 回来（await 可能几十秒~几分钟，中间不能无脸看着像冻死）。
    runningRef.current = true;
    ranProposalIdRef.current = latestProposal.proposal_id;
    const runStartedAt = Date.now(); // fix6-v2：本轮起点，判「这轮的链」用
    // 方案a fix：显式哨兵或真无会话（null）都算 new；现有 thread_id 才是 existing。
    const isNewSession = sessionChoice === NEW_SESSION_CHOICE || sessionChoice === null;
    setRunStartedAtMs(runStartedAt);
    setRunIsNewSession(isNewSession);
    setStarting(true);
    setStartError(null);
    setOutcome(null);
    setChainStatus(null);
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
    });
    try {
      // 方案a：sessionChoice=null → 传 session_choice:"new" 不传 session_id（后端 014c254 先生后绑真建会话）；
      // 有值 → existing 绑现有会话（原样不动）。
      const outcome = await confirmAndStartAuthorizedRun({
        project_root: projectRoot,
        proposal_id: latestProposal.proposal_id,
        session_choice: isNewSession ? "new" : "existing",
        session_id: isNewSession ? undefined : (sessionChoice ?? undefined),
        actor_id: "user",
        // 刀2 所批即所跑：批前预拆成功过就把那份图原样带回 → 后端照图跑不重拆；简单活/预拆失败 previewTasks 空 → 不传。
        approved_planned_tasks: previewTasks ?? undefined,
      });
      // fix6：命令返回 ran = 链已跑完（chain_outcome 在返回体里）→ 直接翻交货脸，不设 running 干等轮询。
      const nextPhase: JiaobanPhase = outcome.stage === "ran" ? "done" : "blocked";
      setOutcome(outcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, { manualPhase: nextPhase, outcome, startError: null });
    } catch (e) {
      // 合流命令对「已确认」会干净拒（后端：方案不是待用户确认状态）。这不是坏了——授权本还活着，
      // 直接调 auto_advance 就能从拆任务接着跑。故这一类翻成「已经批过了，点接着跑」+ 卡住脸给[接着跑]。
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
      // fix6：ran = 链已跑完 → 直接交货脸，不干等轮询。
      const nextPhase: JiaobanPhase = nextOutcome.stage === "ran" ? "done" : "blocked";
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

  async function applyNeedsReworkAction(action: "change_session" | "rework" | "archive") {
    if (!projectWorkflow || !outcome?.chain_outcome || starting) return;
    const plannedTaskId =
      outcome.chain_outcome?.steps.find((step) => step.state === "needs_rework")?.planned_task_id ??
      thisRoundChainStatus?.nodes.find((node) => node.state === "needs_rework")?.node_id;
    const plannedTask = outcome.planned_tasks?.find((task) => task.planned_task_id === plannedTaskId);
    if (!plannedTaskId || !plannedTask) {
      setNeedsReworkActionError("这单的重做信息还没载入，暂时不能换会话、重拆或结束；可以先按原样接着跑。");
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
        explicit_retry_or_reopen: action === "change_session",
        planned_task: plannedTask,
        max_nodes: 1,
      });
      const nextOutcome: AutoAdvanceRoleLoopOutcome = {
        stage: "ran",
        planned_task_count: outcome.planned_task_count,
        prepared_count: outcome.prepared_count,
        needs_binding_count: outcome.needs_binding_count,
        blocked_count: outcome.blocked_count,
        message: result.message,
        chain_outcome:
          action === "change_session" ? (result.chain_outcome ?? null) : action === "rework" ? outcome.chain_outcome : null,
        stop_reason: result.stopped_reason ?? null,
        planned_tasks: outcome.planned_tasks,
        warnings: result.warnings,
      };
      setOutcome(nextOutcome);
      setManualPhase("done");
      writeJiaobanRunCache(projectRoot, { manualPhase: "done", outcome: nextOutcome, startError: null });
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
    setAmendment("");
    ranProposalIdRef.current = null;
    clearJiaobanRunCache(projectRoot);
  }

  // 干完了没有：链状态是否收尾（人话「做好了」）。
  useEffect(() => {
    if (phase !== "running") return;
    // fix6-v2：只认「这一轮」的链收尾才翻交货——防旧链的 completed 状态在拆任务期把新一轮提前翻脸。
    if (thisRoundChainStatus && /(finished|completed|done|succeeded|aborted|stopped|failed)/i.test(thisRoundChainStatus.state)) {
      // 链跑到头 → 交货（aborted/failed 也进交货并给下一步，永不冻；细节人话见结果行）。
      setManualPhase("done");
      writeJiaobanRunCache(projectRoot, { manualPhase: "done" });
    }
  }, [phase, thisRoundChainStatus, projectRoot]);

  // 非测试项目：老实标注 + 跳智能体直连，不装能跑。
  if (!isTestProject) {
    const latestSession =
      projectSessions.sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;
    return (
      <section className="project-jiaoban" aria-label="交办">
        <div className="project-canvas-detail-card" aria-label="交办 · 这个项目暂不能自动干">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">交办</p>
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
              <p className="eyebrow">交办</p>
              <h3>这个项目还没准备好交办</h3>
            </div>
            <Badge tone="warning">缺项目工作流</Badge>
          </div>
          <p className="muted small-note">先在「工作流」tab 建起这个项目的工作流，再回来交办。</p>
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

  return (
    <section className="project-jiaoban project-jiaoban--split" aria-label="交办">
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
            proposalTimeText={formatProposalTime(latestProposal.created_at_ms)}
            proposalIsStale={proposalIsStale}
            proposalAgeDays={proposalAge}
            sessions={projectSessions}
            sessionChoice={sessionChoice}
            onSessionChoiceChange={setSessionChoice}
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
            worksmapWarnings={previewWarnings}
            worksmapLoading={previewLoading}
            worksmapError={previewError}
            onRetryWorksmap={retryPreview}
            boundaryLoading={boundaryLoadingForThisProposal}
            boundaryOutcome={boundaryForThisProposal}
            onBoundaryRetry={() => {
              // [重试]：force 穿透幂等重跑本方案的边界意见。
              if (latestProposal) void requestBoundaryReview(latestProposal.proposal_id, true);
            }}
            onOpenAgentSession={onOpenAgentSession}
          />
        ) : null}

        {phase === "running" ? (
          <JiaobanRunningState
            chainStatus={thisRoundChainStatus}
            isNewSession={runIsNewSession}
            onStop={() => void stopRun()}
            sessionChoice={sessionChoice}
            latestSessionThreadId={latestSessionThreadId}
            onOpenAgentSession={onOpenAgentSession}
          />
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
            onNeedsReworkAction={(action) => void applyNeedsReworkAction(action)}
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
            // 「去工作流看看」需切 tab（onSelectTool 在外壳），本包红线「不动外壳」→ 不在此接线；
            // 保留入口能力（prop 已在），置 null 即不渲染该次按钮，主按钮永在（配对表兜底至少一个主按钮）。
            onOpenWorkflow={null}
            latestSessionThreadId={latestSessionThreadId}
            onOpenAgentSession={onOpenAgentSession}
          />
        ) : null}

          </div>
        )}
      </div>
    </section>
  );
}

// ============================================================
// Part①·工作历史左栏（纯只读呈现·吃 listProjectRunHistory 读模型；五态主区行为 0-diff）
// ============================================================

type HistoryVisual = { dot: string; toneClass: string; word: string };
type HistoryFilter = "all" | "mine" | "running";

// 交货带复核/边界告警、或批前边界对不上 = 「有要看的」。
function historyHasAttention(entry: RunHistoryEntry): boolean {
  const r = entry.review_flags.result_verdict;
  const b = entry.review_flags.boundary_verdict;
  return r === "needs_human_check" || r === "needs_rework" || r === "human_verify" || b === "mismatch";
}

// 九态英文键 → 状态点 + 配色 class + 短词（词表死线：主路径人话，不露英文枚举）。
function historyStateVisual(entry: RunHistoryEntry): HistoryVisual {
  switch (entry.state) {
    case "running":
      return { dot: "●", toneClass: "run-dot-running", word: "跑着" };
    case "blocked":
      return { dot: "⚠", toneClass: "run-dot-blocked", word: "卡住" };
    case "delivered":
      return {
        dot: "✓",
        toneClass: historyHasAttention(entry) ? "run-dot-attention" : "run-dot-done",
        word: "交货",
      };
    case "pending":
      return { dot: "○", toneClass: "run-dot-pending", word: "等你批" };
    case "advice_only":
      return { dot: "◐", toneClass: "run-dot-advice", word: "纯建议" };
    case "confirmed_not_run":
      return { dot: "◌", toneClass: "run-dot-confirmed", word: "批了没跑" };
    case "declined":
      return { dot: "·", toneClass: "run-dot-terminal", word: "已回绝" };
    case "superseded":
      return { dot: "·", toneClass: "run-dot-terminal", word: "被替代" };
    case "changes_requested":
      return { dot: "·", toneClass: "run-dot-terminal", word: "要改" };
    default:
      return { dot: "·", toneClass: "run-dot-terminal", word: entry.state_note || "—" };
  }
}

// A·错误族 → 人话短标（前端映射·不露 family 机器键；未知族兜底「运行错误」）。
function historyErrorFamilyLabel(family: string): string {
  switch (family) {
    case "provider_unavailable":
      return "供给不可用";
    case "network":
      return "网络抽风";
    case "timeout":
      return "超时";
    case "sandbox_denied":
      return "权限/沙箱";
    case "command_failed":
      return "命令失败";
    case "codex_subsystem":
      return "codex 子系统";
    case "readback_failed":
      return "口供没读回";
    default:
      return "运行错误";
  }
}

function matchesHistoryFilter(entry: RunHistoryEntry, filter: HistoryFilter): boolean {
  if (filter === "running") return entry.state === "running";
  if (filter === "mine")
    return (
      entry.state === "pending" ||
      entry.state === "blocked" ||
      (entry.state === "delivered" && historyHasAttention(entry))
    );
  return true;
}

// 紧凑时间：今天 HH:mm / 昨天 / 更早 MM-DD。复用 proposalAgeDays 的日历日判据。
function formatHistoryTime(createdAtMs: number): string {
  const days = proposalAgeDays(createdAtMs);
  const d = new Date(createdAtMs);
  if (days <= 0) return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  if (days === 1) return "昨天";
  return `${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

// verdict 英文枚举 → 人话（词表死线）。
function humanizeVerdict(v: string): string {
  switch (v) {
    case "pass":
      return "通过";
    case "needs_rework":
      return "要返工";
    case "needs_human_check":
    case "human_verify":
      return "建议你亲验";
    case "looks_ok":
      return "看着没问题";
    case "mismatch":
      return "对不上目标";
    case "caution":
      return "留个心";
    default:
      return v;
  }
}

export function JiaobanHistoryColumn({
  entries,
  total,
  loading,
  filter,
  onFilterChange,
  selectedId,
  currentProposalId,
  latestBlockedId,
  onSelectEntry,
  onBackToCurrent,
  onNewJiaoban,
  onContinueRun,
}: {
  entries: RunHistoryEntry[];
  total: number;
  loading: boolean;
  filter: HistoryFilter;
  onFilterChange: (filter: HistoryFilter) => void;
  selectedId: string | null;
  currentProposalId: string | null;
  latestBlockedId: string | null;
  onSelectEntry: (entry: RunHistoryEntry) => void;
  onBackToCurrent: () => void;
  onNewJiaoban: () => void;
  onContinueRun: () => void;
}) {
  const mineCount = entries.filter((entry) => matchesHistoryFilter(entry, "mine")).length;
  const runningCount = entries.filter((entry) => entry.state === "running").length;
  const visible = entries.filter((entry) => matchesHistoryFilter(entry, filter));
  return (
    <aside className="jiaoban-history" aria-label="工作历史">
      <div className="jiaoban-history-head">
        <strong>工作历史</strong>
        <button
          className="primary-button jiaoban-history-new"
          type="button"
          onClick={onNewJiaoban}
          aria-label="新交办"
          title="新交办"
        >
          +
        </button>
      </div>
      <div className="jiaoban-history-filters">
        <button className={`jiaoban-chip ${filter === "all" ? "on" : ""}`} type="button" onClick={() => onFilterChange("all")}>
          全部
        </button>
        <button className={`jiaoban-chip ${filter === "mine" ? "on" : ""}`} type="button" onClick={() => onFilterChange("mine")}>
          等我的{mineCount ? ` ${mineCount}` : ""}
        </button>
        <button className={`jiaoban-chip ${filter === "running" ? "on" : ""}`} type="button" onClick={() => onFilterChange("running")}>
          跑着{runningCount ? ` ${runningCount}` : ""}
        </button>
      </div>
      <div className="jiaoban-history-list">
        {loading && entries.length === 0 ? (
          <p className="muted small-note">正在读工作历史…</p>
        ) : total === 0 ? (
          <div className="jiaoban-history-empty">
            <p className="muted">这个项目还没交办过活。</p>
            <button className="primary-button" type="button" onClick={onNewJiaoban}>
              + 新交办
            </button>
          </div>
        ) : visible.length === 0 ? (
          <p className="muted small-note">这个筛选下没有单子。</p>
        ) : (
          visible.map((entry) => {
            const visual = historyStateVisual(entry);
            const isCurrent = currentProposalId != null && entry.proposal_id === currentProposalId;
            const isSelected = selectedId === entry.proposal_id || (selectedId === null && isCurrent);
            const showContinue = entry.state === "blocked" && entry.proposal_id === latestBlockedId;
            return (
              <button
                key={entry.proposal_id}
                className={`jiaoban-run ${isSelected ? "on" : ""}`}
                type="button"
                onClick={() => (isCurrent ? onBackToCurrent() : onSelectEntry(entry))}
              >
                <span className="jiaoban-run-r1">
                  <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
                    {visual.dot}
                  </span>
                  <span className="jiaoban-run-goal">{entry.goal_text || "（没写目标）"}</span>
                </span>
                <span className="jiaoban-run-meta">
                  <span>{formatHistoryTime(entry.created_at_ms)}</span>
                  <span aria-hidden="true">·</span>
                  <span className="jiaoban-run-note">{entry.state_note}</span>
                  {showContinue ? (
                    <span
                      className="jiaoban-run-quick"
                      role="button"
                      tabIndex={0}
                      onClick={(event) => {
                        event.stopPropagation();
                        onContinueRun();
                      }}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.stopPropagation();
                          onContinueRun();
                        }
                      }}
                    >
                      接着跑
                    </span>
                  ) : null}
                </span>
              </button>
            );
          })
        )}
      </div>
    </aside>
  );
}

// 选中「非当前」历史单 → 主区显只读详情卡（不回放完整五态脸·诚实显读模型有的）。
export function JiaobanHistoryDetail({ entry, onBackToCurrent }: { entry: RunHistoryEntry; onBackToCurrent: () => void }) {
  const visual = historyStateVisual(entry);
  const flags = entry.review_flags;
  const opinions = [
    flags.result_verdict ? `结果复核：${humanizeVerdict(flags.result_verdict)}` : null,
    flags.boundary_verdict ? `批前边界：${humanizeVerdict(flags.boundary_verdict)}` : null,
  ].filter(Boolean);
  return (
    <div className="project-canvas-detail-card jiaoban-history-detail" aria-label="历史单详情">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作历史 · 这一单</p>
          <h3>
            <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
              {visual.dot}
            </span>{" "}
            {visual.word}
          </h3>
        </div>
        <button className="secondary-button" type="button" onClick={onBackToCurrent}>
          回到当前
        </button>
      </div>
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">目标：</span>
        {entry.goal_text || "（没写目标）"}
      </p>
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">状态：</span>
        {entry.state_note}
      </p>
      {/* A·运行错误两层脸：默认显人话摘要+族标；「查看原文」下钻原始 stderr。呈现不阻断（同黄牌哲学）。 */}
      {entry.error ? (
        <div className="jiaoban-run-error" aria-label="运行错误诊断">
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">出错：</span>
            <span className="jiaoban-run-error-family">
              {historyErrorFamilyLabel(entry.error.family)}
            </span>
            <span className="jiaoban-run-error-human">{entry.error.human}</span>
          </p>
          {entry.error.raw_snippet.trim() ? (
            <details className="jiaoban-run-error-raw">
              <summary>查看原文</summary>
              <pre>{entry.error.raw_snippet}</pre>
            </details>
          ) : null}
        </div>
      ) : null}
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">时间：</span>
        {formatProposalTime(entry.created_at_ms)}
      </p>
      {entry.chain ? (
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">进度：</span>
          做到第 {entry.chain.done_count}/{entry.chain.total_count} 步
        </p>
      ) : null}
      {opinions.length ? (
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">意见：</span>
          {opinions.join("；")}
        </p>
      ) : null}
      {entry.correlation === "time_window" ? (
        <p className="muted small-note">意见归属按时间近似匹配（不是精确挂到这一单）。</p>
      ) : null}
      <p className="muted small-note">具体每一步的过程，去「工作流」标签看。</p>
    </div>
  );
}

// ============================================================
// 五态子组件
// ============================================================

// 1. 说
function JiaobanSayState({
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
          <p className="eyebrow">交办</p>
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
      <p className="muted small-note">
        AI 会读你的项目、想方案，大约 1–2 分钟；这期间界面不会卡，也可以先去忙别的。
      </p>
    </div>
  );
}

// 2. 批（授权卡·定稿字段）
// export 供离线 DOM 断言（fix9 诚实脸两态；renderToStaticMarkup 渲染·不平铺调用）。
export function JiaobanAuthorizeState({
  proposal,
  proposalTimeText,
  proposalIsStale,
  proposalAgeDays,
  sessions,
  sessionChoice,
  onSessionChoiceChange,
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
  worksmapWarnings,
  worksmapLoading,
  worksmapError,
  onRetryWorksmap,
  boundaryLoading,
  boundaryOutcome,
  onBoundaryRetry,
  onOpenAgentSession,
}: {
  proposal: ProjectConsultationProposal;
  proposalTimeText: string;
  proposalIsStale: boolean;
  proposalAgeDays: number;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
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
  worksmapWarnings: string[];
  worksmapLoading: boolean;
  worksmapError: string | null;
  onRetryWorksmap: () => void;
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
  // fix7：按钮旁「工序图状态话」——图好那刻在用户视线所在（按钮上方）给到场提示，把「所批即所跑」说出口。
  const worksmapReady = !worksmapLoading && !worksmapError && !!(worksmapTasks && worksmapTasks.length);
  const worksmapNote =
    !worksmapSwitchOn || worksmapError
      ? null
      : worksmapLoading
        ? "工序图绘制中…（可先批，先批就按现场拆）"
        : worksmapReady
          ? "✓ 工序图好了——你批的就是这份图"
          : null;

  return (
    <div className="project-canvas-detail-card jiaoban-authorize" aria-label="AI 的方案">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3>AI 的方案</h3>
        </div>
      </div>

      {/* 旧方案不冒充当前：不是今天生成 → 顶部黄条 + 主按钮换「重新说目标」，防再批库存。 */}
      {proposalIsStale ? (
        <div className="jiaoban-stale-banner" role="note" aria-label="旧方案提醒">
          <span aria-hidden="true">⚠</span>
          {/* 同 advice-only 警条：正文包单 span 防 flex 拆柱（此条现在恰好没内联元素才幸免，统一防）。 */}
          <span className="jiaoban-banner-body">
            这是 {proposalAgeDays} 天前的旧方案（生成于 {proposalTimeText}
            ）。项目可能已经变了，建议重新说一遍目标、出一版新的。
          </span>
        </div>
      ) : null}

      {/* fix9·纯建议诚实脸：写根空（=咨询判定不改文件）→ 批前就喊出来，别等批完空转才发现
          （2026-07-07 两撞：tier-1 偶发不交 execution_scope，用户批了两份空转方案）。 */}
      {!willWrite ? (
        <div className="jiaoban-advice-only-banner" role="note" aria-label="纯建议方案提醒">
          <span aria-hidden="true">⚠</span>
          {/* 正文必须是**单个** flex item：banner 家族是 display:flex，裸文本+<strong> 会被拆成
              多个匿名 item 挤成竖柱（07-07 用户真机截图逮到）。包一层 span=正文内联流恢复。 */}
          <span className="jiaoban-banner-body">
            这份方案<strong>不会改任何文件</strong>——它是纯建议。
            你的目标若是要动手改东西，别批这份，点下面「重新出方案（要动手）」。
          </span>
        </div>
      ) : null}

      <div className="role-loop-plain jiaoban-plan-body" aria-label="方案要点（人话）">
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">目标：</span>
          {proposal.user_goal}
        </p>
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
        {proposal.acceptance_criteria.length ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">改完怎么验：</span>
            {proposal.acceptance_criteria.join("；")}
          </p>
        ) : null}
        <p className="jiaoban-field jiaoban-field-time">
          <span className="jiaoban-field-label">方案生成于：</span>
          {proposalTimeText}
        </p>
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
        loading={worksmapLoading}
        tasks={worksmapTasks}
        warnings={worksmapWarnings}
        error={worksmapError}
        onRetry={onRetryWorksmap}
      />

      <JiaobanSessionPicker
        sessions={sessions}
        sessionChoice={sessionChoice}
        onSessionChoiceChange={onSessionChoiceChange}
        onOpenAgentSession={onOpenAgentSession}
      />

      {willWrite ? (
        <div className="jiaoban-grant" role="note">
          <span aria-hidden="true">🔓</span> 需要你允许：改这个测试项目
        </div>
      ) : null}

      <p className="muted small-note">碰到越界或拿不准的，会停下来问你，不硬来。</p>

      <label className="proposal-decision-field jiaoban-amend">
        <span>想改就直接说</span>
        <input
          type="text"
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
          // fix9·纯建议方案：主按钮改道 [重新出方案（要动手）]；[允许并开始] 降为次按钮**不删死**
          //（用户真想收下纯建议也有路；点了会被后端开工口守卫人话拒，现有失败上脸机制接得住）。
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onRePlan}
            >
              {consultLoading ? "正在出新方案…" : "重新出方案（要动手）"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "仍要允许并开始（纯建议）"}
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
      <p className="muted small-note">点「允许并开始」= 允许这段自动跑，后面不再逐步问你。</p>
    </div>
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

// 刀2「批前看图」·工序图区。开关开着（AI 建议时默认开·可手动关）才显图；不用重型图库——任务名 +「等：X」
// 表达先后（planned_tasks[].depends_on 是前置任务的 title）。预拆慢/失败都不挡批：loading 照常可批、失败给
// 重试 + 人话「不影响批」。词表：只显 title，不露 node_id/planned_task_id。
function JiaobanWorksmap({
  suggestWorkflow,
  switchOn,
  onToggleSwitch,
  loading,
  tasks,
  warnings,
  error,
  onRetry,
}: {
  suggestWorkflow: boolean;
  switchOn: boolean;
  onToggleSwitch: (value: boolean) => void;
  loading: boolean;
  tasks: ProjectDirectorPlannedTask[] | null;
  warnings: string[];
  error: string | null;
  onRetry: () => void;
}) {
  return (
    <div className="jiaoban-worksmap" aria-label="工序图">
      <label className="jiaoban-worksmap-toggle">
        <input type="checkbox" checked={switchOn} onChange={(event) => onToggleSwitch(event.target.checked)} />
        <span>按工作流来（先看工序图再动手）</span>
        {suggestWorkflow ? <span className="jiaoban-worksmap-suggest">AI 建议：这活值得先看图</span> : null}
      </label>

      {switchOn ? (
        <div className="jiaoban-worksmap-body">
          {loading ? (
            <p className="muted small-note">正在画工序图…（大约 1–7 分钟，这期间可以照常批）</p>
          ) : error ? (
            <div className="jiaoban-worksmap-error" role="note">
              <span>{error}</span>
              <button className="secondary-button" type="button" onClick={onRetry}>
                重试
              </button>
            </div>
          ) : tasks && tasks.length ? (
            <>
              {/* fix7：步骤框纵向流——一眼像流程图（框 + 向下箭头）。简单先后靠箭头表达；等多个/跨步的
                  依赖（不是紧邻上一步）才在框里挂「等：X」小标签补充。arrived class 让图画好那刻高亮一次。 */}
              <div className="jiaoban-worksmap-graph jiaoban-worksmap-arrived">
                {tasks.map((task, index) => {
                  const prevTitle = index > 0 ? tasks[index - 1].title : null;
                  const extraDeps = task.depends_on.filter((dep) => dep !== prevTitle);
                  return (
                    <div key={task.planned_task_id} className="jiaoban-worksmap-node-wrap">
                      {index > 0 ? (
                        <span className="jiaoban-worksmap-arrow" aria-hidden="true">
                          ↓
                        </span>
                      ) : null}
                      <div className="jiaoban-worksmap-node">
                        <span className="jiaoban-worksmap-step">{index + 1}</span>
                        <span className="jiaoban-worksmap-task">{task.title}</span>
                        {extraDeps.length ? (
                          <span className="jiaoban-worksmap-dep">等：{extraDeps.join("、")}</span>
                        ) : null}
                      </div>
                    </div>
                  );
                })}
              </div>
              {warnings.length ? (
                <ul className="jiaoban-worksmap-warnings" aria-label="工序图提醒">
                  {warnings.map((warning, index) => (
                    <li key={index}>{warning}</li>
                  ))}
                </ul>
              ) : null}
              <p className="muted small-note">完整工序图可在上方「工作流」标签里看大图。</p>
            </>
          ) : tasks ? (
            <p className="muted small-note">这活拆下来就一步，不用画图，直接批就行。</p>
          ) : (
            <p className="muted small-note">正在准备工序图…</p>
          )}
        </div>
      ) : null}
    </div>
  );
}

// 「看原始对话」桥（定稿承诺补做·2026-07-06）：凡能确定对话的地方给一键钻进智能体页。
// 判据诚实三态：sessionChoice=真 thread_id → 「看原始对话」（就是本单那条）；
// 哨兵/未定（新会话单——真 id 前端拿不到，别猜别反查）→ 给了 latestSessionThreadId 才显
// 「看最近对话」（最近≈本单但不保证，词表不吹大）；两者皆无 → 零渲染。
// 纯导航：走已存在的 onOpenAgentSession（App 级 setFocusedAgentThreadId + 切智能体页，现成）。
// 无 hooks·export——离线 DOM 断言可直接走元素树点 onClick（harness 限制：带 hooks 的只能静态标记断言）。
export function JiaobanRawSessionLink({
  sessionChoice,
  latestSessionThreadId = null,
  onOpenAgentSession,
}: {
  sessionChoice: string | null;
  latestSessionThreadId?: string | null;
  onOpenAgentSession: ((threadId: string) => void) | undefined;
}) {
  if (!onOpenAgentSession) return null;
  const realThreadId =
    sessionChoice && sessionChoice !== NEW_SESSION_CHOICE ? sessionChoice : null;
  const target = realThreadId ?? latestSessionThreadId;
  if (!target) return null;
  return (
    <button
      type="button"
      className="jiaoban-linklike jiaoban-raw-session-link"
      onClick={() => onOpenAgentSession(target)}
    >
      {realThreadId ? "看原始对话" : "看最近对话"}
    </button>
  );
}

// 会话收纳：默认收起一行「用哪个对话干：接现有 · <最近一条标题> ▾」，点开才展开选择。
// 展开后：最近 5 条直列 + 其余折叠/可搜；「开个新的」置灰标「下一阶段支持」（用户已拍方案 a 下阶段）。
// export 供离线 DOM 断言（带 hooks → 测试只静态标记断言，不平铺调用）。
export function JiaobanSessionPicker({
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  onOpenAgentSession,
}: {
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  // 「看原始对话」桥（可选）：批卡传、卡住脸不传（卡住脸自己有面级入口，防一脸双入口）。
  onOpenAgentSession?: (threadId: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [showRest, setShowRest] = useState(false);

  // 最近在前。
  const sorted = useMemo(
    () => [...sessions].sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0)),
    [sessions],
  );
  const selected = sorted.find((s) => s.thread_id === sessionChoice) ?? null;
  // 方案a fix：收起行必须说真话——选了新建就显「开个新的」，别拿最新旧会话标题冒充（真机踩到的帮凶）。
  const summaryTitle =
    sessionChoice === NEW_SESSION_CHOICE
      ? "开个新的（为这单活新建对话）"
      : selected?.title || selected?.thread_id || `${sorted[0]?.title ?? "选一条对话"}（默认）`;

  // 无可用会话：给人话提示，不给空壳单选。
  if (sorted.length === 0) {
    return (
      <div className="jiaoban-session-pick jiaoban-session-empty" aria-label="用哪个对话干">
        <p className="jiaoban-field-label" style={{ margin: 0 }}>
          用哪个对话干
        </p>
        <p className="muted small-note" style={{ margin: 0 }}>
          这个项目还没有可用的对话。先去「智能体」页开一条，再回来交办。
        </p>
      </div>
    );
  }

  const recent = sorted.slice(0, 5);
  const rest = sorted.slice(5);
  const filteredRest = query.trim()
    ? sorted.filter(
        (s) =>
          (s.title || s.thread_id).toLowerCase().includes(query.trim().toLowerCase()) &&
          !recent.includes(s),
      )
    : rest;

  return (
    <div className="jiaoban-session-pick" aria-label="用哪个对话干">
      <div className="jiaoban-session-summary-row">
        <button
          type="button"
          className="jiaoban-session-summary"
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
        >
          <span className="jiaoban-field-label">用哪个对话干：</span>
          <span className="jiaoban-session-summary-value">
            {sessionChoice === NEW_SESSION_CHOICE ? summaryTitle : `接现有 · ${summaryTitle}`}
          </span>
          <span aria-hidden="true" className="jiaoban-session-caret">
            {open ? "▴" : "▾"}
          </span>
        </button>
        {/* 行尾「看原始对话」：只在选了「接现有:X」时显（新建/未定 → 会话还没生，没得看——诚实不显）。 */}
        <JiaobanRawSessionLink
          sessionChoice={sessionChoice}
          onOpenAgentSession={onOpenAgentSession}
        />
      </div>

      {open ? (
        <div className="jiaoban-session-expand">
          {/* 方案a fix：「开个新的」用显式哨兵（不再用 null 一词两用）——重挂载/默认效果都不会吞掉这个选择。 */}
          <label className="jiaoban-radio">
            <input
              type="radio"
              name="jiaoban-session"
              checked={sessionChoice === NEW_SESSION_CHOICE}
              onChange={() => onSessionChoiceChange(NEW_SESSION_CHOICE)}
            />
            开个新的（为这单活新建一个对话）
          </label>

          {recent.map((session) => (
            <label className="jiaoban-radio" key={session.thread_id}>
              <input
                type="radio"
                name="jiaoban-session"
                checked={sessionChoice === session.thread_id}
                onChange={() => onSessionChoiceChange(session.thread_id)}
              />
              接现有：{session.title || session.thread_id}
            </label>
          ))}

          {rest.length > 0 ? (
            <div className="jiaoban-session-rest">
              {!showRest ? (
                <button
                  type="button"
                  className="jiaoban-linklike"
                  onClick={() => setShowRest(true)}
                >
                  还有 {rest.length} 条更早的对话，展开选…
                </button>
              ) : (
                <>
                  <input
                    type="text"
                    className="jiaoban-session-search"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="搜对话标题…"
                  />
                  {filteredRest.map((session) => (
                    <label className="jiaoban-radio" key={session.thread_id}>
                      <input
                        type="radio"
                        name="jiaoban-session"
                        checked={sessionChoice === session.thread_id}
                        onChange={() => onSessionChoiceChange(session.thread_id)}
                      />
                      接现有：{session.title || session.thread_id}
                    </label>
                  ))}
                  {filteredRest.length === 0 ? (
                    <p className="muted small-note" style={{ margin: 0 }}>
                      没有匹配的对话。
                    </p>
                  ) : null}
                </>
              )}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

// 3. 干（人话进度）
function JiaobanRunningState({
  chainStatus,
  isNewSession,
  onStop,
  sessionChoice,
  latestSessionThreadId,
  onOpenAgentSession,
}: {
  chainStatus: ProjectWorkflowChainStatus | null;
  isNewSession: boolean;
  onStop: () => void;
  // 「看原始对话」桥：existing 单跑中→看原始对话（能看实时进度）；哨兵单→latestSession 兜底看最近对话。
  sessionChoice: string | null;
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
}) {
  const progress = humanizeChainProgress(chainStatus);
  return (
    <div className="project-canvas-detail-card jiaoban-running" aria-label="正在干">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3>正在干…</h3>
        </div>
        <Badge tone="candidate">进行中</Badge>
      </div>
      <div className="role-loop-plain" aria-label="进度（人话）">
        <p className="role-loop-plain-lead">
          <span className="jiaoban-spinner" aria-hidden="true" /> {progress}
        </p>
        {isNewSession ? (
          <p className="muted small-note">正在为这单活新建会话（约 1 分钟）…</p>
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
      <p className="muted small-note">想看每一步的过程，切到「工作流」tab。</p>
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
function JiaobanDoneState({
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
  const stepsDone = chain?.completed ?? countDoneNodes(chainStatus);
  const resultLine = chain
    ? `完成 ${chain.completed} 步${chain.stopped_reason ? `；中途停了：${chain.stopped_reason}` : ""}。`
    : outcome?.message || "做完了。";
  const proof = summarizeProof(chainStatus);
  // fix3 后端新 warnings（如「角色已按 codex-dev 执行」「已接续上次中断的运行」）→ 小字列出，不挡主路径。
  const warnings = chain?.warnings ?? [];

  return (
    <div className="project-canvas-detail-card jiaoban-done" aria-label="做好了">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3 className="jiaoban-done-title">
            {needsRework ? "这一步需要重做" : jiaobanDoneTitle(chain?.steps ?? [])}
          </h3>
        </div>
        <Badge tone={needsRework ? "warning" : "candidate"}>{needsRework ? "待你决定" : "已交货"}</Badge>
      </div>
      <div className="role-loop-plain" aria-label="结果（人话）">
        <p className="role-loop-plain-lead">{resultLine}</p>
        {stepsDone > 0 ? <p className="role-loop-plain-note">这次做完 {stepsDone} 步。</p> : null}
      </div>
      <JiaobanStepReportList
        steps={chain?.steps ?? []}
        onConfirmFact={onConfirmFact}
        confirmedTaskIds={confirmedTaskIds}
      />
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">产出：</span>
        {proof ?? "详情见工作流 tab。"}
      </p>
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

// 主管已明确退回时，这里是主界面的用户处置入口；四个动作都要可见，绝不自动重跑。
export function JiaobanNeedsReworkDisposal({
  reason,
  actionsReady,
  starting,
  error,
  onContinue,
  onAction,
}: {
  reason: string;
  actionsReady: boolean;
  starting: boolean;
  error: string | null;
  onContinue: () => void;
  onAction: (action: "change_session" | "rework" | "archive") => void;
}) {
  return (
    <div className="role-loop-plain" aria-label="主管退回处置">
      <p className="role-loop-plain-lead">主管退回理由：{reason}</p>
      {error ? <p className="jiaoban-consult-error" role="alert">{error}</p> : null}
      {!actionsReady ? <p className="muted small-note">重做信息还在载入，三个处置按钮会稍后可用。</p> : null}
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" disabled={starting} onClick={onContinue}>
          接着跑（按原样重做）
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("change_session")}
        >
          换个新会话重做
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("rework")}
        >
          退回主管重拆
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("archive")}
        >
          结束这单
        </button>
      </div>
    </div>
  );
}

// 停因→动作 死配对（§2.2）。给定 outcome / error / 方案是否已确认，判这张卡住脸该主打哪个按钮。
// 铁律：绝不返回零按钮（fallback 双给[接着跑]+[重新说目标]）。
type BlockedPlan = {
  // 主按钮语义：continue=[接着跑,不用重批]，replan=[重新说目标出新方案]，session=先选一条会话（选完再[接着跑]）。
  primary: "continue" | "replan" | "session";
  // 是否同时给次按钮（另一条出路）。
  showReplanSecondary: boolean;
  showContinueSecondary: boolean;
  // 主按钮上方的一句提示（如「上一步失败了，接着跑会从拆任务重来」）。
  note: string | null;
};

// 关键词特征判定（都在前端 message/stop_reason 里找人话词，不碰后端）。
// export 供离线 DOM 断言测试直接喂各类停因验按钮（行为中性，不改运行时）。
export function classifyBlocked(
  outcome: AutoAdvanceRoleLoopOutcome | null,
  error: string | null,
  planIsConfirmed: boolean,
): BlockedPlan {
  const stage = outcome?.stage ?? "";
  const text = `${outcome?.stop_reason ?? ""} ${outcome?.message ?? ""} ${error ?? ""}`;

  // 1) needs_binding / 会话类 → 先选一条会话，选完回[接着跑]。
  const needsBinding =
    stage === "needs_binding" ||
    (outcome?.needs_binding_count ?? 0) > 0 ||
    /没.{0,3}会话|选.{0,3}会话|绑.{0,4}会话|哪个对话|接现有|会话.{0,3}(缺|没|未)/.test(text);
  if (needsBinding) {
    // 会话类：选会话是正路；但方案已确认时，选完就能[接着跑]，故次按钮给 continue。
    return {
      primary: "session",
      showReplanSecondary: true,
      showContinueSecondary: planIsConfirmed,
      note: null,
    };
  }

  // 2) blocked·写范围 / 方案内容类（message 含「方案缺 / 写范围 / 重新让 AI 出方案」）→ 重新说目标。
  //    这类是方案本身不够（少写范围、内容缺），接着跑也过不去，得回去出新方案。
  const planContentIssue = /方案.{0,3}(缺|不全|不够|有问题|没.{0,2}写)|写范围|可写|允许.{0,2}改|重新.{0,4}方案|重新让.{0,3}出/.test(
    text,
  );
  if (planContentIssue) {
    return {
      primary: "replan",
      showReplanSecondary: false,
      // 方案已确认时仍给个[接着跑]次口（万一只是复核噪音，接着跑能过）。
      showContinueSecondary: planIsConfirmed,
      note: null,
    };
  }

  // 3) startError / 拆任务失败 / 超时 / flaky → [接着跑]（注明「上一步失败了，接着跑会从拆任务重来」）。
  const transientFailure =
    /拆任务|超时|timeout|timed out|失败|重试|中断|flaky|临时|偶发|网络|连接/i.test(text) ||
    (!!error && !planContentIssue); // startError 走到这（非方案内容类）多为合流/推进途中失败
  if (transientFailure && planIsConfirmed) {
    return {
      primary: "continue",
      showReplanSecondary: true,
      showContinueSecondary: false,
      note: "上一步失败了，接着跑会从拆任务重来。",
    };
  }

  // 4) blocked·其它（含「角色」类·fix3 后端钳位后应基本消失）→ [接着跑]主、重新说目标次。
  if (planIsConfirmed && (stage === "blocked" || stage === "no_dispatchable" || !!outcome)) {
    return {
      primary: "continue",
      showReplanSecondary: true,
      showContinueSecondary: false,
      note: null,
    };
  }

  // 5) 兜底（识别不了 / 方案未确认无法接着跑）→ 至少给[重新说目标]；方案已确认再补[接着跑]。
  //    绝不零按钮。
  return {
    primary: "replan",
    showReplanSecondary: false,
    showContinueSecondary: planIsConfirmed,
    note: null,
  };
}

// 5. 卡住（永不冻）：按停因死配对给「能点的正确按钮」。绝不零按钮终态。
// export 供离线 DOM 断言测试直接挂载验各分支按钮（行为中性，不改运行时）。
export function JiaobanBlockedState({
  outcome,
  error,
  planIsConfirmed,
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  onContinueRun,
  onRePlan,
  starting,
  onOpenWorkflow,
  latestSessionThreadId,
  onOpenAgentSession,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  error: string | null;
  planIsConfirmed: boolean;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  onContinueRun: () => void;
  onRePlan: () => void;
  starting: boolean;
  onOpenWorkflow: (() => void) | null;
  // 「看原始对话」桥：面级入口（卡住脸不放 picker 行内入口·防一脸双入口）。existing 单→看原始对话；
  // 哨兵单→latestSession 兜底看最近对话；皆无→不显。
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
}) {
  // 停因人话：直接用后端 message / stop_reason（已带具体原因，不包糊话盖住）；再兜底一句 error。
  const reason =
    outcome?.stop_reason?.trim() ||
    outcome?.message?.trim() ||
    error?.trim() ||
    "碰到拿不准的地方，先停下了。";

  const plan = classifyBlocked(outcome, error, planIsConfirmed);
  const warnings = outcome?.chain_outcome?.warnings ?? [];

  // 主/次按钮拼装。continue 主按钮统一文案「接着跑（方案已批过，不用重批）」。
  const continueBtn = (isPrimary: boolean) => (
    <button
      key="continue"
      className={isPrimary ? "primary-button" : "secondary-button"}
      type="button"
      disabled={starting}
      onClick={onContinueRun}
    >
      {starting ? "正在开始…" : "接着跑（方案已批过，不用重批）"}
    </button>
  );
  const replanBtn = (isPrimary: boolean) => (
    <button
      key="replan"
      className={isPrimary ? "primary-button" : "secondary-button"}
      type="button"
      disabled={starting}
      onClick={onRePlan}
    >
      重新说目标出新方案
    </button>
  );

  return (
    <div className="project-canvas-detail-card jiaoban-blocked" aria-label="卡住了">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3 className="jiaoban-blocked-title">⚠ 卡住了</h3>
        </div>
        <Badge tone="warning">停下了</Badge>
      </div>
      <div className="role-loop-plain" aria-label="停下的原因（人话）">
        <p className="role-loop-plain-lead">{reason}</p>
        {plan.note ? <p className="role-loop-plain-note">{plan.note}</p> : null}
      </div>

      {/* 会话类：把选会话入口直接嵌进卡住脸——选完就能点下面[接着跑]。 */}
      {plan.primary === "session" ? (
        <JiaobanSessionPicker
          sessions={sessions}
          sessionChoice={sessionChoice}
          onSessionChoiceChange={onSessionChoiceChange}
        />
      ) : null}

      <div className="workflow-state-actions">
        {plan.primary === "continue" ? continueBtn(true) : null}
        {plan.primary === "replan" ? replanBtn(true) : null}
        {plan.primary === "session" ? (
          // 会话类主路径 = 上面选一条；这里的主按钮是选完[接着跑]（方案已确认时可点，否则引导重新说目标）。
          planIsConfirmed ? (
            continueBtn(true)
          ) : (
            replanBtn(true)
          )
        ) : null}
        {plan.showContinueSecondary && plan.primary !== "continue" ? continueBtn(false) : null}
        {plan.showReplanSecondary && plan.primary !== "replan" ? replanBtn(false) : null}
        {onOpenWorkflow ? (
          <button className="secondary-button" type="button" onClick={onOpenWorkflow}>
            去工作流看看
          </button>
        ) : null}
      </div>

      {/* 面级「看原始对话」：卡了想看它到底干了啥的天然下钻路（picker 行内入口不放·防一脸双入口）。 */}
      <JiaobanRawSessionLink
        sessionChoice={sessionChoice}
        latestSessionThreadId={latestSessionThreadId}
        onOpenAgentSession={onOpenAgentSession}
      />

      {/* fix3 后端新 warnings（如「角色已按 codex-dev 执行」）→ 小字列出，不挡主路径。 */}
      {warnings.length > 0 ? (
        <ul className="jiaoban-warnings muted small-note" aria-label="附带说明">
          {warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      ) : null}

      <p className="muted small-note">卡了总给下一步，不会停在死路。</p>
    </div>
  );
}

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

// 链状态 → 「正在…第 x/y 步」。链事件还没出现的阶段（拿不到节点）= 主管还在拆任务，据实说清可能很久。
function humanizeChainProgress(chainStatus: ProjectWorkflowChainStatus | null): string {
  if (!chainStatus || chainStatus.nodes.length === 0) {
    return "主管正在拆任务…（最长可能十几分钟，偶尔自动重试）";
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

// 产出概要：能从链状态拿到多少步做完就说多少；拿不到细节明确写「详情见工作流 tab」。
function summarizeProof(chainStatus: ProjectWorkflowChainStatus | null): string | null {
  if (!chainStatus || chainStatus.nodes.length === 0) return null;
  const done = countDoneNodes(chainStatus);
  const total = chainStatus.nodes.length;
  return `跑完 ${done}/${total} 步；详情见工作流 tab。`;
}
