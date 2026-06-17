import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { assert, visibleText } from "./offlineInteractionTestUtils";
import {
  appendPendingUserMessage,
  buildPendingUserMessage,
  mergeOlderTranscriptPage,
} from "../../src/lib/conversationEngine";
import type { CodexTranscript, PendingAction, SessionRecord } from "../../src/lib/types";
import { AgentSessionCenter, ChatTranscript } from "../../src/views/AgentView";

export function runConversationEngineScenario({
  captureAction,
  session,
}: {
  captureAction: (action: PendingAction) => void;
  session: SessionRecord;
}) {
  const transcript = buildLargeTranscript(session.thread_id, session.rollout_path ?? "fixture-rollout.jsonl", 180);
  const transcriptMarkup = renderToStaticMarkup(<ChatTranscript transcript={transcript} />);
  const transcriptText = visibleText(<ChatTranscript transcript={transcript} />);

  assert(transcriptMarkup.includes("data-conversation-engine=\"virtualized\""), "M1 对话流应声明使用虚拟化引擎");
  assert(transcriptText.includes("虚拟消息窗口"), "M1 对话流应暴露虚拟窗口计数");
  assert(transcriptText.includes("已渲染"), "M1 对话流应显示当前渲染数量");
  assert(transcriptText.includes("Message fixture 179"), "M1 初始窗口应显示最新消息");
  assert(!transcriptMarkup.includes("Message fixture 20"), "M1 大对话不应默认把早期消息全量放进 DOM");

  const centerText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={session.thread_id}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(centerText.includes("Message fixture 179"), "M1 选中会话加载中也应保留已读历史，不出现空窗");
  assert(centerText.includes("正在刷新这条对话"), "M1 背景刷新应是状态提示，不应清空历史");
  const firstLoadText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={session.thread_id}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(firstLoadText.includes("正在读取这条对话"), "M1 首次加载新会话应显示读取态，不应静默空窗");
  assert(firstLoadText.includes("这不是 0 条结果"), "M1 首次加载态不得暗示读回 0 条");

  const streamingTranscript = {
    ...transcript,
    events: [
      ...transcript.events,
      {
        event_id: "streaming-assistant-draft",
        timestamp: "2026-06-17T00:59:59Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "Streaming fixture draft",
        metadata: { conversation_engine_streaming: true },
        warnings: [],
      },
    ],
  };
  const streamingMarkup = renderToStaticMarkup(<ChatTranscript transcript={streamingTranscript} />);
  const streamingText = visibleText(<ChatTranscript transcript={streamingTranscript} />);
  assert(streamingMarkup.includes("data-stick-to-bottom=\"true\""), "M2 对话流应声明默认黏底");
  assert(streamingMarkup.includes("data-streaming-separated=\"true\""), "M2 流式追加应从稳定虚拟窗口分离");
  assert(streamingText.includes("回到底部"), "M2 滚离底部时应有回到底部入口");
  assert(streamingText.includes("Streaming fixture draft"), "M2 流式草稿应作为单条自然流显示");
  assert(!streamingMarkup.includes("streaming-assistant-draft\" style"), "M2 流式草稿不应进入稳定虚拟窗口绝对定位层");

  const boundedTailTranscript = {
    ...transcript,
    events: transcript.events.slice(-12),
    pagination: {
      mode: "tail",
      page_size: 12,
      returned_events: 12,
      total_line_count: 180,
      selected_line_count: 12,
      has_older: true,
      older_before_line: 169,
    },
  };
  const boundedMarkup = renderToStaticMarkup(
    <ChatTranscript
      olderLoading={false}
      transcript={boundedTailTranscript}
      onLoadOlder={() => {
        throw new Error("offline render should not invoke older loader");
      }}
    />,
  );
  assert(boundedMarkup.includes("data-transcript-load=\"bounded\""), "M2 点开对话应声明 transcript 加载已界定");
  assert(boundedMarkup.includes("加载更早对话"), "M2 有 older cursor 时应显示上滚加载更早入口");

  const internalOnlyTailTranscript: CodexTranscript = {
    ...boundedTailTranscript,
    events: boundedTailTranscript.events.map((event, index) => ({
      ...event,
      event_id: `internal-only-tail-${index}`,
      event_type: "tool_call",
      actor: "tool",
      text: `Tool fixture ${index}`,
    })),
  };
  const internalOnlyMarkup = renderToStaticMarkup(
    <ChatTranscript
      olderLoading={false}
      transcript={internalOnlyTailTranscript}
      onLoadOlder={() => {
        throw new Error("offline render should not invoke older loader");
      }}
    />,
  );
  assert(internalOnlyMarkup.includes("这条会话没有可显示的对话"), "M2 内部事件 tail 应显示空态说明");
  assert(internalOnlyMarkup.includes("加载更早对话"), "M2 内部事件 tail 仍应保留加载更早入口");

  const olderPage = {
    ...transcript,
    events: transcript.events.slice(-24, -12),
    pagination: {
      mode: "older",
      page_size: 12,
      returned_events: 12,
      total_line_count: 180,
      selected_line_count: 12,
      has_older: true,
      older_before_line: 157,
    },
  };
  const mergedTranscript = mergeOlderTranscriptPage(boundedTailTranscript, olderPage);
  assert(mergedTranscript.events[0].text === "Message fixture 156", "M2 更早页应前插到当前尾页之前");
  assert(mergedTranscript.events.at(-1)?.text === "Message fixture 179", "M2 前插更早页后必须保留最新尾部消息");
  assert(mergedTranscript.pagination?.older_before_line === 157, "M2 前插后应延续 older cursor");

  const composerMarkup = renderToStaticMarkup(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      showSoftwareLayer={false}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(composerMarkup.includes("data-send-mode=\"decision-only\""), "M3 撰写区应声明发送只记录决策、不真执行");
  assert(composerMarkup.includes("发送"), "M3 撰写区主按钮应是发送");
  assert(!composerMarkup.includes("生成发送预览"), "M3 普通撰写区不应保留 6 步预览入口");
  assert(!composerMarkup.includes("确认执行 Codex"), "M3 普通撰写区不应出现真实执行按钮");

  const pendingMessage = buildPendingUserMessage({
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  const optimisticTranscript = appendPendingUserMessage(transcript, pendingMessage);
  const optimisticText = visibleText(<ChatTranscript transcript={optimisticTranscript} />);
  assert(optimisticText.includes("M3 optimistic send fixture"), "M3 发送后应立即冒泡用户消息");
  assert(pendingMessage.metadata?.conversation_engine_send_mode === "decision_only", "M3 pending 消息必须标记为 decision-only");
  assert(pendingMessage.metadata?.real_codex_executed === false, "M3 pending 消息不得声明真实 Codex 执行");
  const repeatedPendingMessage = buildPendingUserMessage({
    createdAt: "2026-06-17T01:00:00Z",
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  const nextRepeatedPendingMessage = buildPendingUserMessage({
    createdAt: "2026-06-17T01:00:01Z",
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  assert(
    repeatedPendingMessage.event_id !== nextRepeatedPendingMessage.event_id,
    "M3 相同 prompt 连续发送也不应生成重复 pending event_id",
  );
}

function buildLargeTranscript(threadId: string, rolloutPath: string, count: number): CodexTranscript {
  return {
    thread_id: threadId,
    rollout_path: rolloutPath,
    project_path: "/tmp/offline",
    title: "Large conversation fixture",
    created_at_ms: 1,
    updated_at_ms: count,
    viewer_boundary: {
      view_kind: "session_history_viewer",
      reads_session_history: true,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "not_execution_readback",
      warnings: [],
    },
    events: Array.from({ length: count }, (_, index) => ({
      event_id: `message-fixture-${index}`,
      timestamp: `2026-06-17T00:${String(index % 60).padStart(2, "0")}:00Z`,
      event_type: index % 2 === 0 ? "user_message" : "assistant_message",
      actor: index % 2 === 0 ? "user" : "assistant",
      text: `Message fixture ${index}`,
      warnings: [],
    })),
    summary: {
      total_events: count,
      event_type_counts: {
        user_message: Math.ceil(count / 2),
        assistant_message: Math.floor(count / 2),
      },
      unknown_event_count: 0,
      warning_count: 0,
      encrypted_content_event_count: 0,
      sensitive_like_event_count: 0,
    },
    warnings: [],
    source_stats: {},
  };
}
