# Evidence: desktop shell task fields edit v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented structured task package field editing for existing task drafts.

## Boundary

- Did not generate any real `product-line/tasks/*.md` task package file.
- Did not start Codex CLI.
- Did not run harness.
- Did not dispatch any real Codex session.
- Did not do real Tauri window smoke validation.
- Did not read or print the real workflow state file body.
- Did not write `/Users/yoyi/.codex`, Codex state DB, or project business directories.

## Backend Evidence

- Added `update_task_package_draft_fields` Tauri command.
- Input is limited to:
  - indexed `project_root`
  - existing `work_item_id`
  - structured task package fields
- Update rejects:
  - non-index project
  - missing workflow state file
  - missing workflow
  - missing work item
  - missing `task_package` artifact
- Update backs up the existing state file before writing.
- Update uses existing atomic JSON write helper.
- Update appends `audit_events[]` with `event_type = "task_package_fields_updated"`.
- Update keeps `artifacts[].path = null`.

## Field Storage Mapping

Stored in `work_items[]`:

- `title`
- `assigned_role_id`
- `updated_at`

Stored in the matching `artifacts[]` record with `artifact_type = "task_package"`:

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
- `brief`
- `title`
- `warnings`
- `updated_at`
- `path = null`

## Empty Field Handling

- Empty scalar fields are saved as empty strings.
- Empty list fields are saved as empty arrays.
- `artifacts[].warnings` records missing field markers such as `missing_task_name`.
- Markdown preview renders missing values as `待补充` or `未登记`.
- No business content is invented.

## Preview Evidence

- `render_task_package_preview` now prefers structured fields on the artifact.
- If structured fields are absent, preview falls back to earlier draft title / brief behavior.
- Preview is not parsed back into facts.

## Frontend Evidence

- Task preview area includes an “编辑字段” form.
- Form fields cover:
  - 任务名
  - 所属开发线
  - 背景
  - 目标
  - 允许读取
  - 允许写入
  - 禁止事项
  - 验收标准
  - 必须回传
  - 总指导回收重点
- List fields use multiline text, one item per line.
- Save uses the shared confirmation dialog.
- Confirmation text says it writes the workbench state file and does not generate a real task file or dispatch a real Codex session.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 29 Rust tests passed.
- `find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -name '*task-fields-edit-v1*' -type f`: returned only the original task package path.

## Tests Added

Rust tests added for:

- non-index project update rejection
- missing state file update rejection
- missing workflow update rejection
- missing work item update rejection
- missing `task_package` artifact update rejection
- structured field update
- updated preview using structured fields
- backup before write
- audit event write
- empty fields staying empty and rendering as missing facts

Frontend offline test additions cover:

- field edit entry visibility
- standard task package fields
- save confirmation action
- canceling confirmation without saving
- confirmation text for no real task file and no real Codex dispatch

## Known Weak Points

- Real Tauri window was not smoke tested because this task excludes it.
- Frontend offline tests do not execute a native Tauri save.
- The UI currently edits the first task draft in the field form area; deeper per-draft selection can be tightened in a later UI pass.
