# Evidence: Workbench Architecture Implementation Plan v1

日期：2026-06-01

## 做了什么

- 复核最终蓝图、UI 蓝图、当前软件架构草案、记忆层设计、工作流设计、当前权威入口和关键决策。
- 抽查当前 app 代码入口，重点看 `lib.rs`、`ProjectsView.tsx`、`CanvasView.tsx`、`src-tauri/src/mcp/**` 的职责边界。
- 新增架构落地执行计划：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`。
- 同步入口索引：`CURRENT.md`、`AUTHORITY.md`、`README.md`、`docs/plans/README.md`。

## 主要结论

- 当前 app 方向没有完全偏离最终蓝图，因为已经有项目、会话、工作流、右侧状态入口、Codex 编排和工作流状态。
- 当前代码架构已经偏离目标分层，因为控制核心、项目黑板、适配器、事实层和读模型还没有真正分开。
- 下一步不建议继续直接加功能，建议先执行计划里的 Task A：架构只读审计。

## 边界

- 本轮没有改代码。
- 本轮没有运行测试。
- 本轮没有读取 `/Users/yoyi/.codex`。
- 本轮没有执行 `codex exec` 或 `codex exec resume`。
- 本轮没有写真实业务项目目录。
- 本轮没有迁移数据库。

## 文件

- 新增：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- 更新：`CURRENT.md`
- 更新：`AUTHORITY.md`
- 更新：`README.md`
- 更新：`docs/plans/README.md`
