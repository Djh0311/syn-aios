import { useEffect, useState } from "react";
import { Badge } from "../../components/Badge";
import {
  globalBoundaryReviewStatusLabels,
  summarizeAutoDispatchGuardResult,
  summarizeGlobalBoundaryReview,
  summarizePlanAuthorizationStore,
} from "../../lib/planAuthorization";
import {
  projectDirectorPlannedTaskStatusLabels,
  summarizeProjectDirectorTaskPlan,
} from "../../lib/projectDirectorTaskPlan";
import {
  summarizeProjectConsultationProposalStore,
} from "../../lib/projectConsultationProposal";
import type {
  AutoDispatchGuardResult,
  GlobalBoundaryReviewStatus,
  PendingAction,
  PreviewProjectDirectorTaskPlanInput,
  ProjectConsultationProposal,
  ProjectConsultationProposalDecisionKind,
  ProjectDirectorTaskPlan,
  DirectorChainOutcome,
  ProjectWorkflowChainStatus,
  AutoAdvanceRoleLoopOutcome,
  ProjectRecord,
  TaskDraftSummary,
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  autoAdvanceAuthorizedRoleLoop,
  getProjectWorkflowChainStatus,
  startProjectDirectorChain,
  stopProjectWorkflowChain,
} from "../../lib/tauri";
import { DetailLine } from "./projectWorkflowLabels";
import { HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE } from "./jiaoban/JiaobanConversation";

// 固定测试项目（自动干只在这真跑）。与 ProjectJiaobanPanel.tsx / WorkflowCommandConsoleView.tsx 同值同常量名
// （历史留下的按文件各自一份，非本次新引入的判据——P1-E 复用既有形态，不新造判断）。
const TEST_PROJECT_ROOT = "/Users/yoyi/codex-workflow-mario-test";

