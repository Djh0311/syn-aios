# 回交:方案a 后端(交办「开个新的」= 先生后绑)· 执行线 → 主导线 v1

日期:2026-07-05 · 包:`tasks/2026-07-05-jiaoban-plan-a-new-session-backend-v1.md` · 决策:`decisions/2026-07-05-jiaoban-new-session-birth-before-bind-v1.md`。**子线未 add/未 commit**,工作树留给主导线核。

## 一句话结论

session_choice=new 从清错拒改成真能用:new 分支经**现成** `run_manual_relay_gui_direct_new_session_once`(spawn)+ `poll_manual_relay_attempt`(轮询到终态)在固定测试项目真建会话(初始化消息)→ 回执取 `thread_event_summary.thread_id` → **existing 同一套** `bind_workflow_node_codex_session_for_index_at` 绑定 → 链照旧 resume。真跑一路到 proof 已验(见 §3)。**真跑逮到一个包面外根因、做了最小扩面修复,需主导线核准(见 §5,最重要)。**

## 1. 建了什么(改动面 = 3 文件,git diff --name-only 即全集)

- **director_agent.rs**(包内):
  - `JiaobanNewSessionCreator` trait(可注入·单测 stub)+ 真实现 `ManualRelayJiaobanNewSessionCreator`:组 `ManualRelayGuiDirectNewSessionInput`(cwd/root/write_roots **写死** `WORKFLOW_ENGINE_TEST_PROJECT_ROOT`,trait 签名根本不收路径参数=不可参数化)→ 调 relay 单次路径 → 轮询(1s 间隔,600s 封顶,超时先 `stop_manual_relay_attempt` 再报错)→ 只认 `completed_real_codex` + 非空 thread_id。
  - 合流 inner:new 分支替换清错拒(建→绑→推进);初始化文案人话点名方案标题+工作流;失败 `Err("新会话没建起来:…")` 走 fix3 既有 stopped 审计包裹,**不回落 existing**;成功后 `outcome.warnings` 加「已为这单活新建会话(初始化 ~1 分钟·thread …)」;绑定处只对「会话不在当前索引内」类错误做 ≤30s 兜底重试(落库时差窗口,成功即 break)。
  - inner 签名加 `session_creator` 参数;tauri 包装注入真实现。未知 choice 文案更新。
- **lib.rs**:测试(stub×3 + 新单测×3 + 真跑×1 + 6 处既有调用点补参)+ **1 处非测试代码**(`find_index_thread_or_sqlite` 回退,见 §5)。
- **codex_db.rs**(见 §5):新增 `find_thread_by_id`(按主键精确查·存在性语义)+ 1 回归单测。纯加法,列表查询本体未动。

## 2. §4 机器证据

- 单测(全绿):`confirm_and_start_new_session_births_binds_and_advances`(stub 建→绑→推进全通·warnings 有说明·初始化文案点名方案·绑定实物=stub thread)、`…_failure_audits_no_fallback`(人话错+stopped 审计+绑定数 0+确实走了出生口)、`…_rejected_outside_test_project`(path-lock 在建会话前拒·Panic 桩没炸=出生口没被碰)、`confirm_and_start_rejects_non_pending_and_new_session`(人闸回归·原「new 清错」断言按方案a 重写为未知 choice 清错)、`confirm_and_start_runs_from_pending_with_existing_session`(**existing 回归不变,且现在注入 Panic 桩=existing 分支碰 relay 即炸,回归护栏更强**)、`codex_db::find_thread_by_id_sees_exec_thread_hidden_from_list`(根因回归)。
- 全量:`cargo test --lib` = **658 passed / 0 failed / 39 ignored**(基线 654/0/38 + 单测3 + codex_db 1 + ignored 真跑1;计数不降)。
- fmt:本包全部新增/改动区 `rustfmt --config skip_children=true --check` 净(director_agent/lib 整文件净);**codex_db 预存 fmt 债未动**(check 有 diff 但全在预存区:`is_no_project_cwd`/`truncate_display_title`/旧测试——动它=包外噪音,留整备)。

## 3. §4 真跑证据(`confirm_and_start_new_session_real_run`·47.97s·主导线可按名重跑)

命令:`cargo test --lib confirm_and_start_new_session_real_run -- --ignored --nocapture`

- outcome:`stage=ran`,`completed=1`,warnings=「已为这单活新建会话(…thread **019f323d-c3a8-7fd3-ac40-5f29453c49bf**)」。
- **会话实物**(测试外独立复核过):rollout `~/.codex/sessions/2026/07/05/rollout-2026-07-05T20-25-36-019f323d-….jsonl` 存在;该 thread 在 codex 实时 sqlite 查得到(测试静态索引里**没有**它→只能来自真建);**同一份 rollout 里初始化消息(「交办新会话初始化」×2)和 worker 任务内容(proof 文件名 ×15)都在 = 「链 resume 的就是它」硬证**。
- 绑定记录对:state `workflow_node_session_bindings.native_thread_id` == 出生回执 thread == warnings 里那个(测试内断言,且 ≠ 索引占位)。
- proof:`/Users/yoyi/codex-workflow-mario-test/jiaoban-plan-a-proof.txt` = `plan-a ok 1783254336051`(本轮 token,cat 独立核过)。
- `.codex` 凭据:auth.json mtime 前后都是 **Jun 3 23:54:42 2026**(没碰)。
- 真跑用「所批即所跑」单任务(自包含 objective)喂 `approved_planned_tasks` 跳过真 LM 拆——聚焦本包新链路(出生→绑→resume),减 LM flake 面;LM 拆解路径由既有真跑测试覆盖、本包未动。
- 残留报备:前两轮失败真跑留下 2 条 init-only 孤儿会话(019f3236/019f3238,codex home 里,无绑定无执行,无害可留)。初始化实测 ~7-12s(比决策估的 15-60s 快)。

