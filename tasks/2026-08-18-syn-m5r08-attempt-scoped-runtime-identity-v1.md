# M5R08 narrow implementation package: attempt-scoped runtime identity

## Authority and boundary

- Authority: current `M5R08-m1-consumption-runtime-idempotency-and-acceptance-debts` leaf only.
- Implementer: Grok. Do not commit.
- This package must not activate M6, change frontend layout, touch harness lifecycle, or alter any file outside the four listed below.
- Preserve all pre-existing working-copy changes. At package start these four files are clean.

## Allowed product files

1. `prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs`
2. `prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs`
3. `prototypes/productized-desktop-shell/src-tauri/src/m5_agent_runtime.rs`
4. `prototypes/productized-desktop-shell/src-tauri/src/m5_runner_entry_registry.rs` (tests/static guard only)

Opening SHA-256 values:

- `m5_product_commands.rs`: `b7868f0ecf2ae7c52cb4ae372b2f83f59d9f344873a10a160e4005d2443209a3`
- `m5_controlled_execution.rs`: `a820c1c2aa40a537b244d7f90d8b86e8ecddeb24572c1a8d8de3bf1adb3fc2e3`
- `m5_agent_runtime.rs`: `e79067c6fca63f5fc0f4b80cf41ffe957ac5cdb706f55aacbd1d90ec7ad040a9`
- `m5_runner_entry_registry.rs`: `893566a2e12e3aeca9bd5b0c32260338555718ffda65ffc9b0cf69297cca3eea`

## Required product behavior

1. Remove project-scoped runtime carrier identity from the ordinary authorized runtime path. In particular, `run_m5_authorized_runtime_with_state` must not build `workcell_id` from only `binding.project_id`.
2. Build the workcell identity from the already-admitted, server-owned attempt and grant lineage. The resulting workcell identity must be stable for an exact attempt/grant replay and distinct for two legal attempts of the same project.
3. The durable `operation_id` and runtime `receipt_id` must inherit the attempt/grant-scoped identity. Keep their existing semantic prefixes if practical, but do not let either collapse back to project scope.
4. Persistent duplicate-effect safety must not rely only on the in-memory runtime adapter. Re-entering the same admitted attempt/effect must fail closed or return a previously authoritative result without executing a second effect. It must not overwrite an existing immutable operation row with a fresh run.
5. A legal retry/new attempt for the same project must retain the old attempt/grant/dispatch/operation/receipt lineage and create a distinct new attempt/grant/dispatch/workcell/operation/receipt/effect lineage.
6. Do not weaken the existing exact grant/dispatch/attempt/effect joins, authorization checks, outcome-unknown reconciliation, or control-receipt idempotency.
7. Add direct tests with the `m5r08_runtime_` prefix proving:
   - two legal attempts for one project have distinct workcell, operation, receipt, and effect identities;
   - the old lineage remains queryable and unchanged after the later attempt;
   - an exact duplicate of one attempt cannot execute/persist a second effect;
   - normal authorized runtime still succeeds once;
   - a static guard rejects the former `wc-{project_id}` construction and confirms the carrier chain is attempt/grant scoped.

## Verification to run before handoff

From `prototypes/productized-desktop-shell/src-tauri`:

```bash
cargo test --lib m5r08_runtime_ --offline -- --test-threads=1
cargo test --lib m5_controlled_execution --offline -- --test-threads=1
cargo check --lib --offline
```

Report changed files, exact commands and exits, and any remaining limitation. Do not commit and do not touch unrelated formatting.
