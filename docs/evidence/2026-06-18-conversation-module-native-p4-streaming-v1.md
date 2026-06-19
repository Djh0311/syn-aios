# Conversation Module Native P4 Streaming v1

Date: 2026-06-19
Plan: `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
Status: implementation ready; real UI acceptance pending user-run verification.

## Scope

P4 adds a live ThreadEvent layer to the conversation module:

- running `codex exec --json` attempts expose captured ThreadEvents before process completion;
- poll while running refreshes the receipt with `live_events`;
- Stop preserves live events emitted before the kill/stop response;
- frontend maps live events into temporary transcript rows:
  - `thread.started` / `turn.started` / `turn.completed` as live status rows;
  - reasoning items as live reasoning blocks;
  - tool / command items as live status rows;
  - assistant message `item.started` / `item.updated` deltas as a streaming assistant tail;
- final assistant reply still uses the existing terminal receipt path, so live events do not become canonical history;
- pending/live conversation events are preserved even when the historical transcript uses canonical `event_msg` filtering.

This slice is poll-driven: the UI receives live updates through the existing running-attempt polling loop, not through a new Tauri push event channel. It still consumes the same `codex exec --json` ThreadEvent stream and is the smallest safe step from "final-only" to "visible while running".

## Changed P4 Files

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src/lib/conversationTurns.ts`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`

## TDD / Regression Coverage

Backend:

- `ManualRelayReceipt` now includes `live_events`.
- running poll parses the current stdout capture file and returns live ThreadEvents before process completion.
- final completion path uses the same ThreadEvent report parser as running poll.
- Stop refreshes live events before returning `stopped_by_user`.

New backend regression:

```text
cargo test --lib manual_relay_gui_direct_running_poll_returns_live_thread_events_before_completion
```

Frontend:

- live ThreadEvents map to temporary transcript events;
- live assistant partials render as the existing streaming tail;
- live status rows render in the main transcript flow;
- pending/live assistant messages remain visible even when canonical history prefers `event_msg` records.

## Verification

```text
cargo test --lib manual_relay
```

Result:

```text
22 passed; 0 failed; 2 ignored
```

Known warning:

```text
associated function `invalid_params` is never used
```

```text
cargo test --lib codex_transcript
```

Result:

```text
8 passed; 0 failed
```

```text
npm run test:offline-interaction
```

Result:

```text
offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
npm run typecheck
```

Result:

```text
tsc --noEmit
```

Exit code: 0.

```text
rustfmt --check src/manual_relay.rs
```

Result: exit code 0.

```text
git diff --check
```

Result: exit code 0.

```text
node scripts/harness/guard-state-files.js --target .
```

Result:

```text
PASS (19)
```

## Real UI Acceptance

Not accepted yet.

Per user direction, the user performs all real UI / true-machine acceptance. Agent verification stops at local tests, typecheck, safety checks, and evidence preparation.

Safe fixture for acceptance must remain:

```text
/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project
```

Suggested P4 acceptance prompt:

```text
Run `pwd` once, then reply exactly P4_STREAMING_OK. Do not modify files. Do not run any other commands.
```

Acceptance criteria:

- send to an existing Codex session from the Stage K fixture;
- while Codex is still running, the transcript shows live status rows such as start/reasoning/tool/progress before the final receipt;
- if assistant partial text is emitted by Codex, it appears at the transcript tail before completion;
- composer shows running live progress and event count;
- on completion, final assistant reply appears and Send unlocks;
- Stop still stops only the current attempt and preserves any live events already emitted.
