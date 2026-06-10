# Review：用户审核业务派发真实 README 极小验证 v1

## 结论

接受，但只接受为极小真实写入闭环。

接受为：

- 用户审核业务派发路径已真实执行一次。
- 绑定 Codex 会话可以通过 `codex exec resume` 接收用户审核业务指令。
- 允许范围内的 `/Users/yoyi/codex-workflow-mario-test/README.md` 已追加目标行。
- workflow state 已记录 dispatch、execution control、execution attempt 和 audit event。
- work item 已进入 `ready_for_review`。

不接受为：

- 复杂业务自动编排完成。
- 长任务协议已真实稳定。
- 权限确认队列、失败重试、取消、超时已经真实跑通。
- 允许范围外文件经过全仓库扫描确认未变。

## 薄弱点

- 本轮只改 README 一行，任务复杂度太低。
- 允许范围外文件未变的依据是指定文件 hash 和 Codex 回传，不是全仓库 diff。
- workflow state 中 codex-dev 节点仍是 `running`，但 work item 已是 `ready_for_review`；这是状态收口缺口。
- `transcript_event_count` 和 `transcript_target_hits` 为空；本轮没有保存完整 transcript，符合边界，但也减少了后续自动质量判断的信息量。

## 回收依据

已复核：

- `evidence/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-result.md`
- `/Users/yoyi/codex-workflow-mario-test/README.md`
- 真实 workflow state 摘要

## 关键复核结果

### README 修改

接受。

依据：

- `rg -n -F 'Workflow dispatch smoke passed.' /Users/yoyi/codex-workflow-mario-test/README.md` 命中第 14 行。

### dispatch

接受。

依据：

- prepared dispatch 存在。
- completed dispatch 存在。
- completed dispatch `exit_code=0`。
- warnings 数量为 0。

### execution control / attempt

接受。

依据：

- execution control 存在。
- `control_state=ready_for_review`。
- `long_task_state=completed`。
- execution attempt 存在。
- attempt state 为 `completed`。
- failure reason 为空。

### work item 状态

接受。

依据：

- README smoke work item 为 `ready_for_review`。
- current node 已指向 review 节点。

### node 状态

需要修改。

依据：

- codex-dev node 仍为 `running`。
- 这和 work item 已 `ready_for_review` 不一致。

判断：

- 这不阻止接受本轮极小真实闭环。
- 但它阻止把当前状态机说成已经完整可靠。

## 边界复核

- 是否执行真实 `codex exec resume`：是，且用户已明确批准。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 README：是。
- 是否读取敏感文件：未见依据。
- 是否读取完整 transcript：未见依据，记录显示只保存 last-message 摘要。
- 是否运行 harness：否。

## 回收决定

本轮通过。

下一步建议：

- 写任务包修正 workflow node 状态收口：当 dispatch completed 且 work item 进入 `ready_for_review` 时，实际派发节点不应继续停在 `running`。
- 暂不进入复杂业务自动编排，先把状态机的收口和 review 落账闭合。
