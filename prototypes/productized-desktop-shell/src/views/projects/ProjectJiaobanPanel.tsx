import { useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "../../components/Badge";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  confirmAndStartAuthorizedRun,
  getProjectWorkflowChainStatus,
  stopProjectWorkflowChain,
} from "../../lib/tauri";
import type {
  AutoAdvanceRoleLoopOutcome,
  PendingAction,
  PlanAuthorizationStoreV1,
  ProjectConsultationProposal,
  ProjectConsultationProposalStoreV1,
  ProjectRecord,
  ProjectWorkflowChainStatus,
  SessionRecord,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { buildRunProjectConsultationAction } from "./ProjectWorkflowGovernancePanels";

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

// 结果防丢：换 tab 会卸载本面板（ProjectWorkspaceShell 条件渲染），本地 state 全丢。
// 故把「一轮开始的结果」按 project_root 缓存在模块级，重挂载时恢复——切走再回来结果还在。
// 只缓存呈现所需的最小集：手动相位 + outcome + 报错 + 上次停因（供重出方案预填）+ 这一轮批的方案 id。
type JiaobanRunCache = {
  manualPhase: JiaobanPhase | null;
  outcome: AutoAdvanceRoleLoopOutcome | null;
  startError: string | null;
  lastStopReason: string | null; // 卡住原因，重新出方案时带回「说」面
  ranProposalId: string | null; // 这一轮真按下[允许并开始]批的方案 id（区分「新方案到达」用）
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
  };
  jiaobanRunCacheByProject.set(projectRoot, { ...prev, ...patch });
}
function clearJiaobanRunCache(projectRoot: string) {
  jiaobanRunCacheByProject.delete(projectRoot);
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

function ProjectJiaobanPanelBrowser({
  project,
  sessions,
  workflowState,
  projectConsultationProposalStore,
  planAuthorizationStore,
  onRequestAction,
  onOpenAgentSession,
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
  const [amendment, setAmendment] = useState("");
  // 「说」面顶部的上次停因摘要（重新出方案时带过来；空则不显示）。
  const [sayHint, setSayHint] = useState<string | null>(null);
  // 「用哪个对话干」：默认选最近一条现有会话（下面 effect 里补默认；null 只在真无会话时保留）。
  const [sessionChoice, setSessionChoice] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [outcome, setOutcome] = useState<AutoAdvanceRoleLoopOutcome | null>(cached?.outcome ?? null);
  const [startError, setStartError] = useState<string | null>(cached?.startError ?? null);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);
  const runningRef = useRef(false);
  // 这一轮真按下[允许并开始]批的方案 id（从缓存恢复）。用来判「有没有来一份新方案」。
  const ranProposalIdRef = useRef<string | null>(cached?.ranProposalId ?? null);

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
      setSessionChoice(latest.thread_id);
      sessionDefaultedRef.current = true;
    }
  }, [projectSessions]);

  // 干活期间轮询进度（复用现成只读命令）。stage==ran 才轮。
  useEffect(() => {
    if (outcome?.stage !== "ran" || !projectWorkflow) return;
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
  }, [outcome?.stage, projectWorkflow]);

  // 当前该显哪张脸：手动相位优先；否则由「有没有方案」决定说/批。
  const phase: JiaobanPhase = manualPhase ?? (latestProposal ? "authorize" : "say");

  // ---- 动作 ----

  // 说 → 出方案：走确认弹层（真执行提示）+ App 侧调 runProjectConsultation（异步·1–2 分钟）。
  function submitGoal(text: string) {
    if (!projectWorkflow || !text.trim()) return;
    onRequestAction(buildRunProjectConsultationAction(project, projectWorkflow, text.trim()));
  }

  // 按我说的改：原目标 + 这句意见拼成新目标，重调 runProjectConsultation（新方案仍在本卡出）。
  function submitAmendment() {
    if (!projectWorkflow || !amendment.trim() || !latestProposal) return;
    const merged = `${latestProposal.user_goal}\n\n补充意见：${amendment.trim()}`;
    onRequestAction(buildRunProjectConsultationAction(project, projectWorkflow, merged));
  }

  // 允许并开始 = 方案授权人闸那一下。走刀1 合流命令 confirm_and_start_authorized_run（见 lib/tauri）。
  async function authorizeAndStart() {
    if (!projectWorkflow || !latestProposal || starting || runningRef.current) return;
    // ★ 一进来立刻上「正在干」脸——不等 await 回来（await 可能几十秒~几分钟，中间不能无脸看着像冻死）。
    runningRef.current = true;
    ranProposalIdRef.current = latestProposal.proposal_id;
    setStarting(true);
    setStartError(null);
    setOutcome(null);
    setChainStatus(null);
    setManualPhase("running");
    writeJiaobanRunCache(projectRoot, {
      manualPhase: "running",
      outcome: null,
      startError: null,
      ranProposalId: latestProposal.proposal_id,
    });
    try {
      // 合流命令只支持 existing（绑现有会话）；"开个新的"(null) 下一阶段接，这里当作没选会话。
      const outcome = await confirmAndStartAuthorizedRun({
        project_root: projectRoot,
        proposal_id: latestProposal.proposal_id,
        session_choice: sessionChoice ? "existing" : "new",
        session_id: sessionChoice ?? undefined,
        actor_id: "user",
      });
      const nextPhase: JiaobanPhase = outcome.stage === "ran" ? "running" : "blocked";
      setOutcome(outcome);
      setManualPhase(nextPhase);
      writeJiaobanRunCache(projectRoot, { manualPhase: nextPhase, outcome, startError: null });
    } catch (e) {
      // 合流命令对「已确认/旧方案」会干净拒（后端：方案不是待用户确认状态）→ 翻成人话 + 引导重新说目标。
      const humanized = humanizeAuthorizeError(e);
      setStartError(humanized);
      setManualPhase("blocked");
      writeJiaobanRunCache(projectRoot, { manualPhase: "blocked", startError: humanized, lastStopReason: humanized });
    } finally {
      setStarting(false);
      runningRef.current = false;
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
    setChainStatus(null);
    setAmendment("");
    ranProposalIdRef.current = null;
    clearJiaobanRunCache(projectRoot);
  }

  // 干完了没有：链状态是否收尾（人话「做好了」）。
  useEffect(() => {
    if (phase !== "running") return;
    if (chainStatus && /(finished|completed|done|succeeded|aborted|stopped|failed)/i.test(chainStatus.state)) {
      // 链跑到头 → 交货（aborted/failed 也进交货并给下一步，永不冻；细节人话见结果行）。
      setManualPhase("done");
      writeJiaobanRunCache(projectRoot, { manualPhase: "done" });
    }
  }, [phase, chainStatus, projectRoot]);

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

  return (
    <section className="project-jiaoban" aria-label="交办">
      <div className="project-jiaoban-col">
        {phase === "say" ? (
          <JiaobanSayState
            goal={goal}
            onGoalChange={setGoal}
            onSubmit={() => submitGoal(goal)}
            lastStopHint={sayHint}
          />
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
          />
        ) : null}

        {phase === "running" ? (
          <JiaobanRunningState chainStatus={chainStatus} onStop={() => void stopRun()} />
        ) : null}

        {phase === "done" ? (
          <JiaobanDoneState outcome={outcome} chainStatus={chainStatus} onContinue={backToSay} />
        ) : null}

        {phase === "blocked" ? (
          <JiaobanBlockedState
            outcome={outcome}
            error={startError}
            onRePlan={backToSay}
            // 「去工作流看看」需切 tab（onSelectTool 在外壳），本包红线「不动外壳」→ 不在此接线；
            // 保留入口能力（prop 已在），置 null 即不渲染该次按钮，主按钮「重新出方案」永在。
            onOpenWorkflow={null}
          />
        ) : null}

        {/* 刀2 占位：复杂活的工序图稍后加（本包不实现）。 */}
        <p className="muted small-note jiaoban-worksmap-placeholder">复杂活的工序图稍后加。</p>
      </div>
    </section>
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
}: {
  goal: string;
  onGoalChange: (value: string) => void;
  onSubmit: () => void;
  lastStopHint: string | null;
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
      <label className="proposal-decision-field">
        <span>说一句话，AI 会读项目、想个方案给你审。</span>
        <textarea
          value={goal}
          onChange={(event) => onGoalChange(event.target.value)}
          placeholder="例：给这小游戏加个计分板——吃到东西 +1、显示在右上角。"
          rows={4}
        />
      </label>
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" disabled={!goal.trim()} onClick={onSubmit}>
          出方案
        </button>
      </div>
      <p className="muted small-note">AI 读项目、想方案要花点时间（大约 1–2 分钟），这期间界面不会卡。</p>
    </div>
  );
}

