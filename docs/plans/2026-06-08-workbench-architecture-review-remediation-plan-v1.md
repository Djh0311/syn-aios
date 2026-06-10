# Workbench Architecture Review Remediation Plan v1

日期：2026-06-08

状态：架构复核报告修补计划。本文不改变权威入口、不声明中间版本完成；用于把独立只读架构 / 代码复核报告中的 P0/P1/P2 发现转成可执行修补边界。

## 1. 复核报告结论摘要

独立复核报告接受当前代码已有安全底座，但指出三个核心缺口：

- 真实执行入口仍有两套路径：H5 bridge 是 preview-only；旧 `execute_workflow_node_dispatch`、`run_workflow_machine` 和 `__run_workflow_machine_real` 仍可进入真实 Codex runner。
- UI / 权限弹层存在误导或过期口径：旧派发显示成 “H5 命令”，真实多轮执行确认按钮仍叫 “允许一次”，弹层仍出现 “本轮 H6 开发线未授权实际确认执行”。
- 产品 UI 仍偏内部状态面板：readback unknown 显示为 “空”，秘书文案暗示派任务 / 命令，开发者信息未归位。

## 2. 本轮修补范围

本轮只做低风险收口：

- 写清修补计划。
- 修正误导性 UI 文案。
- 把旧真实执行入口在 UI 上标识为旧项目派发 / 工作流机器真实执行入口，不再包装为 H5 产品命令。
- 修正 readback unknown 文案。
- 配合 UI 信息架构线把开发者内容移入设置。

本轮不做：

- 不重写 Rust runner。
- 不删除 `execute_workflow_node_dispatch`。
- 不删除 `run_workflow_machine`。
- 不删除 `__run_workflow_machine_real`。
- 不改变 workflow state schema。
- 不改变 sidecar schema。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。

## 3. 需要主管决策的事项

以下不在本轮直接做，需要另开任务包或主管明确决策：

- 是否废弃、封存或保留 `__run_workflow_machine_real` CLI 入口。
- 是否把旧 `execute_workflow_node_dispatch` / `run_workflow_machine` 合并进统一 H5 product command routing。
- 是否要求所有真实执行必须经过同一套 H5 permission envelope、continuation、runtime log、audit、readback 契约。
- 是否允许下一轮执行 H3-B retry、H4-Level-B 真实失败 / 超时探针或新的 H5 通用真实项目派发。

## 4. 本轮直接修补清单

### 4.1 权限弹层文案

文件：

- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`

修补：

- 去掉 “本轮 H6 开发线未授权实际确认执行” 这种阶段内历史文案。
- 对真实 Codex 操作显示更明确的确认按钮。
- `run-workflow-machine` 不再用 “允许一次”，改成能表达多轮真实执行的文案。

### 4.2 项目页真实执行摘要

文件：

- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

修补：

- `H5 命令` 改为 `旧派发入口` 或等价文案。
- 明确当前显示的是旧项目工作流派发记录，不代表 H5 formal bridge 已成为唯一产品命令。
- 真实派发按钮文案补上 “真实” 和 “需确认” 语义。

### 4.3 readback unknown

文件：

- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`

修补：

- `result_count = null` 显示为 `未知 / 不可用`。
- 不再显示为 `空`。

### 4.4 UI 信息层级

文件：

- `prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/HomeView.tsx`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`

修补：

- 由 UI Shell 多会话开发线执行。
- 主入口改为项目、智能体、Skill、Harness、运行中工作流。
- 开发 / 内部内容进入 `设置 > 开发者`。

## 5. 验收

推荐命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

如涉及 Rust 真实执行入口封存或 routing 合并，另行追加：

```text
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check ...
```

本轮若只改前端文案和显示层，不需要执行真实 Codex，也不能把验证结果冒充真实 Tauri / 截图验收。
