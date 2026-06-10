# Evidence: desktop shell task draft selection v1

## Scope

- Task package: `product-line/tasks/2026-05-29-desktop-shell-task-draft-selection-v1.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Implemented a shared selected task draft state for task list, preview, copy, and field editing.

## Boundary

- Did not add backend commands.
- Did not add persistent selection fields to workflow state.
- Did not generate any real `product-line/tasks/*.md` task package file.
- Did not start Codex CLI.
- Did not run harness.
- Did not dispatch any real Codex session.
- Did not do real Tauri window smoke validation.
- Did not read or print the real workflow state file body.
- Did not write `/Users/yoyi/.codex`, Codex state DB, or project business directories.

## Selection State

- Selection is stored in frontend component state as `selectedWorkItemId`.
- If there are task drafts and no current selection, the first draft is selected.
- If the current selected id no longer exists after refresh, selection falls back to the first available draft.
- If there are no task drafts, selection is `null` and the UI shows a next-step prompt.

## Shared Use

The same selected task draft drives:

- task list selected marker
- Markdown preview render request
- preview copy request
- field editor target
- field save confirmation target

## Frontend Evidence

- Task list displays `当前选中` for the selected draft and `选择` for others.
- Preview controller receives the selected draft.
- Field editor receives the selected draft.
- Save action uses the selected draft's `work_item_id`.
- Copy preview action uses the selected preview's `work_item_id`.

## Verification

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, output reported `offline interaction tests passed: 3`.
- `npm run build`: passed.
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`: passed, 29 Rust tests passed.
- `find /Users/yoyi/workspace/product-line/tasks -maxdepth 1 -name '*task-draft-selection-v1*' -type f`: returned only the original task package path.

## Test Evidence

- Offline test fixture now includes two task drafts.
- Tests cover explicit selected marker text.
- Pure selection helpers cover:
  - missing selected id falls back to first draft
  - switching to second draft keeps the second draft selected
  - selected draft lookup returns the second draft
- Confirmation actions are checked with the second draft id to ensure they are not bound to the first draft.

## Known Weak Points

- Real Tauri window was not smoke tested because this task excludes it.
- Existing offline test harness is not a real React renderer, so hook-based selection behavior is covered by pure selection helpers plus static component checks.
- The UI selection state is not persisted. That is intentional for this task.
