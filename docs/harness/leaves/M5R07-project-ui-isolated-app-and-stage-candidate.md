# M5R07 项目 UI、隔离 App 与阶段候选

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用冻结 DTO 接现有项目壳，不重写 execution kernel 或页面布局；在隔离 app-data / scratch 上证明完整闭环；形成只含 M5 投影的 candidate series 后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`。不自行关闭 stage-14，不宣布 M5 完成。

状态：`AWAITING_INDEPENDENT_ACCEPTANCE` / `NOT_CLOSEOUT` / `NOT_M5_COMPLETE`。`1433d51466e59352cc8859e1c47f176da04f25b0` 是 gateway/Dispatch readback scoped predecessor（已独立 scoped PASS，不是 evidence-binding，也不是 closeout）。implementation exact `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree exact `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`）已获产品 + Git/Harness scoped independent PASS。fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b` 的 evidence 内容/载体已获 Git/Harness scoped independent PASS，非 closeout。不得把 implementation 或 evidence scoped PASS 写成 M5 / stage 完成。stage-14 仍开；authorization closed；M6 未激活。

来源收据：用户明确把提示内剩余工作做完；M5R06 PASS（`867fd20`）。

产品：正式 M5 authority → Grant → Dispatch readback → runtime → RecordExecutionAttemptReadback → terminal-gated EXECUTED claim。ordinary M1 composition 与 M6 排除。已发生路径覆盖 m5_dto.rs、m5_product_commands.rs、m5_isolated_acceptance.rs、m5_agent_runtime.rs、m5_controlled_execution.rs、m5_orchestration_schema.rs、m5_orchestration_store.rs、m5_orchestration_service.rs、m5_runner_entry_registry.rs、m5_runtime_admission.rs、m5_prepared_attempt.rs、m5_claim_ledger.rs、m5_runtime_receipt.rs。不改 worker_report.rs / 前端；不把定向测试数写成阶段完成。

两栏（不要混读）：

1. ordinary disposable backend full-loop PASS 只是 server fixture 预登记 M1 alias + M3 authority 的后端/产品命令闭环，不是 GUI。
2. shared isolated 真实 Vite+Tauri/Xvfb 只证明 authority-unavailable fail-closed；`NO_UI_PASS` / `NO_WINDOW_CAPTURE`；scene / resume / second launch `NOT_EXECUTED`。

唯一已知 stage-14 blocker：ordinary legacy `ProjectRecord` 到 M1 canonical/exact alias 的可信创建/迁移/GUI composition 尚未闭合，排除于 M5 实现。不得反向写 `f51c3f64` 产品 FAIL，也不得 close stage。

证据：fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b`；evidence 内容/载体已获 Git/Harness scoped independent PASS，非 closeout。当前 `docs/harness/reports/M5R07-project-ui-isolated-app-and-stage-candidate.md` 与 `docs/harness/reports/M5R07-isolated-acceptance-receipt.json` 绑定 implementation exact `f51c3f64` / tree `dbdeaeda`。旧 `faa6ed1` 证据只在 `docs/harness/reports/M5R07-history/faa6ed1/manifest.json`，非 current。`1433d51` 与 `f51c3f64` 是 predecessor / implementation，不是 evidence tip。不得把 implementation 或 evidence scoped PASS 写成 M5 / stage 完成。下一步只写独立 lifecycle/closeout review 或另行适用授权后的 M1 owner correction；不得自动进入 M1 修正、M6 或 closeout。

载体：implementation exact `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`；产品 + Git/Harness scoped independent PASS；非 evidence tip / 非 closeout）。fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b`（evidence 内容/载体 + Git/Harness scoped independent PASS；非 closeout）。gateway/Dispatch predecessor `1433d51466e59352cc8859e1c47f176da04f25b0`。

允许动（M5R07 最窄修正，2026-08-17 独立验收返修；保持已同步路径与 terminal task；下一步仅独立 lifecycle/closeout review 或另行适用授权后的 M1 owner correction；不得自动进入 M1 修正、M6 或 closeout）：

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
- prototypes/productized-desktop-shell/src-tauri/src/m5_runtime_receipt.rs（已发生：terminal RecordExecutionAttemptReadback）
- prototypes/productized-desktop-shell/src-tauri/src/m5_prepared_attempt.rs（已发生：terminal Attempt 状态 / DISPATCHED 后 execution readback）
- prototypes/productized-desktop-shell/src-tauri/src/m5_claim_ledger.rs（已发生：EXECUTED claim 前置 terminal readback gate）
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
- tasks/2026-08-17-syn-m5r07-terminal-execution-readback-v1.md [新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md（仅独立验收 closeout；当前用户边界禁止改 plan/stage/authorization）
- docs/harness/reports/M5R07-*
- docs/harness/leaves/M5R07-project-ui-isolated-app-and-stage-candidate.md
- docs/harness/done/2026-08/M5R07-* [仅独立验收后 closeout 才归档]

不许动：

- 自行关闭 stage-14 或宣布 M5 完成
- 把 implementation scoped PASS 或 evidence scoped PASS 写成 M5 完成、stage closeout
- 自动进入 M1 修正、M6 或 closeout
- 把定向测试数写成阶段完成
- 预列或改 `worker_report.rs`，除非后续实现证明必要时再由模型更新 leaf
- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 真实资料/provider/push/reset；伪造窗口截图或 Hook receipt
