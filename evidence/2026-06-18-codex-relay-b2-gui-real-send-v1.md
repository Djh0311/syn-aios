# Codex Relay B2 GUI Direct Send Evidence v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-b2-gui-real-send-v1.md`
Base HEAD: `9b7360a`
Status: implementation ready for consulting-line review; not committed.

## Scope

Implemented the B2 GUI direct-send chain for an already-bound Codex conversation:

- Backend: added `ManualRelayGuiDirectRunInput` and `run_manual_relay_gui_direct_once`; added Tauri command `run_manual_codex_relay_gui_direct`.
- Frontend: bound Codex conversation composer now uses `data-send-mode="manual_relay_direct"`; Enter / Send calls the GUI direct relay handler only when a project and Codex session are bound.
- UI: target strip is always visible in the composer and shows project path, bound session title, exact `thread_id`, `manual_once`, `auto_chain=false`, and `sandbox=workspace-write`.
- Tests: added backend GUI-direct regression test and offline interaction assertions for target visibility, exact session id visibility, Enter direct-send handler, unbound no-send behavior, non-Codex blocked behavior, receipt / Stop visibility.

## Boundaries Held

- No true Codex relay was run during this implementation package.
- No `MANUAL_RELAY_REAL_CODEX_CONFIRM` was set.
- No ignored true-run tests were executed.
- No `.codex` writes were made by this package.
- Old K3/H2/H5 gate files were not changed:
  - `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- No product global read/write-path switch, auto-chain, multi-agent unlock, K3-B1/K3-B2 unlock, or role/task-package injection was added.

## Command Surface

Shape-gate reports Tauri commands: `105 total`, baseline `97`, warning only.

This package adds exactly one command:

```text
run_manual_codex_relay_gui_direct
```

The task-package impact is intentional: B2 needs a product GUI entrypoint that bypasses the old B1 env-gated test runner while keeping confirmed target + command-plan validation.

## Safety Checks

GUI direct validation requires:

- bound existing `target_session_id`;
- `target_cwd_canonical == project_root_canonical`;
- `sandbox == "workspace-write"`;
- `allowed_write_roots == [project_root_canonical]`;
- command plan program is `codex`;
- prompt is passed by stdin, not command argv;
- no shell invocation;
- argv contains `--sandbox workspace-write`;
- argv contains `--add-dir <project_root_canonical>`;
- argv rejects approval-bypass args: `--full-auto`, `dangerously-bypass`, `--approval*`, `full-auto`.

Safety scan of changed diff:

```text
MANUAL_RELAY_REAL_CODEX_CONFIRM: only std::env::remove_var in GUI direct test
--full-auto / dangerously-bypass: only forbidden-arg validation and test assertions
.codex: only removed legacy UI text saying "不写 .codex"; no new .codex read/write path
Command::new: no new changed-diff hit
Old gate 5-file diff: empty
```

## GUI Verification

Real Tauri desktop program was launched directly, not just a browser/Vite page:

```text
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev --config '{"build":{"beforeDevCommand":""}}' --no-dev-server-wait
```

Because an existing Vite server already occupied port `5173`, the dev shell was launched with `beforeDevCommand` disabled and later the Vite server was restarted with:

```text
env VITE_STAGE_K_INITIAL_VIEW=agents npm run dev
```

No Send button was clicked. No relay command was invoked.

Screenshot evidence:

- `evidence/2026-06-18-codex-relay-b2-gui-real-send-artifacts/tauri-app-real-launch-post-p2.png`
- SHA-256: `357a997d36c031176c0df5c89ed5d9bf35fb139b3b64dbf4bbece3a6e6ffa456`

Observed in screenshot:

- Tauri app title: `Codex 治理工作台 · 首屏已挂载`.
- Page: `智能体`.
- Header: `GUI direct relay 已绑定`.
- Visible target strip: `↔ GUI direct relay target 项目：/Users/yoyi/workspace 会话：全局主管新 会话ID：<exact thread_id> 策略：手动一次一发 · auto_chain=false · sandbox=workspace-write`.
- Composer copy: `Enter 将直接发送给：全局主管新`.
- Manual relay panel and `Stop 本 attempt` area visible.

## Independent Review and P2 Fixes

Review line: Erdos.

Initial result: `CLEAR_WITH_P2`.

- P2 fixed: target strip now renders `会话ID：<thread_id>` as a separate always-visible field in `AgentChatComposer.tsx`.
- P2 fixed: GUI direct-send enablement now requires the selected session software key to be Codex in `AgentConversationShell.tsx`; non-Codex sessions show `仅 Codex 会话可用` and Enter does not invoke the direct-send handler.
- Added offline assertions for exact session id visibility and non-Codex blocked behavior.

Post-fix review result is recorded in `evidence/2026-06-18-codex-relay-b2-gui-real-send-review-erdos-v1.md`.

## Verification Output

### cargo test --lib manual_relay

```text
running 18 tests
test manual_relay::tests::manual_relay_b1_real_codex_runner_entry_requires_user_present_env ... ignored, B1 first true Codex relay requires user-present env authorization; do not run in implementation package
test manual_relay::tests::manual_relay_real_codex_requires_env_authorization ... ignored, real Codex relay is a separate user-present window; this gate proves env authorization is required
test manual_relay::tests::manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass ... ok
...
test result: ok. 16 passed; 0 failed; 2 ignored; 0 measured; 550 filtered out; finished in 0.42s
```

### npm run test:offline-interaction

```text
offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

