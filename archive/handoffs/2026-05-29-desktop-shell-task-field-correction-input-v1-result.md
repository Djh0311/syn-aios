# Handoff: desktop shell task field correction input v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-field-correction-input-v1.md`

## Result

Implemented a dedicated task field correction input flow for not-ready task packages.

The user now has a clear place to enter real task content, preview missing fields, and save corrected structured fields after confirmation. This task did not save real corrected business content and did not generate a new real task package file.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-field-correction-input-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-field-correction-input-v1-result.md`

## Backend

- Added `correct_task_package_dispatch_fields`.
- Audit event for this path:
  - `task_package_fields_corrected_for_dispatch`
- Dispatch correction keeps existing `artifact.path`.
- Empty fields remain missing; the backend does not invent content.
- Existing ordinary field edit behavior remains separate.

## Frontend

- Added `修正任务字段` panel.
- Added field-level preview.
- Added missing-field prompts.
- Save goes through confirmation.
- Confirmation says it does not generate a real task package, dispatch Codex, start Codex CLI, run harness, or write `.codex` / Codex state DB.

## Real State

No real workflow-state correction save was performed in this task.

Field-level check after implementation:

- `dispatch_correction_events = 0`
- `audit_events = 4`
- existing artifact path remains `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`

## Real Task Files

No new real task package file was generated.

Existing generated-prefix file remains:

- `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 46 tests passed and 1 real confirmation test ignored.

## Explicit Non-Actions

- Did not generate a new real task package file.
- Did not save real corrected business fields.
- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not print the full real workflow-state body.

## Risks

- The real task remains not ready until a user provides corrected business fields and confirms saving.
- The correction preview is field-level, not full Markdown preview.
- A future task should connect “save correction” to an automatic readiness refresh in the live UI after the backend mutation returns.
