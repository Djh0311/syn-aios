# 桌面壳工作流事实层 v0 最小读写交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`

## 结论

建议接受为“工作流事实层 v0 最小读写已实现”。

不建议接受为“可编辑工作流完成”。依据：本轮只做状态文件读取、用户确认初始化、备份、audit、原子替换和前端状态展示；没有做节点编辑、边编辑、工作项状态转换、review 登记或接受候选。

## 做了什么

- Rust 后端新增读取工作流状态文件命令。
- Rust 后端新增初始化工作流状态文件命令。
- 缺状态文件时返回空状态和 `exists=false`，不自动创建。
- 初始化写入最小 v0 JSON。
- 初始化写入包含 audit event。
- 旧状态文件存在时先备份到 `backups`。
- 写入使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 前端新增本地事实层状态面板。
- 前端初始化动作必须走确认弹层。
- 更新前端离线测试和 Rust 单测。

## 改了哪些文件

- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `product-line/prototypes/productized-desktop-shell/src/components/WorkflowStatePanel.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/handoffs/2026-05-28-desktop-shell-workflow-state-v0-result.md`

## 新增或修改了哪些测试

Rust 新增：

- `missing_workflow_state_returns_empty_without_creating_file`
- `initializes_workflow_state_with_audit_event`
- `existing_workflow_state_is_backed_up_before_initialize`

前端离线测试新增：

- 本地事实层面板展示未初始化状态。
- 初始化按钮生成待确认动作。
- 确认弹层展示写入边界、目标路径、`backups`、audit event、原子替换说明。

## v0 状态文件路径如何计算

Rust 后端使用 `$HOME` 拼接应用数据目录：

```text
$HOME/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

当前用户环境下是：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

备份路径：

```text
$HOME/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json
```

## 状态文件不存在时如何处理

后端 `load_workflow_state_snapshot`：

- 返回 `exists=false`。
- 返回 `initialized=false`。
- 所有对象数量为 0。
- 返回状态文件路径。
- 返回 warning：“状态文件不存在；不会自动创建。”
- 不创建目录，不创建文件。

## 初始化确认流程

前端：

- 项目页顶部显示“本地事实层 v0”面板。
- 用户点击“初始化工作流事实层”。
- 前端生成 `initialize-workflow-state` 待确认动作。
- 确认弹层显示目标路径、路径来源和写入边界。
- 用户确认后才调用 Rust 命令 `initialize_workflow_state`。
- 初始化成功后用返回 snapshot 更新界面。

## 写入、备份、audit、原子替换如何实现

Rust 初始化流程：

- 创建 workflow-state 目录。
- 如果目标状态文件存在，先复制到 `backups/workflow-state.v0.<timestamp>.json`。
- 如果目标状态文件不存在，audit event 记录 `missing_state_no_backup`。
- 生成最小 v0 JSON。
- 做最小 schema 校验。
- 写 `.workflow-state.v0.<timestamp>.tmp` 临时文件。
- `sync_all`。
- `fs::rename` 替换目标文件。
- 重新读取目标文件并校验 `schema_version` 和 `workflow_version`。

最小 v0 JSON 包含：

- `schema_version = workflow_state_v0`
- `workflow_version = 1`
- `workspace_id`
- `projects`
- `agent_adapters`
- `workflows`
- `nodes`
- `edges`
- `work_items`
- `artifacts`
- `reviews`
- `audit_events`
- `capabilities`
- `harness_resources`

## 前端如何展示本地事实层状态

新增 `WorkflowStatePanel`，展示：

- 是否存在状态文件。
- 状态文件路径。
- schema version。
- workflow version。
- workflows / nodes / edges / reviews / audit events / harness resources 数量。
- 未初始化 warning。
- 重新读取按钮。
- 初始化工作流事实层按钮。

## 是否创建了真实状态文件

没有。

依据：

- 本轮没有启动 Tauri 窗口点击初始化按钮。
- 验证命令 `test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'` 返回不存在。

只在 Rust 单测中使用临时目录夹具创建和备份测试文件。

## 是否触碰禁止事项

未触碰。

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未自动创建真实状态文件。
- 未绕过用户确认写状态文件。
- 未自动运行 harness。
- 未把索引候选自动升级成本地事实。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度。
- 未做 release 打包。

## 验证命令和结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

真实状态文件检查：

- `test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'`
- 结果：不存在。

端口检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN`
- 结果：无监听输出。

## 风险和下一步建议

- 当前 schema 校验仍是最小手写校验，后续需要更严格的字段和引用校验。
- 当前没有文件锁，多窗口并发写入需要后续处理。
- `workspace_id` 当前按决策示例固定为 `workspace:yoyi-workspace`，后续建议实现稳定 hash。
- 下一步可以做候选接受流程，但必须继续用户确认、audit、备份、原子替换，不能把索引候选自动变成本地事实。
