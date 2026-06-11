# AI Agent Workbench Product Line

## 先读这个

当前权威入口是 `product-line/CURRENT.md`。

当前权威文档索引是 `product-line/AUTHORITY.md`。

`README.md` 只保留产品线定位和入口说明；阶段状态、下一步、后置项、安全边界、归档规则都以 `CURRENT.md` 为准。要找从项目开始到现在仍然算数的文档清单，看 `AUTHORITY.md`。

## 当前定位

Root Treatment / Stage R 是当前治理主线，已进入 R4 read model / frontend slimming。R4-A22 Candidate Governance Fixture Helper Extraction 已完成并通过复核线 `STATUS: CLEAR`；记录见 `tasks/2026-06-11-root-treatment-r4-a22-candidate-governance-fixture-helper-extraction-v1.md`、`evidence/2026-06-11-root-treatment-r4-a22-candidate-governance-fixture-helper-extraction-v1.md` 与 `handoffs/2026-06-11-root-treatment-r4-a22-candidate-governance-fixture-helper-extraction-v1-result.md`，implementation commit 为 `069236a4534a926fd0a5af79c0c29bd8a59423db`。R4-A22 只接受为 R4-6 candidate governance 相关离线 fixture cluster 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B 或 backlog 功能解冻。R3-A13 Level A 已完成但 R3 Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar。正式计划见 `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`，决策锚点见 `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`。当前下一步是准备 R4-A23：继续中等粒度离线交互测试按域拆分。Stage L / L1-L6 在治理冻结期内暂挂为 `deferred_during_root_treatment`；L1 任务包已创建但当前暂停执行：`tasks/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md`。治理期不授权真实 Codex 执行、不读写 `/Users/yoyi/.codex`、不启动 K3-B1 retry 或 K3-B2，不解冻 backlog 功能。

Stage K / K6 Final Tauri Dogfood Core Path Screenshot Acceptance 已完成，结论为 `accepted_with_deferred_items`。记录见 `evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。本轮使用真实 Tauri 桌面壳和 ScreenCaptureKit window-only harness 补齐核心入口截图：首页、智能体、运行中工作流、项目、记忆层、知识库、设置、想法箱、Skill 和 Harness。K6 接受为真实 Tauri dogfood 核心入口验收和 Stage K acceptance freeze；Stage K 当前冻结为 `accepted_with_deferred_items`，不接受为严格无缺口完成。K3-B1 retry 申请被安全审查再次拒绝的事实不变：K3-B1 仍未完成，K3-B2 仍不得启动。

上一 checkpoint：Stage K / K5 Running Todo Failure Recovery And Operation Control UX 已完成，结论为 `accepted_non_real_productization_slice`。记录见 `evidence/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md` 与 `handoffs/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1-result.md`。本轮新增前端只读 `operation_control_summary`，把已有运行队列、待确认、失败控制、读回异常、重复阻断、边界阻断、过期清理和 retry / stop / restart / resume readiness 整理成运行中工作流页的普通用户可读层级；复核线最终结论为通过，无 P0/P1/P2。K5 本轮不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`，不接受为真实 retry / stop / restart / resume 已实现、K5 全量完成或 Stage K 完成。

上一 checkpoint：Stage K / K4 Memory Capture, Candidate, And Task Memory Injection UX 已完成，结论为 `accepted_non_real_productization_slice`。记录见 `evidence/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1.md` 与 `handoffs/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1-result.md`。本轮新增前端只读 `memory_workbench_summary`，把已有捕获、观察、候选、正式记忆、任务记忆包、lint 和 run queue 摘要整理成记忆页 / 运行中工作流页的普通用户可读层级；复核线结论为带 P2 通过，唯一 P2 已修补为中文产品口径。K4 本轮不执行真实 Codex、不发送 prompt、不读写 `/Users/yoyi/.codex`，不接受为 K4 全量完成或 Stage K 完成。

上一 checkpoint：Stage K / K3-B1 retry 申请被安全审查再次拒绝，结论为 `blocked_by_safety_review_again`。记录见 `evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md` 与 `handoffs/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1-result.md`。主管线按 K3-B1.1 路径 B 申请受控真实 retry，但审查拒绝理由为：该非沙箱真实 Codex resume 会发送项目/session 派生 prompt 到外部服务并写入 `/Users/yoyi/.codex`，属于高风险外发；审查明确禁止 workaround / indirect execution / policy circumvention。拒绝后没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。K3-B1.1 产品侧分类修补仍已完成，结论为 `accepted_product_classification_retry_gate`；K3-B1 仍未完成。

