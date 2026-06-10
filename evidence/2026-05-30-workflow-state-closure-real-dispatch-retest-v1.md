# Evidence：工作流状态收口真实派发复测 v1

## 结论

真实复测派发已执行，但未通过验收。

本轮证明：

- 真实 `codex exec resume` 超时失败后，workflow state 没有让 codex-dev 节点残留 `running`。
- 超时后 work item 和 codex-dev 节点已收口为 `timed_out`。

本轮未证明：

- completed 成功路径的真实复测。
- README 目标行追加成功。
- 复杂业务自动编排完成。

## 薄弱点

- 首次尝试使用 `-C /Users/yoyi --sandbox workspace-write --add-dir /Users/yoyi/codex-workflow-mario-test` 被拒绝；原因是写入范围覆盖整个 home，超过任务范围。
- 改用更窄的 `-C /Users/yoyi/codex-workflow-mario-test --sandbox workspace-write` 后，真实 resume 启动，但超过约 300 秒未完成。
- 期间出现插件目录鉴权、MCP 进程组终止、远端插件同步超时、GitHub rate limit warning。
- 目标 README 行没有写入。
- last-message 文件为空或未生成有效最终回复。
- 本轮不应回收为 completed 成功复测，只能回收为超时失败且状态收口未挂死。

## 用户批准

- 是否获得用户明确批准：是。
- 批准依据：用户回复“开始完成任务”，在我明确说明会执行真实 `codex exec resume`、写 `/Users/yoyi/.codex`、修改测试 README、写真实 workflow state 后继续执行。

## 执行对象

- project root：`/Users/yoyi/codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- target line：`Workflow dispatch state closure retest passed.`

## 真实执行

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 README：否，目标行仍不存在。
- 是否修改允许范围外文件：否，`README.md`、`index.html`、`styles.css`、`game.js` hash 均未变。
- 是否读取敏感文件：未见依据。
- 是否读取完整 transcript：否；只使用 last-message 摘要路径，且最终为空。
- 是否运行 harness：否。
- 是否联网安装依赖：否；但 Codex 插件启动过程尝试远端同步并触发 warning / rate limit。

## 派发命令

被拒绝的较宽命令：

- `codex exec -C /Users/yoyi --sandbox workspace-write --add-dir /Users/yoyi/codex-workflow-mario-test ... resume 019e7738-5e29-74e0-a22f-5c2481b64c38`
- 结果：拒绝，理由是 `-C /Users/yoyi` 下 `workspace-write` 写入范围过大。

实际执行的较窄命令：

- `codex exec -C /Users/yoyi/codex-workflow-mario-test --sandbox workspace-write --skip-git-repo-check --json --output-last-message <last-message-path> resume 019e7738-5e29-74e0-a22f-5c2481b64c38 -`
- 结果：超过任务超时上限后停止，按 `timed_out` 写回。

## workflow state 写入

写入字段类型：

- `workflow_node_dispatches[]`
- `work_items[].state`
- `work_items[].current_node_id`
- `nodes[].state`
- `workflow_execution_controls[]`
- `execution_attempts[]`
- `audit_events[]`
- 顶层 `updated_at`

备份：

- running 前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780128642386.json`
- 超时写回前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780129407652.json`

dispatch：

- prepared dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest:1780128642386`
- running / failed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest:1780128642387`
- final dispatch state：`failed`
- exit code：`-1`
- warnings：`timeout`

状态收口：

- work item state：`timed_out`
- work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- codex-dev node state：`timed_out`
- completed 后 codex-dev 是否仍为 `running`：不适用，因为没有 completed；超时后 codex-dev 不为 `running`。

execution control：

- `control_state=timed_out`
- `long_task_state=timed_out`
- `failure_reason=timeout`
- `timeout_seconds=300`

execution attempt：

- `state=timed_out`
- `failure_reason=timeout`
- `timed_out_at=1780129407652`

audit event id：

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780128642386`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780128642387`
- `audit:workflow-node-dispatch-failed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780129407652`

## 文件复核

- target line：不存在。
- README hash：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html` hash：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 验证命令和结果

- `rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md || true`：无命中。
- `shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js`：通过，hash 如上。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- workflow state 摘要复核：work item / codex-dev node / execution control / attempt 均为 `timed_out`，没有 `running` 残留。

## Handoff

- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-result.md`
