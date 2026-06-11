# Stage Plan

更新时间：2026-06-11

## 当前阶段

当前阶段已经从“最终工作台骨架执行”切到“中间版本开发”，并在 G5、H-I、PCR10、Stage J、Stage K 和 Stage L/L0 后进入 Root Treatment / Stage R 治理阶段。R4-A9 AgentView Transcript Component Extraction 已完成并通过复核线 `STATUS: CLEAR`，implementation commit 为 `886d3cf9bf7bb70fb37bedfe6fc7d6ec6be3f347`；只接受为 `AgentView.tsx` 第一批低风险 transcript 展示组件抽取，不接受为 R4 完成、智能体页 UI 重做、自由 Codex 控制台能力改变、页面真实数据来源迁移、`query_workbench_page_read_model` 被页面真实消费、`WorkbenchSnapshot` / `load_workbench_snapshot` 废弃、真实 Tauri / 截图验收完成、R3 Level B 或多 agent 并行真实执行解锁。R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review、R3-P0、R3-A1、R3-A2、R3-A3、R3-A4、R3-A5、R3-A6、R3-A7 production preflight scanner / report、R3-A8 copied snapshot temp DB apply / export / rollback boundary、R3-A9 production DB initializer + apply with backup manifest / no read-cut Level A、R3-A10 limited read-cut planning / feature flag fallback Level A、R3-A11 production observation / export verification Level A、R3-A12 stop-write JSON decision / rollback drill Level A、R3-A13 transaction acceptance / cutover gap matrix Level A、R4-A1、R4-A2、R4-A3、R4-A4、R4-A5、R4-A6、R4-A7、R4-A8 和 R4-A9 均已完成。R2-B10 已让 `lib.rs` 达成第一阶段 `<= 15,000` 水位线；R3 Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，不切 app startup / Tauri command / UI / 产品全局读写路径，不停写 JSON / sidecar。当前下一步是准备 R4-A10（对应计划 R4-5）：`styles.css` 风格资产提炼任务包；如要执行任何 R3 Level B，必须另写 execution record、allowed roots、rollback strategy 和 fresh verify。Stage K 当前最终结论冻结为 `accepted_with_deferred_items`；Stage L / L1-L6 在治理冻结期内暂挂为 `deferred_during_root_treatment`，不等于 Stage L 完成或取消，也不等于取消 K3-B1 / K3-B2。治理收口后再回到 Stage L / Stage K 继续处理 K3-B1、K3-B2、真实恢复、操作控制、记忆闭环和日常硬化。治理期不授权真实 Codex 执行、`.codex` 读写、K3-B1 retry、K3-B2、planned adapters 真实接入或 backlog 功能解冻。

中间版本核心目标已经以 `accepted_with_deferred_items` 收口：

1. 自动化工作流闭环。
2. 完整记忆系统闭环。

当前中间版本整体阶段计划：

- `docs/plans/middleware-version-stage-plan-v1.md`

G5 后续 H-I 开发计划：

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

统一 Product Command Routing checkpoint：

- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- PCR10 已完成：`tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`，结论为 `accepted_with_deferred_items`。接受为真实执行归口统一 product command、普通旧入口 guard / legacy 化、PCR9 指定 `mario test` / 指定 `codex-local` session B1/B2 真实 `resume` 探针完成；不接受为任意项目自由执行、通用自由 send / resume 控制台、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、真实 Tauri 全量验收或最终蓝图完成。

Stage J 产品化计划：

- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- Stage J 阶段目标固定为“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”。该计划基于已完成的 C1-C6、M1-M13、G1/G2、H/I 和 PCR0-PCR10，不重新造底座。J0 权限、产品范围和验收矩阵冻结已完成：`tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`；J1-A Codex Control Plane 自由操控入口已完成：`tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`；J1-B Mario Test Codex Control Real Resume Execution Point 已完成：`tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`，结论为 `accepted_with_deferred_items`；J2-A Project Workflow Automation Run Units And Controlled Closed Loop 已完成：`tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`，结论为 `accepted_with_deferred_items`；J2-B Controlled Real Workflow Automation Execution Point 已完成 B1 / B2：`tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`；J3 Memory Capture Bus And Candidate Generation 已完成：`tasks/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`，结论为 `accepted_with_deferred_items`；J4 Run Queue, Failure Control, And User Confirmation Queue 已完成：`tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`，结论为 `accepted_with_deferred_items`；J5 UI Information Hierarchy And Real Tauri Product Acceptance 已完成：`tasks/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`，结论为 `accepted_with_deferred_items`；J6 Final Acceptance And Roadmap Freeze 已完成：`tasks/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md`，Stage J 最终结论冻结为 `accepted_with_deferred_items`，记录见 `evidence/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md` 与 `handoffs/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1-result.md`。Stage J 不接受为最终蓝图完整工作台、任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、所有操作自动写 FormalMemory 或完整真实 Tauri UI 自动化验收完成；后续建议进入 post-J 路线。

Stage K 日常可用工作台产品化计划：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- K0 范围、权限和验收矩阵冻结已完成：`tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`，结论为 `accepted`。K1 智能体对话页日常可用重构已完成。K2 通用 Codex `resume` / `new_session` 产品入口已完成并收口为 `accepted_with_deferred_items`：`tasks/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-v1.md`。K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0、K3-B1.0、K3-B1.1、architecture calibration v2/v3 and gate、K4、K5 和 K6 均已完成。K6 final 记录见 `evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。K3-B1 retry 申请再次被安全审查拒绝，记录见 `evidence/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1.md` 与 `handoffs/2026-06-10-stage-k-k3-b1-retry-safety-review-rejection-v1-result.md`；如恢复 K3-B1/K3-B2 真实执行线，必须重新满足授权和安全审查。

Stage L post-K deferred closure 计划：

- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- L0 任务包已完成，结论为 `accepted`：`tasks/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md`。L0 已冻结 Stage L 的目标、权限、安全边界、分线职责和 L1-L6 验收矩阵；不改产品代码，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。记录见 `evidence/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md` 与 `handoffs/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1-result.md`。
- L1 任务包已创建但治理期暂停执行：`tasks/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md`。L1 目标是 K3-B1 blocked recovery product path；默认不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`，不启动 K3-B1 retry 或 K3-B2。

Root Treatment / Stage R 治理计划：

- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`
- R-Preflight 已完成权威入口同步和 git baseline 建立；R0 / R1 / R2-B1 / R2-B2 / R2-B3 / R2-B4 / R2-B5 / R2-B6 / R2-B7 / R2-B8 / R2-B9 / R2-B10 已完成并提交。
- R0 已建立 workbench shape gate、任务包形状影响节、治理任务包类型和解冻后 `1:3` 治理配额。
- R1 已为 workflow state 最终写入 / rename 增加文件级 StoreLock、corrupt guard 和测试夹具 backup retention；完整 read-modify-write 串行化仍是 P2，等待 R2/R3 后续治理。
- R2-B1 command registry extraction 已完成：`tasks/2026-06-10-root-treatment-r2-b1-command-registry-extraction-v1.md`；只接受为 command registry 物理拆分，不接受为 R2 完成。
- R2-B2 lib map and workflow state helper extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b2-lib-map-and-workflow-state-helper-extraction-v1.md`；只接受为 R2 代码地图和 workflow state JSON helper 抽出，不接受为 R2 完成。
- R2-B3 workflow state lifecycle and task package chain extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b3-workflow-state-lifecycle-and-task-package-chain-extraction-v1.md`；只接受为 workflow state 生命周期入口和 task package 写入链物理抽出，不接受为 R2 完成。
- R2-B4 workflow run binding and legacy dispatch entrypoints extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b4-workflow-run-binding-and-legacy-dispatch-entrypoints-extraction-v1.md`；只接受为 workflow run / binding / legacy dispatch entrypoints 物理拆分，不接受为 R2 完成。
- R2-B5 workflow read model dispatch summary and readback stats extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b5-workflow-read-model-dispatch-summary-and-readback-stats-extraction-v1.md`；只接受为 workflow read model / dispatch summary / readback stats 物理拆分，不接受为 R2 完成。
- R2-B6 workflow execution control offline role and machine extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b6-workflow-execution-control-offline-role-and-machine-extraction-v1.md`；只接受为 workflow dispatch execution control / offline role dispatch / workflow machine 物理拆分，不接受为 R2 完成。
- R2-B7 memory command bridge and context guard extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`；只接受为 memory command bridge / observation bridge / task memory packet preview bridge / context binding guard 物理拆分，不接受为 R2 完成。
- R2-B8 diagnostics provider continuation adapter boundary extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`；只接受为 diagnostics / provider / continuation / adapter / session operation descriptor helper 物理拆分，不接受为 R2 完成。
- R2-B9 index host app assembly extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`；只接受为 index parsing / allowed paths / host OS helper / Tauri app assembly 尾段物理拆分，不接受为 R2 完成。
- R2-B10 C4-C6 automation workflow governance extraction 已完成：`tasks/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`；只接受为 C4-C6 自动化工作流治理连续区块物理拆分和第一阶段 `lib.rs <= 15,000` 水位线达成，不接受为 R2 完成。
- R2 closing / R3 preflight review 已完成，结论为 `DONE_WITH_CONCERNS`：只读复核了剩余 `lib.rs` 结构、inline tests 巨石和 R3 SQLite 前置风险；R3-P0、R3-A1、R3-A2、R3-A3、R3-A4、R3-A5、R3-A6、R3-A7、R3-A8、R3-A9 Level A、R3-A10 Level A、R3-A11 Level A、R3-A12 Level A、R3-A13 Level A、R4-A1、R4-A2、R4-A3、R4-A4、R4-A5、R4-A6、R4-A7、R4-A8 和 R4-A9 已完成，当前下一步是准备 R4-A10（对应计划 R4-5）`styles.css` 风格资产提炼任务包；如需 R3 Level B 必须另行决策，不直接 stop-write。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛。

H-I 当前原则：

- 阶段 H 先产品化 `codex-local` 真实自动化工作流，尤其是通用真实 resume、通用真实 send / 新会话、readback / failure / duplicate guard 和项目工作流真实派发。
- 阶段 I 再建立多 agent / 多模型中立协作抽象，不能把 Codex 多线程协作硬编码为工作台事实模型。
- H0-H6 已由 H7 总复核冻结，阶段 H 最终结论为 `accepted_with_deferred_items`。H7 接受为 H0-H6 总复核、H acceptance matrix、deferred 项冻结和 H-to-I handoff 完成；不接受为 H3-B 真实 new-session 成功、H4-Level-B 真实失败 / 超时探针完成、H6 全量真实 Tauri 截图验收完成、通用自由 Codex 控制台、planned adapters 真实接入或 provider/model verification。除已完成执行点外，未获用户 / 全局主管明确授权前不得执行新的真实 resume、真实新会话、H3-B retry、H4-Level-B 真实失败 / 超时探针、其他 H5 写入型 probe 或其他 H5 真实项目工作流派发。
- I0-I6 已完成，阶段 I 最终结论为 `accepted_with_deferred_items`。I6 接受为 I acceptance matrix、Adapter readiness matrix、planned adapter 后续任务建议和 I-to-next-stage handoff 完成；不接受为 Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入、capability descriptor 等于真实执行能力、provider availability 等于 credential / model 已验证、通用自由 send / resume 控制台或新的真实 Codex 执行授权。H-I 阶段整体收口为 `accepted_with_deferred_items`。
- H2.8 只接受为真实执行前权限弹层预览、审计摘要、runtime log preview、readback 边界、duplicate guard 和 readiness 决策面加固；它自身不授权真实 `codex exec resume`，不发送 prompt，不创建 fixture，也不替代后续已完成的 H2 Phase B 真实探针。

当前中间版本方案：

- `docs/middleware-version-development-plan-v1.md`

当前记忆层实施切片：

- `docs/plans/memory-layer-implementation-slice-v1.md`

当前任务入口：

- M1 已完成：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- M1.1 已完成：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- M2 已完成：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- M3 已完成：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- M4 已完成：`tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- M5 已完成：`tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- M6 已完成：`tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- C1 已完成：`tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- C2 已完成：`tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- C3 已完成：`tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- C4 已完成：`tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- C5 已完成：`tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- C6 已完成：`tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- M7 已完成：`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- M8 已完成：`tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- M9 已完成：`tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- M10 已完成：`tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- M11 已完成：`tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- M12 已完成：`tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- M12.1 已完成：`tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- M13 已完成：`tasks/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`，结论为 `accepted_with_deferred_items`。
- E1 已完成：`tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`。
- E2 已完成：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`。
- E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。
- E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`。
- E5 已完成 Level A：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`。
- E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`。
- E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`，阶段 E 总结论为 `accepted_with_deferred_items`。
- E5 Level B mario test 健康探针已完成：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。
- E/F/G 细化计划已完成：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`；F1 项目工作流画布读模型收敛已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`；F2 节点详情 / evidence surface 已完成：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`；F3 受控工作流编辑提案和布局边界已完成：`tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`；F4 项目画布 / 实验画布边界硬化已完成：`tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`；F5 项目工作流画布产品化验收已完成：`tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`，阶段 F 最终结论为 `accepted_with_deferred_items`。F1/F2/F3/F4/F5 记录见对应 evidence / handoff；不接受为画布编辑器、布局持久化、项目画布和实验画布合一、runtime log 完成、diagnostics 完成、真实 Tauri 验收或阶段 G 已开始。
- H-I 计划已新增：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`。H0 已完成文档冻结并已通过全局主管复核：`tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`；H0 未改产品代码、未执行真实 Codex、未读写 `/Users/yoyi/.codex`。H1 已完成并已通过全局主管复核：`tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`；H1 未执行真实 Codex、未发送 prompt、未读写 `/Users/yoyi/.codex`、未启动 Tauri 或 GUI。H2 任务包已完成到 Phase B `mario test` 真实 resume 产品化探针：`tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`；该结果不代表 H3/H5 或阶段 H 完成。H2.1 已完成执行前授权矩阵和决策工作表：`tasks/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`。H2.2 已完成执行前授权准备读模型和只读 UI：`tasks/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`。H2.3 已完成 request builder 和 CodexLocal guard bridge：`tasks/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`。H2.4 已完成真实执行授权包和 fixture freeze：`tasks/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md`。H2.5 real resume runner execution path and authorized fixture run Phase A 已完成：`tasks/2026-06-07-stage-h-h2-5-real-resume-runner-execution-path-and-authorized-fixture-run-v1.md`。H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成：`tasks/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`；接受为当时前置复核和阻断冻结。H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结：`tasks/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`。后续 H2 Phase B 已在 2026-06-08 对 `mario test` 单独授权并完成一次真实探针；后续任何新的真实 resume 仍需按任务包二次授权。
- H2.8 已完成并回收：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`；它只接受为加固 H2 真实执行前权限弹层预览、审计摘要、runtime log preview、readback 边界和 readiness 决策面，不授权真实执行。
- H3-B final approval / real new session fixture run 已执行一次真实 fixture run 并失败分类：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`；记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。本次不接受为 H3-B 成功，后续 retry 必须重新授权。
- H4 readback / failure / timeout / duplicate guard 产品化任务包已完成 Level A 非真实产品化：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`；记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`。H4-Level-B 真实失败 / 超时探针必须另行授权。
- H5 project workflow real dispatch integration 已完成 Level A 非真实产品路径集成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`；开发记录见 `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`。H5-Level-B 授权与 fixture freeze 已完成；H5-Level-B1 已完成一次 `mario test` read-only 真实 `resume` probe，记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`。H5-Level-B2 已完成一次 `mario test` workspace-write 真实 `resume` probe，记录见 `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1-result.md`。合并型 H5 product command formalization / H5 acceptance checkpoint 已完成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`；后续 H6 真实执行 UI 产品化和 Tauri 验收 checkpoint 已完成。后续任务不再拆过细小 probe，入口文档只在 checkpoint 完成、阻断或阶段边界变化时同步；新的真实执行仍必须执行点授权。如使用 `new_session`，必须等待 H3-B retry 授权回收清楚。

## 阶段状态总览

### 阶段 A：权威入口和底座对齐

状态：已完成，本文已更新为中间版本入口，并已对齐 M1-M13、C1-C6 完成状态和阶段 E / 阶段 G 后续范围。

目标：

- 让 `CURRENT.md`、`AUTHORITY.md`、`README.md`、`STAGE_PLAN.md` 和 `tasks/README.md` 都能识别中间版本方案、记忆层实施切片、M1 / M1.1 / M2 / M3 / M4 / M5 / M6 / M7 / M8 / M9 / M10 / M11 / M12 / M12.1 / M13 完成状态、C1 / C2 / C3 / C4 / C5 / C6 已完成，以及阶段 E / 阶段 G 后续范围。

### 阶段 B：记忆层第一条真实闭环

状态：已完成，M1、M1.1、M2、M3、M4、M5 和 M6 均已完成。

范围：

- M1 到 M6。

当前下一步：

- 阶段 B 已收口；第一条真实记忆闭环已成为 M13 最终验收依据之一。

说明：

- M1 到 M6 只能证明第一条真实记忆闭环完成，不能宣称中间版本记忆层完成。

### 阶段 C：自动化工作流产品化闭环

状态：已完成；C1、C2、C3、C4、C5 和 C6 已完成。

目标：

- 用户确认方案后，项目主管在授权范围内自动派 worker、确认过程事实并推进工作；全局主管只复核方案边界和最终结果。

### 阶段 D：中间版本完整记忆系统

状态：M7 已完成；M8 已完成；M9 已完成；M10 已完成；M11 已完成；M12 已完成；M12.1 已完成；M13 已完成，最终结论 `accepted_with_deferred_items`。

范围：

- M7 到 M13。

目标：

- 补齐正式记忆生命周期、关系治理、维护任务、成熟模式、跨项目记忆、知识库边界和最终验收。

### 阶段 E：会话、adapter、多 agent 和模型凭据底座

状态：E1 已完成；E2 已完成；E3 已完成；E4 已完成；E5 Level A 已完成；E5 Level B mario test 健康探针已完成；E6 已完成；E7 已完成，阶段 E 总结论为 `accepted_with_deferred_items`；E/F/G 细化计划已完成。

目标：

- 在已完成 `agent_adapters[]` 后端读模型的基础上，后续单独设计 Claude Code / OpenClaw / OpenCode 等 adapter，不把 descriptor 当真实接入。
- E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座；`codex-local` 仍是唯一可用 adapter descriptor，planned adapters 不能解释为已接入。
- E2 已完成：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`。E2 只定义会话操作边界契约和只读 / 禁用态可见化，不实现真实发消息、停止、重启、resume、导出、删除或收藏。
- E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。E3 只接受为 provider / model / credential availability 只读边界，不接受为真实凭据、真实 provider 验证、外部模型调用或 planned adapter 接入。
- E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`。E4 只接受为会话继续协议和权限预览，不接受为真实 send / resume、prompt 已发送、attempt / dispatch / readback 写入或阶段 G 真实 Tauri 验收。
- E5 已完成 Level A：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`。接受为代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收；不接受为真实 prompt 已发送、真实 readback 或通用会话继续已验收。
- E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`。E6 只接受为 runtime session attention、readback failed / unavailable 边界、状态摘要 UI 和秘书解释完成，不授权真实执行、自动重试、完整 runtime log 或阶段 G 验收。
- E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`。E7 接受为阶段 E 总复核、accepted / deferred / blocked freeze 和 E-to-F handoff 完成；不授权真实执行、完整 runtime log、diagnostics 或阶段 G 验收。
- E5 Level B mario test 健康探针已完成：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。本轮在用户明确批准后，对 `/Users/yoyi/Documents/mario test` 的“总指导” session 做了一次最小真实 `codex exec resume` 健康探针；真实写入 `/Users/yoyi/.codex`，last message 返回固定标记，四个项目文件 hash 前后一致。该结论只接受为指定 session 的最小健康探针，不接受为通用 send / resume 产品化、自动重试、runtime log、planned adapters 或阶段 G 验收完成。
- E/F/G 细化计划见 `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`；阶段 E、阶段 F 和阶段 G 已收口，中间版本最终结论冻结为 `accepted_with_deferred_items`。
- GEPA 已审核为后置优化层候选，只保留研究和架构预留意识；它不进入当前 E1 / E2 主线，不进入当前执行 backlog。`GEPA-0` 只能在 E1 / E2 / 必要的模型凭据边界任务后重新审核，真正运行 GEPA 必须等阶段 G 的运行日志、诊断、eval、成本、脱敏和回滚底座完成。
- Paseo 已审核为多 agent 运行层外部参考，只保留蓝图约束和后续专题研究意识；它不进入当前 E1 / E2 主线，不进入当前执行 backlog。`PASEO-0` 到 `PASEO-4` 只能在阶段 E / F / G 后续专题中重新审核并单独拆任务包。