上一 checkpoint：Stage J / J6 Final Acceptance And Roadmap Freeze 已完成，Stage J 最终结论冻结为 `accepted_with_deferred_items`。J6 接受为 Stage J 目标“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”的当前产品化 checkpoint 完成：J1/J1-B 覆盖自由操控入口和真实 resume 探针，J2/J2-B 覆盖项目工作流 run units 和真实执行点，J3 覆盖记忆捕获与候选化，J4 覆盖运行队列 / 用户确认 / 失败控制，J5 覆盖 UI 信息层级和真实 Tauri 关键截图探针。记录见 `tasks/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md`、`evidence/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md` 与 `handoffs/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1-result.md`。Stage J 不接受为最终蓝图完整工作台、任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、所有操作自动写 FormalMemory 或完整真实 Tauri UI 自动化验收完成。

上一 accepted checkpoint 是 Stage J / J2-B Controlled Real Workflow Automation Execution Point：B1 / B2 已完成，结论为 `accepted_with_deferred_items`。本轮接受为指定 `mario test` developer run unit read-only 真实 `resume` 探针、指定 Stage J 隔离项目 developer run unit workspace-write 真实 `new_session` 探针完成；B1/B2 均走 J2-B bridge 和统一 `real_execution_product_command` Phase B，readback `result_count=1`；B2 只写 allowed write path，baseline hash 保持冻结值。不接受为完整 C5 / observation / candidate 回收闭环、任意项目无限制自由执行、planned adapters 真实接入、provider/model verification、自动 retry / stop / restart 或 Stage J 完成。记录见 `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`、`evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md`、`handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`、`evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`。

上一 checkpoint：Stage J / J1-B Mario Test Codex Control Real Resume Execution Point 已完成，结论为 `accepted_with_deferred_items`。记录见 `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`handoffs/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1-result.md`、`evidence/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1-result.md`。

再上一 checkpoint：统一 Product Command Routing PCR10 已完成，结论为 `accepted_with_deferred_items`。本轮接受为真实执行归口统一 product command、普通旧入口 guard / legacy 化、PCR9 指定 `mario test` / 指定 `codex-local` session B1/B2 真实 `resume` 探针完成；不接受为任意项目自由执行、通用自由 send / resume 控制台、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、真实 Tauri 全量验收或最终蓝图完成。记录见 `tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`、`evidence/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md` 与 `handoffs/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1-result.md`。

当前主线已经从中间版本 G5、H-I、PCR10 和 Stage J 收口进入 Stage K 日常可用工作台产品化阶段；K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0 bridge / harness、K3-B1.0 prompt freeze repair、K3-B1.1 Codex state permission / retry gate、Stage K architecture calibration v2/v3 and gate、K4 非真实记忆产品化切片、K5 非真实运行 / 待办 / 失败恢复和操作控制切片、K6 真实 Tauri dogfood 核心入口验收均已完成。Stage K 当前最终结论冻结为 `accepted_with_deferred_items`；不接受为严格无缺口完成、K3-B1 retry 成功、K3-B2 可开始、真实 retry / stop / restart / resume 已实现或 planned adapters 真实接入。H-I 整体结论为 `accepted_with_deferred_items`，阶段 I / I6 已完成多 agent / 多模型中立协作抽象和后续 adapter 路线冻结；不接受为 Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。任务包仍只是内部协议、审计、导出和交接物，不是主界面中心。

H3-B final approval / real new session fixture run 已在 2026-06-08 执行一次隔离 fixture 真实 probe：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`。本次确实执行真实 `codex exec`、发送 prompt 并写入 `/Users/yoyi/.codex`，但结果为 `failed_classified`，readback failed / `result_count=null`；不代表真实 Codex session 已成功创建或 H3-B 成功完成。记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。

H4 readback / failure / timeout / duplicate guard 产品化任务包已完成 Level A 非真实产品化：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`。记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`。本轮未执行真实 Codex、未创建 fixture run、未读写 `/Users/yoyi/.codex`。真实失败 / 超时探针必须另行授权为 H4-Level-B。

