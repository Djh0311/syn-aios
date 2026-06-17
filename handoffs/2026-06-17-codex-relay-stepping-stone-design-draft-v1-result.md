# Handoff: Codex Relay Stepping Stone Design Draft v1 Result

Date: 2026-06-17

Status: design draft complete; waiting for consultation-line review and user ratification.

## What Changed

- Added `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`.
- Added `evidence/2026-06-17-codex-relay-stepping-stone-design-review-wegener-v1.md`.

## Product-Visible Conclusion

The recommended design is a new parallel `manual_relay` narrow path:

- Reuse the conversation engine UI surface for input, target selection, pending/receipt display, and last-message return.
- Reuse or mirror the runner safety building blocks: structured argv, stdin prompt, workbench-managed last-message path, timeout/kill, duplicate guard, canonical target checks, denied-material policy, and readback status.
- Do not repurpose or loosen H2/H3/K3/H5/PCR real execution gates, product-command authorization, K3-B1 recovery, K3-B2 gate, or the existing decision-only composer behavior.

## Boundaries

This result is only a design handoff:

- No product code changed.
- No Tauri command was added.
- No runner was called.
- No real `codex exec` or `codex exec resume` was run.
- No `/Users/yoyi/.codex` content was read or written.
- No K3-B1/K3-B2/real-resume/product-command gate was changed or weakened.
- No automatic relay chain, retry, stop-write, rollback, role injection, task-package injection, memory injection, or product global execution path was enabled.

## Review

Independent read-only review line: Wegener (`019ed5d4-7255-7e21-b540-8c8931cb74fb`)

Review status: STATUS: CLEAR

Findings: none.

Residual risk: implementation still needs its own narrow task package and tests for stop, rollback, receipt, CLI argv shape, duplicate guard, exact-payload visibility, and old-gate non-regression.

## Next Step

Consultation line should red-team the draft. If accepted, the user can ratify the design and only then authorize a separate implementation task package.
