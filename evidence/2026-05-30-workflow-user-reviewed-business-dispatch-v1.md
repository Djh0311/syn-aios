# Evidence：工作流用户审核业务派发 v1

## 结论

本轮完成桌面壳代码能力，不执行真实业务派发。

## 薄弱点

- 没有真实执行 `codex exec resume`，所以只能证明代码路径和离线 stub 行为，不能证明当前用户真实 Codex 会话一定可写。
- `max_retries` 当前被后端限制为 0；自动重试协议还不能说已产品化。
- 完成结果里的写入范围判断仍依赖后续 readback / review；本轮没有做真实业务文件差异验证。
- `permission_requests[]` 的权限阻塞分类还没有完整接入真实 Codex 失败输出，只保留了状态字段和后续扩展位置。

## 改动范围

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - `user_reviewed_instruction` 接入 prepare / execute 请求。
  - 后端校验执行目录、沙箱、允许写入根目录、读写边界、禁止事项、超时、回传字段。
  - `RealCodexResumeRunner` 使用参数数组和 stdin；业务派发传 `-C`、`--sandbox`、重复 `--add-dir`。
  - completed 结果补写 `workflow_execution_controls[]` 和 `execution_attempts[]`。
  - 增加 Rust 离线测试，覆盖 safe probe 保持、业务派发参数、字段校验。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 增加用户审核业务派发请求类型和 workflow snapshot 字段。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 业务派发确认弹层展示执行目录、沙箱、允许写入根目录、读写范围、禁止事项、超时 / 重试、必须回传。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 完整指令、已绑定会话、`ready_to_dispatch` 时启用“审核后派发”。
  - 将结构化审核指令转换成后端派发请求。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 更新离线夹具和断言，覆盖业务派发确认动作。
- `CURRENT.md`
- `tasks/README.md`

## 状态写入

- 是否写真实 workflow state：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行真实 `codex exec resume`：否。
- 是否读取授权、密钥、`.env`、token：否。
- 是否读取完整 transcript：否。
- 是否触碰真实业务会话：否。
- 是否修改 `/Users/yoyi/codex-workflow-mario-test`：否。

## 写入字段类型

代码路径支持写入这些 workflow state 字段类型，但本轮只在测试临时状态中验证：

- `workflow_node_dispatches[].user_reviewed_instruction`
- `workflow_node_dispatches[].prompt_kind`
- `workflow_node_dispatches[].last_message_summary`
- `workflow_node_dispatches[].transcript_event_count`
- `workflow_node_dispatches[].transcript_target_hits`
- `workflow_execution_controls[]`
- `execution_attempts[]`
- `audit_events[]`

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，3 个离线交互测试通过。
- `cargo fmt`：通过。
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`：通过，60 passed，1 ignored。
- `npm run build`：通过。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过，`validation_ok`。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过，固定字符串搜索完成；输出只显示历史文档和任务包记录。

## Handoff

- 新增 handoff：`handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-result.md`
