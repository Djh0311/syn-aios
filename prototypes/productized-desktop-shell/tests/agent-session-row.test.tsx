// 会话行(单行三元素)语义锁——07-15 真机走查第一单的修法拍板:
// ① 工作台任务文字章废除,只留第二颗色点(全称进悬停/读屏);
// ② 异常状态词(缺回放记录等)=彩色前缀并进标题一起截断,不再在徽章簇单独占位挤标题;
// ③ 行仍是可聚焦 <button>(键盘可及·与 offline-permission-dialog 的既有锁同向)。
// 轨道 minmax(0,1fr)/240px 默认栏宽是 CSS 面,DOM 断言锁不到,由本文件头注+styles.css 注释共同留痕。
import { renderToStaticMarkup } from "react-dom/server.browser";
import { AgentSessionList } from "../src/views/agent/AgentSessionList";
import type { SessionRecord } from "../src/lib/types";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function fixtureSession(over: Record<string, unknown>): SessionRecord {
  return {
    thread_id: "t-fixture",
    title: "未命名会话",
    updated_at_ms: 1783900000000,
    rollout_exists: true,
    rollout_path: "fixture.jsonl",
    archived: false,
    warnings: [],
    workbench_bound: false,
    project_root: "/p",
    ...over,
  } as unknown as SessionRecord;
}

const sessions = [
  fixtureSession({ thread_id: "t-wb", title: "把 test.txt 压缩成 8 字节并复核", workbench_bound: true }),
  fixtureSession({ thread_id: "t-miss", title: "帮我看看这个报错", rollout_exists: false, rollout_path: null }),
  fixtureSession({ thread_id: "t-plain", title: "查询天气" }),
];

const markup = renderToStaticMarkup(
  <AgentSessionList
    sessions={sessions}
    visibleSessions={sessions}
    groups={[{ key: "/p", label: "/p", sessions }]}
    effectiveGroupBy="project"
    selectedThreadId="t-wb"
    filteredOutCount={0}
    searchQuery=""
    readFilter="all"
    selectedCollapsedGroup={null}
    collapsedKeys={new Set<string>()}
    title="会话"
    onSearchQueryChange={() => {}}
    onReadFilterChange={() => {}}
    onToggleGroup={() => {}}
    onOpenSession={() => {}}
  />,
);

assert(!markup.includes(">工作台任务</span>"), "工作台任务文字章应已废除(07-15 拍:徽标只留颜色点)");
assert(markup.includes('class="sc-dot workbench"'), "工作台会话应有第二颗色点");
assert(markup.includes('aria-label="工作台任务"'), "色点必须带可及名(读屏/悬停出全称)");
assert(
  markup.includes('title="工作台绑定的任务会话（codex exec 建·经工作流节点绑定）"'),
  "色点悬停必须给全称解释,不许变成无名色块",
);
assert(
  /class="sc-title"><span class="sc-status warn">缺回放记录 · <\/span>帮我看看这个报错/.test(markup),
  "异常状态词应为彩色前缀并进标题一起截断,不再单独占位",
);
assert(
  !/sc-badge"><span class="sc-dot[^>]*><span class="sc-status/.test(markup),
  "徽章簇里不许再出现状态词(它已并入标题)",
);
assert(markup.includes("<button"), "会话行必须仍是可聚焦 button");
assert(markup.includes('class="sc-time"'), "时间元素必须在行内");

console.log("agent-session-row: 会话行双色点、状态词并入标题、可及名与按钮语义离线断言全过");
