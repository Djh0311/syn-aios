# Handoff：工作流用户审核业务派发 v1

## 结论

代码能力已完成，真实业务派发未执行。

## 薄弱点

- 本轮没有用户明确批准真实派发，所以没有跑真实 `codex exec resume`，不能证明真实会话在当前机器状态下可完成业务写入。
- 自动重试仍未开放，后端要求 `max_retries = 0`。
- 权限失败分类还没有从真实 Codex stderr/stdout 细分，只在状态结构和测试路径中留了接入点。

## 已完成

- 后端接受 `prompt_kind = user_reviewed_instruction`，要求携带完整用户审核业务指令。
- 后端用参数数组传入 `-C <execution_cwd>`、`--sandbox <sandbox_mode>`、重复 `--add-dir <allowed_write_root>`，prompt 走 stdin。
- safe probe 路径保持可用，测试覆盖没有传业务权限参数。
- UI “审核后派发”只在会话已绑定、工作项 `ready_to_dispatch`、审核指令完整且 `approval_state = reviewed` 时启用。
- 确认弹层展示执行目录、沙箱、允许写入根目录、允许读取、允许写入、禁止事项、超时 / 重试、必须回传，并提示会写 `/Users/yoyi/.codex`。

## 未执行

- 是否执行真实 `codex exec resume`：否。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否写真实 workflow state：否。
- 是否读取授权、密钥、`.env`、token：否。
- 是否读取完整 transcript：否。
- 是否触碰真实业务会话：否。

## 验证结果

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `cargo fmt`：通过。
- `cargo test --offline` 使用共享缓存路径：通过，60 passed，1 ignored。
- `npm run build`：通过。
- `python3 prototypes/index-kernel/build_index.py --check prototypes/index-kernel/codex-index.json`：通过。
- `rg -F 'codex exec resume' tasks evidence handoffs CURRENT.md`：通过，固定字符串搜索完成。

## 下一步

由总指导回收本轮代码能力。若要进入真实业务验证，需要用户明确批准，因为下一步会执行真实 `codex exec resume` 并写 `/Users/yoyi/.codex`。
