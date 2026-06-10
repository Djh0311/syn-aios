# Evidence：工作流派发超时诊断与重试准备 v1

## 理解

本轮任务不是继续真实重试派发，而是先诊断上一轮 `codex exec resume` 超时，并在未获额外批准前只写 evidence / handoff。

## 已知

- 上一轮真实状态收口复测没有通过。依据：`handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md` 判为需要修改。
- README 目标行 `Workflow dispatch state closure retest passed.` 仍不存在。依据：本轮 `rg -n -F` 搜索无命中。
- 旧 work item `workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest` 是 `timed_out`。依据：真实 workflow state 摘要。
- codex-dev 节点是 `timed_out`，不是 `running`。依据：真实 workflow state 摘要。
- 新 work item `workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2` 当前不存在。依据：真实 workflow state 摘要。
- 目标 thread `019e7738-5e29-74e0-a22f-5c2481b64c38` 在 `codex-index.json` 中存在，`project_root=/Users/yoyi/codex-workflow-mario-test`，rollout 存在。依据：索引摘要和 `build_index.py --check`。

## 未知

- 不能确认超时卡在 prompt 投递、模型执行、插件启动、MCP 进程、权限等待、输出回收，还是 runner 等待策略。依据：本轮禁止读取完整 transcript，且 last-message 为空。
- 不能确认 GitHub rate limit / 插件同步 warning 是根因。依据：warning 存在，但没有完整执行链证明它导致超时。
- 不能确认同一 thread 下次一定成功。依据：只读检查只证明 thread / project root / rollout 正常，不证明 resume 稳定完成。

## 假设

- 下轮如果重试，应该创建新 retry work item，而不是把旧 `timed_out` work item 改回 `ready_to_dispatch`。依据：任务包禁止直接复用旧 `timed_out` work item。
- 下轮继续使用同一个测试 thread 是可接受的默认策略。依据：索引中 thread 存在、project root 匹配、rollout 存在，且没有证据表明 thread 已损坏。

## 边界

- 是否执行新的 `codex exec`：否。
- 是否执行新的 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：是。依据：用户随后明确回复“批准”，允许进入第二段准备 retry work item。
- 是否修改 README：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否读取 rollout JSONL 正文：否。
- 是否运行 harness：否。
- 是否创建新 work item：是。
- 是否创建新 binding：是。

## 上次超时事实链

1. 上一轮任务获得用户批准后执行真实 `codex exec resume`。依据：`evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`。
2. 较宽命令 `-C /Users/yoyi --sandbox workspace-write --add-dir ...` 被拒绝。依据：上一轮 evidence。
3. 后续改用 `-C /Users/yoyi/codex-workflow-mario-test --sandbox workspace-write --skip-git-repo-check` 的更窄命令。依据：上一轮 evidence。
4. 真实 resume 启动后超过约 300 秒未完成。依据：上一轮 evidence 和 workflow execution control `timeout_seconds=300`。
5. 执行期间出现插件目录鉴权、MCP 进程组终止、远端插件同步超时、GitHub rate limit warning。依据：上一轮 evidence。
6. last-message 为空或未生成有效最终回复。依据：上一轮 evidence。
7. README 目标行没有写入。依据：上一轮 evidence 和本轮 README 搜索无命中。
8. workflow state 写回 `dispatch=failed`、`execution control=timed_out`、`attempt=timed_out`、work item `timed_out`、codex-dev node `timed_out`。依据：真实 workflow state 摘要。

## workflow state 摘要

- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- 旧 work item：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest`
- 旧 work item state：`timed_out`
- 旧 work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- codex-dev node state：`timed_out`
- dispatch：
  - `prepared`：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest:1780128642386`
  - `failed`：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest:1780128642387`
- execution control：`timed_out`，`failure_reason=timeout`，`timeout_seconds=300`
- execution attempt：`timed_out`，`failure_reason=timeout`
- 新 v2 work item：不存在
- active binding：旧 retest binding lifecycle 为 `active`，native thread 为 `019e7738-5e29-74e0-a22f-5c2481b64c38`，rollout_exists 为 `true`

## 文件复核

- README 目标行：不存在。
- README hash：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html` hash：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 初步根因分类

