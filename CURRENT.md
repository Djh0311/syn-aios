# Current Authority

## 当前结论

最新 checkpoint：Root Treatment / Stage R 已进入 R3 SQLite 治理实现期。R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review、R3-P0、R3-A1、R3-A2、R3-A3、R3-A4、R3-A5、R3-A6、R3-A7、R3-A8、R3-A9 Level A 和 R3-A10 Level A 均已完成。R3-A10 记录见 `evidence/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md` 与 `handoffs/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1-result.md`，implementation commit 为 `b18424c38bf0f36f8c9b8ee783a0010598ca9683`。R3-A10 只接受为 `workflow_state_summary` 单一低风险 read model 的 fixture / temp limited read-cut 合同完成；feature flag、verified JSON fallback、blocked matrix、recovery dry-run 和 A10 专用 projection path guard 已验证。Level B 未执行，真实 workbench state root 未读取，真实 production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读路径，不停写 JSON / sidecar，不接受为 production read-cut、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。当前下一步是准备 R3-A11 production observation / export verification 任务包；如要执行 R3-A9 或 R3-A10 Level B，必须另写 execution record、allowed roots、rollback strategy 和 fresh verify。正式计划见 `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`，决策锚点见 `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`。Stage L / L1-L6 在治理冻结期内暂挂为 `deferred_during_root_treatment`：这不等于 Stage L 完成或取消，也不等于取消 K3-B1 / K3-B2；治理收口后再回到 Stage L / Stage K。治理期不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不启动 K3-B1 retry 或 K3-B2，不解冻 backlog 功能。

上一 checkpoint：Stage K / K6 Final Tauri Dogfood Core Path Screenshot Acceptance 已完成，结论为 `accepted_with_deferred_items`，记录见 `evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。本轮在 K6.2 恢复 ScreenCaptureKit window-only 截图链路后，使用真实 Tauri 桌面壳补齐核心入口截图：首页、智能体、运行中工作流、项目、记忆层、知识库、设置、想法箱、Skill 和 Harness。验证通过 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 和 Stage K architecture gate strict。K6 接受为真实 Tauri dogfood 核心入口验收和 Stage K acceptance freeze；Stage K 当前冻结为 `accepted_with_deferred_items`，不接受为严格无缺口完成。K3-B1 retry 申请被安全审查再次拒绝的事实不变：K3-B1 仍未完成，K3-B2 仍不得启动。

上一 checkpoint：Stage K / K5 Running Todo Failure Recovery And Operation Control UX 已完成，结论为 `accepted_non_real_productization_slice`，记录见 `evidence/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md` 与 `handoffs/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1-result.md`。本轮新增前端只读 `operation_control_summary`，把已有运行队列、待确认、失败控制、读回异常、重复阻断、边界阻断、过期清理和 retry / stop / restart / resume readiness 整理成运行中工作流页的普通用户可读层级；复核线最终结论为通过，无 P0/P1/P2。K5 本轮不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`，不接受为真实 retry / stop / restart / resume 已实现、K5 全量完成或 Stage K 完成。

上一 checkpoint：Stage K / K4 Memory Capture, Candidate, And Task Memory Injection UX 已完成，结论为 `accepted_non_real_productization_slice`，记录见 `evidence/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1.md` 与 `handoffs/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1-result.md`。本轮新增前端只读 `memory_workbench_summary`，把已有捕获、观察、候选、正式记忆、任务记忆包、lint 和 run queue 摘要整理成记忆页 / 运行中工作流页的普通用户可读层级；复核线结论为带 P2 通过，唯一 P2 已修补为中文产品口径。K4 本轮不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`，不接受为 K4 全量完成或 Stage K 完成。

再上一 checkpoint：Stage K / architecture calibration v2 and gate 已完成，结论为 `accepted_architecture_gate_added`，记录见 `evidence/2026-06-10-stage-k-architecture-calibration-v2-and-gate-v1.md` 与 `handoffs/2026-06-10-stage-k-architecture-calibration-v2-and-gate-v1-result.md`。本轮新增 `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v2.md` 和只读 `scripts/harness/stage-k-architecture-gate.js`；gate strict 模式通过，0 error / 0 warning。K3-B1 retry 申请被安全审查再次拒绝的事实不变：K3-B1 仍未完成，K3-B2 仍不得启动。

上一 checkpoint：Stage K / K3-B1 retry 申请被安全审查再次拒绝，结论为 `blocked_by_safety_review_again`，记录见 `evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md` 与 `handoffs/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1-result.md`。主管线按 K3-B1.1 路径 B 申请受控真实 retry，但审查拒绝理由为：该非沙箱真实 Codex resume 会发送项目/session 派生 prompt 到外部服务并写入 `/Users/yoyi/.codex`，属于高风险外发；审查明确禁止 workaround / indirect execution / policy circumvention。拒绝后没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。K3-B1.1 产品侧分类修补仍已完成，结论为 `accepted_product_classification_retry_gate`；K3-B1 仍未完成。

上一 checkpoint：Stage J / J6 Final Acceptance And Roadmap Freeze 已完成，Stage J 最终结论冻结为 `accepted_with_deferred_items`，记录见 `tasks/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md`、`evidence/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md` 与 `handoffs/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1-result.md`。J6 接受为 Stage J 目标“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”的当前产品化 checkpoint 完成：J1/J1-B 覆盖自由操控入口和真实 resume 探针，J2/J2-B 覆盖项目工作流 run units 和真实执行点，J3 覆盖记忆捕获与候选化，J4 覆盖运行队列 / 用户确认 / 失败控制，J5 覆盖 UI 信息层级和真实 Tauri 关键截图探针。Stage J 不接受为最终蓝图完整工作台、任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、所有操作自动写 FormalMemory 或完整真实 Tauri UI 自动化验收完成。后续建议进入 post-J：adapter productization、provider / model / credential verification、Tauri UI acceptance hardening、execution operations hardening 和 memory formalization UX。

上一 checkpoint：Stage J / J5 UI Information Hierarchy And Real Tauri Product Acceptance 已完成，结论为 `accepted_with_deferred_items`，记录见 `tasks/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`、`evidence/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md` 与 `handoffs/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1-result.md`。J5 接受为智能体页普通层从“控制中心”收敛为项目 / 对话 / 对话流 / 任务输入的对话工作区，开发者 / 内部边界默认收进详情，左侧栏入口锁定为 `项目 / 智能体 / 想法箱 / 知识库 / 记忆层 / Skill / Harness / 运行中工作流` 且沿用 inkwash glyph，真实 Tauri 关键截图探针完成：`evidence/tauri-verification/2026-06-09-stage-j-j5/01-agent-workbench-tauri-window.png`。复核线确认无 P0/P1；J5 不接受为 Stage J 完成、完整真实 Tauri UI 自动化验收、真实执行新增授权、自动 retry / stop / restart、FormalMemory 自动写入、任意项目无限制自由执行或 planned adapters 真实接入。

上一 checkpoint：Stage J / J4 Run Queue, Failure Control, And User Confirmation Queue 已完成，结论为 `accepted_with_deferred_items`，记录见 `tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`、`evidence/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md` 与 `handoffs/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1-result.md`。J4 新增前端派生 `run_queue_read_model.v1`，把 Product Command、run unit、runtime attention、workflow fact、J3 capture event 和已确认候选组织成运行队列、用户确认队列和失败控制摘要；右侧运行中 / 待办、运行中工作流页和秘书读模型使用同一套摘要。复核线最终确认无 P0/P1/P2。J4 不接受为 Stage J 完成、真实 retry / stop / restart、FormalMemory 自动写入、任意项目无限制自由执行或 planned adapters 真实接入。

上一 accepted checkpoint：Stage J / J2-B Controlled Real Workflow Automation Execution Point 已完成 B1 / B2 并经主管线收口，记录见 `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`、`evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md`、`handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`、`evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`。结论为 `accepted_with_deferred_items`：接受为指定 `mario test` 开发线 run unit read-only 真实 `resume` 探针、指定 Stage J 隔离项目 developer run unit workspace-write 真实 `new_session` 探针完成；执行路径走 J2-B bridge 和统一 `real_execution_product_command` Phase B，readback 均成功且 `result_count=1`，B2 只写 allowed write path，baseline 文件 hash 保持冻结值。J2-B 不接受为完整 C5 / observation / candidate 回收闭环、任意项目无限制自由执行、planned adapters 真实接入、provider/model verification、自动 retry / stop / restart 或 Stage J 完成。

上一 checkpoint：Stage J / J2-A Project Workflow Automation Run Units And Controlled Closed Loop 已完成，记录见 `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`、`evidence/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`、`handoffs/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1-result.md`、`evidence/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1-result.md`。结论为 `accepted_with_deferred_items`：接受为项目工作流自动编排 run units 非真实执行产品集成完成；项目页可从用户目标生成 J2-A 离线编排记录，五类 run units 可追溯，并复用 J1 `codex_control` / 统一 Product Command preview / prepare / Phase A no-op 链路；worker report fixture 可回收到 C5，低风险本项目 process fact 可写 observation，但不生成 FormalMemory。

再上一 checkpoint：Stage J / J1-B Mario Test Codex Control Real Resume Execution Point 已完成，记录见 `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`handoffs/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1-result.md`、`evidence/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1-result.md`。结论为 `accepted_with_deferred_items`：接受为指定 `/Users/yoyi/Documents/mario test` / 指定 `codex-local` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 的一次 J1 Codex Control Plane read-only 真实 `resume` 探针完成；真实执行通过 J1-A `codex_control` source 和统一 `real_execution_product_command` Phase B 触发，readback 成功且 `result_count=1`，项目核心文件 hash 前后一致。保留 P2：底层 continuation 仍有 `h2_phase_b` / `controlled_session_continuation` 历史命名债。J1-B 不接受为 J1 最终完成、任意项目自由执行、真实 new session 成功、J2 自动化工作流编排完成、J3 记忆捕获完成、planned adapters 真实接入、provider/model verification、自动 retry / stop / restart 或 Stage J 完成。

再上一 checkpoint：Stage J / J1-A Codex Control Plane 自由操控入口已完成，记录见 `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`、`evidence/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1-result.md`。结论为 `accepted_with_deferred_items`：接受为工作台智能体页普通用户可见 Codex 控制入口、`source_kind="codex_control"` 接入统一 Product Command preview / prepare / user confirmation / Phase A no-op trace、prompt body 不持久化、J1 临时运行绑定、普通 UI 不暴露裸 `codex exec -C` 命令计划、`new_session` deferred / blocked 边界完成；不接受为真实 new session 已创建、任意项目自由执行、J2 自动化工作流编排完成、J3 记忆捕获完成、planned adapters 真实接入、provider/model verification、自动 retry / stop / restart 或真实 Tauri 全量验收完成。

当前主线已经从中间版本 G5、H-I、PCR10 和 Stage J 收口进入 Stage K 日常可用工作台产品化阶段；K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0 bridge / harness、K3-B1.0 prompt freeze repair、K3-B1.1 Codex state permission / retry gate、Stage K architecture calibration v2/v3 and gate、K4 非真实记忆产品化切片、K5 非真实运行 / 待办 / 失败恢复和操作控制切片、K6 真实 Tauri dogfood 核心入口验收均已完成。Stage K 当前最终结论冻结为 `accepted_with_deferred_items`：接受为日常可用工作台 checkpoint、核心入口真实 Tauri 截图、自由操控 Codex / 自动化工作流 / 记忆层记录的当前产品化闭环；不接受为严格无缺口完成。K3-B1 已执行但失败分类，K3-B1 retry 申请再次被安全审查拒绝；如恢复 K3-B1/K3-B2 真实执行线，只能由用户手动执行 exact command 并回交，或用户明确重新批准风险后主管线再次申请；不得直接进入 K3-B2。中间版本最终结论已在 G5 冻结为 `accepted_with_deferred_items`：C1-C6 已完成阶段 C 的受控自动化工作流闭环；M1-M13 已完成记忆系统权威验收；E1-E7、F1-F5、G1、G2、G3-A、G3-C、G4 和 G5 已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。中间版本不等于最终蓝图完整工作台，也不代表 planned adapters 真实接入、provider credential / model verification、自动重试、完整恢复策略或 G3 全量真实 Tauri 验收完成。

