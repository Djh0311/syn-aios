# Handoff: desktop shell task fields edit v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`

## Result

Implemented structured task package field editing for existing task drafts in the productized desktop shell.

The edit flow writes only the workbench workflow state model after user confirmation. It does not generate a real task markdown file and does not dispatch a real Codex session.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-result.md`

## Structured Fields

Added support for:

- `task_name`
- `assigned_line`
- `background`
- `goals`
- `allowed_read`
- `allowed_write`
- `forbidden_actions`
- `acceptance_criteria`
- `required_return`
- `review_focus`
- `template_version = "task_package_v1"`

## Storage

- `work_items[]`: keeps synchronized title, assigned role id, and updated timestamp.
- `artifacts[]`: stores the task package structured fields on the matching `artifact_type = "task_package"` record.
- `artifacts[].path` remains `null`.
- `audit_events[]`: appends `task_package_fields_updated`.

## Guards

- Non-index project is rejected.
- Missing workflow state file is rejected.
- Missing workflow is rejected.
- Missing work item is rejected.
- Missing `task_package` artifact is rejected.
- Write backs up the old state file first.
- Write uses atomic replacement.

## Empty Fields

Empty fields are saved as empty values, not replaced by invented content. Missing field warnings are stored on the artifact, and Markdown preview displays `待补充` or `未登记`.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 29 tests.

## Not Done

- No real Tauri window smoke validation.
- No real task markdown file generation.
- No real Codex dispatch.
- No handoff / evidence / review registration beyond this task's evidence and handoff files.

## Next Suggestions

- Tighten task draft selection in the field editor when multiple drafts exist.
- Add real-window smoke only when user resumes that validation mode.
- Next likely step: user-confirmed generation of a real task markdown file from structured fields.
