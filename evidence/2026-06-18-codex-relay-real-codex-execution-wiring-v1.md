# Codex Relay Real Codex Execution Wiring Evidence v1

Date: 2026-06-18

Task package: `tasks/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`

Execution mode: Strict Path. ③a only wires the env-gated real Codex process path. This package did not run true Codex, did not run ignored real-Codex tests, did not set `MANUAL_RELAY_REAL_CODEX_CONFIRM`, and did not read or write `/Users/yoyi/.codex`.

## Summary

This package changes manual relay `real_codex_env_gated` from a hard-coded not-enabled error into a locked real-Codex process configuration:

- Real path remains locked by `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`.
- Normal tests use local mock-codex scripts only under `#[cfg(test)]`.
- The real path config is `process_kind=real_codex`, `program=codex`, structured argv, stdin prompt, and a workbench-managed last-message path.
- Readback reads only the workbench-managed last-message file.
- Stop targets the registered running attempt and preserves receipt flags instead of rewriting them.

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`: wired env-gated process config, mock-codex test process helpers, last-message readback, stop receipt preservation, and tests.
- `docs/agent-mistake-ledger.md`: added M-0005 for the initial review finding where mock process modes were left production-reachable.
- `tasks/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`: task package for this implementation.
- `evidence/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`: this evidence record.
- `handoffs/2026-06-18-codex-relay-real-codex-execution-wiring-v1-result.md`: handoff summary.

Unrelated untracked file intentionally excluded from this package: `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`.

## Implementation Notes

- Added `MANUAL_RELAY_REAL_CODEX_CONFIRM_ENV` / `MANUAL_RELAY_REAL_CODEX_CONFIRM_VALUE`.
- Added process modes:
  - `placeholder_process_sleep`
  - `mock_codex_process:<script>` test-only
  - `mock_codex_process_sleep:<script>` test-only
  - `real_codex_env_gated`
- Added `spawn_codex_like_process`, `run_codex_like_process_to_completion`, `read_last_message_summary`, `process_config_for_mode`, and `ensure_real_codex_env_authorized`.
- Added a production-deny gate after Dirac review: `mock_codex_process_mode_allowed()` returns `true` only under `#[cfg(test)]`; non-test builds reject mock process modes with `manual_relay_mock_codex_process_mode_test_only`.

## TDD Record

Behavior protected before implementation:

- Missing real-Codex env authorization rejects before any process is registered.
- Mock codex writes a workbench-managed last-message and produces hash/size/readback status.
- Mock codex sleep process can be killed and reaped by `stop_manual_relay_attempt`.

Initial focused run failed before implementation, as expected:

```text
cargo test --lib manual_relay
running 14 tests
test manual_relay::tests::manual_relay_mock_codex_process_can_be_stopped_and_reaped ... FAILED
test manual_relay::tests::manual_relay_mock_codex_process_writes_last_message_readback ... FAILED
...
test result: FAILED. 4 passed; 9 failed; 1 ignored; 0 measured; 550 filtered out
```

After implementation and P1 fix, the same focused suite passed.

## Review Finding And Fix

Initial independent review line: Dirac (`019ed78e-f036-78f0-b576-e602fc87a79f`).

Initial result: `STATUS: FINDINGS`.

P1 found: `mock_codex_process:<script>` and `mock_codex_process_sleep:<script>` were production-compiled process modes reachable through `ManualRelayRunInput.mock_behavior`, so a caller could theoretically point manual relay at an arbitrary local executable without the real-Codex env gate.

Fix:

- Added `is_mock_codex_process_mode`.
- Added `mock_codex_process_mode_allowed` with `#[cfg(test)] -> true` and `#[cfg(not(test))] -> false`.
- `run_manual_relay_once` now rejects mock process modes outside test builds before path verification or spawning.
- Added M-0005 to `docs/agent-mistake-ledger.md`.
- Re-ran `cargo check --lib` to compile the non-test cfg path and `cargo test --lib manual_relay` to keep the test-only mock path working.

Final rereview: Dirac (`019ed78e-f036-78f0-b576-e602fc87a79f`) returned `STATUS: CLEAR` after confirming the documentation P2 was closed. Review record: `evidence/2026-06-18-codex-relay-real-codex-execution-wiring-review-dirac-v2.md`.

## Verification Output

### Production cfg compile path

```text
cargo check --lib
warning: `codex-governance-workbench` (lib) generated 605 warnings (run `cargo fix --lib -p codex-governance-workbench` to apply 6 suggestions)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.08s
```

### Focused manual relay

```text
cargo test --lib manual_relay
running 14 tests
test manual_relay::tests::manual_relay_real_codex_requires_env_authorization ... ignored, real Codex relay is a separate user-present window; this gate proves env authorization is required
...
test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 550 filtered out; finished in 0.30s
```

### Old-gate focused regressions

```text
cargo test --lib codex_local_guard
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 558 filtered out; finished in 0.00s
```

```text
cargo test --lib h2_real_resume
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 562 filtered out; finished in 0.11s
```

