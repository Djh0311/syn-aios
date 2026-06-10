# Stage J / J2 Project Workflow Automation Run Units And Controlled Closed Loop v1

日期：2026-06-09

状态：J2-A 已完成并通过长期只读复核线二次审查，结论为 `accepted_with_deferred_items`。开发线已完成 J2-A 非真实执行产品集成和离线闭环实现；主管线补齐项目页 J2-A 离线编排入口并通过 fresh verify；长期只读复核线确认无 P0/P1，旧项目页历史派发/闭环 UI 口径债保留为 P2 后续清理。真实 Codex 执行点必须在 J2-B 单独冻结并由主管线确认后才能启动。

全局主管任务。本文是 Stage J 的 J2 任务包，用于把 J1 Codex Control Plane 的自由操控入口接入项目工作流自动编排：用户目标进入项目后，由项目主管生成开发线 / 验证线 / 回收线 / 主管复核 run units，每个 run unit 通过统一 Product Command 进入可执行、等待授权、阻断或回收状态，并把 worker report、runtime log、audit、readback、process fact decision 串回工作流。J2 必须复用 C1-C6、H5、I1-I5 和 PCR10，不得另建裸调度器，不得绕过统一 Product Command。

## 0. 先说薄弱点

- C1-C6 已经有受控自动化工作流闭环，但那是方案授权 / prepared dispatch / worker report / process fact / final review 的治理链路，不等于 Stage J 的真实可用自动编排产品入口。
- H5 已经证明项目工作流真实 dispatch probe 可跑，但 H5 仍偏“单点派发产品化”；J2 需要把用户目标拆成多 run unit 并形成开发 / 验证 / 回收 / 复核闭环。
- I1-I5 已经有 `worker_protocol` 中立读模型，但它主要是读模型和协议边界；J2 需要补齐从项目页发起目标、生成 run units、绑定 Product Command、回收状态的产品路径。
- 如果 J2 直接新写 scheduler 或直接调 CLI，就会绕过 J1、PCR10、C5/C6、runtime log、audit、readback 和记忆捕获边界。
- 如果 J2 把 prepared / waiting 状态显示成“worker 正在执行”，用户会误以为自动化已经真实跑完。

## 1. 权威依据

必须服从：

- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

必须优先复用：

- C4：`preview_project_director_task_plan` / `prepare_authorized_auto_dispatch` 及 prepared dispatch 语义。
- C5：`record_worker_structured_report` / `record_project_director_process_fact_decision`。
- C6：`record_global_final_result_review` / `record_user_result_decision`。
- H5：project workflow dispatch 到 CodexLocal request / Product Command / readback / worker report handoff 的边界。
- I1-I5：`worker_protocol.rs` 的 WorkerAdapter / WorkThread / RunUnit / DispatchRequest / WorkerHandoff / ReadbackResult / risk envelope 读模型。
- PCR10：真实执行必须归口统一 `real_execution_product_command`，不得回到 H5 / legacy / direct CLI 作为完成证据。

## 2. J2 目标

J2 要交付：

1. 项目页可从用户目标生成自动编排计划。
2. 项目主管 run unit、开发线 run unit、验证线 run unit、回收线 run unit 和主管最终复核 run unit 可追溯。
3. 每个 run unit 都能生成或引用 Product Command preview / readiness / permission envelope。
4. run unit 可处于 `planned`、`waiting_user`、`ready_to_execute`、`executing`、`completed`、`blocked_by_guard`、`failed`、`readback_unavailable`、`needs_review`、`accepted` 等用户可理解状态。
5. worker report 可以回收到既有 C5 链路。
6. 项目主管过程事实确认可以进入 observation，但不自动写正式记忆。
7. 全局主管最终复核和用户结果决定继续使用 C6 链路。
8. 项目工作流画布、运行中工作流和智能体页能解释当前自动编排处于哪一步。

## 3. J2 非目标

J2 不做：

- 不接 Claude Code / OpenClaw / OpenCode / OpenCode-like 的真实执行。
- 不做 provider credential store 或 model verification。
- 不开放任意目录自由执行。
- 不允许 agent 自治批准权限。
- 不做无确认自动 retry / stop / restart。
- 不把 worker report、runtime log、readback、observation 或 candidate 自动写正式记忆。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript 或 rollout。
- 不把 J2 完成说成 J3 记忆捕获总线完成。
- 不把浏览器 smoke 冒充真实 Tauri 验收；真实 Tauri 关键路径归 J5。

