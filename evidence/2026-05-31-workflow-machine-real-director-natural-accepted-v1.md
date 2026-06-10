# 四角色工作流机器真实总指导自然闭环 Evidence

## 薄弱点

- v5 没有证明目标。原因是我运行了旧的 `./target/debug/codex-governance-workbench`，首轮仍走了旧本地 fallback。依据：v5 run 第一条 step 的 `native_thread_id` 是 `local-workflow-machine-director-plan`。
- v6 也没有自然 accepted。原因不是执行链路失败，而是最终总指导真实回复末尾有 `WORKFLOW_MACHINE_FINAL_ACCEPTED`，但 `last_message_summary` 只保留前 240 个字符，状态机没看到控制标记。依据：v6 最后回复文件包含该标记，workflow run 最终仍为 `needs_changes`。
- v7 是本轮有效闭环证据。它仍是马里奥 demo 小项目，不代表复杂业务自动化已经成熟。
- 浏览器服务 `http://127.0.0.1:8765/index.html` 在复核时未运行，`curl` 连接失败；本轮只验证文件、语法和 workflow state，没有做浏览器实玩。

## 做了什么

- 移除首轮总指导本地 fallback 路径，让首轮总指导也通过真实绑定 Codex 会话执行。
- 修复 workflow machine 最终接受判断的读回问题：摘要截断时仍保留 `WORKFLOW_MACHINE_FINAL_ACCEPTED`、`WORKFLOW_MACHINE_CONTINUE`、`WORKFLOW_MACHINE_STEP_STATUS` 控制行。
- 修复离线测试并发输出文件碰撞：last-message 路径加入进程 id 和纳秒时间后缀。
- 创建并运行真实工作流 work item：
  - v5：失败，旧二进制导致仍走本地 fallback。
  - v6：失败，真实总指导已输出 accepted 标记，但摘要截断导致状态机未识别。
  - v7：成功，真实总指导 -> 开发线 -> 验证线 -> 回收线 -> 总指导结论，一轮自然收口为 `accepted`。

## 真实 v7 结果

- project root：`/Users/yoyi/Documents/mario test`
- workflow id：`workflow:users-yoyi-documents-mario-test:default`
- work item id：`workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v7`
- run id：`workflow-machine-run:workflow-users-yoyi-documents-mario-test-default:workflow-users-yoyi-documents-mario-test-default-create-mario-demo-v7:1780195097534`
- final state：`accepted`
- rounds completed：`1`
- steps count：`5`
- first step thread：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- final step thread：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- final summary 保留控制标记：`WORKFLOW_MACHINE_FINAL_ACCEPTED`

## 四角色会话

- 总指导：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- 开发线：`019e798a-ac37-7771-b982-e38084fcd22e`
- 验证线：`019e798a-f9d4-7922-8da8-1b6a8dbd6769`
- 回收线：`019e798b-2ee7-7f90-beb2-9031f6ad3e05`

## 写入情况

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是，通过真实 `codex exec resume`。
- 是否写真实 workflow state：是。
- 是否修改 `/Users/yoyi/Documents/mario test`：本轮 v5/v6/v7 没有发现文件 hash 变化；demo 文件沿用已生成内容。
- 是否读取完整 transcript：否。
- 是否读取 `auth.json`、`.env`、密钥、token、授权文件：否。
- 是否触碰其他业务项目：否。

## 备份

- v5 准备备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780193606043.json`
- v5 run 备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780193674878.json`
- v6 准备备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780194300538.json`
- v6 run 备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780194345547.json`
- v7 准备备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780195075440.json`
- v7 run 备份：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780195097534.json`

## 文件复核

目标项目文件：

- `/Users/yoyi/Documents/mario test/index.html`
- `/Users/yoyi/Documents/mario test/styles.css`
- `/Users/yoyi/Documents/mario test/game.js`
- `/Users/yoyi/Documents/mario test/README.md`

SHA256：

- `index.html`: `f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`: `6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`: `814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`: `02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## 验证命令

- `cargo fmt`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，4 个离线交互测试。
- `cargo test --offline workflow_machine`：通过，2 passed。
- `cargo test --offline workflow_node_dispatch_execute_uses_stub_and_advances_to_review`：通过。
- `cargo test --offline workflow_node_dispatch_execute_user_reviewed_instruction_uses_codex_options`：通过。
- 完整 `cargo test --offline`：通过，69 passed，1 ignored。
- `node --check '/Users/yoyi/Documents/mario test/game.js'`：通过。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json`：`validation_ok`。
- `curl -sS -I http://127.0.0.1:8765/index.html`：失败，端口没有服务。

## 结论

可以回收为：四角色工作流机器已经能用真实 Codex 会话完成马里奥 demo 的一轮自然闭环，首轮总指导也是真实会话，最终状态由真实总指导回复中的 `WORKFLOW_MACHINE_FINAL_ACCEPTED` 自动识别为 `accepted`。

不能回收为：复杂业务自动化已经成熟。还缺产品化 UI、长期任务观测、权限队列、失败重试、取消、并发控制、浏览器/视觉验收和 transcript reader 事件兼容。
