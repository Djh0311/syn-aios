# Evidence：工作流状态收口真实派发复测 v2

## 结论

v2 真实复测已通过。

本轮证明：

- 真实 `codex exec resume` completed 后，v2 work item 进入 `ready_for_review`。
- 实际派发节点 codex-dev 收口为 `ready_for_review`，没有残留 `running`。
- README 已追加目标行 `Workflow dispatch state closure retest passed.`。

本轮不证明：

- 复杂业务自动编排完成。
- 上轮 v1 超时根因已经定位。
- 所有长任务都不会超时。

## 薄弱点

- 这仍是极小 README 追加复测，不是复杂业务。
- 执行期间仍出现 remote plugin sync warning 和 MCP shutdown warning；本轮没有失败，但这些 warning 仍需要后续观察。
- v1 超时根因仍不确定，本轮只是用更短 prompt 和 600 秒 timeout 跑通 completed 路径。
- 本轮通过真实 `codex exec resume` 写了 `/Users/yoyi/.codex`。

## 用户批准

- 是否获得用户明确批准：是。
- 批准内容：
  - 执行真实 `codex exec resume`。
  - 写 `/Users/yoyi/.codex`。
  - 修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
  - 写真实 workflow state。

## 执行对象

- project root：`/Users/yoyi/codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- target line：`Workflow dispatch state closure retest passed.`

## 前置复核

- README 存在。
- 目标行执行前不存在。
- v2 work item 存在，state 为 `ready_to_dispatch`。
- v2 work item assigned role 为 `codex-dev`。
- v2 active binding 存在，node id 为 `workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`。
- binding thread 为 `019e7738-5e29-74e0-a22f-5c2481b64c38`。
- thread 在 `codex-index.json` 中，project root 为 `/Users/yoyi/codex-workflow-mario-test`，rollout 存在。
- 旧 `state-closure-retest` work item 保持 `timed_out`，没有回滚。

## 真实执行

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 README：是，只追加目标行。
- 是否修改允许范围外文件：否。
- 是否读取敏感文件：否。
- 是否读取完整 transcript：否；只读取 last-message 摘要。
- 是否运行 harness：否。
- 是否联网安装依赖：否；Codex 插件远端同步出现 warning，但不是本轮主动联网安装依赖。

真实派发命令形态：

- `codex exec -C /Users/yoyi/codex-workflow-mario-test --sandbox workspace-write --skip-git-repo-check --json --output-last-message /private/tmp/state-closure-retest-v2-last-message.txt resume 019e7738-5e29-74e0-a22f-5c2481b64c38 - < /private/tmp/state-closure-retest-v2-prompt.txt`

最终回复摘要：

- `README_UPDATED_STATE_CLOSURE_RETEST_V2`

transcript 统计：

- `input_tokens=191898`
- `cached_input_tokens=143104`
- `output_tokens=2211`
- `reasoning_output_tokens=1263`

## workflow state 写入

字段类型：

- `workflow_node_dispatches[]`
- `work_items[].state`
- `work_items[].current_node_id`
- `nodes[].state`
- `workflow_execution_controls[]`
- `execution_attempts[]`
- `audit_events[]`
- 顶层 `updated_at`

备份：

- running 前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780135214051.json`
- completed 写回前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780135354486.json`

dispatch：

- prepared dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2:1780135214051`
- completed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2:1780135214052`
- final dispatch state：`completed`
- exit code：`0`
- warnings：`remote_plugin_sync_warning`、`mcp_shutdown_warning`

状态收口：

- v2 work item state：`ready_for_review`
- v2 work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:review`
- codex-dev node state：`ready_for_review`
- completed 后 codex-dev 是否仍为 `running`：否。
- 如果超时，codex-dev 是否收口为 `timed_out`：不适用，本轮 completed。
- 旧 `state-closure-retest` work item：仍为 `timed_out`

execution control：

- `control_state=completed`
- `long_task_state=completed`
- `failure_reason=null`
- `timeout_seconds=600`

execution attempt：

- `state=completed`
- `exit_code=0`
- `failure_reason=null`
- `final_message_summary=README_UPDATED_STATE_CLOSURE_RETEST_V2`

audit event id：

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214051:1780135214051`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135214052`
- `audit:workflow-node-dispatch-completed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135354486`
- `audit:workflow-node-dispatch-readback:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2-1780135214052:1780135354486`

## 文件复核

- target line：存在，README 第 15 行。
- README hash：`8237ec576e3dae2ef1453e13e46a16e55bfe87140876ca6d49487962487a9c18`
- `index.html` hash：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 验证命令和结果

- `rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md`：`15:Workflow dispatch state closure retest passed.`
- `shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js`：通过，hash 如上；非目标三文件未变。
- `/Users/yoyi/miniconda3/bin/python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`：`validation_ok`。
- workflow state 摘要复核：v2 work item / dispatch / control / attempt 均 completed 或 ready_for_review，codex-dev node 为 `ready_for_review`，旧 work item 仍为 `timed_out`。

## Handoff

- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v2-result.md`
