# 桌面壳工作流节点派发 Codex 指令 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“桌面工作流节点派发 Codex 指令 v1 代码路径已实现”。

不接受为“真实 safe probe 已从桌面工作流执行”，不接受为“真实业务自动工作流已完成”，不接受为“长任务、权限确认、失败重试、超时取消或总指导回收判断已完成”。

## 薄弱点

- 本轮没有执行真实 `codex exec resume`。依据：handoff 明确 `Real safe probe dispatch executed: no`。
- 本轮没有写真实 `/Users/yoyi/.codex`。依据：handoff 明确 `Wrote /Users/yoyi/.codex: no`。
- 本轮没有写真实 workflow state。依据：handoff 明确只用测试临时状态。
- 用户审核后的真实业务派发仍被阻止。依据：handoff 明确 reviewed-business dispatch 在 v1 中因字段和协议不完整而 blocked。
- 这轮只证明实现路径和离线/单元验证，不证明真实业务长任务稳定。

## 接受内容

接受后端能力：

- 新增 `prepare_workflow_node_dispatch`。
- 新增 `execute_workflow_node_dispatch`。
- 新增 `read_workflow_node_dispatch_result`。
- 工作台状态扩展 `workflow_node_dispatches[]`。
- 支持 safe probe prompt：`请只回复这一句：WORKFLOW_NODE_DISPATCH_OK_2026_05_29`。
- safe probe 成功路径可将工作项从 `ready_to_dispatch` 推进到 `running` 再到 `ready_for_review`。
- 派发记录只保存最终回复摘要和 transcript 统计，不保存完整 transcript。
- Rust 测试使用临时 workflow state 和 stub Codex runner。

接受前端能力：

- 项目工作流卡片增加“派发指令”区域。
- 显示绑定会话、safe probe 预览和最近派发摘要。
- safe probe 派发走确认弹层。
- 确认弹层说明会写 `/Users/yoyi/.codex`、会写 workflow state、不读取授权或密钥、不运行 harness、不删除/移动/归档会话。
- “审核后派发”入口保留但禁用，字段和协议不完整时不允许真实业务派发。

## 验证依据

开发线回传的验证结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，3 个离线交互测试。
- `npm run build` 通过。
- 使用指定 `CARGO_HOME` / `CARGO_TARGET_DIR` 的 `cargo test --offline` 通过，56 passed，1 ignored。

代码符号复核：

- `prepare_workflow_node_dispatch`、`execute_workflow_node_dispatch`、`read_workflow_node_dispatch_result` 存在于 `src-tauri/src/lib.rs`。
- `workflow_node_dispatches[]`、`workflow_node_dispatch_prepared`、`workflow_node_dispatch_started`、`workflow_node_dispatch_completed`、`workflow_node_dispatch_failed`、`workflow_node_dispatch_readback_completed` 存在于实现中。
- 前端和离线测试中存在 safe probe 文案 `WORKFLOW_NODE_DISPATCH_OK_2026_05_29`。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有执行真实 Codex 派发。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取 `auth.json`、`.env`、授权文件或密钥。
- 没有触碰真实业务会话。
- 没有运行 harness。
- 没有保存完整 transcript 到 evidence、handoff 或 workflow state。
- 没有把用户审核业务派发包装成已可用能力。

## 当前可以说

- 桌面工作流节点已经具备受控 Codex 指令派发的代码路径。
- 工作台状态已经有派发记录结构。
- UI 已有 safe probe 预览、确认和最近派发摘要。
- 离线验证和 Rust stub 路径通过。

## 仍不能说

- 真实桌面 safe probe 派发已经执行。
- 工作台已经写回真实 workflow state。
- 真实业务自动编排已完成。
- 长任务、工具权限确认、失败重试、超时取消或总指导回收判断已完成。

## 下一步建议

下一步建议先派“工作流节点 safe probe 真实确认派发 v1”。

目标：

- 只用无业务 safe probe。
- 用户明确批准后，从真实工作台状态和已绑定测试会话执行一次派发。
- 记录目标 thread id、写入边界、最终回复摘要、transcript 统计和 workflow state 变更。
- 不执行真实业务任务，不运行 harness，不做并发和失败重试。

通过后再进入：

- 派发结果读回 UI / 总指导回收意见 v1。
- 用户审核业务指令 prompt schema。
- 长任务、权限确认、失败重试、超时取消协议。
