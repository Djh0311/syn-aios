# 任务包：任务包字段编辑 v1

## 任务名

在产品化桌面壳中，为任务包草稿补齐标准任务包字段编辑能力。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及 Tauri 后端状态文件写入、前端字段表单、任务包预览刷新和工作流事实层更新。

## 背景

阶段 3 目标要求工作台能在项目级可视化工作流里登记开发线、生成任务包、回收结果摘要。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 明确要求“在工作台里登记开发线、生成任务包、回收结果摘要”，并能“输出标准任务包，登记回收结果和总指导判断”。
- `product-line/TASK_TEMPLATE.md`：当前标准任务包字段入口。
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`：已定义 `work_items[]` 和 `artifacts[]`。
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`：v0 状态文件顶层包含 `work_items`、`artifacts`、`reviews`、`audit_events`。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`：任务包草稿登记 v1 已接受，但只登记了最小标题和目标说明。
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`：任务包 Markdown 预览 v1 已接受，但预览里很多字段只能显示“待补充”或“未登记”。

当前缺口：

- 任务草稿字段太少，直接生成真实任务包文件会留下大量“待补充”。
- 工作台还不能编辑标准任务包字段。
- Markdown 预览现在只是结果展示，不能作为事实来源。

## 目标

实现“任务包字段编辑 v1”。

用户在项目详情页中选择已有任务包草稿后，可以编辑标准任务包字段。保存后写入工作台自己的状态文件，并刷新 Markdown 预览。

字段编辑必须以结构化字段作为事实，不把一整段 Markdown 当唯一事实。

至少支持编辑：

- 任务名
- 所属开发线 / 指派角色
- 背景
- 目标
- 允许读取
- 允许写入
- 禁止事项
- 验收标准
- 必须回传
- 总指导回收重点

建议存储位置：

- `work_items[]` 继续保存任务标题、状态、指派角色、agent 信息和更新时间。
- `artifacts[]` 中对应 `artifact_type=task_package` 的记录保存任务包结构化字段。
- `artifacts[].path` 继续保持 `null`，表示没有生成真实任务文件。
- `audit_events[]` 追加字段更新事件。

建议结构化字段名：

- `task_name`
- `assigned_line`
- `background`
- `goals`
- `allowed_read`
- `allowed_write`
- `forbidden_actions`
- `acceptance_criteria`
- `required_return`
- `review_focus`
- `template_version=task_package_v1`

如果代码已有等价字段名，可以沿用，但必须在 evidence / handoff 里说明映射关系。

## 不做

- 不生成真实 `product-line/tasks/*.md` 任务包文件。
- 不自动派发给真实 Codex 会话。
- 不启动 Codex CLI。
- 不登记真实 handoff / evidence / review。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不做完整状态机。
- 不运行 harness。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做真实 Tauri 窗口 smoke；该类验证按用户要求暂时暂停。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/TASK_TEMPLATE.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端读取工作台自己的状态文件以加载和保存任务包草稿字段：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

约束：

- evidence / handoff 不得打印完整真实状态文件正文。
- 如需测试状态内容，使用临时目录夹具。

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
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。
- 不拉取外网依赖。
- 不实现自动编排执行。
- 不做真实窗口验证作为本任务验收前置条件。

## 实现建议

后端：

- 新增 Tauri 命令，例如 `update_task_package_draft_fields`。
- 输入必须包含：
  - `project_root`
  - `work_item_id`
  - 标准任务包字段
- 从索引确认 project root 是索引内项目。
- 从工作台状态确认项目 workflow、work item 和 `task_package` artifact 存在。
- 只更新当前 workflow 下对应 work item 的任务包字段。
- 保存前备份已有状态文件。
- 使用临时文件 + 原子替换。
- 写入后重新读取校验。
- 追加 audit event，例如 `task_package_fields_updated`。
- 更新 `work_items[].title`、`work_items[].assigned_role_id`、`work_items[].updated_at`。
- 更新 `artifacts[]` 对应 task package 的结构化字段、`updated_at` 和 warnings。
- 如果字段为空，保留空值或占位 warning，但不要补编业务。

前端：

- 在任务包 Markdown 预览区旁边或下方增加“编辑字段”入口。
- 表单按标准任务包栏目分组。
- 列表字段可以先用多行文本，每行一项。
- 保存前走确认弹层，明确写入工作台自己的状态文件，不生成真实任务包文件，不派发 Codex。
- 保存后刷新 workflow state snapshot 和 Markdown 预览。
- 如果项目没有 workflow、没有 task draft 或没有预览目标，显示下一步提示。

预览：

- `render_task_package_preview` 应优先读取结构化字段。
- 缺字段继续显示“待补充”或“未登记”。
- 不把预览 Markdown 反向解析成事实。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 非索引项目拒绝更新。
  - 缺状态文件拒绝更新。
  - 缺 workflow 拒绝更新。
  - 缺 work item 拒绝更新。
  - 缺 `task_package` artifact 拒绝更新。
  - 有任务草稿时更新结构化字段。
  - 更新后 Markdown 预览使用新字段。
  - 写入前备份旧状态文件。
  - 写入 audit event。
  - 空字段不被补编成假事实。
- 前端离线测试覆盖：
  - 有任务草稿时显示字段编辑入口。
  - 表单包含标准任务包字段。
  - 保存前显示确认弹层。
  - 取消确认不会调用保存。
  - 保存确认文案说明“不生成真实任务文件、不派发真实 Codex 会话”。
- evidence / handoff 明确说明没有做真实窗口验证。
- evidence / handoff 明确说明没有生成真实 `product-line/tasks/*.md`。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 新增或更新了哪些结构化字段
6. 字段存储在 `work_items[]` 还是 `artifacts[]`
7. 空字段如何处理
8. 如何保证不生成真实任务包文件
9. 如何保证不派发真实 Codex 会话
10. 验证命令和结果
11. 是否读取或打印真实状态文件正文
12. 是否触碰禁止事项
13. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只编辑工作台状态里的任务包草稿字段。
- 是否没有生成真实任务包文件。
- 是否没有自动派发真实 Codex 会话。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否没有读取会话正文或敏感内容。
- Markdown 预览是否已经能使用编辑后的结构化字段。
- 是否适合下一步做“确认后生成真实任务包文件 v1”。
