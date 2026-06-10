# 任务包：Codex 会话管理与工作流编排纠偏

## 所属开发线

总指导线。

## 背景

当前工作台开发方向一度偏向任务包管理：草稿、预览、字段编辑、生成文件、ready 检查。用户已明确纠正：现在最重要的是先把 Codex 内容正确读出来，再让用户能编排 Codex 工作流。

## 目标

完成路线纠偏：

- 把当前阶段从任务包管理器纠回 Codex 会话管理和 Codex 工作流编排。
- 把 Codex++ 记录为 Codex 会话管理参考源。
- 更新阶段计划、任务队列、开发线和参考源决策。
- 明确下一步顺序：会话全文读取、会话控制探针、工作流编排运行。

## 允许读取

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/decisions/*.md`
- `product-line/evidence/*.md`
- `product-line/handoffs/*.md`
- 用户提供的 Codex++ GitHub 仓库 README 和目录元数据

## 允许写入

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/decisions/2026-05-29-ui-reference-sources.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/evidence/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/handoffs/2026-05-29-codex-session-workflow-route-correction-result.md`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不读真实会话正文。
- 不展示密钥、授权文件、`.env`、token 或 API key。
- 不启动 Codex CLI。
- 不运行 harness。
- 不改前端、Rust 或索引内核代码。

## 验收标准

- 阶段计划把阶段 3 明确改成 Codex 会话读取、会话控制和工作流编排。
- 任务队列不再建议继续做 ready 任务包。
- Codex++ 被记录为参考源，且写清楚只吸收会话管理参考，不照搬删除、注入、provider 写入。
- 新增 Codex 会话线，并写清职责、边界和禁止事项。
- 任务包能力被降级为内部协议和可导出交接物。

## 必须回传

- 改了哪些文件。
- 新增了哪些 evidence / handoff / decision。
- 当前下一步是什么。
- 哪些旧方向被暂停或降级。
- 是否触碰 Codex 状态库或真实会话正文。
