# 任务包：真实任务包文件生成确认 v1

## 任务名

在真实工作台状态和真实 `product-line/tasks/` 目录下，生成一个任务包 Markdown 文件，并回填工作台状态里的 artifact path。

## 所属开发线

桌面应用线。

当前派给桌面应用线，因为任务需要使用产品化桌面壳已经实现的 `generate_task_package_file` 能力，确认真实工作台状态、真实任务目录、状态回填和审计记录能闭环。

## 背景

阶段 3 要求工作台能在项目级可视化工作流中登记开发线、生成任务包、回收结果摘要。

依据：

- `product-line/STAGE_PLAN.md`：阶段 3 通过标准包含“能输出标准任务包，登记回收结果和总指导判断”。
- `product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-v1-review.md`：真实任务包文件生成能力 v1 已接受，但只在临时目录验证，真实 `product-line/tasks/*.md` 尚未生成。
- `product-line/tasks/README.md`：下一步建议是“真实任务包文件生成确认 v1”，在真实工作台状态和真实 `product-line/tasks/` 下生成一个任务包文件，并回填 artifact path。

当前缺口：

- 工作台已经具备生成真实任务包文件的代码能力。
- 但还没有在真实 `product-line/tasks/` 目录下生成过任务包文件。
- 后续要派发 Codex 执行线，需要一个真实任务包文件作为稳定入口。

## 目标

完成一次真实落盘确认：

- 使用当前真实工作台状态文件中已有的任务草稿。
- 在 `/Users/yoyi/workspace/product-line/tasks/` 下生成一个真实任务包 Markdown 文件。
- 生成后回填对应 `task_package` artifact 的 `path`。
- 追加 `task_package_file_generated` audit event。
- 生成后做只读校验，确认文件存在、路径在允许目录内、不是覆盖已有文件。

生成文件必须仍然只是任务包文件，不代表任务已经派发执行。

## 不做

- 不派发真实 Codex 会话。
- 不启动 Codex CLI。
- 不运行 harness。
- 不登记真实 handoff / evidence / review。
- 不做节点拖拽编辑。
- 不做边编辑。
- 不做完整状态机。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。
- 不把真实 Tauri 窗口验证作为验收前置。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/TASK_TEMPLATE.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-v1-review.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`
- `product-line/prototypes/productized-desktop-shell/`

允许由 Tauri 后端读取工作台自己的状态文件：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

约束：

- evidence / handoff 不得打印完整真实状态文件正文。
- 如需确认状态更新，优先做字段级、数量级、路径级校验，不输出正文内容。

## 允许写入

允许写入项目内任务目录：

- `/Users/yoyi/workspace/product-line/tasks/*.md`

允许由 Tauri 后端写入工作台自己的状态文件和备份：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

允许新增交接记录：

- `product-line/evidence/`
- `product-line/handoffs/`

如确实发现真实落盘能力存在小缺口，可修改：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/`

但本任务优先使用已有能力，不主动扩大为功能开发任务。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不写项目业务目录。
- 不覆盖已有 `product-line/tasks/*.md`。
- 不改 `product-line/tasks/README.md`，除非总指导线另行回收后更新队列。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不打印完整真实 `workflow-state.v0.json` 正文。
- 不自动运行 harness。
- 不把索引候选自动升级成“已验证能力”。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不拉取外网依赖。
- 不实现自动编排执行。

## 实现建议

优先路径：

- 使用现有 Tauri 后端 `generate_task_package_file` 能力完成真实落盘。
- 如果不启动真实窗口，也可以通过受控测试入口或临时小工具调用同一后端逻辑；但必须说明调用方式和边界。
- 生成目标目录必须固定为 `/Users/yoyi/workspace/product-line/tasks/`，不能从前端或脚本传任意目录。
- 生成前记录已有 `product-line/tasks/*.md` 文件名列表或目标前缀匹配结果，用于证明没有覆盖。
- 生成后只读校验：
  - 新文件存在。
  - 新文件路径在 `/Users/yoyi/workspace/product-line/tasks/` 下。
  - 新文件名未覆盖旧文件。
  - 新文件包含任务名、目标、禁止事项、验收标准、必须回传等标准段落。
  - 工作台状态里对应 artifact path 已回填。
  - audit event 已追加。

如果真实工作台状态里没有可生成的任务草稿：

- 不要编造状态。
- 回传 blocked，并说明缺哪个前置。
- 可以建议先通过工作台创建一个任务草稿，但不要绕过确认直接写状态。

## 验收标准

- 真实 `product-line/tasks/` 下新增且只新增一个本任务生成的任务包 Markdown 文件。
- 新文件不覆盖已有任务包。
- 新文件内容符合 `product-line/TASK_TEMPLATE.md` 的主要结构。
- 工作台状态里的对应 `task_package` artifact path 已回填为新文件路径。
- 工作台状态里追加了 `task_package_file_generated` audit event。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- evidence / handoff 明确说明：
  - 真实生成文件路径。
  - 是否读取或打印真实状态文件正文。
  - 是否派发 Codex。
  - 是否运行 harness。
  - 是否改写 `.codex` 或 Codex 状态库。

## 必须回传

1. 薄弱点
2. 做了什么
3. 真实生成的任务包文件路径
4. 是否修改了代码；如果有，列出文件
5. 是否新增 evidence / handoff
6. 生成前后如何证明没有覆盖已有任务包
7. 写入了哪些 `artifacts[]` / `audit_events[]` 字段
8. 如何确认 artifact path 已回填
9. 如何保证没有派发真实 Codex 会话
10. 如何保证没有运行 harness
11. 验证命令和结果
12. 是否读取或打印真实状态文件正文
13. 是否触碰禁止事项
14. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否真的在 `/Users/yoyi/workspace/product-line/tasks/` 生成了任务包文件。
- 是否没有覆盖已有任务包文件。
- 是否正确回填 artifact path。
- 是否追加 audit event。
- 是否没有派发 Codex。
- 是否没有运行 harness。
- 是否没有写 `.codex`、Codex 状态库或项目业务目录。
- 新生成任务包是否可以作为下一步“派发 Codex 执行线”的稳定入口。
