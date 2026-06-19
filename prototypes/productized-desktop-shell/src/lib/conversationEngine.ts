import type { CodexTranscript, CodexTranscriptEvent, ManualRelayLiveEvent } from "./types";

export type PendingUserMessageInput = {
  prompt: string;
  threadId: string;
  createdAt?: string;
};

export type ManualRelayPendingUserMessageInput = {
  prompt: string;
  threadId: string;
  relayAttemptId: string;
  confirmationId: string;
  targetProjectRoot: string;
  targetSessionId: string | null;
  promptSha256: string;
  createdAt?: string;
};

export type ManualRelayOptimisticUserMessageInput = {
  prompt: string;
  threadId: string;
  targetProjectRoot: string;
  targetSessionId: string | null;
  createdAt?: string;
};

export type ManualRelayAssistantMessageInput = {
  text: string;
  threadId: string;
  relayAttemptId: string;
  assistantItemId?: string | null;
  promptSha256: string;
  usage?: Record<string, number> | null;
  createdAt?: string;
};

export type ManualRelayLiveTranscriptEventsInput = {
  liveEvents: ManualRelayLiveEvent[];
  threadId: string;
  relayAttemptId: string;
  includeAssistant?: boolean;
  createdAt?: string;
};

export function buildPendingUserMessage({
  prompt,
  threadId,
  createdAt = new Date().toISOString(),
}: PendingUserMessageInput): CodexTranscriptEvent {
  return {
    event_id: `conversation-engine-pending-user:${threadId}:${hashDraft(`${createdAt}:${prompt}`)}`,
    timestamp: createdAt,
    event_type: "user_message",
    actor: "user",
    text: prompt.trim(),
    metadata: {
      conversation_engine_pending: true,
      conversation_engine_send_mode: "decision_only",
      real_codex_executed: false,
    },
    warnings: ["pending_decision_only_no_codex_execution"],
  };
}

export function buildManualRelayPendingUserMessage({
  prompt,
  threadId,
  relayAttemptId,
  confirmationId,
  targetProjectRoot,
  targetSessionId,
  promptSha256,
  createdAt = new Date().toISOString(),
}: ManualRelayPendingUserMessageInput): CodexTranscriptEvent {
  return {
    event_id: `manual-relay-pending-user:${threadId}:${hashDraft(`${createdAt}:${relayAttemptId}:${promptSha256}`)}`,
    timestamp: createdAt,
    event_type: "user_message",
    actor: "user",
    text: prompt.trim(),
    metadata: {
      conversation_engine_pending: true,
      conversation_engine_send_mode: "manual_relay_confirmed_once",
      relay_attempt_id: relayAttemptId,
      relay_confirmation_id: confirmationId,
      target_project_root: targetProjectRoot,
      target_session_id: targetSessionId,
      prompt_sha256: promptSha256,
      prompt_exact_original: true,
      payload_layers_empty: true,
      manual_once: true,
      auto_chain: false,
      real_codex_executed: false,
    },
    warnings: [
      "manual_relay_fixture_only_no_real_codex_execution",
      "manual_relay_prompt_body_visible_only_in_conversation_surface",
    ],
  };
}

export function buildManualRelayOptimisticUserMessage({
  prompt,
  threadId,
  targetProjectRoot,
  targetSessionId,
  createdAt = new Date().toISOString(),
}: ManualRelayOptimisticUserMessageInput): CodexTranscriptEvent {
  return {
    event_id: `manual-relay-optimistic-user:${threadId}:${hashDraft(`${createdAt}:${targetProjectRoot}:${prompt}`)}`,
    timestamp: createdAt,
    event_type: "user_message",
    actor: "user",
    role: "user",
    text: prompt.trim(),
    metadata: {
      conversation_engine_pending: true,
      conversation_engine_send_mode: "manual_relay_direct_pending",
      target_project_root: targetProjectRoot,
      target_session_id: targetSessionId,
      prompt_exact_original: true,
      payload_layers_empty: true,
      manual_once: true,
      auto_chain: false,
      real_codex_executed: false,
    },
    warnings: ["manual_relay_direct_pending_thread_event_reply"],
  };
}