### npm run typecheck

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

### npm run build

```text
vite v7.3.3 building client environment for production...
✓ 259 modules transformed.
dist/index.html                     0.59 kB │ gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB │ gzip:  24.98 kB
dist/assets/index-B4PGnRxI.js   1,006.48 kB │ gzip: 276.01 kB
(!) Some chunks are larger than 500 kB after minification.
✓ built in 1.78s
```

### cargo test --lib

```text
running 568 tests
test manual_relay::tests::manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass ... ok
...
test result: ok. 544 passed; 0 failed; 24 ignored; 0 measured; 0 filtered out; finished in 10.36s
```

### Old Gate Focused Tests

```text
cargo test --lib codex_local_runner
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 556 filtered out; finished in 0.00s

cargo test --lib session_continuation_store
test result: ok. 16 passed; 0 failed; 4 ignored; 0 measured; 548 filtered out; finished in 0.36s

cargo test --lib real_execution_command
test result: ok. 36 passed; 0 failed; 7 ignored; 0 measured; 525 filtered out; finished in 0.39s

cargo test --lib k3_b1_recovery
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 563 filtered out; finished in 0.00s

cargo test --lib h5_project_dispatch_bridge
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 564 filtered out; finished in 0.01s
```

### cargo fmt -- --check

```text
<no output; exit 0>
```

### node scripts/harness/workbench-shape-gate.js --mode check

```text
Status: pass
Errors: 0
Warnings: 1
Tauri commands: 105 total; 0 in lib.rs
Findings:
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":105,"baseline":97}
```

### git diff --check

```text
<no output; exit 0>
```

## TDD Note

The implementation was already partially completed before this recovery turn. Prior run summary says red tests failed as expected for missing GUI-direct backend type and missing direct-send UI mode, then passed after implementation. The raw red output was not persisted in a file, so this evidence package does not treat red-output text as final proof. The preserved proof is the added regression tests plus fresh green verification above.

## Residual Risk

- First real GUI relay was not run in this package. It remains a separate user-present window and should start with a temp / controlled target.
- Browser DOM automation was not used because Chrome/Safari both had Apple Events JavaScript disabled. The Tauri screenshot is real GUI evidence; interaction behavior is covered by offline interaction tests.
- Build still has an existing Vite large chunk warning; not introduced or solved by this package.
