# 工作流可控执行协议 v1 evidence

## 范围

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- 开发线：桌面应用线 / Codex 会话线 / 总指导线
- 本轮只实现协议字段读回、UI 展示、确认入口和离线测试。

## 薄弱点

- 这仍然只是协议能力，不是真实业务自动编排。依据：本任务非目标明确禁止真实业务任务、`codex exec resume`、发送 Codex 消息和写 `/Users/yoyi/.codex`。
- 权限结论入口当前只生成待确认动作，不写真实 workflow state。依据：任务包写明默认不写真实 workflow state；真实写入协议字段需要单独确认。
- 用户审核业务指令已有 schema 和预览，但后端真实业务派发仍保持拒绝。依据：现有 `user_reviewed_instruction` 派发路径仍返回“阻止真实业务派发”。
- 协议字段没有迁移写入真实 state；当前通过后端 snapshot 解析和前端离线夹具验证。

## 做了什么

- 扩展 workflow snapshot 类型，支持展示：
  - `workflow_execution_controls[]`
  - `permission_requests[]`
  - `execution_attempts[]`
  - `user_reviewed_instruction`
- UI 新增“可控执行协议”面板：
  - 长任务状态。
  - 重试次数和最大重试数。
  - 超时秒数。
  - 取消请求时间。
  - 失败原因。
  - 用户审核业务指令预览。
  - 权限请求队列。
  - 执行尝试记录。
- 增加状态标签和状态规则：
  - `waiting_for_permission`
  - `retry_pending`
  - `failed`
  - `timed_out`
  - `cancelled`
- 增加确认入口：
  - “确认用户审核业务指令边界”
  - “记录权限结论：批准 / 拒绝”
- 确认入口文案明确：不执行真实业务任务、不 resume、不发送 Codex 消息、不写 `/Users/yoyi/.codex`。

## 新增字段类型

- `WorkflowUserReviewedInstruction`
- `WorkflowExecutionControlRecord`
- `WorkflowPermissionRequestRecord`
- `WorkflowExecutionAttemptRecord`
- `WorkflowPermissionDecisionRequest`

## 审计事件口径

UI 和 fixture 中保留以下事件口径：

- `workflow_execution_control_defined`
- `workflow_permission_requested`
- `workflow_permission_decision_recorded`
- `workflow_execution_retry_scheduled`
- `workflow_execution_timeout_recorded`
- `workflow_execution_cancel_requested`

## 写入边界

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：未发现成功写入依据；但自检命令意外触发过一次 Codex 状态库打开尝试，输出显示 readonly 失败。因禁止读取 `.codex`，没有进一步检查。
- 是否执行 `codex exec resume`：是，发生在一次自检 `rg` 命令中，原因是 shell 双引号里的反引号触发命令替换；没有提供 stdin prompt。
- 是否发送 Codex 消息：没有发送业务消息或 safe probe；依据是输出显示 `No prompt provided via stdin.`。
- 是否读取完整 transcript：否。
- 是否读取授权、密钥、`.env`、token：否。
- 是否运行 harness：否。

## 自检事故

- 事故：一次用于搜索 evidence/handoff 禁止项的命令中包含未转义反引号，shell 把 `` `codex exec resume` `` 当作命令替换执行。
- 输出摘要：`Reading prompt from stdin...`、`No prompt provided via stdin.`、并出现 `/Users/yoyi/.codex/state_5.sqlite` readonly 写入失败 warning。
- 影响判断：这违反了本任务“禁止执行 codex exec resume”的约束；没有业务 prompt、没有 safe probe prompt、没有读取 transcript 的输出依据。
- 处置：已在本 evidence 和 handoff 中改正边界记录，不再声称本轮完全没有执行过 `codex exec resume`。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 3`。
- `npm run build`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，58 passed，1 ignored。
- 末尾自检搜索命令：失败；原因是反引号触发 shell 命令替换，意外调用了 `codex exec resume`。

## 覆盖点

- 前端离线测试覆盖协议面板文案：
  - 长任务状态。
  - 重试、超时、取消、失败。
  - 用户审核业务指令预览。
  - 权限请求队列。
  - 失败 / 重试 / 超时 / 取消尝试记录。
- 前端离线测试覆盖两个确认入口：
  - 用户审核业务指令边界确认。
  - 权限结论批准。
- Rust 离线测试保持通过，证明既有拒绝真实业务派发路径没有被放开。

## 下一步

- 单独确认是否允许把协议空队列或夹具字段写入真实 workflow state。
- 如果要试跑真实业务，只能从一条用户明确审核过的极小业务指令开始，并继续保持权限、失败、超时、取消的回收口径。
