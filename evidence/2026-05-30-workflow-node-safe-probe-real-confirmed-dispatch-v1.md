# Evidence: workflow node safe probe real confirmed dispatch v1

## Scope

- Task package: `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
- Executed line: desktop app line / workflow runtime line.

## Weak Points

- This proves one real no-business safe probe dispatch, not real business automation.
- The target thread cwd is `/private/tmp/codex-control-probe-v2`, not the target project path `/Users/yoyi/gameai/agent world`.
- The Codex CLI run emitted plugin catalog auth and MCP shutdown warnings; final answer still matched the expected safe probe text.
- Long tasks, permission queues, retry, timeout, and automatic review remain unimplemented.

## User Approval

The user explicitly approved:

- executing real safe probe dispatch;
- running `codex exec resume`;
- writing `/Users/yoyi/.codex`;
- writing real workflow state dispatch records and audit events.

## Target

- Project path: `/Users/yoyi/gameai/agent world`
- Workflow id: `workflow:users-yoyi-gameai-agent-world:default`
- Node id: `workflow:users-yoyi-gameai-agent-world:default:node:codex-dev`
- Work item id: `work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- Thread id: `019e7389-349a-7f02-aa31-a4a90b24e865`

## Dispatch

Prompt sent:

```text
请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

Command shape:

```bash
codex exec resume --skip-git-repo-check --json --output-last-message <last-message-path> 019e7389-349a-7f02-aa31-a4a90b24e865 "<safe-probe-prompt>"
```

Exit code:

- `0`

Final reply summary:

```text
WORKFLOW_NODE_DISPATCH_OK_2026_05_29
```

Final reply exact match:

- yes.

## Workflow State Writes

Real workflow state written:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

Backups created:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780074921611.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780075042172.json`

Written field types:

- `workflow_node_dispatches[]`: prepared record.
- `workflow_node_dispatches[]`: completed record.
- `work_items[].state`: `ready_to_dispatch -> running -> ready_for_review`.
- `work_items[].current_node_id`.
- target workflow node state.
- review workflow node state.
- `audit_events[]`: prepared, started, completed, readback.
- `updated_at` metadata.

No full workflow state body is copied here.

## Dispatch Records

Prepared dispatch id:

- `dispatch:workflow-users-yoyi-gameai-agent-world-default:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420:1780074921611:prepared`

Completed dispatch id:

- `dispatch:workflow-users-yoyi-gameai-agent-world-default:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420:1780074921611:running`

Audit event ids:

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-gameai-agent-world-default-work-item-workflow-users-yoyi-gameai-age:1780074921611`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-gameai-agent-world-default-work-item-workflow-users-yoyi-gameai-age:1780074921611`
- `audit:workflow-node-dispatch-completed:dispatch-workflow-users-yoyi-gameai-agent-world-default-work-item-workflow-users-yoyi-gameai-age:1780075042172`
- `audit:workflow-node-dispatch-readback:dispatch-workflow-users-yoyi-gameai-agent-world-default-work-item-workflow-users-yoyi-gameai-age:1780075042172`

## Transcript Readback Stats

Only statistics were retained:

- total events: `32`
- parsed JSONL lines: `32`
- bad JSONL lines: `0`
- unknown events: `0`
- warning count: `3`
- encrypted content event count: `2`
- sensitive-like event count: `0`
- target text hits: `4`

The temporary transcript JSON produced by the reader was deleted after extracting these statistics.

## Prohibited Actions Check

- Executed real dispatch: yes.
- Wrote `/Users/yoyi/.codex`: yes, through `codex exec resume`.
- Wrote real workflow state: yes.
- Read `auth.json`, `.env`, secrets, tokens, or authorization files: no.
- Read full transcript into evidence or handoff: no.
- Touched real business session: no.
- Ran harness: no.
- Deleted, moved, or archived Codex sessions: no.
- Executed real business task: no.

## Verification Commands

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

Results are recorded in the handoff.
