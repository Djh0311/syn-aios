# M5R05 受控执行、恢复与 runtime conformance

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：经授权的白名单动作可以进入受控执行；stop/retry/resume 改变持久状态；Syn-native 默认 AgentRuntime 与语义独立的第二实现跑同一 conformance；产出可供 M5R03 验证的 RuntimeReceipt。不接真实模型/DSH。

来源收据：用户明确继续完成提示内全部剩余工作；M5R04 PASS（`177399d`）。

产品：m5_controlled_execution.rs、m5_agent_runtime.rs、持久 operation/lease/checkpoint

证据：docs/harness/reports/M5R05-controlled-execution-recovery-and-runtime-conformance.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=177399d）

允许动：

- docs/contracts/（仅新增 M5 runtime/recovery 补充合同）
- prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_agent_runtime.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_store.rs（仅所需接线）
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包最小声明）
- tasks/2026-08-16-syn-m5r05-controlled-execution-recovery-and-runtime-conformance-v1.md [新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md
- docs/harness/reports/M5R05-controlled-execution-recovery-and-runtime-conformance.md [新增]
- docs/harness/leaves/M5R05-controlled-execution-recovery-and-runtime-conformance.md
- docs/harness/done/2026-08/M5R05-controlled-execution-recovery-and-runtime-conformance.md [退场时新增]

不许动：

- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 真实模型/provider/DSH、push/reset、伪造 App/Hook 证据