## 4. relay 门面核实结论(包 §0 ①):**没有本设计无法诚实满足的门**

核过的全部门:输入校验(prompt/target/requested_by 非空);guard(payload 必须 exact original、manual_once 无 auto_chain、new_session 不得带 target_session);runner 闸(`new_session_requires_work_item_id`——**relay 自己锚** `work-item:manual-relay:*`,非自由会话,我们不填不假造;`user_confirmation_state=confirmed`/authorization_scope/audit_refs 全是 relay confirm 步内部自带);target/command_plan 校验(sandbox=workspace-write、cwd==project_root、write_roots==[root]、`-C` 对、非 resume、无审批绕过参数、--json、stdin 无 shell);路径 canonical 校验;查重闸(同 scope 有 running 即拒——原样生效,撞上就是人话错)。**在场类字段**:`MANUAL_RELAY_REAL_CODEX_CONFIRM` env 闸只挂在 `RealCodexEnvGated` 模式;本包走的产品 GUI 模式(`RealCodexProductGui`,与中转页按钮同一条路)不涉及该字段——人闸语义由合流上游 PendingUserConfirmation 校验承担(用户刚点[允许并开始]的直接效果),无伪造。

## 5. ⚠️ 显著报备:包面外最小扩面(需主导线核准/否决)

**真跑逮到的根因**:第一轮真跑,会话真建出来了(thread 019f3236)、绑定却被「会话不在当前索引内(含实时 sqlite)」拒。我先误判为落库时差、加了 30s 重试,第二轮**仍败**——再查钉死:`codex exec` 产的会话 `has_user_event=0`,而 `read_threads_page` 写死 `WHERE has_user_event=1 AND source NOT LIKE '%subagent%'`(会话列表的**显示过滤**)→ exec 会话按 id **永远**查不到,重试无用(先前「几秒后就有」是我手查时没带过滤条件的假象,已撤回)。绑定的「实时 sqlite 回退」注释本意就是让新会话能绑,是列表过滤悄悄把回退语义改窄了(与记忆 `codex-workbench-session-data-sources` 同根)。

**修法(最小加法)**:`codex_db::find_thread_by_id`(按主键精确查一条,不带列表显示过滤)+ `find_index_thread_or_sqlite`(lib.rs)在原列表查询 miss 后**再**精确查一次——原命中路径字节不变。**为什么安全**:找到≠能执行,执行闸(S1/path-lock/沙箱/人闸)全在下游、一字未动;列表 UI 的显示过滤本体未动(subagent 604→136 那刀不受影响,codex_db 单测显式断言列表仍看不见 exec 会话)。**影响面**:`find_index_thread_or_sqlite` 共 4 个生产调用方(bind、dispatch 读、commands 两处读)——行为为原超集:按精确 id 现在能找到 exec/subagent 会话(此前这类「新会话被拒读」本就是已知坑)。两文件都不在 §3 死线名单,但超出包面字面(「director_agent 的 new 分支 + lib 测试」),故单列此节,主导线可否决重做(否决则 new 分支在真机必然 100% 撞绑定拒,需另拍方案)。

## 6. 0-diff 自证

- `git diff --name-only` = **只 3 文件**:`director_agent.rs` / `lib.rs` / `codex_db.rs`。
- 死线逐一 `git diff --stat` 空:**commands.rs / codex_local_runner.rs / manual_relay.rs / real_execution_command.rs / workflow_chain_controller.rs / workflow_execution_entrypoints.rs / workflow_run_dispatch_entrypoints.rs / c4_c6 / control_core / consultant_agent 全部 byte-0-diff**。
- 删除行逐条核过:仅「new 未接」旧错误/旧注释/旧断言(语义被本包取代,已按方案a 重写)+ find_index_thread_or_sqlite 回退改写;codex_db 纯加法零删除。
- 无第二套会话创建(只调 relay 现成路径);cwd 不可参数化(trait 不收路径);人闸未动(new 分支仍在 PendingUserConfirmation 校验+record_decision 之后)。

## 7. 不做/后续

- 前端解禁「开个新的」+ 传 `session_choice:"new"` = 另 5 行小包(包 §2.3,backend 已就绪)。
- codex_db 预存 fmt 债、mcp/protocol.rs 预存 dead_code warning:均未动,留整备。
- 会话复用策略(每单一条)未做优化,按包 §5 不做。
