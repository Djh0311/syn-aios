# Evidence: prepare real workflow state for safe probe multiline v1

## Scope

- Task package: `/Users/yoyi/workspace/product-line/tasks/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1.md`
- Executed line: desktop app line / workflow runtime line only.

## User Confirmed Target

- Project path: `/Users/yoyi/gameai/agent world`
- Workflow id: `workflow:users-yoyi-gameai-agent-world:default`
- Node id: `workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`
- Work item id: `work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- Test thread id: `019e7389-349a-7f02-aa31-a4a90b24e865`
- Test session name: `请只回复这一句：CONTROL_PROBE_OK_2026_05_29`
- Test session cwd: `/private/tmp/codex-control-probe-v2`

## What Was Written

Real workflow state was written:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

Backup created:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780073411272.json`

Written field types:

- Added one active `workflow_node_session_bindings[]` record for the confirmed test thread.
- Updated the target `work_items[]` record from `draft` to `ready_to_dispatch`.
- Updated the target work item `current_node_id`.
- Updated the target workflow node state to `ready_to_dispatch`.
- Added two `audit_events[]` records.
- Updated top-level and related `updated_at` metadata.

No full workflow state body is copied here.

## Warnings Recorded

The binding record includes warnings:

- `session_not_found_in_current_index`
- `session_cwd_differs_from_project_root`
- `test_session_cwd:/private/tmp/codex-control-probe-v2`
- `confirmed_test_session_not_business_session`

Reason:

- The confirmed thread id was not found in current `codex-index.json`.
- The test session cwd differs from the target project path.
- The session is recorded as a user-confirmed test session, not a business session.

## Audit Events

- `audit:workflow-node-session-real-bind:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420:1780073411272`
- `audit:work-item-ready-to-dispatch:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420:1780073411272`

## Prohibited Actions Check

- Executed `codex exec resume`: no.
- Sent safe probe: no.
- Wrote `/Users/yoyi/.codex`: no.
- Read full transcript: no.
- Read `auth.json`, `.env`, secrets, tokens, or authorization files: no.
- Ran harness: no.
- Deleted, moved, or archived Codex sessions: no.
- Touched real business session: no.

## Read-Only Verification

Post-write structural verification:

- Active binding exists: yes.
- Binding thread id equals confirmed thread id: yes.
- Binding node id equals confirmed node id: yes.
- Binding work item id equals confirmed work item id: yes.
- Target work item state is `ready_to_dispatch`: yes.
- `workflow_node_dispatches[]` count remains `0`: yes.
- Project path mismatch warning recorded: yes.

Conclusion:

- The real workflow state now satisfies the next safe probe precondition.
- This is only state preparation; no safe probe was dispatched.

## Commands Run

```bash
npm run typecheck
npm run test:offline-interaction
```

Results:

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, 3 tests.

No application code was changed, so build and Rust test were not required for this state-only task.
