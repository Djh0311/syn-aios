# M5R07 项目 UI、隔离 App 与阶段候选

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用冻结 DTO 接现有项目壳，不重写 execution kernel 或页面布局；在隔离 app-data / scratch 上证明完整闭环；形成只含 M5 投影的 candidate series 后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`。不自行关闭 stage-14，不宣布 M5 完成。

状态：`AWAITING_INDEPENDENT_ACCEPTANCE` / `NOT_CLOSEOUT` / `NOT_M5_COMPLETE`。`1433d51466e59352cc8859e1c47f176da04f25b0` 是 gateway/Dispatch readback scoped predecessor（已独立 scoped PASS，不是 evidence-binding，也不是 closeout）。implementation exact `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree exact `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`）已获产品 + Git/Harness scoped independent PASS。fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b` 的 evidence 内容/载体已获 Git/Harness scoped independent PASS，非 closeout。U01a 默认入口 candidate exact `f962038e725ba4e24b2699a46cd1a8d274f13ae6` 与 U01b 安全有限 control candidate exact `70a15a9c2741b364e0fef38d60ab5d5daad4bea3` 已获产品 + Git/Harness scoped independent PASS；二者都不是 evidence-binding / closeout。不得把上述 scoped PASS 写成 M5 / stage 完成。stage-14 仍开；authorization closed；M6 未激活。不得 close。

来源收据：用户明确把提示内剩余工作做完；M5R06 PASS（`867fd20`）。

产品：正式 M5 authority → Grant → Dispatch readback → runtime → RecordExecutionAttemptReadback → terminal-gated EXECUTED claim。U01a 保持默认 `jiaoban` 与三栏布局，把左侧主工作面接到唯一正式 `ProjectSupervisorPanel`；U01b 增加 server-owned load/apply control、durable revision/CAS/replay，并只开放可证明的 STOP / RESUME，复杂 terminal retry 保持禁用。ordinary M1 composition 与 M6 排除。不改 worker_report.rs / 页面布局 / execution kernel；不把定向测试数写成阶段完成。

两栏（不要混读）：

1. ordinary disposable backend full-loop PASS 只是 server fixture 预登记 M1 alias + M3 authority 的后端/产品命令闭环，不是 GUI。
2. shared isolated 真实 Vite+Tauri/Xvfb 只证明 authority-unavailable fail-closed；`NO_UI_PASS` / `NO_WINDOW_CAPTURE`；scene / resume / second launch `NOT_EXECUTED`。

当前直接 blocker（区分 owner；均不得反向写成既有 scoped candidate FAIL，也不得 close）：

1. M1 owner。ordinary legacy `ProjectRecord` 到 M1 canonical/exact alias 的可信创建/迁移/ordinary GUI composition 缺失。必须另行适用授权；M5 不得自动登记/fallback。真正 ordinary GUI composition 与最终 positive 证据仍受本项阻塞。
2. M5 owner。U01 已关闭默认入口与安全有限 control 产品面，但没有建立 complex terminal retry 的新 Attempt / Grant / Dispatch / effect lineage；worker-blocked、terminal failure/recovery、duplicate operation/effect 仍缺 server-derived ordinary positive 场景，不得用翻状态冒充恢复。
3. M5 owner。当前 shared-isolated launcher 是 unavailable-only；scene / window / restart 均 `NOT_EXECUTED`；缺正向 ordinary-product Tauri acceptance carrier。最终 positive ordinary GUI 证据仍受第 1 项 M1 owner 阻塞。

证据：fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b` 仍只绑定 `f51c3f64`；U01a / U01b 尚无 fresh evidence-binding。U01a 独立复核为 exact 两文件、typecheck/build PASS；U01b 为 exact 九路径、cargo check PASS、execution_control 6/6、typecheck/build PASS。下一步若继续，只能另做 U02 ordinary disposable positive Tauri runner / server-owned fixture，或另开 complex retry；shared-isolated 始终只作 authority-unavailable negative regression。真正 legacy ordinary GUI composition 与最终 positive 证据仍受 M1 owner 阻塞。不得自动进入 M1 修正、M6 或 closeout。

载体：U01a `f962038e725ba4e24b2699a46cd1a8d274f13ae6`；U01b `70a15a9c2741b364e0fef38d60ab5d5daad4bea3`（均 scoped independent PASS；非 evidence / 非 closeout）。implementation `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`；fresh evidence final tip `0e0fcb26233dfbe618129ea05160b835f660f74b`；gateway predecessor `1433d51466e59352cc8859e1c47f176da04f25b0`。

