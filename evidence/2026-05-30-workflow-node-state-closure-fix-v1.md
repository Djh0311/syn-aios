# Evidence：工作流节点状态收口修正 v1

## 结论

已修正后端 workflow node 状态收口规则。

本轮只改代码、测试和构建产物，没有执行新的 README smoke，没有执行 `codex exec` 或 `codex exec resume`，没有写真实 workflow state。

## 薄弱点

- 本轮没有手工修复真实 workflow state 里的存量 `codex-dev=running`，所以真实账本旧脏状态仍存在。
- 本轮没有真实执行新的派发，只用 Rust 离线测试验证状态写入路径。
- 失败和超时路径已收口为 `failed` / `timed_out`，但取消路径当前没有真实 runner 路径覆盖；本轮没有新增取消行为。
- `npm run build` 重新生成了 `prototypes/productized-desktop-shell/dist/`，构建产物已列入本轮变化。
- 末尾有一次复核命令误把包含反引号的搜索文本放进 shell 双引号，zsh 返回 `unmatched "`；该命令未完成。随后已用单引号 / `rg -F` 重新复核。

## 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-DEUb_c1t.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BTACVauc.css`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`

## 状态收口规则

- started：
  - work item state 写为 `running`。
  - work item current node 写为 dispatch 请求里的实际 `node_id`。
  - 实际派发节点写为 `running`。
- completed：
  - dispatch 记录写为 `completed`。
  - work item state 写为 `ready_for_review`。
  - work item current node 写为 review 节点。
  - review 节点写为 `ready_for_review`。
  - 原实际派发节点从 `running` 收口为 `ready_for_review`，不再永久停在 `running`。
- failed：
  - dispatch 记录写为 `failed`。
  - work item state 写为 `failed`。
  - work item current node 保持实际派发节点。
  - 实际派发节点从 `running` 收口为 `failed`。
- timed_out：
  - dispatch 记录仍写为 `failed`，execution attempt / control 写为 `timed_out`。
  - work item state 写为 `timed_out`。
  - work item current node 保持实际派发节点。
  - 实际派发节点从 `running` 收口为 `timed_out`。

## 离线测试覆盖

- `workflow_node_dispatch_started_marks_actual_dispatch_node_running`
  - 验证 started 后 codex-dev 实际派发节点为 `running`。
- `workflow_node_dispatch_execute_uses_stub_and_advances_to_review`
  - 验证 safe probe completed 后 work item 为 `ready_for_review`，current node 为 review，codex-dev 不再 `running`。
- `workflow_node_dispatch_execute_user_reviewed_instruction_uses_codex_options`
  - 验证用户审核业务派发 completed 后同样进入 review，codex-dev 不再 `running`。
- `workflow_node_dispatch_user_reviewed_failure_writes_control_and_attempt`
  - 验证 failed 后 work item / codex-dev 节点为 `failed`，不残留 `running`。
- `workflow_node_dispatch_user_reviewed_timeout_writes_timed_out_attempt`
  - 验证 timed_out 后 work item / codex-dev 节点为 `timed_out`，不残留 `running`。

## 边界

- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否手工修复真实 workflow state 存量 `running`：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`：否。
- 是否读取授权、密钥、`.env` 或完整 transcript：否。
- 是否运行 harness：否。

## 验证命令和结果

- `cargo fmt`：通过。
- `cargo test --offline`：默认 Cargo 缓存失败，原因是离线缓存候选只有 `serde_json 1.0.149`，而锁文件需要 `1.0.150`。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，64 passed、0 failed、1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 3`。
- `npm run build`：通过，重新生成 `dist/`。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成；输出包含历史任务、evidence、handoff 和当前任务包文本命中，没有触发命令替换。
- 误写复核命令：失败，`zsh:1: unmatched "`；原因是 shell 双引号里包含反引号文本。已更正为单引号 / `rg -F` 搜索。

## 存量状态

仍需要单独任务修复真实 workflow state 里的存量 `codex-dev=running`。

理由：

- 本任务明确禁止写真实 workflow state。
- 本轮修的是未来写入路径，不能把真实账本旧值静默改掉。

## Handoff

- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`
