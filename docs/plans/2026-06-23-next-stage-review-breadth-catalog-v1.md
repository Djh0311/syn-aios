# 全面审查 · 广度收敛目录 v1(2026-06-23)

> 来源:6 路并行 workflow 静态扫描(read + grep,7 agents)+ 主导线自核关键项。**全部静态判定,未 cargo build / 未真机**——凡删码/接闸/合一,落地前须编译 + 重跑关键测试。
> 配套:主干结论见 `2026-06-23-next-stage-review-findings-v1.md`;服务于「下一阶段 = 收敛两线 + 重做 UX」。
>
> **主导线自核状态**:§C(两套闸)已亲核——`decide_real_execution_command` 为纯函数判决器(强度取决于调用者传的 bool)、path-lock 抄 5 处、两闸分离且 A 的 authorization_complete 不含 path-lock,均与独立 grep 一致 ✓。**⚠️ §C 的"留 B 退 A"是判断非事实**:反方案是 A 线执行核心更全(强闸+乐观锁+readback+guard+sidecar 已建,B 偏弱),留哪套编排须用户拍,见 §F 待决。其余 A/B/dead 分类未逐条复核,删码前按 §F 第1步 cargo build 兜底。

---

## A. 前端文件分类(摘要)
- **A 线(角色循环 UI,整体弃用但代码 LIVE,~13000+ 行)**:`ProjectWorkflowExecutionPanels`(979)、`ProjectWorkflowGovernancePanels`(754)、`ProjectTaskDraftPanels`(796)、`ProjectWorkflowMemoryPanels`(526)、`ProjectWorkflowSidePanel`(410)、`MemoryCenterView`(676)、`agent/AgentExecutionPanels`(553,H5真执行)、`agent/AgentContinuationBoundaryPanels`(551)、`memory/*` 子树、`lib/secretaryReadModel`(1100)、`lib/memoryCenter`(1200)、`lib/runQueue`(900,H5运行态)、`lib/planAuthorization`(189)、`lib/projectConsultationProposal`(99) 等 ~30 文件。
- **B 线(执行阶梯+画布)**:`WorkflowCanvasEngine`(977)、`lib/projectCanvas`(1500,事实源)、`WorkflowCommandConsoleView`(255,发令台)、`lib/canvasNodeData`(333)、`agent/AgentChatComposer`(220,甲relay)、`lib/types/canvas`(261,⚠️顶部"A模式"是旧标签实际归B)、`lib/types/manualRelay`(216)、`CanvasView`(23)。
- **shared(~35 文件)**:`lib/types/workflow`(1779,A+B类型混居)、`agent/AgentConversationShell`(1268)、`lib/types/memory`(1392)、`lib/types/execution`(1057)、`lib/tauri`(900,含死包装见D)、`ProjectWorkflowCanvasView`(**1309,A-B集成缝=收敛核心**)、`lib/pageSelectors`(720) 等。
- 🔴 **dead(零引用,可删)**:`DiagnosticsView`(66)、`SessionsView`(98)、`SkillsPluginsView`(69)、`TasksEvidenceView`(61)。
- 🧊 **frozen(仅测试引用,别删)**:`RunningWorkflowsView`(1221,测试fixture·命中记忆「别删」)、`OfflineRoleOrchestrationPanel`(466,仅 offline-permission-dialog 测试)。

