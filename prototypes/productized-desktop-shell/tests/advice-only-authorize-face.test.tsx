// 写根空的只读单离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// willWrite=false → 只读警条 + 主按钮=[允许并开始（只读）] + 保留重新出方案次按钮；
// willWrite=true → 原样（无只读警条·主按钮=[允许并开始]）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanAuthorizeState, JiaobanDoneState } from "../src/views/projects/ProjectJiaobanPanel";
import { JiaobanHowRunView } from "../src/views/projects/jiaoban/JiaobanAuthorizeStates";
import type { AutoAdvanceRoleLoopOutcome, ProjectConsultationProposal, ProjectDirectorPlannedTask } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[advice-only-face] ${message}`);
  }
}

function proposalFixture(allowedWriteRoots: string[]): ProjectConsultationProposal {
  return {
    proposal_id: "p1",
    schema_version: "project_consultation_proposal.v1",
    project_id: "proj",
    workflow_id: "wf-1",
    title: "方案",
    user_goal: "加回 1 个怪",
    goal_summary: "在游戏里加回 1 个怪物",
    proposed_steps: ["目标文件：src/views/projects/ProjectJiaobanPanel.tsx"],
    scope_draft: {
      allowed_role_ids: [],
      allowed_agent_ids: [],
      allowed_read_roots: [],
      allowed_write_roots: allowedWriteRoots,
      allowed_tools: [],
      allowed_checks: [],
      allowed_task_package_kinds: [],
      stop_conditions: [],
    },
    risks: [],
    acceptance_criteria: ["pnpm typecheck", "离线交互测试"],
    status: "pending_user_confirmation",
    created_by_role: "project_consultant",
    created_at_ms: Date.now(), // 今天生成（不触发旧方案黄条·隔离本测关注点）
    updated_at_ms: Date.now(),
    suggest_workflow: false,
  };
}

const noop = () => {};

function html(allowedWriteRoots: string[]): string {
  return renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={proposalFixture(allowedWriteRoots)}
      proposalIsStale={false}
      proposalAgeDays={0}
      amendment=""
      onAmendmentChange={noop}
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

// 1) 只读单（写根空）：警条在 + [允许并开始]仍是主按钮 + 保留重新出方案。
{
  const out = html([]);
  assert(out.includes("这单是只读的——AI 只看不改，交货是结论不是改动"), "警条应如实说明只读单");
  assert(out.includes("重新出方案（要动手）"), "应保留 [重新出方案（要动手）] 次按钮");
  assert(out.includes("允许并开始（只读）"), "[允许并开始] 应保留在只读单主按钮");
  // 主按钮位（primary-button）必须是允许并开始。
  const primarySegment = out.slice(out.indexOf("primary-button"), out.indexOf("primary-button") + 300);
  assert(primarySegment.includes("允许并开始（只读）"), "primary 位应是允许并开始");
}

// 2) 正常档位方案（写根非空）：零回退——无只读警条、主按钮=[允许并开始]、无改道按钮。
{
  const out = html(["/Users/yoyi/codex-workflow-mario-test"]);
  assert(!out.includes("这单是只读的"), "正常方案不显只读警条");
  assert(!out.includes("重新出方案（要动手）"), "正常方案无改道按钮");
  assert(out.includes("允许并开始"), "正常方案主按钮原样");
  // 07-15 二审稿:🔓 许可横幅已删(写入范围在「会动什么」事实行·批准按钮本身就是人闸——信息四问·重复即删)。
  assert(!out.includes("🔓"), "许可横幅应已删除");
  // 07-15 走查#2·批态卡定式:一句标题(jiaoban-plan-title)+「会动什么」事实行+「怎么算做好」逐条编号,
  // 取代「我来做:/会改的文件:/改完怎么验:」分号长句字段行。
  assert(out.includes("jiaoban-plan-title"), "授权卡应有一句话标题");
  assert(out.includes("会动什么") && out.includes("会改的文件"), "「会动什么」事实组在");
  assert(out.includes("怎么算做好") && out.includes("<ol"), "「怎么算做好」应逐条编号非分号长句");
  assert(!out.includes("我来做："), "「我来做:」字段行退役(标题已顶位)");
  // 配置件(预填对话/预演开关/执行模式)移右区「怎么跑」视图;卡上只留摘要入口一行。
  assert(!out.includes("给第一个预演节点预填对话"), "预填对话应已移出批卡(归右区怎么跑视图)");
  assert(out.includes("jiaoban-plan-link--howrun") && out.includes("怎么跑"), "「怎么跑」摘要入口应在卡上");
  assert(!out.includes("jiaoban-worksmap-graph"), "授权卡不应再渲染步骤流图");
  assert(!out.includes("目标："), "授权卡应删去重复的目标");
}

// 2b) 右区「怎么跑」视图承载配置件(07-15 二审稿:预填对话/预演开关/执行模式从批卡移来)。
{
  const out = renderToStaticMarkup(
    <JiaobanHowRunView
      suggestWorkflow={false}
      worksmapSwitchOn={false}
      onToggleWorksmapSwitch={noop}
      orchestrationMode="classic"
      onOrchestrationModeChange={noop}
      supervisorPilotDisabledReason={null}
      classicDisabledReason={null}
      disabled={false}
      sessions={[]}
      sessionChoice={null}
      onSessionChoiceChange={noop}
      onOpenAgentSession={noop}
    />,
  );
  assert(out.includes("给第一个预演节点预填对话"), "怎么跑视图应承载预填对话");
  assert(out.includes("按工作流来（在右侧预演画布看工序图）"), "怎么跑视图应承载预演开关");
  assert(out.includes("执行模式"), "怎么跑视图应承载执行模式单选");
}

const readonlyTask: ProjectDirectorPlannedTask = {
  planned_task_id: "readonly-1",
  title: "核验结论",
  objective: "只读核验",
  scope: {
    project_id: "proj",
    workflow_id: "wf-1",
    target_role: "codex-dev",
    task_package_kind: "task_package",
    allowed_read_scope: ["/Users/yoyi/codex-workflow-mario-test"],
    allowed_write_scope: [],
    callable_tool_capabilities: [],
    required_checks: [],
    stop_conditions: [],
  },
  depends_on: [],
  acceptance_criteria: ["返回核验结论"],
  report_format: ["做了什么"],
  status: "prepared",
  blocked_reasons: [],
};

const readonlyOutcome: AutoAdvanceRoleLoopOutcome = {
  stage: "completed",
  planned_task_count: 1,
  prepared_count: 1,
  needs_binding_count: 0,
  blocked_count: 0,
  message: "做完了。",
  chain_outcome: null,
  stop_reason: null,
  planned_tasks: [readonlyTask],
};

// 3) 只读链交货如实标出未改文件。
{
  const out = renderToStaticMarkup(
    <JiaobanDoneState
      outcome={readonlyOutcome}
      chainStatus={null}
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
  assert(out.includes("只读单·未改文件"), "只读链交货应标出未改文件");
}

console.log("advice-only-authorize-face: 3 组离线 DOM 断言全过");
