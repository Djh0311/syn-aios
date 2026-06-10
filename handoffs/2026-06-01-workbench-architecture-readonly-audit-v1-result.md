# Workbench Architecture Readonly Audit v1 Result

日期：2026-06-01

## 1. 这轮做了什么

只执行了“任务 A：架构只读审计”。

产出：

- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`
- `handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`

没有做：

- 没有改代码。
- 没有运行真实 `codex exec` / `codex exec resume`。
- 没有读取 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env`、完整 transcript。
- 没有写真实业务项目目录。
- 没有迁移数据库。

## 2. 主要结论

先说薄弱点：

- 当前 app 最大问题是层边界混在一起，不是少某个功能。
- `lib.rs` 仍是最高风险区：命令、模型、状态读写、派发、工作流机器、读模型都在一个文件里。
- 项目工作流画布和独立可编辑画布有权威冲突风险，需要新 decision。
- `ProjectsView.tsx` 仍有任务包管理器倾向，尤其是任务包草稿、预览、生成文件、派发 readiness 和内部协议面板。
- 秘书和记忆治理还不适合直接做真实写入闭环，第一阶段应先做模型、候选、读模型和确认边界。

可以继续的部分：

- 可以进入“无行为变化拆模块”。
- 第一批只建议拆类型、Tauri command 包装、workflow read model、WorkbenchSnapshot 组装。
- 不建议第一批碰状态机、工作流机器、Codex runner、MCP canvas run、任务包产品规则。

依据：

- 详细依据和表格见 `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`。

## 3. 下一轮建议

建议下一轮只做 Task B 的保守切片：

- 先拆后端类型和读模型。
- 保持所有 Tauri command 名字、请求字段、响应字段不变。
- 不改 workflow state JSON 结构。
- 不改 Codex 执行路径。
- 不改任务包产品规则。
- 不动独立可编辑画布运行逻辑。

进入 Task B 前建议先补两条 decision：

- 项目工作流画布和独立可编辑画布的权威关系。
- 无行为变化拆模块的保护边界。

## 4. 人工复核清单

这轮没有改 UI，所以不是应用内手动测试清单；这里给人工复核清单：

- 打开 `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`，确认有后端模块拆分表。
- 打开同一 evidence，确认有前端组件归属表。
- 打开同一 evidence，确认有画布权威风险判断。
- 打开同一 evidence，确认有高风险拆分点清单。
- 打开同一 evidence，确认有“是否可以进入无行为变化拆模块”的判断。
- 检查本轮只新增 evidence 和 handoff，没有修改 `prototypes/**/src*` 代码。
- 检查本轮没有真实 Codex 执行记录。
- 检查本轮没有数据库迁移产物。

## 5. 当前入口

本轮新证据：

- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`

本轮交接：

- `handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`

当前权威仍然是：

- `CURRENT.md`
- `AUTHORITY.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`

## 6. 风险

- 如果下一轮直接拆 `run_workflow_machine`、Codex resume、MCP canvas run，容易发生行为变化。
- 如果不先定画布权威，项目 workflow state 和独立 canvas 文件会继续并行扩张。
- 如果第一阶段直接做秘书自动写事实或正式记忆写入，会违反当前架构边界。