G5 后续 H-I 开发计划已新增：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`。阶段 H 目标是把 `codex-local` 真实自动化工作流产品化，阶段 I 目标是建立中立多 agent / 多模型协作抽象。H0 阶段 H 安全边界和任务包冻结已完成并通过全局主管复核；H1 CodexLocalRunner 架构和数据契约已完成并通过复核；H2 通用真实 resume 产品化已完成到 Phase B `mario test` 真实探针：2026-06-08 对 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 执行真实 `codex exec resume`，发送固定安全 probe prompt，写入 `/Users/yoyi/.codex`，readback 返回 `H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`，记录见 `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`。H2 接受为受控真实 resume 最小产品路径和一次 `mario test` 真实探针完成；不接受为 H3 真实新会话、H5 项目工作流真实派发、planned adapters 真实接入、provider credential / model verification、自动重试或阶段 H 完成。H3-A 和 H3.1 已完成但只接受为 H3 非执行授权冻结 / no-op 产品路径；H3-B 已在 2026-06-08 对隔离 fixture `/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture` 执行一次真实 `codex exec` new-session probe，真实发送 prompt 并写入 `/Users/yoyi/.codex`，但 attempt 失败，readback 为 `readback_failed` / `result_count=null`；失败原因是当时 command plan 缺少 `--skip-git-repo-check`，记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。H3-B 当前只接受为一次真实 fixture run 已执行且失败分类完成、产品路径已修补等待下一次授权；不接受为 H3-B 成功、真实新会话创建成功、H3 产品化或阶段 H 完成。H4 readback / failure / timeout / duplicate guard Level A 非真实产品化已完成并通过全局主管复核，记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`、`handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`、`evidence/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1-result.md`；接受为 readback / failure / timeout / duplicate guard / stale cleanup 统一产品边界完成，不接受为真实 Codex 执行、H4-Level-B 探针或阶段 H 完成。

H3.1 new session request / guard / permission envelope / no-op runner 已完成并通过全局主管复核：`tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`，记录见 `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1-result.md`。H3.1 只接受为 `new_session` 非执行产品路径完成：request / guard / permission envelope / command plan preview / no-op runner / 智能体页只读展示 / 秘书只读解释。H3.1 不接受为真实 `codex exec`、真实 `codex exec resume`、真实 Codex session 创建、prompt 发送、`/Users/yoyi/.codex` 读写、继承 H2 Phase B 执行授权、H3-B 授权 / 执行、H3 产品化完成或阶段 H 完成。

H3-B final approval / real new session fixture run 任务包已执行一次真实 fixture run 并失败分类：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`。本次执行确实触发真实 `codex exec`、发送 prompt、写入 `/Users/yoyi/.codex`，但 Codex CLI 因 non-trusted directory 且缺 `--skip-git-repo-check` 退出，未生成 last message，fixture 业务文件 hash 前后一致，仅新增 `.workbench/h3-b-runs/...` 运行记录。记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。产品代码已补 `new_session` command plan 的 `--skip-git-repo-check`，但未二次真实执行；任何 H3-B retry 必须再次取得执行点授权。

H4 readback / failure / timeout / duplicate guard 产品化任务包已完成 Level A 非真实产品化并通过全局主管复核：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`。记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`、`handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`、`evidence/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1-result.md`。本轮只用 fake / no-op / 单测注入收敛 H2/H3.1/G1/G2 既有能力，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不创建 fixture run，不读写 `/Users/yoyi/.codex`；`readback_unavailable`、`readback_failed`、`readback_timed_out`、`timed_out`、`not_attempted`、`blocked_by_guard`、`duplicate_blocked`、`stale_cancelled` 等 unknown-result 状态保持 `result_count=null`；duplicate blocked 写 attempt / audit / runtime log 且不调用 runner；stale cleanup 只处理工作台自有 active attempt，必须有 expected revision，不 kill Codex、不自动 retry。H4 不依赖 H3-B 已完成；H4-Level-B 真实失败 / 超时探针必须另行取得执行点授权。

H5 project workflow real dispatch integration 已完成 Level A 非真实产品路径集成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`。开发记录见 `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`；主管复核见 `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`。当前只接受为 C4 prepared dispatch、M6 frozen task memory packet、H1/H2/H3 runner request / guard、H4 duplicate / readback unknown-result 边界、G1/G2 runtime / diagnostics、C5 worker report / process fact handoff 和 C6 final review handoff status 的非真实预览 / 校验链路完成；不接受为 H5 已完成或阶段 H 完成。H5-Level-B 授权与 fixture freeze 任务包已完成：`tasks/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`。H5-Level-B1 执行任务包已完成一次 `mario test` read-only 真实 `resume` probe：`tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`；本轮对 `/Users/yoyi/Documents/mario test` 开发线 worker session `019e798a-ac37-7771-b982-e38084fcd22e` 走后端产品 runner，`prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`，readback 返回 `H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08`，四个项目文件 hash 前后一致，记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`；主管复核见 `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`。H5-Level-B2 写入型 probe 已完成一次 `mario test` workspace-write 真实 `resume` probe：`tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`；本轮只写入 `.workbench/h5-b2/real-dispatch-write-probe.md`，`prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`，readback 返回 `H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08`，探针文件 hash 为 `b3eaf99c09a786ab459721872f63bd7fd78f6e8dcd6d34b5e2c761103c5b69ae`，四个核心项目文件 hash 前后一致，记录见 `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1-result.md`。合并型 H5 product command formalization / H5 acceptance checkpoint 已完成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`；记录见 `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`、`handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md`、`evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-supervisor-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-supervisor-review-v1-result.md`。B1/B2 和 H5 checkpoint 不接受为 H5 通用产品化、H3-B 成功、planned adapters 真实接入、provider/model 验证或阶段 H 完成。

H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`，记录见 `evidence/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1-result.md`。H2.8 接受为 H2 Phase B 真实 resume 前权限弹层预览、审计摘要、runtime log preview、readback 边界、duplicate guard 和 readiness 决策面加固；真实执行已在后续 H2 Phase B probe 中单独授权并完成。

阶段 E / E1 已完成：`tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`。本轮只接受为 adapter descriptor 执行边界和模型 / 凭据只读状态底座完成；`WorkbenchSnapshot.agent_adapters[]` 现在可区分 `codex-local` 与 Claude Code / OpenClaw / OpenCode / OpenCode-like planned descriptors。`codex-local` 仍是唯一可用 adapter descriptor，planned adapters 当前不可执行、未配置凭据、模型未验证、没有已实现动作和执行按钮。记录见 `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md` 与 `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`。

阶段 E / E2 已完成：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`。本轮只接受为会话操作边界契约和智能体页只读 / 禁用态可见化完成；`WorkbenchSnapshot.session_operations[]` 现在覆盖发消息、停止、重启、resume、导出、删除、收藏，并按 `codex-local` 与 planned adapters 派生不可执行 / planned / 破坏性阻断状态。E2 不接受为真实会话操作、通用 `codex exec resume`、导出 / 删除 / 收藏持久化、外部 agent 接入、凭据管理或阶段 G 真实 Tauri 验收完成。记录见 `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md` 与 `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md`。

阶段 E / E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。本轮只接受为模型、凭据、provider availability、外发风险和成本风险只读边界完成；`WorkbenchSnapshot.provider_availability[]` 现在从 `agent_adapters[]` 和 `session_operations[]` 派生安全摘要，智能体页在既有 adapter / operation 区域附近显示只读 provider availability，秘书只解释风险和查看建议。E3 不接受为真实 credential store、provider token 读取或验证、外部模型调用、planned adapter 真实接入、真实会话操作或阶段 G 真实 Tauri 验收完成。记录见 `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md` 与 `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md`。

阶段 E / E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`。本轮只接受为会话继续协议、权限预览、prompt summary、readback expectation、failure boundary、audit impact 和 guard 完成；`WorkbenchSnapshot.session_continuation_previews[]` 现在可派生 `send_message` / `resume` 的只读预览，planned adapters 保持 blocked，智能体页只显示“预览不是执行”。E4 不接受为真实发消息、通用 `codex exec resume`、prompt 已发送、Codex 已收到任务、attempt / dispatch / readback 写入、worker 已执行或阶段 G 真实 Tauri 验收完成。记录见 `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md` 与 `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md`。

阶段 E/F/G 细化计划已完成：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。该计划把剩余中间版本收口拆为 E3-E7、F1-F5、G1-G5：阶段 E 继续收模型 / 凭据 / 会话继续 / runtime attention 边界，阶段 F 收项目工作流画布产品化，阶段 G 收真实 Tauri 验收、运行日志、诊断和最终中间版本验收。

E5 已完成 Level A：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`。Level A 接受为代码路径、guard、stub、user confirmation record、attempt / audit ref、readback unavailable 边界和离线验收完成；不接受为真实 `codex exec resume`、真实 prompt 发送、真实 readback、真实会话继续验收、读写 `/Users/yoyi/.codex` 或阶段 G 真实 Tauri 验收完成。记录见 `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md` 与 `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`。

E5 Level B mario test 健康探针已完成：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。本轮在用户明确批准后，对 `/Users/yoyi/Documents/mario test` 的“总指导” session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 执行了一次真实 `codex exec resume`，真实写入 `/Users/yoyi/.codex`，last message 返回 `E5_LEVEL_B_MARIO_TEST_DIRECTOR_RESUME_OK_2026_06_06`，`index.html` / `styles.css` / `game.js` / `README.md` 四个项目文件 hash 前后一致。接受为 E5 Level B 最小真实 `codex-local` resume 健康探针完成；不接受为通用真实 send / resume 产品化、会话中心自由发消息、项目工作流自动派发、四角色工作流重新验证、runtime log / diagnostics、自动重试、planned adapters 真实接入、provider credential / model verification、阶段 G 真实 Tauri 验收或中间版本最终验收完成。记录见 `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md` 与 `handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`。

E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`。本轮新增 `WorkbenchSnapshot.runtime_session_attention[]` 和 `session_run_status_summaries[]` 最小读模型，智能体页 / 运行中 / 通知 / 待办 / 秘书可显示 runtime attention、readback failed / unavailable、guard 阻断和用户下一步查看建议；接受为读模型、摘要 UI 和秘书只读解释完成，不接受为真实执行、自动重试、stop / restart、完整 runtime log 或阶段 G 验收。记录见 `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` 与 `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md`。

E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`，阶段 E 总结论为 `accepted_with_deferred_items`。E7 接受为 E1-E6 总复核、accepted / deferred / blocked freeze 和 E-to-F handoff 完成；E5 Level B 已作为独立健康探针回收。F1-F5 已完成，阶段 F 最终结论为 `accepted_with_deferred_items`。G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B 已回交但未完成，只接受为真实 Tauri 已启动、目标窗口区域截图探针成功并采集 10 / 13 张编号截图。中间版本最终结论冻结为 `accepted_with_deferred_items`。仍不授权产品功能开发以外的真实执行、planned adapters 真实接入、G3 全量真实 Tauri 验收已完成、通用 send / resume 产品化、自动重试或最终蓝图完整完成。G5 记录见 `evidence/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md` 与 `handoffs/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1-result.md`。

GEPA 研究建议已审核：`docs/research/2026-06-05-gepa-workbench-optimization-layer-recommendation-v1.md` 和 `docs/research/2026-06-05-gepa-workbench-deep-reference-research-v2.md` 只作为后置优化层候选保留；v2 已作为 `docs/workbench-system-architecture-v1.md` 的后置优化层拆解依据登记。GEPA 不进入当前 E1 / E2 主线，不进入当前执行 backlog，不拆任务包。阶段 E 只保留架构预留意识；真正运行 GEPA 必须等阶段 G 的运行日志、诊断、eval、成本预算、脱敏和回滚底座完成。GEPA 输出未来也只能是候选和报告，不能绕过控制核心、记忆状态机、权限、审计或用户确认。

Paseo 研究建议已审核：`docs/research/2026-06-05-paseo-workbench-deep-reference-research-v1.md` 只作为多 agent 运行层、adapter、timeline、worktree、schedule、CLI parity、relay 安全和 daemon operations 的外部参考；已作为 `docs/workbench-system-architecture-v1.md` 的外部 agent runtime 参考约束登记。Paseo 不进入当前 E1 / E2 主线，不进入当前执行 backlog，不拆任务包，不授权实现；后续如果融合，必须先按阶段 E / F / G 专题对比设计，再进入设计文档和任务包。

Codex 当前多线程协作能力已登记为 H-I 参考输入：可以学习其主管线派发、开发线执行、回交复核的协作模式，但不能复制为工作台事实模型。后续工作台会接入多种模型和 agent 产品，因此 H 阶段只产品化 `codex-local` 真实执行，I 阶段再抽象中立协作协议；planned adapters 真实接入仍需后续独立任务。

