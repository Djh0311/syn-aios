# Handoff：task package UI display boundary rule v1

日期：2026-06-04

## 结论

已把“之前确认过的 UI 显示方案”升级为任务包写作硬规则。

新增规则文档：

- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

后续任何可能改前端、读模型展示、UI 文案、导航入口、右侧入口、项目页、画布、记忆、知识库、智能体、秘书或管理入口的任务包，都必须包含“UI 显示边界确认”固定章节。

## 已更新

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `evidence/2026-06-04-task-package-ui-display-boundary-rule-v1.md`
- `handoffs/2026-06-04-task-package-ui-display-boundary-rule-v1-result.md`

## 当前 M3 状态

M3 任务包已经补入：

- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- “UI 显示边界确认”固定章节
- M3 允许显示 / 禁止显示 / 显示位置 / 中间版本范围 / 后端依赖 / 文案边界 / UI 验收要求

## 后续执行要求

如果未来任务包涉及 UI 但没有该固定章节，不能直接执行；先补任务包。

如果任务包声明不改前端，也必须写明：

```text
UI 显示边界：本任务不改前端、不改读模型、不改 UI 文案；因此不需要 UI 验收。
```

## 边界

- 本轮只改文档和任务包。
- 未改产品代码。
- 未读取 `/Users/yoyi/.codex`。
- 未执行真实 Codex。
- 未运行测试。
