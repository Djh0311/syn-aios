# 第一版信息架构回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-v1-information-architecture.md`
- 开发线：信息架构线
- 回传 evidence：`product-line/evidence/2026-05-27-v1-information-architecture.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-v1-information-architecture-result.md`

## 结论

接受为第一版信息架构 handoff。

这份 handoff 可作为桌面应用线的页面结构输入，但不是 UI 框架决策，也不是最终字段协议。

## 先说薄弱点

- 任务包原先“所属开发线”写成桌面应用线，这是总指导层遗留错误。已修正为信息架构线。
- 信息架构线额外读取了任务队列、回收文档和索引 evidence，超出任务包原允许读取清单。其 evidence 已说明该越界仅用于对齐格式和既有状态，没有读取密钥、授权文件、`.env` 或 Codex 原始状态库。接受但记录风险。
- 页面字段仍是 UI 层设计字段，不能当最终索引 schema。
- harness 作用判断仍没有证据自动化，第一版只能做候选台账和人工判断位。

## 接受依据

- handoff 定义了 6 个第一版页面：首页、项目页、会话页、skills 页、harness 页、任务线页。
- 每个页面都说明了用途、展示字段、字段来源、未知点和第一版不做事项。
- handoff 明确区分 ERP 和游戏项目的差异化字段。
- handoff 明确第一版不嵌入 Codex 聊天窗口、不写 Codex 状态库、不迁移/删除/归档会话、不自动安装 skills、不自动运行 harness、不接 OpenClaw / VS Code / Claude Code、不做移动端。
- handoff 明确阶段 2 / 3 / 4 的边界。
- 没有写应用代码，没有引入 UI 框架结论。

## 当前生效结论

- 桌面应用线后续以项目为主轴组织页面，不先按 agent 组织。
- 第一版固定 6 个页面：首页、项目页、会话页、skills 页、harness 页、任务线页。
- 所有无依据字段显示“未知”或“缺少依据”，不能空白吞掉。
- 会话标题和敏感上下文字段默认收紧展示。
- `.codex-global-state.json` 只能作为 UI 提示来源，不能覆盖项目归属。
- harness 页保留人工判断位，但第一版只读，不写回。

## 派生影响

- 桌面应用线可以在索引内核 hardening 完成后接桌面应用壳任务。
- 当前不新增工作线。

## 状态

已回收，接受。
