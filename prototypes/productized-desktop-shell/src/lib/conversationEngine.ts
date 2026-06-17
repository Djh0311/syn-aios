import type { CodexTranscript, CodexTranscriptEvent } from "./types";

export type PendingUserMessageInput = {
  prompt: string;
  threadId: string;
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

export function appendPendingUserMessage(
  transcript: CodexTranscript,
  pendingMessage: CodexTranscriptEvent,
): CodexTranscript {
  return {
    ...transcript,
    events: [...transcript.events, pendingMessage],
    summary: {
      ...transcript.summary,
      total_events: transcript.summary.total_events + 1,
      event_type_counts: {
        ...transcript.summary.event_type_counts,
        user_message: (transcript.summary.event_type_counts.user_message ?? 0) + 1,
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
