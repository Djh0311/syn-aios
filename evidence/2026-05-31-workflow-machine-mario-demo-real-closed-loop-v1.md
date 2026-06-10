# 四角色工作流机器 mario demo 真实闭环 Evidence

## 薄弱点

- 首轮总指导真实 resume 连续两次 600 秒超时，所以本轮最终采用了本地合成首轮总指导计划的 fallback。依据：`create-mario-demo-v1`、`create-mario-demo-v2` 都停在 director，warnings 包含 `timeout`。
- v3 证明开发线也会因 runner 未关闭 stdin 而超时；根因不是会话本身不可用。依据：独立健康探针同一开发线 thread 能快速返回 `MARIO_WORKFLOW_HEALTH_OK`。
- v4 实际完成后，旧 acceptance detector 没识别“我的判断：通过”，误收口为 `needs_changes`；本轮修复判定后写真实 state 修正为 `accepted`。
- 浏览器 `file://` 验证被 Browser 安全策略拒绝；改用 `127.0.0.1` 静态服务验证。截图尝试超时，最终采用 DOM、HTTP、语法和文件检查作为依据。

## 目标

用 `/Users/yoyi/Documents/mario test` 和四个 Codex 会话跑真实闭环：

`总指导 -> 开发线 -> 验证线 -> 回收线 -> 总指导结论 -> 下一轮 / 最终目标`

## 真实会话

- 总指导：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- 开发线：`019e798a-ac37-7771-b982-e38084fcd22e`
- 验证线：`019e798a-f9d4-7922-8da8-1b6a8dbd6769`
- 回收线：`019e798b-2ee7-7f90-beb2-9031f6ad3e05`

## 执行过程

- v1：真实总指导首步超时，run state `failed`。
- v2：缩短总指导 prompt 并加入 stderr 诊断后，真实总指导仍超时；stderr 摘要为 `Reading_prompt_from_stdin...`。
- v3：首轮总指导改本地计划 fallback，真实开发线仍超时；同样是 `Reading_prompt_from_stdin...`。
- 健康探针：用 shell 管道直接 `codex exec resume` 开发线，快速返回 `MARIO_WORKFLOW_HEALTH_OK`。
- 根因：runner 写入 prompt 后没有关闭 child stdin，导致 Codex 一直等待 stdin EOF。
- 修复：runner 写完 prompt 后使用 `child.stdin.take()` 并在写完后 drop stdin。
- v4：真实完成开发线、验证线、回收线、总指导多轮闭环；因 acceptance detector 过窄误判为 `needs_changes`。
- 收口修复：识别 `我的判断：通过` / `判断：通过` / `结论：通过`，并把 v4 真实 workflow state 修正为 `accepted`。

## 写入情况

- 是否执行真实 `codex exec resume`：是，多次。
- 是否写 `/Users/yoyi/.codex`：是，通过真实 resume。
- 是否写真实 workflow state：是。
- 是否修改 `/Users/yoyi/Documents/mario test`：是，由开发线创建/修改四个 demo 文件。
- 是否读取授权、密钥、`.env`、token、完整 transcript：否。
- 是否触碰其他业务项目：否。

## 最终 workflow state

- accepted work item：`workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v4`
- run id：`workflow-machine-run:workflow-users-yoyi-documents-mario-test-default:workflow-users-yoyi-documents-mario-test-default-create-mario-demo-v4:1780164875113`
- run state：`accepted`
- rounds completed：`3`
- step count：`13`
- acceptance repair audit：`audit:workflow-machine-v4-accepted-after-detector-fix:1780165617362`

## 备份

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780163101569.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780164001639.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780164837994.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780165617362.json`

## 项目产物

- `/Users/yoyi/Documents/mario test/index.html`
- `/Users/yoyi/Documents/mario test/styles.css`
- `/Users/yoyi/Documents/mario test/game.js`
- `/Users/yoyi/Documents/mario test/README.md`

SHA256：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## 验证

- `npm run typecheck`：通过。
- `cargo build`：通过。
- 指定 Cargo 缓存的 `cargo test --offline workflow_machine_runs_four_role_loop_to_acceptance`：通过。
- `node --check /Users/yoyi/Documents/mario test/game.js`：通过。
- Browser DOM 验证：`http://127.0.0.1:8765/index.html` 已加载，title 为 `Mario Demo`，页面显示 canvas、金币、时间、状态、本地 `game.js` 和 `styles.css`。
- HTTP 验证：临时静态服务日志记录 `GET /index.html`、`GET /styles.css`、`GET /game.js` 均为 `200`；后续 HEAD `index.html` 和 `game.js` 也返回 `200`。
- 临时静态服务器已停止，8765 / 8766 无残留监听。

## 代码修复

- 新增 CLI 备用入口：`__run_workflow_machine_real`。
- 修复 runner stdin 未关闭导致 `codex exec resume` 卡住。
- 增加失败 stderr 摘要 warning。
- 首轮总指导本地计划 fallback。
- 扩大 workflow machine final acceptance 判定。

## 仍需后续处理

- 产品上仍应恢复“首轮总指导真实计划”的能力；本轮 fallback 是为绕过已验证的 runner/会话问题后继续主链路验收。
- UI 自动化没有走通，原因是 Tauri 进程无法被 Computer Use 按进程名接管；本轮改走 CLI 后端同函数入口。
- acceptance detector 修复后，最好补一个真实新 run，证明无需事后修正即可直接 accepted。
