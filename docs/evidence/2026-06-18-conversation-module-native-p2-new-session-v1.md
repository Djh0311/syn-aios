# Conversation Module Native P2 New Session v1

Date: 2026-06-19
Plan: `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
Status: real UI accepted by user-run verification on 2026-06-19.

## Scope

P2 adds the product path for starting a new Codex conversation from the conversation UI:

- backend GUI direct new-session command: no `resume`, uses `codex exec --json`, `--output-last-message`, `-C`, `--sandbox workspace-write`, and `--add-dir`;
- frontend send mode: `继续当前对话` vs `新建对话`;
- new-session composer unlock rule: selected project root is sufficient; selected existing session is not required;
- new thread handoff: `thread.started.thread_id` is used to search/reload the session page and auto-select the new session if it is already indexed.

Changed product code in this P2 slice:

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/useAgentSessionPage.ts`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`

## Behavior Implemented

- `run_manual_codex_relay_gui_direct_new_session` is registered as a Tauri command.
- `ManualRelayGuiDirectNewSessionInput` carries prompt, project root, cwd, sandbox, allowed write roots, and requester.
- New-session validation rejects bound target sessions and verifies the command plan does not include `resume`.
- The command plan remains inside the existing manual relay safety shell:
  - no prompt in argv;
  - no shell invocation;
  - no `--full-auto`;
  - no dangerous approval/sandbox bypass;
  - `workspace-write` plus project write root only.
- The conversation bar exposes a send-mode selector.
- Existing-session mode keeps the P1 behavior and optimistic message/reply回显.
- New-session mode calls `runManualCodexRelayGuiDirectNewSession()` with selected project root as `target_project_root`, `target_cwd`, and the only `allowed_write_roots` entry.
- When a receipt includes `thread_event_summary.thread_id`, the UI sets the session search query to that id and asks `AgentView` to reload/select it.

## TDD Record

Backend new-session regression:

```text
cargo test --lib manual_relay_gui_direct_new_session_uses_unbound_exec_json_command
```

Result:

```text
test manual_relay::tests::manual_relay_gui_direct_new_session_uses_unbound_exec_json_command ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 573 filtered out
```

The regression asserts:

- target has `target_session_id = None`;
- target has `new_session = true`;
- argv does not include `resume`;
- argv includes `--json`, `--output-last-message`, `--sandbox workspace-write`, and `--add-dir`;
- argv does not include `--full-auto` or dangerous bypass flags;
- mock ThreadEvent parsing returns `thread-new-session-json-fixture`;
- assistant text is `NEW_SESSION_JSON_EVENT_REPLY_OK`.

Frontend offline interaction regression:

```text
npm run test:offline-interaction
```

Relevant P2 assertion:

```text
P2 新建对话无需 selectedSession，Enter 应调用 new-session 发送 handler
```

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
rustfmt --check prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
```

Result: exit code 0.

Note: full `rustfmt --check src/commands.rs` is still not used as completion evidence because `commands.rs` has pre-existing broad formatting drift; this slice only added a small command wrapper there.

```text
git diff --check
```

Result: exit code 0.

```text
node scripts/harness/guard-state-files.js --target .
```

Result: `PASS (19)`.

## Real UI Acceptance

Accepted by user-run real UI verification on 2026-06-19.

Acceptance evidence:

- user selected new-session mode in the running desktop UI and sent from the Stage K fixture;
- a new rollout file was observed by metadata only for thread `019ede51-6ca4-78a2-b658-6c3ef465ea14`;
- after the continuation prompt, metadata-only checks showed the same rollout updated to `2026-06-19 13:38:49 +0800`, size `49597`, line count `20`;
- user confirmed the continued UI response was visible and Send was unlocked after the `P2_CONTINUE_AFTER_NEW_OK` prompt.

Earlier 2026-06-19 user note: "现在都测不了，ui太乱了". P2 was kept unverified until the P3 UI simplification/rendering pass made the flow usable enough for true-machine acceptance.

2026-06-19 follow-up during P2/P3 acceptance: user reported the new thread could not be found via UI search after a new-session send. A rollout file was observed by filename for thread `019ede51-6ca4-78a2-b658-6c3ef465ea14`, but the session list/search path depended on Codex sqlite/index visibility. Follow-up fix added a bounded fallback:

- when `load_codex_session_page` query returns no sqlite/index rows, the backend searches Codex `sessions` / `archived_sessions` rollout filenames for the thread id or short id;
- the fallback returns a readable session card with `rollout_path`, session metadata from `session_meta`, and warning `session_index_pending_rollout_filename_fallback`;
- transcript page loading also falls back to the same rollout filename lookup if sqlite/index has not caught up.

Regression:

```text
cargo test --lib rollout_filename_fallback_finds_new_thread_before_sqlite_index_catches_up
```

Result:

```text
1 passed; 0 failed
```

Additional checks after the fix:

```text
cargo test --lib manual_relay_gui_direct_new_session_uses_unbound_exec_json_command
npm run test:offline-interaction
git diff --check
```

Results:

```text
manual relay new-session regression: 1 passed; 0 failed
offline interaction tests passed: 15
git diff --check: exit code 0
```

Per user direction, the user will perform all real UI / true-machine acceptance. Agent verification stops at local tests, typecheck, safety checks, and evidence preparation.

Safe fixture for acceptance must remain:

```text
/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project
```

Suggested P2 acceptance prompt:

```text
Reply exactly P2_GUI_NEW_SESSION_OK. Do not modify files. Do not run commands.
```

Acceptance criteria:

- switch send mode to `新建对话`;
- choose the safe Stage K fixture project;
- send the prompt;
- Codex returns `P2_GUI_NEW_SESSION_OK`;
- a new thread id is visible via receipt/search/list;
- the new conversation can be selected and then continued.
