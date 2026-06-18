# Codex Relay B2 GUI Direct Send Review - Erdos v1

Date: 2026-06-18
Task: `tasks/2026-06-18-codex-relay-b2-gui-real-send-v1.md`
Review line: Erdos
Agent id: `019ed9ba-3f4c-72a1-b595-6487fceb7b6b`
Status: CLEAR

## Scope

Read-only post-P2 review of the B2 GUI direct-send diff after the initial `CLEAR_WITH_P2` review.

Reviewed:

- Target strip exact session id visibility.
- Frontend Codex-only direct-send predicate.
- Non-Codex blocked coverage.
- Old K3/H2/H5 gates not widened.
- No true Codex relay run, no `MANUAL_RELAY_REAL_CODEX_CONFIRM` set, and no `.codex` write.

## Findings

No P0/P1/P2 findings remain.

## Evidence

- Target strip explicitly shows exact `thread_id`: `AgentChatComposer.tsx` creates `targetSessionId` and renders `会话ID：<thread_id>` in the always-visible target strip.
- Frontend direct-send enablement is Codex-only: `AgentConversationShell.tsx` gates `relayDirectSendEnabled` on `softwareKeyOf(selectedSession) === "codex"`.
- Non-Codex negative coverage exists: `offlineConversationEngineScenario.tsx` constructs a `thread_source: "claude-code"` session, asserts `仅 Codex 会话可用`, and asserts Enter does not invoke the direct relay handler.
- Old gate file diff is empty for:
  - `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`
- Boundary declarations match implementation evidence: the package did not run true Codex relay and did not set `MANUAL_RELAY_REAL_CODEX_CONFIRM`; search results only show env removal in tests or the existing env-gated true-run code path.

## Conclusion

STATUS: CLEAR. The two prior P2 items are fixed, and no remaining review finding blocks consulting-line review.
