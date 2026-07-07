# 回交:质量债·redo 幂等(喂已完成事实) + 超时反馈边(自动打回重拆一次)· 执行线 → 主导线 v1

日期:2026-07-08(包 2026-07-07)· 包:`tasks/2026-07-07-jiaoban-quality-debt-redo-idempotence-timeout-feedback-v1.md`。**子线未 commit。** 轻档·集中 director_agent。

## 一句话结论

两债一管子落地:重拆时主管能看到「本单已完成事实+上轮超时事实」(2026-07-06 双删案端到端复刻单测已证根治);任务超时 fail-stop 后自动打回主管重拆**恰一次**(预算写死 1·授权复查/守卫/四护栏全套照过·再败人话回到人)。**前端真 0-diff**(git status 自证:改动只 2 个 rs)。7/7 新单测 + 全量 711/0/41。

## 0. 过程中钉死的一个事实偏差(设计修正·透明报备)

设计时我按 `attempt_state` 推停因形状为 `state=timed_out`——**写测试逼出真相**:`write_failed_dispatch` 把 **dispatch.state 恒写 "failed"**,timed_out 只落 work_item/attempt;链 fail_msg 用 dispatch.state → 历史上停因从来是 `state=failed`(超时不可辨)。**修**(不新造分类):classify 落在 `dispatch.warnings` 的现成 `"timeout"` 信号,链 fail_msg 忠实带出为 `·timed_out` 标记(`state=failed·timed_out`)——判别/事实收集/审计三处消费同一标记。链 fail_msg 文本变化影响面查过:该文本无既有测试断言;前端 classifyBlocked 对历史形(state=failed)行为不变。**顺带说明:这也意味着 2026-07-06 案发时的盘上停因是 `state=failed`——本包之后超时才可辨,反馈边只对新链生效(存量旧停因不会被误触发,也不会被补触发——语义正确)。**

## 1. 落点清单(2 文件)

**consultant_agent.rs(+9 行·全部 ± 行肉眼核过)**:`ProjectContext.prior_completed_summary: Option<String>` 加法字段 + `load_project_context` 构造处 None(**死锚纪律照 memory_summary**:纯装配不填、调用方真 path 填)+ 防回潜断言一条(同款测试内加)。**分流 match/档位函数/prompt 常量/召回逻辑 0 命中**(grep 自证)。

**director_agent.rs**:
- **债一**:`collect_prior_completed_summary(path, workflow_id, auth_created_at_ms)`——全走 audit_events **授权时间窗**(active.created_at_ms 之后→多轮叠加 A+B 全覆盖):①口供事件(与 B1/c4_c6 同构的现成读法)→「任务标题(work_item.title)」— did 首行(status);产物**文件名**(词表死线:不搬产物本体)②完成没口供(链完成审计有、口供没有)→「标题」—(无自述·执行态 completed)③超时事实(·timed_out 标记审计)→「标题」— 上轮超时被杀(考虑拆细)——**[接着跑]人肉路径同样受益**(读盘不依赖谁触发);0 条→None、读失败→None 不挡重拆(增益不是闸);cap 12 行。**只 re-plan 分支填**(None 路·1325 刀B 同位);prompt 渲染段(injected_documents 后)带禁令「别重复执行这些动作」——首跑所批即所跑不经此 prompt、批前预拆不填字段,两处天然不渲染。
- **债二**:薄壳保签名(`run_auto_advance_authorized_role_loop` → `_with_timeout_budget(…, 1)`,**既有调用点/lib.rs 测试 0 改动**);反馈边在 fix3 审计 wrapper 之后——`timeout_auto_replan_decision`(停因 ·timed_out 标记 + 盘上链记录 `stop_requested` 现成 helper 复查·读不到盘保守不续)→ 审计(`role_loop_timeout_auto_replan`·role_loop 族新成员,同 2.4 plan_retried 先例)→ **递归调自身**(approved=None=现成 re-plan 路·预算-1)——授权双点复查(1253)/fix9 守卫/path-lock/prepare guard/四护栏全在递归路径里照过,**不复制路径**;成功轮 warnings 插「已自动打回主管重拆 1 次」;新一轮再败(任何原因)→ message 前缀「已自动重拆过 1 次」/Err 前缀同款——预算耗尽回到人(卡住脸按钮照旧)。
- 链 fail_msg ·timed_out 标记(§0 修正·唯一的既有行为面变化)。

