# Handoff: project workflow bootstrap v1 Tauri smoke

## Verdict

接受为“真实 Tauri 窗口创建项目默认工作流草稿 smoke 通过”。

不接受为“自动编排执行完成”、不接受为“任务包生成完成”、不接受为“节点状态流转完成”。

依据：真实 Tauri 窗口中为索引内项目 `agent world` 打开确认弹层并点击 `确认执行`，UI 刷新后显示 `state=draft`、`nodes=7`、`edges=6`，真实工作台状态文件存在。

## What was done

- 读取任务包和上游 evidence / handoff / review。
- 复跑前端和 Rust 验证命令。
- 清理前置残留 dev listener。
- 启动真实 Tauri dev 窗口。
- 进入项目页，选中 `agent world`。
- 打开 `创建默认工作流草稿` 确认弹层。
- 核对确认弹层的目标、来源和写入边界。
- 点击 `确认执行`。
- 验证 UI 刷新后的 workflow id、state、nodes、edges。
- 复核真实状态文件存在并保留。
- 停止 Tauri/Vite/cargo-tauri，确认 `5173` 无监听残留。

## Files read

- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-result.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`
- `product-line/prototypes/productized-desktop-shell/`

Real state file content was not read.

## Evidence

- `product-line/evidence/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke.md`

## Validation commands

Passed:

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

Rust tests: `10 passed; 0 failed`.

## Real Tauri observations

Window:

- title: `Codex 治理工作台`
- normal size: `1280 x 820`
- home notice: `已读取索引。所有本机动作仍需用户点击并确认。`

Selected project:

- `agent world`
- `/Users/yoyi/gameai/agent world`

Before creation:

- `项目工作流草稿`
- `当前项目还没有本地工作流草稿`
- `workflow 未创建`
- `state 未创建`
- `nodes 0`
- workflow-state panel showed missing state file.

Confirmation dialog:

- `创建项目默认工作流草稿`
- target path was the selected indexed project path.
- source was `索引内项目路径`.
- boundary said it writes workbench-owned `workflow-state.v0.json`, project / workflow / default nodes / default edges / audit.
- boundary said it does not write `.codex`, Codex state DB, or project business directory.
- page text said it does not dispatch a real Codex session and does not generate task package files.

After creation:

- `当前项目已有本地工作流草稿`
- `workflow:users-yoyi-gameai-agent-world:default`
- `state draft`
- `nodes 7`
- `edges 6`

## State file

Before task:

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json` did not exist.

After task:

- the same path exists.
- file was kept, as required by the task package.

## Cleanup

Cleaned:

- Tauri dev session
- Vite listener
- Tauri app process

Post-cleanup:

- `5173` has no listener.
- `codex-governance-workbench` has no running process.

## Boundary check

No evidence of:

- direct script write to the real state file
- backend command bypass outside UI
- writing `/Users/yoyi/.codex`
- modifying Codex state DB
- writing project business directories
- reading secrets or session bodies
- running harness
- turning candidates into verified capabilities
- non-Codex agent integration
- release packaging or network dependency pull
- automatic orchestration execution

## Risk

- Only one project was created through real UI.
- Duplicate-create behavior remains covered by Rust tests, not by this smoke.
- The created workflow is a draft skeleton, not execution history.
