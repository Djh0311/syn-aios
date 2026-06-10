# Handoff：工作流用户审核业务派发修正 v1

## 结论

本轮修掉上一版 review 指出的主要代码断点，等待总指导回收。

## 薄弱点

- 本轮仍未执行真实 `codex exec resume`，所以不能说真实业务派发已跑通。
- 超时 kill 已接入 runner，但没有用真实长任务验证。
- 失败分类已结构化落账，但真实 stderr/stdout 的精细分类还需要真实试跑后继续校准。

## 已完成

- 业务派发 readback 会从 dispatch 恢复 `user_reviewed_instruction` payload。
- 真实 runner 使用 `timeout_seconds`，超时后 kill 子进程并返回 `timed_out`。
- 业务失败路径写 `workflow_execution_controls[]` 和 `execution_attempts[]`。
- 失败 warning 覆盖 `sandbox_read_only`、`target_path_not_writable`、`allowed_write_roots_missing`、`codex_resume_exit_nonzero`、`codex_resume_spawn_failed`、`timeout`。
- safe probe 测试仍通过。
- `dist/` 产物随 `npm run build` 更新，本任务允许并已记录。

## 未执行

- 是否执行真实 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否读取敏感文件或完整 transcript：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test`：否。
- 是否修改 `/Users/yoyi/gameai/agent world`：否。

## 验证结果

- `cargo fmt`：通过。
- `cargo test --offline` 使用共享缓存路径：通过，63 passed，1 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，更新 `dist/` 构建产物。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过。

## 下一步判断

可以进入总指导回收。是否进入真实 README 极小修改验证，需要用户明确批准，因为那会执行真实 `codex exec resume` 并写 `/Users/yoyi/.codex`。
