# Evidence: Stage G / G2 Diagnostics Health And Degraded State v1

日期：2026-06-07

## 结论

G2 已完成，接受为：

```text
diagnostics_health_and_degraded_state_readonly_completed
```

只接受为：

- `DiagnosticSummary` / `ServiceDegradedState` / `StoreIntegrityFinding` 最小只读读模型完成。
- `WorkbenchSnapshot.diagnostic_summary` 接入完成。
- 管理入口健康 / 诊断摘要展示完成。
- 关键 store / sidecar 可读性、schema、revision、warning、损坏 JSON 诊断完成。
- adapter unavailable、provider / credential / model boundary、runtime attention / readback boundary、runtime log error、Tauri bridge / session index 缺失、测试环境未验证均能被解释为只读 degraded / warning。
- `diagnostic_summary` 可作为只读 diagnostic bundle 引用，不导出 secret、不生成新文件。

不接受为：

- 自动修复、自动重试、自动恢复。
- 真实 `codex exec` / `codex exec resume`、真实 prompt 发送、真实 worker 执行。
- provider / credential / model 真实验证。
- G3 真实 Tauri / 截图验收完成。
- G4 中间版本端到端回放完成。
- G5 最终权威验收、阶段 G 完成或中间版本最终完成。

## 改动文件

代码：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

文档：

- `tasks/2026-06-06-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`
- `handoffs/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

## 实现证据

- `WorkbenchSnapshot` 新增 `diagnostic_summary`。
- `derive_diagnostic_summary` 从现有只读数据派生健康状态，不写任何 store。
- store integrity probe 只读取 JSON / 文本，损坏 JSON 记录为 degraded，不覆盖原文件。
- missing sidecar 记录为 warning，不自动初始化。
- 管理入口新增 `健康 / 诊断边界`，但仍复用右侧 `管理`，不新增一级入口。
- 离线测试覆盖管理入口诊断文案、readback unavailable 边界、诊断 bundle 文案和不新增诊断顶级入口。

## 验证结果

已通过：

```text
cargo test --lib g2_diagnostic
```

结果：通过，1 passed。保留既有 `JsonRpcError::invalid_params` dead code warning。

```text
cargo test --lib runtime_log
```

结果：通过，1 passed。保留既有 `JsonRpcError::invalid_params` dead code warning。

```text
cargo test --lib
```

结果：通过，232 passed / 0 failed / 1 ignored。保留既有 `JsonRpcError::invalid_params` dead code warning。

```text
rustfmt --check src/types.rs src/lib.rs src/runtime_log_store.rs
```

结果：通过。

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 12`。

```text
npm run build
```

结果：通过；保留既有 Vite chunk size warning。

口径扫描：

- G2 待开始 / G2-G5 尚未完成 / G1 任务包尚未创建等旧口径扫描无命中。
- 越界文案扫描命中均为禁止项、边界说明或测试黑名单常量；未发现 G2 新增真实执行、自动修复、G3-G5 已完成等误导口径。

## 脱敏与禁止项

本轮未读取 / 未写入：

- `/Users/yoyi/.codex`
- auth、token、`.env`、secret、keychain、OAuth、provider credential
- 完整 transcript / rollout

本轮未执行：

- `codex exec`
- `codex exec resume`
- 外部 provider / 模型调用
- 自动修复 / 自动重试

## 多线程复核输入

只读复核线指出的问题已处理或记录：

- 已新增 G2 task / evidence / handoff。
- 已补 G2 文档入口同步。
- 已补 `revision` 字段。
- 已补 Tauri bridge / session index、测试环境未验证和 diagnostic bundle 引用状态。
- diagnostic bundle 本轮只作为 `WorkbenchSnapshot.diagnostic_summary` 引用，不落盘导出文件。

验收准备线建议保留：

- G3 应拆为真实 Tauri 验收范围冻结、fixture 准备、截图采集、手动清单、证据回收。
- G4 默认离线 fixture 回放，不默认真实 Codex。

## 当前结论

G2 已完成。下一步是 G3 Real Tauri Acceptance Harness And Screenshot Evidence 待开始 / 待拆。G3-G5 和阶段 G 不得标记完成。
