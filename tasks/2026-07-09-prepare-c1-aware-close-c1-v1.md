# 实现任务包:prepare C1-aware + 合流-new 退 S0 走 C1(C1 架构收官)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**核心逻辑改·按重档仔细度做**(严格论:高危清单不命中=轻档——安全闸[path-lock/沙箱/审批]全 0-diff 不碰;但动 prepare 就绪闸=敏感核心,主导线刚因没吃透 prepare 写错 S0 包,故自加重档仔细度)。正本:canon 决策 `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md`(补拍 A→用户再拍 3)。**取代**撞墙的 `tasks/2026-07-09-jiaoban-new-retire-s0-to-c1-per-task-v1.md`。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **背景死结(前一包撞的墙)**:合流-new 退 S0 → prepare 恒产 needs_binding → 链一个任务不派。根:**prepare 要节点先绑一条会话才产 `prepared`,而 C1 每任务绑在链里、在 prepare 之后**(鸡生蛋)。S0 就是那条「prepare 前的引导绑定」,不是多余会话。
- **主导线已核到底的四个事实(直接用·这次前提是实读不是猜)**:
  1. **prepare 判定**(c4_c6:167-253):`active_binding_for_planned_task` 取不到绑定 →`needs_binding`(168-180);取到但 `!rollout_exists`→`needs_binding`(181-196);有绑定+rollout → 建 prepared dispatch、`status="prepared"`(198-253);
  2. **prepare 把绑定的 `native_thread_id` 烤进 dispatch 记录**(c4_c6:224-225);
  3. **但链派发不用那个烤的 thread**:链用 `node_id`+`work_item_id` 调 `execute_project_workflow_node_at`(director_agent.rs:1205-1212)→ **走节点当前绑定**(C1 的 `create_and_bind_task_session` 1128 刚 rebind 成的每任务会话);
  4. **失败即停已立**(director_agent.rs:1128-1201):C1 建会话失败 → 该任务 failed + 停链、不回落——**派到空会话不可能发生**。
- **结论**:prepare 要绑定,只为(a)过 needs_binding 闸(b)烤 thread 进 dispatch;而 (b) 在 C1 下执行时**根本不用**。所以 C1 自动路可以「prepared 但 thread 延迟、由链补」。

## 1. 拍板摘要

- **做什么**:prepare 加 mode「链会每任务绑」;C1 自动路下**跳过 needs_binding 闸、产 prepared(thread 延迟)**,由链每任务 create_and_bind 补真会话(失败即停兜底);手动挡/existing/前端预 prepare **一字不变**。副产品:合流-new 退 S0 走 C1 变干净(prepare 不再要预绑)。
- **为什么**:消掉「prepare 要预绑 vs C1 绑在链里」的次序死结,C1 全闭(三路统一每任务)。
- **不做**:动任何安全闸(path-lock/沙箱/审批)/ 改 execute 本体 / 改 create_and_bind 本体 / 手动挡语义。

## 一句话判据

**「是不是只:prepare 加 `chain_binds_per_task` mode(Some 变体=true)→ true 时跳 needs_binding、产 prepared·thread 延迟标记;false 时现状零变;+ 合流-new 退 S0 走 Some 变体;+ 下游读 `native_thread_id` 处都能容延迟空——而 path-lock/沙箱/审批/execute 本体/create_and_bind 本体/手动挡分支 0-diff、失败即停兜底在?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 prepare C1-aware mode(c4_c6·敏感核心·改这一处最要小心)

- prepare 输入(`prepare_input` 结构)加字段 `chain_binds_per_task: bool`(默认 false·serde default·旧调用零改);
- prepare 判定(c4_c6:167-196 那两处 needs_binding 分支):`chain_binds_per_task==true` 时——**不因「无绑定/rollout 缺」判 needs_binding**,改产 `prepared`,dispatch 记录 `native_thread_id`/`binding_id` 置 null + 加标记 `"thread_binding_deferred": true`(下游据此知道 thread 由链补)+ 审计新变体 `authorized_prepared_dispatch_thread_deferred`(透明·不吞);
- `chain_binds_per_task==false`(手动挡/existing/前端预 prepare)**逐字不变**(unbound→needs_binding);
- **绝不碰**:授权 active + 边界复核 approved 闸(1039/1312)、path-lock、沙箱、guard_result 授权检查(215)——这些照旧;C1 mode 只放宽「有没有会话」这一条就绪判定,**授权/安全一条不松**。

