# Root Treatment / R3-A8 Copied Snapshot Temp DB Apply Handoff v1

STATUS: DONE

## CHANGED_FILES

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a8/**`
- `evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md`

## LEVEL A SUMMARY

Implemented fixture-only copied snapshot rehearsal helper:

- preflight source fixture snapshot;
- copy source fixture snapshot to temp copy root;
- preflight copied root;
- apply copied root into temp DB;
- export temp DB into temp JSON projection;
- verify canonical runtime logs;
- write dry-run rollback boundary;
- write completed report only on stable success.

`lib.rs` only adds one module declaration.

## LEVEL B STATUS

NOT EXECUTED.

No real workbench state root was read. No real production snapshot was copied. No production root, production DB, read-cut, stop-write, or rollback production workflow was touched.

## COPIED SNAPSHOT MANIFEST SUMMARY

The helper emits a non-sidecar copy manifest named `copied-snapshot-manifest.json` inside the temp copy root. It records path ref/hash, file hash, size, schema_version, and revision for copied files. The name intentionally avoids `*.v1.json` so shape gate does not classify it as a new sidecar kind.

## TEMP DB APPLY SUMMARY

Temp DB apply uses `apply_fixture_dir_to_temp_db` on the copied snapshot root. DB row counts and apply summaries are recorded from the temp DB only. Source root and copied root hashes are compared before apply.

## EXPORT VERIFICATION SUMMARY

Export dry-run uses `export_temp_db_to_json_dry_run`. The helper requires canonical `runtime-logs.v1.json` and rejects legacy singular `runtime-log.v1.json` output or presence.

## ROLLBACK BOUNDARY SUMMARY

Rollback boundary is dry-run only:

- would disable DB read-cut;
- would use copied snapshot / exported JSON projection;
- would preserve temp DB for audit;
- would require supervisor decision;
- `production_restore_performed=false`.

## FAILURE INJECTION SUMMARY

Covered:

- source preflight blocked;
- copy destination inside source root;
- report path inside source root;
- temp DB path inside source root;
- copy interrupted before manifest;
- apply rejected / corrupt snapshot;
- export hash mismatch;
- rollback manifest missing;
- rollback manifest incomplete;
- cleanup failure leaves only temp artifacts and source hash unchanged;
- default denied markers cannot be removed by empty custom config.

## CHECKS RUN

- `node scripts/harness/workbench-shape-gate.js --mode check`: pass; 0 errors / 0 warnings.
- `cargo test --lib sqlite_schema`: pass; 3 passed.
- `cargo test --lib sqlite_snapshot`: pass; 13 passed.
- `cargo test --lib sqlite_preflight`: pass; 8 passed.
- `cargo test --lib sqlite_apply`: pass; 6 passed.
- `cargo test --lib sqlite_export`: pass; 3 passed.
- `cargo test --lib sqlite_observation`: pass; 15 passed.
- `cargo test --lib workflow_state`: pass; 11 passed.
- `cargo test --lib`: pass; 412 passed / 16 ignored.
- `cargo fmt -- --check`: pass.
- `git diff --check`: pass.
- `git status --short`: run; expected R3-A8 files only before evidence/handoff.
- final flag scan: no forbidden `=true` flag pattern hits.
- final sensitive scan: hits are helper denied-marker constants and evidence/handoff boundary text only.

Cargo warning observed: existing unused `JsonRpcError::invalid_params`; not introduced by R3-A8.

## P0 / P1 / P2

P0: none.

P1: none.

P2: none new.

## BOUNDARY CONFIRMATION

Confirmed no Level B, no real workbench state root read, no production DB, no production root write, no product read/write path cutover, no JSON stop-write, no Tauri command/UI/startup hook, no real Codex execution, no prompt send, no `/Users/yoyi/.codex` read/write, and no secret/token/env/keychain/OAuth/provider credential/full transcript/rollout read.

## REQUESTS

Supervisor should run fresh verify, review, and commit if accepted. Do not claim R3 completion or production apply from this result.
