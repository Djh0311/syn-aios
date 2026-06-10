# Evidence: desktop shell workflow-state v0 validation

## Verdict

原任务验收口径不适配真实试用场景，本轮不按“通过验证”回收，也不按“产品验证失败”定性。

依据：真实 Tauri 窗口打开并展示了 workflow-state v0 面板和初始化确认弹层；用户看完弹层后点击了 `确认执行`，真实状态文件曾被创建。用户随后明确更正：这不是纯粹误点，也不是验证坐标点击造成，而是因为不理解为什么弹出这个窗口，看完后自然确认。原任务要求“不点击确认执行、不创建真实状态文件”，更适合旁路观察式验证，不适合真实用户试用。

## Scope

- Task package: `product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0-validation.md`
- Prototype: `product-line/prototypes/productized-desktop-shell/`
- Real state path existence check only:
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

No state file content was read.

## Files read

- `product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0-validation.md`
- `product-line/tasks/README.md`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/WorkflowStatePanel.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json`

## Validation commands

Run from `product-line/prototypes/productized-desktop-shell/`:

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

Results:

- `npm run typecheck`: passed.
- `npm run test:offline-interaction`: passed, `offline interaction tests passed: 3`.
- `npm run build`: passed, Vite build completed.

Run from `product-line/prototypes/productized-desktop-shell/src-tauri/`:

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

Result:

- passed.
- Rust unit tests: `6 passed; 0 failed`.

## Real Tauri window smoke

Started with:

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

Observed through real macOS window screenshots and System Events window metadata:

- Window title: `Codex 治理工作台`.
- Window size: `1280 x 820`.
- Initial page showed: `已读取索引。所有本机动作仍需用户点击并确认。`
- Project page showed:
  - `本地事实层 v0`
  - `工作流状态文件`
  - `未初始化`
  - `exists=false`
  - `状态文件不存在；不会自动创建。`
  - `workflow-state.v0.json`

## Initialization dialog

Clicked `初始化工作流事实层` in the real Tauri window.

Dialog appeared and showed:

- `初始化工作流事实层`
- `目标路径`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `路径来源`
- `Tauri 应用数据目录`
- `写入边界`
- `workflow-state.v0.json`
- `backups`
- `.codex`
- `Codex 状态库`
- `项目业务目录`
- `audit event`
- `临时文件`
- `原子替换`

## Validation-design mismatch

Original expected action: click `取消`.

Actual result: the user clicked `确认执行` after reading the dialog. The user later clarified this was not a pure accidental click and not caused by the validation coordinate-click command. The user did not understand why this dialog appeared, read it, and then naturally confirmed it.

Observed after the click:

- Notice changed to: `已在用户确认后首次初始化工作流事实层；此前无旧状态文件可备份。`
- Panel changed to:
  - `已初始化`
  - `exists=true`
  - `schema version workflow_state_v0`
  - `workflow version 1`
  - `audit events 1`
- Existence check for the real state path returned present.

This means the task's "不点击确认执行" and "不创建真实 workflow-state.v0.json" requirements were not met. However, this is now recorded as a validation-task design mismatch rather than a product failure or a pure misclick.

## Recovery and cleanup

Because the file was created during this validation attempt and the task did not intend to keep a real state file, the single created state file was deleted:

```bash
rm '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

Post-recovery checks:

- Existence check for the real state path returned absent.
- `lsof -nP -iTCP:5173 -sTCP:LISTEN` returned no listener.
- Narrow filtered process check for `codex-governance-workbench|cargo-tauri dev|vite --host 127.0.0.1` returned no match.

Cleaned PIDs identified from this run:

- `37150` npm `tauri:dev`
- `37174` `cargo-tauri dev`
- `37308` npm `dev`
- `37339` Vite node server
- `37340` esbuild helper
- `37367` `codex-governance-workbench`

## Prohibited items status

- Did click `确认执行`: yes. User clarification: this was not a pure accidental click and not caused by the validation coordinate-click command; the user did not understand why the dialog appeared and naturally confirmed after reading it.
- Did create real `workflow-state.v0.json`: yes, during this validation attempt.
- Did remove the accidentally created file afterward: yes.
- Read real workflow-state file content: no.
- Wrote `/Users/yoyi/.codex`: no evidence from performed commands.
- Modified Codex state DB: no evidence from performed commands.
- Wrote project business directories: no evidence from performed commands.
- Read or displayed secrets/auth/session bodies/tool outputs/input history/memory bodies: no.
- Ran harness: no.
- Pulled network dependencies: no.

## Risk

- The environment was restored to "state file absent", but this run should not be used as a clean pass for the original observation-only task.
- The larger issue is not button misclick prevention. The user needs to understand why the workbench asks to initialize its own state file before the first write.
- Next task should not be write-confirmation hardening. It should either clarify the onboarding/explanation around workflow-state initialization or proceed with workflow features once the initialization purpose is accepted.
