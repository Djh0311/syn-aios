# M5R07 项目 UI、隔离 App 与阶段候选

> **2026-08-18 恢复为唯一 current**：M5R00 内容候选 `99a5afc` / tree `08669b0` 已通过独立验收并归档。已获得的全部 M5R07 scoped independent PASS 继续有效，不得反向写成 FAIL。验收标准按 `docs/harness/stages/stage-14.md` 的 2026-08-18 修订执行：界面类证据已取消，组合类六项必须真实通过，真桌面像素证据记为新壳 F5 欠项。

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用冻结 DTO 接现有项目壳，不重写 execution kernel 或页面布局；在隔离 app-data / scratch 上证明完整闭环；形成只含 M5 投影的 candidate series 后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`。不自行关闭 stage-14，不宣布 M5 完成。

状态：`AWAITING_INDEPENDENT_ACCEPTANCE` / `NOT_CLOSEOUT` / `NOT_M5_COMPLETE`。M5R07 修订标准的 final 内容候选是 `7cab37203fe70fe69f696e45fc6a12b314d1fd84`（tree `df6b7432f2a1e5d56eb434e4c5ed979a4f4144b1`），fresh evidence 在 `/home/synadmin/workspace/.syn-gates/evidence/M5R07-7cab372/`；尚未获得本节点的独立验收。`1433d51466e59352cc8859e1c47f176da04f25b0` 是 gateway/Dispatch readback scoped predecessor（已独立 scoped PASS，不是 evidence-binding，也不是 closeout）。implementation exact `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree exact `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`）已获产品 + Git/Harness scoped independent PASS。fresh evidence final tip exact `0e0fcb26233dfbe618129ea05160b835f660f74b` 的旧 evidence 内容/载体已获 Git/Harness scoped independent PASS，非本次修订标准候选、非 closeout。U01a 默认入口 candidate exact `f962038e725ba4e24b2699a46cd1a8d274f13ae6`、U01b 安全有限 control candidate exact `70a15a9c2741b364e0fef38d60ab5d5daad4bea3`、U01c terminal retry 新 lineage candidate exact `23642bbdfe14f9d5d5d83dc4089c3c86503fdfe7` 与 U02 ordinary disposable positive Tauri candidate exact `0952c83d20304cd589a8c641d6d30120d04d91f4` 已获 scoped PASS；均不是 stage closeout。不得把上述 scoped PASS 或本次候选写成 M5 / stage 完成。stage-14 仍开；authorization closed；M6 未激活。不得 close。

来源收据：用户明确把提示内剩余工作做完；M5R06 PASS（`867fd20`）。

本轮做完的标准（2026-08-18 修订，并纳入 M5R00 独立验收欠账）：

1. 普通产品真实启动路径取得并消费 M1 正式项目身份；运行期不得继续用 `project_id(project_root_value)` 的路径派生值填充项目身份（D1），不得依赖测试 fixture 预登记。
2. 在隔离 app-data / scratch 验收启动前，以明确、可复现的产品输入铺设 `m1-ordinary-project-identity-source-v1.json`，并把其产生方式、来源与边界写清（D2）；来源缺失、损坏或不可用仍须 fail-closed，不得静默 fallback、path 派生或自动导入 legacy index。
3. 真实 Tauri 二进制使用普通 `AppState` 装配与 command 注册，不以测试专用 composition 冒充普通产品。
4. 用户拒绝保持零 spawn、零业务 mutation。
5. 强杀并重启后，持久状态、项目身份与绑定保持一致。
6. 端口返回精确对象引用，可回到权威事实；只证明端口语义，不要求旧壳像素或点击证据。
7. 以现成旧壳作为真实非测试客户端，把 `Proposal → AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → Grant → Dispatch → runtime → RuntimeReceipt/ExecutedReport → independent Review → ResultUserDecision` 最小端到端走通一次。
8. 已有后端定向矩阵继续通过；在 disposable checkout 上形成绑定不可变候选 SHA 的新鲜原始 receipts、候选报告与精确写域提交，随后 authorization closed 并停在独立验收节点。

产品：正式 M5 authority → Grant → Dispatch readback → runtime → RecordExecutionAttemptReadback → terminal-gated EXECUTED claim。U01a 保持默认 `jiaoban` 与三栏布局，把左侧主工作面接到唯一正式 `ProjectSupervisorPanel`；U01b 增加 server-owned load/apply control、durable revision/CAS/replay，并只开放可证明的 STOP / RESUME；U01c 对 authoritative FAILED/TIMED_OUT 且可证明无 effect 的终态创建全新 Attempt / Grant / Dispatch / effect lineage，RETRY 本身不执行 runtime，旧链保持 immutable；U02 用普通 disposable AppState 的 server-only fixture 显式登记 M1 exact alias、建立 M3 三角色，并经真实 Vite + Tauri + Xvfb 走默认 `jiaoban`、拒绝零副作用、失败后新 lineage、显式 runtime、重复 runtime 零第二 effect、重启同 binding/project。U02 不等于 legacy production M1 composition。M6 排除。不改 worker_report.rs / 页面布局 / execution kernel；不把定向测试数写成阶段完成。

