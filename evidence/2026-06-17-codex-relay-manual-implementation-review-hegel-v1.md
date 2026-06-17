# Codex relay manual implementation review Hegel v1

Date: 2026-06-18
Review line: Hegel
Agent id: `019ed696-09cb-78d3-897a-1a60f259a2c5`
Scope: read-only review of current uncommitted `manual_relay` implementation against `tasks/2026-06-17-codex-relay-manual-implementation-v1.md` and `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`.

## Status

STATUS: CLEAR_WITH_P2

## Initial Findings And Fixes

Initial review returned `STATUS: FINDINGS` with no P0 and three P1 findings:

- P1: confirmation id was not consumed once after terminal fixture completion.
- P1: receipt contract was thinner than the design.
- P1: target hash mismatch, confirmation id mismatch, and H3 regression evidence were incomplete.

Those P1 findings were fixed before this review file was written:

- Sequential confirmation replay now blocks after a terminal completed fixture receipt via `consumed_confirmations`.
- Receipt now includes target, prompt hash/length/exactness, command plan, timestamps, exit code, last-message size, changed files, and git before/after fields.
- Tests now cover prompt hash mismatch, target hash mismatch, confirmation id mismatch, duplicate running attempts, confirmation replay, and receipt contract fields.
- Evidence now includes `cargo test --lib h3_b` regression output.

## Final Review Result

P0: none.

P1: none remaining.

P2:

- Path canonicalization uses `std::fs::canonicalize` when possible, then lexical clean fallback for missing fixture/mock paths. Accepted as mock-only P2/deferred; first real relay package must re-audit this before execution.
- Confirmation consumption is tested for sequential replay, not concurrent double-submit. Current check and insert use separate lock windows. Accepted as fixture-only P2; before real relay, reserve/consume should be made atomic.

Notes:

- Allowed write roots are now shown in UI and covered by offline test assertions.
- Old decision-only path remains separate.
- Four Tauri commands are narrow preview/confirm/run/stop wrappers for manual relay.
- No changed old-gate file was identified in the diff.
