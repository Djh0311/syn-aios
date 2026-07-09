# 实现任务包:B2·C4b 主管总结→记忆候选(工作流末)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**中等**(链末加主管 LM 总结+喂候选·走现成候选机器)。主导线已 measure-first 亲读(见 §0)。正本:设计 §4.8 + 七查⑥⑦(§4.7)+ 定稿拍2(LM 成本)。上承:**C4a 已核过**(主管终标已在·本片是工作流末的总结治理)。**C4 三片之二;C4c(failed 四选一)另包后置。**

## 0. 接手须知(冷启即读·前提已核到底)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **链末落点(主导线亲读)**:全任务跑完→循环外 `finalize_chain_run(&mut closing, &chain_run_id, "completed", &ts_close)`(director_agent.rs:2131)+ 随后 return DirectorChainOutcome。**主管总结插这**(链 completed 前后)。
- **候选机器现成(直接用·别新造)**:`memory_capture_bus::capture_event`(memory_capture_bus.rs:42)是候选入口;来源枚举已含 `worker_report`/`final_review`(374-377),`final_review`→映射 `global_director_review`(452)。**capture_event 内走现成校验**(control_core:630 `validate_memory_candidate_create`/scope/source_refs);候选落 `memory_candidate_store`(:128 push)。
- **候选治理链不动**:候选 →[属实]确认→转正 是现行制(架构 §8.2「主管总结/工作流总结是候选来源·**转正走确认**」)。**C4b 只造候选·不自动转正·不碰确认门。**
- **成本(定稿拍2)**:主管 LM 只出手两处=判黄牌(C4a 已做)+ **出终局总结(本片)**。总结 = 每条链末**一次** LM 调用。
- **七查⑥⑦(§4.7)**:⑥是否需要生成记忆候选 ⑦是否需要生成用户汇报——本片落这两项(总结=用户汇报·候选=⑥)。

## 1. 拍板摘要

- **做什么**:链末(全任务终标完成后)主管 LM 出**工作流总结**(做了什么/关键事实/未决项)→ ①上货脸主导位呈现(用户汇报)②经现成 capture_event 喂**记忆候选**(转正仍走确认门·不自动)。
- **canon**:主管总结→候选是候选来源·**转正走确认零改**(架构 §8.2);人闸不动。
- **不做**:候选自动转正/绕确认门/failed 四选一(C4c)/审查智能体。

## 一句话判据

**「是不是只:链末加主管 LM 总结(一次调用)+ 总结上货脸 + 经现成 capture_event 喂候选(走现成校验·不绕不自动转正)——而候选确认门/转正/c4_c6 record/worker_report/沙箱/人闸 0-diff、总结失败不崩链?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 链末主管总结(director:2131·一次 LM)

- 全任务终标完成、链 finalize completed 前后:主管 LM(CliDirectorAgent·复用)出**工作流总结**——读本链各任务口供/终标结果 → 出「做了什么·关键事实·未决/风险·后续建议」人话总结;
- **一次调用/链**(守成本);**总结失败/LM 断供 → 软着陆**(链照常 completed·总结缺就出一条 warning·不崩链·不阻断——总结是增益不是闸,区别于 C4a 终标那种驱动性判定);
- 只在**链正常完成**时出总结(waiting_decision/failed/stopped 的链不在本片范围·那些是未完成态)。

### 2.2 总结上货脸(DirectorChainOutcome)

- DirectorChainOutcome 加总结字段(`director_summary: Option<...>`·加法·前端可忽略旧路);链末把总结塞进去→上交货脸主导位呈现;审计记一条 `workflow_chain_director_summary`。

### 2.3 总结→记忆候选(走现成 capture_event)

- 经 `memory_capture_bus::capture_event` 把总结喂成候选:source_type 用能表达「主管/工作流总结」的现成来源(`final_review`→global_director_review·或按现有枚举择最贴的·**不新造绕过校验的旁路**);
- **必经现成校验**(validate_memory_candidate_create/scope/source_refs·脱敏若现有 capture 已做则复用)、**落 memory_candidate_store**、**status=候选态**(不自动转正·转正仍走[属实]确认门);
- source_refs 给指向本链/总结的引用(满足非空校验)。

### 2.4 明确不做

候选自动转正/绕确认门(架构 §8.2 转正走确认)/ failed 四选一(C4c)/ 审查智能体 / 项目级候选主管确认升档(定稿另拍·挂信任阶梯·不进本片)。

## 3. 安全死线

- 候选**确认门/转正流程 0-diff**(只造候选·不动采纳);`c4_c6` record/worker_report(C3)/沙箱/path-lock/授权/execute/人闸 — **0-diff**;
- 总结是**增益不是闸**:总结失败绝不崩链/不改链 completed 判定(区别 C4a 终标);LM 断供软着陆;
- 候选**必经现成校验**(不新造绕校验的旁路);真跑圈测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:①链正常完成→主管总结出(stub LM 返总结)→上 DirectorChainOutcome.director_summary+经 capture_event 落候选(候选态·未转正);②总结 LM 断供→链仍 completed+warning·不崩(软着陆);③候选走现成校验(source_refs 非空等)·落 memory_candidate_store·**status 是候选态不是正式记忆**(证不自动转正);④一次调用/链(stub LM 被调 1 次·不是每任务);
- **回归**:C4a 的 744 测全绿(证终标没碰);候选确认门/转正既有测不破;
- **真跑**(`#[ignore]`):一条链完成→货脸见总结+候选库多一条候选(未转正);
- 三闸绿 + 死线 0-diff 自证 + 计数不降 + fmt **`cargo fmt --check` 真跑**。

## 5. 回交

- §4 证据(尤其「候选态不自动转正」+「总结软着陆不崩链」+「一次 LM/链」)+ 死线 0-diff 自证 → 主导线核实物(**我重点核:确认门没碰、候选没自动转正、总结失败不崩链**）。**子线不 commit。**

## 7. 不接受为

- 候选自动转正/绕确认门(违架构 §8.2)/ 绕候选校验旁路造候选 / 总结失败崩链或改 completed 判定(总结是增益)/ 每任务都调 LM 出总结(一次/链·守成本）/ 碰 c4_c6 record/worker_report/沙箱/人闸 / 提前做 C4c / 做审查智能体或候选主管确认升档 / 自报 fmt 或 ad-hoc rustfmt。
