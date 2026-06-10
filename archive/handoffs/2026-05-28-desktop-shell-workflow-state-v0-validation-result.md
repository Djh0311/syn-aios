# Handoff: desktop shell workflow-state v0 validation result

## Verdict

原任务验收口径不适配真实试用场景。

依据：真实 Tauri 窗口验证中，workflow-state v0 面板和初始化确认弹层都已看到；用户看完弹层后点击了 `确认执行`，导致真实 `workflow-state.v0.json` 曾被创建。用户随后明确更正：这不是纯粹误点，也不是验证坐标点击造成，而是因为不理解为什么弹出这个窗口，看完后自然确认。这个结果不能按原任务“旁路观察、不执行写入”的验收口径回收为通过，但也不应定性为产品验证失败或按钮误触风险。

## What was validated

- Static and offline checks passed:
  - `npm run typecheck`
  - `npm run test:offline-interaction`
  - `npm run build`
  - `cargo test --offline`
- Real Tauri dev window opened.
- Window metadata:
  - title: `Codex 治理工作台`
  - size: `1280 x 820`
- Real window project page showed missing-state UI:
  - `本地事实层 v0`
  - `exists=false`
  - `未初始化`
  - `状态文件不存在；不会自动创建。`
  - `workflow-state.v0.json`
- Real window initialization dialog opened and showed:
  - target path
  - `Tauri 应用数据目录`
  - write boundary
  - `workflow-state.v0.json`
  - `backups`
  - not writing `.codex`
  - not writing Codex state DB
  - not writing project business dir
  - `audit event`
  - temp file and atomic replace wording

## Validation-design mismatch

The original intended action was `取消`, but the user clicked `确认执行` after reading the dialog. The user later clarified this was not a pure accidental click and not from the validation coordinate-click command. The user did not understand why the dialog appeared, read it, and then naturally confirmed it.

After that, the UI showed:

- `已初始化`
- `exists=true`
- `workflow_state_v0`
- `workflow version 1`
- `audit events 1`

The real state file existence check also returned present.

## Recovery done

The single state file created during this validation attempt was removed:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

Post-cleanup checks:

- The same state file path no longer exists.
- Port `5173` has no listener.
- Narrow filtered process check found no remaining Tauri/Vite/cargo-tauri process for this app.

Cleaned process IDs:

- `37150`
- `37174`
- `37308`
- `37339`
- `37340`
- `37367`

## Evidence

Evidence file:

- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0-validation.md`

## Read/write boundary

Read:

- task package and task queue
- productized desktop shell source and Tauri config
- real state file existence only, not content

Wrote:

- this handoff
- evidence file
- removed the accidentally created real state file

No evidence of:

- reading real state file content
- writing `/Users/yoyi/.codex`
- modifying Codex state DB
- writing project business directories
- running harness
- pulling network dependencies

## Recommendation

Do not treat this as a write-confirmation hardening task. The next task should clarify the product meaning of workflow-state initialization or continue workflow implementation after accepting that initializing the workbench's own state file is a normal first-use action.
