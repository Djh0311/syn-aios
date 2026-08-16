# M5R05 受控执行、恢复与 runtime conformance 报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R05
- 结果：`M5R05_CONTROLLED_EXECUTION_AND_RUNTIME_CONFORMANCE=PASS`
- 合同：`docs/contracts/m5-controlled-execution-and-runtime-conformance-v1.md`

## 行为

- `stop` / `retry` / `resume` 改 SQLite 持久状态；进程重开可读回。
- `OUTCOME_UNKNOWN` 必须先按 effect id reconcile，禁止盲 retry。
- 超限进入 `DEAD_LETTERED`。
- `SynNativeAgentRuntime`（生产默认）与 `ObservingFakeRuntime`（观察日志，语义独立）共跑同一 conformance：child grant 不扩权、dynamic package 默认关闭、duplicate effect、receipt lost → UNKNOWN。
- 产出 `RuntimeReceipt` 供 M5R03 验证。不接真实模型/DSH。

## 验证

`cargo test --lib --offline -- m5_`：76 passed。
