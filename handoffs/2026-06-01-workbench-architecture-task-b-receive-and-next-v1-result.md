# Handoff: Workbench Architecture Task B Receive And Next v1

日期：2026-06-01

## 结果

已接收 Task B 保守拆模块切片。

当前状态：

- Task A 架构只读审计：已完成。
- Task B 保守拆模块切片：已完成。
- 下一步建议：Task C 项目工作流画布权威收敛。

## Task B 接收判断

接受为：

- 后端类型定义已从 `lib.rs` 拆到 `src-tauri/src/types.rs`。
- Tauri command 包装已从 `lib.rs` 拆到 `src-tauri/src/commands.rs`。
- 前端 editable canvas 纯类型已从 `src/lib/types.ts` 拆到 `src/lib/types/canvas.ts`。
- 原前端类型入口仍继续转导出 canvas 类型。

不接受为：

- 最终 Rust 模块边界完成。
- workflow 读模型已拆。
- WorkbenchSnapshot 组装已拆。
- 状态机、Codex runner、工作流机器或 MCP 画布运行逻辑已收敛。

## 下一轮建议

派发 Task C，任务口径：

- 明确项目页工作流画布是当前项目工作流主入口。
- 独立 `CanvasView` 暂定为实验/模板/后置能力，不作为项目 workflow state 的事实源。
- 梳理项目页、全局 workflow 入口、右侧运行入口如何展示，避免两个入口都像权威画布。
- 只做小步 UI/入口/文档收敛，不直接合并两套数据模型。

禁止：

- 不执行真实 Codex。
- 不启动 MCP canvas run。
- 不改 workflow state JSON。
- 不把独立 canvas 文件层改成项目事实源。
- 不把工作台改成通用节点执行器。

## 已改文件

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

## 验证

- 本轮只改文档，未运行代码测试。
- 已抽查 Task B 改动入口和 evidence/handoff。
