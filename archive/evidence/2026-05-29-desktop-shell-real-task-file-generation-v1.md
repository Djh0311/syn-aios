# Evidence: desktop shell real task file generation v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented user-confirmed generation of a real task package Markdown file from the selected task draft.

## Boundary

- No real Codex session was dispatched.
- Codex CLI was not started.
- Harness was not run.
- Real Tauri window smoke validation was not run.
- `/Users/yoyi/.codex` was not written.
- Codex state DB was not written.
- No project business directory was written.
- The real workflow state file body was not printed into evidence or handoff.

## Real Task File Status

- No real task file was generated under `/Users/yoyi/workspace/product-line/tasks/` during this implementation pass.
- Verification command for generated files returned no paths:
  - `find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'`
- Generation was tested with temporary `tasks` directories in Rust tests.

## Backend Evidence

- Added `generate_task_package_file`.
- Input is limited to `project_root` and `work_item_id`.
- The backend confirms:
  - project root exists in the static index
  - workflow state file exists
  - workflow exists
  - work item exists
  - matching `task_package` artifact exists
- Markdown generation reuses the same structured field rendering path used by preview.
- Output directory in the real command is fixed to `/Users/yoyi/workspace/product-line/tasks`.
- The frontend cannot pass an arbitrary output directory.

## File Naming

- File name format: `2026-05-29-generated-<slug>.md`.
- Slug only keeps lowercase ASCII letters, digits, and hyphens.
- Empty slug falls back to `task-package-<work-item-short-id>`.
- Slug is truncated before file creation.
- Collision policy: automatically select the next suffix, such as `-2.md`.
- Existing files are not overwritten.

## Write Safety

- Task file write uses a temporary file and final rename.
- If the selected final path already exists, generation is rejected before writing.
- Existing conflicts are handled by choosing a new suffix.
- Workflow state is backed up before writing changes.
- Workflow state is written with the existing atomic JSON writer.
- After writing, the backend rereads the generated file and state snapshot.

## State Updates

- Updated artifact field:
  - `artifacts[].path`
  - `artifacts[].updated_at`
  - `artifacts[].warnings`
- Warning handling:
  - removes `draft_only_no_markdown_file`
  - keeps missing-field warnings instead of pretending facts were filled
- Added audit event:
  - `event_type`: `task_package_file_generated`
  - `target_ref`: selected `work_item_id`
  - `actor_ref`: `user_confirmed_desktop_shell`
  - `permission_level`: `user_confirmed_write`
  - `before_state`: `draft`
  - `after_state`: `draft`
- `work_items[].state` stays `draft`.

## Frontend Evidence

- Selected task draft summary now includes `artifact_path`.
- Project detail shows a “生成任务包文件” entry for the selected draft.
- Existing generated path is shown in the UI.
- If `artifact_path` exists, the button displays `已生成` and is disabled.
- Confirmation dialog states:
  - writes to `/Users/yoyi/workspace/product-line/tasks/`
  - does not dispatch Codex
  - does not start Codex CLI
  - does not run harness
  - does not write `/Users/yoyi/.codex` or Codex state DB

## Verification

- `cargo fmt`: passed.
- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 37 Rust tests passed.

## Rust Test Coverage

- Non-index project is rejected.
- Missing workflow state file is rejected.
- Missing workflow is rejected.
- Missing work item is rejected.
- Missing `task_package` artifact is rejected.
- Temporary tasks directory receives generated Markdown.
- Existing file is not overwritten; suffix path is used.
- `artifacts[].path` is updated.
- `task_package_file_generated` audit event is written.
- Generated content comes from structured fields.
- Missing fields remain placeholders such as `待补充` or `未登记`.

## Frontend Test Coverage

- Selected draft can show the generate entry.
- Confirmation dialog includes write directory and no-dispatch/no-harness/no-Codex-state wording.
- Canceling confirmation does not call the generation action.
- Existing `artifact_path` shows generated state and disables the button.

## Known Weak Points

- Real Tauri window smoke validation is still not covered because this task explicitly excludes it.
- No real `/Users/yoyi/workspace/product-line/tasks/*.md` file was generated in this pass; the real path is only exercised through the backend command definition and UI confirmation.
- The date prefix currently uses the task date fallback unless an internal test/date override is set. That is acceptable for this dated task, but should be revisited if the app needs day-by-day production use.
- This does not dispatch the generated file to Codex; dispatch remains a later task.