## 4. 分阶段执行

### J2-A：自动编排产品集成和离线闭环

J2-A 允许改产品代码，但不允许执行真实 Codex，不允许发送 prompt，不允许读写 `/Users/yoyi/.codex`。

必须完成：

- 新增或扩展 `ProjectWorkflowAutomationPlan` / `ProjectWorkflowRunUnit` / `ProjectWorkflowAutomationResult` 等等价类型。
- 从项目页用户目标生成 deterministic run units：
  - `director_plan`
  - `developer_execution`
  - `verifier_check`
  - `collector_summary`
  - `director_final_review`
- 每个 run unit 必须绑定：
  - `project_id`
  - `project_root`
  - `workflow_id`
  - `workflow_node_id`
  - `work_item_id`
  - `role`
  - `task_package_ref`
  - `memory_packet_ref`
  - `product_command_ref` 或 `product_command_preview_ref`
  - `runtime_log_refs`
  - `audit_refs`
  - `readback_ref`
  - `worker_report_ref`
  - `observation_refs`
  - `memory_candidate_refs`
- 后端必须复用 J1 `codex_control` / PCR10 Product Command preview / prepare / Phase A，不得另起执行 family。
- J2-A 中 Product Command Phase A flags 必须保持：
  - `prompt_sent=false`
  - `real_codex_executed=false`
  - `writes_codex_home=false`
  - `writes_project_files=false`
- J2-A 可以写工作台自有 sidecar / workflow state / runtime log / audit，用于记录计划、ready / blocked / waiting_user、no-op attempt 和 readback unavailable。
- J2-A 必须支持 worker report fixture 回收，调用既有 C5 `record_worker_structured_report` / `record_project_director_process_fact_decision`；低风险本项目 process fact 可写 observation，但不得自动生成正式记忆。
- 项目工作流页新增或修补“自动编排”摘要：显示 run unit 阶段、当前阻断、等待用户确认、readback 状态和下一步。
- 运行中工作流页能看到 J2 run units 的状态摘要。
- 秘书只解释下一步和风险，不替用户批准执行或确认事实。
- `readback_unavailable` / `readback_failed` / `timed_out` 等 unknown 状态必须保持 `result_count=null`，不能显示成真实 0 条。

### J2-B：受控真实闭环执行点

J2-B 只有在 J2-A 通过测试、只读复核线通过、并且主管线明确启动执行点后才允许真实执行。

默认建议分两条验收，不再拆成更多小任务：

1. `mario test` read-only 闭环：使用既有已验证 session，跑开发线或验证线一个 run unit 的真实 `resume` marker probe，不写项目文件。
2. Stage J 隔离测试项目 workspace-write 闭环：只写任务包中冻结的 `.workbench/stage-j/` 子路径，用于证明自动编排链路能把结果回收到 workflow / C5 / observation。

J2-B 执行点必须再次冻结：

- 项目路径。
- 目标 session 或 new-session strategy。
- sandbox。
- allowed write roots。
- 唯一允许写入文件或目录。
- denied paths。
- prompt summary / ref / hash。
- readback marker。
- expected product command id / attempt refs。
- baseline hash 和回滚 / cleanup plan。

J2-B 成功必须满足：

- 真实执行来自统一 `real_execution_product_command` Phase B。
- 不使用 H5 / legacy / direct CLI / test helper / MCP canvas run 冒充。
- runtime log、audit、product command attempt、continuation attempt、readback refs 和 run unit refs 可追溯。
- worker report 和项目主管 process fact decision 可回收。
- 若进入 observation，只能写 observation；正式记忆仍需 J3 / M2 / M9 / M12 确认链路。

## 5. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增项目内动作和状态面板。

J2 普通 UI 应显示：

- 用户目标。
- 自动编排阶段：计划、开发、验证、回收、复核。
- 当前 run unit 状态。
- 等待用户确认事项。
- 阻断原因。
- readback 状态。
- worker report 摘要。
- process fact / observation 摘要。
- 下一步建议。