## 2. §4 证据(7/7 新单测·`quality_debt_tests` 自包含 mod)

① `collect_prior_completed_facts_double_delete_case`:**双删案直击**——窗内口供行(标题从 work_item/did/status/产物文件名)+**多轮累计**(第二轮也在)+窗外旧口供不进+无自述行+超时行+0 条 None+读失败 None;
② `prompt_renders_prior_facts_block_only_when_filled`:填了→块+禁令+事实;None→不渲染;
③ `timeout_triggers_one_auto_replan_with_facts_then_ran`:**端到端**——t1 完成(真契约口供落库)→t2 超时→审计恰一条→重拆 director **真收到** t1 口供文本+超时事实(RecordingDirector 捕获 ctx)→第二轮 ran+warnings 记「已自动打回 1 次」;
④⑤ `budget_one_no_loop_on_consecutive_timeouts`:两连超时 director 恰 2 次调用(**绝不循环**)+message 前缀「已自动重拆过 1 次」+第二轮停因仍超时=回到人+审计恰一条;
⑥ `non_timeout_failures_do_not_trigger_replan`:普通 failed 与供给类 Err 都不触发(director 各恰 1 次·无审计);
⑦ `approved_graph_path_untouched`:所批即所跑路 Bomb director 没炸+ran+无重拆审计+无喂料 warning;
⑧ `stop_requested_blocks_auto_replan_decision`:判别函数三分全覆盖(timed_out 形入/user_stop/普通 fail/供给类/None 不入)+判定层盘上 stop_requested=true→拒、false→放行。
- 全量:`cargo test --lib` = **711/0/41**(基线 704+7;计数不降)。fmt skip_children CLEAN。
- **真跑如实标注**:timeout 难真造,stub 级为主(ScriptedRunner 走真 execute→write_failed_dispatch→真 dispatch.warnings 全链,非浅 mock);真机 = 用户日常自然遇超时看「已自动重拆过 1 次」字样,不硬造(包 §4 原文口径)。

## 3. 0-diff 自证(§2.4 全名单)

`git status` 改动面 = **仅 2 文件**(director_agent/consultant_agent)。死线逐一 diff 空:**c4_c6/controller/commands/runner/control_core/worker_report/secretary_agent/global_supervisor_agent/manual_relay/lib.rs/前端 src/tests/scripts 全 0-diff**。红线逐条:反馈边只给 timeout(⑥钉死)/预算写死 1(薄壳常量·⑤钉死)/`load_project_context` 不填新字段(防回潜断言在)/retry 三分与 worker timeout 上限 0 命中(grep 自证)/不搬产物本体(词表·①断言文件名)/无无人值守循环(预算+stop_requested+授权复查三重)。
- 报备一:**global_supervisor_agent 最终 0-diff**——包允许的「口供投影可见性改 pub(crate)」没用上:债一要的是拼人话行,直接用与 B1 同构的 audit_events 现成读法(optional_string_from 遍历),复用 struct 反而绕一层(包 §2.4 写明「若不需要则 0-diff」)。
- 报备二(既有暗债·不动):`director_build_prompt_variant` **从不渲染 memory_summary**——刀B 填的记忆在 director 重拆 prompt 里其实没上(consultant/预拆同费)。本包只加 prior 段(渲染了),memory 段是否补渲染归主导线另拍。

## 4. 真机待验(用户·自然遇到)

日常使用中任务超时后:交货/卡住脸出现「任务「X」上轮超时,已自动打回主管重拆 1 次」;第二轮跑完=活干完了;第二轮再败=卡住脸照旧有按钮,message 带「已自动重拆过 1 次」前缀。

## 5. 回交动作

§4 证据+落点+**前端真 0-diff 实答:是**(git status 仅 2 rs)+§0 事实偏差修正与 §3 两条报备 → 主导线核实物。**子线不 commit。**