两栏（不要混读）：

1. ordinary disposable positive Tauri PASS：server fixture 预登记 M1 alias + M3 authority，真实 Vite + Tauri + Xvfb 走默认 `jiaoban` DOM 与两次进程；它不是 legacy production composition，也没有窗口截图。
2. shared isolated 真实 Vite+Tauri/Xvfb 只证明 authority-unavailable fail-closed；`NO_UI_PASS` / `NO_WINDOW_CAPTURE`；scene / resume / second launch `NOT_EXECUTED`。

当前剩余事项（不得反向写成既有 scoped candidate FAIL，也不得 close）：

1. 本 leaf 的八项修订完成标准已在内容候选 `7cab372` 的 detached disposable checkout 上得到直接证据；仍须等待总指导独立验收。独立 PASS 之前不得归档 leaf、关闭 stage-14、宣布 M5 完成或进入 M6 / 壳采纳。
2. 真桌面窗口像素证据仍按 stage-14 修订记为新壳 F5 欠项，不属于 M5R07 当前完成标准；本候选明确 `NO_WINDOW_CAPTURE`。

证据：`7cab372` fresh checkout 上 `cargo check --lib --offline` exit 0，`m5_` 180/180、task-memory 15/15、M1 ordinary source 5/5、typecheck/build 均 PASS；真实 Xvfb launcher exit 0，拒绝零业务副作用，失败后新 lineage，完整 Proposal 到 ResultUserDecision/ProjectFact exact chain 成立，独立 reviewer 成立，binding/project/三类 RoleSession/对象 ID/计数/M1 registry 跨重启一致，duplicate runtime 零第二 effect。原始 logs、phase receipts 与 SQLite 在 `/home/synadmin/workspace/.syn-gates/evidence/M5R07-7cab372/`；这是隔离合成输入上的普通产品组合事实，不冒充真实用户老项目、日常运行、部署、发布或独立验收。前驱 `ab5c46e` 的 179/180 失败日志保留并由 `7cab372` 的单文件测试边界修复转绿。shared-isolated 始终只作 authority-unavailable negative regression。不得自动进入 M6 或 closeout。

载体：本次产品序列 `ab5c46e2265121d92f5b9cd58643180e7a2cd7a8` → final `7cab37203fe70fe69f696e45fc6a12b314d1fd84`（tree `df6b7432f2a1e5d56eb434e4c5ed979a4f4144b1`）；候选报告 `docs/harness/reports/M5R07-2026-08-18-7cab372-candidate.md`。既有 scoped 载体继续为 U01a `f962038e725ba4e24b2699a46cd1a8d274f13ae6`、U01b `70a15a9c2741b364e0fef38d60ab5d5daad4bea3`、U01c `23642bbdfe14f9d5d5d83dc4089c3c86503fdfe7`、U02 `0952c83d20304cd589a8c641d6d30120d04d91f4`；均非 stage closeout。

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
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs（2026-08-18 D1 必要相邻：仅普通 `generate_task_package_file` command 经已安装 M1 read port 解析 canonical ProjectId 并向下传递；不得扩成任意 command 改写）
- prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs（2026-08-18 D1 必要相邻：仅接收服务器已解析的 canonical ProjectId 并用于 task memory packet；不得改工作流状态机语义）
- prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs（同上：只补 `m5_store_path: None`，不改读模型边界）
- prototypes/productized-desktop-shell/src/lib/tauri.ts、src/lib/m5ProjectSupervisor.ts [新增；消费 M5 control commands]
- prototypes/productized-desktop-shell/src/views/projects/ProjectSupervisorPanel.tsx [新增；接入 stop/retry/resume 与 recovery 控制；不重画页面布局]
- prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx（U02 必要相邻：普通项目 fallback 保持原内容，但不得绕过 WorkspaceShell 的正式主管 layout）
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
- tasks/2026-08-18-syn-m5r07-terminal-retry-lineage-v1.md [U01c 新增]
- tasks/2026-08-18-syn-m5r07-ordinary-positive-tauri-v1.md [U02 新增]
- tasks/2026-08-18-syn-m5r07-m1-consumption-and-source-provision-v1.md [D1/D2 窄包新增]
- tasks/2026-08-18-syn-m5r07-ordinary-full-chain-and-reopen-evidence-v1.md [ordinary 完整闭环、精确引用与重启证据窄包新增]
- tasks/2026-08-18-syn-m5r07-control-registry-regression-repair-v1.md [fresh 全量 M5 矩阵发现的 control 静态边界假阳性返修窄包新增]
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
