# 任务包：工作流可控执行协议 v1

## 任务名

工作流可控执行协议 v1。

## 所属开发线

桌面应用线 / Codex 会话线 / 总指导线。

验证线按需复核。

## 当前判断

无业务 safe probe 已经完成真实派发、读回和总指导 review 落账。

下一步不能直接做真实业务自动编排。原因是长任务、权限确认、失败重试、超时取消和用户审核业务指令 schema 还没有协议和 UI 承接。

依据：

- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-dispatch-result-readback-ui-and-director-review-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-director-review-real-state-write-v1.md`

## 薄弱点

- 当前只证明一次无业务测试指令闭环，不证明真实业务自动工作流。
- 当前 review 落账不自动推进 work item 状态。
- 当前没有长任务恢复、权限确认、失败重试、超时和取消。
- 当前没有用户审核业务指令的结构化入口。

## 目标

定义并实现工作流可控执行协议的最小版本：

1. 增加用户审核业务指令的结构化 schema 和预览。
2. 增加长任务状态字段和 UI 展示。
3. 增加权限确认队列的数据结构和确认入口。
4. 增加失败、重试、超时、取消的状态规则。
5. 增加审计事件口径。
6. 保持 safe probe 和真实业务指令的边界分离。

大白话目标：

让工作台先知道“任务跑久了怎么办、要权限怎么办、失败怎么办、用户审核过的业务指令怎么安全进入”，再谈真实业务自动跑。

## 非目标

- 不执行真实业务任务。
- 不执行 `codex exec resume`。
- 不发送新的 safe probe。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。
- 不读取授权、密钥、`.env`、token。
- 不运行 harness。
- 不做多 agent 接入。
- 不做项目团队工作区 v1 表达层。

## 允许读取

允许读取项目内：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/STAGE_PLAN.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-director-review-real-state-write-v1.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许只读真实 workflow state 的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 完整 transcript 正文

## 允许写入

允许写入项目内：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`

本任务默认不写真实 workflow state。

如确实需要写真实 workflow state，只能写协议字段夹具或空队列初始化，并且必须单独获得用户确认。

## 禁止事项

- 禁止执行 `codex exec resume`。
- 禁止发送任何消息到 Codex 会话。
- 禁止写 `/Users/yoyi/.codex`。
- 禁止读取完整 transcript。
- 禁止读取授权、密钥、`.env`、token。
- 禁止运行 harness。
- 禁止把协议实现说成真实业务自动工作流完成。
- 禁止在 shell 双引号里写未转义反引号模式；搜索包含反引号的文本时必须使用单引号或 `rg -F`。

## 建议数据模型

按现有 schema 保守扩展。不要为了本任务重写整个 workflow state。

建议字段或结构：

- `workflow_execution_controls[]`
- `permission_requests[]`
- `execution_attempts[]`
- `timeout_policy`
- `retry_policy`
- `cancel_requested_at`
- `user_reviewed_instruction`

建议状态：

- `ready_to_dispatch`
- `running`
- `waiting_for_permission`
- `retry_pending`
- `failed`
- `timed_out`
- `cancelled`
- `ready_for_review`

建议审计事件：

- `workflow_execution_control_defined`
- `workflow_permission_requested`
- `workflow_permission_decision_recorded`
- `workflow_execution_retry_scheduled`
- `workflow_execution_timeout_recorded`
- `workflow_execution_cancel_requested`

## UI 要求

- 工作项上能看见当前执行控制状态。
- 能看见权限请求队列。
- 能看见失败原因、重试次数、超时和取消状态。
- 用户审核业务指令必须有预览和确认边界。
- UI 文案不能暗示真实业务已自动运行。

## 验收标准

必须满足：

- 用户审核业务指令 schema 存在，并有前端预览或后端渲染。
- 长任务状态协议能在 UI 中展示。
- 权限确认队列有数据结构和 UI 入口。
- 失败、重试、超时、取消规则有代码路径或测试夹具。
- 有审计事件设计或最小实现。
- 不执行 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不读取完整 transcript。

建议验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

安全搜索要求：

- 搜索普通固定文本时优先使用 `rg -F '固定文本' ...`。
- 搜索包含反引号的文本时必须使用单引号包裹，或使用 `rg -F` 固定字符串模式。
- 禁止写成 shell 双引号里的未转义反引号模式；这会触发 shell 命令替换。

## 必须回传

回传必须包含：

1. 薄弱点。
2. 做了什么。
3. 改了哪些文件。
4. 是否写真实 workflow state。
5. 是否写 `/Users/yoyi/.codex`。
6. 是否执行 `codex exec resume`。
7. 是否发送 Codex 消息。
8. 新增或修改了哪些字段类型。
9. 新增 evidence / handoff。
10. 测试命令和结果。
11. 下一步建议。

## 总指导回收重点

总指导回收时必须判断：

- 是否仍然只是协议能力，不是真实业务自动编排。
- 是否把权限、失败、超时、取消讲清楚。
- 是否有测试覆盖坏路径。
- 是否误写 `/Users/yoyi/.codex` 或读取敏感文件。

通过后再考虑第一条用户确认后的真实业务小步试跑。