// 2. 批（授权卡·定稿字段）
function JiaobanAuthorizeState({
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
}) {
  const targetFiles = extractTargetFiles(proposal.proposed_steps);
  const willWrite = proposal.scope_draft.allowed_write_roots.length > 0;

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
          <span aria-hidden="true">⚠</span> 这是 {proposalAgeDays} 天前的旧方案（生成于 {proposalTimeText}
          ）。项目可能已经变了，建议重新说一遍目标、出一版新的。
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

      <JiaobanSessionPicker
        sessions={sessions}
        sessionChoice={sessionChoice}
        onSessionChoiceChange={onSessionChoiceChange}
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
        />
      </label>

      <div className="workflow-state-actions">
        {proposalIsStale ? (
          // 旧方案：主按钮 = 重新说目标；[允许并开始] 降为次按钮（防再批库存），但仍可手动点。
          <>
            <button className="primary-button" type="button" disabled={starting} onClick={onRePlan}>
              重新说目标出新方案
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting}
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
              disabled={starting}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "允许并开始"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || !amendment.trim()}
              onClick={onAmend}
            >
              按我说的改
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

// 会话收纳：默认收起一行「用哪个对话干：接现有 · <最近一条标题> ▾」，点开才展开选择。
// 展开后：最近 5 条直列 + 其余折叠/可搜；「开个新的」置灰标「下一阶段支持」（用户已拍方案 a 下阶段）。
function JiaobanSessionPicker({
  sessions,
  sessionChoice,
  onSessionChoiceChange,
}: {
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
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
  const summaryTitle = selected?.title || selected?.thread_id || sorted[0]?.title || "选一条对话";

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
      <button
        type="button"
        className="jiaoban-session-summary"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
      >
        <span className="jiaoban-field-label">用哪个对话干：</span>
        <span className="jiaoban-session-summary-value">接现有 · {summaryTitle}</span>
        <span aria-hidden="true" className="jiaoban-session-caret">
          {open ? "▴" : "▾"}
        </span>
      </button>

      {open ? (
        <div className="jiaoban-session-expand">
          {/* 「开个新的」下一阶段支持（用户已拍方案 a 下阶段）→ 置灰不可选。 */}
          <label className="jiaoban-radio jiaoban-radio-disabled" aria-disabled="true">
            <input type="radio" name="jiaoban-session" disabled checked={false} readOnly />
            开个新的 <span className="jiaoban-soon">下一阶段支持</span>
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
  onStop,
}: {
  chainStatus: ProjectWorkflowChainStatus | null;
  onStop: () => void;
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
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onStop}>
          停下
        </button>
      </div>
      <p className="muted small-note">想看每一步的过程，切到「工作流」tab。</p>
    </div>
  );
}

// 4. 交货
function JiaobanDoneState({
  outcome,
  chainStatus,
  onContinue,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  chainStatus: ProjectWorkflowChainStatus | null;
  onContinue: () => void;
}) {
  const chain = outcome?.chain_outcome ?? null;
  const stepsDone = chain?.completed ?? countDoneNodes(chainStatus);
  const resultLine = chain
    ? `完成 ${chain.completed} 步${chain.stopped_reason ? `；中途停了：${chain.stopped_reason}` : ""}。`
    : outcome?.message || "做完了。";
  const proof = summarizeProof(chainStatus);

  return (
    <div className="project-canvas-detail-card jiaoban-done" aria-label="做好了">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3 className="jiaoban-done-title">✓ 做好了</h3>
        </div>
        <Badge tone="candidate">已交货</Badge>
      </div>
      <div className="role-loop-plain" aria-label="结果（人话）">
        <p className="role-loop-plain-lead">{resultLine}</p>
        {stepsDone > 0 ? <p className="role-loop-plain-note">这次做完 {stepsDone} 步。</p> : null}
      </div>
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">产出：</span>
        {proof ?? "详情见工作流 tab。"}
      </p>
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" onClick={onContinue}>
          继续弄别的
        </button>
      </div>
    </div>
  );
}

// 5. 卡住（永不冻）
function JiaobanBlockedState({
  outcome,
  error,
  onRePlan,
  onOpenWorkflow,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  error: string | null;
  onRePlan: () => void;
  onOpenWorkflow: (() => void) | null;
}) {
  // 停因人话：直接用后端 message / stop_reason（已带具体原因，不包糊话盖住）；再兜底一句 error。
  const reason =
    outcome?.stop_reason?.trim() ||
    outcome?.message?.trim() ||
    error?.trim() ||
    "碰到拿不准的地方，先停下了。";
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
      </div>
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" onClick={onRePlan}>
          重新出方案
        </button>
        {onOpenWorkflow ? (
          <button className="secondary-button" type="button" onClick={onOpenWorkflow}>
            去工作流看看
          </button>
        ) : null}
      </div>
      <p className="muted small-note">卡了总给下一步，不会停在死路。</p>
    </div>
  );
}

// ============================================================
// 接缝：允许并开始 = 方案授权人闸那一下 → 走刀1 合流命令 confirm_and_start_authorized_run
// （lib/tauri.ts 的 confirmAndStartAuthorizedRun）。后端一原子命令做完 确认方案 + 边界复核 +
// 授权生效 + 绑现有会话 + 自动推进；返回同形 outcome，组件按 stage 分支不变。人闸不省。
// ============================================================

// 合流命令的报错翻人话。最要紧的一类：对「已确认/旧方案」后端会拒（方案不是待用户确认状态）——
// 那不是系统坏了，是这份方案已经用过/过期，引导用户重新说目标出一版新的。
function humanizeAuthorizeError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  // 后端拒词：ProjectConsultationProposalStatus 不是 PendingUserConfirmation 时的那句。
  if (raw.includes("待用户确认") || raw.includes("PendingUserConfirmation") || raw.includes("不是「待")) {
    return "这份方案已经用过或已过期（不再是待确认状态），没法再从它开始。点「重新出方案」，说一遍目标出一版新的。";
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

// 链状态 → 「正在…第 x/y 步」。拿不到就给个中性进行时。
function humanizeChainProgress(chainStatus: ProjectWorkflowChainStatus | null): string {
  if (!chainStatus || chainStatus.nodes.length === 0) return "AI 正在动手…";
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