```text
cargo test --lib h3_b
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 560 filtered out; finished in 0.07s
```

```text
cargo test --lib k3_b
test result: ok. 11 passed; 0 failed; 2 ignored; 0 measured; 551 filtered out; finished in 0.47s
```

```text
cargo test --lib h5
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 556 filtered out; finished in 0.00s
```

```text
cargo test --lib product_command_store_summary_keeps_legacy_and_runner_blocked
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 563 filtered out; finished in 0.00s
```

### Full backend

```text
cargo test --lib
running 564 tests
...
test result: ok. 541 passed; 0 failed; 23 ignored; 0 measured; 0 filtered out; finished in 11.96s
```

### Frontend

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
dist/index.html                     0.59 kB │ gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB │ gzip:  24.98 kB
dist/assets/index-Bbojue3u.js   1,008.04 kB │ gzip: 276.29 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 3.97s
```

### Formatting / shape / diff

```text
cargo fmt -- --check
```

No output; exit code 0.

```text
node scripts/harness/workbench-shape-gate.js --mode check
Status: pass
Errors: 0
Warnings: 1
Git HEAD: a65e6d7a391dbf2f03530b77101c3a15fdcd36db
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":104,"baseline":97}
```

The shape warning is an existing Tauri command count baseline warning. This package added no Tauri command.

```text
git diff --check
```

No output; exit code 0.

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
```

No output; old-gate 5-file diff is empty.

### Guard-state check

```text
node scripts/harness/guard-state-files.js --target .
PASS (19)
FAIL (0)
```

The command emits a large protected-path inventory; relevant summary is PASS / no FAIL.

## Boundary Scans

```text
rg -n -F Command::new prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
755:    let mut command = Command::new(&command_plan.program);
1161:    let mut command = Command::new(&command_plan.program);
```

Classification:

- line 755: new codex-like process helper. Mock executable paths are test-only after P1 fix; real `codex` requires the env gate.
- line 1161: existing placeholder `/bin/sleep` helper.

```text
rg -n -F "codex exec" prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
```

No output.

```text
rg -n -F /Users/yoyi/.codex prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
1467:            "read /Users/yoyi/.codex/auth.json",
```

Classification: sensitive-request blocking test string only.

```text
rg -n "read_to_string|fs::read|File::|\\.codex|Command::new|spawn_codex_like_process|read_last_message_summary|ensure_real_codex_env_authorized|process_config_for_mode|manual_relay_process_mode|mock_codex_process_mode_allowed|manual_relay_mock_codex_process_mode_test_only" prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
334:    if is_mock_codex_process_mode(&input.mock_behavior) && !mock_codex_process_mode_allowed() {
335:        return Err("manual_relay_mock_codex_process_mode_test_only".to_string());
337:    let process_mode = manual_relay_process_mode(&input.mock_behavior);
419:        let process_config = process_config_for_mode(process_mode, command_plan)?;
539:fn manual_relay_process_mode(mock_behavior: &str) -> Option<ManualRelayProcessMode> {
560:fn mock_codex_process_mode_allowed() -> bool {
565:fn mock_codex_process_mode_allowed() -> bool {
569:fn process_config_for_mode(
603:            ensure_real_codex_env_authorized()?;
615:fn ensure_real_codex_env_authorized() -> Result<(), String> {
644:    let child = spawn_codex_like_process(
697:    let mut child = spawn_codex_like_process(
707:        read_last_message_summary(&process_config.command_plan.last_message_path);
744:fn spawn_codex_like_process(
755:    let mut command = Command::new(&command_plan.program);
789:fn read_last_message_summary(path: &str) -> (String, Option<String>, Option<i64>) {
790:    match fs::read_to_string(path) {
1108:        || manual_relay_process_mode(mock_behavior).is_some()
1161:    let mut command = Command::new(&command_plan.program);
1252:        "/users/yoyi/.codex",
1253:        ".codex",
1467:            "read /Users/yoyi/.codex/auth.json",
1882:            process_config_for_mode(ManualRelayProcessMode::RealCodexEnvGated, plan.clone())
```

Classification: `fs::read_to_string` is only in `read_last_message_summary`, reading `command_plan.last_message_path`, the workbench-managed last-message file. `.codex` hits are deny-list and test strings.

## Boundaries Held

- No real `codex` process was run.
- No `codex exec` / `codex exec resume` shell command was run.
- No ignored real-Codex test was run.
- No `.codex` file was read or written in this package.
- No browser verification was attempted, per M-0004 boundary.
- No old-gate file was modified.
- No new Tauri command was added.
- `payload_layers` remains empty and `effective_prompt == original_user_text`.

## Residual / Deferred

- ③a proves terminal process wiring with mock-codex scripts and proves the real path is env-gated.
- First true real-Codex terminal behavior, including real terminal receipt/readback, remains ③b user-present validation.
- This package does not claim true Codex relay completed, Codex auth validity, or product-wide execution unlock.
