# Evidence: desktop shell task markdown preview v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented read-only Markdown preview rendering for existing task package drafts.

## Boundary

- Did not generate any real `product-line/tasks/*.md` task package file.
- Did not write the real workbench state file.
- Did not start Codex CLI.
- Did not run harness.
- Did not dispatch any real Codex session.
- Did not do real Tauri window smoke validation.
- Did not read or print the real workflow state file body.
- Did not write `/Users/yoyi/.codex`, Codex state DB, or project business directories.

## Backend Evidence

- Added read-only `render_task_package_preview` Tauri command.
- Added `copy_task_package_preview` command that re-renders the preview and copies text to clipboard after frontend confirmation.
- Preview input is limited to indexed project root plus existing `work_item_id`.
- Rendering rejects:
  - non-index project
  - missing workflow state file
  - missing project workflow
  - missing work item
  - missing `task_package` artifact
- Rendering does not append audit events and does not mutate workflow state.
- Preview uses `work_items[]`, `artifacts[]`, and indexed project metadata.

## Preview Fields

Markdown preview includes:

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

When a field is missing, preview uses `待补充` or `未登记`.

## Frontend Evidence

- Project detail workflow panel now shows a Markdown preview area.
- Task draft area says `预览，不是已派发任务包`.
- Existing task draft list remains visible.
- The preview panel asks the user to choose a task draft before rendering.
- Copying preview text uses the shared confirmation dialog.
- Confirmation text says it only copies preview text and does not write a real task file or dispatch a real Codex session.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 22 Rust tests passed.
- `find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -name '*task-markdown-preview-v1*' -type f`: returned only the original task package path.

## Tests Added

Rust tests added for:

- non-index project preview rejection
- missing state file preview rejection
- missing workflow preview rejection
- missing work item preview rejection
- successful Markdown preview rendering from a task draft
- missing fields producing `待补充` / `未登记`

Frontend offline test additions cover:

- preview area text
- preview-not-dispatched label
- copy preview confirmation dialog
- canceling copy confirmation without executing copy

## Known Weak Points

- Real Tauri window was not smoke tested because this task explicitly excludes it.
- Frontend offline tests do not fully execute async preview rendering through a real browser or Tauri IPC.
- The preview template is minimal and uses placeholders for fields not yet captured by task drafts.
