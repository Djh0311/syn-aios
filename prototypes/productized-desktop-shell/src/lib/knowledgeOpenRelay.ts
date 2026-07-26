// Fixed UI-side contract for the host-owned `knowledge_open` relay.  This is
// intentionally not a navigation, file-system, or command bridge: the host
// sends one already-validated Markdown relative path and the UI can only
// acknowledge that exact same intent.

export const KNOWLEDGE_OPEN_RELAY_EVENT_NAME = "knowledge-open-intent" as const;

const RELAY_SCHEMA_VERSION = 1;
const MAX_RELATIVE_PATH_BYTES = 512;
const MAX_PATH_SEGMENTS = 32;
const MAX_PATH_SEGMENT_CHARS = 128;
const MAX_INTENT_ID_CHARS = 256;
const UNSAFE_PATH_SEGMENT = /[\\/:*?\[\]{}'"=|<>]/u;

export type KnowledgeOpenRelayIntent = Readonly<{
  schemaVersion: 1;
  intentId: string;
  relativePath: string;
}>;

export type KnowledgeOpenRelayOutcome = "opened" | "rejected";

export type KnowledgeOpenRelayAckRequest = Readonly<{
  intent_id: string;
  relative_path: string;
  outcome: KnowledgeOpenRelayOutcome;
}>;

export type KnowledgeOpenRelayWorkspaceCommit = Readonly<{
  typedReadCompleted: boolean;
  selectedRelativePath: string | null;
  focusedRelativePath: string | null;
}>;

export function parseKnowledgeOpenRelayIntent(payload: unknown): KnowledgeOpenRelayIntent | null {
  if (!isExactRelayPayload(payload)) return null;
  if (
    payload.schema_version !== RELAY_SCHEMA_VERSION
    || !isKnowledgeOpenRelayIntentId(payload.intent_id)
    || !isKnowledgeOpenRelayMarkdownPath(payload.relative_path)
  ) {
    return null;
  }
  return Object.freeze({
    schemaVersion: RELAY_SCHEMA_VERSION,
    intentId: payload.intent_id,
    relativePath: payload.relative_path,
  });
}

export function sameKnowledgeOpenRelayIntent(
  left: KnowledgeOpenRelayIntent | null | undefined,
  right: KnowledgeOpenRelayIntent | null | undefined,
): boolean {
  return Boolean(
    left
      && right
      && left.schemaVersion === right.schemaVersion
      && left.intentId === right.intentId
      && left.relativePath === right.relativePath,
  );
}

export function knowledgeOpenRelayCanAcknowledgeOpened(
  intent: KnowledgeOpenRelayIntent,
  commit: KnowledgeOpenRelayWorkspaceCommit,
): boolean {
  return (
    commit.typedReadCompleted
    && commit.selectedRelativePath === intent.relativePath
    && commit.focusedRelativePath === intent.relativePath
  );
}

export function knowledgeOpenRelayAckRequest(
  intent: KnowledgeOpenRelayIntent,
  outcome: KnowledgeOpenRelayOutcome,
): KnowledgeOpenRelayAckRequest {
  if (
    intent.schemaVersion !== RELAY_SCHEMA_VERSION
    || !isKnowledgeOpenRelayIntentId(intent.intentId)
    || !isKnowledgeOpenRelayMarkdownPath(intent.relativePath)
    || (outcome !== "opened" && outcome !== "rejected")
  ) {
    throw new Error("knowledge_open_relay_ack_rejected");
  }
  return Object.freeze({
    intent_id: intent.intentId,
    relative_path: intent.relativePath,
    outcome,
  });
}

function isExactRelayPayload(value: unknown): value is Record<string, unknown> & {
  schema_version: unknown;
  intent_id: unknown;
  relative_path: unknown;
} {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;
  const keys = Object.keys(value).sort();
  return keys.length === 3
    && keys[0] === "intent_id"
    && keys[1] === "relative_path"
    && keys[2] === "schema_version";
}

function isKnowledgeOpenRelayIntentId(value: unknown): value is string {
  return typeof value === "string"
    && value.startsWith("intent:")
    && value.length <= MAX_INTENT_ID_CHARS
    && value.length > "intent:".length
    && /^[A-Za-z0-9:_-]+$/u.test(value);
}

function isKnowledgeOpenRelayMarkdownPath(value: unknown): value is string {
  if (
    typeof value !== "string"
    || !value
    || value.trim() !== value
    || value.includes("\\")
    || value.startsWith("/")
    || new TextEncoder().encode(value).byteLength > MAX_RELATIVE_PATH_BYTES
  ) {
    return false;
  }
  const segments = value.split("/");
  if (segments.length > MAX_PATH_SEGMENTS || !segments.length) return false;
  if (!segments.every(isSafeRelativePathSegment)) return false;
  const fileName = segments.at(-1) ?? "";
  return fileName.endsWith(".md") && fileName.length > ".md".length;
}

function isSafeRelativePathSegment(segment: string): boolean {
  return (
    Boolean(segment)
    && segment !== "."
    && segment !== ".."
    && !segment.startsWith(".")
    && !segment.startsWith("-")
    && !segment.includes("--")
    && Array.from(segment).length <= MAX_PATH_SEGMENT_CHARS
    && !Array.from(segment).some((character) => /\p{Cc}/u.test(character))
    && !UNSAFE_PATH_SEGMENT.test(segment)
  );
}
