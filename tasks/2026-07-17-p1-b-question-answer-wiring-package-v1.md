# 任务包:P1-B 问答接线(问一句-答一句-出方案)v1

日期:2026-07-17 · 档位:**轻档·后端+协议**(仓内改码;不碰高危 5 条;前端 UI 归 P1-C 不碰) · 执行者:执行线 · 上位:总执行计划 §一 P1-B(防跑偏总则 7 条先读)· 前置 P1-A 已收口(`deae9ce`)· 数据流勘察笔记(先读):`/private/tmp/claude-501/-Users-yoyi-workspace-product-line/cb32c0e7-68c5-4963-b125-c9fd5640ab66/scratchpad/p1b-dataflow-recon.md`

## 背景与载体拍板(一段)

病历二号:用户的答复没有任何通道进链。勘察证实:`RequestUserDecision` 只活在**旧 pilot**(exec 步进循环,`waiting_user`=run 终态整圈死,答复零通道,同 run 幂等回放旧问题);**P1-A resident 常驻会话今天发不出反问**(回合产物被 `parse_consultation_proposal` 硬约束为全量方案 JSON,反问无 schema 位;MCP 面只读三件、七动作刻意绕开)。**载体拍板:P1-B 全部长在 resident 常驻会话上;pilot 的 RequestUserDecision/waiting_user 面零改(随 P1-E 退役)。**「问一句」=回合末 schema 的新「问题」变体,不是 pilot 那个动作的搬家。

## A·干什么

1. **A1 回合 schema 开「问题」分支**:resident 回合产物=严格二选一——`方案 JSON`(现状硬闸照旧)或 `主管问题`(新变体:question 原文/question_id/所属 workflow+project/round 序号;严格 schema,两者都解析不出=保守停不猜,tier-1 输出不稳警报器条款)。parse 层双分支落在 P1-A 的 parse 硬约束处(勘察 §2 坐标)。
2. **A2 问题→对话消息(读模型可见)**:问题以 canonical 审计形落 workflow state 派生源(先例 `director_agent.rs:3931` 一族)→**扩黑板派生函数**新增「主管消息/主管问题」kind(勘察 §3:`workflow_read_model_entrypoints.rs:204-439` 六 kind 之外+1)→ 读模型可查(前端 P1-C 接;本包验收=读模型能列出待答问题)。**落对派生源=换代注入自动携带问答事实**(resident 换代吃同一读模型),这是选这个落点的核心理由,别绕开。
3. **A3 答复通道(新 tauri command)**:按 `run_project_consultation` 先例(spawn_blocking·`command_registry.rs` 宏注册+`tauri.ts` wrapper):入参 {project_id, workflow_id, question_id, answer_text} → 校验该问题存在且未答(幂等:已答重复提交=拒绝返回既有结果,禁静默重注入)→ `prompt_kind` 白名单新增 `user_reply` 一种(勘察 §2:`supervisor_resident_session.rs:748-753`)→ `codex-reply` 注入同 threadId 续跑(唤醒=对 `consult_supervisor_resident` 再调,勘察已证半边现成)→ 回合产物再走 A1 双分支(可再问,可出方案;方案出=确认时重校契约照旧)。
4. **A4 换代兜底**:答复到来时宿主已死/thread 失效 → 走 P1-A 既有换代(事实注入含 A2 落的问答消息)后注入答复续跑;禁把答复丢弃或降级成重开整圈。
5. **A5 审计每步留痕**:`…question_asked` / `…question_answered` / `…reply_injected`(+换代路径沿用 P1-A 四类)——各自独立 event_type,走 M5 `update_store`。
6. **A6 验收路径**:①mock 离线状态机:问→答→续跑→出方案;重复答幂等;двойное问(连环问两轮)②固定测试项目真跑 ignored 测试(P1-A `p1_a_live_…` 模式先例):真模型「问一句-答一句-出方案」闭环,threadId 全程一致,审计序列打印③四闸。

## B·红线(违者停手报回)

1. **不冒充用户**:`user_reply` 只能源于新 command 入参(真用户动作);resident/pilot/任何 LM 路径禁自产答复注入;测试里 mock 用户答复要显式标注 test-only。
2. **schema 硬闸不松**:问题变体同样严格 schema;解析失败=保守停+人话上账,禁猜、禁降级成自由文本采信。
3. pilot 面零改;blocked followUp 框(P3-C)零碰;前端组件零碰(读模型可见即达标);guard 本体/`final_mark` 实证闸/S1 三支/写域锁定/高危 5 条零碰。
4. **serde 前科点名**(P1-A 核复拦截案):任何进持久化的新字段必须 `#[serde(default, skip_serializing_if = …)]` 双件齐;收工前亲跑全量到 **0 failed**,基线 **982/0/46 只增不减**,闸数字必须对应最终 diff(禁跑完闸再改码)。
5. 新 sidecar 禁;写点全走 M5 显式桥(DB 主写+投影同笔;22 表口径不破);不新增用户确认点/提示牌(防跑偏总则 1——答复框是回话不是新闸)。
6. 幂等键前科(勘察 §1):同 run 同问题的幂等回放路径别复用 pilot 的键法,新通道自设 question_id 级幂等。

## C·交付

1. 代码+离线测试(mock 状态机全绿)+真跑 ignored 测试原文(问一句-答一句-出方案·threadId 一致·审计序列);
2. 读模型新 kind 的形状说明(P1-C 写包直接引用);prompt_kind 新增与 schema 问题变体的契约说明(一页);
3. 10 项回传模板:shape gate 三数原文(基线 13/5/5 仓根跑·新 [error] 零容忍)/cargo 982/46 口径只增不减/typecheck 0/离线套件全过/不 commit;被闸拦过的事如实列。
