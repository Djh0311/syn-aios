# Handoff：真实 workflow state README smoke 节点状态收口修复 v1

## 结论

真实 workflow state 存量 `codex-dev=running` 已修复。

## 薄弱点

- 这是存量状态修复，不是新派发。
- 只修复 README smoke 对应的一个节点。
- 本轮写了真实 workflow state，需要总指导复核备份、审计和目标范围。

## 已完成

- 备份真实 workflow state。
- 将 `workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev` 从 `running` 改为 `ready_for_review`。
- 追加审计事件 `workflow_node_state_closure_real_state_fix`。
- 只读复核 work item、node、review node、audit event。

## 写入情况

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否修改 README：否。
- 是否读取敏感文件或完整 transcript：否。

## 关键对象

- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- completed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197766`

## 备份和审计

- 备份路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780125824485.json`
- 审计事件 id：`audit:workflow-node-state-closure-real-state-fix:workflow-users-yoyi-codex-workflow-mario-test-default-node-codex-dev:1780125824485`

## 复核结果

- work item state：`ready_for_review`
- work item current node：review
- codex-dev node state：`ready_for_review`
- review node state：`ready_for_review`
- README 目标行仍在第 14 行。
- `index.html`、`styles.css`、`game.js` hash 未变。
- 索引校验：`validation_ok`

## 下一步建议

总指导回收本次真实 state 存量修复。

建议回收口径：

- 接受为：README smoke 存量节点状态旧账已修复。
- 不接受为：新派发已再次真实验证。
