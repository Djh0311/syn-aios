# Stage J / J2-A Project Workflow Automation Evidence v1

日期：2026-06-09

状态：J2-A 已完成并通过长期只读复核线二次审查，结论为 `accepted_with_deferred_items`。

## 范围

- 实现 J2-A 非真实执行产品集成：用户目标 -> 五类 run units -> J1/PCR Product Command preview / prepare / user decision / Phase A no-op -> runtime/audit/readback unavailable -> worker report fixture -> C5 process fact observation -> WorkbenchSnapshot / UI / secretary 摘要。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Browser/Chrome/Tauri/Vite dev/screenshot。
- 未同步 CURRENT.md / tasks README / AUTHORITY.md / STAGE_PLAN.md / README.md / Stage J plan。

## 修改摘要

- 后端新增 `project_workflow_automation.rs`，注册 `run_project_workflow_automation_phase_a`。
- `types.rs` 新增 `ProjectWorkflowAutomationInput`、`ProjectWorkflowAutomationPlan`、`ProjectWorkflowRunUnit`、`ProjectWorkflowAutomationResult`、`ProjectWorkflowAutomationReadModel`，并接入 `WorkbenchSnapshot.project_workflow_automation`。
- `lib.rs` 将 J2-A read model 纳入 snapshot 和 warning count。
- TS 同步类型与 `runProjectWorkflowAutomationPhaseA` wrapper。
- `ProjectsView`、`RunningWorkflowsView`、`AgentView`、`secretaryReadModel` 增加普通用户可读 J2-A 摘要。
- 主管线补齐项目页 J2-A 用户目标输入和“生成 J2-A 离线编排记录”入口；该入口仍走 `PermissionDialog`，只调用 `runProjectWorkflowAutomationPhaseA`，不调用 Phase B、H5、legacy 或真实 runner。
- 离线测试 fixture 增加 J2-A read model，并断言 UI/secretary 不把 readback null 显示为 0。

## 后端链路

- 五类 run units：`director_plan`、`developer_execution`、`verifier_check`、`collector_summary`、`director_final_review`。
- 每个 run unit 绑定 project/workflow/node/work item/task package/memory packet/product command preview 或 ref/runtime/audit/readback/worker report/observation/candidate refs。
- 开发线 run unit 走 `codex_control` -> `prepare_real_execution_product_command_at` -> `record_real_execution_product_command_decision_at` -> `run_real_execution_product_command_phase_a_at`。
- 成功路径 flags 均保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- readback unavailable 的 `result_count` 保持 `null`。
- worker report 通过 `record_worker_structured_report_at` 回收。
- 低风险本项目 process fact 通过 `record_project_director_process_fact_decision_at` 写 observation；未生成 FormalMemory。
- duplicate closed loop 返回 blocked，不再次写 Phase A / report / observation。

## 验证命令

主管线 fresh verify 于 2026-06-09 重新执行并确认：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 13`。
- `npm run build`：通过；保留 Vite chunk size warning。
- `cargo test --lib project_workflow_automation`：通过，4 passed。
- `cargo test --lib worker_protocol`：通过，8 passed。
- `cargo test --lib real_execution_command`：通过，33 passed / 3 ignored，ignored 均为显式真实执行授权测试。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib workflow_authorization`：通过，1 passed。
- `cargo test --lib formal_memory`：通过，29 passed。
- `cargo test --lib memory_candidate`：通过，9 passed。
- `cargo test --lib`：通过，306 passed / 8 ignored，ignored 均为显式真实执行或确认写文件授权测试。
- `cargo fmt -- --check`：通过。

主管线补齐项目页入口后再次确认：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 13`。
- `npm run build`：通过；保留 Vite chunk size warning。

## 扫描分类

禁止口径扫描：

- `prototypes/productized-desktop-shell/src/lib/canvasSurfaceBoundaries.ts:83`：UI 边界黑名单样例，非产品正向文案。
- `AUTHORITY.md`、`tasks/README.md`、`CURRENT.md`：历史入口文档口径；本轮未同步入口文档。
- `docs/plans/2026-06-08-workbench-ui-redesign-brief-for-external-ai-v1.md`：历史设计边界说明。
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`：阶段计划边界说明。
- `runProjectWorkflowAutomationPhaseA` 在 `src/App.tsx` 命中：J2-A 新增受控项目页入口，只调用 Phase A no-op command。
- `src/lib/tauri.ts` 与 `src/App.tsx` legacy/H5/PhaseB wrapper 命中：既有封存/guard 入口或 wrapper 名称；J2-A 新入口未调用 Phase B、H5 或 legacy。

敏感路径 / UI 文案扫描：

- `AgentView.tsx`、`ProjectsView.tsx`、`PermissionDialog.tsx` 中的 `.codex` / `codex exec resume` 等命中为既有真实执行授权边界说明或开发者/权限弹层说明，不是 J2-A 新增执行入口。
- `sessionContinuation.ts`、`h2RealResumeAuthorization.ts` 命中为 guard / denied path / forbidden material 检查。
- `types.ts`、`candidateGovernance.ts`、`projectCanvas.ts` 等命中为 schema 字段或只读状态字段。
- 本轮 J2-A 普通 UI 未新增 raw JSON、sidecar 绝对路径、裸 CLI、full transcript 或自动正式记忆文案。

后端窄扫描：

- `project_workflow_automation.rs` 未命中 `Command::new("codex")`、Phase B 调用或真实 resume runner。
- `prompt_sent: true|real_codex_executed: true|writes_codex_home: true` 命中仅在 `real_execution_command.rs` PCR9A fake/Phase B 既有测试配置区域，不属于 J2-A。

## 剩余风险

- UI 未做真实 Tauri 窗口 / screenshot 验收，按任务边界 deferred。
- J2-A 只证明 Phase A no-op 闭环；不代表 J2-B 真实 Codex 多角色闭环完成。
- 既有项目页旧派发/闭环卡仍有历史真实执行文案和 legacy action handler；后端已 sealed，非 J2-A 新入口，但建议后续迁入历史/开发者区或继续清理普通 UI 口径。
- 入口文档已在主管线 checkpoint 中同步；J2-B 真实执行点仍需另行冻结。

## 长期只读复核线二次审查

日期：2026-06-09

结论：带 P2 通过。J2-A delta 已补齐项目页产品入口，可接受为 `accepted_with_deferred_items`，建议主管线做 checkpoint 入口同步；不得把它表述为 J2-B 真实 Codex 自动编排已完成。

P0/P1：无。

P2：

- 旧项目页派发/闭环区域仍保留历史真实执行口径和 legacy action handler，但后端已 sealed，且不是 J2-A 新入口，维持 P2 后续清理。

复核要点：

- 新 action `run-project-workflow-automation-phase-a` 和 `projectWorkflowAutomation` 已接入。
- `App.tsx` 新确认分支只调用 `runProjectWorkflowAutomationPhaseA`，随后刷新 snapshot / workflow state。
- 项目页入口具备目标输入、可用性约束、`codex-local` session 绑定和 Phase A no-op 边界。
- `PermissionDialog` 明确 J2-A 不发送 prompt、不执行真实 Codex、不写项目文件或 `/Users/yoyi/.codex`。
- Phase B / H5 / legacy 命中仍为既有 wrapper 或旧 handler，不是 J2-A 新入口。

复核边界：

- 复核线未改文件，未跑测试，未执行真实 Codex，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Browser / Chrome / Tauri / Vite / screenshot。
