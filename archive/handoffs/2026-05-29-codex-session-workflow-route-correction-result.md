# Handoff: Codex 会话管理与工作流编排纠偏

## 薄弱点

这轮只是方向纠偏和权威文档更新，不是代码实现。没有读取真实会话正文，没有验证 Codex 会话创建或发送消息，也没有启动自动编排。

## 结论

产品主线已从“任务包管理器”纠偏回“Codex 会话管理 + Codex 工作流编排”。

当前第一优先级：

1. 正确读取 Codex 会话，包括完整正文、工具调用、工具结果和时间线。
2. 确认能否在工作台里创建、继续、发送和接收 Codex 会话。
3. 把总指导、执行会话、回收评审做成可自动跑的工作流。

任务包继续保留，但只作为内部协议和可导出交接物，不再作为主界面中心。

## 改动文件

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/decisions/2026-05-29-ui-reference-sources.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/evidence/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/handoffs/2026-05-29-codex-session-workflow-route-correction-result.md`

## Codex++ 吸收口径

Codex++ 作为参考源保留。

可参考：

- 外部 launcher / 管理工具思路。
- 不修改 Codex 原始安装文件。
- 会话管理、Markdown 导出、Timeline、项目移动。
- Tauri + React + Rust 数据层组织。

不直接吸收：

- 会话删除。
- 写 Codex 配置或供应商切换。
- CDP 注入作为唯一控制路线。
- 用户脚本注入。
- 中转站功能。

## 新阶段顺序

1. Codex 会话全文读取 v1。
2. Codex 会话控制能力探针 v1。
3. Codex 工作流编排运行模型 v1。
4. 再把任务包能力藏进工作流内部。

## 禁止事项复核

- 没写 `/Users/yoyi/.codex`。
- 没改 Codex 状态库。
- 没读真实会话正文。
- 没展示密钥、授权文件、`.env`、token 或 API key。
- 没启动 Codex CLI。
- 没运行 harness。
