# Handoff: desktop shell workflow node session binding v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`

## Result

Implemented workflow node binding to existing indexed Codex sessions.

The workflow can now store a node/work-item binding to a Codex thread in the workbench workflow state, show that binding in the project workflow UI, rebind it, detach it, and jump to the project Agent session view.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-node-session-binding-v1-result.md`

## Backend

- Added `workflow_node_session_bindings[]`.
- Added `bind_workflow_node_codex_session`.
- Added `unbind_workflow_node_codex_session`.
- Added binding summaries to project workflow snapshots.
- Added audit event writes for bind, rebind, and unbind.
- Rejects non-index projects, missing workflows, missing nodes, missing requested work items, and non-index sessions.

## Frontend

- Current work item card now shows node session binding status.
- Candidate sessions can be selected for binding.
- Bound sessions show title, thread id, update time, source, rollout/readability state, and warnings.
- Binding/unbinding goes through the existing confirmation dialog.
- `打开会话` switches to the project Agent session view and focuses the bound session id.

## Real State

No real workflow-state write was performed in this task.

If confirmed in the UI, this feature writes these binding field types:

- `workflow_node_session_bindings[].binding_id`
- `project_id`
- `workflow_id`
- `node_id`
- `work_item_id`
- `agent_type`
- `adapter_id`
- `native_thread_id`
- `native_rollout_path`
- `session_title`
- `session_updated_at_ms`
- `rollout_exists`
- `project_binding_source`
- `binding_source`
- `binding_mode`
- `lifecycle`
- timestamps
- warnings
- `audit_events[]`

No full workflow-state body was printed or copied into evidence.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with shared local cargo home and target dir: passed, 53 tests passed and 1 real-write confirmation test ignored.

## Explicit Non-Actions

- Did not write `/Users/yoyi/.codex`.
- Did not run Codex CLI.
- Did not run `codex resume`.
- Did not create a real Codex business session.
- Did not send messages to Codex.
- Did not automatically read business session transcript bodies.
- Did not run harness.
- Did not delete, move, or archive original Codex sessions.

## Remaining Gap

The workflow can now point at an existing Codex session, but it still cannot automatically execute Codex. The missing pieces are controlled session creation/resume, prompt dispatch, transcript readback rules, retry rules, and review-to-execution feedback.
