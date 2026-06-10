# Handoff：Stage H / H2 Phase B Mario Test Real Resume Productization Probe v1

日期：2026-06-08

结论：H2 Phase B `mario test` 真实 resume 产品化探针已完成。接受为 `codex-local` 受控真实 resume 最小产品路径和一次 `mario test` 真实探针完成；不接受为 H3、H5、阶段 H、planned adapters、provider credential / model verification、自动重试或任意项目无限制执行完成。

## 完成内容

- 实现并复核 H2 Phase B real resume runner 产品路径。
- 真实执行一次 `codex exec resume` 到 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f`。
- 成功 readback 固定标记：`H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`。
- 写入 continuation sidecar、runtime log sidecar、audit event 和 workbench-managed last message。
- 复核项目文件 hash，`index.html` / `styles.css` / `game.js` / `README.md` 未发现变化。
- 主管复核修补了 Phase B 当前 continuation warnings 混入 Level A 旧状态的问题。

## 关键路径

- Evidence：`evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md`
- 成功 run：`tmp/h2-phase-b-real-resume/run-1780855910749629000/`
- Last message：`tmp/h2-phase-b-real-resume/run-1780855910749629000/runtime/h2-phase-b/mario.last-message.txt`
- Continuation sidecar：`tmp/h2-phase-b-real-resume/run-1780855910749629000/session-continuations.v1.json`
- Runtime log sidecar：`tmp/h2-phase-b-real-resume/run-1780855910749629000/runtime-logs.v1.json`

## 关键代码

- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

## 验证

已通过：

```text
cargo test --lib session_continuation_store -- --nocapture
cargo test --lib codex_local_runner -- --nocapture
cargo test --lib runtime_log_store -- --nocapture
cargo test --lib runtime_session_attention -- --nocapture
cargo test --lib
rustfmt --check src/session_continuation_store.rs src/codex_local_runner.rs src/commands.rs src/types.rs src/lib.rs
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果摘要：

- Rust 全量：249 passed，2 ignored。
- 离线交互：12 scenarios passed。
- Vite build：通过，仅既有 chunk-size warning。
- 既有 Rust warning：`JsonRpcError::invalid_params` unused。

## 边界

本轮确实执行了真实 `codex exec resume`，确实发送了 prompt，确实写入了 `/Users/yoyi/.codex`。

本轮没有执行 H3 真实新会话，没有执行 H5 项目工作流真实派发，没有接 planned adapters，没有读取 credential/secret/auth/token/`.env`，没有读取完整 transcript / rollout 作为 readback，没有把 prompt body 写入 sidecar/audit/runtime/evidence。

## 过程偏差

全局主管收尾扫描时曾出现一次非产品路径偏差：扫描命令中的 Markdown 反引号触发了 ``codex exec resume`` 命令替换。该命令没有 stdin prompt，尝试访问 `/Users/yoyi/.codex/state_5.sqlite` 后因 readonly database 失败；这不是 H2 产品代码路径，也不是成功的 H2 Phase B probe。后续扫描必须使用单引号或避免反引号。

## 下一步

可以进入 H3-B final approval / real new session fixture run，但必须单独批准。H3-B 不能自动继承 H2 Phase B 授权，也不能因为 H2 成功而跳过 fixture、work item / workflow / node、allowed write roots、`.codex` 最小范围、prompt hash/ref、runtime log、audit、readback 和 rollback 确认。
