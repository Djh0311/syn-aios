// M4R05 renderer boundary for the persistent Secretary conversation.
//
// M3 owns RoleSession/Turn lifecycle and the provider transcript port owns raw
// message content.  The renderer receives one complete display snapshot and
// replaces it after every load/send; it does not reconstruct either truth.

export const M4_SECRETARY_CONVERSATION_SCHEMA = "syn.m4.secretary.conversation.v1" as const;
export const M4_SECRETARY_CONVERSATION_SEND_SCHEMA = "syn.m4.secretary.conversation-send.v1" as const;

export type M4SecretaryConversationTurnState =
  | "ACCEPTED"
  | "STARTING"
  | "ACTIVE"
  | "SUCCEEDED"
  | "FAILED"
  | "CANCELLED"
  | "TIMED_OUT";

export type M4SecretaryConversationMessage = Readonly<{
  message_ref: string;
  text: string;
  created_at_utc: string;
}>;

export type M4SecretaryConversationTurn = Readonly<{
  turn_ref: string;
  client_message_ref: string;
  state: M4SecretaryConversationTurnState;
  user_message: M4SecretaryConversationMessage;
  assistant_message: M4SecretaryConversationMessage | null;
  error_code: string | null;
  started_at_utc: string;
  terminal_at_utc: string | null;
}>;

export type M4SecretaryConversation = Readonly<{
  schema_version: typeof M4_SECRETARY_CONVERSATION_SCHEMA;
  role_session_ref: string;
  role_ref: string;
  scope_ref: string;
  channel_key: string;
  history_ref: string;
  turns: readonly M4SecretaryConversationTurn[];
}>;

export type M4SecretaryMessageSendRequest = Readonly<{
  message: string;
  client_message_ref: string;
}>;

export type M4SecretaryMessageSendOutcome = Readonly<{
  schema_version: typeof M4_SECRETARY_CONVERSATION_SEND_SCHEMA;
  command_receipt_ref: string;
  turn_ref: string;
  replayed: boolean;
  conversation: M4SecretaryConversation;
}>;

const TURN_STATES = new Set<M4SecretaryConversationTurnState>([
  "ACCEPTED",
  "STARTING",
  "ACTIVE",
  "SUCCEEDED",
  "FAILED",
  "CANCELLED",
  "TIMED_OUT",
]);

export function createSecretaryMessageSendRequest(value: M4SecretaryMessageSendRequest): M4SecretaryMessageSendRequest {
  const raw = exactRecord(value, ["message", "client_message_ref"], "send_request");
  const message = boundedString(raw.message, "message", 1, 64_000).trim();
  const clientMessageRef = boundedString(raw.client_message_ref, "client_message_ref", 1, 512);
  const messageBytes = secretaryMessageUtf8ByteLength(message);
  if (messageBytes === 0) throw new Error("m4_secretary_conversation_message_blank");
  if (messageBytes > 16_000) throw new Error("m4_secretary_conversation_message_too_large");
  assertClientMessageRef(clientMessageRef);
  return { message, client_message_ref: clientMessageRef };
}

export function parseSecretaryConversation(value: unknown): M4SecretaryConversation {
  const raw = exactRecord(value, [
    "schema_version",
    "role_session_ref",
    "role_ref",
    "scope_ref",
    "channel_key",
    "history_ref",
    "turns",
  ], "conversation");
  if (raw.schema_version !== M4_SECRETARY_CONVERSATION_SCHEMA) {
    throw new Error("m4_secretary_conversation_schema_unknown");
  }
  if (!Array.isArray(raw.turns)) throw new Error("m4_secretary_conversation_turns_invalid");
  const turns = raw.turns.map((turn, index) => parseTurn(turn, index));
  const turnRefs = new Set(turns.map((turn) => turn.turn_ref));
  const clientRefs = new Set(turns.map((turn) => turn.client_message_ref));
  if (turnRefs.size !== turns.length || clientRefs.size !== turns.length) {
    throw new Error("m4_secretary_conversation_duplicate_turn_identity");
  }
  return {
    schema_version: M4_SECRETARY_CONVERSATION_SCHEMA,
    role_session_ref: boundedString(raw.role_session_ref, "role_session_ref", 1, 512),
    role_ref: boundedString(raw.role_ref, "role_ref", 1, 256),
    scope_ref: boundedString(raw.scope_ref, "scope_ref", 1, 512),
    channel_key: boundedString(raw.channel_key, "channel_key", 1, 128),
    history_ref: boundedString(raw.history_ref, "history_ref", 1, 512),
    turns,
  };
}

export function parseSecretaryMessageSendOutcome(value: unknown): M4SecretaryMessageSendOutcome {
  const raw = exactRecord(value, [
    "schema_version",
    "command_receipt_ref",
    "turn_ref",
    "replayed",
    "conversation",
  ], "send_outcome");
  if (raw.schema_version !== M4_SECRETARY_CONVERSATION_SEND_SCHEMA) {
    throw new Error("m4_secretary_conversation_send_schema_unknown");
  }
  if (typeof raw.replayed !== "boolean") throw new Error("m4_secretary_conversation_replayed_invalid");
  const turnRef = boundedString(raw.turn_ref, "turn_ref", 1, 512);
  const conversation = parseSecretaryConversation(raw.conversation);
  if (!conversation.turns.some((turn) => turn.turn_ref === turnRef)) {
    throw new Error("m4_secretary_conversation_send_turn_missing");
  }
  return {
    schema_version: M4_SECRETARY_CONVERSATION_SEND_SCHEMA,
    command_receipt_ref: boundedString(raw.command_receipt_ref, "command_receipt_ref", 1, 512),
    turn_ref: turnRef,
    replayed: raw.replayed,
    conversation,
  };
}

