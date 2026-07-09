import { emptySnapshot } from "./emptySnapshot";
import type { CodexSessionPage, CodexSessionPageRequest, CodexTranscript, ProjectRecord, SessionRecord, WorkbenchSnapshot } from "./types";

export const previewProjectRoot = "/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell";
const previewUpdatedAt = Date.parse("2026-06-19T10:00:00.000Z");

const previewProject: ProjectRecord = {
  project_root: previewProjectRoot,
  name: "productized-desktop-shell",
  active_hint: true,
  thread_count: 3,
  active_thread_count: 3,
  archived_thread_count: 0,
  latest_updated_at_ms: previewUpdatedAt,
  authority_files: [],
  handoff_files: [],
  evidence_files: [],
  harness_candidates: [],
  harness_resources: [],
  context_warnings: ["browser_preview_fixture_only"],
  warnings: [],
};

export const browserPreviewSessions: SessionRecord[] = [
  {
    thread_id: "browser-preview-thread-layout",
    title: "智能体布局打磨",
    project_root: previewProjectRoot,
    updated_at_ms: previewUpdatedAt,
    archived: false,
    rollout_exists: true,
    rollout_path: "/browser-preview/rollouts/layout.jsonl",
    model: "gpt-5-codex",
    reasoning_effort: "medium",
    thread_source: "codex",
    warnings: [],
  },
  {
    // 工作台绑定的任务会话（codex exec 建·has_user_event=0·经 store 绑定合并进列表）——带「工作台任务」徽标。
    thread_id: "browser-preview-thread-workbench",
    title: "交办任务专用会话：搭骨架",
    project_root: previewProjectRoot,
    updated_at_ms: previewUpdatedAt - 30 * 60 * 1000,
    archived: false,
    rollout_exists: true,
    rollout_path: "/browser-preview/rollouts/workbench-task.jsonl",
    model: "gpt-5-codex",
    reasoning_effort: "medium",
    thread_source: "codex",
    warnings: [],
    workbench_bound: true,
  },
  {
    thread_id: "browser-preview-thread-polling",
    title: "轮询恢复与超时",
    project_root: previewProjectRoot,
    updated_at_ms: previewUpdatedAt - 60 * 60 * 1000,
    archived: false,
    rollout_exists: true,
    rollout_path: "/browser-preview/rollouts/polling.jsonl",
    model: "gpt-5-codex",
    reasoning_effort: "medium",
    thread_source: "codex",
    warnings: [],
  },
  {
    thread_id: "browser-preview-thread-missing",
    title: "缺回放记录示例",
    project_root: previewProjectRoot,
    updated_at_ms: previewUpdatedAt - 2 * 60 * 60 * 1000,
    archived: false,
    rollout_exists: false,
    rollout_path: null,
    model: "gpt-5-codex",
    reasoning_effort: "low",
    thread_source: "codex",
    warnings: ["rollout_missing_on_disk"],
  },
];

export const browserPreviewSnapshot: WorkbenchSnapshot = {
  ...emptySnapshot,
  summary: {
    ...emptySnapshot.summary,
    generated_at: "2026-06-19T10:00:00.000Z",
    project_count: 1,
    session_count: browserPreviewSessions.length,
    warning_count: 1,
  },
  projects: [previewProject],
  sessions: browserPreviewSessions,
  agent_adapters: [
    {
      adapter_id: "codex-local",
      agent_type: "codex",
      agent_id: "codex-local",
      display_name: "Codex",
      provider: "OpenAI",
      status: "available",
      permission_level: "read_only",
      source_kind: "frontend_read_model",
      capabilities: [],
      implemented_action_kinds: ["session_index_read", "session_transcript_read"],
      hidden_unimplemented_adapters: [],
      warnings: [],
      execution_status: "not_implemented",
      credential_status: "not_read",
      model_access_status: "local_read_model_only",
      permission_boundary: "browser preview fixture; no real execution",
      requires_user_setup: false,
    },
  ],
  provider_availability: [
    {
      adapter_id: "codex-local",
      provider_id: "openai",
      provider_label: "OpenAI",
      provider_kind: "local_cli",
      adapter_status: "available",
      availability_status: "available_readonly",
      credential_status: "not_required_by_workbench",
      model_status: "local_cli_managed",
      external_call_status: "not_needed_for_readonly",
      cost_risk_status: "none_known",
      user_visible_reason: "浏览器预览示例数据；不执行真实调用。",
      safe_to_display: true,
      requires_user_configuration: false,
      requires_future_task: false,
      warnings: [],
    },
  ],
};

export function browserPreviewTranscript(threadId: string): CodexTranscript {
  const session = browserPreviewSessions.find((item) => item.thread_id === threadId) ?? browserPreviewSessions[0];
  return {
    thread_id: session.thread_id,
    rollout_path: session.rollout_path ?? "/browser-preview/rollouts/layout.jsonl",
    project_path: session.project_root,
    title: session.title,
    created_at_ms: previewUpdatedAt - 10 * 60 * 1000,
    updated_at_ms: session.updated_at_ms,
    viewer_boundary: {
      view_kind: "browser_preview_session_history",
      reads_session_history: true,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "browser_preview_only",
      warnings: ["browser_preview_fixture_not_real_session"],
    },
    events: [
      {
        event_id: `${session.thread_id}:user:1`,
        timestamp: "2026-06-19T09:55:00.000Z",
        event_type: "user_message",
        actor: "user",
        role: "user",
        text: "把智能体页改成 Codex 式布局，左边会话列表，中间对话，底部输入。",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: `${session.thread_id}:status:1`,
        timestamp: "2026-06-19T09:56:00.000Z",
        event_type: "tool_call",
        actor: "assistant",
        role: "assistant",
        tool_name: "shell",
        text: "读取 AgentConversationShell / AgentSessionList / AgentChatComposer",
        metadata: { raw_type: "response_item", payload_type: "function_call" },
        warnings: [],
      },
      {
        event_id: `${session.thread_id}:assistant:1`,
        timestamp: "2026-06-19T09:58:00.000Z",
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: "已移除旧的软件层和顶部下拉条，左栏现在负责新对话、搜索和项目分组。发送链路、Stop、轮询恢复和超时逻辑保持原样。",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
    ],
    summary: {
      total_events: 3,
      event_type_counts: { user_message: 1, tool_call: 1, assistant_message: 1 },
      unknown_event_count: 0,
      warning_count: 0,
      encrypted_content_event_count: 0,
      sensitive_like_event_count: 0,
    },
    pagination: null,
    warnings: ["browser_preview_fixture_not_real_session"],
    source_stats: {},
  };
}

export function browserPreviewSessionPage(request: CodexSessionPageRequest): CodexSessionPage {
  const query = request.query?.trim().toLowerCase() ?? "";
  const sessions = query
    ? browserPreviewSessions.filter((session) =>
        [session.thread_id, session.title, session.project_root ?? ""].some((value) => value.toLowerCase().includes(query)),
      )
    : browserPreviewSessions;
  return {
    sessions,
    page_size: request.page_size ?? 100,
    offset: request.offset ?? 0,
    has_more: false,
    include_archived: Boolean(request.include_archived),
    archived_only: Boolean(request.archived_only),
    warnings: ["browser_preview_fixture_only"],
    source: "browser_preview",
  };
}
