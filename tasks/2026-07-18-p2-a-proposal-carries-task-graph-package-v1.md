# 任务包:P2-A 终版方案自带任务图(拆任务一跳退场)v1

日期:2026-07-18 · 档位:**轻档·前后端+协议**(高危 5 条零碰) · 执行者:执行线 · 上位:总执行计划 §一 阶段二 P2-A(防跑偏总则先读) · 前置阶段一六包已收(`13edc99`) · 勘察笔记(先读透):`/private/tmp/claude-501/-Users-yoyi-workspace-product-line/cb32c0e7-68c5-4963-b125-c9fd5640ab66/scratchpad/p2-dataflow-recon.md`

## 背景(一句)

07-18 真单首跑:批准后的「主管拆任务」LM 调用是新链最大故障源(toolbox lint 重拆死循环卡用户 40 分钟)。P2-A 把任务图前移进方案——**你批的就是已质检的任务图**,批后拆任务一跳整个消失,「空任务列表」失败类随之灭绝。勘察证实「所批即所跑」引擎路已建成(`approved_planned_tasks` Some 路),本包本质=数据源换正本+三处 None 清零。

## A·干什么

1. **A1 方案 schema 扩任务图**:`supervisor_resident_turn.v1` proposal 变体加 `tasks` 字段(每任务:title/task_goal/acceptance_criteria/report_format/depends_on/角色——对齐 `ProjectDirectorPlannedTask` 契约);**勘察 §2 的 9 个同笔改点一个不落**(DTO/parse 白名单硬闸[第一咬人点:漏加=protocol_invalid 保守停]/schema prompt 两处/map/types.rs 持久化正本/store/TS 类型/前端);**serde 双件前科**(default+skip_serializing_if)+TS 字段名与 Rust 对齐(勘察点名 `objective` vs `task_goal` 暗雷,统一 task_goal);两个方案产出口(首问+问答后)同漏斗覆盖。
2. **A2 lint 前移**:`planned_task_toolbox_lint_reason` 在**方案 parse/确认时**跑(方案带任务图→lint 不过=protocol_invalid 人话上账,主管在方案期就被打回重出,用户看不到拆任务死循环);resident 出方案 prompt 搬入 `DIRECTOR_WORKER_TOOLBOX_FACTS`(:140-146 整段,教主管任务文本怎么写);「确认时重校」用现成件 `validate_approved_planned_tasks`(:1661,含 lint 重跑)——批准那一下重校整图。
3. **A3 三处 None 清零(勘察 §3 关键缺口,漏一处=病灶换入口复发)**:confirm 内层(director_agent.rs:5051/5065)+独立 `auto_advance_authorized_role_loop` 命令(:4737,[接着跑]通道)+超时递归(:4673)——全部改为「方案带图则用图」;方案无图(旧存量方案/schema 前数据)=fallback 走既有 plan 路**保留**(渐进换血,不许硬断旧方案)。
4. **A4 拆任务机制归置**:方案带图时 `director_plan_with_retry`/超时自动打回重拆/重拆事实块**不再触发**(代码保留给 fallback 路);「空任务列表」parse 坐标(:300)随主路退场,validate 兜底(:1667)保留;审计:`used_approved_plan_graph` 既有笔照旧,方案图路的审计序列在 mock E2E 里打印核对。
5. **A5 验收路径**:①mock E2E:方案带图→批准→重校→零 plan 调用直进 prepare→链,审计序列打印;带毒任务图(引用 read_file)在方案期被拦的案发测试;旧无图方案走 fallback 的回归测试;②连坐测试 20+ 逐名翻案/保留判定(勘察 §5 名单);③四闸(cargo 基线 988/0/44 口径按删测预登记制处理)+m5 定向;④真机:用户说一句→方案(含任务图,右区工序图=方案图)→批→直通跑完——**首单即测「lint 拦在方案期」的体感**。

## B·红线(违者停手报回)

1. S1(commands.rs:3190)/guard(c4_c6:1884)/`final_mark` 闸(勘察红线原文 414-417)/复核实证闸/P3 占位件/写域/高危 5 条零碰;P1-C/D/E 刚收形态零碰。
2. **schema 硬闸不松**:tasks 字段解析失败=protocol_invalid 保守停,禁降级采信;白名单逐字段加,禁通配。
3. 旧方案(无 tasks 字段)必须走 fallback plan 路——**禁止让存量方案变废纸**;fallback 路径回归测试必附。
4. serde 双件+m5b/m5c 定向亲跑;闸对应最终 diff;行数水线(Panel 1846/launcher 2977/director 7706)只减不增优先。
5. P2-B(挑会话归置)不在本包,`until_task_session_binding` 壳等塌缩候选**零碰**(留 P2-B 判)。

## C·交付

1. 代码+mock E2E+案发测试+fallback 回归;三处 None 清零对照表;
2. schema 9 改点核对清单(逐点勾);连坐测试处置表(翻案/保留各列);
3. 10 项回传模板(cargo 按预登记口径/m5 定向/typecheck/离线/gate 仓根 13/5/5);真机走查点清单(交用户,一单到底)。
