# Evidence: desktop shell project agent session entry v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-project-agent-session-entry-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented a project detail entry for read-only Codex Agent sessions.

## Boundary

- Did not add send-message UI or behavior.
- Did not add new-session UI or behavior.
- Did not add `codex resume`, `codex fork`, delete, move, archive, or dispatch behavior.
- Did not run Codex CLI.
- Did not run harness.
- Did not write `/Users/yoyi/.codex`.
- Did not read `auth.json`, `.env`, authorization files, or secret files.
- Did not read or write full real transcript content into this evidence.
- Did not change Rust backend code.

## Frontend Evidence

- Project detail rail now includes `Agent 会话`.
- Project detail rail has a selectable project tool state.
- Project sessions are filtered with:
  - `session.project_root === project.project_root`
- The project Agent session panel reuses `AgentSessionCenter` from the Agent page.
- `AgentSessionCenter` now accepts project-scoped labels and empty-state text.
- The project-scoped panel shows:
  - `项目内 Agent 会话`
  - `项目归属来源：索引推断`
  - per-session source label `索引推断`
  - empty state `当前项目没有索引推断关联的 Codex 会话。`
- Transcript loading is still single-session only:
  - project page passes the same `loadCodexSessionTranscript` loader from `App`
  - project panel calls the loader only from the selected session open/read path
  - no batch transcript loading was added

## Tests

Frontend offline tests cover:

- Project rail contains `Agent 会话`.
- Project session filtering keeps only sessions whose `project_root` matches the current project.
- Project-scoped `AgentSessionCenter` shows `索引推断`.
- Project-scoped empty state is present.
- Project Agent session panel does not show send, new-session, resume, delete, or move entry text.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.

## Known Weak Points

- The project binding is still index inference only; it is not user-confirmed binding.
- The current offline test checks the reusable panel and filter function, not a full browser click-through of the project rail.
- Project tool placeholder behavior for non-Agent rail entries remains minimal and was not expanded in this task.
