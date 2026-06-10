# 任务包：任务包 Markdown 预览 v1

## 任务名

在产品化桌面壳中，把已有任务包草稿渲染成标准任务包 Markdown 预览。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及工作流状态读取、任务草稿展示、前端预览界面和 Tauri 后端辅助渲染。

## 背景

阶段 3 目标要求工作台能在项目级可视化工作流里登记开发线、生成任务包、回收结果摘要。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 明确要求“在工作台里登记开发线、生成任务包、回收结果摘要”，并能“输出标准任务包，登记回收结果和总指导判断”。
- `product-line/TASK_TEMPLATE.md`：当前标准任务包字段入口。
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`：已定义 `work_items[]` 和 `artifacts[]`。
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`：v0 状态文件顶层包含 `work_items`、`artifacts`、`reviews`、`audit_events`。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`：任务包草稿登记 v1 已接受，但还没有生成真实任务包文件或 Markdown 预览。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke-review.md`：真实 Tauri 窗口创建任务包草稿 smoke 带缺口通过；真实窗口验证现在按用户要求暂停。

当前缺口：

- 工作台里已有 `work_items[]` 任务草稿，但用户还看不到它会变成什么任务包。
- 还没有从任务草稿到标准任务包文本的中间预览。
- 不能直接生成真实 `product-line/tasks/*.md`，否则缺少用户确认和后续派发边界。

## 目标

实现“任务包 Markdown 预览”。

用户在项目详情页中选择已有任务包草稿后，能看到一份标准任务包 Markdown 预览。预览必须来自工作台自己的状态文件和当前项目索引元数据。

预览至少包含：

- 任务名
- 所属开发线
- 背景
- 目标
- 允许读取
- 允许写入
- 禁止事项
- 验收标准
- 必须回传
- 总指导回收重点

如果草稿里没有足够信息，预览必须显示“待补充”或“未登记”，不能补编业务。

前端至少提供：

- 项目详情页展示当前项目已有任务草稿列表。
- 选择一个任务草稿后显示 Markdown 预览。
- 预览区明确标注“预览，不是已派发任务包”。
- 提供复制预览文本入口；复制前仍走现有本机动作确认弹层，说明只复制文本，不写真实任务文件。

后端至少提供：

- 一个只读渲染入口，例如 `render_task_package_preview`。
- 输入必须限定在索引内项目和现有 `work_item_id`。
- 找不到项目、workflow、work item 或 artifact 时返回明确错误。
- 不写真实任务包文件。
- 不派发真实 Codex 会话。

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
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-tauri-smoke-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端读取工作台自己的状态文件以渲染预览：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

约束：

- evidence / handoff 不得打印完整真实状态文件正文。
- 如需测试状态内容，使用临时目录夹具，不读取真实状态文件正文。

## 允许写入

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`
- `product-line/evidence/`
- `product-line/handoffs/`

本任务不允许写入真实工作台状态文件，除非已有测试夹具在临时目录中写入。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不生成真实 `product-line/tasks/*.md` 任务包文件。
- 不写真实工作台状态文件。
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

- 新增 Tauri 命令，例如 `render_task_package_preview(project_root, work_item_id)`。
- 从索引确认 project root 是索引内项目。
- 从工作台状态确认项目 workflow 和 work item 存在。
- 查找与 work item 对应的 `artifact_type=task_package` 草稿。
- 用稳定模板生成 Markdown 字符串。
- 缺字段时输出“待补充”，不要生成看似确定但没有依据的内容。
- 渲染只读，不新增 audit event。

前端：

- 在项目工作流草稿区增加任务草稿列表。
- 选中任务草稿后展示 Markdown 预览。
- 预览区使用等宽文本或只读代码块，方便复制。
- 复制预览文本前复用确认弹层。
- 如果项目没有 workflow 或没有 task draft，显示下一步提示，不显示假数据。

任务包模板建议沿用 `product-line/TASK_TEMPLATE.md`，但允许先生成阶段 3 需要的最小字段。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 非索引项目拒绝渲染。
  - 没有状态文件时返回明确错误或空状态提示。
  - 没有 workflow 时拒绝渲染。
  - 没有 work item 时拒绝渲染。
  - 有任务草稿时生成 Markdown 预览。
  - 缺字段时输出“待补充”或“未登记”，不补编业务。
- 前端离线测试覆盖：
  - 有任务草稿时显示预览入口。
  - 没有任务草稿时显示下一步提示。
  - 预览区标注“预览，不是已派发任务包”。
  - 复制预览文本前显示确认弹层。
  - 取消确认不会执行复制。
- evidence / handoff 明确说明没有做真实窗口验证。
- evidence / handoff 明确说明没有生成真实 `product-line/tasks/*.md`。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. Markdown 预览包含哪些字段
6. 缺字段时如何处理
7. 如何保证只读、不写真实任务包文件
8. 如何保证不派发真实 Codex 会话
9. 验证命令和结果
10. 是否读取或打印真实状态文件正文
11. 是否触碰禁止事项
12. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只生成预览，没有生成真实任务包文件。
- 是否没有自动派发真实 Codex 会话。
- 是否没有写 `.codex`、Codex 状态库、项目业务目录或真实工作台状态文件。
- 是否没有读取会话正文或敏感内容。
- Markdown 预览是否能作为下一步“确认后写入真实任务包文件”的前置。
- 是否适合下一步做任务包文件生成 v1，还是应先补任务草稿字段。
