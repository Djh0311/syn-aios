# Handoff：Codex 角色编排离线账本复核修复 v1

## 薄弱点

- 这仍不是真实多会话自动编排。
- 没有执行 `codex exec` 或 `codex exec resume`。
- 没有通过真实 UI 写真实 workflow state。
- 派发预览仍不是实时编辑预览，只是默认示例。

## 做了什么

- 修复回传后 UI 丢失账本锚点的问题。
- 修复 completed 离线派发未 review 时“写入总指导回收”不可用的问题。
- 后端拒绝同一 work item 重复写 `prepared` 离线派发。
- 补前端 completed 未回收场景测试。
- 补 Rust 重复 prepared 拒绝测试。

## 改了哪些文件

- `prototypes/productized-desktop-shell/src/views/OfflineRoleOrchestrationPanel.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DDJziFyU.css`
- `prototypes/productized-desktop-shell/dist/assets/index-DfqNlrl_.js`

## 新增 Evidence

- `evidence/2026-05-30-codex-role-orchestration-offline-ledger-review-fix-v1.md`

## 验证

- `cargo fmt`：通过。
- 指定共享 Cargo 缓存的 `cargo test --offline`：通过，67 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过。
- 索引校验：`validation_ok`。
- 固定字符串搜索 `codex exec resume`：完成，没有触发命令替换。

## 当前可回收口径

可接受为：

- 离线账本 UI 三步闭环的 P1 阻塞已修复。
- 重复 prepared 派发已被后端拒绝。
- 测试覆盖已补齐。

不可接受为：

- 真实 Codex 多角色自动执行。
- 总指导自动计划生成。
- 真实业务工作流自动化完成。