H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`。记录见 `evidence/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1-result.md`。它自身不授权真实 `codex exec resume`，不发送 prompt，不创建 fixture；后续 H2 Phase B 真实探针已单独授权并完成。

依据：

- `decisions/2026-05-29-codex-session-workflow-route-correction.md` 明确把主线纠偏为 Codex 会话管理和 Codex 工作流编排。
- `archive/handoffs/2026-05-29-codex-bound-session-dispatch-probe-v1-result.md` 证明无业务绑定会话 resume 派发已经通过。
- `archive/handoffs/2026-05-29-desktop-shell-workflow-node-dispatch-codex-instruction-v1-review.md` 接受桌面工作流节点派发代码路径，但明确不接受为真实业务自动工作流。

## 当前工作流顺序

这是已跑通和继续演进的核心闭环，不是最新进度清单。最新完成项、未完成项和下一步以 `CURRENT.md` 为准。

1. 会话全文读取。
2. 会话控制探针。
3. 工作流状态流转。
4. 工作流节点绑定会话。
5. 工作流节点派发 Codex 指令。
6. 执行结果读回。
7. 总指导回收。

## 当前权威文件

- 当前入口：`CURRENT.md`
- 权威索引：`AUTHORITY.md`
- 阶段计划：`STAGE_PLAN.md`
- 任务队列：`tasks/README.md`
- 开发线分工：`DEV_LINES.md`
- 原型工作线：`PROTOTYPE_WORK_LINES.md`
- 原则：`principles.md`
- 想法收纳：`backlog.md`
- 归档索引：`archive/README.md`

## 当前设计草案和计划

- 软件开发架构：`docs/workbench-system-architecture-v1.md`
- 前端显示边界：`docs/workbench-frontend-display-boundary-v1.md`
- 记忆层设计：`docs/memory-layer-design-v1.md`
- 工作流和任务包设计：`docs/workflow-task-package-design-v1.md`
- 中间版本方案：`docs/middleware-version-development-plan-v1.md`
- 中间版本整体阶段计划：`docs/plans/middleware-version-stage-plan-v1.md`
- 阶段 E/F/G 细化计划：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- 阶段 H/I 后续计划：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- 记忆层实施切片：`docs/plans/memory-layer-implementation-slice-v1.md`
- 任务包 UI 显示边界规则：`docs/plans/task-package-ui-display-boundary-rule-v1.md`
- 架构落地执行计划：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- 最终工作台骨架总执行包：`tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- 项目工作流画布节点 schema：`docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

## 当前权威决策

- 技术栈与扩展架构：`decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- 高扩展开发规则：`decisions/2026-05-28-extensible-first-development-rule.md`
- 最小工作流数据模型：`decisions/2026-05-28-codex-workflow-min-model.md`
- 工作流事实层 v0 存储：`decisions/2026-05-28-workflow-state-storage-v0.md`
- UI 与信息架构方向：`decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- Codex 会话管理与工作流编排纠偏：`decisions/2026-05-29-codex-session-workflow-route-correction.md`
- 会话中心与项目内 Agent 会话架构：`decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- 保留会话方案但优先推进工作流：`decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- Codex 会话展示名规则：`decisions/2026-05-30-codex-session-display-name-rule.md`
- 先做真实工作流闭环，再迭代工作台：`decisions/2026-05-30-workflow-first-before-workbench-iteration.md`
- 可编辑画布 + Codex 会话当主管：`decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`
- 项目工作流画布权威关系：`decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- 架构拆模块保护边界：`decisions/2026-06-01-architecture-module-split-guardrail-v1.md`

## 当前技术栈

- 桌面壳：Tauri 2
- 本地核心：Rust
- 前端：React + TypeScript + Vite
- 关系/画布方向：React Flow
- v0 工作流事实层：JSON 文件
- 长期本地事实库方向：SQLite + FTS
- 向量库：后置

依据：`decisions/2026-05-27-technical-stack-and-expansion-architecture.md`、`decisions/2026-05-28-workflow-state-storage-v0.md`。

## 任务包口径

任务包保留为内部协议、审计、导出和交接物，不作为主界面中心。

依据：`decisions/2026-05-29-codex-session-workflow-route-correction.md`。

## 安全边界

- 不写 Codex 内部状态库，除非用户对具体无业务探针给出明确批准。
- 不读取 `auth.json`、`.env`、密钥、token、授权文件内容。
- 不默认全量展开所有会话正文。
- 不把索引推断当成用户确认事实。
- 不把 safe probe 包装成真实业务自动执行。
- 不绕过用户确认写 `/Users/yoyi/.codex`。

