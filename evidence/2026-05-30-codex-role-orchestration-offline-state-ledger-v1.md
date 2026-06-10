# Evidence：Codex 角色编排离线状态账本 v1

## 结论

已把 Codex 角色编排从“离线入口”推进到“工作台自己的状态账本闭环”。

本轮接受为：

- 工作台能展示总指导、开发线、验证线、回收线四个角色。
- 工作台能解析固定字段派发块。
- 用户确认后可把离线角色派发写入 `workflow_node_dispatches[]`，状态为 `prepared`。
- 用户确认后可记录角色回传，派发状态变为 `completed`，并写入 `handoff` artifact。
- 用户确认后可记录总指导回收，写入 `reviews[]`，并推进 work item 到 `accepted` 等结论状态。
- 离线派发记录会带回 `offline_role_dispatch` payload，角色回传优先使用已落账派发块，不回退到默认示例。

本轮不接受为：

- 真实多 Codex 会话自动编排已经完成。
- 总指导能自动制定计划并连续调度各角色。
- 角色会话已经通过 `codex exec resume` 真实执行。
- 真实业务自动工作流已经完成。

## 薄弱点

- 这是离线账本闭环，不是多会话真实执行闭环。
- UI 仍使用固定示例派发块，尚不是总指导真实输出的动态解析器。
- 角色回传仍是桩结果，不来自真实 Codex 会话。
- 当前只覆盖一个 ready_to_dispatch work item 的线性闭环；还没有多角色并发、重试、取消、权限队列的真实编排。

## 改动内容

新增 / 强化：

- `prepare_offline_role_dispatch`
- `record_offline_role_result_handoff`
- `record_offline_director_review`
- `offline_role_dispatch` payload readback

修改文件：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`
- `prototypes/productized-desktop-shell/dist/assets/index-DmWTx_Ms.js`

## 状态写入边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否发送 Codex 消息：否。
- 是否修改业务项目文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。
- 是否写真实 workflow state：本轮没有通过 UI 对真实状态文件执行确认动作；代码路径支持用户确认后写工作台自己的 `workflow-state.v0.json`，测试只写临时状态文件。

## 验证结果

- `cargo fmt`：通过。
- 默认 `cargo test --offline`：未作为最终依据；默认 Cargo 缓存曾有 `serde_json` 离线版本不匹配的历史问题。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，66 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成，输出为历史记录和任务文本命中，没有触发命令替换。

## Subagent 复核

使用 subagent 做了只读前端 / 测试风险复核。

复核结论：

- 前端类型链条基本接上。
- 明确指出离线测试仍按旧 UI 文案和无 workflow 锚点编写。
- 建议测试传入 ready_to_dispatch workflow，并把“只能预览”拆成独立场景。

本轮已按该建议修正测试。

## 下一步

下一步建议不是直接放开真实多会话自动执行，而是做“总指导计划块接入 v1”：

- 让用户给总指导发需求。
- 总指导回复固定字段计划 / 派发块。
- 工作台把计划块拆成多个离线派发候选。
- 每个候选仍需用户确认后写入账本或真实派发。
