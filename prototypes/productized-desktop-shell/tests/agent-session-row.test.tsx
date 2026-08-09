// 会话行(单行三元素)语义锁——07-15 真机走查第一单的修法拍板:
// ① 工作台任务文字章废除,只留第二颗色点(全称进悬停/读屏);
// ② 异常状态词(缺回放记录等)=彩色前缀并进标题一起截断,不再在徽章簇单独占位挤标题;
// ③ 行仍是可聚焦 <button>(键盘可及·与 offline-permission-dialog 的既有锁同向)。
// 轨道 minmax(0,1fr)/240px 默认栏宽是 CSS 面,DOM 断言锁不到,由本文件头注+styles.css 注释共同留痕。
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  agentRoleSessionContinuationBlockedReason,
  AgentRoleSessionReadBoundary,
  AgentSessionCenter,
} from "../src/views/agent/AgentConversationShell";
import { AgentSessionList } from "../src/views/agent/AgentSessionList";
import type { AgentRoleSessionReadState } from "../src/views/agent/useAgentSessionPage";
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

const quarantinedRead: AgentRoleSessionReadState = {
  status: "ready",
  project_locator: "/fixture/agent-project",
  directory: {
    request_nonce: "agent-directory-fixture",
    projection_revision: "directory:agent-fixture",
    entries: [{
      selection: "m3rs:agent-selection",
      role_session_id: "session:sha256:agent",
      session_revision: 1,
      labels: {
        role_label: "role:fixture",
        project_label: "project:fixture",
        object_label: "object:fixture",
        channel_label: "channel:fixture",
        permission_label: "permission:fixture",
      },
      session_state: "QUARANTINED",
      permission_state: "CURRENT",
      resolution_reason: "SESSION_QUARANTINED",
    }],
    next_cursor: null,
  },
  detail: {
    request_nonce: "agent-detail-fixture",
    selection: "m3rs:agent-selection",
    role_session_id: "session:sha256:agent",
    session_revision: 1,
    projection_revision: "1:1:fixture",
    labels: {
      role_label: "role:fixture",
      project_label: "project:fixture",
      object_label: "object:fixture",
      channel_label: "channel:fixture",
      permission_label: "permission:fixture",
    },
    session_state: "QUARANTINED",
    permission_state: "CURRENT",
    resolution_reason: "SESSION_QUARANTINED",
    context: {
      state: "AVAILABLE",
      retrieval_status: "COMPLETE",
      context_sources: [],
      knowledge_refs: [],
      gaps: [],
      source_links: [],
      request_more_material_available: false,
    },
    continuation: { state: "DISABLED", selector: null, reason: "SESSION_QUARANTINED" },
  },
  selected_selection: "m3rs:agent-selection",
  loading_more: false,
  selection_error: null,
  error: null,
  legacy_display_only: true,
};
assert(
  agentRoleSessionContinuationBlockedReason(quarantinedRead, "/fixture/agent-project") === "角色会话已隔离，不能续聊。",
  "quarantine 的服务端 DTO 必须关闭 Agent 续聊",
);
assert(
  agentRoleSessionContinuationBlockedReason({ ...quarantinedRead, project_locator: "/fixture/other-project" }, "/fixture/agent-project")
    === "当前显示会话与服务端角色会话项目不一致；暂不续聊。",
  "跨项目回包不得借 legacy SessionRecord 续聊",
);
assert(
  agentRoleSessionContinuationBlockedReason(
    {
      ...quarantinedRead,
      status: "error",
      detail: null,
      error: { code: "M3_BINDING_UNAVAILABLE", user_message: "历史会话仅供阅读，当前不能续聊。" },
    },
    "/fixture/agent-project",
  ) === "历史会话仅供阅读，当前不能续聊。",
  "read-model 失败不得回退本地会话缓存",
);

