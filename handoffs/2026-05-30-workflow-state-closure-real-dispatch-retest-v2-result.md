# Handoff：工作流状态收口真实派发复测 v2

## 结论

真实复测 v2 已通过。

README 已追加 `Workflow dispatch state closure retest passed.`，v2 dispatch completed，v2 work item 进入 `ready_for_review`，codex-dev 节点收口为 `ready_for_review`，没有残留 `running`。

## 薄弱点

- 这只是极小 README 追加复测，不是复杂业务自动编排。
- v1 超时根因仍不确定。
- 本轮执行期间仍有 remote plugin sync warning 和 MCP shutdown warning，虽然没有导致失败。
- 本轮通过真实 `codex exec resume` 写了 `/Users/yoyi/.codex`。

## 边界

- 是否获得用户明确批准：是。
- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是。
- 是否写真实 workflow state：是。
- 是否修改 README：是。
- 是否修改允许范围外文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 执行对象

- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- cwd：`/Users/yoyi/codex-workflow-mario-test`
- sandbox：`workspace-write`
- timeout：600 秒
- final message：`README_UPDATED_STATE_CLOSURE_RETEST_V2`

## 写入结果

- README：目标行存在于第 15 行。
- dispatch：`completed`
- exit code：`0`
- work item state：`ready_for_review`
- work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:review`
- codex-dev node state：`ready_for_review`
- completed 后 codex-dev 是否仍为 `running`：否。
- 超时收口：不适用，本轮未超时。
- 旧 `state-closure-retest` work item：仍为 `timed_out`。

## 备份和审计

备份：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780135214051.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780135354486.json`

audit event id：

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214051:1780135214051`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135214052`
- `audit:workflow-node-dispatch-completed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135354486`
- `audit:workflow-node-dispatch-readback:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135354486`

写入字段类型：

- `workflow_node_dispatches[]`
- `work_items[].state`
- `work_items[].current_node_id`
- `nodes[].state`
- `workflow_execution_controls[]`
- `execution_attempts[]`
- `audit_events[]`
- 顶层 `updated_at`

## transcript 统计

- `input_tokens=191898`
- `cached_input_tokens=143104`
- `output_tokens=2211`
- `reasoning_output_tokens=1263`

## 验证

- README 目标行搜索：命中第 15 行。
- 业务文件 hash：README 已变化；`index.html`、`styles.css`、`game.js` hash 未变。
- 索引校验：`validation_ok`。
- workflow state 摘要：v2 work item `ready_for_review`，dispatch `completed`，control / attempt `completed`，codex-dev node `ready_for_review`。

## 下一步建议

总指导可回收为接受：接受为 completed 成功路径下实际派发节点不残留 `running` 的真实复测；不接受为复杂业务自动编排完成。
