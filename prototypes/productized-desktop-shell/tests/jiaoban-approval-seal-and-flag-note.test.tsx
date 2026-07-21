// G3 盖章时刻离线 DOM 断言：方案卡章显隐/日期对平/fresh 动效类；交货卡 tone-yellow 三处
// 朱砂批注式（jaoban-flag-note）+ 概览行 aria-label「这单概览」+ tone-red 仍有形章。
// renderToStaticMarkup·同现有 harness（2026-07-20·包 §三.6）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanAuthorizeState } from "../src/views/projects/jiaoban/JiaobanAuthorizeStates";
import { JiaobanAcceptanceEvidence } from "../src/views/projects/jiaoban/JiaobanDoneStates";
import { JiaobanDoneState, JiaobanStepReportList } from "../src/views/projects/ProjectJiaobanPanel";
import type { AutoAdvanceRoleLoopOutcome, DirectorChainStep, ProjectConsultationProposal } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[jiaoban-approval-seal] ${message}`);
  }
}

const noop = () => {};

function proposalWith(status: string, updatedAtMs: number): ProjectConsultationProposal {
  return {
    proposal_id: `proposal-seal-${status}`,
    schema_version: "project_consultation_proposal.v1",
    project_id: "project:seal",
    workflow_id: "workflow:seal",
    title: "盖章测试单",
    user_goal: "验证盖章时刻。",
    goal_summary: "验证盖章时刻",
    proposed_steps: ["目标文件：src/views/projects/ProjectJiaobanPanel.tsx"],
    scope_draft: {
      allowed_role_ids: [],
      allowed_agent_ids: [],
      allowed_read_roots: ["/tmp/seal"],
      allowed_write_roots: ["/tmp/seal"],
      allowed_tools: [],
      allowed_checks: [],
      allowed_task_package_kinds: [],
      stop_conditions: [],
    },
    risks: [],
    worker_acceptance_criteria: ["断言全绿"],
    control_core_acceptance_criteria: [],
    supervisor_acceptance_criteria: [],
    acceptance_criteria: [],
    status,
    created_by_role: "project_consultant",
    created_at_ms: updatedAtMs - 3600_000,
    updated_at_ms: updatedAtMs,
    suggest_workflow: false,
  } as ProjectConsultationProposal;
}

function renderAuthorize(proposal: ProjectConsultationProposal, sealFresh?: boolean): string {
  return renderToStaticMarkup(
    <JiaobanAuthorizeState
      proposal={proposal}
      proposalIsStale={false}
      amendment=""
      onAmend={noop}
      onAuthorizeAndStart={noop}
      onRePlan={noop}
      onDecline={noop}
      starting={false}
      consultLoading={false}
      consultError={null}
      howRunSummary="经典状态机"
      onShowGovernance={noop}
      onShowHowRun={noop}
      boundaryLoading={false}
      boundaryOutcome={null}
      onBoundaryRetry={noop}
      readOnly={false}
      {...(sealFresh === undefined ? {} : { sealFresh })}
    />,
  );
}

// 章面 MM-DD 期望式（与组件同口径：本地时·补零）。
function expectedSealDate(atMs: number): string {
  const date = new Date(atMs);
  return `${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

const confirmedAt = Date.UTC(2026, 6, 20, 14, 5);

// 1) pending_user_confirmation → 无章。
{
  const html = renderAuthorize(proposalWith("pending_user_confirmation", confirmedAt));
  assert(!html.includes("jiaoban-seal"), "pending 方案卡不许出章");
  assert(!html.includes("已批准"), "pending 方案卡不许有「已批准」");
}

// 2) user_confirmed → 有章：「已批准」+「SYN · 」+ MM-DD 与 updated_at_ms 对平；缺省 historical 静态（无 is-fresh）。
{
  const html = renderAuthorize(proposalWith("user_confirmed", confirmedAt));
  assert(html.includes("jiaoban-seal"), "confirmed 方案卡必须有章");
  assert(html.includes("已批准"), "章面必须有「已批准」");
  assert(html.includes(`SYN · ${expectedSealDate(confirmedAt)}`), `章面日期须=SYN · ${expectedSealDate(confirmedAt)}（updated_at_ms 对平）`);
  assert(!html.includes("is-fresh"), "historical（缺省/重进）态静态，不带 is-fresh");
}

// 3) fresh 态（刚批首现）→ 章与卡都带 is-fresh 动效类。
{
  const html = renderAuthorize(proposalWith("user_confirmed", confirmedAt), true);
  assert(html.includes("jiaoban-seal is-fresh"), "fresh 态章带 is-fresh");
  assert(html.includes("jiaoban-authorize is-fresh"), "fresh 态卡带 is-fresh（thud）");
}

// 4) 其余五态抽样（changes_requested）→ 无章。
{
  const html = renderAuthorize(proposalWith("changes_requested", confirmedAt));
  assert(!html.includes("jiaoban-seal"), "非 confirmed 态不许出章");
}

function step(overrides: Partial<DirectorChainStep>): DirectorChainStep {
  return {
    planned_task_id: overrides.planned_task_id ?? "t1",
    title: overrides.title ?? "任务",
    state: overrides.state ?? "completed",
    report_summary: overrides.report_summary,
    report_warning: overrides.report_warning,
    report_status: overrides.report_status,
  };
}

const greenA = step({ planned_task_id: "a", title: "搭骨架", report_summary: "建好了", report_status: "done" });
const yellowB = step({ planned_task_id: "b", title: "接业务", report_summary: "只做了一半", report_status: "partial" });
const failedC = step({ planned_task_id: "c", title: "写用例", state: "failed" });

// 5) 步条：yellow→jiaoban-flag-note 批注式且零 spec-pill-warn；red 仍 spec-pill-bad；green 仍 spec-pill-ok。
{
  const html = renderToStaticMarkup(<JiaobanStepReportList steps={[greenA, yellowB, failedC]} />);
  assert(html.includes("jiaoban-flag-note"), "黄牌步条必须改朱砂批注");
  assert(!html.includes("spec-pill-warn"), "交货卡步条不许再出 spec-pill-warn");
  assert(html.includes("spec-pill-bad"), "失败步条仍是有形章 spec-pill-bad");
  assert(html.includes("spec-pill-ok"), "绿步条仍是 spec-pill-ok");
}

// 6) 全绿链：零 flag-note。
{
  const html = renderToStaticMarkup(<JiaobanStepReportList steps={[greenA]} />);
  assert(!html.includes("jiaoban-flag-note"), "全绿链零批注");
}

// 7) 闸条：「⚠ 要改」批注式；「✗ 卡住」仍 spec-pill-bad。
{
  const derivedWorkflow = {
    result_summary: {
      project_id: "project:seal",
      workflow_id: "workflow:seal",
      final_review_status: "pending",
      user_decision_status: "pending",
      stage_c_acceptance: {
        project_id: "project:seal",
        workflow_id: "workflow:seal",
        gates: [
          { gate_id: "g1", label: "验收检查", status: "needs_changes", reason: "自检没全过", evidence_refs: [] },
          { gate_id: "g2", label: "控制核心", status: "blocked", reason: "被挡", evidence_refs: [] },
        ],
        final_review_status: "pending",
        user_decision_status: "pending",
        accepted_as_stage_c_complete: false,
        deferred_items: [],
        open_blockers: [],
        warnings: [],
      },
      open_issues: [],
      deferred_items: [],
      warnings: [],
    },
  };
  const html = renderToStaticMarkup(
    <JiaobanAcceptanceEvidence derivedWorkflow={derivedWorkflow as never} />,
  );
  assert(html.includes("jiaoban-flag-note") && html.includes("⚠ 要改"), "闸条黄牌「⚠ 要改」批注式");
  assert(!html.includes("spec-pill-warn"), "闸条不许再出 spec-pill-warn");
  assert(html.includes("spec-pill-bad"), "闸条「✗ 卡住」仍是 spec-pill-bad");
}

// 8) 交货卡概览行：「⚠ N 项要看一眼」批注式 + PillRow aria-label「这单概览」逐字回；red/green 断言不变。
{
  const outcome: AutoAdvanceRoleLoopOutcome = {
    stage: "completed",
    planned_task_count: 2,
    prepared_count: 2,
    needs_binding_count: 0,
    blocked_count: 0,
    message: "完整完成",
    chain_outcome: {
      total: 2,
      dispatched: 2,
      completed: 2,
      skipped: 0,
      chain_run_id: "chain-seal",
      steps: [greenA, yellowB],
      warnings: [],
      stopped_reason: null,
    },
    stop_reason: null,
  };
  const html = renderToStaticMarkup(
    <JiaobanDoneState
      outcome={outcome}
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
  assert(html.includes('aria-label="这单概览"'), "概览行 PillRow 必须带回「这单概览」（G2 挂账清）");
  assert(html.includes("⚠ 1 项要看一眼"), "概览行黄牌计数人话不变");
  assert(!html.includes("spec-pill-warn"), "概览行黄牌批注式·零 spec-pill-warn");
  assert(html.includes("完成 2 步"), "概览行绿 pill 文案不变");
  assert(html.includes("已交货"), "交货卡头 pill 文案不变");
}

console.log("jiaoban-approval-seal-and-flag-note: 章显隐/日期/fresh 类 + 黄牌三处批注式 + aria-label 断言全过");
