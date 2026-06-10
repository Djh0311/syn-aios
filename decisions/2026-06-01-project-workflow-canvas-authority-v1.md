# 决策：项目工作流画布权威关系 v1

日期：2026-06-01

## 结论

项目工作流画布的权威事实源是项目 workflow state，后续补项目黑板后，是项目 workflow state + 项目黑板 + 控制核心确认后的事件和审计。

独立可编辑画布 `CanvasView` 和 `src-tauri/src/mcp/**` 的 canvas/run/audit 文件层暂定为实验性画布能力、模板编辑能力或后置能力，不是当前项目工作流的权威事实源。

大白话：

- 项目里看到的工作流，必须以项目自己的工作流事实为准。
- 独立画布可以保留，但现在不能和项目工作流抢“谁是真的”。
- MCP 画布运行逻辑不能绕过项目规则、权限、控制核心和审计，直接改项目工作流事实。

## 依据

- `docs/workbench-system-architecture-v1.md` 明确项目是最高级业务对象，控制核心负责项目身份、工作流状态机、权限、审计和候选转事实。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md` 把“项目工作流画布和独立可编辑画布权威不清”列为偏离点。
- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md` 确认：
  - `ProjectsView.tsx` 的项目工作流画布读取 `workflowState` / `derived_workflow`。
  - `CanvasView.tsx` 读取和保存独立 `CanvasDefinition`。
  - `src-tauri/src/mcp/**` 有独立 canvas/run/audit 文件层，并能启动 Codex。

## 接受

- 项目详情里的工作流画布读取项目 workflow state 派生读模型。
- 后续项目黑板落地后，项目画布可以展示黑板中的候选、风险、汇报、权限请求和记忆候选。
- 独立 `CanvasView` 可以作为实验画布或模板编辑器保留。
- 如果未来要把独立画布并入项目工作流，必须另开迁移计划，把 `CanvasDefinition`、`CanvasNode`、`CanvasRunState` 映射到项目 workflow、workflow node、workflow run、blackboard entry、event 和 audit。
- MCP canvas/run 文件层如要影响项目工作流，必须通过应用服务、控制核心、适配器、事件和审计。

## 不接受

- 不接受独立 canvas 文件成为项目工作流事实源。
- 不接受 MCP 工具直接推进项目工作流状态。
- 不接受独立画布运行逻辑绕过项目权限和项目隔离。
- 不接受把工作台改成通用节点执行器。
- 不接受因为有 React Flow 或 MCP 画布，就把项目 workflow state 降级为展示缓存。

## 对当前代码的影响

本决策不要求立刻改代码。

后续 UI 和架构收敛时：

- 项目页的工作流入口应以项目 workflow state 读模型为准。
- `CanvasView` 不应继续扩大为项目主工作流事实源。
- `src-tauri/src/mcp/orchestrator.rs`、`src-tauri/src/mcp/tools.rs` 的运行逻辑暂不纳入 Task B 第一批拆模块。

## 未定

- 独立画布最终是模板编辑器、实验区，还是并入项目工作流的一种编辑模式，目前未定。
- 若要合一，需要另开详细迁移和验收计划。
