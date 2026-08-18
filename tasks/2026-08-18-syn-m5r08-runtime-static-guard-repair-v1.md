# M5R08 narrow repair package: runtime bypass static guard scope

## Trigger

Candidate `cf0ad36c1b59433089091b541ca7d00efe1a23d2` full matrix command:

```bash
cargo test --lib --offline m5_ -- --test-threads=1
```

returned exit 101: 187 passed / 1 failed. The only failure was `m5_runner_entry_registry::tests::product_workcell_has_no_unregistered_bypass`. Its assertion scans all of `m5_product_commands.rs` for `run_authorized_workcell`, so the new `m5r08_runtime_duplicate_admitted_effect_fails_closed` test-only call at line 3346 is mistaken for a production bypass. Production code begins before the first `#[cfg(test)]` at line 1019 and still has exactly one `run_admitted_workcell` call.

## Authority and only allowed product file

- Authority: current M5R08 leaf and this directly affected regression only.
- Implementer: Grok. Do not commit.
- Only edit `prototypes/productized-desktop-shell/src-tauri/src/m5_runner_entry_registry.rs`.
- Opening SHA-256: `a6142ded98bf7feba95d0e8df46d31d43b32d96aaae975896883063005bbe289`.
- Do not change runtime/product behavior, the new duplicate-effect test, registry inventory, other assertions, or any other file.

## Required repair

Scope `product_workcell_has_no_unregistered_bypass` production-bypass assertions to `production_prefix(product)` (the existing helper) rather than the entire file. Keep the exact formal-runtime one-call assertion. The guard must still fail if production code calls `run_authorized_workcell`, contains `fail_cell`, or contains the forged `-fail` marker, while allowing those strings under `#[cfg(test)]` tests.

## Verification

From `prototypes/productized-desktop-shell/src-tauri`:

```bash
cargo test --lib m5_runner_entry_registry::tests::product_workcell_has_no_unregistered_bypass --offline -- --exact --test-threads=1
cargo test --lib m5_runner_entry_registry::tests --offline -- --test-threads=1
cargo test --lib m5r08_runtime_ --offline -- --test-threads=1
```

Report exact exits and changed path. Do not commit.