export function buildManualRelayAssistantMessage({
  text,
  threadId,
  relayAttemptId,
  assistantItemId,
  promptSha256,
  usage,
  createdAt = new Date().toISOString(),
}: ManualRelayAssistantMessageInput): CodexTranscriptEvent {
  return {
    event_id: `manual-relay-assistant:${threadId}:${hashDraft(`${createdAt}:${relayAttemptId}:${assistantItemId ?? ""}:${promptSha256}`)}`,
    timestamp: createdAt,
    event_type: "assistant_message",
    actor: "assistant",
    role: "assistant",
    text,
    metadata: {
      conversation_engine_pending: true,
      conversation_engine_send_mode: "manual_relay_thread_event_reply",
      relay_attempt_id: relayAttemptId,
      assistant_item_id: assistantItemId ?? null,
      prompt_sha256: promptSha256,
      usage: usage ?? {},
      real_codex_executed: true,
    },
    warnings: ["manual_relay_reply_from_thread_event"],
  };
}

export function buildManualRelayLiveTranscriptEvents({
  liveEvents,
  threadId,
  relayAttemptId,
  includeAssistant = true,
  createdAt = new Date().toISOString(),
}: ManualRelayLiveTranscriptEventsInput): CodexTranscriptEvent[] {
  const transcriptEvents: CodexTranscriptEvent[] = [];
  const assistantTextByItem = new Map<string, string>();
  let latestAssistantItemId: string | null = null;

  for (const liveEvent of liveEvents) {
    if (liveEvent.item_type === "agent_message") {
      const itemId = liveEvent.item_id ?? "agent";
      latestAssistantItemId = itemId;
      const current = assistantTextByItem.get(itemId) ?? "";
      if (liveEvent.text) {
        assistantTextByItem.set(itemId, liveEvent.text);
      } else if (liveEvent.delta) {
        assistantTextByItem.set(itemId, `${current}${liveEvent.delta}`);
      }
      continue;
    }
    transcriptEvents.push(buildManualRelayLiveStatusEvent(liveEvent, threadId, relayAttemptId, createdAt));
  }

  if (includeAssistant && latestAssistantItemId) {
    const assistantText = assistantTextByItem.get(latestAssistantItemId)?.trim();
    if (assistantText) {
      transcriptEvents.push({
        event_id: `manual-relay-live-assistant:${threadId}:${relayAttemptId}:${latestAssistantItemId}`,
        timestamp: createdAt,
        event_type: "assistant_message",
        actor: "assistant",
        role: "assistant",
        text: assistantText,
        metadata: {
          conversation_engine_pending: true,
          conversation_engine_streaming: true,
          conversation_engine_send_mode: "manual_relay_live_thread_event",
          relay_attempt_id: relayAttemptId,
          assistant_item_id: latestAssistantItemId,
          real_codex_executed: true,
        },
        warnings: ["manual_relay_live_thread_event"],
      });
    }
  }

  return transcriptEvents;
}

function buildManualRelayLiveStatusEvent(
  liveEvent: ManualRelayLiveEvent,
  threadId: string,
  relayAttemptId: string,
  createdAt: string,
): CodexTranscriptEvent {
  const eventType = liveTranscriptEventType(liveEvent);
  const text = liveEvent.text ?? liveEvent.delta ?? liveEvent.output_preview ?? liveEvent.arguments_preview ?? "";
  return {
    event_id: `manual-relay-live:${threadId}:${relayAttemptId}:${liveEvent.sequence}`,
    timestamp: createdAt,
    event_type: eventType,
    actor: eventType === "tool_result" || eventType === "command_output" ? "tool" : "assistant",
    tool_name: liveEvent.tool_name ?? liveEvent.item_type ?? liveEvent.event_type,
    text,
    arguments: liveEvent.arguments_preview ?? null,
    output: liveEvent.output_preview ?? null,
    stdout: liveEvent.stdout,
    stderr: liveEvent.stderr,
    exit_code: liveEvent.exit_code,
    metadata: {
      conversation_engine_pending: true,
      conversation_engine_live_status: true,
      conversation_engine_send_mode: "manual_relay_live_thread_event",
      relay_attempt_id: relayAttemptId,
      live_sequence: liveEvent.sequence,
      live_event_type: liveEvent.event_type,
      live_status: liveEvent.status,
      live_title: liveEvent.title,
      live_item_id: liveEvent.item_id,
      live_item_type: liveEvent.item_type,
      real_codex_executed: true,
      ...(liveEvent.item_type === "reasoning" ? { payload_type: "reasoning" } : {}),
    },
    warnings: ["manual_relay_live_thread_event"],
  };
}

