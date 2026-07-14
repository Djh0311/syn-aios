import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanBlockedState,
  JiaobanDoneState,
  JiaobanPlanPreviewCanvas,
  JiaobanWaitingDecisionState,
  jiaobanPhaseForOutcome,
  jiaobanRuntimeNodeStates,
  jiaobanStageFromChainOutcome,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { AutoAdvanceRoleLoopOutcome, DirectorChainOutcome } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[jiaoban-chain-result-semantics] ${message}`);
}

const noop = () => {};
const chain = (stoppedReason: string | null): DirectorChainOutcome => ({
  total: 1,
  dispatched: 1,
  completed: stoppedReason === null ? 1 : 0,
  skipped: 0,
  chain_run_id: "chain-result-semantics",
  steps: [],
  warnings: [],
  stopped_reason: stoppedReason,
});
const outcome = (stage: string, stoppedReason: string | null): AutoAdvanceRoleLoopOutcome => ({
  stage,
  planned_task_count: 1,
  prepared_count: 1,
  needs_binding_count: 0,
  blocked_count: 0,
  message: stoppedReason ?? "完整完成",
  chain_outcome: chain(stoppedReason),
  stop_reason: stoppedReason,
});

assert(jiaobanStageFromChainOutcome(chain(null)) === "completed", "无停因必须四分为 completed");
assert(
  jiaobanStageFromChainOutcome(chain("user_stop_requested")) === "interrupted",
  "用户停下必须四分为 interrupted",
);
assert(
  jiaobanStageFromChainOutcome(chain("fail_stop:node_error:worker")) === "failed",
  "失败停因必须四分为 failed",
);
assert(
  jiaobanStageFromChainOutcome(chain("waiting_decision:worker_help:缺权限")) === "waiting_decision",
  "worker 求助必须四分为 waiting_decision",
);

assert(jiaobanPhaseForOutcome(outcome("completed", null)) === "done", "只有 completed 进 done");
assert(jiaobanPhaseForOutcome(outcome("failed", "fail_stop:worker")) === "blocked", "failed 进卡住脸");
assert(
  jiaobanPhaseForOutcome(outcome("interrupted", "user_stop_requested")) === "blocked",
  "interrupted 进可续脸",
);
assert(
  jiaobanPhaseForOutcome(outcome("waiting_decision", "waiting_decision:worker_help")) === "waiting_decision",
  "waiting_decision 进独立决定脸",
);

const doneOutput = renderToStaticMarkup(
  <JiaobanDoneState
    outcome={outcome("completed", null)}
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
assert(doneOutput.includes('aria-label="做好了"') && doneOutput.includes("已交货"), "completed 才显示交货态");

const blockedBase = {
  error: null,
  planIsConfirmed: true,
  sessions: [],
  sessionChoice: null,
  onSessionChoiceChange: noop,
  onContinueRun: noop,
  onRePlan: noop,
  starting: false,
  onOpenWorkflow: null,
  latestSessionThreadId: null,
};
const failedOutput = renderToStaticMarkup(
  JiaobanBlockedState({ ...blockedBase, outcome: outcome("failed", "fail_stop:worker 执行失败") }),
);
assert(failedOutput.includes("卡住了"), "failed 应显示卡住脸");
assert(!failedOutput.includes("做好了") && !failedOutput.includes("已交货"), "failed 绝不能伪装做好了/已交货");

const interruptedOutput = renderToStaticMarkup(
  JiaobanBlockedState({ ...blockedBase, outcome: outcome("interrupted", "user_stop_requested") }),
);
assert(interruptedOutput.includes("已停下·可接着跑"), "interrupted 应显示可接着跑");
assert(interruptedOutput.includes("接着跑") && interruptedOutput.includes("重新说目标"), "停下脸不得零按钮");

const archivedOutput = renderToStaticMarkup(
  JiaobanBlockedState({ ...blockedBase, outcome: outcome("interrupted", "archived:waiting_decision_action") }),
);
assert(archivedOutput.includes("这单已结束") && archivedOutput.includes("已结束"), "archived 应显示真实结束态");
assert(!archivedOutput.includes("已交货") && !archivedOutput.includes("接着跑（方案已批过"), "结束态不是交货也不可续跑");

const sameClickFailureOutput = renderToStaticMarkup(
  <JiaobanBlockedState
    {...blockedBase}
    outcome={null}
    error="同击边界批准自动补记失败：plan_authorization_store_locked"
    onOpenWorkflow={noop}
  />,
);
assert(sameClickFailureOutput.includes("同击边界批准自动补记失败"), "补记失败应原话进入卡住脸");
assert(sameClickFailureOutput.includes("接着跑（方案已批过"), "补记失败后应保留用户点击续跑入口");

const helpText = "worker 求助：缺少读取 /secure 的权限；方向风险：无法核验真实配置。";
const waitingOutput = renderToStaticMarkup(
  JiaobanWaitingDecisionState({
    reason: helpText,
    actionsReady: true,
    starting: false,
    error: null,
    onContinue: noop,
    onChangeSession: noop,
    onRework: noop,
    onArchive: noop,
  }),
);
assert(waitingOutput.includes(helpText), "待决定脸必须保留 worker 求助原文");
for (const label of ["让它继续（按现状态）", "换个新会话重做", "退回主管重拆", "结束这单"]) {
  assert(waitingOutput.includes(label), `待决定脸应显示「${label}」`);
}
assert(waitingOutput.includes("未自动重跑"), "待决定脸必须明确未自动重跑");

const runtime = jiaobanRuntimeNodeStates(
  [{ preview_node_id: "waiting-node", title: "等待决定", depends_on: [] }],
  { chain_run_id: "waiting-chain", state: "waiting_decision", nodes: [{ node_id: "waiting-node", state: "waiting_decision" }] },
);
assert(runtime["waiting-node"]?.state === "waiting_decision", "画布 waiting_decision 不得回落 pending");
const waitingCanvas = renderToStaticMarkup(
  <JiaobanPlanPreviewCanvas
    nodes={[{ preview_node_id: "waiting-node", title: "等待决定", depends_on: [] }]}
    bindings={[{ preview_node_id: "waiting-node", session_choice: "new" }]}
    sessions={[]}
    waitingForPreview={false}
    previewError={null}
    previewWarnings={[]}
    readOnly
    runtimeNodeStates={runtime}
    onBindingChange={noop}
    onRetryPreview={noop}
    onOpenAgentSession={noop}
  />,
);
assert(waitingCanvas.includes("任务 · 待你决定"), "画布 waiting_decision 必须显示待你决定标签");

// ③ 卡住态乙型（定稿 F·2026-07-14）：真出问题那一支给「直接回它一句」回话框。
// follow-up 后端通道未就绪 → 形态立住但整块 disabled + 人话原因（宪法 §四.3 禁死按钮：不可用必给原因）。
{
  const typeB = renderToStaticMarkup(
    JiaobanBlockedState({ ...blockedBase, outcome: outcome("failed", "fail_stop:styles.css 里没找到计分板挂载点") }),
  );
  assert(typeB.includes("⚠ 卡住了"), "乙型=真出问题那一支");
  assert(typeB.includes('aria-label="直接回它一句"'), "乙型应有回话框");
  assert(typeB.includes("发送并继续"), "乙型应有[发送并继续]");
  assert(typeB.includes("回话通道还在接线"), "通道未接通必须给人话原因，不许静默死按钮");
  assert(/<textarea[^>]*disabled/.test(typeB), "通道未接通时回话框应 disabled");
  assert(
    /<button[^>]*disabled[^>]*>发送并继续<\/button>/.test(typeB),
    "通道未接通时[发送并继续]应 disabled，零假按钮",
  );
  assert(typeB.includes("接着跑（方案已批过"), "通道没通时仍有能点的主路径——永不冻");

  // 已结束/已停下两支不是「出问题」，不该出现回话框。
  const archivedTypeA = renderToStaticMarkup(
    JiaobanBlockedState({ ...blockedBase, outcome: outcome("interrupted", "archived:waiting_decision_action") }),
  );
  assert(!archivedTypeA.includes('aria-label="直接回它一句"'), "已结束态不给回话框");
  assert(!archivedTypeA.includes("发送并继续"), "已结束态不给[发送并继续]");
}

console.log("jiaoban-chain-result-semantics: 四分、四脸、待决定四按钮、archived 真终态和卡住乙型回话框断言全过");
