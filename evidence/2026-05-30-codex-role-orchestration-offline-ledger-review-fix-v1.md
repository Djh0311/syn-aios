# Evidence：Codex 角色编排离线账本复核修复 v1

## 结论

已修复复核发现的两个 P1 问题：

- 回传后 work item 进入 `ready_for_review`，UI 仍能保留账本锚点并启用“写入总指导回收”。
- 同一 work item 已有 `prepared` 离线派发时，后端拒绝重复写入。

本轮仍不接受为：

- 真实多 Codex 会话自动编排完成。
- 总指导真实自动制定计划并连续调度。
- 角色会话真实执行和回传。

## 薄弱点

- 派发预览仍不是实时编辑预览，只是默认示例；已在 UI 文案中说明“提交以文本框为准，预览来自默认示例”。
- 当前仍是单工作项自动选择，不是多工作项手动选择器。
- 没有通过真实 UI 写真实 workflow state；验证仍只写临时测试状态。

## 改动内容

- 前端 `selectedOfflineWorkItem` 改为生命周期感知：
  - 优先选择 `completed + offline_role_dispatch + 未 review` 的 work item，用于总指导回收。
  - 其次选择 `prepared + offline_role_dispatch` 的 work item，用于角色回传。
  - 最后选择普通 `ready_to_dispatch` work item，用于新离线派发。
- “写入离线派发”在已有 prepared / completed 离线派发时禁用。
- 后端 `prepare_offline_role_dispatch_at` 增加重复 prepared 检查。
- 前端测试新增 completed 离线派发未回收场景。
- Rust 测试新增重复 prepared 拒绝场景。

## 改动文件

- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`
- `prototypes/productized-desktop-shell/dist/assets/index-DfqNlrl_.js`

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否发送 Codex 消息：否。
- 是否通过 UI 写真实 workflow state：否。
- 是否修改业务项目文件：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 验证结果

- `cargo fmt`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，67 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 4`。
- `npm run build`：通过。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成，输出为历史记录和任务文本命中，没有触发命令替换。

## Subagent 复核

使用 subagent 做了只读复核。

复核结论：

- 前端第三步确实会因只选择 `ready_to_dispatch` 而卡住。
- 后端确实会重复写 prepared 离线派发。
- 建议按 completed 未 review、prepared、ready_to_dispatch 的优先级选择工作项。

本轮已按该建议修复。

