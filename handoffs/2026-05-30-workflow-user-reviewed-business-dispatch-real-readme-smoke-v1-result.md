# Handoff：用户审核业务派发真实 README 极小验证 v1

## 结论

真实 README smoke 已完成，等待总指导回收。

## 薄弱点

- 这只证明一次极小 README 写入闭环，不证明复杂业务自动编排。
- 未做全仓库扫描；允许范围外文件未变的依据是指定文件 hash 复核和 Codex 回传。
- workflow state 里 work item 已是 `ready_for_review`，但 codex-dev 节点仍显示 `running`，后续需要修正节点状态收口或在回收里明确接受该缺口。
- 没有真实验证失败重试、权限确认、取消和长任务超时。

## 已完成

- 获得用户明确批准后执行真实 `codex exec resume`。
- 通过绑定 thread `019e7738-5e29-74e0-a22f-5c2481b64c38` 派发用户审核业务指令。
- 在 `/Users/yoyi/codex-workflow-mario-test/README.md` 追加目标行：`Workflow dispatch smoke passed.`
- 写入真实 workflow state。
- 写入 dispatch、execution control、execution attempt 和 audit event。
- 只保存 last-message 摘要，没有保存完整 transcript。

## 边界

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是，通过 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 README：是。
- 是否修改允许范围外文件：未发现。`index.html`、`styles.css`、`game.js` hash 未变。
- 是否读取敏感文件：否。
- 是否读取完整 transcript：否。
- 是否运行 harness：否。

## 关键对象

- project id：`project:users-yoyi-codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`

## dispatch

- prepared dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197765`
- completed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197766`
- exit code：`0`
- warnings：`0`
- last message path：`/tmp/codex-workflow-node-dispatch-v1/dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo-1780122197766-last-message.txt`

## workflow state

- work item state：`ready_for_review`
- execution control：`ready_for_review` / `completed`
- execution attempt：`completed`
- retry_count：`0`
- max_retries：`0`
- timeout_seconds：`300`
- failure_reason：空

注意：codex-dev 节点状态仍是 `running`。

## 审计事件

- `audit:workflow-node-dispatch-prepared:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122197765`
- `audit:workflow-node-dispatch-started:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122197766`
- `audit:workflow-node-dispatch-completed:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`
- `audit:workflow-node-dispatch-readback:dispatch-workflow-users-yoyi-codex-workflow-mario-test-default-workflow-users-yoyi-codex-workflo:1780122306967`

## 验证结果

- README 目标行存在：`/Users/yoyi/codex-workflow-mario-test/README.md:14`
- README hash：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html` hash：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`
- 索引校验：`validation_ok`

## 下一步建议

总指导回收本次真实 README smoke。

建议回收口径：

- 接受为：用户审核业务派发极小真实写入闭环已跑通一次。
- 不接受为：复杂业务自动编排完成。
- 必须点名：codex-dev 节点状态仍为 `running` 的状态收口缺口。
