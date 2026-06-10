# Stage J / J1-B Mario Test Codex Control Real Resume Execution Point Handoff v1

日期：2026-06-09

状态：已执行并通过长期只读复核线复核，结论为 `accepted_with_deferred_items`。

## 1. 回交结论

J1-B 已完成一次指定 `mario test` / 指定 session 的真实 `codex-local resume` 探针。执行通过 J1-A `codex_control` source 构造统一 `real_execution_product_command`，经过 prepare / user decision / Phase A no-op / Phase B 真实 runner，不使用 H5、legacy、direct CLI 或 MCP canvas run 冒充。

本轮真实写入 `/Users/yoyi/.codex` 属于 J1-B 授权执行点范围内的 Codex CLI 原生运行状态写入。mario test 四个核心文件 hash 前后一致。

长期只读复核线已回交带 P2 通过，无 P0/P1。P2 为底层 continuation 仍有 `h2_phase_b` / `controlled_session_continuation` 历史命名债，不影响本轮产品路径判定。

## 2. 关键产物

- Evidence：`evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- 任务包：`tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- 代码：`prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- J1-B run root：`tmp/j1-b-codex-control/runs/j1-b-run-1780998017713782000/`

## 3. 执行摘要

- project root：`/Users/yoyi/Documents/mario test`
- target session：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
- sandbox：`read-only`
- prompt ref：`workbench-runtime-prompt:j1-b:mario-test:2547d65c4e86`
- prompt sha256：`2547d65c4e86e6357906a7a55b5923f806f719b952658606e2a6ff9d3797755b`
- product command id：`real-exec-command:codex-control:bc2914f7ad1b`
- product attempt id：`real-exec-command-attempt:phase-b:real-exec-command:codex-control:bc2914f7ad1b:4`
- continuation attempt：`session-continuation-attempt:h2-phase-b:2026-06-09T10:00:03Z:61c8c795710af3a5`
- runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-09T10:00:03Z:61c8c795710af3a5`
- readback：成功读回 J1-B 预期 marker，`result_count=1`

## 4. 验证

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，13 scenarios。
- `npm run build` 通过，仅既有 Vite chunk-size warning。
- `cargo test --lib real_execution_command` 通过，33 passed / 3 ignored。
- `cargo test --lib session_continuation` 通过，17 passed / 4 ignored。
- `cargo test --lib runtime_log` 通过，6 passed。
- `cargo test --lib codex_local_runner` 通过，11 passed。
- `cargo test --lib` 通过，302 passed / 8 ignored。
- `cargo fmt -- --check` 通过。
- J1-B 授权真实执行测试通过，1 passed / 0 failed。

## 5. 边界

- 本轮确实触发真实 `codex-local resume`，并写入 `/Users/yoyi/.codex` 原生运行状态。
- 未直接执行裸 CLI。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未写 mario test 核心业务文件。
- 未自动写 Observation、MemoryCandidate 或 FormalMemory。
- 未启动真实 Tauri / Browser / Chrome / 截图工具。

## 6. 交给复核线

请复核：

- 是否认可 J1-B 是通过 `codex_control` source 而不是 H5 source 完成真实 Phase B。
- 是否认可 prompt body 未进入 product sidecar / continuation / runtime / audit / memory / evidence / handoff。
- 是否认可 readback marker、flags、hash 对比和验证矩阵足以支持 J1-B checkpoint。
- 是否存在 P0/P1 阻断同步入口。

若复核通过，主管线再同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 Stage J 计划，把 J1-B 收口为 `accepted_with_deferred_items`，下一步进入 J2。
