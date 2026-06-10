# Handoff: desktop shell task package dispatch readiness v1 result

## Task

`product-line/tasks/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`

## Result

Implemented dispatch readiness inspection for generated task packages.

The current real generated task package was correctly judged `not_ready`; no new ready task package file was generated because the existing content is polluted and no corrected business content was provided.

## Files Changed

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1-result.md`

## Backend

- Added `inspect_task_package_dispatch_readiness`.
- Response includes:
  - `status`
  - `blocking_reasons`
  - `warnings`
  - `artifact_path`
  - `can_generate_next_version`
- Status values:
  - `not_ready`
  - `ready`
  - `blocked`
- The check is read-only and does not mutate real workflow state.

## Readiness Rules

Blocks dispatch when:

- task file is missing
- title is missing or looks like a test draft
- goals are missing, placeholder, or polluted
- allowed read/write are missing or placeholder
- acceptance criteria are missing or placeholder
- forbidden actions contain old “do not generate real task package” rules
- required return lacks standard fields

## Current Real Status

- File checked:
  - `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- Status:
  - `not_ready`
- Reasons:
  - task title still looks like a test draft
  - input method pollution exists
  - placeholders remain
  - historical conflicting generation ban remains

## Frontend

- Added dispatch readiness panel in the task draft area.
- Shows readiness status and blocking reasons.
- “生成可派发版本” is disabled unless readiness is `ready`.
- Ready generation action still routes through the existing confirmation boundary and does not dispatch Codex.

## Tests

- Rust tests now cover polluted draft, missing fields, conflicting generation ban, ready after field correction, and next-version generation without overwriting.
- Frontend offline tests cover readiness display and disabled ready generation before ready.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed.
- `npm run build`: passed.
- `cargo test --offline` with the shared local cargo home and target dir: passed, 41 tests passed and 1 real confirmation test ignored.

## Explicit Non-Actions

- Did not generate a new real ready task package file.
- Did not dispatch a real Codex session.
- Did not start Codex CLI.
- Did not run harness.
- Did not write `/Users/yoyi/.codex`.
- Did not write Codex state DB.
- Did not print the full real workflow-state body.
- Did not modify `product-line/tasks/README.md`.

## Risks

- Readiness is rule-based and conservative.
- There is no corrected task content yet, so the current real generated package remains not ready.
- A future task should add a clearer correction workflow or require user-provided corrected fields before generating a ready version.
