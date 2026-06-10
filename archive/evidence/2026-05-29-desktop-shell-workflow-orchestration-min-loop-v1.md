# Evidence: desktop shell workflow orchestration min loop v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented a minimum work item orchestration loop in the desktop shell.

## Boundary

- Did not start Codex CLI.
- Did not run `codex resume`.
- Did not create a real Codex business session.
- Did not read business session transcript bodies.
- Did not run harness.
- Did not generate a real task package file for this task.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not write the real workflow state file during this implementation pass.
- Rust tests used temporary workflow-state files.

## Backend Evidence

- Added Tauri command:
  - `update_work_item_state`
- Added request type:
  - `WorkItemStateUpdateRequest`
- Legal state transitions:
  - `draft -> ready_to_dispatch`
  - `ready_to_dispatch -> running`
  - `running -> ready_for_review`
  - `ready_for_review -> accepted`
  - `ready_for_review -> needs_changes`
  - `needs_changes -> ready_to_dispatch`
  - `paused -> ready_to_dispatch`
  - non-accepted states may move to `paused`
- Illegal transitions are rejected by the backend.
- Non-index projects are rejected by the backend.
- State update writes:
  - `work_items[].state`
  - `work_items[].current_node_id`
  - matching node `state`
  - `updated_at`
  - `audit_events[]` with event type `work_item_state_changed`
- Existing work item summaries now include:
  - `current_node_id`
  - `next_states`
  - `next_action_label`
  - recent audit event summaries

## Frontend Evidence

- Project workflow view now centers on orchestration:
  - workflow state strip
  - Director / Developer / Review roles
  - current work item
  - responsible role
  - current node
  - next action
  - state transition buttons
  - recent audit events
- Task package field editing was moved below the main project layout, so it no longer occupies the workflow main visual.
- State transition actions go through the existing confirmation dialog.
- Confirmation copy says the action only writes the workbench workflow state and appends audit events.
- Session line remains read-only:
  - shows project session count
  - does not send messages
  - does not resume
  - does not read transcript bodies

## Tests

Rust tests cover:

- Legal work item state update writes state, current node, and audit event.
- Illegal transition is rejected and leaves state unchanged.
- Non-index project is rejected.

Frontend offline tests cover:

- Project workflow main view shows orchestration text and missing workflow gap.
- Workflow with work items shows state strip, current work item, responsible role, next action, and audit summary.
- State transition button builds an `advance-work-item-state` pending action.
- Confirmation dialog shows target state and no Codex CLI / resume / harness boundary.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 51 tests passed and 1 real-write confirmation test ignored.

## Known Weak Points

- This is still a workbench state loop, not real Codex execution.
- Work item selection is still minimal: the main orchestration card shows the first available work item.
- Node state is lightweight and derived from work item state; there is no drag/drop or graph editing.
- Session binding remains a placeholder; no workflow-bound Codex session is created or resumed.
