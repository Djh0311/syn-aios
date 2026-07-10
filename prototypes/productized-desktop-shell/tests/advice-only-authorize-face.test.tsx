// 写根空的只读单离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// willWrite=false → 只读警条 + 主按钮=[允许并开始（只读）] + 保留重新出方案次按钮；
// willWrite=true → 原样（无只读警条·主按钮=[允许并开始]）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanAuthorizeState, JiaobanDoneState } from "../src/views/projects/ProjectJiaobanPanel";
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
    proposed_steps: [],
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
    acceptance_criteria: [],
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
      proposalTimeText="2026-07-07 16:48"
      proposalIsStale={false}
      proposalAgeDays={0}
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
      worksmapWarnings={[]}
      worksmapLoading={false}
      worksmapError={null}
      onRetryWorksmap={noop}
      boundaryLoading={false}
      boundaryOutcome={null}
      onBoundaryRetry={noop}
      onOpenAgentSession={noop}
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
  assert(out.includes("🔓"), "正常方案 🔓 许可行原样（willWrite=true）");
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
  stage: "ran",
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
