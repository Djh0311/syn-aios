# Codex Relay Pre-Real Relay Must-Fix Implementation Evidence v1

Date: 2026-06-18

Task package: `tasks/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`

Execution mode: single main agent implementation + independent read-only review line.

Review: Darwin (`019ed72a-4661-7502-988c-c57dedc60f32`) `STATUS: CLEAR_WITH_NOTE`; see `evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-review-darwin-v1.md`.

## Summary

This package implemented the three pre-real-relay must-fix guards for manual relay:

- Path precision: preview keeps a lexical fallback for display but records `path_verified`; process-mode run re-canonicalizes target paths and target hash before allowing the placeholder process path.
- One-shot confirmation: backend confirmation consumption now reserves under one lock; concurrent run attempts with the same confirmation allow exactly one winner. Frontend clears the draft on run trigger and locks send / preview / confirm while the relay receipt is running; Stop remains enabled for the running attempt and terminal receipts unlock the controls.
- Stop correctness: manual relay can spawn a harmless `/bin/sleep 30` placeholder process, register its child handle for the attempt, and stop kills + waits only that attempt. Receipt records `process_kind`, `process_id`, and `real_process_killed`.

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`: added `path_verified`, strict path verification for process modes, atomic confirmation reservation, placeholder process spawn / handle registry / stop kill+wait, process receipt fields, and an ignored env-gated real-Codex runner test that remains locked in this package.
- `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`: mirrored new target / receipt fields.
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`: locked send / preview / confirm while a relay attempt is running; showed path verification and process receipt fields; Stop remains available while running.
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`: clears the draft immediately when manual relay run is triggered.
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`: added offline assertions for running lock, terminal unlock, visible path verification, and process receipt fields.
- `docs/agent-mistake-ledger.md`: recorded the `.codex`-hosted browser plugin skill read as a process mistake.

## Boundaries Held

- No real Codex process was run.
- No `codex exec` / `codex exec resume` command was executed by this package.
- No new Tauri command was added.
- Existing old gate files had empty diff: `session_continuation_store.rs`, `k3_b1_recovery.rs`, `real_execution_command.rs`, `codex_local_runner.rs`, `h5_project_dispatch_bridge.rs`.
- Manual relay `real_codex_env_gated` mode returns `manual_relay_real_codex_env_gated_not_enabled_in_this_package`; the ignored runner exists only to prove env authorization is required and the real Codex path remains locked.
- Dirty target trees are still not auto-rolled back.

## Process Note

During attempted UI browser verification, the executor read the browser plugin skill at `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.611.61753/skills/control-in-app-browser/SKILL.md`. No auth, token, transcript body, rollout body, prompt body, or `.codex` state content was read; no `.codex` path was written. Because this package has a sensitive `.codex` boundary, the incident is recorded in `docs/agent-mistake-ledger.md` as a process mistake.

## Verification Output

### cargo test --lib manual_relay

```text
running 11 tests
test manual_relay::tests::manual_relay_real_codex_requires_env_authorization ... ignored, real Codex relay is a separate user-present window; this gate proves env authorization is required
test manual_relay::tests::manual_relay_blocks_hash_mismatch_and_duplicate ... ok
test manual_relay::tests::manual_relay_blocks_sensitive_or_codex_home_requests ... ok
test manual_relay::tests::manual_relay_confirmation_reservation_is_atomic_for_reentrant_submit ... ok
test manual_relay::tests::manual_relay_consumes_confirmation_once_and_receipt_records_contract_fields ... ok
test manual_relay::tests::manual_relay_duplicate_scope_reservation_is_atomic_for_distinct_confirmations ... ok
test manual_relay::tests::manual_relay_placeholder_process_can_be_stopped_and_reaped ... ok
test manual_relay::tests::manual_relay_preview_keeps_payload_exact_and_structured_command_safe ... ok
test manual_relay::tests::manual_relay_run_consumes_confirmation_once_for_concurrent_submit ... ok
test manual_relay::tests::manual_relay_stop_only_kills_current_attempt_and_dirty_tree_never_auto_rolls_back ... ok
test manual_relay::tests::manual_relay_strict_run_requires_verified_paths ... ok

test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 550 filtered out; finished in 0.02s
```

### Old-Gate Focused Regressions

```text
cargo test --lib codex_local_guard
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 554 filtered out; finished in 0.00s

cargo test --lib h2_real_resume
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 558 filtered out; finished in 0.04s

cargo test --lib h3_b
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 556 filtered out; finished in 0.06s

cargo test --lib k3_b
test result: ok. 11 passed; 0 failed; 2 ignored; 0 measured; 547 filtered out; finished in 0.12s

cargo test --lib h5
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 552 filtered out; finished in 0.01s

cargo test --lib product_command_store_summary_keeps_legacy_and_runner_blocked
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 559 filtered out; finished in 0.00s
```

### cargo test --lib

```text
test result: ok. 538 passed; 0 failed; 23 ignored; 0 measured; 0 filtered out; finished in 11.69s
```

