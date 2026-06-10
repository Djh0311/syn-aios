# 四角色工作流机器 mario demo 真实闭环 Handoff

## 结果

真实闭环已完成，最终收口为 `accepted`。

目标项目：`/Users/yoyi/Documents/mario test`

最终产物：

- `index.html`
- `styles.css`
- `game.js`
- `README.md`

## 薄弱点

- 不是纯粹“总指导真实首轮计划”闭环：首轮总指导真实 resume 连续超时后，本轮采用本地合成计划 fallback。
- v4 最终先被旧判定误收口为 `needs_changes`，随后修复 detector 并写真实 state 修正为 `accepted`。
- Browser 截图没有成功，页面验证依据是 DOM、HTTP 200、文件和语法检查。

## 关键根因和修复

根因：

runner 写入 prompt 后没有关闭 child stdin，导致 `codex exec resume` 一直处于 `Reading prompt from stdin...`，直到 600 秒超时。

修复：

- 写入 prompt 后使用 `child.stdin.take()` 并让 stdin 在作用域结束时关闭。
- 失败时保留 stderr 短摘要 warning。
- 首轮总指导改为本地计划 fallback。
- 接受判定增加 `我的判断：通过`、`判断：通过`、`结论：通过`。
- 增加 CLI 备用入口 `__run_workflow_machine_real`，用于 UI 不可接管时跑同一后端逻辑。

## 真实状态

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是。
- 是否写真实 workflow state：是。
- 是否修改 `/Users/yoyi/Documents/mario test`：是。
- 是否读取 `auth.json`、`.env`、密钥、token、完整 transcript：否。
- 是否触碰其他业务项目：否。

最终 work item：

- `workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v4`
- state：`accepted`
- current node：`workflow:users-yoyi-documents-mario-test:default:node:review`

最终 run：

- `workflow-machine-run:workflow-users-yoyi-documents-mario-test-default:workflow-users-yoyi-documents-mario-test-default-create-mario-demo-v4:1780164875113`
- state：`accepted`
- rounds：`3`
- steps：`13`

审计：

- `audit:workflow-machine-v4-accepted-after-detector-fix:1780165617362`

## 验证

- `npm run typecheck`：通过。
- `cargo build`：通过。
- `cargo test --offline workflow_machine_runs_four_role_loop_to_acceptance`，指定既有 Cargo 缓存：通过。
- `node --check /Users/yoyi/Documents/mario test/game.js`：通过。
- Browser DOM：页面加载，显示 `Mario Demo`、canvas、状态、金币、时间、本地脚本和样式。
- HTTP：临时静态服务曾返回 `index.html`、`styles.css`、`game.js` 的 200。
- 临时服务端口已清理。

## 当前建议

可以回收为：四角色工作流机器已能驱动 Codex 会话完成一个真实小项目闭环。

不要回收为：复杂自动化已经成熟。下一步至少要补 UI 可触发验证、首轮总指导真实计划稳定性、直接 accepted 的新 run 验证、失败重试策略和可观测性。
