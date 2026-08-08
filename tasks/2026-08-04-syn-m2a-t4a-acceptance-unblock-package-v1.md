# SYN M2a · T4-A 派发包：解除 T1-R2 验收阻塞

date: 2026-08-04
status: ACTIVE — user-approved sequencing exception
dispatcher: 总指导线
executor: existing OpenCode session

---

## 0. Why this runs before T2

T1-R2's production wiring and isolated runtime evidence are physically verified for A1-A5. Its A6 certification is not reproducible:

- director standard run: `cargo check --lib` exit 0 / **693 warnings**; the delivery record says 694;
- director standard `cargo test --lib`: **1341 passed / 2 failed / 45 ignored**;
- `codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child` passes alone outside the sandbox but fails in a normal full parallel run because `mock-child.pid` is absent at its assertion;
- `workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report` stably fails: it expects `preflight_not_ready`, but the production-apply rehearsal returns `completed` and creates a DB.

The latter is explicitly T4 item ② and the process-fixture family is T4 item ③ in `tasks/2026-08-03-syn-m2a-kickoff-v1.md`. User approved this narrow T4-A exception before T1 is closed. T2 remains blocked.

Worktree: `/Users/yoyi/workspace/product-line-syn-fnd-002`, branch `syn-fnd-002-dev`. Do not switch branch, merge, push, reset, clean, stash, or bulk-stage.

## 1. Scope: exactly two blockers

### A. Restore the preflight deny contract

Fix the real mismatch covered by
`workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report`.

- The `production-preflight-blocked-denied-path` fixture must still cause preflight rejection before DB/report creation.
- Keep the assertion semantic: result contains `preflight_not_ready`, the DB does not exist, and the report does not exist.
- Diagnose whether the defect is in fixture input, preflight classification, or production-apply routing; make the smallest correct fix and add a focused regression test only if the existing test does not fully cover the repaired branch.
- Do not change the test to expect success, delete the assertion, mark it ignored, or accept a DB/report and call it a preflight block.

### B. Make the real-process timeout fixture deterministic

Fix the full-suite instability in
`codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child`.

- Preserve what the test proves: a real mock child reaches the timeout path, is reaped, produces no retained stale message, and provides a PID for reaping verification.
- The normal command `cargo test --lib` must pass this test without `--test-threads=1`, retry wrappers, skips, ignore attributes, or timing-only acceptance scripts.
- Prefer a test-only deterministic child-readiness/handshake boundary. Do not lengthen or weaken product timeout semantics to satisfy the fixture.
- Retain a focused test command that can be run alone, but full-suite green is the acceptance proof.

## 2. Out of scope

- No T2 crash-recovery acceptance.
- No M2 UoW/reference-slice rewiring and no edits to the T1-R2 functional behavior unless a compiler requirement from the two fixes makes it unavoidable; report any such need before doing it.
- No grant-store work, code-map cleanup, external provider, real user store, App launch, migration, or production read/write cutover.
- No warning cleanup. The current verified `cargo check --lib` figure is 693 warnings; report any change as a diff, do not tune the count.

## 3. Required verification and evidence

Run from `prototypes/productized-desktop-shell/src-tauri` in the ordinary build profile:

```text
cargo test --lib workbench_sqlite_production_apply::tests::sqlite_production_preflight_blocked_creates_no_db_or_report -- --exact --nocapture
cargo test --lib codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child -- --exact --nocapture
cargo check --lib
cargo test --lib
git diff --check
```

The standard full run is the gate. Its target is `1343 passed / 0 failed / 45 ignored` at the current test inventory. If count changes, enumerate the changed test(s) and reason; do not report a guessed baseline.

Create `test-fixtures/m2a-acceptance/t4a-acceptance-record-2026-08-04.md` with:

- exact commands, exit status, final test summaries, and the `cargo check` warning count;
- the preflight denial fact (no DB and no report) and the fixture/source path read;
- the real-process fixture's deterministic readiness/reaping proof;
- an explicit statement that no real HOME store or App was touched.

Evidence labels: focused tests = `UNIT`; complete `cargo test --lib` = `UNIT`; no claim may exceed that level.

## 4. Delivery rules

1. Start by reading this package, the M2a kickoff T4 paragraph, and the failing source/tests. Do not guess from test names.
2. Keep the diff restricted to the two described failure families plus their evidence and `docs/harness/CURRENT.md` status update.
3. Before commit, list the exact files, run verification, and inspect `git diff --check`.
4. Commit with an explicit file list and a message containing `catch:`. Do not use `git add -A`.
5. After commit, run separate commands for `git status --short`, `git log -1 --oneline`, `git rev-parse HEAD^{tree}`, and `git write-tree`; report whether the two tree hashes match.
6. Report facts separately from inference. Do not claim T1 or M2 complete. T1 acceptance and T2 dispatch remain director actions.

## 5. Acceptance

This task is accepted only when both named tests and a standard full `cargo test --lib` are green, `cargo check --lib` is recorded with its actual warning count, the evidence record exists, and no out-of-scope behavior changed. The director then re-runs T1-R2 A6 and decides T1 acceptance.
