# Review：工作流状态收口真实派发复测 v1

## 结论

需要修改。

接受为：

- 真实 `codex exec resume` 已执行。
- 超时失败后，work item 已收口为 `timed_out`。
- 超时失败后，codex-dev 节点已收口为 `timed_out`，没有残留 `running`。
- 真实 workflow state 已记录 dispatch、execution control、execution attempt 和 audit。

不接受为：

- README 目标行已追加成功。
- completed 成功路径已被真实复测证明。
- work item 已进入 `ready_for_review`。
- codex-dev completed 后已收口为 `ready_for_review`。
- 复杂业务自动编排完成。

## 薄弱点

- 本轮真实复测没有通过成功验收。依据：README 中没有 `Workflow dispatch state closure retest passed.`。
- 本轮没有覆盖 completed 成功路径。依据：dispatch 最终为 `failed`，execution attempt 为 `timed_out`。
- 只能证明超时失败路径不会把 codex-dev 卡在 `running`。依据：真实 workflow state 中 retest work item 和 codex-dev node 均为 `timed_out`。
- 下次不能直接复用当前 retest work item。依据：当前 work item 状态已是 `timed_out`，不是 `ready_to_dispatch`。

## 回收依据

已复核：

- `evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-result.md`
- 真实 workflow state 摘要
- README 目标行搜索结果
- README / `index.html` / `styles.css` / `game.js` hash

## 关键复核结果

### README

未通过。

依据：

- `rg -n -F 'Workflow dispatch state closure retest passed.' /Users/yoyi/codex-workflow-mario-test/README.md` 无命中。
- README hash 仍为 `5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`。

### workflow state

接受为超时收口正确，不接受为成功完成。

依据：

- retest work item state：`timed_out`
- retest work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- codex-dev node state：`timed_out`
- dispatch state：`failed`
- execution attempt state：`timed_out`
- failure reason：`timeout`

### 允许范围外文件

接受。

依据：

- `index.html` hash 未变：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css` hash 未变：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js` hash 未变：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 边界

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是。
- 是否写真实 workflow state：是。
- 是否修改 README：否。
- 是否修改允许范围外文件：否。
- 是否读取敏感文件或完整 transcript：未见依据；没有读取完整 transcript。
- 是否运行 harness：否。

## 回收决定

本轮不通过，结论为需要修改。

下一步建议：

- 先不要继续派发。
- 写一个小任务包，目标是诊断这次 `codex exec resume` 超时原因，并准备合法的下一轮重试 work item。
- 下一轮如果仍要真实派发，必须再次明确批准，因为会再次执行 `codex exec resume`、写 `/Users/yoyi/.codex`、写真实 workflow state，并修改测试 README。
