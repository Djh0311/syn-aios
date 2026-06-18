# Codex Relay B2 GUI Direct Send Result v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-b2-gui-real-send-v1.md`
Base HEAD: `9b7360a`
Commit: not committed; awaiting consulting-line review and user decision.

## Summary

B2 GUI direct-send implementation is ready for review. The bound Codex conversation composer now sends through `run_manual_codex_relay_gui_direct` only when a project and existing Codex session are bound. The UI keeps the target visible, including the exact `thread_id`, and preserves the one-shot boundary: `manual_once`, `auto_chain=false`, `sandbox=workspace-write`.

No true Codex relay was run. No ignored true-run tests were executed. No `MANUAL_RELAY_REAL_CODEX_CONFIRM` was set.

## Changed Areas

- Backend contract and command:
  - `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- Frontend invocation and types:
  - `prototypes/productized-desktop-shell/src/lib/tauri.ts`
  - `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`
- Agent conversation UI:
  - `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
  - `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- Offline interaction fixtures:
  - `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`
  - `prototypes/productized-desktop-shell/tests/helpers/offlineExecutionRunQueueTextFixtures.ts`
  - `prototypes/productized-desktop-shell/tests/helpers/offlineShellScenarioTextFixtures.ts`
- Evidence:
  - `evidence/2026-06-18-codex-relay-b2-gui-real-send-v1.md`
  - `evidence/2026-06-18-codex-relay-b2-gui-real-send-artifacts/tauri-app-real-launch-post-p2.png`
  - `evidence/2026-06-18-codex-relay-b2-gui-real-send-review-erdos-v1.md`

## Key Safety Facts

- GUI direct path uses `RealCodexProductGui`, not the old env-gated test-runner string.
- Test helper for GUI direct refuses `RealCodexEnvGated` and only accepts mock/fixture modes.
- Target validation requires bound Codex session, canonical project-root cwd, `workspace-write`, and allowed root exactly equal to project root.
- Command validation rejects shell invocation, prompt-in-argv, missing `--sandbox`, missing `--add-dir`, and approval-bypass args.
- Changed diff does not add `Command::new`.
- Old gate 5-file diff is empty:
  - `session_continuation_store.rs`
  - `k3_b1_recovery.rs`
  - `real_execution_command.rs`
  - `codex_local_runner.rs`
  - `h5_project_dispatch_bridge.rs`

## Verification

- `cargo test --lib manual_relay`: 16 passed / 2 ignored.
- `npm run test:offline-interaction`: 15 passed plus R4 checks passed.
- `npm run typecheck`: passed.
- `npm run build`: passed with existing large chunk warning.
- `cargo test --lib`: 544 passed / 24 ignored.
- Old gate focused tests:
  - `codex_local_runner`: 12 passed.
  - `session_continuation_store`: 16 passed / 4 ignored.
  - `real_execution_command`: 36 passed / 7 ignored.
  - `k3_b1_recovery`: 5 passed.
  - `h5_project_dispatch_bridge`: 4 passed.
- `cargo fmt -- --check`: passed.
- `node scripts/harness/workbench-shape-gate.js --mode check`: pass; 0 errors / 1 warning (`tauri_command_total_increased`, explained as the single new GUI direct command).
- `git diff --check`: passed.
- Tauri app was launched directly and post-P2 screenshot evidence was captured; Send was not clicked.

## Independent Review

Review line: Erdos. Initial status was `CLEAR_WITH_P2`; both P2s were fixed before this handoff:

- Exact `thread_id` is now always visible in the target strip as `会话ID：<thread_id>`.
- Frontend direct-send enablement is explicitly Codex-only; non-Codex sessions stay blocked and have offline coverage.

## Review Ask

Please review:

1. The GUI direct command is narrow enough and does not weaken the B1/B2/H/K old gates.
2. The command-plan guard is at least as strict as direct Codex use for approval/sandbox: no approval-bypass argv, stdin prompt, no shell, `--sandbox workspace-write`, `--add-dir <project root>`.
3. The frontend does not hide target identity and only sends when the conversation is bound to a Codex session.
4. The task-package boundary is honest: implementation only; first real GUI relay is still a separate user-present window.

## Do Not Claim

- Do not claim first GUI real relay was run.
- Do not claim mariotest or a real project was sent through GUI.
- Do not claim old K3/H2/H5 gates were opened.
- Do not claim auto-chain, multi-agent execution, role injection, or product global read/write path switching was enabled.
