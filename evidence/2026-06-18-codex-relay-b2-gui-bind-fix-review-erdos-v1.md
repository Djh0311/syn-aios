# Codex Relay B2 GUI Bind Fix Review - Erdos v1

Date: 2026-06-18
Review line: Erdos
Agent id: `019ed9ba-3f4c-72a1-b595-6487fceb7b6b`
Status: CLEAR

## P0/P1/P2

None.

## Scope Reviewed

Read-only review of the current uncommitted B2 GUI bind-fix diff.

Reviewed files:

- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`

Boundary check:

- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`

## Findings

- `AgentConversationShell.tsx` adds `deriveRelayBindingState(selectedSession)`, deriving `targetProjectRoot` only from `selectedSession.project_root`; non-Codex sessions and missing `project_root` are disabled instead of guessed.
- `visibleSessions` and `conversationSessionOptions` now use all sessions and no longer filter by current project selection, so other-project Codex sessions remain selectable.
- The send path uses the same `relayTargetProjectRoot` for `target_project_root`, `target_cwd`, and `allowed_write_roots`.
- The composer receives and displays the same `relayTargetProjectRoot`, avoiding stale `selectedProjectRoot`.
- Offline coverage includes cross-project Codex sessions not being hidden, Codex selected-session binding, missing `project_root` blocking, and non-Codex blocking without Enter-triggered direct relay.
- The five old gate files have empty `git diff`; no true Codex authorization path or old gate relaxation was found.

## Conclusion

STATUS: CLEAR. No P0/P1/P2 found.
