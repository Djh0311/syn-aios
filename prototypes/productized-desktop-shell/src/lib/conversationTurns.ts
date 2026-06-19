import type { CodexTranscriptEvent } from "./types";

const CONVERSATION_EVENT_TYPES = new Set(["user_message", "assistant_message"]);

const SYSTEM_INJECTION_PREFIXES = [
  "<environment_context",
  "<permissions instructions",
  "<developer",
  "<system",
  "# tools",
  "knowledge cutoff:",
  "current date:",
];

const SYSTEM_INJECTION_MARKERS = [
  "</environment_context>",
  "</permissions instructions>",
  "sandbox_mode",
  "filesystem sandboxing",
  "cwd=",
  "codex exec resume",
];

function metadataOf(event: CodexTranscriptEvent): Record<string, unknown> | null {
  const metadata = event.metadata;
  return metadata && typeof metadata === "object" && !Array.isArray(metadata) ? metadata : null;
}

export function rawTypeOf(event: CodexTranscriptEvent): string | null {
  const raw = metadataOf(event)?.raw_type;
  return typeof raw === "string" ? raw : null;
}

function payloadTypeOf(event: CodexTranscriptEvent): string | null {
  const payloadType = metadataOf(event)?.payload_type;
  return typeof payloadType === "string" ? payloadType : null;
}

function isConversationEvent(event: CodexTranscriptEvent): boolean {
  return CONVERSATION_EVENT_TYPES.has(event.event_type ?? "");
}

function isSystemInjectedText(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed) return true;
  const lower = trimmed.toLowerCase();
  if (SYSTEM_INJECTION_PREFIXES.some((prefix) => lower.startsWith(prefix))) return true;
  return SYSTEM_INJECTION_MARKERS.some((marker) => lower.includes(marker));
}

function isCleanConversationEvent(event: CodexTranscriptEvent, allowResponseItem: boolean): boolean {
  if (!isConversationEvent(event)) return false;
  if ((event.text ?? "").trim().length === 0) return false;
  if (event.role === "system") return false;
  if (!allowResponseItem && rawTypeOf(event) !== "event_msg") return false;
  if (payloadTypeOf(event) === "reasoning") return false;
  return !isSystemInjectedText(event.text ?? "");
}

function isPendingConversationEvent(event: CodexTranscriptEvent): boolean {
  return metadataOf(event)?.conversation_engine_pending === true && isCleanConversationEvent(event, true);
}

export function conversationTurns(events: CodexTranscriptEvent[]): CodexTranscriptEvent[] {
  const pending = events.filter(isPendingConversationEvent);
  const fromEventMsg = events.filter((event) => rawTypeOf(event) === "event_msg" && isCleanConversationEvent(event, false));
  if (fromEventMsg.length === 0) return events.filter((event) => isCleanConversationEvent(event, true));

  const hasUser = fromEventMsg.some((event) => event.event_type === "user_message");
  const hasAssistant = fromEventMsg.some((event) => event.event_type === "assistant_message");
  if (hasUser && hasAssistant) {
    const selectedIds = new Set([...fromEventMsg, ...pending].map((event) => event.event_id));
    return events.filter((event) => selectedIds.has(event.event_id));
  }

  const fallback = events.filter((event) => {
    if (rawTypeOf(event) !== "response_item") return false;
    if (!isCleanConversationEvent(event, true)) return false;
    if (!hasUser && event.event_type === "user_message") return true;
    if (!hasAssistant && event.event_type === "assistant_message") return true;
    return false;
  });
  const selectedIds = new Set([...fromEventMsg, ...fallback, ...pending].map((event) => event.event_id));
  return events.filter((event) => selectedIds.has(event.event_id));
}
