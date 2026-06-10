# Handoff: desktop shell task markdown preview v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-markdown-preview-v1.md`

## Result

Implemented read-only Markdown preview rendering for existing task package drafts in the productized desktop shell.

This does not generate a real task markdown file. It does not dispatch a real Codex session. It does not mutate the real workflow state file.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-result.md`

## Backend Details

- Added `TaskPackagePreviewRequest`.
- Added `TaskPackagePreview`.
- Added `render_task_package_preview`.
- Added `copy_task_package_preview`.
- Added read-only helper `render_task_package_preview_at`.
- Added lookup helpers for work item and task package artifact.
- Added a minimal standard task package Markdown renderer.

## Preview Fields

The generated preview contains:

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

Missing data is rendered as `待补充` or `未登记`.

## Guards

- Non-index project is rejected.
- Missing workflow state file is rejected.
- Missing workflow is rejected.
- Missing work item is rejected.
- Missing `task_package` artifact is rejected.
- Rendering is read-only and writes no audit event.
- Copy action only copies preview text after confirmation.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 22 tests.

## Not Done

- No real Tauri window smoke validation.
- No real task markdown file generation.
- No real Codex dispatch.
- No real workflow state mutation.
- No full browser layout verification.

## Next Suggestions

- Add live Tauri smoke only when user resumes real-window validation.
- Decide which extra task draft fields are required before file generation v1.
- Next likely step: explicit user-confirmed task markdown file generation from the preview.
