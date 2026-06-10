# Evidence: desktop shell real task file generation confirmation v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Confirmed one real task package file was generated in `/Users/yoyi/workspace/product-line/tasks/`.

## Boundary

- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not run real Tauri window smoke validation.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not write project business directories.
- Did not print the full real `workflow-state.v0.json` body.

## Preflight

- Real workflow state existed.
- Field-level check found:
  - `workflows`: 1
  - `work_items`: 1
  - `artifacts`: 1
  - `audit_events`: 3
  - one `task_package` draft with `artifact_path: null`
- Before generation, no `2026-05-29-generated-*.md` file existed in `/Users/yoyi/workspace/product-line/tasks/`.

## Real Generated File

- Generated file path:
  - `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- File exists.
- File is under `/Users/yoyi/workspace/product-line/tasks/`.
- File size from verification: 1610 bytes.
- First heading:
  - `# 任务包：task draft他日smoke`
- Standard sections verified present:
  - `# 任务包：`
  - `## 任务名`
  - `## 目标`
  - `## 禁止事项`
  - `## 验收标准`
  - `## 必须回传`

## No Overwrite Evidence

- Preflight `find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'` returned no paths.
- Post-generation the same command returned exactly:
  - `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- No existing generated-prefix task package was replaced.

## State Confirmation

- Artifact path after confirmation:
  - `artifacts[].path = /Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- Artifact updated:
  - `artifacts[].updated_at = 1780043100407`
  - `artifacts[].warnings = []`
- Audit after confirmation:
  - `event_type = task_package_file_generated`
  - `event_id = audit:task-file:work-item-workflow-users-yoyi-gameai-agent-world-default-1780032043420:1780043100407`
  - `target_ref = work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
  - `before_state = draft`
  - `after_state = draft`
  - `created_at = 1780043100407`
- Total audit events increased from 3 to 4.
- Generated audit count became 1.

## Backup

- Backup path produced by the confirmation run:
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780043100407.json`
- The backup file exists.

## Calling Method

- Added an ignored Rust confirmation test:
  - `real_task_package_file_generation_confirmation_v1`
- It calls the same backend generation helper used by `generate_task_package_file`.
- It is ignored by default so normal `cargo test --offline` does not write real task files or real workflow state.
- Explicit command used for real confirmation:
  - `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline real_task_package_file_generation_confirmation_v1 -- --ignored --nocapture`

## Implementation Fix During Confirmation

- First real confirmation attempt failed because sandboxed execution could not write the real workflow-state backup directory.
- That first attempt created the task file before state backup failed.
- This exposed a real ordering weakness.
- Backend generation was adjusted so state backup happens before new task-file write.
- Backend generation also now supports same-content idempotent recovery: if the expected file already exists and content matches exactly, it can backfill state instead of creating a second file.
- The successful confirmation used that recovery path to backfill `artifact.path` and audit for the already-created matching file.

## Verification Commands

- `cargo fmt`: passed.
- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 37 tests passed and 1 confirmation test ignored.
- Explicit confirmation test with `--ignored --nocapture`: passed, 1 test passed.

## Known Weak Points

- The confirmation was run through a controlled ignored Rust test, not through the real Tauri window.
- The first failed attempt revealed a partial-write risk; the code now backs up state before new file write and can recover same-content orphan files.
- The generated file has a mixed English/Chinese draft title because it came from the existing real task draft; this evidence does not judge that task title quality.
