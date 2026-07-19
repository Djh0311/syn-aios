// 交办页内容清理：七项删减均以离线 DOM 断言，守住右侧画布、原始对话和既有执行入口。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanAuthorizeState,
  JiaobanBlockedState,
  JiaobanDoneState,
  JiaobanHistoryDetail,
  JiaobanPlanPreviewCanvas,
  JiaobanRunningState,
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

function authorizeHtml(proposalIsStale = false): string {
  return renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={proposalFixture()}
      proposalIsStale={proposalIsStale}
      amendment=""
      onAmend={noop}
      onAuthorizeAndStart={noop}
      onRePlan={noop}
      onDecline={noop}
      starting={false}
      consultLoading={false}
      consultError={null}
      howRunSummary="经典状态机 · 开个新对话 · 预演图关"
      onShowGovernance={noop}
      onShowHowRun={noop}
      boundaryLoading={false}
      boundaryOutcome={null}
      onBoundaryRetry={noop}
    />,
  );
}

// 1、4、5、6：所有交办内引导回到右侧画布；会话预填与人闸仍在，旧方案提示牌从呈现退场。
{
  const authorize = authorizeHtml();
  // 07-15 二审稿:预填对话移右区「怎么跑」视图(覆盖在 advice-only-authorize-face 2b);卡上=摘要入口一行。
  assert(!authorize.includes("给第一个预演节点预填对话"), "预填对话应已移出批卡");
  assert(authorize.includes("jiaoban-plan-link--howrun"), "批卡应有「怎么跑」摘要入口");
  assert(!authorize.includes(removedCopy("批了就自动跑完；碰到越界或拿不准会", "停下来问你。")), "授权教学提示应删除");
  assert(!authorize.includes(removedCopy("AI 的", "方案")), "授权卡标题应删除");
  assert(!authorize.includes(removedCopy("想改就", "直接说")), "改方案输入的教学标签应删除");
  assert(!authorize.includes(removedCopy("点「允许并开始」= 允许这段自动跑，", "后面不再逐步问你。")), "遗留授权教学提示应删除");
  assert(!authorize.includes(removedCopy("这里的选择与右侧第一个预演节点", "同步")), "授权卡应删会话同步小字");
  assert(!authorize.includes(removedCopy("碰到越界或拿不准的，会停下来", "问你，不硬来。")), "旧授权提醒不应残留");

  const stale = authorizeHtml(true);
  assert(!stale.includes('aria-label="旧方案提醒"'), "旧方案提示牌应从呈现退场");
  assert(!stale.includes("jiaoban-stale-banner"), "旧方案提示牌的卡形钩子不应残留");
  assert(!stale.includes("天前的旧方案，项目可能已变"), "旧方案提示牌文案不得换壳残留");
  assert(stale.includes("重新说目标出新方案"), "旧方案主按钮仍应引导重新出方案");
  assert(stale.includes("仍要允许并开始（旧方案）"), "旧方案手动开工次按钮仍应保留");
  assert(!stale.includes('aria-label="修改方案"'), "旧方案仍应收起无消费者的修改框");
  // P1-D:卡上修改框(input)彻底退场——正常态也不再有,只留常驻框(见下方开放问题块的收编测试)。
  assert(!authorize.includes('aria-label="修改方案"'), "正常态也不应再有卡上修改框，只留常驻框");
}

// P1-D 人闸收敛:说态卡 JiaobanSayState 已退场——phase="say" 改由常驻框 new_goal 路由承载(修单3);
// 说态卡本体、教育句、eyebrow 等断言随卡退场,不再有独立说态卡可测。

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
      directorPlanningElapsedMinutes={0}
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
      main={<div>说相主区</div>}
      proposalIndex={<div>历届方案</div>}
      workflowPanel={null}
      onOpenWorkflow={noop}
    />,
  );
  assert(layout.includes("出方案后，这里会出现工序图预演。"), "说相右侧应显示预演空态");
  // 07-15 走查修:窄条=纯提示句,跳转钮随画布宽态走(见 jiaoban-merged-layout 测试)。
  assert(!layout.includes("在工作流页打开"), "说相窄条不再渲染跳转钮");
  assert(!layout.includes("工作流数据暂不可用"), "说相不应显示笼统无数据占位");
}

console.log("jiaoban-page-content-cleanup: 七项交办内容清理离线 DOM 断言全过");

// P1-D 人闸收敛(翻案自 07-16 真单实案):方案步骤里"用户确认/补充…"正则止血件 extractOpenQuestions
// 已退场——P1-B 结构化 waiting_user 读模型已替代它(待答追问独占常驻框答复通道、方案卡本身随
// hasPendingSupervisorQuestion 冻结交互，见 jiaoban-conversation-center.test.tsx)。这里锁住:
// 即使方案步骤文本长得像旧启发式会命中的"用户确认/补充…"句式，批卡也不再解析出问题块或降级按钮。
{
  assert(
    !("extractOpenQuestions" in (await import("../src/views/projects/jiaoban/JiaobanAuthorizeStates"))),
    "止血件正则函数应已删除，不再从模块导出",
  );

  const looksLikeOpenQuestions = renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={{
        ...proposalFixture(),
        proposed_steps: [
          "目标文件：index.agent-copy.html",
          "用户确认副本名是否采用 index.agent-copy.html。",
          "用户补充唯一 worker 在副本上需要完成的具体改动。",
        ],
      }}
      proposalIsStale={false}
      amendment=""
      onAmend={noop}
      onAuthorizeAndStart={noop}
      onRePlan={noop}
      onDecline={noop}
      starting={false}
      consultLoading={false}
      consultError={null}
      howRunSummary="经典状态机 · 开个新对话 · 预演图关"
      onShowGovernance={noop}
      onShowHowRun={noop}
      boundaryLoading={false}
      boundaryOutcome={null}
      onBoundaryRetry={noop}
    />,
  );
  assert(!looksLikeOpenQuestions.includes("方案在等你答"), "止血件退场后不应再解析出开放问题块");
  assert(!looksLikeOpenQuestions.includes("答完问题出新方案"), "止血件退场后主按钮不应降级");
  assert(!looksLikeOpenQuestions.includes("仍要允许并开始（"), "止血件退场后次按钮不应带未答计数");
  assert(looksLikeOpenQuestions.includes("允许并开始"), "正常写根方案应回落三态收敛后的默认按钮组");

  const noQuestions = authorizeHtml();
  assert(!noQuestions.includes("方案在等你答"), "零问题方案原样不出问题块");
  assert(!noQuestions.includes("仍要允许并开始（"), "零问题方案按钮原样");
}

// 07-16:空任务列表卡住脸=方案内容类死配对(主按钮重新说目标+人话原因),不许配成「接着跑」死循环。
{
  const { classifyBlocked } = await import("../src/views/projects/jiaoban/JiaobanBlockedStates");
  const plan = classifyBlocked(
    {
      stage: "failed",
      planned_task_count: 0,
      prepared_count: 0,
      needs_binding_count: 0,
      blocked_count: 0,
      message: "自动推进失败（已留档）：主管产出空任务列表",
      chain_outcome: null,
      stop_reason: null,
      planned_tasks: [],
    },
    null,
    true,
  );
  assert(plan.primary === "replan", "空任务列表:主按钮=重新说目标(非接着跑)");
  assert(!!plan.note && plan.note.includes("拆不出可执行的任务"), "空任务列表:人话原因在");
}
