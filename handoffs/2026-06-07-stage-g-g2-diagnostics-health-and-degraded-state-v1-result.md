# Handoff: Stage G / G2 Diagnostics Health And Degraded State v1

日期：2026-06-07

## 回收结论

G2 可接受为：

```text
diagnostics_health_and_degraded_state_readonly_completed
```

下一步：

```text
G3 Real Tauri Acceptance Harness And Screenshot Evidence 待开始 / 待拆
```

## 接受范围

- `DiagnosticSummary` / `ServiceDegradedState` / `StoreIntegrityFinding` 已接入 `WorkbenchSnapshot.diagnostic_summary`。
- 诊断层只读解释健康状态、store integrity、degraded state、最近错误和边界说明。
- 管理入口显示健康 / 诊断摘要，不新增一级入口或右侧诊断顶级入口。
- 关键 sidecar 的 JSON 可读性、schema、revision、warning 和损坏状态可见。
- `readback_unavailable` 明确不是 0 条结果。
- Tauri bridge / session index 缺失、测试环境未验证、diagnostic bundle 引用均以只读 degraded / warning / info 状态呈现。

## 不接受范围

- 不接受为自动修复、自动初始化 sidecar、自动重试或自动恢复。
- 不接受为真实 Codex / worker 执行、真实 prompt 发送、真实 readback 完成。
- 不接受为 provider / model / credential 真实验证。
- 不接受为 G3 真实 Tauri / 截图验收完成。
- 不接受为 G4 回放、G5 最终冻结、阶段 G 完成或中间版本最终完成。

## 验证

已通过：

- `cargo test --lib g2_diagnostic`
- `cargo test --lib runtime_log`
- `cargo test --lib`
- `rustfmt --check src/types.rs src/lib.rs src/runtime_log_store.rs`
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

结果：

- Rust 全量：232 passed / 0 failed / 1 ignored。
- 离线交互：12 passed。
- build：通过，保留既有 Vite chunk size warning。
- 保留既有 warning：`JsonRpcError::invalid_params` dead code。

## 边界

本轮主线程未执行真实 `codex exec` / `codex exec resume`，未发送真实 prompt，未读写 `/Users/yoyi/.codex`，未读取 auth/token/`.env`/secret/keychain/OAuth/provider credential，未读取完整 transcript / rollout，未调用 provider，未做自动修复或自动重试。

真实 Tauri / 截图验收未做，必须留给 G3。

## 给全局主管

请基于本 handoff 复跑最终验证。如果通过，入口只能推进到 G3 待开始 / 待拆，不得声明阶段 G 完成。
