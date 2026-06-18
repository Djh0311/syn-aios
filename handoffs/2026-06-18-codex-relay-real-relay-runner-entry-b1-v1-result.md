# Codex Relay Real Relay Runner Entry B1 Result

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md`
Baseline HEAD: `5ea7c48`
Commit status: not staged, not committed.

## Result

B1 runner-entry implementation is in place for the manual relay path, but no true Codex run was performed.

The code now has:

- a B1 ignored real-runner entry that requires `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`;
- temp fixture defaults for the first user-present run: temp project, `git init`, new session, `workspace-write`, allowed root limited to the temp project, and a minimal `hello.txt` prompt;
- mock structural tests for runner-entry completion and stop behavior;
- test-only gating for placeholder process behavior, alongside the existing mock process test-only gate.

## Product-Visible State

This does not make the product UI send to Codex. It only prepares the locked runner entry that a later user-present window may execute.

The next true relay step still needs explicit user presence and env authorization, and should use the temp fixture first. Sending to a real project or wiring a frontend button is out of this package.

## Verification

Fresh local checks passed:

- `cargo test --lib manual_relay`: `15 passed; 2 ignored`.
- Old-gate focused tests: `codex_local_guard`, `h2_real_resume`, `h3_b`, `k3_b`, `h5`, and `product_command_store_summary_keeps_legacy_and_runner_blocked` passed.
- `cargo test --lib`: `543 passed; 24 ignored`.
- `cargo fmt -- --check`: pass.
- `npm run typecheck`: pass.
- `npm run test:offline-interaction`: pass, 15 offline interaction tests plus R4 checks.
- `npm run build`: pass with existing Vite large-chunk warning.
- `node scripts/harness/workbench-shape-gate.js --mode check`: pass, Errors 0, Warnings 1 for existing Tauri command ratchet warning.
- `git diff --check`: pass.
- Old-gate 5-file diff: empty.

See `evidence/2026-06-18-codex-relay-real-relay-runner-entry-b1-v1.md` for raw output tails.

## Boundary

Not done:

- no true Codex run;
- no ignored tests run;
- no `MANUAL_RELAY_REAL_CODEX_CONFIRM` set;
- no frontend/Tauri command wiring;
- no old-gate file changes;
- no product claim that stop is clickable in UI;
- no `.codex` auth/session/transcript reads or writes.

## Review / Handoff

Independent review:

- Review line: Newton writing as `B1 Real Relay Runner Entry Review - Einstein`
- File: `evidence/2026-06-18-codex-relay-real-relay-runner-entry-b1-review-einstein-v1.md`
- Status: `CLEAR_WITH_NOTE`
- Findings: none for P0/P1/P2/P3

The note is that the review line did not read frontend/Tauri command registry files. Main-line evidence covers that: no frontend/command files changed, shape gate passed, and the Tauri command warning is an existing ratchet warning.

Hand back to consulting line for final scan before any `git add` / `git commit`.