### Frontend

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 259 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                     0.59 kB │ gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB │ gzip:  24.98 kB
dist/assets/index-yb1Kg5qp.js   1,007.93 kB │ gzip: 276.24 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.39s
```

### Formatting / Shape / Diff

```text
cargo fmt -- --check
```

No output; exit code 0.

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
Git HEAD: b99f16cb149ccc644e487b9dd840cc8f1f7f5528
...
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":104,"baseline":97}
```

The shape-gate warning is an existing baseline warning; this package added no Tauri command.

```text
git diff --check
```

No output; exit code 0.

### Boundary Scans

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
```

No output; old gate file diff is empty.

```text
rg -n -F 'Command::new' <changed files>
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs:843:    let mut command = Command::new(&command_plan.program);
```

Classification: harmless placeholder process spawn only; no real Codex command path.

```text
rg -n -F 'codex exec' <changed files>
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:368:        redacted_preview: "codex exec resume <session> <stdin prompt>",
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:399:      redacted_preview: "codex exec resume <session> <stdin prompt>",
```

Classification: offline fixture redacted preview only.

## Independent Review

Darwin first returned `STATUS: FINDINGS` with:

- P1: duplicate-scope check was not atomic for distinct confirmations.
- P2: frontend button lock could be bypassed via textarea / Enter / parent submit.

Both findings were fixed and re-reviewed. Final review result:

```text
STATUS: CLEAR_WITH_NOTE
FINDINGS:
- No P1/P2 blocking findings remain.
```

Residual note: Darwin did not independently rerun the full test suite and true browser interaction verification remains unavailable in this window.

```text
rg -n -F '/Users/yoyi/.codex' <changed files>
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs:1099:            "read /Users/yoyi/.codex/auth.json",
```

Classification: sensitive-request blocking test only.

## UI Browser Verification

Attempted but not completed:

- Starting a temporary Vite server on `127.0.0.1:4173` failed in sandbox with `listen EPERM`.
- Escalation request to start the local dev server was rejected by policy.
- Existing `127.0.0.1:5173` was occupied by another node process; opening it showed only `工作台启动失败` / `Cannot read properties of undefined (reading 'metadata')`, so it could not prove the changed manual relay UI.

UI behavior is verified by `npm run typecheck`, `npm run test:offline-interaction`, and `npm run build`; true browser verification remains a residual gap for consulting-line review.

## Continuation Fresh Verify

Continuation agent re-ran the key gates on 2026-06-18 before handoff. The tree remains intentionally dirty because this package stops before `git add` / `git commit`.

### checkpoint-audit

```text
checkpoint-audit: /Users/yoyi/workspace/product-line
Package: 2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1
Resolved claims:
- impl commit:   (none)
- task commit:   (none)
- review file:   evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-review-darwin-v1.md
- CURRENT.md block found: false

Checks:
- [WARN] tree_clean: {"declared_dirty":true,...}
- [PASS] review_status: {"file":"evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-review-darwin-v1.md","status":"CLEAR"}
- [NA] gates_green: skipped (--skip-gates)

VERDICT: PASS
```

### Fresh Test Tails

```text
cargo test --lib manual_relay
test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 550 filtered out; finished in 0.04s
```

```text
cargo test --lib
test result: ok. 538 passed; 0 failed; 23 ignored; 0 measured; 0 filtered out; finished in 10.64s
```

```text
npm run typecheck
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

```text
npm run test:offline-interaction
offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
npm run build
dist/index.html                     0.59 kB | gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB | gzip:  24.98 kB
dist/assets/index-Bbojue3u.js   1,008.04 kB | gzip: 276.29 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 1.87s
```

```text
cargo fmt -- --check
```

No output; exit code 0.

```text
node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 1
Git HEAD: b99f16cb149ccc644e487b9dd840cc8f1f7f5528
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":104,"baseline":97}
```

The shape warning is the existing Tauri command count warning; this package added no Tauri command.

```text
git diff --check
```

No output; exit code 0.

### Fresh Boundary Scans

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
```

No output; old gate file diff remains empty.

```text
rg -n -F Command::new <changed files and package docs>
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs:843:    let mut command = Command::new(&command_plan.program);
```

Classification: harmless placeholder process spawn only; no real Codex command path.

```text
rg -n -F 'codex exec' <changed files and package docs>
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:374:        redacted_preview: "codex exec resume <session> <stdin prompt>",
prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx:405:      redacted_preview: "codex exec resume <session> <stdin prompt>",
docs/agent-mistake-ledger.md:182:- A verification scan intended to search for misleading H3-B completion wording unexpectedly attempted to run `codex exec` because Markdown backticks inside a double-quoted shell argument were interpreted by the shell.
```

Classification: offline fixture redacted previews plus historical mistake-ledger entry only.

```text
rg -n -F '/Users/yoyi/.codex' <changed files and package docs>
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs:1148:            "read /Users/yoyi/.codex/auth.json",
docs/agent-mistake-ledger.md:84:- While attempting real browser verification for the manual relay UI, the agent read `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.611.61753/skills/control-in-app-browser/SKILL.md`.
```

Classification: sensitive-request blocking test plus process-mistake ledger/package evidence. No auth, token, transcript body, rollout body, prompt body, or `.codex` state content was read or written by the implementation.
