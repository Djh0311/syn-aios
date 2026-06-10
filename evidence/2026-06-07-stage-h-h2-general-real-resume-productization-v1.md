# Stage H / H2 General Real Resume Productization Evidence v1

日期：2026-06-07

状态：H2.5 Phase A 已完成；H2 / H2.5 Phase B 未完成。  
结论：接受为 `codex-local` real resume runner 产品路径的非执行 Phase A 完成；不接受为真实 `codex exec resume` 已执行、prompt 已发送、`.codex` 已读写、H2 通用真实 resume 完成、H3 通用 send / 新会话可开始、H5 项目工作流真实派发完成或阶段 H 完成。

## 1. 本轮范围

本轮执行 `tasks/2026-06-07-stage-h-h2-5-real-resume-runner-execution-path-and-authorized-fixture-run-v1.md` 的 Phase A。

Phase A 允许：

- 实现可替换 runner / process runner 边界。
- 从 H2.3 `CodexLocalExecutionRequest` 派生结构化 command plan。
- 写入工作台自有 `session-continuations.v1.json` continuation attempt / audit。
- 记录 non-execution runner attempt、readback unavailable / failed / timed out 分类和 duplicate guard。
- 用 fake / no-op runner 测试成功、超时、readback failed、duplicate blocked 等分类。

Phase A 禁止且本轮未做：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未创建真实 fixture session。
- 未启动 Tauri / GUI / 截图。

## 2. 代码落点

- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
  - 新增 `CodexLocalPhaseAProcessRunner` trait、`CodexLocalPhaseAProcessResult` 和 `NoopCodexLocalPhaseAProcessRunner`。
  - 新增 `run_h2_phase_a_with_runner`，保留结构化 command plan / stdin prompt ref / no shell 边界。
  - H2.5 Phase A attempt 固定 `execution_level = "h2_phase_a_runner_path_no_real_codex"`。
  - 固定 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
  - `readback_unavailable` / `readback_failed` / `timed_out` 的 `result_count` 保持 `None`。

- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - 新增 `run_real_resume_phase_a` 和内部可替换 runner 版本。
  - 每次运行 Phase A 前重新检查 H2 授权矩阵和 CodexLocal guard，不复用旧 preflight 结果。
  - 写入 `SessionContinuationAttempt`、started / completed / blocked audit event 和 continuation 状态。
  - duplicate `queued/running/running_h2_phase_a` 等状态会阻断为 `duplicate_blocked`。
  - 显式 `execution_decision = "rejected"` 会写 `user_rejected`。

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 `RunControlledSessionContinuationRealResumePhaseAInput`。
  - 新增 `RunControlledSessionContinuationRealResumePhaseAOutput`。

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - 新增 Tauri command `run_controlled_session_continuation_real_resume_phase_a`。
  - 默认只调用 no-op Phase A runner，不调用真实 Codex process。

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 注册 Phase A command。

- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 Phase A input / output TS 类型。

- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
  - 新增 Phase A Tauri wrapper。

## 3. 验证记录

已通过：

```text
cargo test --lib codex_local
cargo test --lib session_continuation
cargo test --lib
rustfmt --check src/codex_local_runner.rs src/session_continuation_store.rs src/types.rs src/commands.rs src/lib.rs
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果摘要：

- `cargo test --lib codex_local`：6 passed。
- `cargo test --lib session_continuation`：9 passed。
- `cargo test --lib`：242 passed，1 ignored。
- `npm run test:offline-interaction`：12 passed。
- `npm run build`：通过，保留既有 Vite chunk-size warning。
- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。

过程说明：

- 曾误执行一次无效命令 `cargo test --lib codex_local session_continuation`，Cargo 返回参数错误；未触发产品代码、未触发真实 Codex。
- 尝试派发只读复核子线程失败，原因是 agent thread limit reached；未打断已有线程。

## 4. 测试覆盖

新增 / 更新覆盖：

- CodexLocal guard 仍只允许 `codex-local` + contractual operation + confirmed authorization + structured command plan。
- H2 Phase A no-op runner 记录非真实执行 attempt。
- H2 Phase A timeout / readback failed 分类不会把 `result_count` 写成 0。
- H2 Phase A continuation store 写 attempt / audit / continuation 状态。
- duplicate running attempt 阻断为 `duplicate_blocked`，且不会调用 runner。
- corrupt JSON / revision conflict / planned adapter 等既有边界继续通过。

## 5. 扫描结果

误导文案扫描命中均为历史任务包里的禁止项 / 不接受项 / 测试黑名单常量，不是本轮新增产品 UI 完成态。

本轮没有新增：

- `Codex 已收到任务` 的产品展示。
- `真实 resume 已执行` 的产品展示。
- `prompt 已发送` 的产品展示。
- `planned adapters 已接入` 的产品展示。
- `provider credential 已验证` 的产品展示。

## 6. 接受范围

接受为：

- H2.5 Phase A 非执行产品路径完成。
- `CodexLocalPhaseAProcessRunner` 可替换边界完成。
- 工作台自有 Phase A attempt / audit / readback summary / duplicate guard 链路完成。
- command safety、runner no-op、timeout / failed / readback unavailable 分类测试完成。

不接受为：

- H2 通用真实 resume 产品化完成。
- H2.5 Phase B 已授权或已执行。
- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- 真实 readback 已完成。
- H3 通用真实 send / 新会话完成或可直接开始。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 7. 下一步

下一步有两个可选方向，不能混淆：

1. H2.5 Phase B：只有用户 / 全局主管再次明确授权 fixture、target session、allowed write roots、prompt summary/ref/hash、`.codex` 最小读写、timeout、readback、runtime log、audit、evidence 和 rollback 后，才能执行一次真实 fixture resume。
2. H2.x 修补：如果主管复核认为 Phase A 的 UI 可见化、runtime log store 显式 sidecar 写入或 permission dialog 还不足，应先补 H2.x，不直接进入 H3。
