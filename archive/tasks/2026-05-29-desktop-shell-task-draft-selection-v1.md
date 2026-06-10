# 任务包：任务草稿选择态 v1

## 任务名

在产品化桌面壳中，让任务列表、Markdown 预览、字段编辑表单共享同一个明确选中的任务草稿。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及前端任务草稿选择状态、预览渲染、字段编辑表单绑定和少量类型调整。

## 背景

阶段 3 目标要求工作台能在项目级可视化工作流里登记开发线、生成任务包、回收结果摘要。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 明确要求“在工作台里登记开发线、生成任务包、回收结果摘要”，并能“输出标准任务包，登记回收结果和总指导判断”。
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-review.md`：任务包字段编辑 v1 已接受，但薄弱点是多任务草稿时字段编辑表单绑定第一个草稿。
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`：任务包 Markdown 预览 v1 已接受，但还没有统一选择态。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`：任务包草稿登记 v1 已接受，后续会出现多个 task draft。

当前缺口：

- 任务列表、Markdown 预览、字段编辑表单不是同一个明确选择态。
- 多任务草稿时，字段编辑表单当前绑定第一个草稿。
- 如果直接做真实任务包文件生成，可能生成或编辑错草稿。

## 目标

实现“任务草稿选择态 v1”。

用户在项目详情页中能明确选择一个任务草稿。选中后：

- 任务列表显示当前选中项。
- Markdown 预览渲染当前选中项。
- 字段编辑表单绑定当前选中项。
- 保存字段只更新当前选中项。
- 复制预览只复制当前选中项。
- 后续生成真实任务文件的入口可以安全依赖同一个 `selected_work_item_id`。

如果没有任务草稿：

- 不显示假选择。
- 显示“先创建任务包草稿”的下一步提示。

如果当前选中草稿被删除或状态刷新后不存在：

- 自动选择第一个可用草稿，或清空选择并提示用户重新选择。
- 不能静默继续编辑旧 id。

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
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-draft-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-markdown-preview-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端按现有命令读取工作台自己的状态文件：

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

本任务原则上只改 UI 选择态。除非现有类型缺字段，否则不应新增真实状态写入能力。

真实运行时如用户保存字段，仍允许沿用已有 Tauri 后端命令写入：

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

前端：

- 在项目 workflow / task draft 区新增 `selectedWorkItemId` 状态。
- 有草稿时默认选择第一个草稿，但 UI 必须明确显示“当前选中”。
- 任务草稿列表中的每个草稿提供“选择”或整行选择能力。
- 预览按钮和字段编辑表单都使用 `selectedWorkItemId`。
- 字段编辑表单的默认值应来自当前选中的草稿及其 task package artifact。
- 切换草稿时刷新 Markdown 预览和字段表单。
- 保存字段确认弹层显示当前选中任务名和 `work_item_id`。
- 如果没有选择，不允许保存字段、复制预览或后续生成任务文件。

后端：

- 优先不新增命令。
- 如当前 snapshot 缺少表单所需 artifact 字段，可以扩展只读 snapshot 映射，但不得读取会话正文或敏感内容。
- 不新增真实任务文件生成能力。

状态：

- 选择态可以先保存在前端组件 state，不必写入工作台状态文件。
- 如果后续要持久化最近选中任务，应另开任务。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 前端离线测试覆盖：
  - 有多个 task draft 时，能显示明确选中项。
  - 切换选中项后，预览入口使用新 `work_item_id`。
  - 切换选中项后，字段编辑表单绑定新草稿。
  - 保存字段确认弹层显示当前选中任务，不是第一个任务。
  - 没有任务草稿时，保存字段和预览动作不可用或显示明确提示。
- 如果改了 Rust 类型或映射，Rust 测试必须继续覆盖不读取正文、不生成真实任务文件。
- evidence / handoff 明确说明没有做真实窗口验证。
- evidence / handoff 明确说明没有生成真实 `product-line/tasks/*.md`。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 选择态如何存储
6. 多任务草稿时如何保证预览、编辑和复制使用同一个 `work_item_id`
7. 没有任务草稿时如何处理
8. 是否新增后端命令或状态字段
9. 如何保证不生成真实任务包文件
10. 如何保证不派发真实 Codex 会话
11. 验证命令和结果
12. 是否读取或打印真实状态文件正文
13. 是否触碰禁止事项
14. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否修复了“字段编辑表单绑定第一个草稿”的问题。
- 多任务草稿时是否能明确选择目标草稿。
- Markdown 预览、字段编辑、复制预览是否都使用同一个选中草稿。
- 是否没有生成真实任务包文件。
- 是否没有自动派发真实 Codex 会话。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否适合下一步做“确认后生成真实任务包文件 v1”。
