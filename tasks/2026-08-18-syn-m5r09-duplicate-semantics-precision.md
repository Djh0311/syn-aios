# M5R09 duplicate semantics precision

## Goal

Close only M5R09 standard 6 by separating the ordinary command dispatch-state counterexample from the durable duplicate-effect counterexample. Preserve the accepted runtime chain and production behavior unless a directly exposed in-scope defect makes a minimal repair necessary.

## Allowed product files

- `prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs`

Do not modify any other file. Do not run git operations. Do not touch M6, Harness files, protected WIP, real data, providers, credentials, network business writes, deployment, or release paths.

## Required changes

1. In `m5_product_commands.rs`, make `m5r08_runtime_authorized_runtime_succeeds_once` (renaming to an M5R09-specific name is allowed) prove the exact dispatch-state gate on the second ordinary command call. Replace its broad multi-code / substring disjunction with the exact current error `dispatch_not_pending_delivery`. Keep the mechanical single durable operation and single distinct effect assertions.
2. In `m5_controlled_execution.rs`, replace or refine `m5r08_runtime_duplicate_attempt_cannot_persist_second_effect` as a direct durable-layer reentry test. From the module-local test, load the stored grant and call private `persist_and_execute_workcell` twice with the same attempt/grant/effect. The second call must return exactly `duplicate_effect` before the fresh runtime adapter executes.
3. Mechanically prove after the direct second call that the same attempt/effect has:
   - exactly one durable operation row;
   - exactly one persisted non-null receipt identity and the original receipt remains unchanged;
   - exactly one distinct effect identity and no second runtime/business-effect event;
   - no mutation of the original operation state, receipt, timestamp, retry count, grant, dispatch, attempt, or effect fields.
4. Do not weaken any existing assertion into a set of acceptable errors. Do not add test-only bypasses to production gates. Do not change unrelated runtime semantics.

## Verification before handoff

Run all of the following and report exact exit codes and test counts:

```bash
cd prototypes/productized-desktop-shell/src-tauri
cargo fmt --check
cargo test --lib --offline m5r09_runtime_dispatch_state_gate_is_exact -- --test-threads=1
cargo test --lib --offline m5r09_runtime_direct_durable_reentry_rejects_duplicate_effect -- --test-threads=1
cargo check --lib --offline
cargo test --lib --offline m5_ -- --test-threads=1
```

Also run `git diff --check` and report the exact modified path list. This is local/offline validation only; do not claim Tauri GUI, real project, provider, deployment, or release evidence.
