# Handoff: desktop shell task draft v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`

## Result

Implemented the minimum task package draft registration flow for the productized desktop shell.

The flow only registers a draft in the workbench workflow state model. It does not generate a real task markdown file and does not dispatch a real Codex session.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-draft-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-result.md`

## State Writes

The backend writes these draft fields when the user confirms creation:

- `work_items[].work_item_id`
- `work_items[].project_id`
- `work_items[].workflow_id`
- `work_items[].title`
- `work_items[].state = "draft"`
- `work_items[].source_kind = "workspace_state"`
- `work_items[].source_ref = artifact_id`
- `work_items[].assigned_role_id`
- `work_items[].agent_type = "codex"`
- `work_items[].adapter_id = "codex-local"`
- `work_items[].permission_level = "user_confirmed_write"`
- `work_items[].created_at`
- `work_items[].updated_at`
- `artifacts[].artifact_id`
- `artifacts[].artifact_type = "task_package"`
- `artifacts[].project_id`
- `artifacts[].path = null`
- `artifacts[].title`
- `artifacts[].brief`
- `artifacts[].source_kind = "workspace_state"`
- `artifacts[].source_ref = work_item_id`
- `artifacts[].permission_level = "user_confirmed_write"`
- `artifacts[].created_at`
- `artifacts[].updated_at`
- `artifacts[].warnings = ["draft_only_no_markdown_file"]`
- `audit_events[]` for `task_draft_created`

## Existing Workflow Guard

- Tauri command checks the project root exists in the current index before writing.
- The write helper computes the default workflow id from the project root.
- If the workflow does not exist in `workflows[]`, creation is rejected.
- Unit tests cover missing workflow state, missing project workflow, and non-index project rejection.

## No Real Dispatch Guard

- Frontend confirmation text says the action only writes the workbench state file.
- Backend writes `artifacts[].path = null` and does not create task markdown.
- No Codex CLI or harness command is invoked.
- No code path dispatches a Codex session.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 16 tests.
- Real workflow state file existed before and after by existence check only.

## Not Verified

- Real Tauri window click-through was not performed.
- Browser visual layout was not checked in this task.
- Real application support state file content was not inspected.

## Next Suggestions

- Add a Tauri smoke pass that creates a temporary or explicitly approved real draft through the window.
- Add browser-level UI verification for the task draft form layout.
- Decide whether duplicate protection should become an explicit idempotency key instead of workflow plus title.
