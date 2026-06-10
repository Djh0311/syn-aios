# Handoff: desktop shell project agent session entry v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-project-agent-session-entry-v1.md`

## Result

Implemented a read-only `Agent 会话` entry inside the project detail page.

The project page now filters Codex sessions to the current project and reuses the Agent page session center instead of creating another chat system.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-project-agent-session-entry-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-agent-session-entry-v1-result.md`

## Implementation Notes

- Reused `AgentSessionCenter`.
- Added project-scoped props for title, description, source label, and empty state.
- Added `filterProjectSessionsForProject`.
- Filtering rule:
  - `session.project_root === project.project_root`
- `App` passes the existing `loadCodexSessionTranscript` loader into `ProjectsView`.
- `ProjectAgentSessionsPanel` reads transcript only when a single selected session is opened/read.
- UI displays `项目归属来源：索引推断`.

## Explicit Non-Actions

- Did not add send-message capability.
- Did not add create-session capability.
- Did not add `codex resume`.
- Did not add delete, move, archive, or dispatch controls.
- Did not run Codex CLI.
- Did not run harness.
- Did not write `/Users/yoyi/.codex`.
- Did not read auth, env, authorization, secret files, or real business transcript bodies.
- Did not change Rust backend code.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.

## Risks

- `project_root` is still inferred from the static index, so the UI labels it as `索引推断`.
- There is no user-confirmed project/session binding yet.
- Full browser-level click-through for the project rail is still a future test improvement.
