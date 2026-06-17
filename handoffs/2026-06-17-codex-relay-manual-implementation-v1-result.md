# Codex relay manual implementation result handoff v1

Date: 2026-06-18 01:17 CST
Task: `tasks/2026-06-17-codex-relay-manual-implementation-v1.md`
Design authority: `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`
Status: implementation complete, waiting for independent review result and consulting-line approval before commit.

## What Changed

- Backend:
  - Added `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`.
  - Registered `manual_relay` in `src-tauri/src/lib.rs`.
  - Added four narrow commands in `src-tauri/src/commands.rs` and `src-tauri/src/command_registry.rs`:
    - `preview_manual_codex_relay`
    - `confirm_manual_codex_relay_once`
    - `run_manual_codex_relay_once`
    - `stop_manual_codex_relay_attempt`
- Frontend:
  - Added manual relay TypeScript contracts in `src/lib/types/manualRelay.ts`.
  - Exported them through `src/lib/types.ts`.
  - Added four Tauri wrappers in `src/lib/tauri.ts`.
  - Added `manual_relay` pending metadata through `buildManualRelayPendingUserMessage`.
  - Added a visible manual relay panel to `AgentChatComposer`.
  - Added preview/confirm/mock-run/stop handlers to `AgentConversationShell`.
  - Added isolated styles in `src/manualRelay.css`, imported from `src/main.tsx`.
- Tests:
  - Added Rust unit tests for manual relay hash mismatch, duplicate blocking, denied sensitive material, stop-only-current-attempt, dirty-tree no auto rollback, exact payload, empty payload layers, and structured command plan.
  - Added Rust unit tests for target hash mismatch, confirmation id mismatch, terminal confirmation replay blocking, and receipt contract fields.
  - Extended offline conversation engine tests for manual relay UI, one-shot warning, exact payload, target display, Stop control, and `real_codex_executed=false` metadata.

## Boundary Confirmed

- No real Codex run in this package.
- No `Command::new` added by this package.
- No `.codex` write/read path added by this package.
- No old gate was weakened:
  - `run_real_resume_phase_b_with_runner` untouched.
  - K3-B1/K3-B2 recovery and gate files untouched.
  - H5/PCR product-command implementation files untouched.
  - `codex_local_runner` untouched; manual relay only calls `inspect_codex_local_execution_guard`.
- Existing decision-only send path stays intact; `handleSubmitConversationDraft()` remains the old pending-message-only path.
- Manual relay run is fixture/mock only and returns receipts with `real_codex_executed=false`.
- Stop is clickable in UI when the fixture receipt is running and maps to `stop_manual_codex_relay_attempt`; it only stops the requested manual relay attempt registry entry.
- Confirmation ids are consumed once; a completed fixture receipt cannot be replayed with the same confirmation.
- Receipt now carries target, prompt hash/length/exactness, command plan, timestamps, exit code, last-message size, changed files, and git before/after fields.
- UI now displays allowed write roots in the manual relay preview.

## Verification

- `cargo test --lib manual_relay`: passed, 5 passed.
- `cargo test --lib codex_local_guard`: passed, 6 passed.
- `cargo test --lib h2_real_resume`: passed, 2 passed.
- `cargo test --lib h3_b`: passed, 3 passed / 1 ignored.
- `cargo test --lib k3_b`: passed, 11 passed / 2 ignored.
- `cargo test --lib h5`: passed, 6 passed / 2 ignored.
- `cargo test --lib product_command_store_summary_keeps_legacy_and_runner_blocked`: passed, 1 passed.
- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed; existing bundle-size warning remains.
- `cargo test --lib`: passed, 533 passed / 22 ignored.
- `cargo fmt -- --check`: passed.
- `node scripts/harness/workbench-shape-gate.js --mode check`: passed with one expected command-count warning.
- `git diff --check`: passed.

Full verification excerpts are in `evidence/2026-06-17-codex-relay-manual-implementation-v1.md`.

## Consulting-Line Checkpoints

- Confirm the manual relay contract is sufficiently narrow for "甲·中转".
- Confirm `payload_layers=[]` and `future_hooks` are adequate v1 placeholders.
- Confirm the frontend wording does not overclaim true relay or true Codex execution.
- Confirm Hegel initial `STATUS: FINDINGS` P1 items are sufficiently fixed before consulting-line approval.
- Confirm path canonicalization fallback is acceptable for mock-only implementation, and re-audit before first real relay.
- Confirm the shape-gate command-count warning is accepted as expected package impact.
- Confirm whether this package should add any extra governance checkpoint before the first user-present real relay package.

## Not Done

- Did not run the first real relay.
- Did not connect real Codex execution.
- Did not write or read `.codex`.
- Did not implement roles, task package injection, memory packet injection, automatic chaining, background worker dispatch, true rollback, or true stop of external Codex processes.
- Did not commit.
