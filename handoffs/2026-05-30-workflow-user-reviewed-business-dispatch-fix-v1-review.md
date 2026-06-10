# Review：工作流用户审核业务派发修正 v1

## 结论

接受为代码修正完成。

不接受为真实业务派发已验证。

## 薄弱点

- 本轮没有真实执行 `codex exec resume`，所以真实业务会话可用性还没有被证明。
- 超时 kill 已接入真实 runner，但只通过离线测试验证，没有用真实长任务验证。
- 失败分类已经结构化落账，但真实 Codex stderr/stdout 的细分类仍要靠下一次真实试跑校准。
- `dist/` 产物变化已被任务包允许并记录，但后续仍要决定项目是否长期保留构建产物。

## 回收依据

已读取或复核：

- `/Users/yoyi/workspace/product-line/tasks/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 复核结果

### readback 断点

接受。

依据：

- `read_workflow_node_dispatch_result_at` 会在 `prompt_kind == "user_reviewed_instruction"` 时，从 dispatch 的 `user_reviewed_instruction` 字段恢复 payload。
- 不再固定传 `None`。

### 超时

接受为代码能力。

依据：

- `RealCodexResumeRunner` 使用 `try_wait` 轮询。
- 超过 `timeout_seconds` 后调用 `child.kill()`，再 `wait()` 回收。
- 返回结果带 `timed_out`。

不接受为真实长任务已验证。

### 失败落账

接受。

依据：

- `write_failed_dispatch` 对 `user_reviewed_instruction` 会写 `workflow_execution_controls[]`。
- 同时写 `execution_attempts[]`。
- 超时时 attempt state 为 `timed_out`，并写 `timed_out_at`。

### 失败分类

接受为初版分类。

依据：

代码里已有：

- `timeout`
- `codex_resume_exit_nonzero`
- `codex_resume_spawn_failed`
- `sandbox_read_only`
- `allowed_write_roots_missing`
- `target_path_not_writable`

真实输出细分后续继续校准。

### `dist/` 边界

接受。

依据：

- 修正任务包允许 `dist/` 作为 `npm run build` 产物变化。
- evidence / handoff 记录了 `dist/index.html`、`dist/assets/index-CoLiWPD6.js`、`dist/assets/index-BTACVauc.css` 更新。

## 验证记录

开发线回传：

- `cargo fmt`：通过。
- `cargo test --offline`：通过，63 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过。

## 回收决定

本轮接受为：

- 用户审核业务派发代码修正完成。
- readback payload 断点修复。
- timeout kill 代码路径接入。
- 业务失败 execution control / attempt 落账接入。
- 初版失败分类接入。

本轮不接受为：

- 真实业务派发闭环已完成。
- 真实长任务超时已验证。
- 真实业务会话写入能力已验证。

## 下一步

可以进入真实 README 极小修改验证的候选阶段，但必须先获得用户明确批准。

原因：

- 下一步会执行真实 `codex exec resume`。
- 会写 `/Users/yoyi/.codex`。
- 会修改 `/Users/yoyi/codex-workflow-mario-test/README.md`。
- 会写真实 workflow state。
