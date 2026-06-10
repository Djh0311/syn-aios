# Handoff: desktop shell workflow node dispatch Codex instruction v1

## Summary

Implemented the first controlled dispatch path from a desktop workflow node to its bound Codex session.

The implemented path supports safe probe dispatch only:

```text
请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

User-reviewed business instruction dispatch is intentionally blocked in v1 when required task fields and protocol are incomplete.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-result.md`

## Backend Details

Added Tauri commands:

- `prepare_workflow_node_dispatch`
- `execute_workflow_node_dispatch`
- `read_workflow_node_dispatch_result`

Added workflow state field:

- `workflow_node_dispatches[]`

Execution behavior:

- Requires indexed project.
- Requires default workflow.
- Requires existing work item.
- Requires active node/session binding.
- Requires bound session to exist in the current index and have rollout.
- Safe probe execution requires work item state `ready_to_dispatch`.
- Execution writes prepared and running dispatch records, then completes or fails the running record.
- Work item advances `ready_to_dispatch -> running -> ready_for_review` on successful safe probe.
- Transcript reader result is reduced to statistics only.

## Frontend Details

Project workflow card now shows:

- bound session metadata
- safe probe prompt preview
- safe probe dispatch button
- disabled reviewed-business dispatch entry
- latest dispatch result summary

Confirmation dialog now states the dispatch risk boundary:

- sends a message to the bound Codex session
- writes `/Users/yoyi/.codex`
- writes workflow state
- does not read authorization or secrets
- does not run harness
- does not delete, move, or archive sessions

## Verification

Passed:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

Cargo result:

- 56 passed
- 1 ignored
- 0 failed

The ignored test is the existing real task package file generation confirmation test.

## Real Dispatch Status

- Real safe probe dispatch executed: no.
- Real thread id used: none.
- Wrote `/Users/yoyi/.codex`: no.
- Wrote real workflow state: no.
- Read `auth.json`, `.env`, authorization files, or secrets: no.
- Touched real business session: no.

Reason: this task did not include a new explicit user confirmation to run a real `codex exec resume` that writes `/Users/yoyi/.codex`.

## Known Gaps

- Long-running business tasks are not covered.
- Permission handling inside resumed Codex sessions is not covered.
- Timeout, retry, cancellation, and partial failure recovery are not implemented.
- User-reviewed instruction dispatch still needs a complete prompt schema and acceptance protocol.
- Real business automatic workflow still needs explicit dispatch confirmation, long-task protocol, permission confirmation, failure retry, and review/recovery rules.
