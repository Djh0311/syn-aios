# Evidence：准备 README smoke 测试 workflow state v2

## 结论

已按用户明确批准写入真实 workflow state，只准备 README smoke 前置状态，没有执行 README smoke。

## 薄弱点

- 本轮写了真实 workflow state，需要总指导复核结构是否满足后端派发路径。
- 本轮没有执行 `codex exec resume`，所以不能证明 README smoke 派发会成功。
- 本轮只绑定已有测试 thread；如果该 thread 后续状态变化，下一轮派发前仍需复核索引和 rollout。

## 用户批准

- 是否获得用户明确批准：是。用户回复“允许”后执行写入。

## 写入对象

- project id：`project:users-yoyi-codex-workflow-mario-test`
- workflow id：`workflow:users-yoyi-codex-workflow-mario-test:default`
- node id：`workflow:users-yoyi-codex-workflow-mario-test:default:node:codex-dev`
- work item id：`workflow:users-yoyi-codex-workflow-mario-test:default:readme-smoke`
- binding id：`binding:workflow-users-yoyi-codex-workflow-mario-test-default:workflow-users-yoyi-codex-workflow-mario-test-default-node-codex-dev:workflow-users-yoyi-codex-workflow-mario-test-default-readme-smoke`
- bound thread id：`019e7738-5e29-74e0-a22f-5c2481b64c38`
- rollout exists：true

## 备份和审计

- backup path：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780119961972.json`
- audit event ids：
  - `audit:workflow-readme-smoke-project-registered:1780119961972`
  - `audit:workflow-readme-smoke-ready-to-dispatch:1780119961972`
  - `audit:workflow-readme-smoke-session-bound:1780119961972`

## 状态边界

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec` 或 `codex exec resume`：否。
- 是否修改 README：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否修改 `/Users/yoyi/gameai/agent world` 旧记录：否；只读复核显示旧 workflow / work item / binding 仍存在且状态未替换。

## 只读复核

- `/Users/yoyi/codex-workflow-mario-test` project：存在。
- workflow：存在，id 为 `workflow:users-yoyi-codex-workflow-mario-test:default`。
- codex-dev node：存在。
- README smoke work item：存在，state 为 `ready_to_dispatch`。
- active binding：存在，thread id 为 `019e7738-5e29-74e0-a22f-5c2481b64c38`。
- thread 在索引中：存在。
- rollout：存在。
- README 目标行 `Workflow dispatch smoke passed.`：仍不存在。
- README hash 仍为 `6f9cc4be0f3ad0cdf7926af9bcbbd747a383ce6d3e2085a9322786b8176811db`。
- `index.html` / `styles.css` / `game.js` hash 未变化。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。

## Handoff

- `handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v2-result.md`