## B. 后端模块分类(摘要,src-tauri/src 共 89 文件)
- **A 线(角色循环+记忆闭环,全 LIVE)**:`session_continuation_store`(5218)、`project_workflow_automation`(5056)、`c4_c6_workflow_governance_entrypoints`(2509)、`formal_memory_lifecycle`(1816)、`memory_entity_relation_governance`(1250)、`memory_lint_engine`(1142)、`plan_authorization_store`(1082)、`memory_capture_bus`(1012)、`mature_pattern_governance`(986)、`project_consultation_proposal_store`(875)、`h5_project_dispatch_bridge`(738)、`observation_store`(688) 等 + 8 个 A 行为测试。
- **B 线**:`manual_relay`(4006)、`workflow_chain_controller`(719)、`mcp/*`(tools/storage/orchestrator/commands…)。
- **shared(真执行共用核心)**:`real_execution_command`(**8754,A+B 双线共用 + A 强闸 + legacy 文案**)、`lib.rs`(6834)、`types.rs`(5308)、`commands.rs`(2731,含 B path-lock:1677 + legacy stub:2657)、`codex_local_runner`(2068,`command_plan_for`:1429 拼沙箱·两线共用)、`control_core`(1246)、`codex_db`(947,真会话数据源)等。
- 🔴 **dead(可删)**:`ru_dogfood.rs`(497,零生产调用·仅自测)、`memory_daily_loop.rs`(276,2 fn 零调用)。
- 🧊 **frozen = R3 封存迁移(非死码!)**:`workbench_sqlite_*` 整集群 13 文件 ~1.6 万行(read_cut 2996/production_apply 2107/stop_write 2017/observation_period 1858/…)= **R3 JSON→sqlite 整层迁移**(迁 `workflow_state_meta` + 整套记忆表 formal_memory/candidates/observations/lint/entity_relations/mature_pattern)。**已建+演完(fixture+Level-B)+ 用户拍板 `ready_but_not_executed`「先不翻闸」(ac7813b,CURRENT ④b)**——开关没拨故零 live 触达,**是封存待命、不是废码**。决策 = 保持封存 vs 放弃 JSON→sqlite;与决策②(记忆闭环)联动(它把记忆层也一起迁)。
- 🧊 **frozen 能力(寄生 shared,文件本体不删)**:legacy 四角色引擎 `run_workflow_machine_at`(`workflow_execution_entrypoints.rs:1489`)真实现完整,但生产入口 `commands.rs:2657`+`lib.rs:91` 均 blocked stub,仅 `#[cfg(test)]` 可达。

## C. 两套闸 + 两条执行路径 + 合一建议
| | A 线闸 | B 线闸 |
|---|---|---|
| 名称 | `decide_real_execution_command` | `workflow_engine_test_project_unsealed` |
| 位置 | real_execution_command.rs:185 | commands.rs:1677 |
| 形态 | **多条件强门**(7 拦:user_rejected→duplicate→diagnostics→stale_memory→guard→!readback→!authorization) | **path-lock 单行**:project_root 全等 mario-test |
| 强度 | 强、fail-closed;**但本体仅纯函数判决器**,真强度取决于调用者传的 bool 是否真由授权矩阵/H4/guard/prompt-sha256 算出 | 弱但够狭;只管"能不能跑"不管"怎么跑/几次/重复" |
| 收口 | 单点 | **被抄 5 处**(commands.rs:1688/1757/1920/2254 + chain_controller:304) |
| 缺口 | — | 无在飞/并发闸、无 readback 闸、无 codex-local guard、无乐观锁 |

第三套 `run_workflow_machine` 旧引擎门 = 全封 blocked stub。

**两条执行路径**:
- **A 线**(H5/controlled_session_continuation):两阶段(Phase A 走通管线但 NoopRunner 不真跑 / Phase B 真起进程,prompt 经 stdin 不落盘)+ 三件套(sidecar 乐观锁 expected_store_revision + runtime_log + audit)+ 闸控/授权/readback/续跑全要素齐。
- **B 线**(画布/节点派发+自动连环):单道 path-lock 后直接真跑、resume-only;自动连环 `workflow_chain_controller.rs:296` 4 护栏(runaway 上限 min(节点,请求,50)/拓扑序每节点≤1/stop 可中断/失败即停);落盘仅 dispatch 记录、**无乐观锁**。

**合一建议(workflow 给的,⚠️判断非事实,见 §F 待决)**:留 B 编排作唯一真执行面,把 A 的 `decide_real_execution_command` 强闸搬到 B 边界、path-lock 并入作 `authorization_complete` 必要项(从抄 5 遍收敛为算一次),`command_plan_for` 沙箱底座一字节不动。**反方案**:A 执行核心更全,改留 A 编排 + 给它 B 的 UX。**留哪套须用户拍。**

**安全前提(无论留哪套,落地顺序)**:① 强闸接好但默认仍只对测试项目放行(authorized 必含 path-lock 命中)② 扩真实项目 = 改闸语义 = **高危#3** 须明确授权那一下 ③ 合一**不得**顺手把自动连环放开到非测试/多项目/auto-approve(**高危#4**)④ 每步 cargo build + 重跑测试。

