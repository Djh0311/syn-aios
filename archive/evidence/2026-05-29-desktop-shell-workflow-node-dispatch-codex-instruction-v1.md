# Evidence: desktop shell workflow node dispatch Codex instruction v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`

## What Changed

- Added backend workflow node dispatch commands:
  - `prepare_workflow_node_dispatch`
  - `execute_workflow_node_dispatch`
  - `read_workflow_node_dispatch_result`
- Added workflow state extension field:
  - `workflow_node_dispatches[]`
- Added safe probe prompt:
  - `请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29`
- Added dispatch audit events:
  - `workflow_node_dispatch_prepared`
  - `workflow_node_dispatch_started`
  - `workflow_node_dispatch_completed`
  - `workflow_node_dispatch_failed`
  - `workflow_node_dispatch_readback_completed`
- Added frontend workflow-node dispatch panel with:
  - safe probe preview
  - disabled user-reviewed dispatch entry
  - confirmation dialog copy covering `.codex`, workflow state, no secrets, no harness, no delete/move/archive
  - recent dispatch result summary

## Safety Notes

- No real Codex dispatch was executed in this task.
- No `codex exec resume` command was run against a real thread.
- No write to `/Users/yoyi/.codex` was performed by this task.
- No real workflow state file was mutated by manual execution in this task.
- Rust tests used temporary workflow state files and a stub Codex runner.
- No `auth.json`, `.env`, authorization file, or secret file was read.
- No complete transcript was saved to evidence, handoff, or workflow state.

## State Fields Added

`workflow_node_dispatches[]` records these fields:

- `dispatch_id`
- `project_id`
- `workflow_id`
- `node_id`
- `work_item_id`
- `binding_id`
- `native_thread_id`
- `prompt_preview`
- `prompt_kind`
- `state`
- `started_at_ms`
- `ended_at_ms`
- `exit_code`
- `last_message_path`
- `last_message_summary`
- `transcript_event_count`
- `transcript_target_hits`
- `warnings`

## Verification

Commands run:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

Results:

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, 3 offline interaction tests.
- `npm run build`: passed.
- `cargo test --offline` with the task package Cargo paths: passed, 56 passed, 1 ignored.

Notes:

- A plain `cargo check --offline` using the default Cargo cache failed before code validation because local offline cache did not have the locked `serde_json 1.0.150`. The task package Cargo paths were then used and passed.

## Remaining Gaps

- This is not real business workflow automation yet.
- User-reviewed instruction dispatch remains blocked until required fields and review protocol are complete.
- Long task handling, permission prompts during tool use, retry, timeout, and recovery are not implemented.
- Real dispatch still requires explicit user confirmation because it writes `/Users/yoyi/.codex`.