### 2.2 mode 信号接线(director_agent)

- `run_auto_advance_authorized_role_loop_with_timeout_budget`(1679)喂 prepare 前:`chain_binds_per_task = session_creator.is_some()`(Some=C1 自动路=true;None=手动挡/旧壳=false);
- 一处赋值·不新造第二套判断。

### 2.3 合流-new 退 S0 走 C1(director_agent·现在不死结了)

- `run_confirm_and_start_authorized_run_inner` 的 `"new" =>` 分支(2311-2352):**删 S0 单条建+绑**;
- 推进调用(2373)按 `session_choice` 分流:`new`→`_with_session_creator`(Some·每任务)· `existing`→现状 None(手动挡·**不动**);
- `new_session_notice` 改 C1 语义或移除(每任务说明由链侧给)。

### 2.4 下游容延迟空(核·别漏)

- 全仓 grep 读 `native_thread_id`/dispatch `binding_id` 的地方(read-model/上脸/审计/run-history):C1 mode 下这些可能是 null → **逐处确认能优雅容 null**(显示「会话由链内每任务建」之类·不 panic 不误判);
- **有读处硬依赖非空 → 停手报回**(别硬塞·这正是 S0 那类隐藏依赖的翻版·宁停勿猜)。

### 2.5 测试(逐个判·不一锅端)

- **6607 守卫必须仍绿·零改**:它走 None 公有壳 → `chain_binds_per_task=false` → unbound 仍 needs_binding。若它变红=改错了(C1 mode 漏了 false 路);
- **新断言**:C1 自动路 + 全新未绑工作流 → prepare 产 prepared(非 needs_binding)→ 链每任务建会话 → 跑完;失败即停仍成立(建会话失败→fail-stop·不派空);
- 合流-new 那 4 个 `texts.len()==1` 测逐个判(S0 拐杖→改每任务/失败无回落守卫→留·同前包纪律);
- path-lock(7311)/授权闸/reject 守卫**全绿未改**(git diff 自证)。

## 3. 安全死线

- **安全闸神圣 0-diff**:path-lock(7311)、沙箱、`授权 active+边界复核 approved`(1039/1312)、guard_result 授权检查(c4_c6:215)、existing 手动挡分支(2294-2309)——**一字不动**;
- C1 mode **只放宽「有无会话」就绪判定·不碰任何授权/安全**;放宽处必须有失败即停兜底(已立)——**若发现兜底有洞(某路径能派到空会话)→ 停手报回**;
- 真跑圈固定测试项目;memories 观察模式不加旗;`.codex` 凭据不碰。

## 4. 验收(重档仔细度)

- **对抗式自检(必答)**:「C1 mode 放宽 needs_binding 后,有没有任何路径让任务派到『没有真实会话』?」——逐条走:prepare 产 prepared(thread 延迟)→ 链 create_and_bind(建真会话或失败即停)→ execute 走节点绑定。**证明每一步要么有真会话要么 fail-stop**,写进回交;
- **6607 + 守卫全绿零改**(git diff 证 path-lock/授权/existing 分支未动);
- **新真跑**(`#[ignore]`·测试项目):全新未绑工作流走合流 new → 每任务建新会话跑完·codex home 见 N 条任务命名会话·**无 S0 孤儿**·无 needs_binding 空转;
- 下游容延迟空的核查清单(grep 结果 + 逐处结论);
- 三闸绿 + 冻结核/安全闸 0-diff 自证 + 计数不降 + fmt **自己真跑 `rustfmt --check` 别自报**(前两轮前科)。

## 5. 回交

- §4 全证据(尤其对抗式自检的逐步证明 + 下游 grep 清单)+ 安全闸未改 git diff + 落点清单 → 主导线核实物(**我会重点核:授权/安全闸真没动、失败即停兜底真无洞**)。**子线不 commit。**

## 7. 不接受为

- 碰任何安全闸(path-lock/沙箱/授权/边界复核)/ 放宽 needs_binding 时连授权判定一起放(只放「有无会话」这一条)/ 兜底有洞却硬上(能派空会话=红线·停手)/ 下游硬依赖非空 thread 却硬塞 null / 动 execute 本体·create_and_bind 本体·手动挡分支 / 留 S0 孤儿 / 一锅端改测试 / 自报 fmt 不真跑 / 做 existing 逐任务手动指定(非本包)。
