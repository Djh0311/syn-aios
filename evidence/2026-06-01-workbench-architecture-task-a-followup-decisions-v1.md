# Evidence: Workbench Architecture Task A Follow-up Decisions v1

日期：2026-06-01

## 做了什么

- 读取并复核 Task A 产物：
  - `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`
  - `handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`
- 根据 Task A 结论补两个 Task B 前置决策：
  - `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
  - `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`
- 同步当前入口：
  - `CURRENT.md`
  - `AUTHORITY.md`
  - `README.md`
  - `tasks/README.md`
  - `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

## 结论

- Task A 已完成。
- 可以进入 Task B，但只能进入保守切片。
- Task B 第一批只允许拆类型、Tauri command 包装、workflow 读模型、WorkbenchSnapshot 组装和前端纯类型。
- Task B 第一批不能碰状态机、workflow state JSON、真实 Codex resume、工作流机器、MCP 可编辑画布运行逻辑和任务包产品规则。

## 边界

- 本轮没有改代码。
- 本轮没有运行测试。
- 本轮没有读取 `/Users/yoyi/.codex`。
- 本轮没有执行 `codex exec` 或 `codex exec resume`。
- 本轮没有写真实业务项目目录。
- 本轮没有迁移数据库。

## 风险

- 如果下一轮把“无行为变化拆模块”理解成重构业务规则，会破坏当前工作流闭环。
- 如果不遵守画布权威决策，项目 workflow state 和独立 canvas 文件层会继续并行扩张。
