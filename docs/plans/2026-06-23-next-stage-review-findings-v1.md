# 全面审查 · findings v1(2026-06-23)

> 状态：**审查文档(独立于方向草案)。本版只覆盖"主干"**——A 线真实状态 + A/B 两线接缝(最 load-bearing、主导线亲自核、未二手)。**广度部分(逐文件死码/重复/过期文档全目录)= TODO,见 §5。**
> 方法(已核实物):读命令注册面 `command_registry.rs`、锁边界 `real_execution_command.rs`、两套真执行闸、前端视图与命令调用 grep。每条结论挂 file:line。
> 服务于:`2026-06-23-next-stage-direction-draft-v1.md`(审完回改其方向)。

## 1. 主干结论(一句话)
**A 线(角色循环/方案授权/记忆闭环)不是"建了被冻在后端"——它后端命令全活、前端 ~3700+ 行真 UI 也在,是个"完整建成但 UX 不行、被弃用"的功能;我这一个多月建的 B 线(画布/relay/顺序链/path-lock)在项目页里和它缠在一起,还重复造了一套更简单的真执行路径+闸。** "偏差"的真相 = **在一个 UX 没修好的完整功能旁边,平行又建了一条简化路径。**

## 2. A 线真实状态:全建成、活着(非冻结后端、非占位)
- **后端命令全部注册、可被前端调**(`command_registry.rs`):方案授权 C1(`create_plan_authorization`/`record_plan_authorization_user_confirmation`/`record_global_boundary_review`/`inspect_auto_dispatch_authorization` 等,:21–27)、主管拆任务 C4(`preview_project_director_task_plan`/`prepare_authorized_auto_dispatch` :28–29)、worker汇报+事实确认 C5/C6(`record_worker_structured_report`/`record_project_director_process_fact_decision`/`record_global_final_result_review`/`record_user_result_decision` :42–45)、咨询出方案 C2(`create_project_consultation_proposal` 等 :47–50)、**H5 统一真执行**(`run_real_execution_product_command_phase_a/b` :35–37)、受控会话续真 resume(`run_controlled_session_continuation_real_resume_phase_a/b` :55–56)、记忆 M1–M13 全套(:57–82)。
- **前端是真功能不是占位**:`secretaryReadModel.ts` 1225 行、`ProjectWorkflowExecutionPanels.tsx` 979、`ProjectWorkflowGovernancePanels.tsx` 754、`OfflineRoleOrchestrationPanel.tsx` 466、`planAuthorization.ts` 188、`projectConsultationProposal.ts` 98;`proposal`(方案)和 `ideas`(想法箱)视图在导航里真渲染(`ActiveWorkbenchView.tsx:203/241`,无占位标记)。
- **结论**:用户口述"UI 不行→冻起来" = **完整功能 + 差 UX → 弃用**,不是代码层封存。只有老的 `run_workflow_machine`(四角色引擎)在 wrapper 层 blocked(commands.rs:2657)。

## 3. 四角色引擎(run_workflow_machine):几个月前已被取代+封,不是我退役
- boundary spec(`real_execution_command.rs:144`)写明它 deprecated、`product_routing_allows_real_execution:false`,**replacement = `preview_h5_project_workflow_dispatch + controlled_session_continuation`**(:156–157)。
- 即:A 线的**活**真执行走 H5 受控会话续(`decide_real_execution_command`),四角色引擎是被它取代的旧路径。**"退役/复用"之争其实早有定论——项目自己几个月前就把它换掉封了。**

## 4. 两线接缝:重复 + 缠绕(最关键)
- **两套真执行路径 + 两套闸**:
  - A 线:`controlled_session_continuation` → `decide_real_execution_command`(`real_execution_command.rs:185`,被 `session_continuation_store.rs:1000/1319` 调)= 完整授权状态机(rejected/duplicate/diagnostics/stale_memory/guard/readback/authorization)。
  - B 线:`execute_project_workflow_node`(`commands.rs:1916`)→ `workflow_engine_test_project_unsealed`(:1677,path-lock 只放固定测试项目)= 简单闸。
  - **我建 B 时,A 的更全闸已存在。两套真执行闸并存 = 安全面分叉(两处都要守对)。**
- **UI 层缠绕**:项目页(`ProjectWorkflowCanvasView` + `ProjectWorkflowExecutionPanels`/`GovernancePanels`)**同时调 A 线命令(plan_authorization/real_execution/governance)和 B 线命令(canvas/chain)**——两线不是干净的平行,是在项目工作流 UI 里交织。
- **不是 100% 纯重复**:B 偏"画布单节点/顺序链派发"、A 偏"角色循环/授权自动化";但二者在"真执行 + 闸"这层重叠,应**收敛成一条**。

## 5. 对方向草案的修正(审完回改)
- 原草案说"把冻结的角色循环 UI-first 救活" → **修正:不是救活冻结后端,是"给一个建成但 UX 差、且被 B 线缠绕的完整功能,做 UX 重做 + 两线/两闸收敛"**。
- 下一阶段核心从"建/救" → **"收敛 + 重做 UX"**:① 两条真执行路径+两套闸合一(留 A 的授权状态机、退 B 的 path-lock,还是反过来——待定)② 项目页 UX 重做(显示边界§6:去任务包中心感、字段进详情)③ 角色循环走通一次真用。
- 这也解释了为什么越做越"怪":一直在加平行件,没收敛、没修原 UX。

## 6. 广度 = 已做(workflow 并行扫,见 `2026-06-23-next-stage-review-breadth-catalog-v1.md`)
广度收敛目录已出(6 路并行 + 主导线自核 §C 两闸)。要点:
- 前端 A 线 ~13000+ 行 / B 线 ~10 文件 / dead 4 视图 / frozen RunningWorkflows+OfflineRole(测试 fixture)。
- 后端 A 线巨量(session_continuation 5218/automation 5056…)/ dead `ru_dogfood`+`memory_daily_loop` / 🧊 `workbench_sqlite_*` ~1.6 万行死重 / legacy `run_workflow_machine_at` 可删真实现留 stub。
- **两套闸合一**:留 B 退 A(workflow 建议,⚠️判断非事实,反方案=留 A 更全核心)+ 强闸接边界、path-lock 作 authorization 必要项;高危#1/#3/#4 分档。
- 过期文档:🔴 README(冻治理期·与 relay 收口冲突)、CURRENT ④a(未反映 run_workflow_machine 已取代+封)、CURRENT ②/④(没提两线收敛)、master-roadmap 2 断链。
- **待用户拍 3 决策**:① 留哪套编排 ② A 线记忆闭环去留 ③ sqlite 集群清不清。