export function mintSecretaryClientMessageRef(): string {
  return `secretary-client-message:${crypto.randomUUID().replaceAll("-", "")}`;
}

export function secretaryMessageUtf8ByteLength(value: string): number {
  return new TextEncoder().encode(value.trim()).byteLength;
}

export function isCurrentSecretaryConversationEpoch(currentEpoch: number, responseEpoch: number): boolean {
  return currentEpoch === responseEpoch;
}

export function shouldStartSecretaryConversationReload(sendPending: boolean): boolean {
  return !sendPending;
}

function parseTurn(value: unknown, index: number): M4SecretaryConversationTurn {
  const raw = exactRecord(value, [
    "turn_ref",
    "client_message_ref",
    "state",
    "user_message",
    "assistant_message",
    "error_code",
    "started_at_utc",
    "terminal_at_utc",
  ], `turn_${index}`);
  if (typeof raw.state !== "string" || !TURN_STATES.has(raw.state as M4SecretaryConversationTurnState)) {
    throw new Error("m4_secretary_conversation_turn_state_unknown");
  }
  const state = raw.state as M4SecretaryConversationTurnState;
  const assistantMessage = raw.assistant_message === null
    ? null
    : parseMessage(raw.assistant_message, `turn_${index}_assistant_message`);
  const errorCode = nullableBoundedString(raw.error_code, "error_code", 1, 256);
  const terminalAtUtc = nullableBoundedString(raw.terminal_at_utc, "terminal_at_utc", 1, 128);
  if (state === "SUCCEEDED"
    && (assistantMessage === null || errorCode !== null || terminalAtUtc === null)) {
    throw new Error("m4_secretary_conversation_succeeded_turn_invalid");
  }
  if (state === "FAILED"
    && (assistantMessage !== null || errorCode === null || terminalAtUtc === null)) {
    throw new Error("m4_secretary_conversation_failed_turn_invalid");
  }
  if ((state === "ACCEPTED" || state === "STARTING" || state === "ACTIVE")
    && (assistantMessage !== null || errorCode !== null || terminalAtUtc !== null)) {
    throw new Error("m4_secretary_conversation_nonterminal_turn_invalid");
  }
  if ((state === "CANCELLED" || state === "TIMED_OUT")
    && (assistantMessage !== null || terminalAtUtc === null)) {
    throw new Error("m4_secretary_conversation_terminal_turn_invalid");
  }
  const clientMessageRef = boundedString(raw.client_message_ref, "client_message_ref", 1, 512);
  assertClientMessageRef(clientMessageRef);
  return {
    turn_ref: boundedString(raw.turn_ref, "turn_ref", 1, 512),
    client_message_ref: clientMessageRef,
    state,
    user_message: parseMessage(raw.user_message, `turn_${index}_user_message`),
    assistant_message: assistantMessage,
    error_code: errorCode,
    started_at_utc: boundedString(raw.started_at_utc, "started_at_utc", 1, 128),
    terminal_at_utc: terminalAtUtc,
  };
}

function parseMessage(value: unknown, field: string): M4SecretaryConversationMessage {
  const raw = exactRecord(value, ["message_ref", "text", "created_at_utc"], field);
  return {
    message_ref: boundedString(raw.message_ref, "message_ref", 1, 512),
    text: boundedString(raw.text, "text", 1, 64_000),
    created_at_utc: boundedString(raw.created_at_utc, "created_at_utc", 1, 128),
  };
}

function exactRecord(value: unknown, allowedKeys: readonly string[], field: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`m4_secretary_conversation_${field}_invalid`);
  }
  const raw = value as Record<string, unknown>;
  for (const key of Object.keys(raw)) {
    if (!allowedKeys.includes(key)) throw new Error(`m4_secretary_conversation_unknown_${field}_field:${key}`);
  }
  for (const key of allowedKeys) {
    if (!(key in raw)) throw new Error(`m4_secretary_conversation_missing_${field}_field:${key}`);
  }
  return raw;
}

function boundedString(value: unknown, field: string, minLength: number, maxLength: number): string {
  if (typeof value !== "string" || value.length < minLength || value.length > maxLength) {
    throw new Error(`m4_secretary_conversation_${field}_invalid`);
  }
  return value;
}

function nullableBoundedString(value: unknown, field: string, minLength: number, maxLength: number): string | null {
  return value === null ? null : boundedString(value, field, minLength, maxLength);
}

function assertClientMessageRef(value: string): void {
  if (!/^secretary-client-message:[a-f0-9]{32}$/.test(value)) {
    throw new Error("m4_secretary_conversation_client_message_ref_invalid");
  }
}
