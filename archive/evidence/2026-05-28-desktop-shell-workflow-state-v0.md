# 桌面壳工作流事实层 v0 最小读写证据

## 结论

薄弱点先说：

- 这轮实现的是 v0 JSON 最小读写底座，不是可编辑工作流。依据：没有做节点编辑、边编辑、工作项状态转换、review 登记或接受 harness 候选。
- v0 JSON 不是最终事实库形态。依据：存储决策明确长期仍可迁移 SQLite。
- 真实状态文件没有被本轮创建。依据：本轮没有点击初始化动作，且 `test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'` 返回不存在。
- 索引候选仍没有自动升级成本地事实。依据：初始化 JSON 中 `projects`、`workflows`、`nodes`、`edges`、`work_items`、`artifacts`、`reviews`、`capabilities`、`harness_resources` 都为空数组。

可接受点：

- Rust 后端实现了读取 v0 状态文件。
- 状态文件不存在时返回 `exists=false` 和空计数，不自动创建。
- Rust 后端实现了用户确认后可调用的初始化写入命令。
- 初始化 JSON 包含 v0 顶层 schema 字段。
- 初始化写入包含 audit event。
- 已存在状态文件时写入前会备份到 `backups`。
- 写入使用临时文件 + `rename` 原子替换。
- 写入后会重新读取校验。
- 前端显示状态文件路径、存在状态、schema version、workflow version、对象数量和未初始化状态。
- 前端初始化动作走确认弹层，确认文案显示目标路径和写入边界。

## 本轮读取依据

- `product-line/tasks/2026-05-28-desktop-shell-workflow-state-v0.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-review.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- `product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/prototypes/productized-desktop-shell/`

没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文、工具输出、命令输出、输入历史、记忆正文

## 修改文件

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

## v0 状态文件路径如何计算

Rust 后端使用：

```text
$HOME/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

当前用户环境下对应：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

备份路径：

```text
$HOME/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json
```

## 状态文件不存在时如何处理

`load_workflow_state_snapshot`：

- 检查路径是否存在。
- 不存在时返回 `exists=false`。
- `initialized=false`。
- 所有对象数量为 0。
- warnings 包含“状态文件不存在；不会自动创建。”
- 不创建目录，不创建文件。

Rust 单测覆盖：

- `missing_workflow_state_returns_empty_without_creating_file`

## 初始化确认流程

前端项目页显示“本地事实层 v0”面板。

用户点击“初始化工作流事实层”时：

- 前端只创建待确认动作。
- 确认弹层显示目标路径。
- 确认弹层显示路径来源：Tauri 应用数据目录。
- 确认弹层显示写入边界：只写 `workflow-state.v0.json` 和同目录 `backups`；不写 `.codex`、不写 Codex 状态库、不写项目业务目录。
- 用户确认后才调用 `initialize_workflow_state`。

本轮没有实际点击这个动作，所以没有创建真实状态文件。

## 写入、备份、audit、原子替换

初始化写入：

- 创建状态目录。
- 如果旧状态文件存在，先复制到 `backups/workflow-state.v0.<timestamp>.json`。
- 如果旧状态文件不存在，audit event 的 `before_state` 写 `missing_state_no_backup`。
- 生成最小 v0 JSON。
- 校验 schema_version、workflow_version 和顶层数组字段。
- 写入临时文件 `.workflow-state.v0.<timestamp>.tmp`。
- `sync_all` 后使用 `fs::rename` 替换目标文件。
- 写入后重新读取并校验 schema/version。

audit event：

- `event_type = workflow_state_initialized`
- `actor_ref = user_confirmed_desktop_shell`
- `permission_level = user_confirmed_write`
- 包含首次初始化无旧文件或旧文件已备份的 reason。

## 前端如何展示

新增 `WorkflowStatePanel`：

- 状态文件路径。
- `exists=true/false`。
- schema version。
- workflow version。
- workflows 数量。
- nodes 数量。
- edges 数量。
- reviews 数量。
- audit events 数量。
- harness resources 数量。
- warning / 未初始化提示。

面板放在项目页顶部，没有新增主导航入口。

## 测试

Rust 新增测试：

- `missing_workflow_state_returns_empty_without_creating_file`
- `initializes_workflow_state_with_audit_event`
- `existing_workflow_state_is_backed_up_before_initialize`

前端离线交互测试新增覆盖：

- 事实层面板显示 `exists=false`、schema / workflow version、workflows / nodes / edges / reviews / audit events。
- 初始化按钮只产生待确认动作。
- 确认弹层显示写入边界、`workflow-state.v0.json`、`backups`、不写 `.codex`、audit event、原子替换。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

真实状态文件检查：

- `test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'`
- 返回不存在。

端口检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 禁止事项检查

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未写项目业务目录。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动创建真实状态文件。
- 未绕过用户确认写状态文件。
- 未自动运行 harness。
- 未把索引候选自动升级成本地事实。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度。
- 未做 release 打包。

## 风险

- 当前 schema 校验是最小手写校验，不是完整 JSON Schema。
- 当前没有文件锁；多窗口并发写入仍需要后续处理。
- `workspace_id` 当前按决策示例固定为 `workspace:yoyi-workspace`，后续应换成稳定 hash 规则。
- 初始化会覆盖旧状态文件，但覆盖前会备份；后续需要更细的迁移和冲突处理。
