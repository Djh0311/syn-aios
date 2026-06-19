# Conversation Module Native P3 Native Rendering v1

Date: 2026-06-19
Plan: `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
Status: implementation ready; real UI acceptance pending user-run verification.

## Scope

P3 changes the conversation surface from a chat-bubble/developer-field-heavy view into a Codex-style transcript surface:

- main transcript uses flat `codex-transcript-item` rows for user/assistant messages;
- native Codex events are promoted into the main flow as quiet `codex-status-item` rows:
  - `tool_call`;
  - `tool_result`;
  - `command_output`;
  - `compacted`;
  - `system_context` with `payload_type=reasoning`;
- status rows keep raw/dev fields in expandable details: arguments, output, stdout, stderr, exit code, and warnings;
- compacted events render as an "上下文已自动压缩" divider;
- reasoning events render as a folded "思考" block;
- inline fenced code blocks still render with copy affordance;
- composer visible surface is reduced to target summary, prompt, Send, and Stop only while running;
- manual relay target, boundary, preview, and receipt details are collapsed under "发送边界与回执".

This slice does not implement P4 true streaming. It also does not invent a `parsed_cmd` field when the current transcript read model does not expose one; command labels are derived from available fields (`tool_name`, `arguments.cmd`, stdout/stderr/exit_code).

## Changed P3 Files

- `prototypes/productized-desktop-shell/src/views/agent/TranscriptViews.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/src/manualRelay.css`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineProjectRuntimeTranscriptRoleTextFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## TDD / Regression Coverage

Frontend offline interaction now asserts:

- transcript markup includes `codex-transcript-item`;
- transcript markup no longer includes `chat-bubble`;
- tool, command output, reasoning, and compacted fixtures render as `codex-status-item` status rows;
- tool-only transcript tails no longer show the empty conversation state;
- existing send, new-session send, optimistic pending messages, Stop, and manual relay details remain covered.

The session-center hardening fixture was updated for the P3 contract:

- hidden virtual-window count changed from 3 to 4 because the tool event is now visible in the main transcript;
- the old "工具事件不应默认进入主对话流" assertion was replaced with "工具事件应作为 Codex 状态行进入主对话流详情".

## Verification

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
cargo test --lib manual_relay
```

Result:

```text
21 passed; 0 failed; 2 ignored; 551 filtered out
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
8 passed; 0 failed; 0 ignored; 566 filtered out
```

Known warning:

```text
associated function `invalid_params` is never used
```

```text
git diff --check
```

Result: exit code 0.

```text
node scripts/harness/guard-state-files.js --target .
```

Result: `PASS (19)`.

## Real UI Acceptance

Not accepted yet.

Per user direction, the user performs all real UI / true-machine acceptance. Agent verification stops at local tests, typecheck, safety checks, and evidence preparation.

P2 new-session and continuation flow is now real-UI accepted by user-run verification on 2026-06-19. P3 visual rendering acceptance remains separate.

Safe fixture for acceptance must remain:

```text
/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project
```

Suggested P3 acceptance checklist:

- open a Codex conversation with normal user/assistant text and verify the transcript is flat, not bubble-style;
- verify tool calls and command output appear as quiet status rows in the main flow;
- expand a tool/status row and confirm raw fields are in details rather than always visible;
- verify reasoning appears as a folded "思考" block when present;
- verify compacted history appears as "上下文已自动压缩";
- verify composer is no longer dominated by relay target/receipt/boundary fields;
- verify P1 existing-session send and Stop still work;
- verify P2 new-session send still uses the Stage K fixture and remains usable after the P3 rendering changes.