当前已完成阶段 D / M7：记忆管理 UI 最小入口；已完成阶段 D / M8：知识库 / Obsidian-compatible 接口占位和边界；已完成阶段 D / M9：正式记忆生命周期操作；已完成阶段 D / M10：实体和关系治理；已完成阶段 D / M11：维护任务和记忆 lint；已完成阶段 D / M12：成熟模式、跨项目记忆和 M1-M12 集成验收摘要；已完成阶段 D / M12.1：acceptance summary freshness 修补；已完成阶段 D / M13：中间版本记忆系统最终权威验收：

- M7 已完成任务包：`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`。
- M7 只接受为记忆管理 UI 最小入口，重点是正式记忆、候选、来源、版本、审计、冲突和任务包 eligibility 的可理解展示；真实窗口 / 截图验收未完成。
- M8 已完成任务包：`tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`。
- M8 只接受为知识库 / Obsidian-compatible 接口占位和边界，重点是知识库资料、来源引用、候选生成入口和正式记忆反向引用；不接受为 Obsidian 原生同步、vault 自动扫描或正式记忆写入。
- M9 已完成任务包：`tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`。
- M9 只接受为正式记忆生命周期操作，重点是版本化编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉 scope、确认权、审计和记忆中心最小入口；不接受为关系治理、维护任务、成熟模式、完整记忆系统或真实 worker / Codex 执行。
- M10 已完成任务包：`tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`。
- M10 只接受为最小实体 registry、alias / dedupe 候选、关系候选、已确认关系和任务包关系解释的受控闭环；不接受为维护任务、成熟模式、完整记忆系统、图谱推断自动写事实、向量库 / 图数据库 / GraphRAG 或真实 worker / Codex 执行。
- M10 记录见 `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`。
- M10 真实窗口 / 截图验收未完成，不能声称 M10 UI 已完成真实窗口验收。
- M11 已完成任务包：`tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`。
- M11 只接受为维护 run、maintenance finding、维护报告、权限撤回 / 过期 / 缺来源 / 实体漂移 / 私密风险 / 索引状态摘要、任务包 blocking 协作和记忆中心维护摘要完成；不接受为自动修改正式记忆、成熟模式正式化、跨项目记忆、完整记忆系统或真实 worker / Codex 执行。
- M11 记录见 `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md` 与 `handoffs/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1-result.md`。
- M12 已完成任务包：`tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`。
- M12 只接受为 `memory-patterns.v1.json` sidecar、成熟模式候选、跨项目主题报告、用户确认后受控写正式 mature pattern 记忆、任务包召回边界和 M1-M12 gate 摘要完成；不接受为自动技能化、跨项目摘要直接影响 worker、向量库 / 图数据库 / GraphRAG、真实 worker / Codex 执行或最终权威验收完成。
- M12 记录见 `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`。
- M12.1 已完成任务包：`tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`。
- M12.1 只接受为用户确认 mature pattern 正式化后，当次 acceptance summary 使用写入后的 fresh formal store。
- M13 已完成任务包：`tasks/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`。
- M13 结论为 `accepted_with_deferred_items`，记录见 `evidence/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md` 与 `handoffs/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1-result.md`。
- 阶段 E / E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座；E2 已完成会话操作边界契约和智能体页只读 UI。阶段 G 已完成 G1 runtime log 最小底座、G2 diagnostics / health / degraded state、G4 离线回放和 G5 最终冻结；真实 Tauri 全量验收和自动重试仍是 deferred。

C6 已完成但仍不能把 worker 汇报直接写正式事实，不能把 observation / candidate 当正式记忆，不能把 readback 失败伪装成真实 0 条读回，不能在未获明确授权时执行真实 worker、`codex exec` 或 `codex exec resume`，也不能读写 `/Users/yoyi/.codex`。

依据：

- `docs/middleware-version-development-plan-v1.md` 第 0 节明确中间版本采用方案授权制。
- `docs/plans/middleware-version-stage-plan-v1.md` 明确阶段 C 是自动化工作流产品化闭环，阶段 D 从 M7 开始继续完整记忆系统。
- `tasks/README.md` 当前任务队列已记录 C1-C6、M7-M13 完成，并指向阶段 E / 阶段 G 后续。

## 当前目标

当前目标已从中间版本 G5、H-I 后续阶段和 PCR10 checkpoint 转为 Stage J 产品化开发，并已完成 J0-J6 收口；Stage J 最终结论为 `accepted_with_deferred_items`。新的真实 `codex exec` / `codex exec resume` 仍必须另行执行点授权。

中间版本 G5 Final Authoritative Acceptance And Deferred Freeze 已完成：E5 Level B mario test 最小真实 resume 健康探针已完成，阶段 E / E7 已冻结为 `accepted_with_deferred_items`，F1-F5 已完成，阶段 F 最终结论为 `accepted_with_deferred_items`，G1、G2、G3-A、G3-C、G4 和 G5 已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。中间版本最终结论为 `accepted_with_deferred_items`：

1. C1-C6 已完成方案授权、用户确认、全局边界复核、项目主管拆任务、prepared dispatch、worker 结构化汇报、过程事实确认、最终结果复核、用户结果决定和阶段 C 验收摘要。
2. 阶段 C 接受为完成，但完整真实 worker / Codex 执行、自动重试、运行日志、运维诊断和真实 Tauri 全面验收仍是后置项。
3. 阶段 D 已完成 M7-M13，记忆系统最终权威验收结论为 `accepted_with_deferred_items`。
4. 阶段 E / E1、E2、E3、E4、E5 Level A、E5 Level B 健康探针、E6 和 E7 已完成；阶段 E 总复核结论仍为 `accepted_with_deferred_items`。不能把 E2 operation boundary、E3 provider availability、E4 preview、E5 Level A stub、E5 Level B 单 session 健康探针、E6 runtime attention 或 E7 接受结论解释成通用真实会话操作、真实 provider 接入、真实模型验证、通用 send / resume 产品化、自动重试完成或阶段 E 无遗留项。
5. E5 Level B 任务包为 `tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`，状态为已完成；证据显示真实 `codex exec resume` exit code `0`、last message 返回固定标记，且 `/Users/yoyi/Documents/mario test` 四个项目文件 hash 前后一致。
6. 阶段 F 已确定为 F1-F5 项目工作流画布产品化链路；F1 已完成，记录见 `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md` 与 `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`；F2 已完成，记录见 `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md` 与 `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`；F3 已完成，记录见 `evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md` 与 `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`；F4 已完成，记录见 `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md` 与 `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`；F5 已完成，记录见 `evidence/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md` 与 `handoffs/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1-result.md`，阶段 F 结论为 `accepted_with_deferred_items`。不能把 F1/F2/F3/F4/F5 任务包解释为项目画布和实验画布已经合一、画布编辑器、真实执行、diagnostics 完成、阶段 G 完成或真实 Tauri 验收完成。
7. 阶段 G 已确定为 G1-G5 真实 Tauri、运行日志、诊断和最终验收链路；G1、G2、G3-A、G3-C、G4 和 G5 已完成，G3-B 已回交但未完成。G1 记录见 `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md` 与 `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`。G2 记录见 `evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md` 与 `handoffs/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1-result.md`。G3-B 记录见 `evidence/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1.md` 与 `handoffs/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1-result.md`。G3-C 记录见 `evidence/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1.md` 与 `handoffs/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1-result.md`。G4 记录见 `evidence/2026-06-07-stage-g-g4-middle-version-end-to-end-acceptance-replay-v1.md` 与 `handoffs/2026-06-07-stage-g-g4-middle-version-end-to-end-acceptance-replay-v1-result.md`。G5 记录见 `evidence/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md` 与 `handoffs/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1-result.md`。G3-B 只接受为 10 / 13 真实 Tauri 部分截图证据；G3-C 只接受为截图证据回收和缺口矩阵完成；G4 只接受为离线端到端回放完成；G5 只接受为最终权威验收和 deferred freeze。中间版本不接受为最终蓝图完整工作台、G3 全量真实 Tauri 验收、通用真实 send / resume 产品化、planned adapters 真实接入、provider credential / model verification、自动重试、自动修复、GraphRAG / 向量库 / 图数据库 / Obsidian 原生同步或自动技能化完成。
8. GEPA 只作为后置优化层研究候选保留；不能把 GEPA-0 提前塞进当前 E/F/G，除非后续全局主管重新审核并单独批准。
9. Paseo 只作为外部运行层研究参考保留；不能把 Paseo 的 agent 自治编排、MCP 直接管 worker 或 schedule/loop 直接并入当前阶段，除非后续全局主管重新审核并单独批准。
10. H-I 计划已登记为 G5 后续开发计划；H 阶段优先产品化 `codex-local` 通用真实 send / resume 和项目工作流真实派发，I 阶段再抽象多 agent / 多模型协作协议。Codex 多线程协作只作为参考模式，不能照搬成产品模型。

## 当前状态

已完成：

