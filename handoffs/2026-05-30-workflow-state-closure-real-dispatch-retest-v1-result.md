# Handoff：工作流状态收口真实派发复测 v1

## 结论

真实复测已执行，但未通过。

结果是超时失败：README 没有追加目标行，dispatch 写为 `failed`，execution control / attempt 写为 `timed_out`，work item 和 codex-dev 节点收口为 `timed_out`。

## 薄弱点

- 本轮没有得到 completed 成功路径。
- 插件启动阶段出现远端同步、GitHub rate limit、MCP 进程组 warning。
- 初始较宽执行命令被拒绝，后改用测试项目目录作为更窄 cwd。
- 目标 README 行未写入。
- last-message 为空，不能证明业务会话完成了指令。

## 边界

- 是否获得用户明确批准：是。
- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是。
- 是否写真实 workflow state：是。
- 是否修改 README：否。
- 是否修改允许范围外文件：否。
- 是否读取敏感文件或完整 transcript：未见依据；没有读取完整 transcript。
- 是否运行 harness：否。

## 写入结果

- work item：`timed_out`
- work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- codex-dev node：`timed_out`
- dispatch：`failed`
- exit code：`-1`
- control：`timed_out`
- attempt：`timed_out`

completed 后 codex-dev 是否仍为 `running`：

- 不适用，因为本轮没有 completed。
- 超时失败后 codex-dev 不为 `running`，已收口到 `timed_out`。

## 备份和审计

- running 前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780128642386.json`
- 超时写回前备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780129407652.json`
- audit event id：
  - `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780128642386`
  - `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780128642387`
  - `audit:workflow-node-dispatch-failed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780129407652`

## 文件复核

- target line：不存在。
- README hash：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html` hash：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 验证

- README 目标行搜索：无命中。
- 业务文件 hash：未变。
- 索引校验：`validation_ok`。
- workflow state 摘要：无 `running` 残留，retest work item / codex-dev node 为 `timed_out`。

## 下一步建议

总指导应回收为未通过或需要修改。若要再试，需要先把 retest work item 从 `timed_out` 合法推进到可重试状态，或新建下一轮 work item；再次真实派发仍需明确批准。
