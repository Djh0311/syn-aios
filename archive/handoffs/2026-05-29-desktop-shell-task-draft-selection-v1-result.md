# Handoff: desktop shell task draft selection v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-draft-selection-v1.md`

## Result

Implemented a shared frontend-selected task draft state so the task list, Markdown preview, preview copy, and field editor all target the same `work_item_id`.

No backend command or persistent state field was added.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-draft-selection-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-selection-v1-result.md`

## Selection Model

- Stored in frontend component state as `selectedWorkItemId`.
- Defaults to the first available draft.
- Keeps the selected draft if it still exists.
- Falls back to the first draft if the previous selected id disappears.
- Clears to `null` when there are no drafts.

## Multi-Draft Behavior

- The list marks the selected row with `当前选中`.
- Non-selected rows show `选择`.
- Preview receives the selected draft.
- Copy preview receives the selected preview/work item id.
- Field editor receives the selected draft.
- Save fields action uses the selected draft id.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 29 tests.

## Not Done

- No real Tauri window smoke validation.
- No real task markdown file generation.
- No real Codex dispatch.
- No persistent recent-selection storage.

## Next Suggestions

- Run a real Tauri UI smoke once user resumes real-window validation.
- Use this shared selection as the prerequisite for user-confirmed real task markdown file generation.
