# 四角色工作流机器真实总指导自然闭环 Handoff

## 薄弱点

- v5 失败：运行了旧本地 CLI 产物，首轮仍是 `local-workflow-machine-director-plan`。
- v6 失败：最终总指导真实输出了 `WORKFLOW_MACHINE_FINAL_ACCEPTED`，但摘要截断导致状态机没识别。
- v7 成功：这才是本轮有效验收。
- 仍未做浏览器实玩；8765 端口复核时没有服务。

## 完成结果

v7 已完成真实四角色闭环：

- 总指导 -> 开发线 -> 验证线 -> 回收线 -> 总指导结论
- 首轮总指导是真实 Codex 会话，不是本地 fallback。
- final state：`accepted`
- rounds completed：`1`
- steps count：`5`
- final summary 包含：`WORKFLOW_MACHINE_FINAL_ACCEPTED`

## 关键对象

- project root：`/Users/yoyi/Documents/mario test`
- workflow id：`workflow:users-yoyi-documents-mario-test:default`
- work item id：`workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v7`
- run id：`workflow-machine-run:workflow-users-yoyi-documents-mario-test-default:workflow-users-yoyi-documents-mario-test-default-create-mario-demo-v7:1780195097534`

角色会话：

- 总指导：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- 开发线：`019e798a-ac37-7771-b982-e38084fcd22e`
- 验证线：`019e798a-f9d4-7922-8da8-1b6a8dbd6769`
- 回收线：`019e798b-2ee7-7f90-beb2-9031f6ad3e05`

## 代码修改

- 移除首轮总指导本地 fallback 分支。
- 删除本地 fallback helper。
- `compact_last_message_summary` 现在会在截断摘要后追加工作流控制行，避免 `WORKFLOW_MACHINE_FINAL_ACCEPTED` 被截掉。
- last-message 输出路径增加进程 id 和纳秒时间，避免离线并发测试串文件。
- 新增测试：长回复末尾控制标记必须保留并能触发 acceptance。

## 边界

- 是否执行真实 `codex exec resume`：是。
- 是否写 `/Users/yoyi/.codex`：是。
- 是否写真实 workflow state：是。
- 是否读取完整 transcript：否。
- 是否读取 `auth.json`、`.env`、密钥、token、授权文件：否。
- 是否触碰其他业务项目：否。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，4 个测试。
- 完整 `cargo test --offline`：通过，69 passed，1 ignored。
- `node --check '/Users/yoyi/Documents/mario test/game.js'`：通过。
- `build_index.py --check`：`validation_ok`。

## 回收建议

接受为：真实四角色工作流机器已经能闭环完成马里奥 demo，并自然进入 `accepted`。

不要接受为：复杂自动化完成。下一步应补 UI 直接触发、运行过程监控、失败重试、取消、权限队列、浏览器验收和更强的总指导计划结构。
