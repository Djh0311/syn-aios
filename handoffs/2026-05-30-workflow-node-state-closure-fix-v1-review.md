# Review：工作流节点状态收口修正 v1

## 结论

接受。

接受为：

- 后端新派发路径的 workflow node 状态收口已修正。
- completed 后 work item 进入 `ready_for_review`，current node 进入 review，实际派发节点不再残留 `running`。
- failed / timed_out 后 work item 和实际派发节点分别收口为 `failed` / `timed_out`。
- Rust 离线测试覆盖 started、completed、user reviewed completed、failed、timed_out。

不接受为：

- 真实 workflow state 里的存量 `codex-dev=running` 已修复。
- 新一轮真实 README smoke 已执行。
- 取消路径已覆盖。
- 复杂业务自动编排完成。

## 薄弱点

- 存量真实 workflow state 仍显示 README smoke 的 codex-dev node 为 `running`；本任务边界禁止写真实 state，所以这不是本轮失败，但必须另起任务处理。
- 取消路径没有新增测试覆盖。
- 本轮通过离线测试验证代码路径，没有真实执行新的 `codex exec resume`。
- `dist/` 因 `npm run build` 重新生成，接受为构建产物变化。
- 执行线有一次错误复核命令，zsh 返回 `unmatched "`；已记录且后续用 `rg -F` 复核，没有扩大影响。

## 回收依据

已复核：

- `evidence/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 真实 workflow state 摘要

## 关键复核结果

### started 路径

接受。

依据：

- `write_started_dispatch` 使用 `context.node_id` 写 work item `current_node_id`。
- `write_started_dispatch` 使用 `context.node_id` 把实际派发节点置为 `running`。
- 测试 `workflow_node_dispatch_started_marks_actual_dispatch_node_running` 覆盖。

### completed 路径

接受。

依据：

- `write_completed_dispatch` 从 dispatch 记录读取实际派发 `node_id`。
- work item 写为 `ready_for_review`。
- work item current node 写为 review。
- review 节点写为 `ready_for_review`。
- 实际派发节点写为 `ready_for_review`，不再残留 `running`。
- safe probe 和 user reviewed completed 测试均覆盖。

### failed / timed_out 路径

接受。

依据：

- `write_failed_dispatch` 从 dispatch 记录读取实际派发 `node_id`。
- failed 时 work item 和实际派发节点写为 `failed`。
- timed_out 时 work item 和实际派发节点写为 `timed_out`。
- dispatch state 仍沿用 `failed`，超时语义由 execution control / attempt 表示。
- 对应 Rust 测试覆盖。

### 存量真实 state

未接受为已修复。

依据：

- 只读复核显示：
  - README smoke work item：`ready_for_review`
  - current node：review
  - codex-dev node：`running`
- 本任务明确禁止写真实 workflow state。

## 验证结果

总指导复跑：

- `cargo fmt`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，64 passed、0 failed、1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，3 个离线交互测试。
- `npm run build`：通过，重新生成 `dist/`。
- `/Users/yoyi/miniconda3/bin/python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：固定字符串搜索完成，输出为历史记录和任务文本命中，没有触发命令替换。

## 边界

- 是否执行新的 `codex exec`：否。
- 是否执行新的 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test/README.md`：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否运行 harness：否。

## 回收决定

本轮通过。

下一步建议：

- 写一个单独任务包，修复真实 workflow state 里的存量 README smoke `codex-dev=running`。
- 该任务必须获得明确批准，因为会写真实 workflow state。
- 修复内容应只针对已完成派发的存量状态，不执行新的 `codex exec resume`，不写 `/Users/yoyi/.codex`，不改 README。