### 阶段 F：项目工作流画布产品化深化

状态：已完成整体细化；E5 Level B 健康探针已回收，F1 项目工作流画布读模型收敛已完成，F2 节点详情 / evidence surface 已完成，F3 受控工作流编辑提案和布局边界已完成，F4 项目画布 / 实验画布边界硬化已完成，F5 项目工作流画布产品化验收已完成；阶段 F 最终结论为 `accepted_with_deferred_items`。

阶段 F 剩余：无。阶段 G / G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，只接受为真实 Tauri 10 / 13 部分截图证据。中间版本最终结论冻结为 `accepted_with_deferred_items`。

目标：

- 让项目工作流画布服务项目主管、任务包、权限、readback、记忆召回和审计；React Flow 仍只是渲染映射，不是事实源。
- 阶段 F 按 F1-F5 执行：画布读模型收敛、节点详情、受控编辑 / 布局边界、项目画布 / 实验画布边界、阶段 F 验收。

### 阶段 G：真实验收、运维日志和中间版本收口

状态：已完成整体细化；G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。阶段 G 和中间版本最终结论冻结为 `accepted_with_deferred_items`。

目标：

- 真实 Tauri 验收、运行日志、运维诊断、最终回收报告。
- 阶段 G 按 G1-G5 执行：运行日志边界、诊断 / 健康 / degraded state、真实 Tauri 截图验收、中间版本端到端回放、最终权威验收和 deferred freeze。

