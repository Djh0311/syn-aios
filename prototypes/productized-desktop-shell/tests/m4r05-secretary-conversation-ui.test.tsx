import { renderToStaticMarkup } from "react-dom/server.browser";
import { SecretaryConversationHistory } from "../src/components/SecretaryBoardView";
import { WorkbenchDock, canSendSecretaryDraft } from "../src/components/WorkbenchShell";
import {
  M4_SECRETARY_CONVERSATION_SCHEMA,
  M4_SECRETARY_CONVERSATION_SEND_SCHEMA,
  createSecretaryMessageSendRequest,
  isCurrentSecretaryConversationEpoch,
  parseSecretaryConversation,
  parseSecretaryMessageSendOutcome,
  shouldStartSecretaryConversationReload,
} from "../src/lib/types/m4SecretaryConversation";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

const clientRef = (character: string) => `secretary-client-message:${character.repeat(32)}`;

const conversation = parseSecretaryConversation({
  schema_version: M4_SECRETARY_CONVERSATION_SCHEMA,
  role_session_ref: "role-session:secretary:fixture",
  role_ref: "Secretary",
  scope_ref: "scope:personal:primary",
  channel_key: "daily",
  history_ref: "provider-history:fixture",
  turns: [
    {
      turn_ref: "turn:first",
      client_message_ref: clientRef("a"),
      state: "SUCCEEDED",
      user_message: {
        message_ref: "message:first:user",
        text: "第一句",
        created_at_utc: "2026-08-11T01:00:00Z",
      },
      assistant_message: {
        message_ref: "message:first:assistant",
        text: "第一句回复",
        created_at_utc: "2026-08-11T01:00:01Z",
      },
      error_code: null,
      started_at_utc: "2026-08-11T01:00:00Z",
      terminal_at_utc: "2026-08-11T01:00:01Z",
    },
    {
      turn_ref: "turn:second",
      client_message_ref: clientRef("b"),
      state: "FAILED",
      user_message: {
        message_ref: "message:second:user",
        text: "第二句",
        created_at_utc: "2026-08-11T01:01:00Z",
      },
      assistant_message: null,
      error_code: "M4_SECRETARY_PROVIDER_FAILURE",
      started_at_utc: "2026-08-11T01:01:00Z",
      terminal_at_utc: "2026-08-11T01:01:01Z",
    },
  ],
});

function expectThrows(action: () => unknown, fragment: string, label: string) {
  let message = "";
  try {
    action();
  } catch (error) {
    message = error instanceof Error ? error.message : String(error);
  }
  assert(message.includes(fragment), label);
}

// 1) The load DTO is a strict full snapshot and preserves backend order.
assertDeepEqual(
  conversation.turns.map((turn) => turn.turn_ref),
  ["turn:first", "turn:second"],
  "renderer preserves authoritative turn order",
);
expectThrows(
  () => parseSecretaryConversation({ ...conversation, provider: "renderer-must-not-select" }),
  "unknown_conversation_field:provider",
  "load parser rejects authority/provider fields",
);
expectThrows(
  () => parseSecretaryConversation({
    ...conversation,
    turns: [{ ...conversation.turns[0], client_message_ref: "secretary-client-message:not-hex" }],
  }),
  "client_message_ref_invalid",
  "history accepts only the fixed client-message reference format",
);
expectThrows(
  () => parseSecretaryConversation({
    ...conversation,
    turns: [{ ...conversation.turns[0], error_code: "IMPOSSIBLE_SUCCEEDED_ERROR" }],
  }),
  "succeeded_turn_invalid",
  "SUCCEEDED requires assistant, no error and a terminal timestamp",
);
expectThrows(
  () => parseSecretaryConversation({
    ...conversation,
    turns: [{
      ...conversation.turns[0],
      state: "ACTIVE",
      terminal_at_utc: null,
    }],
  }),
  "nonterminal_turn_invalid",
  "nonterminal lifecycle cannot expose assistant content",
);
expectThrows(
  () => parseSecretaryConversation({
    ...conversation,
    turns: [{ ...conversation.turns[1], terminal_at_utc: null }],
  }),
  "failed_turn_invalid",
  "FAILED requires a durable terminal timestamp",
);
expectThrows(
  () => parseSecretaryConversation({
    ...conversation,
    turns: [{ ...conversation.turns[0], state: "CANCELLED" }],
  }),
  "terminal_turn_invalid",
  "CANCELLED and TIMED_OUT cannot carry assistant content",
);