function liveTranscriptEventType(liveEvent: ManualRelayLiveEvent): string {
  if (liveEvent.item_type === "reasoning") return "system_context";
  if (liveEvent.item_type === "local_shell_call" || liveEvent.item_type === "function_call") {
    return liveEvent.status === "completed" && (liveEvent.stdout || liveEvent.stderr || liveEvent.exit_code !== null)
      ? "command_output"
      : "tool_call";
  }
  if (liveEvent.item_type === "function_call_output") return "tool_result";
  return "system_context";
}

export function appendPendingUserMessage(
  transcript: CodexTranscript,
  pendingMessage: CodexTranscriptEvent,
): CodexTranscript {
  const eventType = pendingMessage.event_type ?? "unknown";
  return {
    ...transcript,
    events: [...transcript.events, pendingMessage],
    summary: {
      ...transcript.summary,
      total_events: transcript.summary.total_events + 1,
      event_type_counts: {
        ...transcript.summary.event_type_counts,
        [eventType]: (transcript.summary.event_type_counts[eventType] ?? 0) + 1,
      },
      warning_count: transcript.summary.warning_count + pendingMessage.warnings.length,
    },
  };
}

export function mergeOlderTranscriptPage(current: CodexTranscript, olderPage: CodexTranscript): CodexTranscript {
  if (current.thread_id !== olderPage.thread_id) return current;
  const currentIds = new Set(current.events.map((event) => event.event_id));
  const olderEvents = olderPage.events.filter((event) => !currentIds.has(event.event_id));
  const events = [...olderEvents, ...current.events];
  return {
    ...current,
    events,
    pagination: {
      mode: "bounded_merged",
      page_size: current.pagination?.page_size ?? olderPage.pagination?.page_size ?? events.length,
      returned_events: events.length,
      total_line_count: Math.max(current.pagination?.total_line_count ?? 0, olderPage.pagination?.total_line_count ?? 0),
      selected_line_count: events.length,
      has_older: olderPage.pagination?.has_older ?? false,
      older_before_line: olderPage.pagination?.older_before_line ?? null,
    },
    summary: summarizeTranscriptEvents(events),
    warnings: uniqueStrings([...olderPage.warnings, ...current.warnings]),
  };
}

function summarizeTranscriptEvents(events: CodexTranscriptEvent[]): CodexTranscript["summary"] {
  const event_type_counts: Record<string, number> = {};
  let encrypted_content_event_count = 0;
  let sensitive_like_event_count = 0;
  let warning_count = 0;
  let unknown_event_count = 0;
  for (const event of events) {
    const eventType = event.event_type ?? "unknown";
    event_type_counts[eventType] = (event_type_counts[eventType] ?? 0) + 1;
    if (eventType === "unknown") unknown_event_count += 1;
    warning_count += event.warnings.length;
    if (event.warnings.includes("encrypted_content_omitted")) encrypted_content_event_count += 1;
    if (event.warnings.includes("sensitive_like_content")) sensitive_like_event_count += 1;
  }
  return {
    total_events: events.length,
    event_type_counts,
    unknown_event_count,
    warning_count,
    encrypted_content_event_count,
    sensitive_like_event_count,
  };
}

function uniqueStrings(values: string[]): string[] {
  return Array.from(new Set(values));
}

function hashDraft(value: string): string {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}
