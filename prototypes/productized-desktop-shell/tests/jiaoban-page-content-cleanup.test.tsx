// 交办页内容清理：七项删减均以离线 DOM 断言，守住右侧画布、原始对话和既有执行入口。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanAuthorizeState,
  JiaobanBlockedState,
  JiaobanDoneState,
  JiaobanHistoryDetail,
  JiaobanPlanPreviewCanvas,
  JiaobanRunningState,
  JiaobanSayState,
} from "../src/views/projects/ProjectJiaobanPanel";
import { JiaobanMergedLayout } from "../src/views/projects/ProjectWorkspaceShell";
import type {
  AutoAdvanceRoleLoopOutcome,
  ProjectConsultationProposal,
  ProjectWorkflowChainStatus,
  RunHistoryEntry,
} from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-page-content-cleanup] ${message}`);
}

const noop = () => {};
const removedCopy = (...parts: string[]) => parts.join("");

function proposalFixture(): ProjectConsultationProposal {
  return {
    proposal_id: "cleanup-proposal",
    schema_version: "project_consultation_proposal.v1",
    project_id: "cleanup-project",
    workflow_id: "cleanup-workflow",
    title: "清理交办页",
    user_goal: "清理交办页内容",
    goal_summary: "把页面文案收紧",
    proposed_steps: ["目标文件：src/views/projects/ProjectJiaobanPanel.tsx"],
    scope_draft: {
      allowed_role_ids: [],
      allowed_agent_ids: [],
      allowed_read_roots: [],
      allowed_write_roots: ["/tmp/cleanup"],
      allowed_tools: [],
      allowed_checks: [],
      allowed_task_package_kinds: [],
      stop_conditions: [],
    },
    risks: [],
    acceptance_criteria: ["离线 DOM 断言"],
    status: "pending_user_confirmation",
    created_by_role: "project_consultant",
    created_at_ms: 0,
    updated_at_ms: 0,
    suggest_workflow: false,
  };
}

function authorizeHtml(proposalIsStale = false, proposalAgeDays = 0): string {
  return renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={proposalFixture()}
      proposalIsStale={proposalIsStale}
      proposalAgeDays={proposalAgeDays}
      sessions={[]}
      sessionChoice={null}
      onSessionChoiceChange={noop}
      amendment=""
      onAmendmentChange={noop}
      onAmend={noop}
      onAuthorizeAndStart={noop}
      onRePlan={noop}
      onDecline={noop}
      starting={false}
      consultLoading={false}
      consultError={null}
      worksmapSwitchOn={false}
      onToggleWorksmapSwitch={noop}
      worksmapTasks={null}
      worksmapLoading={false}
      worksmapError={null}
      boundaryLoading={false}
      boundaryOutcome={null}
      onBoundaryRetry={noop}
      onOpenAgentSession={noop}
    />,
  );
}

// 1、4、5、6：所有交办内引导回到右侧画布；会话预填与人闸仍在，旧方案黄条不再重复时间和建议。
{
  const authorize = authorizeHtml();
  assert(authorize.includes("给第一个预演节点预填对话"), "预填对话标签应保留");
  assert(!authorize.includes(removedCopy("批了就自动跑完；碰到越界或拿不准会", "停下来问你。")), "授权教学提示应删除");
  assert(!authorize.includes(removedCopy("AI 的", "方案")), "授权卡标题应删除");
  assert(!authorize.includes(removedCopy("想改就", "直接说")), "改方案输入的教学标签应删除");
  assert(!authorize.includes(removedCopy("点「允许并开始」= 允许这段自动跑，", "后面不再逐步问你。")), "遗留授权教学提示应删除");
  assert(!authorize.includes(removedCopy("这里的选择与右侧第一个预演节点", "同步")), "授权卡应删会话同步小字");
  assert(!authorize.includes(removedCopy("碰到越界或拿不准的，会停下来", "问你，不硬来。")), "旧授权提醒不应残留");

  const stale = authorizeHtml(true, 3);
  assert(stale.includes("这是 3 天前的旧方案，项目可能已变——建议重新说一遍。"), "旧方案黄条应收为一句");
  assert(
    !stale.includes(removedCopy("生", "成于")) && !stale.includes(removedCopy("出一版", "新的")),
    "旧方案黄条不应重复时间或建议",
  );
}

{
  const say = renderToStaticMarkup(
    <JiaobanSayState
      goal="清理交办页"
      onGoalChange={noop}
      onSubmit={noop}
      lastStopHint={null}
      loading={false}
      error={null}
      onEditAgain={noop}
    />,
  );
  assert(!say.includes(removedCopy("这期间界面不会卡，也可以", "先去忙别的")), "说相底部重复时长说明应删除");
  assert(say.includes("说一句话，AI 会读你的项目、想个方案给你审。"), "说相输入提示应保留");
  assert(!say.includes(removedCopy('class="eyebrow">交', "办")), "说相 eyebrow 应删除");
}

{
  const historyEntry = {
    proposal_id: "history-cleanup",
    workflow_id: "cleanup-workflow",
    goal_text: "看看历史过程",
    created_at_ms: 0,
    state: "delivered",
    state_note: "做完了",
    advice_only: false,
    chain: null,
    review_flags: {},
    correlation: "exact",
  } as RunHistoryEntry;
  const history = renderToStaticMarkup(<JiaobanHistoryDetail entry={historyEntry} onBackToCurrent={noop} />);
  assert(history.includes("具体每一步的过程，看右侧画布。"), "历史详情应指向右侧画布");
  assert(!history.includes(removedCopy("去「工作流」", "标签看")), "历史详情不应再引导切页");

  const running = renderToStaticMarkup(
    <JiaobanRunningState
      chainStatus={null}
      isNewSession={false}
      onStop={noop}
      sessionChoice={null}
      latestSessionThreadId={null}
    />,
  );
  assert(running.includes("想看每一步的过程，看右侧画布。"), "运行态应指向右侧画布");
  assert(!running.includes(removedCopy("切到「工作流」", "tab")), "运行态不应再引导切页");
  assert(!running.includes(removedCopy('class="eyebrow">交', "办")), "运行相 eyebrow 应删除");
}

// 2：结果行承担完成数和停因，不能再另起步数或伪产出行。
{
  const chainStatus: ProjectWorkflowChainStatus = {
    chain_run_id: "cleanup-chain",
    state: "stopped",
    nodes: [
      { node_id: "first", state: "completed" },
      { node_id: "second", state: "completed" },
    ],
  };
  const outcome: AutoAdvanceRoleLoopOutcome = {
    stage: "completed",
    planned_task_count: 2,
    prepared_count: 2,
    needs_binding_count: 0,
    blocked_count: 0,
    message: "做完了。",
    chain_outcome: {
      total: 2,
      dispatched: 2,
      completed: 2,
      skipped: 0,
      chain_run_id: "cleanup-chain",
      steps: [],
      warnings: [],
      stopped_reason: "等你确认下一步",
    },
    stop_reason: "等你确认下一步",
    planned_tasks: [],
  };
  const done = renderToStaticMarkup(
    <JiaobanDoneState
      outcome={outcome}
      chainStatus={chainStatus}
      onContinue={noop}
      needsRework={null}
      needsReworkActionError={null}
      needsReworkActionStarting={false}
      onNeedsReworkContinue={noop}
      onNeedsReworkAction={noop}
      onRequestAction={noop}
      factCtx={null}
      sessionChoice={null}
      latestSessionThreadId={null}
      supervisorLoading={false}
      supervisorOutcome={null}
      onSupervisorRetry={noop}
      onSupervisorReplan={noop}
    />,
  );
  assert(done.includes("完成 2 步；中途停了：等你确认下一步。"), "结果行应保留完成数和停因");
  assert(
    !done.includes(removedCopy("这次做", "完")) &&
      !done.includes("产出：") &&
      !done.includes(removedCopy("跑完 2/", "2 步")),
    "重复步数和伪产出行应删除",
  );
  assert(!done.includes(removedCopy('class="eyebrow">交', "办")), "交货相 eyebrow 应删除");
}

// 1：卡住态仍给非零按钮，只改为看右侧画布。
{
  const blocked = renderToStaticMarkup(
    <JiaobanBlockedState
      outcome={null}
      error="需要你决定"
      planIsConfirmed={false}
      sessions={[]}
      sessionChoice={null}
      onSessionChoiceChange={noop}
      onContinueRun={noop}
      onRePlan={noop}
      starting={false}
      onOpenWorkflow={noop}
      latestSessionThreadId={null}
    />,
  );
  assert(blocked.includes("看右侧画布") && blocked.includes("重新说目标出新方案"), "卡住态应保留可行动按钮并改画布文案");
  assert(!blocked.includes(removedCopy("去工作", "流看看")), "卡住态不应保留旧跳转文案");
  assert(!blocked.includes(removedCopy('class="eyebrow">交', "办")), "卡住相 eyebrow 应删除");
}

// 第二波：预演图保留节点与语义，不再额外解释它是什么。
{
  const preview = renderToStaticMarkup(
    <JiaobanPlanPreviewCanvas
      nodes={[{ preview_node_id: "cleanup-preview", title: "清理页面", depends_on: [] }]}
      bindings={[{ preview_node_id: "cleanup-preview", session_choice: "new" }]}
      sessions={[]}
      waitingForPreview={false}
      previewError={null}
      previewWarnings={[]}
      onBindingChange={noop}
      onRetryPreview={noop}
      onOpenAgentSession={noop}
    />,
  );
  assert(preview.includes("任务 · 预演") && preview.includes('aria-label="方案预演工序图"'), "预演节点和语义应保留");
  assert(
    !preview.includes(removedCopy("<strong>预演工序图", "</strong>")) &&
      !preview.includes(removedCopy("你批的就是", "这份图")),
    "预演画布教学性头部应删除",
  );
}

// 7：说相没有运行画布数据时，右侧给出下一步会出现预演图的明确空态；完整工作流入口仍在。
{
  const layout = renderToStaticMarkup(
    <JiaobanMergedLayout
      phase="say"
      history={<div>历史</div>}
      main={<div>说相主区</div>}
      workflowPanel={null}
      onOpenWorkflow={noop}
    />,
  );
  assert(layout.includes("出方案后，这里会出现工序图预演。"), "说相右侧应显示预演空态");
  assert(layout.includes("在工作流页打开"), "完整工作流入口应保留");
  assert(!layout.includes("工作流数据暂不可用"), "说相不应显示笼统无数据占位");
}

console.log("jiaoban-page-content-cleanup: 七项交办内容清理离线 DOM 断言全过");
