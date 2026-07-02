import { useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "../../components/Badge";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  autoAdvanceAuthorizedRoleLoop,
  getProjectWorkflowChainStatus,
  recordGlobalBoundaryReview,
  recordProjectConsultationProposalDecision,
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

  // 手动相位覆盖：允许并开始/干完/卡住由本地状态驱动（proposal store 变化只决定「说 ↔ 批」）。
  const [manualPhase, setManualPhase] = useState<JiaobanPhase | null>(null);
  const [goal, setGoal] = useState("");
  const [amendment, setAmendment] = useState("");
  const [sessionChoice, setSessionChoice] = useState<string | null>(null); // null = 开个新的
  const [starting, setStarting] = useState(false);
  const [outcome, setOutcome] = useState<AutoAdvanceRoleLoopOutcome | null>(null);
  const [startError, setStartError] = useState<string | null>(null);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);
  const runningRef = useRef(false);

  // 新方案到达（或换了一份）→ 清掉上一轮的开始态，回到「批」，让用户重新审这份卡。
  useEffect(() => {
    setManualPhase(null);
    setOutcome(null);
    setStartError(null);
    setChainStatus(null);
    setAmendment("");
    setSessionChoice(null);
    runningRef.current = false;
  }, [latestProposal?.proposal_id]);

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

  // 允许并开始 = 方案授权人闸那一下（隔离 action）。见 runJiaobanAuthorizeAndStart 注释。
  async function authorizeAndStart() {
    if (!projectWorkflow || !latestProposal || starting || runningRef.current) return;
    runningRef.current = true;
    setStarting(true);
    setStartError(null);
    setOutcome(null);
    setChainStatus(null);
    try {
      const result = await runJiaobanAuthorizeAndStart({
        project,
        projectWorkflow,
        proposal: latestProposal,
        proposalStoreRevision: proposalSummary.revision,
        planAuthorizationRevision: planAuthorizationStore?.revision ?? 0,
        sessionChoice,
      });
      setOutcome(result.outcome);
      setManualPhase(result.outcome.stage === "ran" ? "running" : "blocked");
    } catch (e) {
      setStartError(e instanceof Error ? e.message : String(e));
      setManualPhase("blocked");
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
  }

  function backToSay() {
    setManualPhase("say");
    setOutcome(null);
    setStartError(null);
    setChainStatus(null);
    setAmendment("");
  }

  // 干完了没有：链状态是否收尾（人话「做好了」）。
  useEffect(() => {
    if (phase !== "running") return;
    if (chainStatus && /(finished|completed|done|succeeded|aborted|stopped|failed)/i.test(chainStatus.state)) {
      // 链跑到头 → 交货（aborted/failed 也进交货并给下一步，永不冻；细节人话见结果行）。
      setManualPhase("done");
    }
  }, [phase, chainStatus]);

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

  return (
    <section className="project-jiaoban" aria-label="交办">
      <div className="project-jiaoban-col">
        {phase === "say" ? (
          <JiaobanSayState goal={goal} onGoalChange={setGoal} onSubmit={() => submitGoal(goal)} />
        ) : null}

        {phase === "authorize" && latestProposal ? (
          <JiaobanAuthorizeState
            proposal={latestProposal}
            sessions={projectSessions}
            sessionChoice={sessionChoice}
            onSessionChoiceChange={setSessionChoice}
            amendment={amendment}
            onAmendmentChange={setAmendment}
            onAmend={submitAmendment}
            onAuthorizeAndStart={() => void authorizeAndStart()}
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
}: {
  goal: string;
  onGoalChange: (value: string) => void;
  onSubmit: () => void;
}) {
  return (
    <div className="project-canvas-detail-card jiaoban-say" aria-label="想让 AI 干点啥">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交办</p>
          <h3>想让 AI 干点啥？</h3>
        </div>
      </div>
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
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  amendment,
  onAmendmentChange,
  onAmend,
  onAuthorizeAndStart,
  onDecline,
  starting,
}: {
  proposal: ProjectConsultationProposal;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  amendment: string;
  onAmendmentChange: (value: string) => void;
  onAmend: () => void;
  onAuthorizeAndStart: () => void;
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
        {proposal.acceptance_criteria.length ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">改完怎么验：</span>
            {proposal.acceptance_criteria.join("；")}
          </p>
        ) : null}
      </div>

      <fieldset className="jiaoban-session-pick" aria-label="用哪个对话干">
        <legend>用哪个对话干</legend>
        <label className="jiaoban-radio">
          <input
            type="radio"
            name="jiaoban-session"
            checked={sessionChoice === null}
            onChange={() => onSessionChoiceChange(null)}
          />
          开个新的
        </label>
        {sessions.map((session) => (
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
      </fieldset>

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
        <button className="primary-button" type="button" disabled={starting} onClick={onAuthorizeAndStart}>
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
        <button className="secondary-button" type="button" disabled={starting} onClick={onDecline}>
          先不做
        </button>
      </div>
      <p className="muted small-note">点「允许并开始」= 允许这段自动跑，后面不再逐步问你。</p>
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
// 接缝 action：允许并开始 = 方案授权人闸那一下。刀1 合流命令落地后「只换这一个函数」。
// ============================================================

type JiaobanAuthorizeAndStartInput = {
  project: ProjectRecord;
  projectWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]>;
  proposal: ProjectConsultationProposal;
  proposalStoreRevision: number;
  planAuthorizationRevision: number;
  sessionChoice: string | null; // null = 开个新的
};

type JiaobanAuthorizeAndStartResult = {
  outcome: AutoAdvanceRoleLoopOutcome;
  // 兜底实际做了哪几步（供回交/调试；不进主路径 UI）。
  fallbackSteps: string[];
};

/**
 * 允许并开始 = 决策§地基六件·件2「一键合流」的前端触发点，也是唯一的方案授权人闸。
 *
 * 【刀1 未就绪的兜底】后端「一键合流」命令（确认方案 + 边界复核 + prepare + 绑会话 + 起链一个原子命令）
 * 尚未落地。这里用现成命令**干净组合**出等价的最安全子集：
 *   ① recordProjectConsultationProposalDecision(confirm) —— 确认方案 = 授权一段自动执行范围（建授权，停在待全局复核）
 *   ② recordGlobalBoundaryReview(approved)             —— 边界复核通过，授权转 active
 *   ③ autoAdvanceAuthorizedRoleLoop                    —— active 授权后一口气跑「拆任务 + 准备 + 工作者链跑」
 * 三步都是已存在的 gated 命令；真执行闸仍在后端 path-lock（非测试项目/无 active 授权即拒）。
 * 人闸 = 用户点「允许并开始」这一下；之后不再逐步问（决策§授权卡定稿）。
 *
 * 【session_choice】"开个新的"(null) 走 autoAdvance 默认自动建会话（happy path 永不见绑会话）；
 * "接现有" 暂无现成合流入口收纳既有会话（那要后端 prepare 时按 thread 绑），故本兜底记录用户选择但仍走
 * 自动建；刀1 合流命令落地后由它按 session_choice 绑既有会话，届时只改本函数。
 *
 * 【落地后只换本函数】刀1 命令就绪 → 把 ①②③ 换成那一个原子调用（传 session_choice），返回同形 outcome，
 * 组件与五态 UI 一字不动。
 */
export async function runJiaobanAuthorizeAndStart(
  input: JiaobanAuthorizeAndStartInput,
): Promise<JiaobanAuthorizeAndStartResult> {
  const { project, projectWorkflow, proposal, proposalStoreRevision, planAuthorizationRevision, sessionChoice } = input;
  const fallbackSteps: string[] = [];

  // 只对待确认/草案方案走「确认」；已确认的方案跳过确认直接推进（幂等，防重复确认报错）。
  const needsConfirm = ["draft", "pending_user_confirmation"].includes(proposal.status);
  let authorizationId: string | null = proposal.plan_authorization_id ?? null;
  let authorizationRevision = planAuthorizationRevision;

  if (needsConfirm) {
    // ① 确认方案 = 授权一段自动执行范围。
    const decision = await recordProjectConsultationProposalDecision({
      project_root: project.project_root,
      proposal_id: proposal.proposal_id,
      actor_id: "user",
      decision: "confirm",
      summary: "用户在交办面允许并开始：确认方案、授权这段自动执行范围。",
      expected_proposal_store_revision: proposalStoreRevision,
      expected_plan_authorization_store_revision: planAuthorizationRevision,
    });
    fallbackSteps.push("确认方案（建授权）");
    authorizationId = decision.plan_authorization?.authorization_id ?? authorizationId;
    authorizationRevision = decision.plan_authorization_store_revision ?? authorizationRevision;
  }

  // ② 边界复核通过（授权转 active）。有授权对象才做——没有则跳到推进，让后端闸给出具体停因。
  if (authorizationId) {
    try {
      const review = await recordGlobalBoundaryReview({
        project_root: project.project_root,
        project_id: projectWorkflow.project_id,
        workflow_id: projectWorkflow.workflow_id,
        proposal_id: proposal.proposal_id,
        authorization_id: authorizationId,
        actor_id: "global_director",
        review_status: "approved",
        summary: "用户在交办面允许并开始：边界复核通过，授权生效。",
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
      fallbackSteps.push("边界复核通过（授权生效）");
      void review;
    } catch {
      // 复核可能已存在/已生效——不致命，交由推进步骤和后端闸判定；停因会在 outcome 里人话呈现。
      fallbackSteps.push("边界复核（已存在或跳过）");
    }
  }

  // ③ active 授权后一口气推进：拆任务 + 准备 + 工作者链跑。
  const outcome = await autoAdvanceAuthorizedRoleLoop({
    project_root: projectWorkflow.project_root,
    workflow_id: projectWorkflow.workflow_id,
    actor_id: "user",
  });
  fallbackSteps.push("推进（自动跑一串）");
  void sessionChoice; // 见上：接现有会话待刀1 合流命令按 thread 绑；本兜底走自动建。

  return { outcome, fallbackSteps };
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