### 阶段 H：Codex-local 真实自动化工作流产品化

状态：计划已写，H0 已完成文档冻结并已通过全局主管复核；H1 已完成并已通过全局主管复核；H2 通用真实 resume 产品化任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针；H2.0-H2.8 已完成并回收；后续 H2 Phase B 已在 2026-06-08 对 `mario test` 授权并完成一次真实探针；H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核，只接受为非执行产品路径；H3-B 已执行一次真实 new-session fixture run 但失败分类完成，只接受为失败可追溯和产品路径修补，不接受为成功；H4 readback / failure / timeout / duplicate guard Level A 非真实产品化已完成并通过全局主管复核，只接受为统一产品边界完成，不接受为真实 Codex 执行或 H4-Level-B 探针；H5 project workflow real dispatch integration 已完成 Level A 非真实产品路径集成并通过全局主管复核，只接受为预览 / 校验链路；H5-Level-B 授权与 fixture freeze 已完成；H5-Level-B1 已完成一次 `mario test` read-only 真实 `resume` probe并已通过全局主管复核，只接受为单项目 read-only probe 完成，不接受为 H5 通用产品化或阶段 H 完成；H5-Level-B2 已完成一次 `mario test` workspace-write 真实 `resume` probe，只接受为单项目写入 probe 完成，不接受为 H5 通用产品化或阶段 H 完成；合并型 H5 product command formalization / H5 acceptance checkpoint 已完成并通过全局主管复核，只接受为 H5 product command / bridge、B1/B2 evidence matrix 和测试矩阵收束完成，不接受为 H5 通用产品化或阶段 H 完成。除已完成执行点外，未获用户 / 全局主管明确授权前不得执行新的真实 resume、真实新会话、H3-B retry、真实失败 / 超时探针、其他 H5 写入型 probe 或其他 H5 真实项目工作流派发。

