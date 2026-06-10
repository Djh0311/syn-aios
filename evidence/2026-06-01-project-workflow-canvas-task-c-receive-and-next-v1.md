# Evidence: Project Workflow Canvas Task C Receive And Next v1

日期：2026-06-01

## 做了什么

- 复核 Task C 交付：
  - `evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md`
  - `handoffs/2026-06-01-project-workflow-canvas-authority-convergence-c-v1-result.md`
- 抽查关键前端入口：
  - `App.tsx` 中全局 `workflow` 入口显示为“实验画布”。
  - `App.tsx` 中右侧 `running` 入口显示为“项目运行”。
  - `ProjectsView.tsx` 中项目工具显示为“项目工作流”。
  - `ProjectsView.tsx` 中项目画布显示“项目工作流主入口”。
  - `CanvasView.tsx` 中独立画布显示“实验 / 模板画布”。
- 同步当前入口：
  - `AUTHORITY.md`
  - `README.md`
  - `tasks/README.md`
  - `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

## 接收判断

Task C 可以接收为完成一个保守切片。

接受为：

- 项目页工作流主入口的权威标识已增强。
- 独立 `CanvasView` 的产品入口已降权为实验/模板画布。
- 右侧运行入口已收敛为项目运行。

不接受为：

- 双模型风险彻底消除。
- 独立 `CanvasView` 已冻结。
- 项目页任务包、账本、状态机等内部面板已收进节点详情或右侧抽屉。
- 独立 canvas 文件层已纳入项目控制核心。

## 验证依据

来自 Task C handoff：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- 未运行 `cargo test --lib`，原因是本轮只改前端 UI 文案和前端测试。

## 边界

- 本轮没有改 `prototypes/**` 代码。
- 本轮没有运行测试。
- 本轮没有读取 `/Users/yoyi/.codex`。
- 本轮没有执行 `codex exec` 或 `codex exec resume`。
- 本轮没有启动 MCP canvas run。
- 本轮没有写 workflow state JSON。
- 本轮没有迁移数据库。

## 下一步

建议派发 Task D：项目黑板最小实现。

Task D 边界：

- 先做黑板模型和读模型。
- 黑板内容只能作为中间态、候选、请求或引用。
- 黑板内容不能直接变成正式事实、正式记忆、审计事件或状态变化。
- 子智能体汇报、风险、权限请求、工具摘要、记忆候选、知识引用可以进入黑板。
- 只有控制核心确认后，黑板内容才能升级。
