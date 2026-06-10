# 任务包：任务包内容修正与派发准备 v1

## 任务名

让工作台能识别并修正已生成任务包的内容质量问题，生成可作为后续派发入口的新版任务包，并标记派发准备状态。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及工作台状态里的任务字段、artifact 状态、任务包重新生成策略、前端提示和后端写入流程。

## 背景

阶段 3 要求工作台能输出标准任务包，登记回收结果和总指导判断。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 通过标准包含“能输出标准任务包，登记回收结果和总指导判断”。
- `product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1-review.md`：真实任务包文件生成确认 v1 已接受，真实落盘和 artifact path 回填已完成。
- `product-line/tasks/2026-05-29-generated-task-draft-smoke.md`：当前真实生成任务包存在输入法污染、待补充字段和历史禁令，不能直接作为正式派发入口。
- `product-line/TASK_TEMPLATE.md`：标准任务包的字段结构。

当前缺口：

- 工作台已经能生成真实任务包文件。
- 但当前真实文件内容质量不足。
- 进入真实 Codex 派发前，工作台必须能发现任务包内容不合格，并支持修正后生成新版文件。

## 目标

实现“任务包内容修正与派发准备 v1”。

至少交付：

- 后端能对选中 `work_item_id` 的 `task_package` artifact 做派发准备检查。
- 检查应识别：
  - 标题为空或明显仍是测试草稿。
  - 目标为空、待补充或含输入法污染。
  - 允许读写为空或仍是待补充。
  - 禁止事项中含和当前生成行为冲突的历史禁令，例如“不生成真实 `product-line/tasks/*.md`”。
  - 验收标准为空或仍是待补充。
  - 必须回传缺少标准字段。
- 前端能显示派发准备状态：
  - `not_ready`
  - `ready`
  - `blocked`
- 前端能列出不合格原因，不把不合格任务包装成可派发。
- 后端能保存修正后的结构化任务字段。
- 修正字段后可生成新版任务包文件。
- 新版任务包不能覆盖旧文件。
- 新版任务包路径要回填到 artifact，并保留旧生成记录的审计信息。
- 写入状态前备份，写入后重新读取校验。

本任务只做到“准备好给后续派发”，不真正派发 Codex。

## 不做

- 不派发真实 Codex 会话。
- 不启动 Codex CLI。
- 不运行 harness。
- 不登记真实 handoff / evidence / review。
- 不做自动执行状态机。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。
- 不把真实 Tauri 窗口验证作为验收前置。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/TASK_TEMPLATE.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- `product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端读取工作台自己的状态文件：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

约束：

- evidence / handoff 不得打印完整真实状态文件正文。
- 如需确认状态更新，只输出字段级、数量级、路径级结果。

## 允许写入

允许修改产品化桌面壳：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`

允许新增交接记录：

- `product-line/evidence/`
- `product-line/handoffs/`

允许由 Tauri 后端在用户确认或受控确认测试中写入：

- `/Users/yoyi/workspace/product-line/tasks/*.md`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不覆盖已有 `product-line/tasks/*.md`。
- 不改 `product-line/tasks/README.md`，除非总指导线回收后更新队列。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不打印完整真实 `workflow-state.v0.json` 正文。
- 不自动运行 harness。
- 不把索引候选自动升级成“已验证能力”。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不拉取外网依赖。
- 不实现自动编排执行。
- 不把内容不合格的任务包标记成 ready。

## 实现建议

后端：

- 新增或扩展只读检查函数，例如 `inspect_task_package_dispatch_readiness`。
- 新增或扩展写入函数，例如 `update_task_package_dispatch_fields`。
- readiness 检查返回结构化结果：
  - `status`
  - `blocking_reasons`
  - `warnings`
  - `artifact_path`
  - `can_generate_next_version`
- 生成新版任务包时不要覆盖旧文件；可以使用新后缀，例如 `-2.md` 或 `ready-<slug>.md`。
- artifact 可以保留当前 `path` 指向最新生成文件，同时用 audit event 记录旧路径到新路径的变化。
- 追加 audit event，例如：
  - `task_package_dispatch_fields_updated`
  - `task_package_dispatch_readiness_checked`
  - `task_package_ready_file_generated`
- 如果新增 artifact 字段，必须兼容旧状态文件；缺字段时降级显示 warning。

前端：

- 在项目详情的任务草稿区域显示 readiness 状态。
- 如果不 ready，显示原因列表。
- 提供“修正派发字段”表单，复用现有任务包字段编辑能力。
- 提供“生成可派发版本”按钮，但必须走确认弹层。
- 确认弹层明确：
  - 只生成任务包文件。
  - 不派发 Codex。
  - 不启动 Codex CLI。
  - 不运行 harness。
  - 不写 `/Users/yoyi/.codex` 或 Codex 状态库。

内容规则：

- 不要自动编造业务目标。
- 如果没有用户提供的新任务内容，只能把状态标成 `not_ready` 并展示缺口。
- 可以提供字段修正入口，但不能替用户猜测真实业务需求。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 当前污染草稿会被判定为 `not_ready`。
  - 缺目标、缺允许写入、缺验收标准时不 ready。
  - 禁止事项含“不生成真实任务包”这类冲突禁令时不 ready。
  - 字段修正后可以变为 ready。
  - 生成新版文件不覆盖旧文件。
  - 生成新版文件后 artifact path 更新。
  - audit event 写入。
  - 缺字段不被自动补编。
- 前端离线测试覆盖：
  - 显示 readiness 状态。
  - 不 ready 时显示原因。
  - ready 前不显示或禁用正式派发准备按钮。
  - 生成新版文件确认弹层说明不派发 Codex、不运行 harness。
- evidence / handoff 明确说明：
  - 是否生成新版真实任务包文件。
  - 真实生成路径。
  - 是否读取或打印真实状态文件正文。
  - 是否派发 Codex。
  - 是否运行 harness。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. readiness 检查规则
6. 当前真实任务包被判定为什么状态，依据是什么
7. 是否生成新版真实任务包文件；如果生成，列出路径
8. 是否更新 artifact path 和 audit event
9. 如何保证没有覆盖旧任务包
10. 如何保证没有派发真实 Codex 会话
11. 如何保证没有运行 harness
12. 验证命令和结果
13. 是否读取或打印真实状态文件正文
14. 是否触碰禁止事项
15. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否能识别当前生成任务包内容不合格。
- 是否没有把不合格任务包标成 ready。
- 是否没有编造业务目标。
- 是否没有覆盖已有任务包文件。
- 是否没有派发 Codex。
- 是否没有运行 harness。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 是否为下一步真实 Codex 派发准备了稳定入口。
