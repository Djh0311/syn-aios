# Evidence：真实 workflow state README smoke 节点状态收口修复 v1

## 结论

已修复真实 workflow state 中 README smoke 的存量节点状态。

本轮只修真实 workflow state 旧账，不执行新的 Codex 派发。

## 薄弱点

- 本轮是存量账本修复，不证明新派发链路再次真实执行。
- 只修复一个明确目标节点：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`。
- 本轮没有全量扫描并修复其它潜在脏状态。
- 写了真实 workflow state，因此必须依赖备份和审计事件回滚 / 追踪。

## 用户批准

- 是否获得用户明确批准：是。
- 批准内容：修复真实 workflow state 中存量 `codex-dev=running`。

## 修复对象

- project id：`project:users-yoyi-codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- completed dispatch id：`dispatch:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke:1780122197766`

## 修复前依据

- work item state：`ready_for_review`
- work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:review`
- completed dispatch state：`completed`
- completed dispatch exit code：`0`
- codex-dev node state：`running`
- workflow state hash：`4cf5d597cd07678e5945df31192f17daa46c15af5a06a2206c1e090b8ade4413`

## 写入内容

写入真实 workflow state：是。

字段类型：

- `nodes[].state`
- `nodes[].updated_at`
- `audit_events[]`
- 顶层 `updated_at`

具体变化：

- codex-dev node：`running -> ready_for_review`
- work item：未改，保持 `ready_for_review`
- review node：未改，保持 `ready_for_review`
- dispatch：未改，保持 `completed`

## 备份和审计

备份路径：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780125824485.json`

审计事件：

- `audit:workflow-node-state-closure-real-state-fix:workflow-users-yoyi-codex-workflow-mario-test-default-node-codex-dev:1780125824485`

审计事件类型：

- `workflow_node_state_closure_real_state_fix`

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否修改 README：否。
- 是否修改 `index.html` / `styles.css` / `game.js`：否。
- 是否读取敏感文件：否。
- 是否读取完整 transcript：否。
- 是否运行 harness：否。

## 修复后复核

- work item state：`ready_for_review`
- work item current node：`workflow:users-yoyi-codex-workflow-mario-test:default:node:review`
- codex-dev node state：`ready_for_review`
- review node state：`ready_for_review`
- audit event 存在。
- workflow state hash：`b933c2cf557d069e62c57b430fa174073fa03c2a1c1b2a44bc6d087f249b7766`

业务文件 hash：

- `README.md`：`5c3331c1eca9376d1b037bee06136cabbb13cd85e1913bef651ce0a7f8032d26`
- `index.html`：`35ecf58229427d00a3087b729995adf263aeefefbda79bb3bae7b288c3fbcaa8`
- `styles.css`：`7c866d69c5d6c52d69ede7e14803b5b6eb2fdacca3b7ef76917ac061b65bde1e`
- `game.js`：`a794af1e4116edf3cdf456c14f20da36394de8bb989c0799a3b017bf48c7f2ee`

## 验证

- workflow state 只读复核：通过。
- `rg -n -F 'Workflow dispatch smoke passed.' /Users/yoyi/codex-workflow-mario-test/README.md`：通过，命中第 14 行。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。

## Handoff

- `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-result.md`