export function ProjectDirectorTaskPlanCard({
  project,
  request,
  plan,
  loading,
  error,
  workflowRevision,
  onPreview,
  onRequestAction,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput | null;
  plan: ProjectDirectorTaskPlan | null;
  loading: boolean;
  error: string | null;
  workflowRevision: number | null;
  onPreview: () => void;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeProjectDirectorTaskPlan(plan);
  const prepareBlockedReason = projectDirectorPrepareBlockedReason(request, plan, loading, error);
  const canPrepare = !prepareBlockedReason && request && plan;
  const previewDisabled = !request || loading;

  return (
    <section className="project-canvas-detail-card" aria-label="项目主管拆任务与准备派发">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目主管拆任务</p>
          <h3>{projectDirectorTaskPlanHeadline(request, plan, loading, error)}</h3>
        </div>
        <Badge tone={projectDirectorTaskPlanTone(plan, loading, error)}>{summary.status_label}</Badge>
      </div>
      {/* 裁掉数字栅格（已计划 / 已准备 / 待绑定 / 已阻断 / 记忆快照）——只动展示层；标题/状态/按钮保留。 */}
      {error ? <p className="state-warning">拆任务草案读取失败：{error}</p> : null}
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="workflow-draft-grid">
          <DetailLine label="生效授权" value={summary.active_authorization_id ?? request?.authorization_id ?? "暂无"} />
        </div>
      </details>
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {plan ? (
        <div className="workflow-compact-list" aria-label="项目主管计划任务摘要">
          {plan.planned_tasks.slice(0, 3).map((task) => (
            <div className="workflow-compact-item" key={task.planned_task_id}>
              <strong>{task.title}</strong>
              <span>{projectDirectorPlannedTaskStatusLabels[task.status] ?? task.status}</span>
              <em>{task.blocked_reasons.slice(0, 2).join("；") || `${task.scope.target_role} / ${task.scope.task_package_kind}`}</em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">生成拆任务草案后才会显示工作者子任务摘要。</p>
      )}
      {!request && !loading ? (
        <p className="state-warning">
          还不能「生成拆任务草案」：需先在上方完成 ① 确认方案范围 → ② 全局边界复核通过（出现生效授权）后再回到这里。
        </p>
      ) : null}
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" disabled={previewDisabled} onClick={onPreview}>
          {loading ? "正在生成" : "生成拆任务草案"}
        </button>
        <button
          className="primary-button"
          type="button"
          disabled={!canPrepare}
          onClick={() => {
            if (request && plan) {
              onRequestAction(
                buildPrepareAuthorizedAutoDispatchAction({
                  project,
                  request,
                  plan,
                  workflowRevision,
                }),
              );
            }
          }}
        >
          准备授权范围内派发
        </button>
      </div>
      {prepareBlockedReason ? <p className="state-warning">{prepareBlockedReason}</p> : null}
      {/* C1 手动挡（override）：按计划开干整条主管链。hooks 抽进带 typeof window 守卫的子组件，
          否则离线 findButtonByText 平铺调用本卡会撞 useState（无 dispatcher）——backend 线挂起件漏了这点。 */}
      <DirectorChainRunButton plan={plan} />
    </section>
  );
}

// C1 主管链运行按钮（手动挡/override）：含 useState/useEffect/真起链命令。
// typeof window 守卫放 hooks 之前——离线测试以普通函数平铺调用组件（findButtonByText/renderComposite·无 React dispatcher），
// 守卫在服务端/离线先返回 null、不触任何 hook；浏览器侧才进 hooks。（同 ProjectWorkflowReactFlowCanvas 套路。）
function DirectorChainRunButton({ plan }: { plan: ProjectDirectorTaskPlan | null }) {
  if (typeof window === "undefined") return null;
  return <DirectorChainRunButtonBrowser plan={plan} />;
}

function DirectorChainRunButtonBrowser({ plan }: { plan: ProjectDirectorTaskPlan | null }) {
  const [chainRunning, setChainRunning] = useState(false);
  const [chainOutcome, setChainOutcome] = useState<DirectorChainOutcome | null>(null);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);
  const [chainError, setChainError] = useState<string | null>(null);
  // 钉死：只用 status==prepared 那份（传 preview 那份后端 B1 filter 会全跳成空链）。
  const preparedTasks = plan ? plan.planned_tasks.filter((task) => task.status === "prepared") : [];
  const canRunChain =
    !!plan && plan.prepared_dispatch_count > 0 && preparedTasks.length > 0 && !chainRunning;

  // 链跑期间轮询运行态（复用 #19 只读命令；主管链记录同种结构）。
  useEffect(() => {
    if (!chainRunning || !plan) return;
    const projectRoot = plan.project_root;
    const workflowId = plan.workflow_id;
    let active = true;
    const poll = async () => {
      try {
        const status = await getProjectWorkflowChainStatus(projectRoot, workflowId);
        if (active && status) setChainStatus(status);
      } catch {
        // 轮询失败不致命
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [chainRunning, plan]);

  async function runDirectorChain() {
    if (!plan) return;
    // 现取 prepared 那份（防闭包拿到旧值），空则不发——绝不把 preview 计划送去空跑。
    const tasks = plan.planned_tasks.filter((task) => task.status === "prepared");
    if (tasks.length === 0) return;
    setChainRunning(true);
    setChainError(null);
    setChainOutcome(null);
    setChainStatus(null);
    try {
      const outcome = await startProjectDirectorChain({
        project_root: plan.project_root,
        workflow_id: plan.workflow_id,
        planned_tasks: tasks,
      });
      setChainOutcome(outcome);
    } catch (chainStartError) {
      setChainError(
        chainStartError instanceof Error ? chainStartError.message : String(chainStartError),
      );
    } finally {
      setChainRunning(false);
    }
  }

  async function stopDirectorChain() {
    if (!plan) return;
    try {
      await stopProjectWorkflowChain({
        project_root: plan.project_root,
        workflow_id: plan.workflow_id,
      });
    } catch (chainStopError) {
      setChainError(
        chainStopError instanceof Error ? chainStopError.message : String(chainStopError),
      );
    }
  }

  return (
    <>
      <div className="workflow-state-actions">
        <button
          className="primary-button"
          type="button"
          disabled={!canRunChain}
          onClick={() => void runDirectorChain()}
        >
          {chainRunning ? "主管链运行中…" : "按计划开干（整条主管链）"}
        </button>
        {chainRunning ? (
          <button className="secondary-button" type="button" onClick={() => void stopDirectorChain()}>
            停链
          </button>
        ) : null}
      </div>
      <p className="muted small-note">
        ⚠️ 在固定测试项目真起 Codex 链（真执行·非预览）：按已准备任务依赖序逐个真跑工作者；非测试项目被后端闸拒。
      </p>
      {plan && plan.prepared_dispatch_count === 0 && !chainRunning ? (
        <p className="muted small-note">先「准备授权范围内派发」并刷新拆任务草案（出现已准备任务）后才能开干。</p>
      ) : null}
      {chainStatus ? (
        <div className="workflow-compact-list" aria-label="主管链进度">
          <div className="workflow-compact-item">
            <strong>链状态</strong>
            <span>{chainStatus.state}</span>
          </div>
          {chainStatus.nodes.map((node) => (
            <div className="workflow-compact-item" key={node.node_id}>
              <strong>{node.node_id}</strong>
              <span>{node.state}</span>
            </div>
          ))}
        </div>
      ) : null}
      {chainOutcome ? (
        <p className="muted small-note">
          主管链结束：completed {chainOutcome.completed} / dispatched {chainOutcome.dispatched} / skipped{" "}
          {chainOutcome.skipped}
          {chainOutcome.stopped_reason ? `；停因 ${chainOutcome.stopped_reason}` : "；全跑完"}
        </p>
      ) : null}
      {chainError ? <p className="state-warning">主管链失败：{chainError}</p> : null}
    </>
  );
}

// 件 D 核心 · 一键自动推进（步骤塌缩）：方案授权生效后出现，一下把 拆任务→prepare→worker 链跑 串完，
// 取代手点那 3 步。前端只造请求 + 发，闸在后端 path-lock（无 active 授权 / 非测试项目后端拒）。
// hooks 进 typeof window 守卫子组件（离线平铺调用安全·同 DirectorChainRunButton）。
export function AutoAdvanceRoleLoopButton({
  project,
  request,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput | null;
}) {
  if (typeof window === "undefined") return null;
  // 只在已授权（active 授权 + 边界复核通过 → request 非空）后才出现；没授权时不渲染（人闸前不给一键跑）。
  if (!request) return null;
  return <AutoAdvanceRoleLoopButtonBrowser project={project} request={request} />;
}

function AutoAdvanceRoleLoopButtonBrowser({
  project,
  request,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput;
}) {
  void project;
  const [running, setRunning] = useState(false);
  const [outcome, setOutcome] = useState<AutoAdvanceRoleLoopOutcome | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [chainStatus, setChainStatus] = useState<ProjectWorkflowChainStatus | null>(null);

  // 链完整完成后补读最终进度，复用现成只读命令（与 C1 同种链记录）。
  useEffect(() => {
    if (outcome?.stage !== "completed") return;
    let active = true;
    const poll = async () => {
      try {
        const status = await getProjectWorkflowChainStatus(request.project_root, request.workflow_id);
        if (active && status) setChainStatus(status);
      } catch {
        // 轮询失败不致命
      }
    };
    void poll();
    const id = setInterval(() => void poll(), 2500);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, [outcome?.stage, request.project_root, request.workflow_id]);

  async function runAutoAdvance() {
    setRunning(true);
    setError(null);
    setOutcome(null);
    setChainStatus(null);
    try {
      const result = await autoAdvanceAuthorizedRoleLoop({
        project_root: request.project_root,
        workflow_id: request.workflow_id,
        actor_id: "user",
      });
      setOutcome(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setRunning(false);
    }
  }

  return (
    <section className="project-canvas-detail-card role-loop-auto-advance" aria-label="一键自动推进">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">一键自动推进</p>
          <h3>授权范围内自动跑：拆任务 → 准备 → 工作者链跑</h3>
        </div>
        <Badge tone={outcome?.stage === "completed" ? "candidate" : outcome ? "warning" : "unknown"}>
          {outcome ? autoAdvanceStageLabel(outcome.stage) : running ? "运行中" : "待运行"}
        </Badge>
      </div>
      <div className="role-loop-plain" aria-label="一键自动推进在做什么（人话）">
        <p className="role-loop-plain-lead">
          方案已授权 → 点这个，AI 自动把「拆任务 + 准备派发 + 工作者链跑」一口气做完，你不用再手点那 3 步。
        </p>
        <p className="role-loop-plain-note">
          ⚠️ 这是<strong>真执行·非预览</strong>：在固定测试项目真起 Codex 工作者链。碰到没绑会话 / 越界 / 失败会停下问你；非测试项目被后端闸拒。
        </p>
      </div>
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" disabled={running} onClick={() => void runAutoAdvance()}>
          {running ? "自动推进中…" : "▶▶ 一键自动推进"}
        </button>
      </div>
      {error ? <p className="state-warning">自动推进失败：{error}</p> : null}
      {outcome ? (
        <>
          <div className="workflow-draft-grid">
            <DetailLine label="阶段" value={autoAdvanceStageLabel(outcome.stage)} />
            <DetailLine label="拆出任务" value={String(outcome.planned_task_count)} />
            <DetailLine label="已准备" value={String(outcome.prepared_count)} />
            <DetailLine label="待绑会话" value={String(outcome.needs_binding_count)} />
            <DetailLine label="越界阻断" value={String(outcome.blocked_count)} />
          </div>
          <p className={outcome.stage === "completed" ? "muted small-note" : "state-warning"}>{outcome.message}</p>
          {outcome.stage === "needs_binding" ? (
            <p className="state-warning">
              差会话绑定：到下面「工作项执行 · 节点会话绑定」给 codex-dev 节点选一条已有 Codex 会话，再回来点一键自动推进。
            </p>
          ) : null}
          {outcome.stop_reason ? <p className="state-warning">停因：{outcome.stop_reason}</p> : null}
          {outcome.stage === "blocked" ||
          /写范围|范围为空/.test(`${outcome.message ?? ""} ${outcome.stop_reason ?? ""}`) ? (
            <button
              type="button"
              className="secondary-button auto-advance-refix-button"
              onClick={() => {
                // 纯前端锚点：直达同侧栏「方案与授权」的方案卡（阻断时那里可拒绝当前方案 → 重新说目标出方案）；
                // 说目标输入在场时顺手聚焦。不发任何后端调用。
                const card = document.getElementById("project-consultation-card");
                card?.scrollIntoView({ behavior: "smooth", block: "start" });
                (document.getElementById("project-consultation-goal-input") as HTMLTextAreaElement | null)?.focus();
              }}
            >
              ↻ 去重新出方案（说清读写范围）
            </button>
          ) : null}
          {outcome.chain_outcome ? (
            <p className="muted small-note">
              链：completed {outcome.chain_outcome.completed} / dispatched {outcome.chain_outcome.dispatched} / skipped{" "}
              {outcome.chain_outcome.skipped}
              {outcome.chain_outcome.stopped_reason ? `；停因 ${outcome.chain_outcome.stopped_reason}` : ""}
            </p>
          ) : null}
          {chainStatus ? (
            <div className="workflow-compact-list" aria-label="自动推进链进度">
              <div className="workflow-compact-item">
                <strong>链状态</strong>
                <span>{chainStatus.state}</span>
              </div>
              {chainStatus.nodes.map((node) => (
                <div className="workflow-compact-item" key={node.node_id}>
                  <strong>{node.node_id}</strong>
                  <span>{node.state}</span>
                </div>
              ))}
            </div>
          ) : null}
        </>
      ) : null}
      <p className="muted small-note">下面「拆任务草案 / 准备派发 / 按计划开干」是手动挡——想分步看 / 干预时用；日常走上面这一键即可。</p>
    </section>
  );
}

function autoAdvanceStageLabel(stage: string) {
  if (stage === "completed") return "已完整完成";
  if (stage === "interrupted") return "已停下·可接着跑";
  if (stage === "failed") return "执行失败";
  if (stage === "waiting_decision") return "待你决定";
  if (stage === "needs_binding") return "差会话绑定";
  if (stage === "blocked") return "越界阻断";
  if (stage === "no_dispatchable") return "无可派发";
  return stage;
}

function projectDirectorTaskPlanHeadline(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "等待用户确认方案和全局边界复核";
  if (loading) return "正在生成拆任务草案";
  if (error) return "拆任务草案未生成";
  if (!plan) return "尚未生成项目主管拆任务草案";
  if (plan.prepared_dispatch_count > 0) return "已准备；仍未执行工作者";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0) return "等待会话绑定后才能准备派发";
  return "授权范围内可准备";
}

function projectDirectorTaskPlanTone(plan: ProjectDirectorTaskPlan | null, loading: boolean, error: string | null) {
  if (error || plan?.blocked_count) return "warning";
  if (loading || !plan) return "unknown";
  if (plan.prepared_dispatch_count > 0 || plan.authorized_task_count > 0) return "candidate";
  return "unknown";
}

function projectDirectorPrepareBlockedReason(
  request: PreviewProjectDirectorTaskPlanInput | null,
  plan: ProjectDirectorTaskPlan | null,
  loading: boolean,
  error: string | null,
) {
  if (!request) return "缺少生效授权或已确认方案，不能准备派发。";
  if (loading) return "拆任务草案生成中，暂不能准备派发。";
  if (error) return "拆任务草案读取失败，暂不能准备派发。";
  if (!plan) return "请先生成拆任务草案。";
  if (plan.blocked_count > 0) return "越界任务已阻断";
  if (plan.needs_binding_count > 0)
    return "等待会话绑定：需先给 codex-dev 节点绑定一条已有 Codex 会话（到执行面板「节点会话绑定 · 选择已有 Codex 会话」选一条），再回来「准备授权范围内派发」。";
  if (plan.prepared_dispatch_count >= plan.planned_task_count && plan.planned_task_count > 0) {
    return "已准备；仍未执行工作者";
  }
  if (plan.planned_task_count === 0) return "没有可准备的工作者子任务。";
  return null;
}

export function buildPrepareAuthorizedAutoDispatchAction({
  project,
  request,
  plan,
  workflowRevision,
}: {
  project: ProjectRecord;
  request: PreviewProjectDirectorTaskPlanInput;
  plan: ProjectDirectorTaskPlan;
  workflowRevision: number | null;
}): PendingAction {
  return {
    kind: "prepare-authorized-auto-dispatch",
    label: "准备授权范围内派发",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "只创建准备派发记录、任务包草案和记忆快照；不启动工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    authorizedAutoDispatch: {
      project_root: request.project_root,
      project_id: request.project_id,
      workflow_id: request.workflow_id,
      proposal_id: request.proposal_id,
      authorization_id: request.authorization_id,
      actor_id: request.actor_id,
      planned_tasks: plan.planned_tasks,
      expected_workflow_revision: workflowRevision,
      expected_authorization_revision: request.expected_authorization_revision ?? null,
    },
    authorizedAutoDispatchPreview: plan,
  };
}

export function ProjectConsultationProposalCard({
  project,
  projectWorkflow,
  selectedTask,
  selectedTaskPackage,
  summary,
  planAuthorizationRevision,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  summary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationRevision: number;
  onRequestAction: (action: PendingAction) => void;
}) {
  const proposal = summary.latest_proposal;
  const [decisionSummary, setDecisionSummary] = useState("");
  // 件 D · 说目标：让 AI 真咨询出方案的输入（取代手填模板）。
  const [goal, setGoal] = useState("");
  // P1-E 诚实关门（用户拍板 a·不豁免站 3b）：非固定测试项目不再默认走塞纸条 fallback，本表单只出一句人话。
  const isTestProject = project.project_root === TEST_PROJECT_ROOT;

  useEffect(() => {
    setDecisionSummary("");
  }, [proposal?.proposal_id]);

  const canDecide = proposal && ["draft", "pending_user_confirmation"].includes(proposal.status);
  const defaultDecisionSummary =
    "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。";

  return (
    <section id="project-consultation-card" className="project-canvas-detail-card" aria-label="项目咨询方案草案">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">项目咨询方案草案</p>
        </div>
        <Badge tone={proposal?.status === "user_confirmed" ? "candidate" : proposal ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>

      {!proposal ? (
        <>
          {/* 件 D · 说目标 → AI 出方案（主路径）：真 codex 只读咨询出方案，取代手填模板。 */}
          {projectWorkflow && !isTestProject ? (
            <p className="role-loop-plain-note" aria-label="老实说明">
              {HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE}
            </p>
          ) : projectWorkflow ? (
            <div className="role-loop-consult-trigger">
              <label className="proposal-decision-field">
                <span>说目标 — 想让 AI 围绕什么出方案？</span>
                <textarea
                  value={goal}
                  onChange={(event) => setGoal(event.target.value)}
                  placeholder="例：把首页加载从 3 秒优化到 1 秒内，先让红队找瓶颈、再改。"
                />
              </label>
              <div className="workflow-state-actions">
                <button
                  className="primary-button"
                  type="button"
                  disabled={!goal.trim()}
                  onClick={() => onRequestAction(buildRunProjectConsultationAction(project, projectWorkflow, goal.trim()))}
                >
                  让 AI 出方案
                </button>
              </div>
            </div>
          ) : null}
        </>
      ) : (
        <>
          {/* 砍一波（从用户实际使用出发）：只留「想达成」给用户决策；会动到哪/分几步/必停点等细节栅格 + 步骤列表已砍。
              所有安全状态 state-warning（确认/缺授权回链等）原样保留，安全语义不弱化。 */}
          <div className="role-loop-plain" aria-label="这份方案在说什么（人话）">
            <p className="role-loop-plain-lead">想达成：{proposal.goal_summary}</p>
          </div>
          {summary.authorization_missing_after_confirmation ? (
            <p className="state-warning">方案已确认但缺少 C1 授权回链；不能显示为可自动推进。</p>
          ) : null}
          {proposal.status === "user_confirmed" ? (
            <p className="state-warning">已记录用户确认；仍需全局主管复核后才可自动推进。</p>
          ) : null}
          {/* 已确认方案没有决策/重新创建按钮（canDecide 只认 draft/pending，重新创建只认 rejected/changes_requested）。
              一旦确认→授权→自动推进阻断（如「授权写入范围为空」），用户在此本无路重新出方案。补一个「重新说目标出方案」入口，
              复用现成 AI 咨询动作（buildRunProjectConsultationAction）——不加新后端调用；自动推进卡的「去重新出方案」按钮滚到这。 */}
          {proposal.status === "user_confirmed" && projectWorkflow && !isTestProject ? (
            <p className="role-loop-plain-note" aria-label="老实说明">
              {HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE}
            </p>
          ) : proposal.status === "user_confirmed" && projectWorkflow ? (
            <div className="role-loop-consult-trigger" aria-label="重新说目标出方案">
              <label className="proposal-decision-field">
                <span>被阻断 / 想改方案？重新说目标让 AI 出新方案（会带上执行需要的读写范围）</span>
                <textarea
                  id="project-consultation-goal-input"
                  value={goal}
                  onChange={(event) => setGoal(event.target.value)}
                  placeholder="例：把首页加载优化到 1 秒内，允许改 src/ 下文件（把要写的目录说清）。"
                />
              </label>
              <div className="workflow-state-actions">
                <button
                  className="secondary-button"
                  type="button"
                  disabled={!goal.trim()}
                  onClick={() => onRequestAction(buildRunProjectConsultationAction(project, projectWorkflow, goal.trim()))}
                >
                  让 AI 重新出方案
                </button>
              </div>
            </div>
          ) : null}
          {(proposal.status === "rejected" || proposal.status === "changes_requested") && projectWorkflow ? (
            <div className="workflow-state-actions">
              <button
                className="secondary-button"
                type="button"
                onClick={() =>
                  onRequestAction(
                    buildProjectConsultationProposalCreationAction(
                      project,
                      projectWorkflow,
                      selectedTask,
                      selectedTaskPackage,
                      summary.revision,
                    ),
                  )
                }
              >
                重新创建方案草案
              </button>
            </div>
          ) : null}
          {canDecide ? (
            <>
              <label className="proposal-decision-field">
                <span>修改 / 拒绝原因</span>
                <textarea
                  value={decisionSummary}
                  onChange={(event) => setDecisionSummary(event.target.value)}
                  placeholder="要求修改或拒绝时填写原因；确认方案可留空。"
                />
              </label>
              <div className="workflow-state-actions">
                <button
                  className="primary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "confirm",
                        summary: defaultDecisionSummary,
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  确认方案范围
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "request_changes",
                        summary: decisionSummary.trim() || "用户要求修改项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  要求修改
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() =>
                    onRequestAction(
                      buildProjectConsultationProposalDecisionAction({
                        project,
                        proposal,
                        decision: "reject",
                        summary: decisionSummary.trim() || "用户拒绝当前项目咨询方案草案。",
                        proposalStoreRevision: summary.revision,
                        planAuthorizationRevision,
                      }),
                    )
                  }
                >
                  拒绝方案
                </button>
              </div>
            </>
          ) : null}
        </>
      )}
    </section>
  );
}

export function GlobalBoundaryReviewCard({
  project,
  projectWorkflow,
  proposalSummary,
  planAuthorizationSummary,
  guardResult,
  guardError,
  onRequestAction,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  proposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const summary = summarizeGlobalBoundaryReview(proposalSummary, planAuthorizationSummary, guardResult);
  const [reviewSummary, setReviewSummary] = useState("");

  useEffect(() => {
    setReviewSummary("");
  }, [summary.authorization?.authorization_id, summary.review?.status]);

  const canReview = Boolean(projectWorkflow && summary.proposal && summary.authorization && summary.canReview);
  const approvedSummary = "全局主管复核通过方案边界；授权有效，仍未派发工作者。";

  return (
    <section className="project-canvas-detail-card" aria-label="全局边界复核">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">全局边界复核</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={summary.authorization?.status === "active" ? "candidate" : summary.review?.status === "blocked" ? "warning" : "unknown"}>
          {summary.status_label}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="用户确认" value={summary.proposal?.status === "user_confirmed" ? "已确认" : "未就绪"} />
        <DetailLine label="复核状态" value={summary.review?.status ? globalBoundaryReviewStatusLabels[summary.review.status as GlobalBoundaryReviewStatus] ?? summary.review.status : "待全局复核"} />
        <DetailLine label="守卫验证" value={summary.guard_display_text} />
        <DetailLine label="发现" value={String(summary.findings.length)} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="workflow-draft-grid">
          <DetailLine label="授权对象" value={summary.authorization?.authorization_id ?? "未建立"} />
          <DetailLine label="有效授权" value={summary.active_authorization_id ?? "暂无"} />
        </div>
      </details>
      {summary.blocked_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.guard_reasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
      {summary.findings.map((finding) => (
        <p className="state-warning" key={finding.finding_id}>{finding.summary}</p>
      ))}
      {canReview && summary.proposal && summary.authorization ? (
        <>
          <label className="proposal-decision-field">
            <span>修改 / 阻断原因</span>
            <textarea
              value={reviewSummary}
              onChange={(event) => setReviewSummary(event.target.value)}
              placeholder="要求修改或阻断方案时填写原因；批准并生效可留空。"
            />
          </label>
          <div className="workflow-state-actions">
            <button
              className="primary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "approved",
                    summary: approvedSummary,
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              批准并生效
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "needs_changes",
                    summary: reviewSummary.trim() || "全局主管要求修改方案边界；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              要求修改
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={() =>
                onRequestAction(
                  buildGlobalBoundaryReviewAction({
                    project,
                    proposal: summary.proposal!,
                    authorization: summary.authorization!,
                    reviewStatus: "blocked",
                    summary: reviewSummary.trim() || "全局主管阻断当前方案；不能自动推进。",
                    authorizationRevision: planAuthorizationSummary.revision,
                  }),
                )
              }
            >
              阻断方案
            </button>
          </div>
        </>
      ) : null}
    </section>
  );
}

// 件 D · 说目标 → AI 出方案：构造 run-project-consultation 动作（经确认弹层 = 真执行提示，App 执行 + 刷新方案 store）。
export function buildRunProjectConsultationAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number],
  goal: string,
): PendingAction {
  return {
    kind: "run-project-consultation",
    label: "让 AI 出方案（项目咨询）",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "真起 Codex 只读咨询、读项目上下文出一份方案，写成「待用户确认」草案；结构性只读、不碰执行闸、不启动工作者、不写 /Users/yoyi/.codex；方案不自动确认。",
    runProjectConsultation: {
      project_root: project.project_root,
      project_id: projectWorkflow.project_id,
      workflow_id: projectWorkflow.workflow_id,
      goal,
      actor_id: "user",
    },
  };
}

export function buildProjectConsultationProposalCreationAction(
  project: ProjectRecord,
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number],
  selectedTask: TaskDraftSummary | null,
  selectedTaskPackage: TaskPackage | null,
  proposalStoreRevision: number,
): PendingAction {
  const goal = selectedTaskPackage?.task_goal?.trim() || selectedTask?.title?.trim() || `围绕 ${project.name} 建立受控自动推进方案。`;
  const title = `项目咨询方案：${selectedTask?.title ?? projectWorkflow.title}`;
  const allowedReadRoots = uniqueNonEmpty([...(selectedTaskPackage?.allowed_read_scope ?? []), project.project_root]);
  const allowedWriteRoots = uniqueNonEmpty(selectedTaskPackage?.allowed_write_scope ?? []);
  const allowedTools = uniqueNonEmpty([...(selectedTaskPackage?.callable_tool_capabilities ?? []), "read_file"]);
  const allowedChecks = uniqueNonEmpty(selectedTaskPackage?.harness_requirements ?? []);
  const allowedRoleIds = uniqueNonEmpty([
    selectedTaskPackage?.target_role ?? null,
    selectedTask?.assigned_role_id ?? null,
    "project_director",
  ]);
  const allowedAgentIds = uniqueNonEmpty([
    selectedTaskPackage?.target_session_id ?? null,
    ...projectWorkflow.node_session_bindings.map((binding) => binding.native_thread_id),
  ]);

  return {
    kind: "create-project-consultation-proposal",
    label: "创建项目咨询方案草案",
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入工作台自己的 project-proposals.v1.json 辅助状态文件；不调用真实项目咨询智能体、不启动 Codex、不执行工作者、不写 /Users/yoyi/.codex。",
    projectConsultationProposalCreation: {
      project_root: project.project_root,
      project_id: projectWorkflow.project_id,
      workflow_id: projectWorkflow.workflow_id,
      title,
      user_goal: goal,
      goal_summary: goal,
      proposed_steps: [
        "整理用户目标和项目上下文。",
        "确认允许角色、agent、读写范围、工具、检查和停止条件。",
        "用户确认方案范围后，等待全局主管做边界复核。",
        "只有后续 C3/C4 授权生效后，项目主管才可在范围内准备自动推进。",
      ],
      scope_draft: {
        allowed_role_ids: allowedRoleIds,
        allowed_agent_ids: allowedAgentIds,
        allowed_read_roots: allowedReadRoots,
        allowed_write_roots: allowedWriteRoots,
        allowed_tools: allowedTools,
        allowed_checks: allowedChecks,
        allowed_task_package_kinds: ["task_package"],
        stop_conditions: ["出现超出读写范围、权限升级、用户偏好或高风险事实时必须停下请用户确认。"],
        max_worker_dispatches: 3,
        max_runtime_minutes: 60,
      },
      risks: [
        {
          risk_id: "risk:scope-draft-needs-global-review",
          severity: "warning",
          summary: "模板草案只来自当前工作流上下文，仍需全局主管复核边界。",
          mitigation: "用户确认后不自动派发，等待 C3 全局边界复核。",
        },
      ],
      acceptance_criteria: selectedTaskPackage?.acceptance_criteria.length
        ? selectedTaskPackage.acceptance_criteria
        : ["用户能看懂方案范围，并确认或要求修改；确认后授权仍停在待全局复核。"],
      created_by_role: "project_consultant",
      actor_id: "desktop_project_consultation_template",
      expected_store_revision: proposalStoreRevision,
    },
  };
}

export function buildProjectConsultationProposalDecisionAction({
  project,
  proposal,
  decision,
  summary,
  proposalStoreRevision,
  planAuthorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  decision: ProjectConsultationProposalDecisionKind;
  summary: string;
  proposalStoreRevision: number;
  planAuthorizationRevision: number;
}): PendingAction {
  return {
    kind: "record-project-consultation-proposal-decision",
    label: proposalDecisionActionLabel(decision),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      "写入 project-proposals.v1.json；确认方案时联动 plan-authorizations.v1.json 并停在待全局复核；不启动真实工作者、不执行 codex exec resume、不写 /Users/yoyi/.codex。",
    projectConsultationProposalDecision: {
      project_root: project.project_root,
      proposal_id: proposal.proposal_id,
      actor_id: "user",
      decision,
      summary,
      expected_proposal_store_revision: proposalStoreRevision,
      expected_plan_authorization_store_revision: planAuthorizationRevision,
    },
    projectConsultationProposalPreview: {
      title: proposal.title,
      goalSummary: proposal.goal_summary,
      allowedReadRoots: proposal.scope_draft.allowed_read_roots,
      allowedWriteRoots: proposal.scope_draft.allowed_write_roots,
      allowedTools: proposal.scope_draft.allowed_tools,
      allowedChecks: proposal.scope_draft.allowed_checks,
      stopConditions: proposal.scope_draft.stop_conditions,
    },
  };
}

function proposalDecisionActionLabel(decision: ProjectConsultationProposalDecisionKind) {
  if (decision === "confirm") return "确认方案范围";
  if (decision === "request_changes") return "要求修改项目咨询方案";
  return "拒绝项目咨询方案";
}

export function buildGlobalBoundaryReviewAction({
  project,
  proposal,
  authorization,
  reviewStatus,
  summary,
  authorizationRevision,
}: {
  project: ProjectRecord;
  proposal: ProjectConsultationProposal;
  authorization: NonNullable<ReturnType<typeof summarizeProjectConsultationProposalStore>["linked_plan_authorization"]>;
  reviewStatus: GlobalBoundaryReviewStatus;
  summary: string;
  authorizationRevision: number;
}): PendingAction {
  const findings =
    reviewStatus === "approved"
      ? []
      : [
          {
            finding_id: `finding:global-boundary-review:${reviewStatus}`,
            severity: reviewStatus === "blocked" ? ("blocking" as const) : ("warning" as const),
            summary,
            recommendation: reviewStatus === "blocked" ? "阻断后不能自动推进。" : "修改方案后再复核。",
          },
        ];
  return {
    kind: "record-global-boundary-review",
    label: globalBoundaryReviewActionLabel(reviewStatus),
    path: project.project_root,
    source: "索引内项目路径",
    boundary:
      reviewStatus === "approved"
        ? "写入 plan-authorizations.v1.json 的全局边界复核，并让授权有效；只让授权生效，不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。"
        : "写入 plan-authorizations.v1.json 的全局边界复核，并让授权保持不可自动推进；不启动工作者、不执行 codex exec、不写 /Users/yoyi/.codex。",
    globalBoundaryReview: {
      project_root: project.project_root,
      project_id: proposal.project_id,
      workflow_id: proposal.workflow_id,
      proposal_id: proposal.proposal_id,
      authorization_id: authorization.authorization_id,
      actor_id: "global_director",
      review_status: reviewStatus,
      summary,
      checklist: completeGlobalBoundaryReviewChecklist(),
      findings,
      expected_authorization_revision: authorizationRevision,
    },
    globalBoundaryReviewPreview: {
      proposalTitle: proposal.title,
      goalSummary: proposal.goal_summary,
      reviewStatus,
      readWriteScope: `读 ${authorization.scope.allowed_read_roots.length} / 写 ${authorization.scope.allowed_write_roots.length}`,
      toolsAndChecks: `工具 ${authorization.scope.allowed_tools.length} / 检查 ${authorization.scope.allowed_checks.length}`,
      stopConditions: authorization.scope.stop_conditions.map((condition) => condition.summary),
      findings,
    },
  };
}

function completeGlobalBoundaryReviewChecklist() {
  return {
    architecture_boundary_checked: true,
    cross_project_impact_checked: true,
    permission_scope_checked: true,
    read_write_scope_checked: true,
    tool_and_check_scope_checked: true,
    memory_boundary_checked: true,
    stop_conditions_checked: true,
    acceptance_criteria_checked: true,
  };
}

function globalBoundaryReviewActionLabel(reviewStatus: GlobalBoundaryReviewStatus) {
  if (reviewStatus === "approved") return "批准并生效";
  if (reviewStatus === "needs_changes") return "要求修改全局边界";
  return "阻断方案";
}

function uniqueNonEmpty(values: Array<string | null | undefined>) {
  return Array.from(new Set(values.map((value) => value?.trim()).filter((value): value is string => Boolean(value))));
}

export function PlanAuthorizationSummaryCard({
  summary,
  guardResult,
  guardError,
}: {
  summary: ReturnType<typeof summarizePlanAuthorizationStore>;
  guardResult: AutoDispatchGuardResult | null;
  guardError: string | null;
}) {
  const guardSummary = summarizeAutoDispatchGuardResult(guardResult ?? summary.recent_guard_result ?? null);
  const blockedReasons = guardSummary.reasons.slice(0, 3);
  return (
    <section className="project-canvas-detail-card" aria-label="方案授权摘要">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">方案授权摘要</p>
          <h3>{summary.display_text}</h3>
        </div>
        <Badge tone={guardSummary.status === "authorized" ? "candidate" : guardSummary.status === "not_checked" ? "unknown" : "warning"}>
          {guardSummary.status}
        </Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="允许角色" value={String(summary.actor_scope?.allowed_role_ids.length ?? 0)} />
        <DetailLine label="允许智能体" value={String(summary.actor_scope?.allowed_agent_ids.length ?? 0)} />
        <DetailLine label="读写范围" value={`读 ${summary.resource_scope?.allowed_read_roots.length ?? 0} / 写 ${summary.resource_scope?.allowed_write_roots.length ?? 0}`} />
        <DetailLine label="工具 / 检查" value={`工具 ${summary.resource_scope?.allowed_tools.length ?? 0} / 检查 ${summary.resource_scope?.allowed_checks.length ?? 0}`} />
        <DetailLine label="停止条件" value={String(summary.stop_condition_count)} />
        <DetailLine label="当前检查" value={guardSummary.display_text} />
      </div>
      {guardError ? <p className="state-warning">授权检查读取失败：{guardError}</p> : null}
      <details className="agent-boundary-details">
        <summary className="agent-boundary-summary">开发者详情</summary>
        <div className="workflow-draft-grid">
          <DetailLine label="边车" value={`${summary.sidecar_name} / 版本 ${summary.revision}`} />
          <DetailLine label="授权对象" value={summary.latest_authorization_id ?? "未建立"} />
          <DetailLine label="生效授权" value={summary.active_authorization_id ?? "暂无"} />
          <DetailLine label="最近审计" value={summary.recent_audit_event_id ?? "暂无"} />
        </div>
      </details>
      {blockedReasons.map((reason) => (
        <p className="state-warning" key={reason}>{reason}</p>
      ))}
    </section>
  );
}
