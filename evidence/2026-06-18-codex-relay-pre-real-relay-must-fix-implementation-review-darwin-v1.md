# Codex Relay Pre-Real Relay Must-Fix Review - Darwin v1

Date: 2026-06-18

Review line: Darwin

Agent id: `019ed72a-4661-7502-988c-c57dedc60f32`

STATUS: CLEAR_WITH_NOTE

## Findings

- No P0 / P1 / P2 blocking findings remain after the P1/P2 repair pass.

## P1 / P2 Resolution Check

- P1 fixed: running attempt registration now uses `register_running_attempt_once` to check duplicate scope, reserve confirmation, and insert the registry entry in the protected path. The placeholder branch checks duplicate scope and reserves confirmation before spawning `/bin/sleep`, preventing different-confirmation same-scope concurrent double start and avoiding spawn-before-duplicate orphan risk.
- P2 fixed: `AgentChatComposer` guards form submit, Enter key handling, textarea onChange, and textarea state with `relayInputLocked`; the textarea becomes `readOnly` / `aria-busy` while running. `AgentConversationShell.handleSubmitConversationDraft` also has a running/busy guard.

## Boundary Check

- Old gate 5-file diff is empty.
- `Command::new` remains limited to the manual relay placeholder spawn path.
- `real_codex_env_gated` still returns `manual_relay_real_codex_env_gated_not_enabled_in_this_package`.
- `.codex`, secret, token, and `.env` hits in changed product files are deny-list / blocking-test / fixture-only classifications.
- `git diff --check` was clean during review.

## Evidence Reviewed

- Read-only diff and line review for `manual_relay.rs`, `manualRelay.ts`, `AgentChatComposer.tsx`, `AgentConversationShell.tsx`, and `offlineConversationEngineScenario.tsx`.
- Main-agent verification summary after fixes:
  - `cargo test --lib manual_relay`: 10 passed / 1 ignored.
  - `cargo test --lib`: 538 passed / 23 ignored.
  - `npm run test:offline-interaction`: 15 passed plus R4 checks.
  - `npm run typecheck`, `npm run build`, focused old-gate tests, `cargo fmt -- --check`, shape gate, and `git diff --check` passed.

## Notes

- Darwin did not independently rerun the complete test suite; this review relied on the main line's fresh verification output plus read-only diff / grep / line review.
- True browser interaction verification remains a residual gap. Offline/static tests cover `readOnly` and button states, but a live browser keyboard/click path could not be proven in this execution window.

