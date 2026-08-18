# M5R08 narrow implementation package: M1 ordinary source snapshot safety

## Authority and boundary

- Authority: current `M5R08-m1-consumption-runtime-idempotency-and-acceptance-debts` leaf only.
- Implementer: Grok. Do not commit.
- Preserve all existing working-copy changes. This allowed file is clean at package start.
- Do not change registry semantics, ordinary Tauri bootstrap ordering, ProjectId allocation, alias policy, frontend, runtime identity, harness lifecycle, or any file other than the one listed below.

## Only allowed product file

`prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`

Opening SHA-256: `19ae9a9effc0e25d8603a3bcbbeaef27541094573529e6d213eb6183a96a76bb`

## Problem to close

`load_ordinary_identity_source` currently checks `symlink_metadata(path)` and later calls `fs::read(path)`. An attacker or concurrent replacement can swap the pathname between validation and read, so the validated object and consumed bytes are not demonstrably the same object.

## Required behavior

1. Open the ordinary identity source in a way that rejects a symlink at open time (Linux/Unix is the production target here), then validate regular-file metadata and read bytes through that exact opened handle. Do not validate a pathname object and reopen it for the read.
2. Preserve the existing stable public error families for missing, unreadable, malformed, and unsupported source input. A symlink/non-regular source must fail closed before any registry/marker write.
3. The parsed document and replay must consume only bytes read from the validated opened handle. A path replacement after the handle is opened must not switch the consumed document to the replacement pathname target.
4. Keep ordinary app-data-root checks and all registry locking/replay rules unchanged.
5. Add direct `m5r08_m1_source_` tests proving:
   - a source symlink fails closed and writes neither registry nor established marker;
   - a source pathname replaced after the validated handle is opened cannot change the bytes/document consumed from that handle (use a minimal test-only seam if needed, without weakening production);
   - valid ordinary source replay remains successful;
   - a static production-source guard demonstrates there is no `symlink_metadata(path)` then `fs::read(path)` reopen sequence in `load_ordinary_identity_source`.
6. Avoid unrelated formatting and do not broaden this change to the registry-file loader.

## Verification before handoff

From `prototypes/productized-desktop-shell/src-tauri`:

```bash
cargo test --lib m5r08_m1_source_ --offline -- --test-threads=1
cargo test --lib m1_ordinary_identity_source_ --offline -- --test-threads=1
cargo check --lib --offline
```

Report changed files, exact commands and exits, and any remaining limitation. Do not commit.