const readableRead: AgentRoleSessionReadState = {
  status: "ready",
  project_locator: "/fixture/agent-project",
  directory: {
    request_nonce: "agent-directory-readable",
    projection_revision: "directory:agent-readable",
    entries: [{
      selection: "m3rs:agent-readable-selection",
      role_session_id: "session:sha256:agent-readable",
      session_revision: 2,
      labels: {
        role_label: "角色标签",
        project_label: "项目标签",
        object_label: "对象标签",
        channel_label: "通道标签",
        permission_label: "权限标签",
      },
      session_state: "ACTIVE",
      permission_state: "CURRENT",
      resolution_reason: null,
    }],
    next_cursor: null,
  },
  detail: {
    request_nonce: "agent-detail-readable",
    selection: "m3rs:agent-readable-selection",
    role_session_id: "session:sha256:agent-readable",
    session_revision: 2,
    projection_revision: "2:1:fixture",
    labels: {
      role_label: "角色标签",
      project_label: "项目标签",
      object_label: "对象标签",
      channel_label: "通道标签",
      permission_label: "权限标签",
    },
    session_state: "ACTIVE",
    permission_state: "CURRENT",
    resolution_reason: null,
    context: {
      state: "AVAILABLE",
      retrieval_status: "COMPLETE",
      context_sources: ["上下文来源 A"],
      knowledge_refs: ["知识来源 A"],
      gaps: ["资料缺口 A"],
      source_links: [{ source_ref: "source:opaque", label: "来源链接 A" }],
      request_more_material_available: false,
    },
    continuation: { state: "AVAILABLE", selector: "m3rs:agent-continuation", reason: null },
  },
  selected_selection: "m3rs:agent-readable-selection",
  loading_more: false,
  selection_error: null,
  error: null,
  legacy_display_only: true,
};
const readableBoundaryMarkup = renderToStaticMarkup(
  <AgentRoleSessionReadBoundary
    roleSessionRead={readableRead}
    blockedReason={null}
    onSelectRoleSession={() => {}}
    onLoadMoreRoleSessions={() => {}}
  />,
);
for (const expected of [
  "服务端角色会话目录",
  "角色标签",
  "项目标签",
  "对象标签",
  "通道标签",
  "权限标签",
  "上下文来源 A",
  "知识来源 A",
  "资料缺口 A",
  "来源链接 A",
  "source:opaque",
  "服务端已签发可续聊状态",
]) {
  assert(readableBoundaryMarkup.includes(expected), `Agent 边界必须渲染当前 DTO 字段：${expected}`);
}
assert(
  !readableBoundaryMarkup.includes("m3rs:agent-continuation"),
  "Agent 边界不得把 opaque continuation selector 渲染到 markup",
);

const selectionRequiredRead: AgentRoleSessionReadState = {
  ...readableRead,
  status: "selection_required",
  detail: null,
  selected_selection: null,
  directory: {
    ...readableRead.directory!,
    next_cursor: "m3rs:agent-more-cursor",
    entries: [
      ...readableRead.directory!.entries,
      {
        selection: "m3rs:agent-other-selection",
        role_session_id: "session:sha256:agent-other",
        session_revision: 3,
        labels: {
          role_label: "角色标签 B",
          project_label: "项目标签",
          object_label: "对象标签 B",
          channel_label: "通道标签",
          permission_label: "权限标签",
        },
        session_state: "ACTIVE",
        permission_state: "CURRENT",
        resolution_reason: null,
      },
    ],
  },
};
assert(
  agentRoleSessionContinuationBlockedReason(selectionRequiredRead, "/fixture/agent-project")
    === "服务端返回多个角色会话；请先明确选择，历史会话仅供阅读。",
  "Agent 多条目录未选择时必须关闭 composer",
);
const selectionRequiredMarkup = renderToStaticMarkup(
  <AgentRoleSessionReadBoundary
    roleSessionRead={selectionRequiredRead}
    blockedReason={agentRoleSessionContinuationBlockedReason(selectionRequiredRead, "/fixture/agent-project")}
    onSelectRoleSession={() => {}}
    onLoadMoreRoleSessions={() => {}}
  />,
);
assert(
  selectionRequiredMarkup.includes('data-role-session-detail="unselected"')
    && selectionRequiredMarkup.includes("composer 保持关闭")
    && selectionRequiredMarkup.includes("角色标签 B"),
  "Agent 多条服务器目录必须可选择、未选择时不得显示可续聊 detail",
);
const composerSelectedSession = fixtureSession({
  thread_id: "t-role-session-selection-required",
  thread_source: "codex",
  project_root: "/fixture/agent-project",
});
const selectionRequiredCenterMarkup = renderToStaticMarkup(
  <AgentSessionCenter
    sessions={[composerSelectedSession]}
    selectedThreadId={composerSelectedSession.thread_id}
    selectedSession={composerSelectedSession}
    transcript={null}
    loadingThreadId={null}
    transcriptError={null}
    projectSessionCount={1}
    roleSessionRead={selectionRequiredRead}
    onOpenSession={() => {}}
    onRequestAction={() => {}}
  />,
);
assert(
  selectionRequiredCenterMarkup.includes('data-send-mode="decision-only"')
    && selectionRequiredCenterMarkup.includes("服务端返回多个角色会话；请先明确选择"),
  "Agent 多条目录未选择时，真实 composer 必须是 decision-only 而非 legacy 续聊",
);

console.log("agent-session-row: 会话行双色点、状态词并入标题、可及名与按钮语义离线断言全过");
