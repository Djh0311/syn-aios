# 总指导回收意见：工作流可控执行协议 v1

## 回收对象

- 任务包：`/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- Evidence：`/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-controlled-execution-protocol-v1.md`
- Handoff：`/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-controlled-execution-protocol-v1-result.md`

## 结论

需要修改。

不是因为实现方向错误，而是因为本轮违反了任务包禁止项：意外执行过一次 `codex exec resume`。

## 薄弱点

- 任务包明确禁止执行 `codex exec resume`，本轮实际触发过一次。即使输出为 `No prompt provided via stdin.`，也不能按干净完成验收。
- 这轮仍是协议能力，不是真实业务自动编排。
- 权限结论入口目前只生成待确认动作，不写真实 workflow state。
- 真实 workflow state 中没有写入 `workflow_execution_controls[]`、`permission_requests[]`、`execution_attempts[]`。

## 接受的部分

以下部分方向正确，可以保留：

- 工作台展示长任务状态、权限请求、失败、重试、超时、取消。
- 用户审核业务指令 schema 和预览已出现。
- 后端仍拒绝 `user_reviewed_instruction` 真实派发，没有开放真实业务任务。
- 前端确认弹层明确不 resume、不发送消息、不写 `/Users/yoyi/.codex`。
- evidence 和 handoff 没有掩盖事故，边界记录诚实。

## 不能接受的部分

- 不能接受为“工作流可控执行协议 v1 已完成”。
- 不能接受为“下一步可以直接真实业务试跑”。
- 不能接受为“本轮完全没有触碰 `codex exec resume`”。

## 复核结果

总指导复跑验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `cargo test --offline`：通过，58 passed，1 ignored。

只读复核真实 workflow state：

- `workflow_execution_controls`：未写入。
- `permission_requests`：未写入。
- `execution_attempts`：未写入。
- `reviews`：1。

## 必须修正

补一个小修任务，目标不是重做协议，而是防止同类事故再次发生：

1. 增加或修正本轮自检命令规范，禁止在 shell 双引号中直接写带反引号的模式。
2. 在任务包或验证说明里写明：搜索包含反引号的文本时使用单引号或 `rg -F`。
3. 如有必要，补一个脚本或文档片段，避免自检搜索误触发命令替换。
4. 回传新的 evidence / handoff，说明没有再次执行 `codex exec resume`。

## 下一步

先派“工作流可控执行协议 v1 事故防护小修”。

通过后再回收本阶段是否接受。
