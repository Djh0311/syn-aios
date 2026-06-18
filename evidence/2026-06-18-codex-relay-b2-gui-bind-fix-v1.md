# Codex Relay B2 GUI Bind Fix Evidence v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-b2-gui-bind-fix-v1.md`
Base HEAD: `157738c`
Status: implementation ready for consulting-line review; not committed.

## Scope

Fixed the B2 GUI bind bug where opening a Codex conversation could remain unbound because relay target state depended on stale project selection.

- `AgentConversationShell.tsx`: added `deriveRelayBindingState(selectedSession)` and made GUI direct relay derive target project from the selected session's `project_root`.
- `AgentConversationShell.tsx`: removed conversation-mode filtering of visible/session options by `selectedProjectRoot`, so cross-project Codex sessions are not hidden by the old project dropdown.
- `AgentConversationShell.tsx`: direct-send payload now uses one source of truth, `relayTargetProjectRoot`, for `target_project_root`, `target_cwd`, and `allowed_write_roots`.
- `offlineConversationEngineScenario.tsx`: added regression coverage for Codex selected-session binding, stale project root avoidance, missing `project_root` blocking, and cross-project Codex sessions remaining visible in the conversation dropdown.

## Root Cause

The pre-fix UI mixed two pieces of state:

- user-selected project dropdown state (`selectedProjectRoot`);
- the actual clicked conversation (`selectedSession`).

When those diverged, `relayDirectSendEnabled` could stay false or the target Codex conversation could be hidden by project filtering. The fix makes the clicked session authoritative for relay binding. Missing `project_root` is still blocked explicitly; the UI does not guess a target path.

## Boundaries Held

- No true Codex relay was run in this package.
- No `MANUAL_RELAY_REAL_CODEX_CONFIRM` or true-run env authorization was set.
- No ignored true-run tests were executed.
- No `.codex` read/write path was touched by this package.
- No backend relay safety gate was changed.
- Old gate files have empty diff:
  - `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- No product global read/write-path switch, auto-chain, K3-B1/K3-B2 unlock, multi-agent unlock, or role/task-package injection was added.

## Tauri App Verification

Port `5173` was already occupied by the existing Vite server:

```text
COMMAND   PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
node    28004 yoyi   12u  IPv4 0xf63b0fa11cbde7f6      0t0  TCP 127.0.0.1:5173 (LISTEN)
```

So the Tauri desktop program was launched directly while reusing the existing dev server:

```text
/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev --config '{"build":{"beforeDevCommand":""}}' --no-dev-server-wait
```

Launch tail:

```text
Running DevCommand (`cargo  run --no-default-features --color always --`)
Info Watching /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri for changes...
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.37s
Running `target/debug/codex-governance-workbench`
```

Screenshot evidence:

- Before/current bug evidence: `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-artifacts/tauri-before-bind-fix.png`
  - SHA-256: `b130451eb8772448955a72a149e9496eeda4f240a1c2a627407a50265eeda977`
- Tauri direct launch after fix: `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-artifacts/tauri-after-bind-fix.png`
  - SHA-256: `ce0a52100898b74571e5be16ce3993624fbc210c5e744661cf7a4ffbfa7d74d7`

Important limitation: macOS denied Accessibility control for automated clicking:

```text
execution error: "System Events"遇到一个错误："osascript"不允许辅助访问。 (-25211)
```

Therefore the after screenshot proves the Tauri desktop app was launched directly, but it remains on the current selected non-bound state. The actual binding behavior is pinned by regression tests and code review; final sensory confirmation should be the user manually selecting a Codex conversation in the desktop app.

## Verification Output

### npm run test:offline-interaction

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

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
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 259 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                     0.59 kB │ gzip:   0.42 kB
dist/assets/index-4h8GP3AC.css    146.46 kB │ gzip:  24.98 kB
dist/assets/index-CbrzdqoQ.js   1,006.87 kB │ gzip: 276.14 kB

(!) Some chunks are larger than 500 kB after minification.
✓ built in 908ms
```

### cargo test --lib manual_relay

```text
running 18 tests
test manual_relay::tests::manual_relay_b1_real_codex_runner_entry_requires_user_present_env ... ignored, B1 first true Codex relay requires user-present env authorization; do not run in implementation package
test manual_relay::tests::manual_relay_real_codex_requires_env_authorization ... ignored, real Codex relay is a separate user-present window; this gate proves env authorization is required
test manual_relay::tests::manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass ... ok
...
test result: ok. 16 passed; 0 failed; 2 ignored; 0 measured; 550 filtered out; finished in 0.22s
```

### Old Gate Focused Tests

```text
cargo test --lib codex_local_runner
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 556 filtered out; finished in 0.00s

cargo test --lib session_continuation_store
test result: ok. 16 passed; 0 failed; 4 ignored; 0 measured; 548 filtered out; finished in 0.30s

cargo test --lib real_execution_command
test result: ok. 36 passed; 0 failed; 7 ignored; 0 measured; 525 filtered out; finished in 0.37s

cargo test --lib k3_b1_recovery
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 563 filtered out; finished in 0.00s

cargo test --lib h5_project_dispatch_bridge
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 564 filtered out; finished in 0.00s
```

### cargo test --lib

```text
running 568 tests
test manual_relay::tests::manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass ... ok
...
test result: ok. 544 passed; 0 failed; 24 ignored; 0 measured; 0 filtered out; finished in 7.84s
```

### cargo fmt -- --check

```text
<no output; exit 0>
```

### git diff --check

```text
<no output; exit 0>
```

### node scripts/harness/workbench-shape-gate.js --mode check

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 1
Info: 9
Git HEAD: 157738c2d08d6fe18ef41c9ab0899de89d627d62
Findings:
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":105,"baseline":97}
```

## Independent Review

Review line: Erdos.

Review file: `evidence/2026-06-18-codex-relay-b2-gui-bind-fix-review-erdos-v1.md`

Status: CLEAR.

P0/P1/P2: none.

Review conclusion:

```text
STATUS: CLEAR
P0/P1/P2: None
```

## Do Not Claim

- Do not claim the Send button was clicked in this fix package.
- Do not claim a true Codex relay was run.
- Do not claim old K3/H2/H5 gates were opened.
- Do not claim auto-chain, K3-B1/K3-B2 unlock, multi-agent execution, role injection, or product global read/write path switching was enabled.
