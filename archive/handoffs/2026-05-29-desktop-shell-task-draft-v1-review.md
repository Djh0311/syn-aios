# 任务包草稿登记 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-task-draft-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“任务包草稿登记 v1 已实现”。

不接受为“真实任务包文件生成完成”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口创建任务草稿已验证”。

依据：

- 后端新增 `create_task_draft` 命令。
- 后端先确认项目存在于当前 `codex-index.json`。
- 后端要求工作台状态文件里已有项目默认 workflow；没有 workflow 会拒绝创建。
- 后端写入 `work_items[]`、`artifacts[]` 和 `audit_events[]`。
- `artifacts[].artifact_type = task_package`，但 `path = null`，表示没有生成真实 markdown 文件。
- 前端项目页显示任务草稿数量、创建入口、标题和目标说明输入。
- 前端确认弹层明确说明不生成真实任务包文件、不派发真实 Codex 会话。
- 总指导线复跑验证通过。

## 先说薄弱点

- 本轮没有打开真实 Tauri 窗口点击创建任务草稿，所以不能说真实窗口写入链路通过。
- 前端离线测试只检查组件和回调，不检查真实浏览器布局。
- 任务草稿只是工作台状态里的 draft，不是 `product-line/tasks/*.md` 文件。
- 没有真实 Codex 会话派发，没有 Codex CLI 调用，没有 harness。
- 重复保护目前是同 workflow 下的 trimmed title，后续如果支持编辑或并发提交，需要更明确的幂等键。

## 接受内容

接受后端能力：

- `create_task_draft` 命令。
- 非索引项目拒绝。
- 缺状态文件拒绝。
- 项目没有默认 workflow 时拒绝。
- 有 workflow 时创建 `work_items[]` 和 `artifacts[]`。
- 写入 audit event。
- 写入前备份已有状态文件。
- 原子写入。
- 同 workflow 同标题不重复写入。

接受前端能力：

- 项目 workflow 区显示任务草稿数量。
- 有 workflow 时显示创建任务包草稿表单。
- 无 workflow 时提示先创建默认工作流草稿。
- 表单包含标题、目标说明和默认指派 `codex-dev`。
- 创建前走确认弹层。
- 创建后可以刷新 workflow state snapshot。
- 已有任务草稿列表显示标题、状态和 artifact 类型。

## 写入字段

`work_items[]` 写入：

- `work_item_id`
- `project_id`
- `workflow_id`
- `title`
- `state=draft`
- `source_kind=workspace_state`
- `source_ref`
- `assigned_role_id`
- `agent_type=codex`
- `adapter_id=codex-local`
- `permission_level=user_confirmed_write`
- `created_at`
- `updated_at`

`artifacts[]` 写入：

- `artifact_id`
- `artifact_type=task_package`
- `project_id`
- `path=null`
- `title`
- `brief`
- `source_kind=workspace_state`
- `source_ref`
- `permission_level=user_confirmed_write`
- `created_at`
- `updated_at`
- `warnings=["draft_only_no_markdown_file"]`

`audit_events[]` 写入：

- `event_type=task_draft_created`
- 如果同步任务节点状态，也会追加对应 audit event。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 16 个 Rust 单测通过。

真实状态文件复核：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 0。
- 判断为真实工作台状态文件存在。
- 总指导线没有读取状态文件正文。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 返回码 1。
- 无输出。
- 判断为 5173 当前无监听残留。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有生成真实 `product-line/tasks/*.md`。
- 没有启动 Codex CLI。
- 没有派发真实 Codex 会话。
- 没有运行 harness。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有写项目业务目录。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有知识库、向量搜索、LM 调度、release 打包或网络依赖拉取。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳能在已有项目 workflow 下登记任务包草稿。
- 任务包草稿写入工作台自己的状态文件。
- UI 能展示任务草稿数量、创建入口和草稿列表。

仍不能说：

- 真实任务包 markdown 文件生成完成。
- 真实 Tauri 窗口创建任务草稿链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议派验证线做“任务包草稿真实窗口创建 smoke”。

目标：

- 在真实 Tauri 窗口里为已有项目 workflow 创建一个任务包草稿。
- 验证 UI 显示任务草稿数量和草稿列表。
- 保留工作台状态文件。
- 不生成真实 markdown 任务包文件。
- 不派发真实 Codex 会话。
