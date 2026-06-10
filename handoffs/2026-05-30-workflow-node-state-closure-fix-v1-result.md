# Handoff：工作流节点状态收口修正 v1

## 结论

工作流节点状态收口代码已修正，等待总指导回收。

这不是新一轮真实 README smoke，也没有修复真实 workflow state 里的存量脏状态。

## 薄弱点

- 存量真实 workflow state 中 README smoke 的 codex-dev 节点仍可能是 `running`，本轮按任务边界没有写真实 state。
- 本轮通过离线测试验证代码路径，没有执行新的真实派发。
- 取消路径没有新增覆盖；本轮覆盖 completed / failed / timed_out。
- `dist/` 因 `npm run build` 重新生成，需要按构建产物处理。
- 有一次末尾复核命令写法错误，把含反引号文本放进 shell 双引号，zsh 返回 `unmatched "`；随后已用单引号 / `rg -F` 重跑。

## 做了什么

- `write_started_dispatch` 改为使用实际 dispatch `node_id` 标记 work item current node 和 node running。
- `write_completed_dispatch` 从 dispatch 记录读回实际派发 `node_id`，completed 后同时更新 review 节点和原实际派发节点。
- `write_failed_dispatch` 从 dispatch 记录读回实际派发 `node_id`，failed / timed_out 后更新 work item 和实际派发节点，不再让执行节点停在 `running`。
- 增加 Rust 离线测试覆盖 started、completed、user reviewed completed、failed、timed_out。

## 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DEUb_c1t.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BTACVauc.css`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`

## 状态规则

- completed：work item 进 `ready_for_review`，current node 进 review，review 节点为 `ready_for_review`，原 codex-dev 实际派发节点也收口为 `ready_for_review`。
- failed：work item 和实际派发节点收口为 `failed`。
- timed_out：work item 和实际派发节点收口为 `timed_out`；dispatch state 仍为现有 `failed`，attempt/control 表达超时。

## 边界复核

- 是否执行 `codex exec` 或 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`：否。
- 是否运行 harness：否。

## 验证结果

- `cargo fmt`：通过。
- 默认 `cargo test --offline`：失败于本机默认 Cargo 缓存版本不匹配，未联网。
- 指定既有缓存路径后 `cargo test --offline`：通过，64 passed、0 failed、1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，`dist/` 已重新生成。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成，没有触发命令替换。
- 一次错误复核命令：失败，`zsh:1: unmatched "`；已更正，没有继续沿用该失败结果。

## 后续

仍需要单独任务、并获得明确批准后，才能修复真实 workflow state 里的存量 `codex-dev=running`。