J2 普通 UI 禁止显示：

- 裸 CLI 命令。
- raw Product Command JSON。
- sidecar 绝对路径。
- internal id 列表。
- 完整 prompt body。
- 完整 transcript / rollout。
- raw stdout / stderr。
- secret / credential。
- “自动执行已完成”“Codex 已收到任务”“worker 正在执行”等无真实 attempt 支撑的文案。
- “已正式记忆”“自动记住”之类绕过确认的文案。

显示位置：

- `项目`：项目工作流侧栏 / 节点详情显示自动编排计划和 run unit 状态。
- `运行中工作流`：显示 run unit 阶段、等待确认、阻断、失败、readback unknown。
- `智能体`：可显示与 run unit 相关的 session / Product Command 状态，但不重复铺工作流全量管理。
- `记忆`：J2-A 只显示 observation / process fact 关联摘要；完整 capture bus 归 J3。
- `设置 / 开发者`：raw refs、diagnostics、internal ids、sidecar 路径只进开发者区。

本任务不做手机端 UI，不新增 mobile responsive 规则。

## 6. 后端改动范围

允许改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 必要时新增小型模块，例如 `project_workflow_automation.rs`。

默认不改：

- `workflow-state.v0.json` 顶层结构。
- FormalMemory schema。
- provider / credential store。
- planned adapter 真实执行逻辑。
- legacy / H5 真实 runner 的产品入口状态。

数据写入边界：

- 允许写工作台自有 workflow state 既有数组项、product command sidecar、session continuation sidecar、runtime log、audit/readback refs、observation sidecar。
- 不允许写 `/Users/yoyi/.codex`，除非进入 J2-B 明确真实执行点。
- 不允许写测试项目核心业务文件，除非 J2-B 冻结唯一 allowed project write path。

## 7. 前端改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`

默认不改：

- 主导航结构。
- 设置 / 开发者归档规则。
- 记忆中心正式化流程。
- 手机端 / mobile responsive 规则。

## 8. 测试矩阵

J2-A 必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib worker_protocol`
- `cargo test --lib real_execution_command`
- `cargo test --lib runtime_log`
- `cargo test --lib workflow_authorization`
- `cargo test --lib formal_memory`
- `cargo test --lib memory_candidate`
- `cargo test --lib`
- `cargo fmt -- --check`

J2-A 必须新增或覆盖测试：

- 用户目标生成五类 run units。
- run unit 绑定 project / workflow / node / work item / task package / memory packet。
- run unit Product Command preview / Phase A 不发送 prompt、不执行真实 Codex。
- duplicate run unit 或 duplicate active attempt 被阻断或要求新 run。
- blocked_by_guard / readback_unavailable / timed_out 保持 `result_count=null`。
- worker report fixture 可回收为 C5 report。
- 低风险本项目 process fact 可写 observation，但不生成 FormalMemory。
- 普通 UI 不出现裸 CLI、raw JSON、full transcript、自动正式记忆或 planned adapter 可执行文案。

J2-B 真实执行前必须追加：

- 执行点任务包或本包内执行点章节。
- 目标项目 baseline hash。
- prompt summary/ref/hash。
- target session 或 new-session strategy。
- `.codex` 最小副作用说明。
- allowed write path / rollback / timeout plan。
- 真实执行后 evidence / handoff。
- 长期只读复核线复核。

## 9. 分线职责

主管线：

- 维护 J2 任务包、授权边界和 acceptance matrix。
- 派发开发线执行 J2-A。
- 派发长期只读复核线审查 J2-A 和 J2-B。
- 只在 checkpoint 时同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 Stage J plan。

开发线：

- 实现 J2-A 后端、前端和测试。
- 默认不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
- 如认为必须进入 J2-B，先回交执行点授权清单。

记忆线：

- 复核 process fact observation 边界。
- 确认 J2 不绕过 J3 memory capture bus 和 FormalMemory 确认链路。
- 确认 secret/full transcript/rollout 不进入 observation/candidate。

UI 线：

- 收敛项目 / 运行中 / 智能体的信息层级。
- 保证普通用户看到“现在在哪一步、需要我做什么、结果是什么”。
- 不把开发者 raw 状态铺进首屏。

复核线：

- 只读审查 P0/P1/P2。
- 检查 J2 是否绕过 Product Command、C5/C6、记忆确认链路或 UI 显示边界。

## 10. 回交产物

本任务包：

- `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`

J2-A 完成后新增：

- `evidence/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `handoffs/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1-result.md`

