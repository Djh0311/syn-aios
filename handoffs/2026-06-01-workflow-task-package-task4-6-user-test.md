# Workflow Task Package Task 4-6 Handoff

## 先说薄弱点

- 本轮只验证了代码路径、Rust 单测和前端静态渲染测试，没有打开 Tauri 窗口做真实点击。
- Task 7-12 仍未确认，所以账本写入边界、子智能体汇报、审查权力、状态机完成判定和端到端验收没有开发。
- 运行前检查现在是阻止运行和派发的只读检查，不代表可以自动执行。
- 任务包预览只展示字段完整性，不证明真实任务已经派发。

## 这轮做了什么

- 补了 v0 workflow state 到 v1 草案对象的兼容读模型。
- 补了 `inspect_workflow_run_check` 运行前检查。
- 补了任务包版本、missing 字段、stale 状态和字段预览展示。
- 前端项目页展示运行前检查、派生 v1 读模型和任务包字段预览。
- 测试覆盖缺模型、缺读写范围、缺验收标准、工具/harness warning、知识库/记忆为空不自动补、任务包 stale。

## 改了哪些文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-9x3mY5U7.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BxCF2HBh.css`

`dist/**` 是 `npm run build` 刷新的构建产物，不是手写规则。

## 新增 evidence

- `evidence/2026-06-01-workflow-task-package-plan-baseline.md`

## 当前入口

- 计划入口：`docs/plans/2026-06-01-workflow-task-package-design-v1-execution-plan.md`
- 草案入口：`docs/workflow-task-package-design-v1.md`
- 代码入口：`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- 后端入口：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- Evidence 入口：`evidence/2026-06-01-workflow-task-package-plan-baseline.md`

## 手动测试清单

1. 打开应用，进入“项目”页。
2. 在左侧项目列表选择一个已有 workflow state 的项目。
3. 看项目内工具栏是否显示：工作流、Agent 会话、任务包、Handoff / Evidence、Skills、Harness、设置。
4. 留在“工作流”页，看页面上方是否出现“运行前检查”。
5. 点击“检查运行前状态”。
6. 如果项目没有 workflow，结果应该是 blocked，并显示没有工作流。
7. 如果要派发的节点没有绑定会话，结果应该是 blocked，并显示缺绑定会话。
8. 如果任务包缺模型，结果应该是 blocked，并显示系统不会自动选择模型。
9. 如果缺读范围、写范围或验收标准，结果应该是 blocked。
10. 如果节点没有声明需要工具，工具白名单为空可以显示 warning 或 empty，不能被系统自动填工具。
11. 如果节点没有要求 harness，harness 可以显示 warning 或 empty，不能自动运行 harness。
12. 看“派生 v1 读模型”和“任务包预览字段”。
13. 任务包预览里应该能看到允许读取、允许写入、知识库引用、记忆引用、工具白名单、技能、模型、harness、验收标准、回传格式、禁止事项。
14. 缺失字段应该显示 missing；非必需且未声明的知识库、记忆、工具、harness 应显示 empty 或 warning。
15. 确认页面没有把模型、权限、知识库、记忆或业务事实自动补成一段看似完整的说明。
16. 保存任务包字段后，看任务包版本或 stale 提示；编辑后应该要求重新检查。
17. 如果任务包 stale，派发准备应该不能直接通过，必须重新生成或重新检查。
18. 点击“任务包”草稿区的“预览 Markdown”，只应该看到预览，不应该派发真实 Codex 会话。
19. 点击保存字段或生成文件时，确认弹层必须写清写入边界，不应写 `/Users/yoyi/.codex`，不应启动 Codex CLI。

## 通过标准

- 运行前检查能显示 runnable、warning 或 blocked。
- blocked reason 能明确指出缺什么。
- 缺模型时没有自动选择模型。
- 没有显式知识库或记忆引用时，显示为空，不生成猜测内容。
- 任务包 preview 能看到 missing/stale。
- 保存字段、生成任务包文件都需要确认弹层。
- 页面没有出现真实派发、真实 resume、自动运行 harness 的行为。

## 失败标准

- 缺模型但页面显示了自动选择的模型。
- 缺写范围或验收标准仍显示可派发。
- 知识库、记忆或业务背景被系统编成了事实。
- stale 任务包仍能直接派发。
- 点击预览或检查时触发真实 Codex 会话、写 `/Users/yoyi/.codex`、读取完整 transcript 或运行 harness。
- 任务包生成覆盖已有文件。

## 本轮验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过。
- `cargo test --lib snapshot_keeps_metadata_without_session_body -- --nocapture`：通过。
- `cargo test --lib workflow_task_package_read_model -- --nocapture`：通过。
- `cargo test --lib workflow_run_check -- --nocapture`：通过。
- `cargo test --lib task_package -- --nocapture`：通过，真实写文件确认测试 ignored。
- `cargo test --lib`：通过，73 passed，1 ignored。

## 边界

- 未迁移数据库。
- 未写真实用户 workflow state。
- 未写 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env` 或完整 transcript。
- 未执行 Task 7-12。
