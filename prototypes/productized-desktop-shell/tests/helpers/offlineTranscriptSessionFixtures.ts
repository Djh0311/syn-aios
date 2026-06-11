import type {
  CodexTranscript,
  CodexTranscriptEvent,
  ProjectRecord,
  SessionRecord,
} from "../../src/lib/types";

export function transcriptCleaningFixtures(): {
  events: CodexTranscriptEvent[];
  mixedStream: CodexTranscriptEvent[];
  noisyFallback: CodexTranscriptEvent[];
  onlyResponseItems: CodexTranscriptEvent[];
} {
  return {
    events: [
      {
        event_id: "e1",
        event_type: "user_message",
        text: "<environment_context>cwd=/x system prompt 注入</environment_context>",
        metadata: { raw_type: "response_item" },
        warnings: [],
      },
      {
        event_id: "e2",
        event_type: "user_message",
        text: "帮我修复登录 bug",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "e3",
        event_type: "assistant_message",
        text: "好的，我先看一下代码",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "e4",
        event_type: "assistant_message",
        text: "好的，我先看一下代码",
        metadata: { raw_type: "response_item" },
        warnings: [],
      },
      {
        event_id: "e5",
        event_type: "tool_call",
        text: "",
        metadata: { raw_type: "response_item" },
        warnings: [],
      },
      {
        event_id: "e6",
        event_type: "assistant_message",
        text: "   ",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
    ],
    mixedStream: [
      { event_id: "m1", event_type: "user_message", text: "用户消息在 event_msg", metadata: { raw_type: "event_msg" }, warnings: [] },
      { event_id: "m2", event_type: "assistant_message", text: "Agent 回复在 response_item", metadata: { raw_type: "response_item" }, warnings: [] },
      { event_id: "m3", event_type: "tool_call", text: "tool", metadata: { raw_type: "response_item" }, warnings: [] },
    ],
    noisyFallback: [
      { event_id: "n1", event_type: "user_message", text: "<environment_context>cwd=/tmp</environment_context>", metadata: { raw_type: "response_item" }, warnings: [] },
      { event_id: "n2", event_type: "assistant_message", text: "thinking hidden", metadata: { raw_type: "response_item", payload_type: "reasoning" }, warnings: [] },
      { event_id: "n3", event_type: "tool_call", text: "tool", metadata: { raw_type: "response_item" }, warnings: [] },
      { event_id: "n4", event_type: "user_message", text: "真实旧会话用户消息", metadata: { raw_type: "response_item" }, warnings: [] },
      { event_id: "n5", event_type: "assistant_message", text: "真实旧会话回复", metadata: { raw_type: "response_item" }, warnings: [] },
    ],
    onlyResponseItems: [
      { event_id: "r1", event_type: "user_message", text: "只有 response_item 的会话", metadata: { raw_type: "response_item" }, warnings: [] },
      { event_id: "r2", event_type: "assistant_message", text: "回复", metadata: { raw_type: "response_item" }, warnings: [] },
    ],
  };
}

export function sessionCenterHardeningFixtures(
  project: ProjectRecord,
  session: SessionRecord,
  otherProjectSession: SessionRecord,
): {
  archivedSession: SessionRecord;
  missingSession: SessionRecord;
  sessions: SessionRecord[];
  transcript: CodexTranscript;
} {
  const missingSession: SessionRecord = {
    ...session,
    thread_id: "offline-thread-missing",
    title: "Missing rollout fixture",
    rollout_exists: false,
    rollout_path: null,
    warnings: ["rollout_missing_on_disk"],
  };
  const archivedSession: SessionRecord = {
    ...session,
    thread_id: "offline-thread-archived",
    title: "Archived fixture",
    archived: true,
    rollout_path: "/offline-fixture/rollouts/offline-thread-archived.jsonl",
  };
  const longMessage = Array.from({ length: 14 }, (_, index) => `line ${index + 1}`).join("\n");
  const transcript: CodexTranscript = {
    thread_id: session.thread_id,
    rollout_path: "/offline-fixture/rollouts/transcript-hardening.jsonl",
    project_path: project.project_root,
    title: "Transcript hardening",
    created_at_ms: null,
    updated_at_ms: null,
    viewer_boundary: {
      view_kind: "session_history_viewer",
      reads_session_history: true,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "not_execution_readback",
      warnings: ["test_fixture_session_history_is_not_readback"],
    },
    events: [
      ...Array.from({ length: 13 }, (_, index): CodexTranscriptEvent => ({
        event_id: `old-${index}`,
        event_type: index % 2 === 0 ? "user_message" : "assistant_message",
        actor: index % 2 === 0 ? "user" : "assistant",
        role: index % 2 === 0 ? "user" : "assistant",
        text: `较早消息 ${index}`,
        metadata: { raw_type: "event_msg" },
        warnings: [],
      })),
      {
        event_id: "long",
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: longMessage,
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "code",
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: "```ts\nconst ok = true;\n```",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "tool",
        event_type: "tool_call",
        actor: "assistant",
        text: "should be internal",
        metadata: { raw_type: "response_item" },
        warnings: [],
      },
    ],
    summary: {
      total_events: 16,
      event_type_counts: {},
      unknown_event_count: 0,
      warning_count: 0,
      encrypted_content_event_count: 0,
      sensitive_like_event_count: 0,
    },
    warnings: [],
    source_stats: {},
  };

  return {
    archivedSession,
    missingSession,
    sessions: [session, otherProjectSession, missingSession, archivedSession],
    transcript,
  };
}
