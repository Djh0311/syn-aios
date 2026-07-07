// fix9·批卡纯建议诚实脸·离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// §4：willWrite=false（写根空=纯建议）→ 警条在 + 主按钮=[重新出方案（要动手）] + [允许并开始]降次但**不删死**；
// willWrite=true → 原样（无警条·主按钮=[允许并开始]）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanAuthorizeState } from "../src/views/projects/ProjectJiaobanPanel";
import type { ProjectConsultationProposal } from "../src/lib/types";

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

// 1) 纯建议（写根空）：警条在 + 主按钮改道 + [允许并开始]降次不删死。
{
  const out = html([]);
  assert(out.includes("不会改任何文件"), "警条应喊出「不会改任何文件」");
  assert(out.includes("纯建议"), "警条应点名纯建议");
  assert(out.includes("重新出方案（要动手）"), "主按钮应改道 [重新出方案（要动手）]");
  assert(out.includes("仍要允许并开始（纯建议）"), "[允许并开始] 降为次按钮但不删死（按钮永远有路）");
  // 主按钮位（primary-button）应是改道按钮而非允许并开始。
  const primarySegment = out.slice(out.indexOf("primary-button"), out.indexOf("primary-button") + 300);
  assert(primarySegment.includes("重新出方案（要动手）"), "primary 位应是改道按钮");
}

// 2) 正常档位方案（写根非空）：零回退——无警条、主按钮=[允许并开始]、无改道按钮。
{
  const out = html(["/Users/yoyi/codex-workflow-mario-test"]);
  assert(!out.includes("不会改任何文件"), "正常方案不显纯建议警条");
  assert(!out.includes("重新出方案（要动手）"), "正常方案无改道按钮");
  assert(out.includes("允许并开始"), "正常方案主按钮原样");
  assert(out.includes("🔓"), "正常方案 🔓 许可行原样（willWrite=true）");
}

console.log("advice-only-authorize-face: 2 组离线 DOM 断言全过");