## 后置或暂停

- 通用真实 send / resume 产品化进入阶段 H 规划；H0 任务包已完成文档冻结并已通过全局主管复核，H1 任务包已完成并已通过全局主管复核，H2 已完成到 Phase B `mario test` 真实 resume 产品化探针；H2.0 授权预检 guard 已完成，H2.1 执行前授权矩阵和决策工作表已冻结，H2.2 授权准备读模型和只读 UI 已完成，H2.3 request builder 和 CodexLocal guard bridge 已完成，H2.4 真实执行授权包和 fixture freeze 已完成，H2.5 real resume runner execution path and authorized fixture run Phase A 已完成，H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成，H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结，H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成；后续 H2 Phase B 已在 2026-06-08 对 `mario test` 授权并完成一次真实 `codex exec resume` 探针。H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核。H3-B 真实 `codex exec` 新会话仍需用户 / 全局主管二次授权。H4 readback / failure / timeout / duplicate guard Level A 非真实产品化已完成。H5-Level-B 授权与 fixture freeze 已完成；H5-Level-B1 已完成一次 `/Users/yoyi/Documents/mario test` 开发线 worker session 的 `resume` read-only 真实 probe并已通过全局主管复核，记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`、`handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`、`evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`。H5-Level-B2 已完成一次 workspace-write 真实 probe，记录见 `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`。H5 product command formalization / acceptance checkpoint 已完成，记录见 `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md`；它不授权新的真实 Codex 执行，也不接受为 H5 通用产品化或阶段 H 完成。
- H2.8 已完成并回收，作为 H2 Phase B 前的非真实执行修补：补齐权限弹层预览、审计摘要、runtime log preview、readback 边界、duplicate guard 和 readiness 决策面；不授权真实 resume，不发送 prompt，不创建 fixture。
- H3-B 已执行一次真实 fixture run 但失败分类完成；产品路径已补 `--skip-git-repo-check`，任何 retry 仍必须再次确认 fixture、work item / workflow / node 绑定、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、readback、runtime log、audit、evidence 和 rollback。
- 多 agent 真实接入进入阶段 I 规划；E1-E7 已完成 adapter descriptor、会话操作、provider availability、会话继续 preview、Level A stub、Level B 单 session 健康探针、runtime attention 和阶段 E acceptance freeze，planned adapters 仍不可执行。
- OpenClaw / OpenCode / Claude Code / VS Code 真接入。
- 个人知识库。
- 向量搜索和向量库选型。
- 模型调度。
- Skill 自动安装和仓库化。
- Harness 自动运行。
- 复杂画布编辑器。
- Codex++ 式删除、移动、归档、CDP 注入。
- AionUi / Multica / Langflow / Dify / n8n 等参考源的功能复刻。

## 下一步

当前下一步按 `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md` 准备 R4-A23：继续中等粒度离线交互测试按域拆分；不得跳过任务包 / execution record 直接 stop-write JSON / sidecar，也不得把 R4-A22 冒充为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、视觉重做、页面真实数据来源迁移完成、`query_workbench_page_read_model` 被页面真实消费、`WorkbenchSnapshot` 废弃、自由 Codex 控制台能力改变或真实 Tauri / 截图验收完成。Stage K 已完成 K0-K6 并冻结为 `accepted_with_deferred_items`；Stage L / L0 已完成，L1-L6 治理期暂停。若要恢复 K3-B1/K3-B2 真实执行线，必须等治理收口后在 Stage L 中设计合法恢复路径，并由用户手动执行 exact command 并回交，或重新取得风险批准；K3-B1 未成功前不得进入 K3-B2。

J4 已消费 J1 / J2 / J3 已形成的 Product Command、run unit、runtime log、readback、capture event、observation / candidate refs，统一 running / waiting_user / blocked_by_guard / failed / readback_unavailable / timed_out / duplicate_blocked，并保持用户确认和 audit 边界。

后续建议进入 post-J 路线：adapter productization、provider / model / credential verification、Tauri UI acceptance hardening、execution operations hardening 和 memory formalization UX。J2-B / J3 / J4 / J5 / J6 都不能被包装成任意项目无限制自由执行、自动 retry / stop / restart、planned adapters 真实接入、完整真实 Tauri UI 自动化验收或最终蓝图完整工作台完成。
