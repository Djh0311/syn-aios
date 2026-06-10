# Evidence：工作流用户审核业务派发修正 v1

## 结论

修正任务已完成代码验证；没有执行真实业务派发。

## 薄弱点

- 真实业务会话仍未跑本轮修正后的业务派发，所以真实 Codex 环境中的失败输出分类只通过离线 stub 覆盖。
- 超时已在 runner 中生效，会 kill 子进程并记录 `timeout`，但没有真实长任务验证。
- warning 分类基于派发参数和 runner 结果，不能替代真实业务文件 diff。

## 做了什么

- 修复业务派发 readback：`read_workflow_node_dispatch_result_at` 现在从 dispatch 中恢复 `user_reviewed_instruction`，不再固定传 `None`。
- `RealCodexResumeRunner` 使用 `timeout_seconds` 做 `try_wait` 轮询，超时后 kill 子进程并返回 `timed_out`。
- 业务派发失败时写入 `workflow_execution_controls[]` 和 `execution_attempts[]`。
- 增加失败 warning 分类：
  - `sandbox_read_only`
  - `target_path_not_writable`
  - `allowed_write_roots_missing`
  - `codex_resume_exit_nonzero`
  - `codex_resume_spawn_failed`
  - `timeout`
- 修复离线测试并发下 last-message 文件路径碰撞：输出路径增加毫秒后缀。
- 保持 safe probe 路径不带业务权限参数。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `CURRENT.md`
- `tasks/README.md`
- `prototypes/productized-desktop-shell/dist/index.html`
- `prototypes/productized-desktop-shell/dist/assets/index-CoLiWPD6.js`
- `prototypes/productized-desktop-shell/dist/assets/index-BTACVauc.css`

## 状态边界

- 是否执行真实 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test`：否。
- 是否修改 `/Users/yoyi/gameai/agent world`：否。

## `dist/` 处理

- 已执行 `npm run build`。
- `dist/` 是任务包本轮明确允许的构建产物。
- `dist/index.html`、`dist/assets/index-CoLiWPD6.js`、`dist/assets/index-BTACVauc.css` 时间戳更新；文件名未变化。

## 验证

- `cargo fmt`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，63 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，3 个离线交互测试通过。
- `npm run build`：通过。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过，固定字符串搜索完成，没有触发命令替换；输出包含历史记录和任务包文本。

## 新增 Handoff

- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-result.md`