允许动（M5R07 最窄 UI/control/recovery 修包；写域只使用已列 M5/前端/main/runner/report/task 路径；不新增 M1/M6/shared-isolated authority，不改冻结合同正文；真正 ordinary GUI composition 与最终 positive 证据仍受 M1 owner 阻塞；不得自动进入 M1 修正、M6 或 closeout）：

- docs/contracts/（仅新增/修正 M5 UI/DTO 补充合同，以及已发生的 `m5-r07-product-path-correction-v1` 产品路径修正；不改冻结合同正文；不新增 M1/M6/shared-isolated authority；不泛化为任意合同改写）
- prototypes/productized-desktop-shell/src-tauri/src/m5_dto.rs（仅 M5 operation read/control DTO）
- prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs（server-owned operation read/control commands 消费既有 durable funcs；不改 execution kernel）
- prototypes/productized-desktop-shell/src-tauri/src/m5_m3_identity.rs [新增：只消费 M3 RoleSession]
- prototypes/productized-desktop-shell/src-tauri/src/m5_isolated_acceptance.rs（只维护 shared-isolated authority-unavailable negative regression；不得安装 authority 或承载 positive failure/recovery 场景）
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_supervisor.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_summary.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_service.rs（正式授权入口接线；独立锚链 admission；Dispatch readback 同事务与 truth carriers；不泛化为任意编排改写）
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（AppState 最小安装 + command 声明 + `runtime_admission` 模块声明；不泛化为任意 lib 改写）
- prototypes/productized-desktop-shell/src-tauri/src/m5_agent_runtime.rs（已发生：正式 runtime 删除 synthetic fail_cell；`run_conformance_suite` 仅测试）
- prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs（已发生：effect 首写前证明 Dispatch readback substrate。只消费既有 stop/retry/resume 持久函数；不改 execution kernel）
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
- prototypes/productized-desktop-shell/src/lib/tauri.ts、src/lib/m5ProjectSupervisor.ts [新增；消费 M5 control commands]
- prototypes/productized-desktop-shell/src/views/projects/ProjectSupervisorPanel.tsx [新增；接入 stop/retry/resume 与 recovery 控制；不重画页面布局]
- prototypes/productized-desktop-shell/src/views/projects/ProjectWorkspaceShell.tsx（保持默认 `jiaoban` 与三栏布局；左侧主工作面接正式主管并移除 overview 重复实例；不重画页面）
- prototypes/productized-desktop-shell/src/views/ProjectsView.tsx（仅证明默认/切项目保持 `jiaoban` 且该入口已接正式主管；不改 gallery/秘书路由/其它工具）
- prototypes/productized-desktop-shell/src/main.tsx（U01 只校验默认入口；U02 只接 ordinary acceptance DOM driver；shared-isolated 逻辑保持 negative；不改其它验收桥）
- prototypes/productized-desktop-shell/scripts/run-m5-isolated-app-acceptance.mjs [新增；只维护 shared-isolated authority-unavailable negative evidence；不得升级为 positive GUI]
- prototypes/productized-desktop-shell/src-tauri/src/m5_ordinary_control_acceptance.rs [U02 新增；只用 ordinary disposable AppState 与 server-only fixture，不改 production startup]
- prototypes/productized-desktop-shell/scripts/run-m5-ordinary-control-acceptance.mjs [U02 新增；独立 ordinary positive Tauri runner，不复用 shared-isolated profile]
- prototypes/productized-desktop-shell/scripts/m5-x11-screenshot.py [新增]
- tasks/2026-08-16-syn-m5r07-narrow-acceptance-fix-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-gateway-dispatch-readback-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-dispatch-readback-consumption-repair-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-terminal-execution-readback-v1.md [新增]
- tasks/2026-08-17-syn-m5r07-ui-control-recovery-v1.md [U01 新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md（仅独立验收 closeout；当前用户边界禁止改 plan/stage/authorization）
- docs/harness/reports/M5R07-*
- docs/harness/leaves/M5R07-project-ui-isolated-app-and-stage-candidate.md
- docs/harness/done/2026-08/M5R07-* [仅独立验收后 closeout 才归档]

不许动：

- 自行关闭 stage-14 或宣布 M5 完成
- 把 implementation scoped PASS 或 evidence scoped PASS 写成 M5 完成、stage closeout
- 把本叶新发现反向写成 `f51c3f64` terminal scoped FAIL 或 `0e0fcb26` evidence integrity FAIL
- 自动进入 M1 修正、M6 或 closeout
- 新增 M1/M6/shared-isolated authority 或改冻结合同正文
- 把定向测试数写成阶段完成
- 预列或改 `worker_report.rs`，除非后续实现证明必要时再由模型更新 leaf
- 改页面布局或 execution kernel
- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 真实资料/provider/push/reset；伪造窗口截图或 Hook receipt