目标：

- 把 E4/E5 的 preview / stub / 单 session 健康探针推进成可在工作台内受控使用的 `codex-local` 真实 send / resume 产品能力。
- 真实执行必须经过项目、方案授权、任务包、任务记忆包、权限确认、运行日志、审计、readback 和失败分类。

推荐任务顺序：

- H0：阶段 H 安全边界和任务包冻结。
- H1：CodexLocalRunner 架构和数据契约。已完成，并已通过全局主管复核。
- H2：通用真实 resume 产品化。H2.0 已完成执行前授权预检 guard / blocked attempt / audit 底座；H2.1 已完成执行前授权矩阵和决策工作表；H2.2 已完成执行前授权准备读模型和只读 UI；H2.3 已完成 request builder 和 CodexLocal guard bridge；H2.4 已完成真实执行授权包和 fixture freeze；H2.5 Phase A 已完成 real runner 非执行产品路径；H2.6 已完成 Phase B 前置复核与 runtime log hardening；H2.7 已完成当时的 Phase B 授权准备复核并冻结阻断；H2.8 已完成真实执行前权限弹层、审计摘要和 readiness 决策面加固；H2 Phase B `mario test` 真实 resume 探针已完成，后续真实执行仍需按任务包授权。
- H2.8：真实执行权限弹层、审计摘要和 readiness 决策面已完成并回收；只接受为执行前决策材料和 UI / readback / audit / runtime log preview 加固，不授权真实 `codex exec resume`。
- H3-A：通用真实 send / 新会话授权冻结、fixture 和边界准备已完成；只接受为非执行授权冻结，不授权真实新会话。
- H3.1：new session request、guard、permission envelope 和 no-op runner 已完成；只接受为非执行产品路径，不授权真实新会话。
- H3-B：通用真实 send / 新会话产品化真实 fixture run。已执行一次真实 fixture run，但结果为 failed / readback_failed；产品路径已补 `--skip-git-repo-check`，下一次 retry 必须重新授权，不能把本次失败分类当作 H3-B 成功。
- H4：readback、失败、超时、取消和重复派发保护。Level A 非真实产品化已完成，记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`；H4-Level-B 真实失败 / 超时探针必须单独授权。
- H5：项目工作流真实派发集成。Level A 非真实产品路径集成已完成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`，开发记录见 `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`；H5-Level-B 授权与 fixture freeze 已完成：`tasks/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`；H5-Level-B1 已完成一次 `mario test` read-only 真实 `resume` probe并已通过全局主管复核：`tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`，记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`；H5-Level-B2 已完成一次 `mario test` workspace-write 真实 `resume` probe：`tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`，记录见 `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`；合并型 H5 checkpoint 已完成：`tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`，记录见 `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md`。
- H6：真实执行 UI 产品化和 Tauri 验收。已完成 checkpoint，结论为 `accepted_with_deferred_items`；不接受为真实 Tauri H6 关键截图清单完整完成或阶段 H 完成。
- H7：H 阶段最终验收和冻结。已完成，阶段 H 最终结论为 `accepted_with_deferred_items`。
- I0：Codex 多线程协作参考复核和抽象映射。已完成，结论为 `accepted`。
- I1-I2 合并 checkpoint：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议。已完成。
- I3-I4 合并 checkpoint：capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。已完成。
- I5：Adapter SDK / CLI parity 和运维诊断预留。已完成。
- I6：I 阶段最终验收和后续 adapter 路线冻结。已完成，阶段 I 结论为 `accepted_with_deferred_items`。

禁止：

- 不经过 H0 直接执行真实 `codex exec` / `codex exec resume`。
- 不把 E5 Level B 单 session 健康探针说成通用真实 send / resume 产品化。
- 不开放自由聊天式裸 Codex 控制器。
- 不让真实执行绕过控制核心、任务包、记忆包、权限、运行日志或审计。

### 阶段 I：多 agent / 多模型中立协作抽象

状态：H7 已冻结阶段 H，I0-I6 已完成，阶段 I 最终结论为 `accepted_with_deferred_items`。H-I 阶段整体收口为 `accepted_with_deferred_items`；后续如继续开发，应进入新的 adapter productization / multi-provider runtime 阶段。

目标：

- 在 H 阶段验证过 `codex-local` 真实执行链路后，抽象 WorkerAdapter、RunUnit、DispatchRequest、PermissionEnvelope、WorkerHandoff、ReadbackResult、RuntimeLog 和 AuditEvent。
- 学习 Codex 多线程协作的主管线派发、开发线执行、回交复核模式，但不能硬编码 Codex thread / subagent 模型。
- 为 Claude Code / OpenClaw / OpenCode / OpenCode-like 等 planned adapters 后续真实接入建立中立协议和安全边界。

推荐任务顺序：

- I0：Codex 多线程协作参考复核和抽象映射。已完成。
- I1-I2：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议。已完成。
- I3-I4：Capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。已完成。
- I5：Adapter SDK / CLI parity 和运维诊断预留。已完成。
- I6：I 阶段最终验收和后续 adapter 路线冻结。已完成。

禁止：

- 不把 Codex thread id 当成工作台业务主键。
- 不让 agent 自治创建、取消、归档、kill 或批准正式 worker。
- 不把 capability descriptor、provider availability 或 planned adapter 说成真实接入。

## 已完成历史阶段

历史阶段 0 到 4 已完成或阶段性收口：

- 阶段 0：接管与事实盘点。
- 阶段 1：只读索引与会话读取。
- 阶段 2：桌面应用壳。
- 阶段 3：Codex 项目级可视化编排。
- 阶段 4：最终工作台骨架执行。

骨架执行已完成到 `final-skeleton-16`，后续又完成：

- 会话中心底座硬化。
- 工作流派发 readback native parser 迁移。
- Agent adapter 后端能力声明读模型。
- 记忆层实施切片和 M1-M4 任务包。

历史细节以 `CURRENT.md`、`tasks/README.md`、`evidence/**` 和 `handoffs/**` 为准。

## 使用规则

- 当前事实先看 `CURRENT.md`。
- 任务入口看 `tasks/README.md`。
- 阶段顺序看本文和 `docs/plans/middleware-version-stage-plan-v1.md`。
- 单个任务执行以对应 `tasks/*.md` 为准。
- 旧骨架总包、旧阶段描述和 archive 只作为历史依据。

## 总边界

- 不默认执行真实 Codex。
- 不默认执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不把候选、知识库命中、LLM 摘要或图谱推断当正式记忆。
- 不让秘书确认事实、派活或写正式记忆。
- 不让画布、UI 或 Markdown 绕过控制核心。
