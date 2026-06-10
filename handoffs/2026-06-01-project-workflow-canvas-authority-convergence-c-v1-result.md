# Project Workflow Canvas Authority Convergence Task C Result

日期：2026-06-01

## 结果

Task C 已完成一个保守切片。

本轮只做 UI/入口/文档收敛：

- 项目页工作流入口标为“项目工作流”。
- 项目页画布标为“项目工作流主入口”和“项目工作流画布”。
- 项目页派生画布展示“项目事实”和“事实源：项目 workflow state / 派生读模型”。
- 全局独立画布入口从“工作流”改成“实验画布”。
- 独立 `CanvasView` 页面标为“实验 / 模板画布”，运行区标为“实验运行”。
- 右侧栏 `running` 入口从“运行中工作流”改成“项目运行”，点击仍回项目页。

## 改动文件

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md`
- `handoffs/2026-06-01-project-workflow-canvas-authority-convergence-c-v1-result.md`

## 没做的事

- 没有执行真实 `codex exec` / `codex exec resume`。
- 没有启动 MCP canvas run。
- 没有改 workflow state JSON。
- 没有迁移数据库。
- 没有读取 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env` 或完整 transcript。
- 没有改状态机、工作流机器、派发、回收或 MCP 可编辑画布运行逻辑。
- 没有把独立 canvas 文件层改成项目事实源。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过。

说明：

- `npm run test:offline-interaction` 第一次因旧断言“工作流编排”失败；更新测试期望后通过。
- `npm run build` 保留 Vite chunk 大小 warning，不是本轮引入的功能错误。
- 没跑 `cargo test --lib`，因为本轮只改前端 UI 文案和前端测试；按计划前端-only 任务至少跑前端三条命令。

## 手动测试清单

1. 打开应用，查看左侧主导航：应看到“项目”和“实验画布”，不应再看到一个全局“工作流”入口和项目工作流并列抢主入口。
2. 点击“项目”，选择任意有 workflow state 的项目，进入项目详情；项目工具栏应显示“项目工作流”。
3. 进入“项目工作流”页；顶部应显示“项目工作流主入口”，画布区域应显示“项目工作流画布”和“项目事实”。
4. 在项目工作流画布详情里检查“事实源”：应显示“项目 workflow state / 派生读模型”。
5. 打开右侧栏“项目运行”；列表里的运行项点击后应回到项目页，不应打开独立实验画布。
6. 点击左侧“实验画布”；页面标题应显示“实验 / 模板画布”，运行区应显示“实验运行”“启动实验运行”“停止实验运行”。
7. 本轮手动测试不要点击“启动实验运行”、项目页“派发指令”“审核后派发”或“启动四角色工作流机器”，这些会触发真实运行或真实 Codex 路径，不属于 Task C 验收。

## 剩余风险

- 独立 `CanvasView` 仍有保存和实验运行能力；本轮只降权入口，没有删除或冻结它。
- `ProjectsView.tsx` 仍有任务包、账本、状态机、异常和验收场景等内部面板；本轮没有把它们收进节点详情或右侧抽屉。
- 如果后续要让独立画布并入项目 workflow，仍需要单独迁移计划，不能直接把 `CanvasDefinition` 当项目事实源。