主管复核如单独记录，可新增：

- `evidence/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1-result.md`

J2-B 若进入真实执行点，必须另行新增执行点 evidence / handoff，不得混进 J2-A evidence 冒充。

## 11. 收口要求

J2-A 完成后可接受为：

- 项目工作流自动编排 run units 产品集成完成。
- 用户目标到开发 / 验证 / 回收 / 主管复核链路可追溯。
- Product Command preview / Phase A / runtime / audit / readback unavailable / worker report / process fact observation 边界完成。

J2-A 不接受为：

- 真实 Codex 自动多角色闭环完成。
- J2-B 真实执行点完成。
- J3 记忆捕获总线完成。
- 自动 retry / stop / restart 完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- Stage J 完成。

J2-B 完成后可接受为：

- 指定测试项目 / 指定 run units 的一次受控真实自动编排闭环证据。

J2-B 仍不接受为：

- 任意项目无限制自由执行。
- 所有自动工作流场景完成。
- 记忆正式化自动完成。
- 最终蓝图完整工作台完成。

## 12. 禁止口径扫描

J2 收口前必须扫描并分类：

```text
rg -n "J2 已完成|Stage J 已完成|自动化工作流真实编排已完成|Codex 已收到任务|worker 正在执行|已正式记忆|自动记住|runRealExecutionProductCommandPhaseB|executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|previewH5ProjectWorkflowDispatch" CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans prototypes/productized-desktop-shell/src
```

J2-A 普通 UI 不应出现：

```text
codex exec -C
full transcript
rollout
secret
token
.env
keychain
OAuth
provider credential
```

如命中是禁止项、历史记录、测试黑名单或开发者区说明，必须在 evidence 分类。

## 13. 开发线执行结果草稿

日期：2026-06-09

状态：已通过主管线和长期只读复核线收口；J2-A 可接受为 `accepted_with_deferred_items`。

开发线已实现 J2-A：

- 新增后端 `project_workflow_automation.rs`，注册 `run_project_workflow_automation_phase_a`。
- 新增 J2-A automation input / plan / run unit / result / read model 类型，并接入 `WorkbenchSnapshot.project_workflow_automation`。
- 复用 J1 `codex_control` 与 PCR Product Command preview / prepare / decision / Phase A no-op。
- 五类 run units 已覆盖：`director_plan`、`developer_execution`、`verifier_check`、`collector_summary`、`director_final_review`。
- Phase A flags 保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- worker report fixture 回收到 C5；低风险本项目 process fact 写 observation，不生成 FormalMemory。
- Projects / RunningWorkflows / Agent / Secretary 只显示普通用户摘要，不新增一级入口，不新增真实执行按钮。

验证和扫描详见：

- `evidence/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `handoffs/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1-result.md`

## 14. 主管复核收口

日期：2026-06-09

结论：J2-A 带 P2 通过，接受为 `accepted_with_deferred_items`。

接受范围：

- 项目页可从用户目标生成 J2-A 离线自动编排记录。
- 用户目标到五类 run units 的非真实执行产品集成完成。
- J1 `codex_control` / 统一 Product Command preview / prepare / Phase A no-op 链路完成。
- runtime / audit / readback unavailable / worker report fixture / C5 process fact observation 边界完成。
- Projects / RunningWorkflows / Agent / Secretary 只显示普通用户摘要，不新增真实执行按钮。

复核结论：

- P0/P1：无。
- P2：旧项目页派发/闭环区域仍保留历史真实执行口径和 legacy action handler；后端已 sealed，且不是 J2-A 新入口，建议后续迁入历史/开发者区域或继续收敛普通 UI 文案。

记录见：

- `evidence/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1-result.md`

下一步：

- 进入 J2-B 执行点冻结任务包准备；只冻结授权矩阵、session/new-session strategy、sandbox、write roots、prompt hash、readback marker、baseline / rollback / cleanup，不直接执行真实 Codex。
- J3 记忆捕获总线仍是后续任务，不能由 J2-A 冒领完成。
