# Evidence: desktop shell task draft v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented task package draft registration under an existing project workflow.

## Boundary

- Did not generate any real `product-line/tasks/*.md` task package file.
- Did not start Codex CLI.
- Did not run harness.
- Did not dispatch any real Codex session.
- Did not read or print the real workflow state file body.
- Did not write `/Users/yoyi/.codex`, Codex state DB, or project business directories.

## Real State File Existence

- Before task implementation, existence check for `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json` returned code 0.
- After task implementation, existence check for the same path returned code 0.
- The file body was not read or printed for this evidence.
- No real Tauri create action was run in this task; only unit and offline tests exercised temporary files or React fixtures.

## Backend Evidence

- Added `create_task_draft` Tauri command.
- The command first confirms the project root exists in `codex-index.json`.
- The write helper rejects missing state files and projects without an existing default workflow.
- Writes are backed up before mutation and then committed with the existing atomic JSON write helper.
- The mutation writes:
  - `work_items[]`
  - `artifacts[]` with `artifact_type = "task_package"` and `path = null`
  - `audit_events[]` with `event_type = "task_draft_created"`
- If the task node is not already `draft`, it is updated to `draft` and an extra audit event is recorded.
- Duplicate protection uses same workflow id plus trimmed title.

## Frontend Evidence

- Project workflow panel now shows task draft count.
- If the project has no workflow, the UI tells the user to create the default workflow first.
- If the project has a workflow, the UI shows a task draft form with:
  - title
  - objective
  - default assignment to `codex-dev`
- Creation goes through the shared confirmation dialog.
- Confirmation text states that no real task package file is generated and no real Codex session is dispatched.
- Existing task drafts are listed with title, state, and artifact type.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 16 Rust tests passed.
- `test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'`: passed after implementation.
- `lsof -nP -iTCP:5173 -sTCP:LISTEN`: returned code 1 with no output, meaning no dev server was listening on port 5173.

## Rust Test Coverage Added

- Missing workflow state rejects task draft creation.
- Existing state without project workflow rejects task draft creation.
- Existing workflow creates `work_items[]`, `artifacts[]`, and audit event.
- Non-index project is rejected before draft creation.
- Existing state is backed up before draft write.
- Same workflow and title do not duplicate the draft.

## Known Weak Points

- Real Tauri window was not used to click through task draft creation, so this evidence does not prove the live app writes the real application support state file.
- Offline frontend tests inspect React elements and callbacks, not browser layout or native Tauri IPC.
- Duplicate protection is intentionally minimal: same workflow id plus same trimmed title.
