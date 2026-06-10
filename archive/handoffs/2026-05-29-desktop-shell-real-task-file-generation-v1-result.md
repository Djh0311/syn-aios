# Handoff: desktop shell real task file generation v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-v1.md`

## Result

Implemented user-confirmed generation of a real task package Markdown file from the selected task draft.

The implementation writes only through the new backend command path. This pass did not generate a real task file under `/Users/yoyi/workspace/product-line/tasks/`; Rust tests used temporary task directories.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-real-task-file-generation-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-v1-result.md`

## Backend Behavior

- New command: `generate_task_package_file`.
- Request fields:
  - `project_root`
  - `work_item_id`
- Real output directory is fixed in backend code:
  - `/Users/yoyi/workspace/product-line/tasks`
- The command rejects:
  - non-index project roots
  - missing workflow state file
  - missing workflow
  - missing work item
  - missing `task_package` artifact

## File Naming And Collision

- File name format: `2026-05-29-generated-<slug>.md`.
- Slug allows only lowercase ASCII letters, digits, and hyphens.
- Empty title slug falls back to `task-package-<work-item-short-id>`.
- Existing files are not overwritten.
- Collision strategy is suffix generation, for example `-2.md`.

## State Writes

- Updates `artifacts[].path`.
- Updates `artifacts[].updated_at`.
- Removes `draft_only_no_markdown_file`.
- Keeps missing-field warnings.
- Adds `audit_events[]` entry:
  - `event_type`: `task_package_file_generated`
  - `target_ref`: selected `work_item_id`
  - `before_state`: `draft`
  - `after_state`: `draft`
- Leaves `work_items[].state` as `draft`.
- Backs up workflow state before writing.

## Frontend Behavior

- `TaskDraftSummary` now carries `artifact_path`.
- Selected draft can show a “生成任务包文件” action.
- Confirmation dialog states the write directory and the no-dispatch/no-harness/no-Codex-state boundary.
- After success, `App` replaces the workflow state snapshot with the backend result snapshot.
- If `artifact_path` already exists, UI shows the path and disables the generate button with `已生成`.

## Verification

- `cargo fmt`: passed.
- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 37 tests.

## Explicit Non-Actions

- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not run real Tauri window smoke validation.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not print real workflow state body.
- Did not create a real `/Users/yoyi/workspace/product-line/tasks/*.md` task file during this pass.

## Risks

- Real-window validation remains paused by task scope.
- The real generation path is covered by code and confirmation text, while file creation itself was tested in temp directories.
- The date prefix is fixed to this task date fallback unless the app later gets a production date provider.
