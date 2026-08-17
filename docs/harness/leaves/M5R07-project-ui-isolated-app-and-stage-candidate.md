# M5R07 项目 UI、隔离 App 与阶段候选

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用冻结 DTO 接现有项目壳，不重写 execution kernel 或页面布局；在隔离 app-data / scratch 上证明完整闭环；形成只含 M5 投影的 candidate series 后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`。不自行关闭 stage-14，不宣布 M5 完成。

状态：`AWAITING_INDEPENDENT_ACCEPTANCE`。`1433d51466e59352cc8859e1c47f176da04f25b0` 仅 gateway/Dispatch readback scoped implementation candidate 已独立验收，不是 evidence-binding，也不是 closeout。terminal RecordExecutionAttemptReadback + claim 尚未完成。

来源收据：用户明确把提示内剩余工作做完；M5R06 PASS（`867fd20`）。

产品：m5_dto.rs、m5_product_commands.rs、m5_isolated_acceptance.rs；已发生必要相邻执行路径 m5_agent_runtime.rs、m5_controlled_execution.rs、m5_orchestration_schema.rs、m5_orchestration_store.rs、m5_orchestration_service.rs、m5_runner_entry_registry.rs、m5_runtime_admission.rs。下一最窄：RecordExecutionAttemptReadback + claim 前置。

证据：docs/harness/reports/M5R07-project-ui-isolated-app-and-stage-candidate.md、docs/harness/reports/M5R07-isolated-acceptance-receipt.json。`1433d51` 不得当作 evidence 或 closeout。

载体：gateway/Dispatch readback scoped implementation candidate `1433d51466e59352cc8859e1c47f176da04f25b0`（scoped 独立验收，非 evidence/closeout）。terminal readback 尚无新 candidate。

允许动（M5R07 最窄修正，2026-08-17 独立验收返修；同步已发生必要相邻执行路径，并预列下一最窄 terminal readback）：

- docs/contracts/（仅新增/修正 M5 UI/DTO 补充合同，以及已发生的 `m5-r07-product-path-correction-v1` 产品路径修正；不泛化为任意合同改写）
- prototypes/productized-desktop-shell/src-tauri/src/m5_dto.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_m3_identity.rs [新增：只消费 M3 RoleSession]
- prototypes/productized-desktop-shell/src-tauri/src/m5_isolated_acceptance.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_supervisor.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_summary.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_service.rs（正式授权入口接线；独立锚链 admission；Dispatch readback 同事务与 truth carriers；不泛化为任意编排改写）
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（AppState 最小安装 + command 声明 + `runtime_admission` 模块声明；不泛化为任意 lib 改写）
- prototypes/productized-desktop-shell/src-tauri/src/m5_agent_runtime.rs（已发生：正式 runtime 删除 synthetic fail_cell；`run_conformance_suite` 仅测试）
- prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs（已发生：effect 首写前证明 Dispatch readback substrate）
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_schema.rs（已发生：Dispatch readback / origin-outbox exact join 所需 schema）
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_store.rs（已发生：origin-outbox exact join、exact replay loaders）
- prototypes/productized-desktop-shell/src-tauri/src/m5_runner_entry_registry.rs（已发生：登记真实 admission symbol）
- prototypes/productized-desktop-shell/src-tauri/src/m5_runtime_admission.rs [新增：opaque capability，首写前 admission，按值单次消费]
- prototypes/productized-desktop-shell/src-tauri/src/m5_runtime_receipt.rs（仅下一最窄 terminal RecordExecutionAttemptReadback）
- prototypes/productized-desktop-shell/src-tauri/src/m5_prepared_attempt.rs（仅下一最窄 terminal Attempt 状态 / DISPATCHED 后 execution readback）
- prototypes/productized-desktop-shell/src-tauri/src/m5_claim_ledger.rs（仅下一最窄 claim 前置；不得在 terminal readback 完成前把 claim 写成可接受 EXECUTED）
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs（仅登记 M5 command）
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs（**仅因** `AppState` 新增 `m5_store_path` 后测试字面量 E0063；只补该字段，不改其它 command 语义）
- prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs（同上：只补 `m5_store_path: None`，不改读模型边界）
- prototypes/productized-desktop-shell/src/lib/tauri.ts、src/lib/m5ProjectSupervisor.ts [新增]
- prototypes/productized-desktop-shell/src/views/projects/ProjectSupervisorPanel.tsx [新增]
- prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx（仅接入主管面板）
- prototypes/productized-desktop-shell/src/main.tsx（仅新增 M5 隔离 DOM 驱动，不改其它验收桥）
- prototypes/productized-desktop-shell/scripts/run-m5-isolated-app-acceptance.mjs [新增]
- prototypes/productized-desktop-shell/scripts/m5-x11-screenshot.py [新增]
- tasks/2026-08-16-syn-m5r07-narrow-acceptance-fix-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-gateway-dispatch-readback-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-dispatch-readback-consumption-repair-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-terminal-execution-readback-v1.md [拟新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md（仅独立验收 closeout；当前用户边界禁止改 plan/stage/authorization）
- docs/harness/reports/M5R07-*
- docs/harness/leaves/M5R07-project-ui-isolated-app-and-stage-candidate.md
- docs/harness/done/2026-08/M5R07-* [仅独立验收后 closeout 才归档]

不许动：

- 自行关闭 stage-14 或宣布 M5 完成
- 把 `1433d51` 写成 evidence、closeout 或 M5 完成
- 预列或改 `worker_report.rs`，除非后续实现证明必要时再由模型更新 leaf
- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 真实资料/provider/push/reset；伪造窗口截图或 Hook receipt
