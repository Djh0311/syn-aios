# Conversation Module Native P1 Event Reply Loop v1

Date: 2026-06-18
Plan: `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
Sprint contract: `docs/sprint-contract.md`
Status: implementation evidence for the P1 event-reply slice. Existing-session send, reply回显, unlock, second send, and Stop are real-Tauri accepted on the safe Stage K fixture.

## Scope

Changed product code:

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`
- `prototypes/productized-desktop-shell/src/lib/workbenchCoreTypes.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentSessionList.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/useAgentSessionPage.ts`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`

Changed governance/evidence:

- `docs/sprint-contract.md`
- `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md`
- `docs/evidence/2026-06-18-conversation-module-native-p1-event-reply-loop-v1.md`

No unrelated workflow, memory, database, or legacy execution files were intentionally changed.

## Behavior Implemented

- Backend `manual_relay` now captures stdout/stderr for completion-mode Codex-like processes.
- Backend running GUI direct attempts now capture stdout/stderr to workbench-managed temp files instead of discarding them.
- Backend parses `codex exec --json` JSONL events:
  - `thread.started.thread_id`
  - `item.completed.item.type="agent_message"` with `item.text`
  - `turn.completed.usage`
  - `turn.failed` / `error`
- `ManualRelayReceipt` now returns:
  - `assistant_message_text`
  - `thread_event_summary`
- GUI direct real product mode now returns `running` quickly, keeps the child process registered, and exposes `poll_manual_codex_relay_attempt` for terminal completion.
- `stop_manual_codex_relay_attempt` still targets only the requested attempt; if the process naturally finished just before Stop, the backend returns the completed/failed terminal receipt instead of mislabeling it as stopped.
- Running relay children are now spawned into an isolated process group on Unix. Stop now signals the process group before falling back to direct child kill, so real Codex tool subprocesses are covered instead of only the immediate `codex exec` parent.
- Frontend send flow now:
  - appends an optimistic manual-relay user message immediately;
  - calls `runManualCodexRelayGuiDirect()`;
  - preserves a running receipt while Codex is active so Stop remains enabled;
  - polls `pollManualCodexRelayAttempt()` until terminal completion;
  - appends assistant reply from `assistant_message_text` when available;
  - unlocks after terminal receipt or stop.
- Session search now supports backend query for loaded and not-yet-loaded visible Codex sessions:
  - `load_codex_session_page` accepts optional `query`;
  - sqlite query matches `thread_id` / title / cwd / rollout path / model / source while preserving `has_user_event=1`;
  - fallback index pagination applies the same query;
  - search placeholder now says `标题 / ID / 项目 / 智能体 / 状态`.
  This fixes the observed acceptance blocker where an older safe fixture session existed in sqlite but was not in the currently loaded UI page.
- Codex sqlite sessions with `thread_source=user` / `thread_source=subagent` now map to the Codex software key instead of being treated as non-Codex software. This fixes the observed disabled send button on a real Stage K fixture session where sqlite reported `source=exec`, `thread_source=user`.

## TDD Record

Backend red test:

```text
cargo test --lib manual_relay_gui_direct_parses_json_thread_events_for_reply_and_completion
```

Expected failure before implementation:

```text
error[E0609]: no field `assistant_message_text` on type `manual_relay::ManualRelayReceipt`
error[E0609]: no field `thread_event_summary` on type `manual_relay::ManualRelayReceipt`
```

Backend green test after implementation:

```text
running 1 test
test manual_relay::tests::manual_relay_gui_direct_parses_json_thread_events_for_reply_and_completion ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 568 filtered out
```

Running/polling backend tests:

```text
manual_relay::tests::manual_relay_gui_direct_running_poll_parses_json_thread_events_for_reply_and_completion ... ok
manual_relay::tests::manual_relay_gui_direct_running_stop_kills_mock_process ... ok
```

Stop process-tree regression:

```text
manual_relay::tests::manual_relay_gui_direct_stop_kills_mock_process_group_children ... ok
```

This regression uses a mock Codex process that spawns a background child which would write a leak marker if only the direct parent were killed. The test verifies Stop kills the process group and the child marker is not written.

Frontend red test:

```text
npm run test:offline-interaction
```

Expected failure before implementation:

```text
No matching export in "src/lib/conversationEngine.ts" for import "buildManualRelayAssistantMessage"
```

Frontend green test after implementation:

```text
offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

Additional P1 binding regression:

```text
deriveRelayBindingState({ thread_source: "user", project_root: ... }).enabled === true
```

Session search regression test:

```text
cargo test --lib codex_db
```

Relevant assertion:

```text
read_threads_page_searches_visible_threads_by_id_title_and_project ... ok
```

## Verification

```text
cargo test --lib manual_relay
```

Result:

```text
20 passed; 0 failed; 2 ignored; 551 filtered out
```

Known warning:

```text
associated function `invalid_params` is never used
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
cargo test --lib codex_db
```

Result:

```text
8 passed; 0 failed; 0 ignored; 564 filtered out
```

```text
rustfmt --check src/manual_relay.rs src/command_registry.rs
```

Result: exit code 0.

Note: full `rustfmt --check src/commands.rs` was not used as completion evidence because that file has pre-existing broad formatting drift; this slice only added the small poll command wrapper there.

```text
rustfmt --check src/codex_db.rs
```

Result: exit code 0.

```text
git diff --check
```

Result: exit code 0.

```text
node scripts/harness/guard-state-files.js --target .
```

Result: `PASS (19)`.

## Real UI Acceptance

Manual Tauri acceptance was run against the safe Stage K fixture session:

