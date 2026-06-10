# 任务包：任务包草稿登记 v1

## 任务名

在产品化桌面壳中，为已有项目工作流登记一个任务包草稿。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及 Tauri 后端状态文件写入、前端项目详情页交互和工作流事实层更新。

## 背景

阶段 3 目标要求工作台能表达 Director、Codex 会话、任务包、handoff、evidence、review 的流转，并能输出标准任务包、登记回收结果。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 明确要求“在工作台里登记开发线、生成任务包、回收结果摘要”，并能“输出标准任务包，登记回收结果和总指导判断”。
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`：已定义 `work_items[]` 和 `artifacts[]`。
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`：v0 状态文件顶层包含 `work_items`、`artifacts`、`reviews`、`audit_events`。
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`：项目默认工作流草稿初始化 v1 已回收。
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke-review.md`：真实 Tauri 窗口已能为 `agent world` 创建默认工作流草稿，工作台状态文件当前已存在。

当前缺口：

- 工作流里已有任务包节点，但还没有真正的 `work_items[]` 任务草稿记录。
- 还没有 `artifacts[]` 任务包草稿引用。
- 还不能登记“这条工作流下一步要派发什么任务”。

## 目标

实现最小可用的“任务包草稿登记”。

用户在已有项目工作流下，可以创建一个任务包草稿。草稿只写入工作台自己的状态文件：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

至少写入：

- 一个 `work_items[]` 记录。
- 一个 `artifacts[]` 记录，`artifact_type=task_package` 或等价值。
- 一条 `audit_events[]` 记录。
- 必要时更新相关 task 节点状态为 `draft` 或保持原状态；如果更新节点，必须记录 audit。

前端至少提供：

- 在项目详情页或工作流草稿区展示当前项目的任务草稿数量。
- 一个“创建任务包草稿”入口。
- 最小输入：
  - 标题
  - 目标说明 / 简短描述
  - 指派角色，默认 Codex 开发线
- 创建前确认弹层，说明只登记到工作台状态文件，不生成真实任务包 markdown 文件，不派发真实 Codex 会话。
- 创建后 UI 显示任务草稿标题、状态和 artifact 类型。

## 不做

- 不生成真实 `product-line/tasks/*.md` 任务包文件。
- 不自动派发给真实 Codex 会话。
- 不启动 Codex CLI。
- 不读取或展示 Codex 会话正文、工具输出、输入历史。
- 不登记真实 handoff / evidence / review。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不做完整状态机。
- 不运行 harness。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-project-workflow-bootstrap-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-project-workflow-bootstrap-v1-tauri-smoke-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许做存在性检查，不读取内容：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

如实现需要读取工作台自己的状态文件，请只通过已有 Rust 后端读取，不在前端直接读文件；不要在 evidence / handoff 中打印完整状态文件正文。

## 允许写入

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`
- `product-line/evidence/`
- `product-line/handoffs/`

真实运行时允许由 Tauri 后端在用户确认后写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不生成真实 `product-line/tasks/*.md` 任务包文件。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不把索引候选自动升级成“已验证能力”。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、LM 调度。
- 不做 release 打包。
- 不拉取外网依赖。
- 不实现自动编排执行。

## 实现建议

后端：

- 新增 Tauri 命令，例如 `create_task_draft`。
- 输入只接受：
  - project root 或 workflow id
  - title
  - brief / objective
  - assigned role，默认 `codex-dev`
- 后端从工作台状态文件确认项目 workflow 已存在。
- 不允许为没有 workflow 的项目创建任务草稿。
- 写入前备份已有状态文件。
- 写入使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 任务草稿 id 可用稳定前缀加时间戳，例如 `work-item:<workflow-id>:<timestamp>`。
- artifact id 与 work item 关联。
- 避免重复提交：可以用 title + workflow id + created_at 或显式 id 去重；本轮至少要防止同一次提交重复写入。

前端：

- 在项目工作流草稿区增加任务草稿摘要。
- 如果项目没有 workflow，提示先创建默认工作流草稿。
- 有 workflow 时显示创建任务包草稿入口。
- 输入尽量少，先只要标题和目标说明。
- 创建前走确认弹层。
- 创建后刷新 workflow state snapshot。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 没有 workflow 时拒绝创建任务草稿。
  - 有 workflow 时创建 `work_items[]` 和 `artifacts[]`。
  - 写入 audit event。
  - 非索引项目或不匹配 workflow 被拒绝。
  - 写入前备份旧状态文件。
- 前端离线测试覆盖：
  - 有 workflow 时显示创建任务包草稿入口。
  - 无 workflow 时提示先创建默认工作流。
  - 取消确认不会调用创建动作。
  - 创建确认弹层显示“不生成真实任务包文件、不派发真实 Codex 会话”。
- evidence / handoff 说明真实状态文件任务前后是否存在。
- 如果真实 Tauri 窗口不验证创建动作，要明确写出。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 写入了哪些 `work_items[]` / `artifacts[]` 字段
6. 如何保证只在已有项目 workflow 下创建
7. 如何避免生成真实任务包文件或派发真实 Codex 会话
8. 验证命令和结果
9. 真实状态文件任务前后是否存在
10. 是否触碰禁止事项
11. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只实现任务包草稿登记，没有生成真实任务包文件。
- 是否没有自动派发真实 Codex 会话。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否不读取会话正文或敏感内容。
- 是否正确写入 `work_items[]`、`artifacts[]`、`audit_events[]`。
- 是否适合下一步做任务包 markdown 生成或节点状态流转。
