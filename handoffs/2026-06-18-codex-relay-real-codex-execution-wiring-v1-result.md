# Codex Relay Real Codex Execution Wiring Result v1

Date: 2026-06-18

Task: `tasks/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`

Status: implementation complete after P1 fix; Dirac rereview `STATUS: CLEAR`; not committed.

## Product-Visible Result

Manual relay ③a now has a wired but locked real-Codex process path:

- Without `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`, `real_codex_env_gated` rejects before registering or spawning a process.
- With authorization, the real-Codex process config is `process_kind=real_codex`, `program=codex`, structured argv, stdin prompt, and a workbench-managed last-message path.
- Mock-codex scripts prove the shared process helper can send stdin, read back last-message hash/size/status, and kill/reap a running process through `stop`.
- After Dirac found P1, mock executable process modes are test-only: production cfg rejects them with `manual_relay_mock_codex_process_mode_test_only`.

## Not Done / Not Claimed

- No real Codex relay was run.
- No ignored real-Codex test was run.
- No `.codex` file was read or written.
- No browser verification was attempted.
- No K3-B1 / K3-B2 / H2 / H3 / H5 / PCR old gate was weakened.
- No new Tauri command was added.
- ③b first true real-Codex relay remains a separate user-present authorization window.

## Evidence

Primary evidence: `evidence/2026-06-18-codex-relay-real-codex-execution-wiring-v1.md`

Fresh verification after P1 fix:

- `cargo check --lib`: passed, compiling the non-test production cfg path.
- `cargo test --lib manual_relay`: 13 passed / 1 ignored.
- `cargo test --lib`: 541 passed / 23 ignored.
- Old-gate focused tests passed: `codex_local_guard`, `h2_real_resume`, `h3_b`, `k3_b`, `h5`, `product_command_store_summary_keeps_legacy_and_runner_blocked`.
- `npm run typecheck`, `npm run test:offline-interaction`, `npm run build`: passed.
- `cargo fmt -- --check`, `node scripts/harness/workbench-shape-gate.js --mode check`, `git diff --check`: passed.
- Old-gate 5-file diff is empty.

## Review State

- Initial independent review line: Dirac (`019ed78e-f036-78f0-b576-e602fc87a79f`) returned `STATUS: FINDINGS`.
- Finding: P1 mock process modes were production-reachable.
- Fix: test-only cfg gate plus M-0005 mistake ledger entry.
- Final rereview: Dirac (`019ed78e-f036-78f0-b576-e602fc87a79f`) returned `STATUS: CLEAR` after confirming the documentation P2 was closed.
- Review record: `evidence/2026-06-18-codex-relay-real-codex-execution-wiring-review-dirac-v2.md`.

## Residual / Caveat

The env-gated real-Codex branch is wired as a running process path so `stop` can target it. Terminal last-message readback is proven with the mock-codex completion path using the same helper. A real Codex terminal receipt/readback is not claimed until ③b user-present validation.
