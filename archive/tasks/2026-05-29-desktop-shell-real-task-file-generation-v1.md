# 任务包：真实任务包文件生成 v1

## 任务名

在产品化桌面壳中，从选中的任务草稿生成真实 `product-line/tasks/*.md` 任务包文件。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及 Tauri 后端文件写入、路径白名单、工作台状态更新、前端确认流程和任务包预览复用。

## 背景

阶段 3 目标要求工作台能在项目级可视化工作流里登记开发线、生成任务包、回收结果摘要。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 明确要求“在工作台里登记开发线、生成任务包、回收结果摘要”，并能“输出标准任务包，登记回收结果和总指导判断”。
- `product-line/TASK_TEMPLATE.md`：当前标准任务包字段入口。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`：任务包草稿登记 v1 已接受。
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`：任务包 Markdown 预览 v1 已接受。
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-review.md`：任务包字段编辑 v1 已接受。
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-selection-v1-review.md`：任务草稿选择态 v1 已接受，后续真实文件生成可以依赖选中的 `work_item_id`。

当前缺口：

- 工作台能登记草稿、编辑字段、预览 Markdown，但还不能生成真实任务包文件。
- 后续派发 Codex 开发线需要真实任务包文件作为稳定交接入口。
- 生成真实文件前必须继续保留用户确认、路径限制和审计记录。

## 目标

实现“真实任务包文件生成 v1”。

用户在项目详情页中选中一个任务草稿后，可以在确认弹层确认生成真实任务包 Markdown 文件。

生成范围限定为：

```text
/Users/yoyi/workspace/product-line/tasks/
```

文件内容来自当前选中任务草稿的结构化字段和已验证的 Markdown 预览渲染逻辑。

生成后至少要：

- 写入一个新的 `product-line/tasks/*.md` 文件。
- 文件名可追溯、稳定、避免覆盖，例如：
  - `YYYY-MM-DD-desktop-shell-<slug>.md`
  - 或 `YYYY-MM-DD-generated-<work-item-short-id>.md`
- 如果目标文件已存在，必须拒绝覆盖或生成不冲突文件名；不能静默覆盖。
- 更新工作台状态里对应 `task_package` artifact：
  - `path` 指向新任务包文件路径。
  - `updated_at` 更新。
  - `warnings` 移除或保留与生成状态相关的 warning，但不能把缺字段伪装成已补齐。
- 追加 audit event，例如 `task_package_file_generated`。
- 写入后重新读取文件和状态快照做校验。

前端至少提供：

- 选中任务草稿后显示“生成任务包文件”入口。
- 生成前确认弹层明确：
  - 将写入 `product-line/tasks/`。
  - 不派发真实 Codex 会话。
  - 不运行 harness。
  - 不写 `/Users/yoyi/.codex` 或 Codex 状态库。
- 生成后显示生成文件路径。
- 已生成文件的草稿再次点击时不能静默覆盖。

## 不做

- 不自动派发给真实 Codex 会话。
- 不启动 Codex CLI。
- 不运行 harness。
- 不登记真实 handoff / evidence / review。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不做完整状态机。
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
- `product-line/tasks/2026-05-29-desktop-shell-task-fields-edit-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-task-draft-selection-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-markdown-preview-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-draft-selection-v1-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端读取工作台自己的状态文件：

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

- `/Users/yoyi/workspace/product-line/tasks/*.md`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不写 `product-line/tasks/README.md`，除非本任务明确另行要求；本轮只生成任务包文件。
- 不覆盖已有 `product-line/tasks/*.md`。
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

- 新增 Tauri 命令，例如 `generate_task_package_file`。
- 输入必须包含：
  - `project_root`
  - `work_item_id`
- 从索引确认 project root 是索引内项目。
- 从工作台状态确认项目 workflow、work item 和 `task_package` artifact 存在。
- 渲染 Markdown 时复用 `render_task_package_preview` 的结构化字段逻辑。
- 目标目录固定为 `/Users/yoyi/workspace/product-line/tasks/`，不要从前端接收任意输出目录。
- 文件名使用安全 slug：
  - 只允许小写字母、数字、连字符。
  - 空标题用 `task-package-<work-item-short-id>`。
  - 长度要截断。
- 目标文件存在时拒绝覆盖，或自动生成不冲突后缀；必须在 evidence 里说明策略。
- 写状态文件前备份旧状态。
- 写任务文件可以先写临时文件再原子 rename。
- 写入后重新读取任务文件，确认包含任务名、目标、禁止事项等关键段落。
- 更新 artifact `path` 和 `updated_at`。
- 追加 audit event。

前端：

- 在选中任务草稿后显示“生成任务包文件”按钮。
- 如果 task package artifact 已有 `path`，显示路径，并把按钮文案改成“已生成”或要求明确重新生成策略。
- 生成前走确认弹层。
- 成功后刷新 workflow state snapshot，并显示生成路径。
- 生成后不自动派发 Codex。

状态：

- `artifacts[].path` 是生成文件路径。
- `work_items[].state` 可以保持 `draft`，或改成 `ready_to_dispatch`；如果改状态，必须写 audit 并在 evidence 里说明。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 非索引项目拒绝生成。
  - 缺状态文件拒绝生成。
  - 缺 workflow 拒绝生成。
  - 缺 work item 拒绝生成。
  - 缺 `task_package` artifact 拒绝生成。
  - 生成文件写入临时测试 tasks 目录。
  - 已存在文件不会被静默覆盖。
  - 生成后 artifact `path` 更新。
  - 生成后 audit event 写入。
  - 生成内容来自结构化字段。
  - 缺字段仍显示“待补充”或“未登记”，不补编业务。
- 前端离线测试覆盖：
  - 选中草稿后显示生成入口。
  - 生成确认弹层显示写入目录和“不派发 Codex、不运行 harness”。
  - 取消确认不会调用生成动作。
  - 已有 `path` 时 UI 显示已生成状态。
- evidence / handoff 明确说明没有做真实窗口验证。
- evidence / handoff 明确说明是否在真实 `product-line/tasks/` 下生成了任务包文件；如果生成，必须列出文件路径。
- evidence / handoff 明确说明没有派发 Codex、没有运行 harness。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 真实任务包文件生成路径
6. 文件名生成和冲突处理策略
7. 写入了哪些 `artifacts[]` / `audit_events[]` 字段
8. 如何保证不覆盖已有任务包文件
9. 如何保证不派发真实 Codex 会话
10. 如何保证不运行 harness
11. 验证命令和结果
12. 是否读取或打印真实状态文件正文
13. 是否触碰禁止事项
14. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否只生成真实任务包文件，没有派发 Codex。
- 是否没有运行 harness。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否没有覆盖已有任务包文件。
- 是否正确更新 artifact `path` 和 audit event。
- 生成的任务包文件是否可以作为下一步“派发 Codex 执行线”的稳定入口。
