# 任务包：任务字段修正输入 v1

## 任务名

在工作台里提供清晰的任务字段修正输入流程，让用户能把 not_ready 任务修正为可检查的 ready 候选。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务涉及前端任务字段输入、预览、后端状态写入、readiness 复检和确认边界。

## 背景

阶段 3 要求工作台能输出标准任务包，登记回收结果和总指导判断。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 通过标准包含“能输出标准任务包，登记回收结果和总指导判断”。
- `product-line/handoffs/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1-review.md`：任务包派发准备检查与拦截 v1 已接受，但当前没有可派发任务包。
- `product-line/tasks/2026-05-29-generated-task-draft-smoke.md`：当前真实任务包仍是 not_ready，存在输入法污染、待补充字段和历史冲突禁令。
- `product-line/TASK_TEMPLATE.md`：标准任务包字段入口。

当前缺口：

- 工作台能识别当前任务包不适合派发。
- 但用户还缺一个清楚的地方，把真实任务内容填进去并保存为结构化字段。
- 没有真实业务内容时，系统不能自动补编任务目标。

## 目标

实现“任务字段修正输入 v1”。

至少交付：

- 前端提供清楚的字段修正入口，不依赖用户去猜当前哪个旧表单能修。
- 字段至少包含：
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
- 前端保存前显示字段级预览，明确哪些字段仍缺失。
- 保存前走确认弹层。
- 保存后写入工作台自己的 workflow state。
- 保存后自动或手动触发 readiness 复检。
- readiness 通过时，可以显示“可生成新版任务包”状态；但本任务不要求真实生成新版文件。
- readiness 不通过时，列出仍需修正的字段。

本任务只做到“让用户能输入和保存真实任务字段”，不替用户编写真实业务目标。

## 不做

- 不派发真实 Codex 会话。
- 不启动 Codex CLI。
- 不运行 harness。
- 不强制生成新版真实任务包文件。
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
- `product-line/tasks/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-package-dispatch-readiness-v1-review.md`
- `product-line/handoffs/2026-05-29-desktop-shell-task-fields-edit-v1-review.md`
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

允许由 Tauri 后端在用户确认或受控测试中写入：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

如果实现需要测试新版任务包生成，只能在临时目录测试；本任务不要求写真实 `product-line/tasks/*.md` 新文件。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不覆盖已有 `product-line/tasks/*.md`。
- 不生成新版真实 `product-line/tasks/*.md`，除非用户另行明确确认。
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
- 不把缺少用户业务内容的任务包标记成 ready。
- 不自动编造任务目标、验收标准或允许写入范围。

## 实现建议

前端：

- 在派发准备面板旁边增加“修正任务字段”入口。
- 用一个明确的字段编辑面板替代“猜旧表单在哪里”的体验。
- 字段编辑面板应显示：
  - 当前值
  - 缺失提示
  - 字段级预览
  - 保存确认按钮
- 保存前确认弹层要说明：
  - 只写工作台自己的状态文件。
  - 不生成真实任务包文件。
  - 不派发 Codex。
  - 不启动 Codex CLI。
  - 不运行 harness。
  - 不写 `/Users/yoyi/.codex` 或 Codex 状态库。
- 保存后刷新 workflow state 和 readiness 状态。

后端：

- 可复用现有 `update_task_package_draft_fields` 能力。
- 如果现有能力缺少字段级 warning 或预览结果，可补结构化返回。
- 写入前备份状态。
- 写入后重新读取校验。
- 追加 audit event，例如 `task_package_fields_corrected_for_dispatch`。

内容规则：

- 不要自动填真实业务目标。
- 可以保留用户输入的内容。
- 空字段应继续作为 missing warning 或 not_ready reason。
- 历史冲突禁令应提示用户删除，但不要自动删除，除非用户保存的新字段已明确不包含它。

## 验收标准

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Rust 测试覆盖：
  - 保存修正字段前拒绝非索引项目。
  - 保存修正字段前拒绝缺状态文件。
  - 保存修正字段前拒绝缺 workflow / work item / task_package artifact。
  - 保存修正字段会备份旧状态。
  - 保存修正字段会写入 audit event。
  - 空字段不会被自动补编。
  - 保存后 readiness 可重新检查。
- 前端离线测试覆盖：
  - 修正入口存在。
  - 字段预览显示关键字段。
  - 缺字段显示提示。
  - 保存确认弹层说明不生成真实任务包、不派发 Codex、不运行 harness。
  - 保存后触发刷新或 readiness 复检。
- evidence / handoff 明确说明：
  - 是否写了真实 workflow state。
  - 是否生成新版真实任务包文件。
  - 是否读取或打印真实状态文件正文。
  - 是否派发 Codex。
  - 是否运行 harness。

## 必须回传

1. 薄弱点
2. 做了什么
3. 改了哪些文件
4. 新增或修改了哪些测试
5. 字段修正入口怎么用
6. 保存了哪些字段
7. 是否写入真实 workflow state；如果写入，字段级说明写入结果
8. 是否生成新版真实任务包文件
9. readiness 是否重新检查，结果是什么
10. 如何保证没有派发真实 Codex 会话
11. 如何保证没有运行 harness
12. 验证命令和结果
13. 是否读取或打印真实状态文件正文
14. 是否触碰禁止事项
15. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否给用户提供了清楚的任务字段修正入口。
- 是否没有自动编造业务目标。
- 是否保存前有预览和确认。
- 是否保存后能复检 readiness。
- 是否没有生成真实新版任务包，除非用户明确确认。
- 是否没有派发 Codex。
- 是否没有运行 harness。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