// 2) The send boundary has exactly two renderer-controlled keys and its
// response embeds an authoritative complete snapshot containing the turn.
assertDeepEqual(
  createSecretaryMessageSendRequest({ message: "继续", client_message_ref: clientRef("c") }),
  { message: "继续", client_message_ref: clientRef("c") },
  "send request retains only the exact two-key payload",
);
expectThrows(
  () => createSecretaryMessageSendRequest({
    message: "继续",
    client_message_ref: clientRef("c"),
    role_session_ref: "renderer-guess",
  } as never),
  "unknown_send_request_field:role_session_ref",
  "renderer cannot select conversation identity",
);
expectThrows(
  () => createSecretaryMessageSendRequest({ message: "   ", client_message_ref: clientRef("d") }),
  "message_blank",
  "blank messages stop before invoke",
);
const sendOutcome = parseSecretaryMessageSendOutcome({
  schema_version: M4_SECRETARY_CONVERSATION_SEND_SCHEMA,
  command_receipt_ref: "command-receipt:fixture",
  turn_ref: "turn:first",
  replayed: false,
  conversation,
});
assert(sendOutcome.conversation === conversation || sendOutcome.conversation.history_ref === conversation.history_ref,
  "send outcome carries the full authoritative conversation snapshot");
expectThrows(
  () => parseSecretaryMessageSendOutcome({
    schema_version: M4_SECRETARY_CONVERSATION_SEND_SCHEMA,
    command_receipt_ref: "command-receipt:fixture",
    turn_ref: "turn:missing",
    replayed: true,
    conversation,
  }),
  "send_turn_missing",
  "send receipt cannot claim a turn absent from its snapshot",
);

// 3) Ordered turns render from the snapshot. FAILED has a user message plus a
// failure marker, never a fabricated assistant message.
const markup = renderToStaticMarkup(
  <SecretaryConversationHistory conversation={conversation} errorCode={null} state="ready" />,
);
assert(markup.includes('data-secretary-conversation-state="READY"'), "READY selector is stable");
assert(markup.includes('data-secretary-conversation-role-session-ref="role-session:secretary:fixture"'),
  "role session selector comes from backend snapshot");
assert(markup.indexOf("第一句") < markup.indexOf("第二句"), "turns render in backend order");
assert(markup.includes('data-secretary-turn-state="FAILED"'), "FAILED lifecycle is visible");
assert(markup.includes('data-secretary-conversation-error-code="M4_SECRETARY_PROVIDER_FAILURE"'),
  "FAILED provider code is visible");
const failedTurnMarkup = markup.slice(markup.indexOf('data-secretary-turn-ref="turn:second"'));
assert(failedTurnMarkup.includes('data-secretary-message-role="user"'), "FAILED turn keeps authoritative user content");
assert(!failedTurnMarkup.includes('data-secretary-message-role="assistant"'), "FAILED turn has no fake assistant content");

// 4) Composer is enabled, blank/pending states do not call the handler, and
// product copy no longer claims the transport is absent.
assert(!canSendSecretaryDraft("   ", false), "blank draft is not sendable");
assert(!canSendSecretaryDraft("消息", true), "pending draft is not sendable twice");
assert(!canSendSecretaryDraft("汉".repeat(5_334), false), "UTF-8 payload above 16000 bytes is disabled");
assert(canSendSecretaryDraft("汉".repeat(5_333), false), "UTF-8 payload at 15999 bytes remains sendable");
assert(canSendSecretaryDraft("消息", false), "nonblank idle draft is sendable");
const dockMarkup = renderToStaticMarkup(
  <WorkbenchDock
    sendPending={false}
    onActiveRightPanelChange={() => undefined}
    onActiveViewChange={() => undefined}
    onSendSecretaryMessage={async () => true}
  />,
);
assert(dockMarkup.includes('data-secretary-composer="true"'), "composer selector is present");
assert(dockMarkup.includes('data-secretary-send="true"'), "send selector is present");
assert(dockMarkup.includes('data-secretary-open-conversation="true"'), "conversation navigation selector is present");
assert(dockMarkup.includes('data-secretary-send-pending="false"'), "pending state is independent from transcript");
assert(!dockMarkup.includes("持续消息发送尚未接入") && !dockMarkup.includes("消息发送未接入"),
  "old unavailable copy is absent");

// 5) A send epoch invalidates an older initial load. Even if that deferred
// read resolves later, it cannot replace the newer conversation history.
let epoch = 0;
const initialLoadEpoch = ++epoch;
let releaseInitialLoad: () => void = () => undefined;
const deferredInitialLoad = new Promise<void>((resolve) => { releaseInitialLoad = resolve; });
const committedSnapshots: string[] = [];
const staleSettlement = deferredInitialLoad.then(() => {
  if (isCurrentSecretaryConversationEpoch(epoch, initialLoadEpoch)) committedSnapshots.push("stale-load");
});
const sendEpoch = ++epoch;
if (isCurrentSecretaryConversationEpoch(epoch, sendEpoch)) committedSnapshots.push("send-snapshot");
releaseInitialLoad();
await staleSettlement;
assertDeepEqual(committedSnapshots, ["send-snapshot"], "older load cannot overwrite a send snapshot");
const epochBeforePendingReload = epoch;
if (shouldStartSecretaryConversationReload(true)) epoch += 1;
assert(epoch === epochBeforePendingReload, "global reload is a no-op while send owns the epoch");
assert(shouldStartSecretaryConversationReload(false), "reload resumes after send reaches a terminal UI state");

console.log("m4r05-secretary-conversation-ui: strict wire, authoritative history, failure face, and enabled composer passed");
