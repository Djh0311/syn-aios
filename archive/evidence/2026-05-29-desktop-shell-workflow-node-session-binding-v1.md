# Evidence: desktop shell workflow node session binding v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented workflow node binding to existing indexed Codex sessions.

## Boundary

- Did not start Codex CLI.
- Did not run `codex resume`.
- Did not create a real Codex business session.
- Did not send messages to Codex.
- Did not automatically read business session transcript bodies.
- Did not run harness.
- Did not delete, move, archive, or modify original Codex sessions.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not write the real workflow state file during this implementation pass.
- Rust tests used temporary workflow-state files.

## Backend Evidence

- Added Tauri commands:
  - `bind_workflow_node_codex_session`
  - `unbind_workflow_node_codex_session`
- Added state array:
  - `workflow_node_session_bindings[]`
- Binding fields written after user confirmation:
  - `binding_id`
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
  - `created_at_ms`
  - `updated_at_ms`
  - `warnings`
- Binding source is `workflow_bound`.
- Project binding source remains `index_inferred`.
- Missing rollout is allowed with warning:
  - `index_session_rollout_missing`
- Binding, rebinding, and unbinding audit event types:
  - `workflow_node_session_bound`
  - `workflow_node_session_rebound`
  - `workflow_node_session_unbound`
- Backend rejects:
  - non-index project
  - missing workflow
  - missing node
  - missing work item when work-item binding is requested
  - non-index session

## Frontend Evidence

- Project workflow current work item now shows `节点会话绑定`.
- Bound node shows:
  - session title
  - thread id
  - update time
  - project binding source
  - rollout/readability state
  - warnings
- Candidate session list uses indexed Codex sessions and marks project source as `索引推断`.
- Binding and unbinding go through confirmation.
- Confirmation copy says no Codex start, no message send, no resume, no transcript body read, and no Codex state DB write.
- `打开会话` switches to project Agent session view and focuses the bound thread id without reading transcript automatically.

## Tests

Rust tests cover:

- Binding writes one active binding and audit event.
- Rebinding updates the active binding and writes rebound audit event.
- Unbinding detaches the binding and writes unbound audit event.
- Non-index session is rejected.
- Missing node is rejected.

Frontend offline tests cover:

- Workflow view displays binding state.
- Candidate session button builds `bind-node-session` pending action.
- Unbind button builds `unbind-node-session` pending action.
- Confirmation dialog includes no-start, no-send, no-transcript-read, no-delete/move/archive boundaries.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 53 tests passed and 1 real-write confirmation test ignored.

## Known Weak Points

- Binding still selects an existing indexed session only.
- No new Codex session is created.
- No automatic Codex execution exists.
- Node binding is single active binding per node/work item.
- The Agent session jump focuses the session in the project Agent view, but transcript body still requires the user to click read/open.
