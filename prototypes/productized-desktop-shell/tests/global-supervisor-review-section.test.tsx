// B1·全局主管复核区·离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// 覆盖 §4 四态：loading / 意见到（含建议按钮）/ 不可用+重试 / pass 绿行；+ 词表断言（无「审批」·
// 意见不是闸）+ 零渲染（没起）。组件无 hooks → 直接调函数拿元素树验 onClick 回调。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { JiaobanSupervisorReviewSection } from "../src/views/projects/ProjectJiaobanPanel";
import type { GlobalSupervisorReviewOutcome, GlobalSupervisorReviewRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[supervisor-review-section] ${message}`);
  }
}

function record(overrides: Partial<GlobalSupervisorReviewRecord>): GlobalSupervisorReviewRecord {
  return {
    review_id: "r1",
    project_id: "proj",
    workflow_id: "wf-1",
    chain_started_at: "1000",
    status: "ready",
    overall: "pass",
    summary: "",
    suggested_action: "none",
    human_note: "",
    tasks: [],
    unavailable_reason: null,
    model: "codex-cli-default",
    profile_version: "global-supervisor-profile.v1",
    created_at_ms: 1,
    updated_at_ms: 1,
    ...overrides,
  };
}

function ready(overrides: Partial<GlobalSupervisorReviewRecord>): GlobalSupervisorReviewOutcome {
  return { status: "ready", review: record(overrides), reason: null, warnings: [] };
}

const noop = () => {};

function html(
  loading: boolean,
  outcome: GlobalSupervisorReviewOutcome | null,
  onRetry: () => void = noop,
  onReplan: () => void = noop,
): string {
  return renderToStaticMarkup(
    <JiaobanSupervisorReviewSection loading={loading} outcome={outcome} onRetry={onRetry} onReplan={onReplan} />,
  );
}

// 1) loading：复核中文案（不影响交货措辞在）。
{
  const out = html(true, null);
  assert(out.includes("全局主管复核中"), "loading 态应显「复核中」");
  assert(out.includes("不影响交货"), "loading 应说明不挡交货");
}

// 2) 意见到 + suggested_action=replan：总判 + 每任务点评（黄牌 ⚠）+ [按建议打回重拆] 按钮回调可点。
{
  const clicks: string[] = [];
  const outcome = ready({
    overall: "needs_rework",
    summary: "有一单没做完",
    suggested_action: "replan",
    tasks: [
      { title: "建小游戏", verdict: "ok", comment: "对得上" },
      { title: "手动验收", verdict: "issue", comment: "worker 自报没验完" },
    ],
  });
  const element = JiaobanSupervisorReviewSection({
    loading: false,
    outcome,
    onRetry: noop,
    onReplan: () => clicks.push("replan"),
  });
  assert(element, "意见到应渲染");
  const out = renderToStaticMarkup(element);
  assert(out.includes("全局主管意见"), "标题词表");
  assert(out.includes("建议打回重拆"), "needs_rework 总判行");
  assert(out.includes("⚠ 手动验收：worker 自报没验完"), "issue 任务带 ⚠ 点评");
  assert(out.includes("建小游戏：对得上"), "ok 任务点评在");
  assert(out.includes("按建议打回重拆"), "replan 建议按钮在");
  // 元素树里找到 replan 按钮并点它（无 hooks 组件可直调）。
  const buttons: Array<{ props: { onClick?: () => void; children?: unknown } }> = [];
  const walk = (node: unknown) => {
    if (!node || typeof node !== "object") return;
    const el = node as { type?: unknown; props?: { children?: unknown; onClick?: () => void } };
    if (el.type === "button" && el.props) buttons.push(el as (typeof buttons)[number]);
    const children = el.props?.children;
    if (Array.isArray(children)) children.forEach(walk);
    else if (children) walk(children);
  };
  walk(element);
  const replanBtn = buttons.find((b) => String(b.props.children) === "按建议打回重拆");
  assert(replanBtn, "应找到打回按钮");
  replanBtn.props.onClick?.();
  assert(clicks.length === 1 && clicks[0] === "replan", "点击应走 onReplan 回调（现成用户动作）");
}

// 3) 不可用：人话原因 + [重试复核] 回调可点（绝不零出路）。
{
  const clicks: string[] = [];
  const outcome: GlobalSupervisorReviewOutcome = {
    status: "unavailable",
    review: null,
    reason: "codex 额度用完了，明天再试或升级订阅",
    warnings: [],
  };
  const element = JiaobanSupervisorReviewSection({
    loading: false,
    outcome,
    onRetry: () => clicks.push("retry"),
    onReplan: noop,
  });
  assert(element, "不可用应渲染（不是消失）");
  const out = renderToStaticMarkup(element);
  assert(out.includes("复核不可用"), "不可用词表");
  assert(out.includes("额度用完"), "人话原因透出");
  assert(out.includes("重试复核"), "重试按钮在");
  const walk = (node: unknown, fire: (el: { props: { onClick?: () => void; children?: unknown } }) => void) => {
    if (!node || typeof node !== "object") return;
    const el = node as { type?: unknown; props?: { children?: unknown; onClick?: () => void } };
    if (el.type === "button" && el.props) fire(el as never);
    const children = el.props?.children;
    if (Array.isArray(children)) children.forEach((c) => walk(c, fire));
    else if (children) walk(children, fire);
  };
  walk(element, (btn) => {
    if (String(btn.props.children) === "重试复核") btn.props.onClick?.();
  });
  assert(clicks.length === 1, "重试回调应被点到");
}

// 4) pass：绿行「没发现问题」+ 无建议按钮；human_verify：显亲验一句话。
{
  const passOut = html(false, ready({ overall: "pass", suggested_action: "none", summary: "都对得上" }));
  assert(passOut.includes("这轮没发现问题"), "pass 绿行");
  assert(!passOut.includes("按建议打回重拆"), "pass 无打回按钮");
  const verifyOut = html(
    false,
    ready({ overall: "needs_human_check", suggested_action: "human_verify", human_note: "打开 index.html 玩一遍" }),
  );
  assert(verifyOut.includes("建议你亲验：打开 index.html 玩一遍"), "human_verify 显 human_note");
}

// 5) 没起（outcome null 且不 loading）→ 零渲染（无本轮链/旧数据整区不出现）。
{
  assert(html(false, null) === "", "没起 → 零渲染");
}

// 6) 词表死线：全部状态输出不含「审批」（意见不是闸）；不露 thread_id/store 黑话。
{
  const all = [
    html(true, null),
    html(false, ready({ overall: "needs_rework", suggested_action: "replan", tasks: [{ title: "t", verdict: "issue", comment: "c" }] })),
    html(false, { status: "unavailable", review: null, reason: "x", warnings: [] }),
    html(false, ready({ overall: "pass" })),
  ].join("");
  assert(!all.includes("审批"), "词表：任何态不得出现「审批」");
  assert(!all.includes("sidecar") && !all.includes("store"), "词表：不露 store 黑话");
}

console.log("global-supervisor-review-section: 6 组离线 DOM 断言全过");
