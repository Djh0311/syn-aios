# Agent View Info Surfacing Cleanup v1 Evidence

Date: 2026-06-20

Plan: `docs/plans/2026-06-20-agent-view-info-surfacing-cleanup-plan-v1.md`

## Scope

- Completed the Agent view presentation cleanup for the normal conversation surface.
- Normal composer no longer shows Manual relay boundary/envelope/receipt internals.
- Manual relay and guard errors now show a user-facing sentence plus a next step.
- Raw relay diagnostics are preserved under the existing bottom `开发者详情` disclosure.
- Backend guard, sandbox, and relay execution logic were not changed.

## Changed Surface

- `src/views/agent/AgentChatComposer.tsx`
  - Removed the composer-level `manual-relay-boundary-details` disclosure.
  - Shows project labels/path tails instead of full project roots in the composer target.
  - Renders `manualRelayError` through the user-facing error mapper.
  - Keeps `Stop` and `恢复轮询` controls on the main path.
- `src/views/agent/AgentConversationShell.tsx`
  - Adds `AgentManualRelayDeveloperDetails` inside the existing `开发者详情` disclosure.
  - Preserves raw guard reason, envelope target binding, command plan, and receipt fields there.
- `src/views/agent/agentLabels.ts`
  - Adds Manual relay reason labels and `userFacingAgentError`.
- `src/views/agent/AgentExecutionPanels.tsx`
  - Replaces raw top-level `操作失败：{error}` with a user-facing message and raw details disclosure.
- `src/styles.css` and `src/manualRelay.css`
  - Adds compact error styling and removes the obsolete composer relay-boundary flyout styling.

## Verification

Commands run from `product-line/prototypes/productized-desktop-shell`:

- `npm run typecheck` passed.
- `git diff --check`
- `npm run test:offline-interaction` passed.
- `git diff --check` passed.

## Evidence Notes

- Offline tests now assert the normal composer does not render `target_cwd_canonical`, `real_codex_executed`, `会话ID`, or `manual-relay-boundary-details`.
- Offline tests assert a blocked Manual relay error renders human-facing sensitive-material copy and does not expose `manual_relay_guard_blocked` or `manual_relay_denied_material_requested` on the main composer.
- Offline tests assert `AgentManualRelayDeveloperDetails` still contains raw diagnostic fields including `allowed_write_roots`, `path_verified`, `manual_once / auto_chain=false`, `process_kind=fixture`, and `real_codex_executed=false`.
- The worktree already had unrelated backend changes, including `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`; this cleanup did not edit backend guard, sandbox, or runner logic.