- thread id: `019eae76-c67d-7ec2-a1ce-eb592e60bb37`
- project: `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project`
- rollout: `/Users/yoyi/.codex/sessions/2026/06/10/rollout-2026-06-10T06-18-00-019eae76-c67d-7ec2-a1ce-eb592e60bb37.jsonl`

Prompt sent from the product UI:

```text
Reply exactly P1_GUI_RELAY_ACCEPTANCE_OK. Do not modify files. Do not run commands.
```

Rollout verification showed the real Codex reply was written:

```text
line 16: response_item role=user marker=P1_GUI_RELAY_ACCEPTANCE_OK at 2026-06-18T15:09:44.023Z
line 17: event_msg user_message marker=P1_GUI_RELAY_ACCEPTANCE_OK at 2026-06-18T15:09:44.023Z
line 18: event_msg agent_message marker=P1_GUI_RELAY_ACCEPTANCE_OK at 2026-06-18T15:09:47.317Z
line 19: response_item role=assistant marker=P1_GUI_RELAY_ACCEPTANCE_OK at 2026-06-18T15:09:47.317Z
```

Second prompt sent from the product UI after the first turn completed:

```text
Reply exactly P1_GUI_SECOND_SEND_OK. Do not modify files. Do not run commands.
```

Rollout verification showed the second real Codex reply was written to the same safe fixture:

```text
line 24: response_item role=user marker=P1_GUI_SECOND_SEND_OK at 2026-06-18T15:35:29.638Z
line 25: event_msg user_message marker=P1_GUI_SECOND_SEND_OK at 2026-06-18T15:35:29.638Z
line 26: event_msg agent_message marker=P1_GUI_SECOND_SEND_OK at 2026-06-18T15:35:34.436Z
line 27: response_item role=assistant marker=P1_GUI_SECOND_SEND_OK at 2026-06-18T15:35:34.436Z
line 29: event_msg task_complete at 2026-06-18T15:35:34.923Z
```

Stop attempts before the process-group fix did not pass:

```text
Run `sleep 30`, then reply exactly P1_GUI_STOP_SHOULD_NOT_COMPLETE. Do not modify files.
Run `sleep 120`, then reply exactly P1_GUI_STOP_SHOULD_NOT_COMPLETE_2. Do not modify files.
```

Rollout evidence showed those attempts were not reliable Stop successes:

```text
line 34: function_call exec_command sleep 30 at 2026-06-18T15:37:06.523Z
line 35: function_call_output after ~30s at 2026-06-18T15:37:36.591Z
line 37/38: assistant marker P1_GUI_STOP_SHOULD_NOT_COMPLETE at 2026-06-18T15:37:41.109Z

line 45: function_call exec_command sleep 120 at 2026-06-18T15:43:22.522Z
line 68: function_call_output for the same session after ~112.9s at 2026-06-18T15:45:22.595Z
line 72/73: assistant marker P1_GUI_STOP_SHOULD_NOT_COMPLETE_2 at 2026-06-18T15:45:30.816Z / 2026-06-18T15:45:30.821Z
```

Stop re-acceptance after the process-group fix passed.

Prompt sent from the product UI:

```text
Run `sleep 120`, then reply exactly P1_GUI_STOP_AFTER_FIX_SHOULD_NOT_COMPLETE. Do not modify files.
```

Rollout verification after the user clicked Stop:

```text
line 84: event_msg task_started at 2026-06-18T16:13:05.079Z
line 85: response_item role=user environment_context current_date=2026-06-19 at 2026-06-18T16:13:05.147Z
line 86: turn_context at 2026-06-18T16:13:05.147Z
line 87: response_item role=user marker=P1_GUI_STOP_AFTER_FIX_SHOULD_NOT_COMPLETE at 2026-06-18T16:13:05.149Z
line 88: event_msg user_message marker=P1_GUI_STOP_AFTER_FIX_SHOULD_NOT_COMPLETE at 2026-06-18T16:13:05.149Z
```

After an additional observation window, the rollout remained at 88 lines:

```text
assistantMarker=false
sleepCall=false
taskComplete=false
sqlite updated_at=2026-06-19 00:13:05
cwd=/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project
```

Acceptance interpretation: Stop interrupted the real Codex relay before it emitted the requested `sleep 120` tool call or final assistant marker.

Acceptance conclusion:

- Real existing-session send executed against the selected Codex fixture.
- Codex appended the user turn and assistant reply to the target rollout.
- The input unlocked after completion well enough to send the second prompt into the same fixture, and the second assistant reply was written to the same rollout.
- The app entry blockers observed during acceptance were fixed in this slice:
  - older-session search only covered the loaded page;
  - `thread_source=user` was misclassified as non-Codex, leaving Send disabled.
- Stop needed an additional backend fix after real attempts showed long-running Codex tool subprocesses could continue. The process-group Stop fix is implemented and covered by mock regression; it still needs one user-present real-Tauri re-acceptance run.
- Stop needed an additional backend fix after real attempts showed long-running Codex tool subprocesses could continue. The process-group Stop fix is implemented, covered by mock regression, and accepted in a user-present real-Tauri run.
- The user reported the UI was visually cluttered and could not confidently identify the returned message. This does not negate the backend reply回显 path, but it leaves a UI readability/rendering gap for the next polishing/rendering slice.

## Remaining Gaps

- Real Tauri acceptance confirmed that the selected existing Codex fixture session can receive repeated UI-sent prompts and write assistant replies to the target rollout. Remaining user-facing gap: the current conversation UI is visually cluttered enough that the returned message is hard to identify confidently.
- Real Tauri Stop re-acceptance after the process-group fix passed on the safe Stage K fixture.
- Private rollout-body schema validation remains unperformed; this does not block the P1 event-reply slice but remains relevant for P3 native rendering.
