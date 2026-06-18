# Codex Relay Real Relay Runner Entry B1 Evidence v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md`
Baseline HEAD: `5ea7c48`
Status: implementation verified locally; independent review `CLEAR_WITH_NOTE`.

## Scope

This package adds the B1 runner-entry shape for the manual relay path:

- Adds an ignored, env-gated B1 real Codex runner test entry that calls `run_manual_relay_once` with `real_codex_env_gated`.
- Adds temp fixture defaults for the first user-present real run: temp project root, `git init`, new session, `workspace-write`, allowed root limited to that temp project, minimal `hello.txt` prompt.
- Adds mock-process structural tests proving the runner shape, last-message readback, and stop behavior without spawning real Codex.
- Gates placeholder process modes as test-only in production builds, matching the prior mock-process test-only guard.

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md` is present as an untracked task package supplied for this work.

No frontend files, Tauri command registration files, old execution-gate files, or `.codex` files were changed.

## Boundary

This package did not:

- run true Codex;
- set `MANUAL_RELAY_REAL_CODEX_CONFIRM`;
- run ignored tests;
- add Tauri commands or frontend wiring;
- modify old-gate files: `session_continuation_store.rs`, `k3_b1_recovery.rs`, `real_execution_command.rs`, `codex_local_runner.rs`, `h5_project_dispatch_bridge.rs`;
- read transcript/rollout/prompt body or `.codex` auth/session content;
- claim product UI stop is wired.

## TDD Note

The continuation summary records that the B1 mock runner-entry test was first run red before helper implementation. The current app terminal has no recoverable raw red output, so this evidence does not present that as fresh proof. Fresh evidence below proves the implemented green path and regression gates.

## Verification Matrix

- `cargo test --lib manual_relay`: PASS, `15 passed; 2 ignored`.
- Old-gate focused tests: PASS for `codex_local_guard`, `h2_real_resume`, `h3_b`, `k3_b`, `h5`, and `product_command_store_summary_keeps_legacy_and_runner_blocked`.
- `cargo test --lib`: PASS, `543 passed; 24 ignored`.
- `cargo fmt -- --check`: PASS.
- `npm run typecheck`: PASS.
- `npm run test:offline-interaction`: PASS, `offline interaction tests passed: 15`.
- `npm run build`: PASS with existing Vite large-chunk warning.
- `node scripts/harness/workbench-shape-gate.js --mode check`: PASS, `Errors: 0`, `Warnings: 1` for existing Tauri command ratchet warning.
- `git diff --check`: PASS, no output.
- Old-gate 5-file diff: PASS, no output.
- Danger scans: `codex exec` absent in `manual_relay.rs`; `.codex` hit is only an existing denied prompt test sample; `Command::new` hits are two unified spawn sites plus one test-only `git init` helper.

## Raw Output Tails

### `cargo test --lib manual_relay`

```text
running 17 tests
test manual_relay::tests::manual_relay_b1_real_codex_runner_entry_requires_user_present_env ... ignored, B1 first true Codex relay requires user-present env authorization; do not run in implementation package
...
test manual_relay::tests::manual_relay_strict_run_requires_verified_paths ... ok

test result: ok. 15 passed; 0 failed; 2 ignored; 0 measured; 550 filtered out; finished in 0.15s
```

### Old-Gate Focused Tests

```text
cargo test --lib codex_local_guard
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 561 filtered out; finished in 0.00s

cargo test --lib h2_real_resume
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 565 filtered out; finished in 0.04s

cargo test --lib h3_b
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 563 filtered out; finished in 0.06s

cargo test --lib k3_b
test result: ok. 11 passed; 0 failed; 2 ignored; 0 measured; 554 filtered out; finished in 0.11s

cargo test --lib h5
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 559 filtered out; finished in 0.00s

cargo test --lib product_command_store_summary_keeps_legacy_and_runner_blocked
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 566 filtered out; finished in 0.00s
```

### `cargo test --lib`

```text
running 567 tests
...
test workbench_sqlite_observation_period::tests::sqlite_production_observation_hash_projection_manifest_and_drift_failures_block ... ok

test result: ok. 543 passed; 0 failed; 24 ignored; 0 measured; 0 filtered out; finished in 8.13s
```

### `cargo fmt -- --check`

```text
<no output; exit 0>
```

### `npm run typecheck`

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

### `npm run test:offline-interaction`

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

### `npm run build`

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
dist/assets/index-Bbojue3u.js   1,008.04 kB │ gzip: 276.29 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 834ms
```

### `node scripts/harness/workbench-shape-gate.js --mode check`

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
Info: 9
Git HEAD: 5ea7c48c4f5179ad7485fd18cf820683ec3d7104
...
- Tauri commands: 104 total; 0 in lib.rs
...
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":104,"baseline":97}
```

### `git diff --check`

```text
<no output; exit 0>
```

### Old-Gate 5-File Diff

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs
<no output; exit 0>
```

### Boundary Scans

```text
rg -n -F "Command::new" prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
777:    let mut command = Command::new(&command_plan.program);
1183:    let mut command = Command::new(&command_plan.program);
2144:        let init_status = std::process::Command::new("git")

rg -n -F "codex exec" prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
<no output; exit 1>

rg -n -F "/Users/yoyi/.codex" prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
1491:            "read /Users/yoyi/.codex/auth.json",
```

Classification:

- `Command::new` at 777/1183 are existing unified process spawn sites used by placeholder/mock/real-gated process configs.
- `Command::new("git")` at 2144 is inside the test module for temp fixture `git init`; it is not a Codex execution path.
- `.codex` hit is an existing denied prompt sample in `manual_relay_blocks_sensitive_or_codex_home_requests`.

## Review

Independent review file:

- `evidence/2026-06-18-codex-relay-real-relay-runner-entry-b1-review-einstein-v1.md`

Review line: Newton writing as `B1 Real Relay Runner Entry Review - Einstein`.

Result: `STATUS: CLEAR_WITH_NOTE`; no P0/P1/P2/P3 findings.

Note: the reviewer did not read frontend/Tauri command registry files because they were outside its allowed read scope. Main-line evidence covers that gap: no frontend/command files changed, shape gate passed, and Tauri command total warning is an existing ratchet warning rather than a package delta.