- 单会话 transcript 读取 v1。
- 无业务 `codex exec` 新建测试会话并读回。
- 无业务 `codex exec resume` 向绑定测试会话派发第二轮 prompt 并读回。
- Agent 会话中心只读 UI。
- 项目内 Agent 会话入口。
- 工作流最小状态流转。
- 工作流节点绑定已有 Codex 会话。
- 桌面工作流节点派发 Codex 指令 v1 代码路径。
- 工作流运行模型 dry-run。
- 真实工作流节点 safe probe 派发闭环一次。
- 派发结果 UI 读回与总指导 review 记录入口。
- 总指导 `accepted` review 已写入真实 workflow state。
- Codex 角色编排离线入口 v1：总指导派发块解析、离线确认、角色桩结果、回传总指导摘要。
- Codex 角色编排离线状态账本 v1：prepared dispatch、role handoff、director review 三步已能写入工作台自己的 workflow state。
- Codex 角色编排离线账本复核修复 v1：回传后仍能总指导回收，重复 prepared 离线派发会被拒绝。
- 四角色工作流机器 v1：已新增 `run_workflow_machine`，可按总指导、开发线、验证线、回收线、总指导结论的顺序串行调用绑定 Codex 会话，并将 run 写入 `workflow_machine_runs[]`；stub 测试已验证能收口到 accepted。
- 四角色工作流机器 mario demo 真实闭环 v1：已在 `/Users/yoyi/Documents/mario test` 上跑出真实马里奥 demo，并将 v4 work item 收口为 `accepted`。本轮定位并修复 runner 未关闭 stdin 导致 `codex exec resume` 卡住的问题；首轮总指导计划使用本地 fallback，开发线、验证线、回收线和最终总指导均走真实 Codex 会话。
- 四角色工作流机器真实总指导自然闭环 v1：v7 已用真实总指导、开发线、验证线、回收线、最终总指导一轮自然收口为 `accepted`，首轮不再使用本地 fallback；本轮修复了最终接受标记被 240 字摘要截断的问题和离线测试 last-message 并发碰撞问题。
- uiwork 水墨工作台界面替换 v1：已用 `/Users/yoyi/Documents/uiwork` 下 ui总指导、ui开发线、ui验证线、ui回收线四个 Codex 会话，通过工作流机器把 `inkwash-full.html` 水墨 UI 接入真实工作台；第一轮 `needs_changes`，第二轮有效 run 收口为 `accepted`。本轮同时修复工作流机器目标硬编码和 `execution_root` 支持。依据见 `evidence/2026-05-31-uiwork-inkwash-workbench-replacement-workflow-v1.md` 与 `handoffs/2026-05-31-uiwork-inkwash-workbench-replacement-workflow-v1-result.md`。
- workflow task package design v1 Task 7-12：已完成保守闭环读模型和只读 UI 草案，包含工作流账本、子智能体汇报、审查结果、异常通知、状态机、项目主管完成闸门、接口边界、端到端验收场景展示；测试和 Chrome headless 只读 mock 截图已通过。不接受为真实业务自动编排完成，也不接受为真实 Tauri 窗口完整验收。依据见 `evidence/2026-06-01-workflow-task-package-design-v1-execution.md` 与 `handoffs/2026-06-01-workflow-task-package-design-v1-execution-result.md`。
- 会话中心底座硬化 v1 已完成：后端新增 Rust 原生 `codex_transcript.rs` JSONL parser，会话中心 Tauri transcript command 改为 sqlite 目录优先、index 仅兼容回退；验收复核后补掉“index 读取失败会挡住 sqlite 会话”的缺口；`reveal_indexed_rollout` 允许 sqlite 中合法 rollout 路径；前端抽出 `conversationTurns` 纯函数，新增搜索、状态过滤、错误分类、代码块容器和复制按钮，清理会话中心孤儿 CSS；验收复核后补掉 `event_msg` 用户消息 + `response_item` Agent 回复混合流只显示半边对话的问题。`npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --lib`、`rustfmt --check src/codex_transcript.rs src/codex_db.rs` 均通过。未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未改 workflow state JSON，未迁移数据库，未做真实 Tauri 窗口验收。依据见 `evidence/2026-06-03-session-center-foundation-hardening-v1.md` 与 `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`。
- 工作流派发 readback native parser 迁移 v1 已完成：`dispatch_readback_stats` / execute 成功 readback / 手动 readback / workflow machine 相关路径不再构造或调用 `transcript_reader.py`，改用 Rust 原生 transcript parser + sqlite/index catalog helper；复核后已删除桌面壳 deprecated Python reader 兼容函数，`src-tauri/src/lib.rs` / `commands.rs` 不再包含 `Command::new("python3")`。`prototypes/index-kernel/transcript_reader.py` 文件本身未删除。依据见 `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 与 `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`。
- 记忆层 M1 正式记忆受控存储和审计骨架已完成：新增 `formal_memory_store.rs`，sidecar 路径为 `<workflow_state_dir>/formal-memories.v1.json`；显式 `create_formal_memory_record` 会同步写入 `MemoryRecord`、第一版 `MemoryVersion` 和 `MemoryAuditEvent`；`candidate_confirmed` 不会自动创建正式记忆；项目页和记忆入口只读展示正式记忆骨架摘要。验证通过 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --lib`、`rustfmt --check src/formal_memory_store.rs`。不接受为候选采纳、任务包召回、任务包注入、完整正式记忆生命周期、Obsidian / 知识库、向量库 / 图数据库或中间版本记忆层完成。依据见 `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md` 与 `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`。
- 记忆层 M1.1 正式记忆上下文绑定 guard 已完成：正式记忆创建必须通过 `project_root` 推导出的 `project_id` / `workflow_id` / scope 绑定校验，并要求 workflow state `projects[]` 包含项目。依据见 `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md` 与 `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`。
- 记忆层 M2 候选到正式记忆受控采纳已完成：新增 `adopt_memory_candidate_to_formal_memory`，低风险本项目 `candidate_confirmed` 可由 `project_director` 受控采纳为正式 `MemoryRecord` / 第一版 `MemoryVersion` / `memory_candidate_adopted_to_formal_memory` 审计；候选 sidecar 保留 adoption 回链。必须用户确认的候选不能被项目主管、秘书、worker 或 system 绕过；普通 `candidate_confirmed` 仍不等于正式记忆。依据见 `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md` 与 `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`。
- 记忆层 M3 ObservationStore 和工作流观察入口已完成：新增独立 `observations.v1.json` sidecar、`ObservationRecord` / `ObservationAuditRef`、`load_observation_store` / `create_observation` / `create_memory_candidate_from_observation` 命令；worker 汇报等明确工作流事件可记录为 `recorded` observation，项目主管可从 recorded observation 生成 `candidate_needs_review` 记忆候选，observation 回链 `candidate_key` 并进入 `candidate_created`。ignored / quarantined / duplicate observation 不会生成候选；普通聊天自动捕获会被拒绝。依据见 `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md` 与 `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`。
- 记忆层 M4 任务记忆包生成器和预览已完成：新增 `TaskMemoryPacketBuilder` / `preview_task_memory_packet`，可按状态、权限、冲突、过期、模型外发、token 和相关性生成 included / excluded / review materials 预览；candidate / observation 只作为待审查材料，不能进入正式 included list。依据见 `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md` 与 `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`。
- 工作流 C1 方案授权和受控自动推进基础已完成：新增 `plan-authorizations.v1.json` sidecar、方案授权对象读写骨架、授权范围 guard、自动推进 inspect / prepare 前置检查和项目工作流侧栏只读“方案授权摘要”。不接受为阶段 C 完成、自动化工作流产品化闭环完成、真实 worker 已执行或真实 Codex 已执行。依据见 `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md` 与 `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`。
- 工作流 C2 项目咨询方案草案和用户确认入口已完成：新增 `project-proposals.v1.json` sidecar、方案草案创建 / Markdown 渲染 / 用户决定命令、用户确认到 C1 `PlanAuthorization` 的受控联动，以及项目工作流侧栏“项目咨询方案草案”卡片。用户确认后授权停在 `pending_global_boundary_review`，C1 guard 仍为 `needs_review`，不会自动派发。不接受为全局主管复核完成、授权 active、真实 worker 已执行、真实 Codex 已执行、真实项目咨询 agent 已接入或自动化工作流产品化闭环完成。依据见 `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md` 与 `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`。
- 工作流 C3 全局边界复核和授权生效已完成：新增严格 `record_global_boundary_review` wrapper，复用 `plan-authorizations.v1.json`，校验 C2 proposal 与 C1 authorization 回链、用户确认、checklist 和 findings；approved 后 authorization 进入 `active`，needs_changes / blocked 后进入 `paused`，C1 guard 对匹配输入返回 `authorized` 且越界仍阻断。项目工作流侧栏新增“全局边界复核”卡片和确认弹层。接受为授权有效，不接受为项目主管拆任务、真实 worker 已执行、真实 Codex 已执行、最终结果复核完成或自动化工作流产品化闭环完成。依据见 `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md` 与 `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`。
- 工作流 C4 项目主管拆任务和授权范围内 prepared auto dispatch 已完成：新增 `preview_project_director_task_plan` / `prepare_authorized_auto_dispatch`，基于 C3 active authorization、C2 proposal 回链和 C1 guard 生成 deterministic planned tasks、任务包 artifact、M6 记忆快照和 `state: "prepared"` dispatch；缺 binding 返回 `needs_binding`，不创建可执行 dispatch；重复 prepare 幂等。项目工作流侧栏新增“项目主管拆任务”摘要和确认弹层。接受为项目主管拆任务、授权 guard 校验、任务包 / 记忆包准备和 prepared dispatch 落账；不接受为真实 worker 已执行、真实 Codex 已执行、worker 结构化汇报、项目主管过程事实确认、最终结果复核或自动化工作流产品化闭环完成。依据见 `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md` 与 `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`。
- 工作流 C5 worker 结构化汇报、项目主管过程事实确认和失败 / readback / 权限最小可见化已完成：新增 `record_worker_structured_report` 和 `record_project_director_process_fact_decision`；worker report 只写 workflow state 既有 `audit_events[]`，不会自动写 observation / candidate / formal memory；项目主管确认低风险本项目过程事实后写 `observations.v1.json` 的 `process_fact` observation，仍不是正式记忆；要求返工 / 阻断并上报只写 review / audit，不写 observation。项目工作流侧栏新增“C5 worker 汇报 / 过程事实”摘要、readback / permission / failure 人话状态和确认弹层。不接受为真实 worker 已执行、真实 Codex 已执行、最终结果复核、用户接受结果、完整自动重试系统、正式事实写入或正式记忆写入。依据见 `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md` 与 `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`。
- 工作流 C6 全局主管最终结果复核、用户结果查看和阶段 C 验收已完成：新增 `record_global_final_result_review`、`record_user_result_decision` 和 `generate_stage_c_acceptance_summary`；最终复核 / 用户决定只写既有 `reviews[]` / `audit_events[]`，阶段 C 摘要只写既有 `artifacts[]` / `audit_events[]`；项目工作流侧栏新增“C6 结果 / 阶段验收”摘要和确认弹层。接受为阶段 C 的 C1-C6 受控闭环完成；不接受为中间版本整体完成、完整记忆系统完成、真实 worker 已执行、真实 Codex 已执行、正式事实写入或正式记忆写入。依据见 `evidence/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md` 与 `handoffs/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1-result.md`。

未完成：

- 真实业务自动编排。
- 总指导自动计划和连续调度的产品化 UI。
- resume 长任务、并发、失败重试、权限确认队列、超时和取消的真实多角色闭环。
- readback / 失败完整运维化：C5 已完成项目工作流侧栏的最小可见化和测试覆盖，G1/G2 已完成运行日志和诊断最小底座；完整错误码、自动重试和恢复策略仍是 deferred。
- 真实 Tauri 验收线已经通过 `final-skeleton-04` 建立最小截图证据，G3-B 也已采集 10 / 13 张真实 Tauri 编号截图，但还不是完整自动化 UI 验收；剩余高风险截图项仍未覆盖。
- workflow task package design v1 的真实 Tauri 窗口完整验收仍未补齐；此前只有 Chrome headless + 只读 mock 截图。
- uiwork 历史中间 run 已清账：`workflow-machine-run:workflow-users-yoyi-documents-uiwork-default:workflow-users-yoyi-documents-uiwork-default-inkwash-ui-replacement-v1:1780224885195` 已从 `running` 标记为 `cancelled`，并记录 `superseded_by_run_id` 指向最终 accepted run。依据见 `evidence/2026-05-31-uiwork-stale-running-run-cleanup-v1.md` 与 `handoffs/2026-05-31-uiwork-stale-running-run-cleanup-v1-result.md`。

下一阶段方向：

- 中间版本必须围绕自动化工作流和记忆层落地，不以 SQLite 是否替换现有 JSON / sidecar 作为目标。
- 自动化工作流的目标是方案授权后自动推进：用户确认方案，项目主管管理过程并在授权范围内自动派 worker，全局主管只复核方案边界和最终结果，用户看结果和必须拍板的事项。
- 记忆层的目标是落实 `docs/memory-layer-design-v1.md`：观察、候选、正式记忆、来源、版本、权限、冲突、审计、召回和任务包注入必须形成可验证闭环。
- 秘书在中间版本里只整理、提醒、解释和收纳想法，不确认 worker 汇报、不判断项目过程事实、不当工作成果裁判。
- `docs/middleware-version-development-plan-v1.md` 已落实“确认后的中间版本权威口径”：第 0 节是当前解释权最高的中间版本方案；下方原始阶段草案只能作为历史素材，执行必须按方案授权制、角色边界、记忆层完成标准和后续任务包解释。
- `docs/workbench-frontend-display-boundary-v1.md` 已新增为当前前端显示边界权威文档：一级入口确认为项目 / 智能体 / 画布 / 记忆 / 知识库 / 设置；右侧入口确认为秘书 / 通知 / 待办 / 运行中 / 管理；秘书是独立半身悬浮标，不是底部常驻聊天框；记忆是能看懂的记忆中心，知识库是 Obsidian-compatible 的资料和笔记空间；审计和日志进入管理；开发者模式默认关闭。该文档包含最终形态、中间版本范围、后端依赖和后置能力拆解，不能全部解释为中间版本当前任务。
- `docs/plans/task-package-ui-display-boundary-rule-v1.md` 已新增为任务包 UI 显示边界硬规则。后续任何可能改前端、读模型展示、UI 文案、导航入口、右侧入口、项目页、画布、记忆、知识库、智能体、秘书或管理入口的任务包，都必须包含“UI 显示边界确认”固定章节，不能只把 UI 文档放进必读清单。
- M1.1 正式记忆上下文绑定 guard、M2 候选到正式记忆受控采纳、M3 ObservationStore / 工作流观察入口、M4 任务记忆包生成器 / 预览、M5 冲突与记忆 lint 最小阻断、M6 工作流任务包注入、M7 记忆管理 UI 最小入口、M8 知识库 / Obsidian-compatible 接口占位、M9 正式记忆生命周期操作和 M10 实体 / 关系治理均已完成，记录见 `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`、`handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`、`evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`、`handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`、`evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`、`handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`、`evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`、`handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`、`evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`、`handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`、`evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`、`handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`、`evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`、`handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`、`evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`、`handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`、`evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`、`handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`、`evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`。M3/M4/M5/M6/M7/M8/M9/M10 不能把 observation、candidate、knowledge hit、LLM summary、lifecycle 操作或实体 / 关系治理误当完整记忆系统；M6 只接受为第一条真实记忆闭环完成，M7 只接受为记忆管理 UI 最小入口完成，M8 只接受为知识库边界占位完成，M9 只接受为正式记忆生命周期操作完成，M10 只接受为实体 / 关系治理完成；都不接受为中间版本记忆层完成或真实 worker 已执行。M7/M9/M10 真实窗口 / 截图验收仍未完成。
- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md` 对应的工作台侧 recovery 实现已撤回，且该任务包目标错误，不能继续派发。用户原始问题是 Codex 原生 app 自己的旧对话列表消失，不是工作台智能体页看不到旧会话。真正待执行任务是 `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`；该任务必须以 Codex 原生 app 会话列表恢复为验收，写 `/Users/yoyi/.codex` / Codex sqlite / session index 前必须另行取得用户文件级确认、备份和回滚方案。
- 已新增给下一任全局主管的交接文档：`handoffs/2026-06-03-global-director-handoff-v1.md`。接手新对话时应先读该文档，避免把历史 evidence、已撤回任务或目标错误任务当成当前权威。

## 当前技术栈

- 桌面壳：Tauri 2
- 本地核心：Rust
- 前端：React + TypeScript + Vite
- 关系/画布方向：React Flow
- v0 工作流事实层：JSON 文件
- 长期本地事实库方向：SQLite + FTS
- 向量库：后置

依据：

- `decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `decisions/2026-05-28-workflow-state-storage-v0.md`

