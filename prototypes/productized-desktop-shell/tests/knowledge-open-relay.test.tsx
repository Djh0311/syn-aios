import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import {
  knowledgeOpenRelayAckRequest,
  knowledgeOpenRelayCanAcknowledgeOpened,
  parseKnowledgeOpenRelayIntent,
  sameKnowledgeOpenRelayIntent,
} from "../src/lib/knowledgeOpenRelay";
import { KNOWLEDGE_OPEN_RELAY_TAURI_COMMANDS } from "../src/lib/tauri";
import { NativeKnowledgeWorkspace } from "../src/views/knowledge/NativeKnowledgeWorkspace";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`[knowledge-open-relay] ${message}`);
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message}; actual=${JSON.stringify(actual)}`);
}

const intent = parseKnowledgeOpenRelayIntent({
  schema_version: 1,
  intent_id: "intent:4be78b0c",
  relative_path: "research/open-me.md",
});

assert(intent, "exact host event schema must become a typed relay intent");
assertDeep(
  intent,
  {
    schemaVersion: 1,
    intentId: "intent:4be78b0c",
    relativePath: "research/open-me.md",
  },
  "UI relay intent must contain only schema version, intent, and fixed relative path",
);
assert(
  parseKnowledgeOpenRelayIntent({
    schema_version: 1,
    intent_id: "intent:4be78b0c",
    relative_path: "research/open-me.md",
    route: "knowledge",
  }) === null,
  "extra route-like fields must not enter the UI relay contract",
);
assert(
  parseKnowledgeOpenRelayIntent({
    schema_version: 1,
    intent_id: "intent:4be78b0c",
    relative_path: "../private.md",
  }) === null
    && parseKnowledgeOpenRelayIntent({
      schema_version: 1,
      intent_id: "intent:4be78b0c",
      relative_path: "research/open-me.canvas",
    }) === null,
  "only an exact fixed-vault Markdown relative path may reach the native view",
);
assert(
  parseKnowledgeOpenRelayIntent({
    schema_version: 2,
    intent_id: "intent:4be78b0c",
    relative_path: "research/open-me.md",
  }) === null,
  "schema drift must fail closed before UI navigation",
);

assertDeep(
  knowledgeOpenRelayAckRequest(intent, "opened"),
  {
    intent_id: "intent:4be78b0c",
    relative_path: "research/open-me.md",
    outcome: "opened",
  },
  "ack command payload must only echo the exact host intent and fixed outcome",
);
assertDeep(
  KNOWLEDGE_OPEN_RELAY_TAURI_COMMANDS,
  { acknowledge: "acknowledge_knowledge_open_relay_intent" },
  "UI must retain one fixed host acknowledgement command, not a generic invoke selector",
);
assert(
  !knowledgeOpenRelayCanAcknowledgeOpened(intent, {
    typedReadCompleted: false,
    selectedRelativePath: "research/open-me.md",
    focusedRelativePath: "research/open-me.md",
  })
    && !knowledgeOpenRelayCanAcknowledgeOpened(intent, {
      typedReadCompleted: true,
      selectedRelativePath: "research/other.md",
      focusedRelativePath: "research/open-me.md",
    })
    && !knowledgeOpenRelayCanAcknowledgeOpened(intent, {
      typedReadCompleted: true,
      selectedRelativePath: "research/open-me.md",
      focusedRelativePath: null,
    }),
  "typed read, exact selected path, and verified focus are all required before opened",
);
assert(
  knowledgeOpenRelayCanAcknowledgeOpened(intent, {
    typedReadCompleted: true,
    selectedRelativePath: "research/open-me.md",
    focusedRelativePath: "research/open-me.md",
  }),
  "only the same intent's committed selection and focus can acknowledge opened",
);
assert(
  sameKnowledgeOpenRelayIntent(intent, { ...intent })
    && !sameKnowledgeOpenRelayIntent(intent, { ...intent, relativePath: "research/other.md" }),
  "a later or mismatched intent cannot clear the pending relay acknowledgement",
);

let staticAcknowledgements = 0;
const staticMarkup = renderToStaticMarkup(
  <NativeKnowledgeWorkspace
    knowledgeOpenIntent={intent}
    onKnowledgeOpenIntentOutcome={async () => {
      staticAcknowledgements += 1;
      return true;
    }}
  />,
);
assert(staticAcknowledgements === 0, "SSR/static preview must not acknowledge or touch a relay intent");
assert(staticMarkup.includes("Syn 原生知识工作区"), "relay keeps the target inside Syn's native knowledge workspace");

console.log("knowledge-open relay UI contract tests passed");
