# Handoff: desktop shell workflow orchestration min loop v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`

## Result

Implemented the minimum project workflow orchestration loop.

The project workflow view now shows work item state, responsible role, current node, next actions, and recent audit events. Advancing a work item state goes through confirmation and writes only the workbench workflow state.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1-result.md`

## Backend

- Added `update_work_item_state`.
- Added backend state transition validation.
- Added `work_item_state_changed` audit event writes.
- Added work item summary fields:
  - current node
  - next states
  - next action label
  - recent audit event summaries
- Backend rejects:
  - missing workflow state
  - missing workflow
  - missing work item
  - illegal transitions
  - non-index project

## Frontend

- Workflow view now focuses on orchestration instead of task package fields.
- Main workflow area shows:
  - state strip
  - Director / Developer / Review roles
  - current work item details
  - next action buttons
  - recent audit events
- State advance uses `advance-work-item-state` pending action and confirmation dialog.
- Confirmation copy says no Codex CLI, no resume, no real Codex dispatch, no harness, no `.codex`, and no Codex state DB.

## Real State

No real workflow-state write was performed in this task.

The implementation adds the ability to write these field types only after UI confirmation:

- `work_items[].state`
- `work_items[].current_node_id`
- node `state`
- `updated_at`
- `audit_events[]`

No full workflow-state body was printed or copied into evidence.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with shared local cargo home and target dir: passed, 51 tests passed and 1 real-write confirmation test ignored.

## Explicit Non-Actions

- Did not write `/Users/yoyi/.codex`.
- Did not run Codex CLI.
- Did not run `codex resume`.
- Did not read business session transcript bodies.
- Did not run harness.
- Did not generate a real task package file.
- Did not create or dispatch a real Codex business session.

## Remaining Gap

The workflow can now be arranged and advanced inside the workbench state file, but it still cannot automatically execute Codex. The missing pieces are real session binding, controlled Codex execution, retry rules, and review-to-execution feedback once those permissions and runtime rules are explicitly defined.