## 当前权威文档

- 产品线入口：`README.md`
- 当前权威：`CURRENT.md`
- 权威索引：`AUTHORITY.md`
- 阶段计划：`STAGE_PLAN.md`
- 中间版本方案：`docs/middleware-version-development-plan-v1.md`
- 中间版本整体阶段计划：`docs/plans/middleware-version-stage-plan-v1.md`
- 阶段 H / H0 安全边界任务包：`tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- 阶段 H / H1 CodexLocalRunner 契约任务包：`tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- 阶段 H / H2 通用真实 resume 产品化任务包：`tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- 阶段 H / H2.1 真实 resume 授权矩阵任务包：`tasks/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- 阶段 H / H2.2 授权准备只读 UI 任务包：`tasks/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- 阶段 H / H2.3 request builder 和 CodexLocal guard bridge 任务包：`tasks/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- 阶段 H / H2.4 真实执行授权包和 fixture freeze 任务包：`tasks/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md`
- 阶段 H / H2.5 real resume runner execution path and authorized fixture run 任务包：`tasks/2026-06-07-stage-h-h2-5-real-resume-runner-execution-path-and-authorized-fixture-run-v1.md`
- 阶段 H / H2.8 真实执行权限弹层、审计摘要和 readiness 决策面任务包：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`
- 阶段 H / H3-B 真实新会话最终批准和 fixture run 任务包：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- 阶段 H / H4 readback、失败、超时和重复派发保护产品化任务包：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`
- 前端显示边界：`docs/workbench-frontend-display-boundary-v1.md`
- 记忆层实施切片：`docs/plans/memory-layer-implementation-slice-v1.md`
- 任务队列：`tasks/README.md`
- 开发线：`DEV_LINES.md`
- 原型工作线：`PROTOTYPE_WORK_LINES.md`
- 核心原则：`principles.md`
- 待办和想法收纳：`backlog.md`
- 归档索引：`archive/README.md`

## 当前权威决策

