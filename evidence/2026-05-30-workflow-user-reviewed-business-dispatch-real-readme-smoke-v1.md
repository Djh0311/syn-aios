# Evidence：用户审核业务派发真实 README 极小验证 v1

## 结论

真实 README smoke 已执行完成，结果等待总指导回收。

本轮证明：

- 桌面壳用户审核业务派发路径可以通过绑定 Codex 会话执行一次真实写入。
- `/Users/yoyi/codex-workflow-mario-test/README.md` 已追加目标行。
- workflow state 已记录 prepared / completed dispatch、execution control、execution attempt 和 audit event。

本轮不证明：

- 复杂业务自动编排已经完成。
- 长任务、权限确认、失败重试、取消和超时已经被真实验证。
- 允许范围外文件一定没有被全仓库扫描确认。

## 薄弱点

- 本轮只验证一个 README 追加行，不代表真实业务项目可以自动推进。
- Codex 回传说明“未修改允许范围外文件”的依据是本轮实际执行动作，不是全仓库扫描。
- workflow state 中 `codex-dev` 节点仍显示 `running`，而 work item 已进入 `ready_for_review`；这说明节点状态收口可能还不完整，需要后续修正或总指导判断。
- `transcript_event_count` 和 `transcript_target_hits` 这次为空；本轮保留了 last-message 摘要，没有保存完整 transcript。

## 用户批准

- 是否获得用户明确批准：是。
- 批准内容：执行真实 README smoke，允许真实 `codex exec resume`，允许写 `/Users/yoyi/.codex`，允许修改 `/Users/yoyi/codex-workflow-mario-test/README.md`，允许写真实 workflow state。

## 执行对象

- 项目路径：`/Users/yoyi/codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- instruction id：`user-reviewed-instruction:readme-smoke-v1`

## 真实写入情况

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 README：是。
- 是否修改允许范围外文件：未发现。依据：只读 hash 复核显示 `index.html`、`styles.css`、`game.js` hash 未变；没有做全仓库扫描。
- 是否读取敏感文件：否。未读取 `auth.json`、`.env`、密钥、token、授权文件。
- 是否读取完整 transcript：否。
- 是否运行 harness：否。
- 是否联网安装依赖：否。

## 文件结果

README 目标行：

- `/Users/yoyi/codex-workflow-mario-test/README.md:14`
- 内容：`Workflow dispatch smoke passed.`

hash 复核：

- `README.md`：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html`：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css`：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js`：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## workflow state 写入

写入字段类型：

- `workflow_node_dispatches[]`
- `workflow_execution_controls[]`
- `execution_attempts[]`
- `work_items[].state`
- `nodes[].state`
- `audit_events[]`
- 顶层 `updated_at`

dispatch：

- prepared dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197765`
- completed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197766`
- completed exit code：`0`
- completed warnings：`0`
- final summary：保存为 last-message 摘要，没有保存完整 transcript。

execution control：

- control id：`control:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`
- control_state：`ready_for_review`
- long_task_state：`completed`
- timeout_seconds：`300`
- retry_count：`0`
- max_retries：`0`
- failure_reason：空

execution attempt：

- attempt id：`attempt:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`
- state：`completed`
- attempt_no：`1`
- warnings：`0`

work item：

- state：`ready_for_review`
- current_node_id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:review`

注意：

- `workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev` 节点状态只读复核仍为 `running`。

## 审计事件

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122197765`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122197766`
- `audit:workflow-node-dispatch-completed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`
- `audit:workflow-node-dispatch-readback:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`

## 备份

- prepared / running 前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780122197765.json`
- completed 前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780122306967.json`

## 验证

- `rg -n -F 'Workflow dispatch smoke passed.' /Users/yoyi/codex-workflow-mario-test/README.md`：通过，命中第 14 行。
- `shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js`：通过，hash 如上。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- workflow state 只读复核：project / binding / dispatch / control / attempt / audit 均存在；work item 为 `ready_for_review`。

## 新增 Handoff

- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-result.md`
