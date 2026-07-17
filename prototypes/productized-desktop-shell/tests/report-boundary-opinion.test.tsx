// B2·全局主管批前边界意见·离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// 覆盖：四态（loading / looks_ok / mismatch+points / unavailable+重试 / null 零渲染）+ 触发判据（今天 pending
// 触发·stale 不触发·非 pending 不触发）+ 词表无「审批」+ 授权卡按钮区两态（fix9 改道/正常）不受意见块影响。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanAuthorizeState,
  JiaobanBoundaryReviewSection,
  shouldRequestBoundaryReview,
} from "../src/views/projects/ProjectJiaobanPanel";
import type {
  GlobalSupervisorBoundaryReviewOutcome,
  GlobalSupervisorBoundaryReviewRecord,
  ProjectConsultationProposal,
} from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[boundary-opinion] ${message}`);
  }
}

const noop = () => {};

function record(overrides: Partial<GlobalSupervisorBoundaryReviewRecord>): GlobalSupervisorBoundaryReviewRecord {
  return {
    review_id: "boundary:p1",
    project_id: "proj",
    proposal_id: "p1",
    status: "ready",
    verdict: "looks_ok",
    points: [],
    summary: "",
    model: "codex-cli-default",
    profile_version: "global-supervisor-boundary-profile.v1",
    created_at_ms: 1,
    updated_at_ms: 1,
    ...overrides,
  };
}

function ready(review: GlobalSupervisorBoundaryReviewRecord): GlobalSupervisorBoundaryReviewOutcome {
  return { status: "ready", review, reason: null, warnings: [] };
}

function sectionHtml(
  loading: boolean,
  outcome: GlobalSupervisorBoundaryReviewOutcome | null,
): string {
  return renderToStaticMarkup(
    <JiaobanBoundaryReviewSection loading={loading} outcome={outcome} onRetry={noop} />,
  );
}

// 1) loading：正在看边界 + 明说「意见没到也可以先批」（不拦事）。
{
  const out = sectionHtml(true, null);
  assert(out.includes("全局主管正在看边界"), "loading：文案");
  assert(out.includes("也可以先批"), "loading：明说不拦批");
  assert(!out.includes("审批"), "loading：词表无「审批」");
}

// 2) ready·looks_ok：一行绿「对得上」，无告警色。
{
  const out = sectionHtml(false, ready(record({ verdict: "looks_ok", summary: "范围对得上" })));
  assert(out.includes("范围和你的目标对得上"), "looks_ok：人话行");
  assert(out.includes("jiaoban-boundary-ok"), "looks_ok：绿色调 class");
  assert(!out.includes("jiaoban-boundary-flag"), "looks_ok：不用告警色");
  assert(!out.includes("审批"), "looks_ok：词表无「审批」");
}

// 3) ready·mismatch：告警色 + points 列表逐条 + summary（money-shot：点破目标 vs 纯建议）。
{
  const out = sectionHtml(
    false,
    ready(
      record({
        verdict: "mismatch",
        points: ["你要动手，这方案不改任何文件", "步骤是「建议你考虑」不是能落地的动作"],
        summary: "目标要动手、方案纯建议，对不上",
      }),
    ),
  );
  assert(out.includes("好像对不上你的目标"), "mismatch：人话行");
  assert(out.includes("jiaoban-boundary-flag"), "mismatch：告警色调");
  assert(out.includes("这方案不改任何文件"), "mismatch：point 1 逐条");
  assert(out.includes("不是能落地的动作"), "mismatch：point 2 逐条");
  assert(out.includes("目标要动手、方案纯建议，对不上"), "mismatch：summary");
  assert(out.includes("批不批还是你说了算"), "mismatch：脚注重申不拦批");
  assert(!out.includes("审批"), "mismatch：词表无「审批」");
}

// 4) unavailable：人话原因 + [重试]，绝不零出路；明说不影响批。
{
  const out = sectionHtml(false, {
    status: "unavailable",
    review: null,
    reason: "codex 额度用完了，明天再试",
    warnings: [],
  });
  assert(out.includes("边界意见暂时不可用"), "unavailable：人话");
  assert(out.includes("额度用完"), "unavailable：具体原因透传");
  assert(out.includes("不影响你批"), "unavailable：明说不挡批");
  assert(out.includes("重试"), "unavailable：给重试口·绝不零出路");
  assert(!out.includes("审批"), "unavailable：词表无「审批」");
}

// 5) null 且不 loading → 一行「还没到(不拦批)」+[要意见](07-16 用户终裁:主管意见=批卡常显位,
//    意见没到也示明;非状态回声)。
{
  const out = sectionHtml(false, null);
  assert(out.includes("全局主管意见还没到（不拦批）"), "null:常显位一行在");
  assert(out.includes("要意见"), "null:给[要意见]出路");
}

// 6) 触发判据：今天 pending → 触发；stale（3 天前）→ 不触发（省额度）；非 pending → 不触发；null → 不触发。
{
  const DAY = 24 * 60 * 60 * 1000;
  assert(
    shouldRequestBoundaryReview({ status: "pending_user_confirmation", created_at_ms: Date.now() }),
    "今天 pending → 触发",
  );
  assert(
    shouldRequestBoundaryReview({ status: "draft", created_at_ms: Date.now() }),
    "今天 draft → 触发",
  );
  assert(
    !shouldRequestBoundaryReview({ status: "pending_user_confirmation", created_at_ms: Date.now() - 3 * DAY }),
    "stale（3 天前）→ 不触发（省额度）",
  );
  assert(
    !shouldRequestBoundaryReview({ status: "user_confirmed", created_at_ms: Date.now() }),
    "非 pending（已确认）→ 不触发",
  );
  assert(!shouldRequestBoundaryReview(null), "null → 不触发");
}

// 7) 授权卡按钮区两态不受意见块影响 + 意见块与按钮区共存（§4）。
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
    created_at_ms: Date.now(),
    updated_at_ms: Date.now(),
    suggest_workflow: false,
  };
}

function authorizeHtml(
  allowedWriteRoots: string[],
  boundaryOutcome: GlobalSupervisorBoundaryReviewOutcome | null,
): string {
  return renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={proposalFixture(allowedWriteRoots)}
      proposalIsStale={false}
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
      boundaryOutcome={boundaryOutcome}
      onBoundaryRetry={noop}
    />,
  );
}

// 7a) 只读单（写根空）+ mismatch 意见共存：主按钮仍是[允许并开始]，保留重新出方案次按钮。
{
  const mismatch = ready(record({ verdict: "mismatch", points: ["写根空·不改文件"], summary: "对不上" }));
  const out = authorizeHtml([], mismatch);
  assert(out.includes("允许并开始（只读）"), "只读单：[允许并开始]仍是主按钮");
  assert(out.includes("重新出方案（要动手）"), "只读单：保留重新出方案次按钮");
  assert(!out.includes('aria-label="只读单提醒"'), "只读单：提示牌应从呈现退场");
  assert(!out.includes("jiaoban-advice-only-banner"), "只读单：提示牌卡形钩子不应残留");
  assert(!out.includes("这单是只读的——AI 只看不改"), "只读单：提示牌文案不得换壳残留");
  // 意见块与只读动作机制共存。
  assert(out.includes("全局主管意见（批前边界）"), "意见块与只读单操作共存");
  assert(out.includes("好像对不上你的目标"), "意见块 mismatch 显示");
  assert(!out.includes("审批"), "词表无「审批」");
}

// 7b) 正常（有写根）+ looks_ok 意见共存：主按钮仍是[允许并开始]、[按我说的改]照旧。
{
  const ok = ready(record({ verdict: "looks_ok", summary: "对得上" }));
  const out = authorizeHtml(["/Users/yoyi/codex-workflow-mario-test"], ok);
  assert(out.includes("允许并开始"), "正常：主按钮原样");
  assert(out.includes("按我说的改"), "正常：次按钮原样");
  assert(!out.includes("这单是只读的"), "正常：无只读警条");
  assert(out.includes("全局主管意见（批前边界）"), "意见块与正常按钮共存");
  assert(out.includes("范围和你的目标对得上"), "意见块 looks_ok 显示");
}

// 7c) 意见缺席（boundaryOutcome=null·async 没到）→ 按钮区照常 + 常显位一行(07-16 用户终裁)。
{
  const out = authorizeHtml(["/Users/yoyi/codex-workflow-mario-test"], null);
  assert(out.includes("允许并开始"), "意见缺席：按钮区照常可批");
  assert(out.includes("全局主管意见还没到（不拦批）"), "意见缺席：常显位一行在");
}

console.log("[boundary-opinion] all assertions passed");