- `decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `decisions/2026-05-28-extensible-first-development-rule.md`
- `decisions/2026-05-28-codex-workflow-min-model.md`
- `decisions/2026-05-28-workflow-state-storage-v0.md`
- `decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `decisions/2026-05-30-codex-session-display-name-rule.md`
- `decisions/2026-05-30-workflow-first-before-workbench-iteration.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

## 当前展示规则

Codex 原本会话名保留为基础展示名。

在全局会话列表中，原 Codex 会话名做主标题。

在项目工作区或工作流节点中，角色 / 工作线做主标题，原 Codex 会话名做副标题，并叠加工作流状态。

依据：

- `decisions/2026-05-30-codex-session-display-name-rule.md`
- `evidence/2026-05-29-reference-workflow-session-node-research-v1.md`

## 当前任务

- `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 已完成，依据见 `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 与 `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`。本轮把工作流派发 readback stats 从旧 `transcript_reader.py` / Python 子进程迁到 Rust 原生 transcript parser；只迁 readback stats 主路径，未执行真实 Codex，未改 workflow state JSON 结构，未改工作流状态机，未改变总指导回收策略。
- `tasks/2026-06-03-session-center-foundation-hardening-v1.md` 已完成，依据见 `evidence/2026-06-03-session-center-foundation-hardening-v1.md` 与 `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`。接受为会话中心底座硬化：sqlite 成为会话目录主权威，`index.json` 只做缓存 / 兼容 / 辅助；会话中心 transcript 主路径迁到 Rust 原生 JSONL parser；主对话默认只显示用户消息和 Agent 回复；搜索、过滤、用户控制收纳、固定框架内滚动、错误分类和孤儿样式清理已进入同一批次。不接受为完整 Codex 控制器、发消息 / stop / restart / resume、删除 / 导出 / 收藏 / 分享、实时运行进度、多会话对比、会话 lineage、Claude / OpenClaw / OpenCode 接入或真实 Tauri 窗口验收完成。
- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 已完成，依据见 `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 与 `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`。本轮新增后端 typed `AdapterCapability` / `AgentAdapterDescriptor`，`WorkbenchSnapshot.agent_adapters[]` 现在输出 `codex-local` 后端能力声明；Agent 页优先展示后端 descriptor，秘书只读模型优先读取后端 adapter warnings，前端 `adapterCapabilities.ts` 仅保留 fallback。不接受为 Claude Code / OpenClaw / OpenCode 已接入，不接受为能力已重新真实验证；未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未改 `workflow-state.v0.json` 结构，未迁移数据库，未写正式事实或正式记忆。
- `final-skeleton-10-blackboard-candidate-schema-design-v1` 已完成补充版 schema：用户已确认 schema 方向、独立 sidecar JSON、候选层确认语义，以及 sidecar 文件路径和作用域、原子写入、备份、并发冲突处理、candidate_key 稳定生成规则、记录版本字段、rejected / discarded 后候选再次出现规则。该“暂不自动授权进入 Skeleton-11”的旧闸门已被后续用户授权和 `final-skeleton-11 + final-skeleton-14` 候选治理最小闭环 superseded；当前仍有效的是候选层边界，不是旧暂停状态。
- `final-skeleton-12-adapter-capability-registry-v1` 已完成适配器能力声明骨架，依据见 `evidence/2026-06-03-final-skeleton-12-adapter-capability-registry-v1.md` 与 `handoffs/2026-06-03-final-skeleton-12-adapter-capability-registry-v1-result.md`。本轮新增前端只读 `AgentAdapterDescriptor` / `AdapterCapability` 读模型，Codex 路径声明 `codex-local` 已有能力，智能体页新增只读能力面板；未接 Claude / OpenClaw / OpenCode，未改真实 Codex 执行语义，未执行真实 Codex，未显示未实现能力按钮，未改 workflow state JSON，未读写 `/Users/yoyi/.codex`，未实现黑板候选写入。
- `final-skeleton-13-memory-governance-schema-design-v1` 已完成记忆治理 schema 设计，依据见 `docs/plans/2026-06-01-memory-governance-schema-v1.md`、`evidence/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1.md` 与 `handoffs/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1-result.md`。本轮只定义 `memory_governance.v1`、`MemoryCandidate`、`MemoryRecord`、`MemoryScope`、`MemorySourceRef`、`MemoryLifecycleStatus`、`MemoryConflict`、`MemoryAuditRef`、用户偏好确认规则和 Skeleton-14 草案；未改产品代码，未实现记忆候选 sidecar，未写正式长期记忆，未接向量库 / 图数据库 / Obsidian，未读写 `/Users/yoyi/.codex`。用户已允许进入 Skeleton-14，但该授权只覆盖候选生命周期最小实现。
- `final-skeleton-11` 和 `final-skeleton-14` 候选治理最小闭环批次已完成，依据见 `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md` 与 `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`。本轮新增黑板候选 sidecar `blackboard-candidates.v1.json` 和记忆候选 sidecar `memory-candidates.v1.json` 的最小后端 store / Tauri 命令 / 前端类型 / 读模型 / 项目页候选治理条；候选确认只表示候选层处理，不写正式事实、不写正式 `MemoryRecord`、不改 workflow state JSON 结构、不迁移数据库、不执行真实 Codex、不读写 `/Users/yoyi/.codex`。
- `final-skeleton-15-secretary-core-readonly-model-v1` 已完成，依据见 `evidence/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md` 与 `handoffs/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1-result.md`。本轮新增前端纯读 `SecretaryContext` / `SecretarySuggestion` / `SecretaryRiskSignal` / `SecretaryMemoryCandidate` / `SecretaryActionProposal` 读模型和可复用 `SecretaryBrief` 摘要组件；从 snapshot、workflow state、黑板候选 sidecar、记忆候选 sidecar 和 adapter descriptor 派生风险、建议、候选和下一步查看提案；未做秘书聊天、未直接改事实、未直接派发任务、未批准权限、未写正式记忆、未改 workflow state JSON、未执行真实 Codex、未读写 `/Users/yoyi/.codex`。
- `final-skeleton-16-project-workflow-surface-convergence-v1` 已完成，依据见 `evidence/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md` 与 `handoffs/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1-result.md`。本轮把秘书摘要从通知、待办、审计、项目运行右侧详情中移出，新增右侧“秘书只读摘要”独立入口；项目工作流页保留项目画布主区域，把候选治理降为项目画布侧栏详情卡；未执行真实 Codex，未运行 MCP canvas run，未改 workflow state JSON，未写正式事实或正式记忆，未读写 `/Users/yoyi/.codex`。
- `final-skeleton-16` 验收后残余 UI 风险已清理，依据见 `evidence/2026-06-03-final-skeleton-16-dev-sample-card-cleanup-v1.md` 与 `handoffs/2026-06-03-final-skeleton-16-dev-sample-card-cleanup-v1-result.md`。项目工作流页不再渲染“组件状态样例 / 后续画布开发基准”开发样例卡；`projectCanvasStateExamples()` 仍保留为内部读模型测试基准，不进入可见 UI。
- `workflow-task-package-design-v1` 的 Task 0-12 已按计划分两段执行完：Task 0-6 已有用户测试 handoff，Task 7-12 已补读模型、只读 UI、测试、截图 evidence 和当前权威更新。注意：本轮有一次读取 `/Users/yoyi/.codex/plugins/.../control-in-app-browser/SKILL.md` 的偏差记录，不能声称完全未读 `/Users/yoyi/.codex`。
- `codex-role-orchestration-offline-state-ledger-v1` 已完成并已做复核修复：工作台 UI 已新增总指导 / 开发线 / 验证线 / 回收线离线编排入口，并接入工作台自己的状态账本；固定字段派发块解析、缺字段阻止、prepared dispatch、role handoff、director review 已覆盖测试；回传后仍能回收，重复 prepared 派发会被拒绝。
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md` 已回收为接受。
- 本轮接受为 completed 成功路径下实际派发节点不残留 `running` 的真实复测；不接受为复杂业务自动编排完成。
- `workbench-architecture-implementation-plan-v1` 的 Task A 架构只读审计已完成，依据见 `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md` 与 `handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`。
- 已补两个 Task B 前置决策：`decisions/2026-06-01-project-workflow-canvas-authority-v1.md` 和 `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`。
- `workbench-architecture-implementation-plan-v1` 的 Task B 保守切片已完成，依据见 `evidence/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1.md` 与 `handoffs/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1-result.md`。本轮只拆出后端类型、Tauri command 包装和前端 editable canvas 纯类型；workflow 读模型和 WorkbenchSnapshot 组装未拆，因为依赖私有 helper 太多，强拆会扩大风险。
- `workbench-architecture-implementation-plan-v1` 的 Task C 项目工作流画布权威收敛已完成一个保守切片，依据见 `evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md` 与 `handoffs/2026-06-01-project-workflow-canvas-authority-convergence-c-v1-result.md`。本轮只做前端入口和文案收敛：项目页是项目工作流主入口，独立 `CanvasView` 降权为实验/模板画布，右侧运行入口回到项目运行；未执行真实 Codex，未启动 MCP canvas run，未改 workflow state JSON。
- `workbench-architecture-implementation-plan-v1` 的 Task D 项目黑板最小只读切片已完成，依据见 `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` 与 `handoffs/2026-06-01-project-blackboard-minimal-read-model-d-v1-result.md`。本轮新增 `ProjectBlackboard` / `BlackboardEntry` / `BlackboardEntryKind` / `BlackboardSourceRef` / `BlackboardPromotionDecision` 读模型，项目页只读展示子智能体汇报、风险、权限请求、工具摘要、记忆候选、知识引用；未新增写入命令，未改 workflow state JSON，未让黑板推进状态或写正式记忆。
- `tasks/2026-06-01-control-core-command-convergence-v1.md` 已完成控制核心命令收敛保守切片，依据见 `evidence/2026-06-01-control-core-command-convergence-v1.md` 与 `handoffs/2026-06-01-control-core-command-convergence-v1-result.md`。本轮新增后端 `control_core.rs` helper，接入工作项状态推进、派发准备、派发启动、派发完成/失败、总指导回收、离线派发/回传/回收、工作流机器收口和权限确认；新增权限确认后端命令；黑板候选确认只做边界校验，不写正式事实或正式记忆；未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未改 workflow state JSON 结构。
- `tasks/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` 已完成控制核心第二切片边界拆分，依据见 `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` 与 `handoffs/2026-06-01-control-core-second-slice-boundary-extraction-v1-result.md`。本轮新增 `workflow_state_store.rs`、`workflow_audit.rs`、`workflow_read_model.rs` 三个小模块；状态读写 wrapper、一类 `work_item_state_changed` audit 构造、项目黑板集合派生已接入新 helper；未执行真实 Codex，未读写 `/Users/yoyi/.codex`，未改 workflow state JSON 结构。`cargo fmt --check` 仍因既有 `src/lib.rs` 和 `src/mcp/**` 格式债失败，未在本轮批量格式化。
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` 已完成 `final-skeleton-00` 到 `final-skeleton-10`，以及按用户明确跳过 `final-skeleton-11` 后执行的 `final-skeleton-12` 和 `final-skeleton-13`：
  - `final-skeleton-00-current-facts-freeze-v1` 已完成，只读冻结当前骨架事实，依据见 `evidence/2026-06-01-final-skeleton-00-current-facts-freeze-v1.md` 与 `handoffs/2026-06-01-final-skeleton-00-current-facts-freeze-v1-result.md`。
  - `final-skeleton-01-audit-helper-slice-v1` 已完成，`workflow_permission_decision_recorded` 审计构造已迁入 `workflow_audit.rs`，依据见 `evidence/2026-06-01-final-skeleton-01-audit-helper-slice-v1.md` 与 `handoffs/2026-06-01-final-skeleton-01-audit-helper-slice-v1-result.md`。
  - `final-skeleton-02-read-model-derivation-slice-v1` 已完成，`derive_workflow_ledger_entries` 实际派生逻辑已迁入 `workflow_read_model.rs`，依据见 `evidence/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1.md` 与 `handoffs/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1-result.md`。
  - `final-skeleton-03-tauri-verification-line-design-v1` 已完成真实 Tauri 验收线设计和 Skeleton-04 草案，依据见 `evidence/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1.md` 与 `handoffs/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1-result.md`。
  - `final-skeleton-04-tauri-verification-line-implementation-v1` 已完成真实 Tauri 窗口截图验收线，依据见 `evidence/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md`、`handoffs/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1-result.md` 和 `evidence/tauri-verification/2026-06-01-final-skeleton-04/`。本轮通过真实 Tauri 窗口采集首页、项目页、项目工作流页截图；未覆盖权限确认弹层自动化验收。
  - `final-skeleton-05-canvas-reference-research-v1` 已完成画布参考源复核和能力清单，依据见 `evidence/2026-06-01-final-skeleton-05-canvas-reference-research-v1.md` 与 `handoffs/2026-06-01-final-skeleton-05-canvas-reference-research-v1-result.md`。该轮确认可以进入 Skeleton-06，但只能先写项目画布节点 schema / 计划，不改 UI、不改 workflow state JSON、不启动 MCP canvas run、不执行真实 Codex。
  - `final-skeleton-06-project-canvas-node-schema-v1` 已完成项目工作流画布读模型 schema，依据见 `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`、`evidence/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1.md` 与 `handoffs/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1-result.md`。本轮只写 schema / 计划，未改产品代码，未改 workflow state JSON，未执行真实 Codex，未启动 MCP canvas run；未跑代码测试，因为该切片验收要求是不写实现。
  - `final-skeleton-07` 到 `final-skeleton-09` 画布基础批次已完成，依据见 `evidence/2026-06-02-final-skeleton-07-09-canvas-foundation-batch-v1.md` 与 `handoffs/2026-06-02-final-skeleton-07-09-canvas-foundation-batch-v1-result.md`。本轮新增项目画布读模型、组件状态样例、React Flow 只读项目画布和右侧节点详情收敛；未写真实 workflow state，未改状态机，未启动 MCP canvas run，未执行真实 Codex，未读写 `/Users/yoyi/.codex`。`npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 通过；未产出截图证据，原因是本轮不启动 Tauri 且本地无可用浏览器截图工具。
  - `final-skeleton-10-blackboard-candidate-schema-design-v1` 已完成黑板候选持久状态补充版 schema、迁移计划和 Skeleton-11 实现任务草案，依据见 `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`、`evidence/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1.md` 与 `handoffs/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1-result.md`。补充版已明确 sidecar 路径、原子写入、备份、并发冲突、candidate_key、记录版本和 rejected / discarded 再次出现规则；本轮未实现黑板候选写入，未改 workflow state JSON，未迁移数据库，未写正式事实或正式记忆，未执行真实 Codex，未读写 `/Users/yoyi/.codex`；未跑代码测试，因为该切片只写 schema / 迁移计划。
  - `final-skeleton-12-adapter-capability-registry-v1` 已完成适配器能力声明骨架，依据见 `evidence/2026-06-03-final-skeleton-12-adapter-capability-registry-v1.md` 与 `handoffs/2026-06-03-final-skeleton-12-adapter-capability-registry-v1-result.md`。本轮新增 `adapterCapabilities.ts` 前端只读能力声明读模型，Agent 页展示 Codex adapter 已有能力和边界，`App.tsx` 传入 projects / workflowState 供读模型派生，离线测试覆盖能力声明；未接 Claude / OpenClaw / OpenCode，未改真实 Codex 执行语义，未执行真实 Codex，未改 workflow state JSON，未显示未实现能力按钮，未实现黑板候选写入。
  - `final-skeleton-13-memory-governance-schema-design-v1` 已完成记忆治理 schema 设计，依据见 `docs/plans/2026-06-01-memory-governance-schema-v1.md`、`evidence/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1.md` 与 `handoffs/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1-result.md`。本轮只写 schema / 计划，定义候选、正式记忆目标形状、作用域、来源、生命周期、冲突、审计引用、用户偏好确认规则和 Skeleton-14 草案；未实现候选生命周期，未写正式记忆，未改 workflow state JSON，未迁移数据库，未接 Obsidian / 向量库 / 图数据库。
- 已根据 `archive/decisions/2026-05-29-ui-reference-sources.md` 修正总执行包：画布不是普通 UI 收尾任务，已插入 `final-skeleton-05` 到 `final-skeleton-09` 画布专项链路，覆盖画布参考源复核、节点 schema、组件状态样例、React Flow 项目画布、节点详情面板。
- 当前状态是：`final-skeleton-11` 与 `final-skeleton-14` 已完成候选治理最小闭环；`docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md` 和 `docs/plans/2026-06-01-memory-governance-schema-v1.md` 的候选层边界仍有效。黑板候选仍不能升级为正式事实、正式记忆或 workflow 状态；记忆候选确认仍不能解释为系统已经长期记住。
- 已清理后续任务边界和文档状态：总执行包尾部不再指向已完成的画布基础批次；`final-skeleton-11` 明确不能把黑板候选升级成正式事实 / 正式记忆 / workflow 状态；`final-skeleton-14` 明确只做记忆候选生命周期，不写正式长期记忆；`final-skeleton-12`、`14`、`15` 的完成后跳转已改为继续后续 Skeleton，而不是提前进入最终验收。
- 会话中心可读性重做 v1 已完成（用户提前指派的「Codex 原生感会话中心」第一刀）：去掉智能体页强制选软件层的占位步骤，会话列表从窄栏 4 列表格换成可读会话卡（状态点 + 标题 + 相对时间 · 模型 · 状态），分组头只显示项目末段，提高对比；`format.ts` 新增 `relativeTime` / `pathTail`。只动 `AgentView.tsx`、`format.ts`、`styles.css` 和离线测试；未改 schema / 状态机 / workflow state，未读 `/Users/yoyi/.codex` 真实正文，未截真实 Tauri 窗口。`npm run typecheck` / `npm run test:offline-interaction` / `npm run build` 通过。不接受为真实 Tauri 截图验收，不接受为多智能体会话底座完成。依据见 `evidence/2026-06-02-session-center-legibility-v1.md` 与 `handoffs/2026-06-02-session-center-legibility-v1-result.md`。
- 会话中心可读性重做 v2（五点打磨）已完成：(1) 去掉「已读取索引…需点击确认」常驻提示和同类说明文本，notice 仅在有内容/错误时显示；(2) 全局会话保持按项目分组、标题用 sqlite 真实 codex 标题；(3) 选中会话即自动加载 transcript（不再先点「读取正文」），并定位到 codex rollout 的 `event_msg` / `response_item` 双流问题，前端新增 `conversationTurns` 只显示 `event_msg` 流的人/Agent 消息、过程事件默认折叠，去掉系统提示词注入和重复；(4) 列表列宽 360→248px；(5) 正文区从界面顶部开始。只动 `App.tsx`、`AgentView.tsx`、`styles.css`、离线测试；未改后端 Rust 和 `transcript_reader.py`，未改 schema / 状态机 / workflow state，未读 `/Users/yoyi/.codex` 真实正文。`npm run typecheck`、`npm run test:offline-interaction`（3 scenarios，含双流清洗）、`npm run build` 通过。不接受为真实 Tauri 截图验收，不接受为多智能体会话底座完成；`conversationTurns` 双流判断在真实历史 rollout 上仍建议真机抽查。依据见 `evidence/2026-06-02-session-center-legibility-v2.md` 与 `handoffs/2026-06-02-session-center-legibility-v2-result.md`。
- 会话中心可读性重做 v3（六条快速修复）已完成：(1) 项目分组改可收纳（collapsible），非选中会话所在组默认折叠，选中会话所在组强制展开；(2) 打开会话正文自动滚动到最新消息（非第一条）；(3) 整页缩短，去掉页面外滚动条，列表和正文各自独立内滚；(4) 会话页直接发消息功能用户选择跳过（涉及 `codex exec resume` 和写 `~/.codex`，违反所有治理文档明文禁止）；(5) 定位 rollout 的权限弹窗 z-index 不足、被其他 UI 遮挡、按钮不可点、error 时不关闭，现已修复（z-index: 1000，error 时也关闭弹窗，加 backdrop-click / Escape 键 dismiss）；(6) 很多会话显示「不在索引内，拒绝读取 transcript」，根因是 sqlite 实时会话列表（368 条）vs static index.json（May 31 冻结 356 条）数据源不同步，现已在 Rust backend 新增 sqlite 回退路径（static index 找不到 thread 时从 sqlite 读 thread 记录、临时构建最小 index、传 python reader、删临时文件），python reader 仍验证 rollout 路径安全。前端动 `App.tsx`、`AgentView.tsx`、`PermissionDialog.tsx`、`styles.css`、离线测试；后端动 `lib.rs` 新增 `load_codex_session_transcript_from_sqlite`；未改 schema / 状态机 / workflow state，未读 `/Users/yoyi/.codex` 真实正文。`npm run typecheck`、`npm run test:offline-interaction`（3 scenarios，含收纳/自动滚动）、`cargo test`（90 tests）、`npm run build` 通过。不接受为真实 Tauri 截图验收，不接受为多智能体会话底座完成；sqlite 回退在 368 真实会话上的性能和泛化性仍建议真机抽查。依据见 `evidence/2026-06-02-session-center-legibility-v3.md` 与 `handoffs/2026-06-02-session-center-legibility-v3-result.md`。

判断：

- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md` 已回收为接受：接受为用户审核业务派发极小真实写入闭环已跑通一次；不接受为复杂业务自动编排完成。
- `tasks/2026-05-30-workflow-node-state-closure-fix-v1.md` 已回收为接受：未来派发 completed / failed / timed_out 后，实际派发节点不会永久停在 `running`。
- `workflow-state-real-readme-smoke-node-closure-fix-v1` 已执行：真实 workflow state 中 README smoke 的 codex-dev 节点已从 `running` 修复为 `ready_for_review`。
- `workflow-state-real-readme-smoke-node-closure-fix-v1` 回收为接受：存量节点状态旧账已修复；不接受为新派发再次真实验证。
- `prepare-workflow-state-for-state-closure-retest-v1` 已完成，等待总指导回收：retest work item 已为 `ready_to_dispatch`，active binding 已指向 `019e7738-5e29-74e0-a22f-5c2481b64c38`。
- `prepare-workflow-state-for-state-closure-retest-v1` 回收为接受：retest work item 和 active binding 已准备好。
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md` 回收为需要修改：真实 `codex exec resume` 已运行并写 `/Users/yoyi/.codex`，但超时失败；README 目标行未写入；真实 workflow state 已写 dispatch / control / attempt / audit，retest work item 和 codex-dev node 收口为 `timed_out`，没有残留 `running`。依据见 `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md`。
- `tasks/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md` 已完成：未执行新的 `codex exec resume`，未写 `/Users/yoyi/.codex`，未修改 README；用户批准后已写真实 workflow state，新增 `state-closure-retest-v2` retry work item、active binding、备份和 audit。
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md` 已完成真实派发：使用 v2 work item，cwd 固定为 `/Users/yoyi/codex-workflow-mario-test`，timeout 为 600 秒，prompt 简化为只追加 README 一行；结果 completed。
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md` 回收为接受：README 已追加目标行；v2 work item 进入 `ready_for_review`；codex-dev node 收口为 `ready_for_review`；旧 retest work item 仍为 `timed_out`。依据见 `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v2-review.md`。
- `codex-role-orchestration-offline-closed-loop-v1` 完成：本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未写真实 workflow state；新增离线角色编排 UI、`offline-role-dispatch` 确认动作、固定派发块解析和测试。依据见 `evidence/2026-05-30-codex-role-orchestration-offline-closed-loop-v1.md` 与 `handoffs/2026-05-30-codex-role-orchestration-offline-closed-loop-v1-result.md`。
- `codex-role-orchestration-offline-state-ledger-v1` 完成：本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未通过 UI 写真实 workflow state；新增离线 prepared dispatch、role handoff、director review 后端命令和前端确认动作，测试只写临时状态文件。依据见 `evidence/2026-05-30-codex-role-orchestration-offline-state-ledger-v1.md` 与 `handoffs/2026-05-30-codex-role-orchestration-offline-state-ledger-v1-result.md`。
- `codex-role-orchestration-offline-ledger-review-fix-v1` 完成：本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未通过 UI 写真实 workflow state；修复 completed 未 review 后 UI 丢锚点、重复 prepared 离线派发两个 P1 问题。依据见 `evidence/2026-05-30-codex-role-orchestration-offline-ledger-review-fix-v1.md` 与 `handoffs/2026-05-30-codex-role-orchestration-offline-ledger-review-fix-v1-result.md`。
- `mario-test-four-role-workflow-state-bindings-v1` 完成：用户确认后，已把 `/Users/yoyi/Documents/mario test` 下四个 Codex 会话登记为工作台四角色测试工作流；新增 project、workflow、7 个 nodes、7 条 edges、1 个 `ready_to_dispatch` work item、4 条 active bindings。本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未修改测试项目。依据见 `evidence/2026-05-30-mario-test-four-role-workflow-state-bindings-v1.md` 与 `handoffs/2026-05-30-mario-test-four-role-workflow-state-bindings-v1-result.md`。
- `workflow-machine-four-role-runner-v1` 完成：已实现四角色工作流机器代码路径和 UI 启动入口，新增 `workflow_machine_runs[]`，测试覆盖总指导 -> 开发线 -> 验证线 -> 回收线 -> 总指导结论并收口 accepted；本轮未真实执行 `codex exec resume`，未写 `/Users/yoyi/.codex`，未修改 mario test。依据见 `evidence/2026-05-31-workflow-machine-four-role-runner-v1.md` 与 `handoffs/2026-05-31-workflow-machine-four-role-runner-v1-result.md`。
- `workflow-machine-mario-demo-real-closed-loop-v1` 完成：真实执行四角色工作流机器，最终 v4 work item `workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v4` 已 `accepted`；`/Users/yoyi/Documents/mario test` 生成 `index.html`、`styles.css`、`game.js`、`README.md`。依据见 `evidence/2026-05-31-workflow-machine-mario-demo-real-closed-loop-v1.md` 与 `handoffs/2026-05-31-workflow-machine-mario-demo-real-closed-loop-v1-result.md`。
- `workflow-machine-real-director-natural-accepted-v1` 完成：移除首轮总指导本地 fallback 后，v7 work item `workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v7` 已通过真实四角色会话一轮自然收口为 `accepted`；v5 失败根因是旧 CLI 产物，v6 失败根因是接受标记被摘要截断。依据见 `evidence/2026-05-31-workflow-machine-real-director-natural-accepted-v1.md` 与 `handoffs/2026-05-31-workflow-machine-real-director-natural-accepted-v1-result.md`。

依据：

- `evidence/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md`
- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-result.md`
- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-review.md`
- `tasks/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `evidence/2026-05-30-workflow-node-state-closure-fix-v1.md`
- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-result.md`
- `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-review.md`
- `evidence/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1.md`
- `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-result.md`
- `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-review.md`
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `evidence/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1.md`
- `handoffs/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1-result.md`
- `handoffs/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1-review.md`
- `evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md`
- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-result.md`
- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md`
- `tasks/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md`
- `evidence/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1.md`
- `handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-result.md`
- `handoffs/2026-05-30-workflow-dispatch-timeout-diagnosis-and-retry-prep-v1-review.md`
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md`
- `evidence/2026-05-30-workflow-state-closure-real-dispatch-retest-v2.md`
- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v2-result.md`
- `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v2-review.md`
- `evidence/2026-05-30-codex-role-orchestration-offline-closed-loop-v1.md`
- `handoffs/2026-05-30-codex-role-orchestration-offline-closed-loop-v1-result.md`
- `evidence/2026-05-30-codex-role-orchestration-offline-state-ledger-v1.md`
- `handoffs/2026-05-30-codex-role-orchestration-offline-state-ledger-v1-result.md`
- `evidence/2026-05-30-codex-role-orchestration-offline-ledger-review-fix-v1.md`
- `handoffs/2026-05-30-codex-role-orchestration-offline-ledger-review-fix-v1-result.md`
- `evidence/2026-06-01-workflow-task-package-plan-baseline.md`
- `handoffs/2026-06-01-workflow-task-package-task4-6-user-test.md`
- `evidence/2026-06-01-workflow-task-package-design-v1-execution.md`
- `handoffs/2026-06-01-workflow-task-package-design-v1-execution-result.md`

历史判断：

- `archive/tasks/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1.md` 已回收为代码路径实现。
- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md` 第一次尝试已回收为暂停：没有执行真实 safe probe，因为当时真实 workflow state 没有绑定测试会话，工作项仍为 `draft`。
- `tasks/2026-05-30-prepare-real-workflow-state-for-safe-probe-multiline-v1.md` 已回收为需要修改：workflow state 侧已准备，但绑定 thread 不在当前 `codex-index.json`，后端派发会拒绝。
- `tasks/2026-05-30-refresh-codex-index-for-confirmed-safe-probe-thread-v1.md` 已回收为接受：目标测试 thread 已进入当前 `codex-index.json`，rollout 存在。
- `tasks/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md` 最终回收为接受：真实工作流节点 safe probe 派发闭环已跑通一次。
- `tasks/2026-05-30-dispatch-result-readback-ui-and-director-review-v1.md` 已回收为接受：派发结果 UI 读回和总指导 review 记录入口已实现；随后总指导 `accepted` review 已真实落账。
- `tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md` 先回收为需要修改：协议能力方向正确，验证通过，但本轮自检意外触发 `codex exec resume`，违反任务包禁止项。
- `tasks/2026-05-30-workflow-controlled-execution-protocol-v1-incident-guardrail.md` 已回收为接受：安全搜索规则已补，未再次执行 `codex exec resume`。
- `tasks/2026-05-30-workflow-controlled-execution-protocol-v1.md` 在事故已记录且防护已补后，接受为协议能力完成；不接受为真实业务自动编排完成。
- `tasks/2026-05-30-workflow-protocol-empty-queues-real-state-v1.md` 已回收为接受：真实 workflow state 已初始化协议空队列，并追加审计事件；不接受为真实业务自动编排完成。
- `tasks/2026-05-30-first-user-reviewed-tiny-business-instruction-design-v1.md` 已回收为接受：候选指令设计完成；不接受为用户已批准执行或真实业务试跑已开始。
- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-v1.md` 回收为需要修改：桌面壳已初步接入业务派发参数，但 readback payload、真实超时、失败分类和 `dist/` 产物边界未闭合。依据见 `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-review.md`。
- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md` 已完成代码修正：业务 readback 从 dispatch 恢复 payload，真实 runner 支持 timeout kill，业务失败写入 execution control / attempt，并记录结构化 warning；本轮未执行真实派发、未写 `/Users/yoyi/.codex`。
- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1.md` 回收为接受：接受为代码修正完成，不接受为真实业务派发已验证。依据见 `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-fix-v1-review.md`。
- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md` 此前前置检查暂停：当时真实 workflow state 还没有 `/Users/yoyi/codex-workflow-mario-test` 的 workflow / work item / active binding；后来 README smoke 前置 state 和 UI 派发目标节点解析已补齐，并已通过后续任务处理。它不是当前待派发任务。
- `tasks/2026-05-30-prepare-workflow-state-for-readme-smoke-v1.md` 此前暂停：只读检查发现 `codex-index.json` 没有 `/Users/yoyi/codex-workflow-mario-test` 对应 thread，当时不能创建 active binding；后来测试 thread 已创建，并已通过后续任务处理。它不是当前待派发任务。
- `tasks/2026-05-30-create-codex-workflow-mario-test-session-v1.md` 已完成：用户明确同意后执行真实 `codex exec`，创建 cwd / project_root 为 `/Users/yoyi/codex-workflow-mario-test` 的测试会话 `019e7738-5e29-74e0-a22f-5c2481b64c38`，刷新索引后 rollout 存在；本轮没有执行 `codex exec resume`，没有修改 README，没有写真实 workflow state。
- `tasks/2026-05-30-prepare-workflow-state-for-readme-smoke-v2.md` 回收为需要修改：真实 workflow state 已新增 `/Users/yoyi/codex-workflow-mario-test` project / workflow / README smoke work item / active binding；但桌面壳 UI 当前按 `workItem.current_node_id=director` 查绑定并派发，和实际 Codex binding 所在的 `codex-dev` 节点不一致。依据见 `handoffs/2026-05-30-prepare-workflow-state-for-readme-smoke-v2-review.md`。
- `tasks/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1.md` 回收为接受：UI 现在按 `assigned_role_id` 解析实际派发节点，director 流程节点可派发到 codex-dev 绑定节点；离线测试覆盖该形态；本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未写真实 workflow state，未修改 README。依据见 `handoffs/2026-05-30-workflow-dispatch-target-node-resolution-fix-v1-review.md`。
- `tasks/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1.md` 回收为接受：用户审核业务派发极小真实写入闭环已跑通一次；不接受为复杂业务自动编排完成；codex-dev 节点状态仍为 `running`，需后续修正状态收口。依据见 `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-real-readme-smoke-v1-review.md`。
- `tasks/2026-05-30-workflow-node-state-closure-fix-v1.md` 回收为接受：后端 started 使用实际派发节点置 `running`；completed 后 work item 进 review，实际派发节点收口为 `ready_for_review`；failed / timed_out 后实际派发节点分别收口为 `failed` / `timed_out`。不接受为存量真实 workflow state 已修复。依据见 `handoffs/2026-05-30-workflow-node-state-closure-fix-v1-review.md`。
- `workflow-state-real-readme-smoke-node-closure-fix-v1` 已完成，等待总指导回收：用户明确批准后，真实 workflow state 中 README smoke 的 codex-dev 节点已从 `running` 修复为 `ready_for_review`；本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未修改 README。依据见 `evidence/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1.md` 和 `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-result.md`。
- `workflow-state-real-readme-smoke-node-closure-fix-v1` 回收为接受：README smoke 存量节点状态旧账已修复；不接受为新派发已再次真实验证。依据见 `handoffs/2026-05-30-workflow-state-real-readme-smoke-node-closure-fix-v1-review.md`。
- `prepare-workflow-state-for-state-closure-retest-v1` 已完成，等待总指导回收：用户明确批准后，已新增 retest work item 和 active binding；本轮未执行 `codex exec` / `codex exec resume`，未写 `/Users/yoyi/.codex`，未修改 README。依据见 `evidence/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1.md` 和 `handoffs/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1-result.md`。
- `prepare-workflow-state-for-state-closure-retest-v1` 回收为接受：retest work item 已为 `ready_to_dispatch`，active binding 已指向目标测试 thread；不接受为真实复测派发已执行。依据见 `handoffs/2026-05-30-prepare-workflow-state-for-state-closure-retest-v1-review.md`。
- `tasks/2026-05-30-workflow-state-closure-real-dispatch-retest-v1.md` 回收为需要修改：真实 `codex exec resume` 已执行并写入 `/Users/yoyi/.codex` 与真实 workflow state，但超时失败；README 目标行未追加；retest work item 和 codex-dev 节点均为 `timed_out`，没有 `running` 残留。依据见 `handoffs/2026-05-30-workflow-state-closure-real-dispatch-retest-v1-review.md`。
- `handoffs/2026-05-30-bind-agent-world-business-session-v1-result.md` 已完成：工作流节点 active binding 已从测试会话切换到 cwd 匹配 `/Users/yoyi/gameai/agent world` 的业务会话。
- `handoffs/2026-05-30-first-tiny-business-readonly-check-execution-v1-result.md` 已完成第一次极小业务试跑：结果为目标目录顶层为空，但业务会话执行了 `find/stat` 目录元数据命令，违反候选权限规则，当前记录为 `needs_changes`。
- `handoffs/2026-05-30-workflow-mario-test-project-real-execution-v1-result.md` 已完成测试项目真实创建：第三次尝试通过 `codex exec -C /Users/yoyi --sandbox workspace-write ... resume ...` 让绑定 Codex 会话在 `/Users/yoyi/codex-workflow-mario-test` 创建了四个静态网页小游戏文件；前两次因可写范围 / 只读沙箱失败并已记为 `needs_changes`。
- `handoffs/2026-05-30-workflow-user-reviewed-business-dispatch-v1-review.md` 回收为需要修改：业务派发代码路径已初步接入，但 readback payload、真实超时、失败分类和 `dist/` 产物边界仍未闭合。
- `tasks/2026-05-30-prepare-real-workflow-state-for-safe-probe-v1.md` 是单线草案，当前派发以多线协作版为准。

优先级：

- 先做真实工作流闭环。
- 再用工作流迭代工作台。
- “项目团队工作区 v1”保留为工作流闭环后的表达层，不抢当前优先级。

依据：

- `decisions/2026-05-30-workflow-first-before-workbench-iteration.md`

## 暂停 / 后置

暂停：

- 任务包 ready 流程。
- 写入确认防误触加固。
- Codex 侧边栏项目归属显示修复。
- 每轮默认完整真实窗口自动化验收。当前已有最小 Tauri 截图线，但不是所有 UI 改动都默认完整跑。

后置：

- 多 agent 接入。
- OpenClaw / OpenCode / Claude Code / VS Code 真接入。
- 个人知识库。
- 向量搜索和向量库选型。
- 模型调度。
- Skill 自动安装和仓库化。
- Harness 自动运行。
- 复杂画布编辑器。
- Codex++ 式删除、移动、归档、CDP 注入。
- AionUi / Multica / Langflow / Dify / n8n 等参考源的功能复刻。

## 明确不做

- 不把工作台做成任务包管理器。
- 不直接写 Codex 内部状态库。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件内容。
- 不默认全量展开所有会话正文。
- 不把索引推断当成用户确认事实。
- 不把 safe probe 包装成真实业务自动执行。
- 不绕过用户确认写 `/Users/yoyi/.codex`。

## 任务包口径

任务包保留为：

- 内部协议。
- 审计。
- 导出。
- 交接。

任务包不是：

- 主界面中心。
- 当前产品主流程。
- 用户主要操作对象。

## 下一步建议

Stage J 当前按 `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md` 执行并已完成 J0-J6 收口，最终结论为 `accepted_with_deferred_items`。后续建议进入 post-J 路线：adapter productization、provider / model / credential verification、Tauri UI acceptance hardening、execution operations hardening 和 memory formalization UX。

当前禁止直接声称或执行：

1. 不能把 J2-B 说成完整 C5 / observation / candidate 回收闭环完成。
2. 不能把 J3 说成 Stage J 完成、正式记忆自动写入完成或跨 store 原子性已完整解决；J3 结论为 `accepted_with_deferred_items`。
3. 不能把 J2-B B1/B2 探针说成任意项目无限制自由执行完成。
4. 新的真实 `codex exec` / `codex exec resume` 仍需执行点授权，不能因为 J2-B 通过就默认继续发送 prompt 或读写 `/Users/yoyi/.codex`。
5. planned adapters、provider credential / model verification、自动 retry / stop / restart、真实 Tauri 全量验收仍是后续任务。

已完成底座记录：

1. `tasks/2026-06-03-session-center-foundation-hardening-v1.md` 已完成，入口记录见 `evidence/2026-06-03-session-center-foundation-hardening-v1.md` 和 `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`。
2. `tasks/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 已完成，入口记录见 `evidence/2026-06-03-workflow-dispatch-readback-native-parser-v1.md` 和 `handoffs/2026-06-03-workflow-dispatch-readback-native-parser-v1-result.md`。
3. 后续继续禁止在未获明确批准时执行真实 Codex、真实 `codex exec resume`、真实 workflow state 结构变更、数据库迁移和读写 `/Users/yoyi/.codex`。
4. 真实 Tauri 窗口验收和项目工作流画布深化继续作为后续独立任务。

骨架完成后的最近两条主线：

1. Agent adapter 后端能力声明：`tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 已完成并已回收确认，记录见 `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`、`handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md` 和 `handoffs/2026-06-03-adapter-recovery-and-memory-m1-task-package-v1-result.md`。后端 `WorkbenchSnapshot.agent_adapters[]` 已成为主读模型，前端派生只保留 fallback；仍未接 Claude Code、OpenClaw 或 OpenCode。
2. 记忆层 M1 任务包：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md` 已完成，记录见 `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md` 与 `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`。接受为正式记忆受控 store、第一版 version、审计事件和只读读模型骨架完成；不接受为候选采纳、任务包召回、任务包注入、正式记忆生命周期操作、Obsidian / 知识库、向量库或图数据库完成。M1 完成后仍不能宣称中间版本记忆层完成。
3. 记忆层 M1.1 任务包：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md` 已完成，记录见 `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md` 与 `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`。接受为正式记忆创建必须校验 `project_root`、`project_id`、`workflow_id` 和 scope 与后端推导上下文一致，并且 workflow state `projects[]` 必须包含当前项目。
4. 记忆层 M2 任务包：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md` 已完成，记录见 `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md` 与 `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`。接受为候选到正式记忆的受控采纳链路完成；不接受为自动采纳、任务包召回、任务包注入、完整正式记忆生命周期、Obsidian / 知识库、向量库 / 图数据库或中间版本记忆层完成。
5. 记忆层 M3 任务包：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md` 已完成，记录见 `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md` 与 `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`。接受为 ObservationStore 和工作流观察入口完成；不接受为正式记忆生命周期、任务包召回 / 注入或中间版本记忆层完成。
6. 记忆层 M4 任务包：`tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md` 已完成，记录见 `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md` 与 `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`。接受为 `TaskMemoryPacketBuilder` 和任务记忆包预览完成；不接受为任务包注入、worker 已收到记忆包、自动化工作流完成或中间版本记忆层完成。
7. 记忆层 M5 任务包：`tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md` 已完成，记录见 `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md` 与 `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`。接受为冲突和记忆 lint 最小阻断完成；不接受为任务包注入、完整维护任务系统、正式记忆生命周期或中间版本记忆层完成。
8. 记忆层 M6 任务包：`tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md` 已完成，记录见 `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md` 与 `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`。接受为工作流任务包注入和第一条端到端记忆闭环完成；不接受为中间版本记忆层完成、完整正式记忆生命周期完成、完整维护任务系统完成、真实 worker 已执行或自动化工作流产品化闭环完成。
9. 记忆层 M7 任务包：`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md` 与 `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`。接受为全局记忆管理 UI 最小入口、正式 / 候选 / 来源 / 版本 / 审计 / lint / 任务包 eligibility 只读展示完成；不接受为正式记忆生命周期、知识库接口、关系治理、维护任务、完整记忆系统、真实 worker 或真实截图验收完成。
10. 记忆层 M8 任务包：`tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md` 与 `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`。接受为知识库最小入口、`knowledge_doc` 来源引用、正式记忆 / 候选 / 任务包知识引用反向摘要和从明确知识库资料提出记忆候选入口完成；不接受为 Obsidian 原生同步、vault 自动扫描、正式记忆生命周期操作、知识库文档直接写正式记忆或中间版本完整记忆系统完成。
11. 记忆层 M9 任务包：`tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md` 与 `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`。接受为正式记忆版本化编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉 scope、确认权、版本、审计、影响面和记忆中心最小入口完成；不接受为关系治理、维护任务、成熟模式、跨项目记忆、中间版本完整记忆系统、真实 worker 或真实 Codex 执行。
12. 记忆层 M10 任务包：`tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`。接受为最小 entity registry、alias / dedupe 候选、关系候选、已确认关系和任务包关系解释完成；不接受为维护任务、成熟模式、跨项目记忆、中间版本完整记忆系统、向量库 / 图数据库 / GraphRAG、真实 worker 或真实 Codex 执行。
13. 记忆层 M11 任务包：`tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md` 与 `handoffs/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1-result.md`。接受为维护 run、maintenance finding、维护报告、任务包 blocking 协作和记忆中心维护摘要完成；不接受为自动修改正式记忆、成熟模式正式化、跨项目记忆、中间版本完整记忆系统、真实 worker 或真实 Codex 执行。
14. 记忆层 M12 任务包：`tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md` 已完成，记录见 `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md` 与 `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`。接受为成熟模式候选、跨项目主题报告、用户确认后正式 mature pattern 记忆受控写入、任务包召回边界和 M1-M12 gate 摘要完成；不接受为自动技能化、跨项目摘要直接影响 worker、向量库 / 图数据库 / GraphRAG、真实 worker 或真实 Codex 执行。
15. 阶段 C1 任务包：`tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md` 已完成，记录见 `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md` 与 `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`。接受为方案授权对象、授权范围 guard、自动推进前置检查和只读授权状态完成；不接受为真实 worker 已执行、真实 Codex 已执行、阶段 C 完成或自动化工作流产品化闭环完成。
16. 阶段 C2 任务包：`tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md` 已完成，记录见 `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md` 与 `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`。接受为项目咨询方案草案、用户确认入口、C1 授权对象联动和项目工作流侧栏确认 UI 完成；不接受为全局主管复核完成、授权 active、真实 worker 已执行、真实 Codex 已执行、真实项目咨询 agent 已接入或自动化工作流产品化闭环完成。
17. 阶段 C3 任务包：`tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md` 已完成，记录见 `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md` 与 `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`。接受为全局主管方案边界复核、授权 active 受控生效、needs_changes / blocked 暂停、guard 验证摘要和项目工作流侧栏确认 UI 完成；不接受为项目主管拆任务、真实 worker 已执行、真实 Codex 已执行、最终结果复核完成或自动化工作流产品化闭环完成。
18. 阶段 C4 任务包：`tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md` 已完成，记录见 `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md` 与 `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`。接受为项目主管拆任务、授权范围 guard 校验、任务包 / 记忆包准备和 prepared dispatch 落账；不接受为真实 worker 已执行、真实 Codex 已执行、worker 结构化汇报、项目主管过程事实确认、最终结果复核或自动化工作流产品化闭环完成。
19. 阶段 C5 任务包：`tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md` 已完成，记录见 `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md` 与 `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`。接受为 worker 结构化汇报记录、项目主管低风险本项目过程事实确认、`process_fact` observation 写入、返工 / 阻断 review 落账，以及失败 / readback / 权限最小可见化；不接受为真实 worker 已执行、真实 Codex 已执行、最终结果复核、用户结果接受、正式事实写入、正式记忆写入或完整自动重试系统完成。
20. 阶段 C6 任务包：`tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md` 已完成，记录见 `evidence/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md` 与 `handoffs/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1-result.md`。接受为全局主管最终复核、用户结果决定、阶段 C gate 摘要和阶段 C 的 C1-C6 受控闭环完成；不接受为中间版本整体完成、完整记忆系统完成、真实 worker 已执行、真实 Codex 已执行、正式事实写入或正式记忆写入。

G5 后续主线已经调整为 H-I：先在阶段 H 产品化 `codex-local` 真实自动化工作流，再在阶段 I 抽象多 agent / 多模型协作协议。阶段 D / M13、阶段 E / E7、阶段 F / F5 和阶段 G / G5 均已完成并冻结为 `accepted_with_deferred_items`；不能把 M10 实体 / 关系治理、M11 维护任务、M12 mature pattern gate 摘要、阶段 E 只读边界、E5 Level B 单 session 健康探针或 G1/G2 运维底座单独当成通用真实自动化完成，也不能跳过 H0 安全边界直接做真实派发。模型和凭据基础在阶段 E 已完成只读边界；完整全局模型库、项目模型池、凭据权限、成本统计、provider 验证和外部 adapter 真实接入仍排在 I 阶段之后的独立任务。
