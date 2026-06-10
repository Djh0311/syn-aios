# 决策：架构拆模块保护边界 v1

日期：2026-06-01

## 结论

下一步可以进入架构计划 Task B，但只能执行“无行为变化拆模块”的保守切片。

第一批允许拆：

- 类型定义。
- Tauri command 包装层。
- workflow 读模型派生函数。
- WorkbenchSnapshot 组装。
- 前端类型文件的纯类型拆分。

第一批不能碰：

- workflow state JSON 结构。
- 状态机语义。
- 备份、原子写、真实 state 写入时机。
- 真实 Codex resume 执行路径。
- 工作流机器。
- MCP 可编辑画布运行逻辑。
- 任务包产品规则。

大白话：

先把文件变清楚，不顺手改业务。

## 依据

- `docs/workbench-system-architecture-v1.md` 要求先写 schema、状态机、权限规则、事件规则、审计规则、端口接口，再写实现。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md` 的 Task B 目标是“后端无行为变化拆模块”。
- `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md` 判断：
  - 可以先拆类型、命令包装、读模型和快照组装。
  - 不建议第一批拆状态机、工作流机器、Codex runner、MCP canvas run 和任务包产品规则。

## 允许范围

允许：

- 移动 Rust 类型到目标模块。
- 移动纯读模型函数到 `read_model/**`。
- 移动 Tauri command 包装函数到 `commands/**`，但命令名不变。
- 移动 WorkbenchSnapshot 组装逻辑到读模型模块。
- 移动前端纯类型到多个文件后继续 barrel export。
- 增加模块文件和 `mod` 声明。
- 增加或移动纯测试辅助函数，只要断言不变。

必须保持：

- Tauri command 名字不变。
- 请求字段、响应字段、serde 字段名不变。
- 前端公开类型名不变。
- JSON workflow state 结构不变。
- 现有测试语义不变。

## 禁止范围

禁止：

- 不重命名状态值。
- 不改变状态转移规则。
- 不改变任务包生成、预览、派发 readiness 的业务规则。
- 不改变真实 Codex 执行参数。
- 不执行真实 `codex exec` 或 `codex exec resume`。
- 不读取 `/Users/yoyi/.codex`。
- 不迁移 SQLite。
- 不把独立 `CanvasView` 和项目 workflow state 合一。
- 不修改真实业务项目目录。

## 必跑验证

如果改 Rust：

- 跑相关 Rust 聚焦测试。
- 跑 `cargo test --lib`。

如果改前端类型或 import：

- 跑 `npm run typecheck`。
- 跑相关前端测试。
- 跑 `npm run build`。

如果 Task B 同时改 Rust 和前端：

- 跑 `npm run typecheck`。
- 跑 `npm run test:offline-interaction`。
- 跑 `npm run build`。
- 跑 `cargo test --lib`。

## 停止条件

遇到以下情况必须停止，不要继续硬拆：

- 移动函数需要改变状态机语义。
- 模块循环依赖迫使改业务规则。
- 需要真实 Codex 执行才能验证。
- 需要读取 `/Users/yoyi/.codex`。
- 需要改变 workflow state JSON。
- 需要修改 MCP 画布运行逻辑。
- 需要把任务包重新提升为主界面。

## 对下一轮的影响

下一轮可以派 Task B，但任务包必须写明：

- 本轮只做第一批低风险拆分。
- 不碰高风险路径。
- 任何行为变化都要退回。