## D. 死码 + 平行重复
**可删 dead(删前 cargo build 确认无 cfg(test) 外残留)**:
- 🔴 `ru_dogfood.rs`、`memory_daily_loop.rs`(去对应 `mod` 声明)
- 🔴 `DiagnosticsView`/`SessionsView`/`SkillsPluginsView`/`TasksEvidenceView` 四文件
- 🔴 `lib/tauri.ts` 死前端包装(`runLegacyWorkflowMachine`/`executeLegacyWorkflowNodeDispatch`/`canvasStartRun/AbortRun/RunStatus/TickRun`/`prepareOfflineRoleDispatch`)——sub-agent 已开 spawn_task `task_d4635fc4`
- 🔴 `lib/projectCanvas.ts` 的 `projectCanvasStateExamples`(仅自引用)

**可删 legacy 引擎(纯瘦身、不碰安全闸)**:`run_workflow_machine_at`(workflow_execution_entrypoints.rs:1489)+ helper + lib.rs 两测试。**保留 blocked stub + 类型 `WorkflowMachineRunRequest/Result`**(stub 签名占用;删了前端 invoke 会落空)。

**可合(A/B 各一套同能力)**:真执行守卫(A `inspect_codex_local_execution_guard` ↔ B 无)、重复在飞(A `has_active_attempt_in_h4_scope` ↔ B 无并发检查·同节点可重复派发)、readback(A 强制 ↔ B 无)、乐观锁(A 有 ↔ B 无)、path-lock 收口(A 单点 ↔ B 抄 5 遍)。

## E. 过期文档(🔴=最严重)
- 🔴 **README.md**:整篇冻在 Stage K 治理期,仍写"治理期不授权真实 codex 执行/不读写 .codex",与 CURRENT.md:7"relay 真发 codex 成功"**正面冲突**;把已归档 STAGE_PLAN 当现行正本。→ 砍成纯入口或标"历史快照·状态以 CURRENT 为准"。
- 🔴 **CURRENT.md ④a**:把 `run_workflow_machine` 描述成"故意锁·impl 完整·=#20 真编排底座",**未反映已被 H5 取代+封**(real_execution_command.rs:153 deprecated:true / :156 replacement)。→ 改"已被取代+封"。
- 🔴 **CURRENT.md ②/④**:通篇"甲→中间→乙"叙事,**没提** A 线建成被弃用 / B 线缠绕 / 两套闸需收敛。→ 增"两线接缝待收敛"条,引 findings。
- **master-roadmap:102** 断链(引不存在的 `docs/architecture/...blueprint`,真身在外部绝对路径)/ **:103** 引已归档 STAGE_PLAN / §1 ✅⏳ 6-18 快照已脱节。
- **AUTHORITY.md** 未索引活跃 plans(非错,覆盖面停在治理期)。
- 待reconcile:doc agent 称本仓 458 commits、被引哈希可核,与开机记忆「git 只到 6-11」不符——**以本仓实测为准**(记忆可能过期/指别的上下文)。

## F. 给下一阶段的收敛清单 + 待你拍的决策
**第1步 零风险瘦身(删死码,不碰闸)**:删 §D 的 dead(后端2 + 前端4 + tauri死包装 + projectCanvasStateExamples)+ legacy 真实现(留 stub)。🧊 别动:RunningWorkflowsView/OfflineRoleOrchestrationPanel、sqlite 集群。每步 cargo build + 测试。
**第2步 文档对齐**:README 砍纯入口;CURRENT ④a 改"已取代+封"、②/④ 增"两线收敛"条;master-roadmap 修 2 断链 + 阶段重排。
**第3步 两线收敛(核心,按高危分档)**:强闸接 B(或 A,见待决)边界、path-lock 作必要项;⚠️ 全程默认只测试项目;扩项目=高危#3;不放开连环到非测试=高危#4;authorized 必含 path-lock(漏掉=真跑逃逸=不可逆,高危#1)。
**第4步 重做 UX(收敛后)**:主战场 `ProjectWorkflowCanvasView.tsx:1309`(A-B 集成缝);A 线 ~3700 行前端按"弃编排但可能留记忆闭环展示"决去留。

**待你拍的 3 个决策**:
1. **留哪套编排** —— B(后来·在用·弱闸)还是 A(更全·强闸·但弃用)?这决定整个第3步。
2. **A 线记忆闭环(MemoryCenter + memory/ 全家桶,体量大且独立)** —— 随角色循环一起弃,还是保留作记忆中心?
3. **sqlite 集群 ~1.6 万行死重** —— 这次清,还是单独排?
