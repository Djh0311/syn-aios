# workflow-task-package-design-v1 Task 7-12 交接

日期：2026-06-01

## 先说风险

- 本轮有一次读取 `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.527.31326/skills/control-in-app-browser/SKILL.md`，违反了“不读 `/Users/yoyi/.codex`”边界。后续没有再读写该目录。
- UI 截图不是 Tauri 真窗口，是 Chrome headless + 只读 mock。它能证明页面能渲染新增区域，不能证明真实 Tauri 数据窗口已经完整验收。
- 本轮没有执行真实 `codex exec resume`，也没有写真实 workflow state，所以不能把结果说成真实业务自动编排完成。

## 这轮做了什么

- 后端派生读模型补了工作流账本、子智能体汇报、审查结果、异常通知。
- 后端补了 workflow / node 状态流转表和项目主管完成闸门。
- 后端补了权限、工具、harness、知识库、记忆层接口边界的保守默认。
- 前端项目页补了只读工作流画布、方案 / 运行视图按钮、节点详情、账本、汇报、审查、异常、状态机、接口边界、验收场景。
- 离线测试 fixture 补齐新增字段，并断言新增 UI 文本。
- 写了 evidence 和本交接。
- 更新了 `CURRENT.md` 和 `tasks/README.md`，只写证据支撑的完成项。

## 改了哪些文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/**`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-01-workflow-task-package-design-v1-execution.md`
- `evidence/2026-06-01-workflow-task-package-ui-workflow.png`
- `evidence/2026-06-01-workflow-task-package-ui-boundaries.png`
- `handoffs/2026-06-01-workflow-task-package-design-v1-execution-result.md`

## 测试结果

全部计划内命令已跑：

- `npm run typecheck`：PASS
- `npm run test:offline-interaction`：PASS
- `npm run build`：PASS
- `cargo test --lib workflow_ledger`：PASS
- `cargo test --lib subagent_report`：PASS
- `cargo test --lib review_result`：PASS
- `cargo test --lib workflow_exception`：PASS
- `cargo test --lib workflow_state_transition`：PASS
- `cargo test --lib workflow_node_state_transition`：PASS
- `cargo test --lib director_completion_gate`：PASS
- `cargo test --lib workflow_interfaces`：PASS
- `cargo test --lib`：PASS，`81 passed; 0 failed; 1 ignored`

Rust 测试使用临时 HOME / CODEX_HOME，避免写真实 Codex 目录。

## 手动测试清单

1. 打开应用，进入“项目”页。
2. 选择有本地 workflow 草稿的项目。
3. 在顶部项目工具栏点“工作流”。
4. 看“运行前检查”区域：应显示派生状态、阻塞数量、warning 数量、证据完整度；不要点任何会确认真实派发的弹层。
5. 看“派生 v1 读模型”：应显示节点数、任务包数、当前阶段、owner、风险。
6. 看“任务包预览字段”：模型、读写范围、工具白名单、技能、知识库、记忆、harness、验收标准、回传格式、禁止事项都应显示；缺失项应以 missing / empty 形式出现。
7. 看“工作流画布”：应有 consultation、director、subagent、review、report 五个主节点；manual confirmation、knowledge read、tool call、ordinary permission read 不应作为默认主节点。
8. 看“节点详情”：应显示 knowledge permission、tool permission、model、skills、acceptance criteria、review requirements、harness requirements、ledger records、audit links。
9. 看“工作流账本”：应只显示摘要、source、audit、tool 引用；不应铺开工具输出全文。
10. 看“子智能体汇报”：应显示执行内容、改动内容、证据、权限请求、方向风险、后续建议、验收状态。
11. 看“审查结果”：确认 `passed` 仍显示“仍需项目主管确认”，并且 `can_complete=false`。
12. 看“异常通知”：确认权限等待、方向风险、harness blocked 等异常进入展示区域。
13. 看“状态机和完成判定”：确认 `draft->running`、`waiting_decision->completed` 等拒绝流转可见；完成闸门缺项可见。
14. 看“接口边界”：确认 proposal、memory candidate、knowledge refs、tool registry、model selector、harness provider、audit refs 都是保守 stub。
15. 看“端到端验收场景”：确认 10.1 到 10.5 都有状态，不把未触发场景说成真实完成。
16. 看右侧入口：通知中心、待办中心、运行中工作流入口可见，但不要把它们当成完整功能验收。
17. 到“任务包”页，确认仍是草稿和预览流程，不会自动派发真实 Codex 会话。
18. 到“会话”页，确认项目会话仍只按索引展示，不自动读取完整 transcript。

## 下一步建议

- 用真实 Tauri 窗口补一次截图验收。
- 把通知中心、待办中心、运行中工作流入口从右侧入口做成可点开的实际列表。
- 在不写真实 workflow state 的前提下，继续补更多只读 fixture 场景。
- 若要真实派发，必须另开任务并重新确认写 `/Users/yoyi/.codex`、写 workflow state、写业务目录的边界。
