# Handoff: Workbench Architecture Task A Follow-up Decisions v1

日期：2026-06-01

## 结果

Task A 架构只读审计已接收并同步到当前入口。

新增两个前置决策：

- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`

当前下一步：

- 派发 Task B 的保守切片：后端无行为变化拆模块。

Task B 第一批允许：

- 拆类型。
- 拆 Tauri command 包装。
- 拆 workflow 读模型。
- 拆 WorkbenchSnapshot 组装。
- 拆前端纯类型。

Task B 第一批禁止：

- 不改状态机。
- 不改 workflow state JSON。
- 不碰真实 Codex resume。
- 不碰工作流机器。
- 不碰 MCP 可编辑画布运行逻辑。
- 不改任务包产品规则。

## 已改文件

- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`
- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

## 验证

- 本轮只改文档，未运行代码测试。
- 需要后续自检确认没有改 `prototypes/**` 代码。

## 边界

- 没有执行真实 Codex。
- 没有读取 `/Users/yoyi/.codex`。
- 没有读取密钥、`.env`、token、完整 transcript。
- 没有写真实业务项目目录。
- 没有做数据库迁移。
