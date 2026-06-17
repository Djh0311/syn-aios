# Codex Relay Pre-Real Relay Must-Fix Implementation Result v1

Date: 2026-06-18

Task: `tasks/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`

Status: implementation complete pending consulting-line review; not committed.

Independent review: Darwin (`019ed72a-4661-7502-988c-c57dedc60f32`) final `STATUS: CLEAR_WITH_NOTE`. Initial P1/P2 findings were fixed before this handoff.

## Product-Visible Result

Manual relay now has the three guardrails needed before a separate user-present true relay window:

- Process-mode run requires verified canonical paths and rejects stale or unverified path bindings.
- A confirmation is one-shot at the backend; concurrent run attempts with the same confirmation only allow one winner.
- The UI clears the draft on relay run trigger, disables send / preview / confirm while an attempt is running, and keeps Stop available for that attempt.
- Stop can kill and reap a harmless registered placeholder process; the receipt records process id / kind / kill status without claiming real Codex execution.

## Not Done / Not Claimed

- No real Codex relay was run.
- No `.codex` state, auth, token, transcript body, rollout body, or prompt body was read or written by the product implementation.
- No K3-B1 / K3-B2 / H2 / H3 / H5 / product-command old gate was weakened.
- No new Tauri command was added.
- No automatic retry, auto rollback, auto chaining, or real stop of a Codex process was implemented.

## Evidence

Primary evidence: `evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`

Fresh verification:

- `cargo test --lib manual_relay`: 10 passed / 1 ignored.
- `cargo test --lib`: 538 passed / 23 ignored.
- Old-gate focused tests: `codex_local_guard`, `h2_real_resume`, `h3_b`, `k3_b`, `h5`, and `product_command_store_summary_keeps_legacy_and_runner_blocked` passed.
- `npm run typecheck`, `npm run test:offline-interaction`, `npm run build` passed.
- `cargo fmt -- --check`, `node scripts/harness/workbench-shape-gate.js --mode check`, `git diff --check` passed.
- Old-gate 5-file diff is empty.
- Review file: `evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-review-darwin-v1.md`.
- Continuation recheck also passed `checkpoint-audit --package 2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1 --review evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-review-darwin-v1.md --allow-dirty --skip-gates` with `VERDICT: PASS`. Dirty tree is expected because this handoff stops before commit.

## Residuals

- True browser verification was attempted but not completed: temporary local dev server binding was denied by sandbox/policy, and the already-running 5173 page was in a startup-failed state unrelated to this package. Offline render tests cover the changed control state, but a working local app/browser pass remains desirable before consulting-line acceptance.
- Process note: the browser plugin skill file under `/Users/yoyi/.codex/plugins/.../SKILL.md` was read while attempting UI verification. No `.codex` secrets/state/transcript/prompt bodies were read or written. Recorded as a process mistake.
