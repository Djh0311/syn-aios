# Evidence: desktop shell task package dispatch readiness v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented task package dispatch readiness inspection and frontend display.

## Boundary

- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not generate a new real ready task package file.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not write project business directories.
- Did not print the full real `workflow-state.v0.json` body.
- Did not modify `product-line/tasks/README.md`.

## Readiness Rules

The backend marks a task package as not ready when it detects:

- Missing generated artifact path.
- Task name is empty, `待补充`, or still looks like a test draft.
- Goal is empty, `待补充`, `未登记`, or contains input method noise.
- Allowed read/write is empty or still contains placeholder text.
- Acceptance criteria is empty or still contains placeholder text.
- Forbidden actions contain a historical conflicting generation ban such as `不生成真实任务包文件`.
- Required return list lacks standard fields such as:
  - `做了什么`
  - `改了哪些文件`
  - `验证命令和结果`
  - `风险`

If no blocking reasons remain, status becomes `ready`. A future explicit blocked marker is treated as `blocked`.

## Current Real Task Package Status

- Current real task package path:
  - `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- Current readiness status:
  - `not_ready`
- Field-level reasons:
  - task title still looks like a test draft
  - contains input method noise
  - contains `待补充` / `未登记` placeholder text
  - contains historical conflicting generation ban
- The generated file still has standard sections:
  - `## 任务名`
  - `## 目标`
  - `## 禁止事项`
  - `## 验收标准`
  - `## 必须回传`
- It is not ready because the content quality is not acceptable, not because the file is missing.

## Backend Evidence

- Added command:
  - `inspect_task_package_dispatch_readiness`
- Added structured response:
  - `status`
  - `blocking_reasons`
  - `warnings`
  - `artifact_path`
  - `can_generate_next_version`
- The command checks index membership, workflow existence, work item existence, and matching `task_package` artifact.
- The command is read-only.

## Frontend Evidence

- Project task draft area now includes a dispatch readiness panel.
- The panel can request readiness inspection.
- The panel displays:
  - `not_ready`
  - `ready`
  - `blocked`
  - blocking reasons
  - warnings
  - artifact path
- The “生成可派发版本” button is disabled unless readiness is `ready`.
- The confirmation path for ready generation still says:
  - only generates a task package file
  - does not dispatch Codex
  - does not start Codex CLI
  - does not run harness
  - does not write `.codex` or Codex state DB

## Tests

Rust tests cover:

- Polluted generated draft is `not_ready`.
- Missing fields are `not_ready`.
- Historical conflicting generation ban is `not_ready`.
- Field correction plus generated file can become `ready`.
- A changed next version generates a new file without overwriting the old file.
- Artifact path updates after next-version generation.
- Audit event is written by generation.
- Missing fields are not invented.

Frontend offline tests cover:

- Readiness panel is wired to selected task draft.
- Not-ready display includes blocking reasons.
- Before ready, “生成可派发版本” is disabled.
- Confirmation copy for generation still states no dispatch and no harness.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 41 tests passed and 1 real confirmation test ignored.

## Generated Files

- No new real ready task package file was generated in this task.
- Existing generated-prefix files after this task:
  - `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`

## Known Weak Points

- The frontend does not yet provide a full dedicated “修正派发字段” form separate from the existing task field editor.
- The readiness check is intentionally conservative and string-rule based.
- The current real task remains not ready because there is no user-provided corrected business content.
