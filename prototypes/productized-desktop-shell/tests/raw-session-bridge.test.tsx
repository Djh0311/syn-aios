// 交办「看原始对话」桥·离线 DOM 断言（renderToStaticMarkup·同现有 harness 风格）。
// 覆盖 §4：① 接现有 → 收纳行入口在 + 点击回调收到该 thread_id；② 开个新的 → 批卡不显入口；
// ③ 交货/各脸三态（existing→看原始对话 / 哨兵+latestSession→看最近对话 / 皆无→零渲染）+ 诚实词表。
// JiaobanRawSessionLink 无 hooks → 直接调函数拿元素树验 onClick（带 hooks 的 picker 用 renderToStaticMarkup 静态验）。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  JiaobanRawSessionLink,
  JiaobanSessionPicker,
  NEW_SESSION_CHOICE,
} from "../src/views/projects/ProjectJiaobanPanel";
import type { SessionRecord } from "../src/lib/workbenchCoreTypes";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[raw-session-bridge] ${message}`);
  }
}

function sessionFixture(overrides: Partial<SessionRecord> = {}): SessionRecord {
  return {
    thread_id: "thread-real-1",
    title: "改登录页",
    archived: false,
    rollout_exists: true,
    warnings: [],
    updated_at_ms: 1000,
    ...overrides,
  };
}

// 1) existing（真 thread_id）→ 「看原始对话」+ 点击回调收到该 id + 不露 id。
{
  const captured: string[] = [];
  const el = JiaobanRawSessionLink({
    sessionChoice: "thread-real-1",
    onOpenAgentSession: (id) => captured.push(id),
  });
  assert(el, "existing → 入口在");
  assert(el.props.children === "看原始对话", "existing → 主词「看原始对话」");
  const out = renderToStaticMarkup(el);
  assert(!out.includes("thread-real-1"), "词表：不露 thread_id");
  el.props.onClick();
  assert(captured.length === 1 && captured[0] === "thread-real-1", "点击 → 回调收到该 thread_id");
}

// 2) 哨兵单（新会话）+ latestSession → 「看最近对话」+ 点击收到 latest + 不吹大成「本单/原始对话」。
{
  const captured: string[] = [];
  const el = JiaobanRawSessionLink({
    sessionChoice: NEW_SESSION_CHOICE,
    latestSessionThreadId: "thread-recent",
    onOpenAgentSession: (id) => captured.push(id),
  });
  assert(el, "哨兵+latestSession → 入口在（兜底）");
  assert(el.props.children === "看最近对话", "哨兵 → 兜底词「看最近对话」");
  const out = renderToStaticMarkup(el);
  assert(!out.includes("本单对话") && !out.includes("原始对话"), "词表：哨兵不吹大成「本单/原始对话」");
  assert(!out.includes("thread-recent"), "词表：不露 thread_id");
  el.props.onClick();
  assert(captured[0] === "thread-recent", "点击 → 回调收到 latest thread_id");
}

// 3) 零渲染三态：哨兵+无兜底 / 未定 / 无回调（降级）→ 一律不显。
{
  assert(
    JiaobanRawSessionLink({ sessionChoice: NEW_SESSION_CHOICE, onOpenAgentSession: () => {} }) === null,
    "哨兵 + 无 latestSession → 零渲染",
  );
  assert(
    JiaobanRawSessionLink({ sessionChoice: null, onOpenAgentSession: () => {} }) === null,
    "未定（null）→ 零渲染",
  );
  assert(
    JiaobanRawSessionLink({ sessionChoice: "thread-real-1", onOpenAgentSession: undefined }) === null,
    "无 onOpenAgentSession → 不显（降级·不硬塞死链接）",
  );
}

// 4) §4①②：批卡收纳行（JiaobanSessionPicker 收起态）——接现有显入口 / 开个新的不显。
// 分工说明（审查线逮到过「上游漏传」假绿，这里讲清防线）：本组验 picker 的组件契约——
//   · 收到 onOpenAgentSession（批卡真实用法·修B 后 AuthorizeState 真的透传）→ 接现有显入口 / 新建不显；
//   · 不收（卡住脸真实用法·picker 行内不传·走面级入口）→ 不显。
// 两支都是真实生产路径，非假绿。而「Browser→AuthorizeState→picker」整条透传链离线跑不了
// （主组件带 tauri 副作用·codebase 惯例只离线测子组件）→ 由 **tsc 必填** 兜底：
// AuthorizeState.onOpenAgentSession 设必填，上游任一处漏传即编译错（就是本次逮到的那个 bug）。
{
  const sessions = [sessionFixture()];
  const existingHtml = renderToStaticMarkup(
    <JiaobanSessionPicker
      sessions={sessions}
      sessionChoice="thread-real-1"
      onSessionChoiceChange={() => {}}
      onOpenAgentSession={() => {}}
    />,
  );
  assert(existingHtml.includes("看原始对话"), "§4①：接现有 → 收纳行入口在");
  assert(existingHtml.includes("jiaoban-session-summary-row"), "收纳行套了并排容器（布局 class 在）");

  const newSessionHtml = renderToStaticMarkup(
    <JiaobanSessionPicker
      sessions={sessions}
      sessionChoice={NEW_SESSION_CHOICE}
      onSessionChoiceChange={() => {}}
      onOpenAgentSession={() => {}}
    />,
  );
  assert(!newSessionHtml.includes("看原始对话"), "§4②：开个新的 → 收纳行不显入口（会话还没生·诚实）");

  // 卡住脸用法：picker 不传 onOpenAgentSession（走面级入口）→ 行内也不显。
  const noCbHtml = renderToStaticMarkup(
    <JiaobanSessionPicker
      sessions={sessions}
      sessionChoice="thread-real-1"
      onSessionChoiceChange={() => {}}
    />,
  );
  assert(!noCbHtml.includes("看原始对话"), "picker 无回调（卡住脸用法）→ 行内不显（防一脸双入口）");
}

console.log("raw-session-bridge: 4 组离线 DOM 断言全过");