已知原因：

- 直接原因是 300 秒 timeout。依据：workflow execution control / attempt 均为 `timed_out`，failure_reason 为 `timeout`。
- 业务修改没有完成。依据：README 目标行无命中，last-message 为空。

可能相关但不能定为根因：

- 插件启动或同步耗时。依据：上一轮 evidence 记录了插件目录鉴权、远端插件同步超时 warning。
- GitHub rate limit。依据：上一轮 evidence 记录了 GitHub 429 / rate limit warning。
- MCP 进程组终止 warning。依据：上一轮 evidence。
- runner 等待策略过短。依据：timeout_seconds 为 300，真实 resume 未完成；但没有证据证明 600 秒一定成功。

不能断言：

- 不能断言是 Codex 会话损坏。依据不足，且索引显示 thread / project root / rollout 正常。
- 不能断言是 README 修改权限失败。依据不足，未见 README 写入失败的结构化错误。
- 不能断言是业务 prompt 太复杂。依据不足，但下轮可以简化 prompt 降低变量。
- 不能断言是 sandbox 参数唯一问题。依据：更窄 cwd 命令已启动，只是超时。

## 下一轮重试建议

- 建议使用新 work item：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`。依据：旧 work item 已 `timed_out`，任务包禁止直接复用。
- 建议继续使用同一个 thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`。依据：索引存在、project root 匹配、rollout 存在；没有证据要求新建 thread。
- 暂不建议新建 thread。依据：当前 thread 元数据正常；新建 thread 会额外写 `/Users/yoyi/.codex` 并扩大变量。
- 建议下轮 cwd 固定为 `/Users/yoyi/codex-workflow-mario-test`。依据：上一轮较宽 cwd 被拒绝，较窄 cwd 更符合目标项目范围。
- 建议下轮 prompt 只保留“追加一行 README 并回传最小结果”。依据：当前缺少成功路径证明，先降到最小可验证动作。
- 建议把 timeout 从 300 秒提高到 600 秒。依据：上一轮真实 resume 超过 300 秒未完成，且存在插件同步 / rate limit warning；风险是失败反馈变慢，且仍不能保证成功。

## 准备状态判断

本轮只读诊断已经满足。

用户回复“批准”后，已进入第二段并写真实 workflow state：

- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:state-closure-retest-v2`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- assigned role：`codex-dev`
- current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:director`
- state：`ready_to_dispatch`
- active binding thread：`019e7738-5e29-74e0-a22f-5c2481b64c38`

新增 binding：

- binding id：`binding:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-node-codex-dev:workflow-users-yoyi-codex-workflow-mario-test-default-state-closure-retest-v2`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- lifecycle：`active`
- rollout_exists：true
- warnings：`prepared_after_previous_timeout`

节点摘要调整：

- codex-dev node 从旧复测留下的 `timed_out` 调整为 `ready_to_dispatch`。
- 依据：新 retry work item 已准备，旧 work item 保持 `timed_out`，但工作台派发节点摘要需要反映新可派发态。
- warning：`previous_state_closure_retest_timed_out; retry_v2_ready_to_dispatch`

备份：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780131018244.json`

审计事件：

- `audit:workflow-state-closure-retest-v2-work-item-ready:1780131018244`
- `audit:workflow-state-closure-retest-v2-session-bound:1780131018244`
- `audit:workflow-state-closure-retest-v2-node-ready:1780131018244`

## 验证命令和结果

- `rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md`：无命中。
- `shasum -a 256 /Users/yoyi/codex-workflow-mario-test/README.md /Users/yoyi/codex-workflow-mario-test/index.html /Users/yoyi/codex-workflow-mario-test/styles.css /Users/yoyi/codex-workflow-mario-test/game.js`：通过，hash 如上。
- `/Users/yoyi/miniconda3/bin/python3 /Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py --check /Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`：`validation_ok`。
- workflow state 摘要复核：旧 work item / execution control / attempt 均保持 `timed_out`；新 v2 work item 为 `ready_to_dispatch`；新 v2 binding 为 `active`；codex-dev node 为 `ready_to_dispatch`。
