# Evidence: desktop shell task field correction input v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-field-correction-input-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented a clear task field correction input flow for not-ready task packages.

## Boundary

- Did not generate a new real `product-line/tasks/*.md` file.
- Did not write the real workflow state in this implementation pass.
- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not write project business directories.
- Did not print the full real `workflow-state.v0.json` body.
- Did not modify `product-line/tasks/README.md`.

## Frontend Evidence

- Added a dedicated `修正任务字段` panel near dispatch readiness.
- The panel includes:
  - 任务名
  - 所属开发线
  - 背景
  - 目标
  - 允许读取
  - 允许写入
  - 禁止事项
  - 验收标准
  - 必须回传
  - 总指导回收重点
- The panel shows a field-level preview before save.
- Missing-field preview includes explicit prompts such as:
  - `目标缺失`
  - `允许写入缺失`
- The save action uses the confirmation dialog before writing.
- Confirmation text states:
  - only writes the workbench workflow state
  - does not generate a real task package file
  - does not dispatch Codex
  - does not start Codex CLI
  - does not run harness
  - does not write `.codex` or Codex state DB

## Backend Evidence

- Added command:
  - `correct_task_package_dispatch_fields`
- It reuses the structured task field storage model.
- It rejects non-index projects.
- It rejects missing state file, workflow, work item, or `task_package` artifact.
- It backs up old workflow state before writing.
- It writes audit event:
  - `task_package_fields_corrected_for_dispatch`
- It preserves existing `artifact.path` instead of clearing the generated task file path.
- Empty fields are stored as empty/missing and continue to produce missing warnings.

## Real State Check

Field-level real state check after implementation:

- `artifact_path = /Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- `task_name = null`
- `dispatch_correction_events = 0`
- `audit_events = 4`

This confirms the implementation pass did not save real corrected fields.

## Real Task File Check

Generated-prefix task files after this task:

- `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`

No new real ready task package file was generated.

## Tests

Rust tests cover:

- Non-index project rejected before correction save.
- Missing state file rejected.
- Missing workflow rejected.
- Missing work item rejected.
- Missing `task_package` artifact rejected.
- Correction save backs up old state.
- Correction save writes `task_package_fields_corrected_for_dispatch`.
- Correction save preserves existing `artifact.path`.
- Empty fields are not invented.
- Readiness can be rechecked after correction save.

Frontend offline tests cover:

- Correction entry exists.
- Field preview shows key fields.
- Missing fields show missing prompts.
- Confirmation dialog states no real task file generation, no Codex dispatch, and no harness.
- Canceling confirmation does not execute the save.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 46 tests passed and 1 real confirmation test ignored.

## Known Weak Points

- The current real task remains not ready because no real corrected business fields were provided or saved.
- Saving corrected fields in the real app still requires user confirmation in the Tauri UI.
- The preview is field-level; it does not yet render a full corrected Markdown preview before save.
