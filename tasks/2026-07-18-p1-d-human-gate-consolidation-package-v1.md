# 任务包:P1-D 人闸收敛(批一下→自动到跑)v1

日期:2026-07-18 · 档位:**轻档·前后端**(高危 5 条零碰;安全闸零碰有勘察背书) · 执行者:执行线 · 上位:总执行计划 §一 P1-D(防跑偏总则 7 条先读,尤其第 2 条「人闸只有三下」)· 前置 P1-C 已收口(`01d107b`)· 勘察笔记(先读透):`/private/tmp/claude-501/-Users-yoyi-workspace-product-line/cb32c0e7-68c5-4963-b125-c9fd5640ab66/scratchpad/p1d-dataflow-recon.md`

## 勘察定性(写包依据,两条)

1. **「派发两连确认」不是 UI 确认点**:dispatch_prepared/started 两笔审计在同一命令内背靠背自动写(`workflow_run_dispatch_entrypoints.rs:1226-1227`),actor/reason 冒称「用户确认」——退场=改审计语义为真话,pilot 路同病一处改两路生效。
2. **安全边界干净**:全部确认点不在任何安全闸判定输入(S1 `user_rejected` 硬编码 false;授权快照 scope 每 prepare 重校+执行侧再验+path-lock 四层+guard+沙箱 argv 七层独立);**退场纯属流程减负,唯一缺口在记账不在权限**。

## A·干什么

1. **A1 绑定确认停点退场**:批准后**默认自动新会话直接进 prepare**——扩「预演图预选过会话时同击自动跳过」既有先例(`director_agent.rs:5080-5112`)到全路径;绑定停点(`:4428-4452`,后端注释自认「不是第二道审批」)摘除;**绑定校验失败回退路重设计**(勘察 §5:失败不再回「选对话」面→按人话进对话+replan 语义,禁死卡);「挑会话」能力件本包**只摘停点不删件**(归置属 P2-B)。
2. **A2 派发审计改真话**:两笔审计(JSON+DB 镜像共四处:`workflow_execution_entrypoints.rs:337-348/513-524` 族)的 actor_ref/permission_level/reason 改自动语义——照 `authorized_prepared_dispatch_created`(actor=`plan_authorized_prepared`·reason 引授权 id)先例;「用户确认…」冒称文案退场;改/新 event_type 同步 `ledger_entry_type_from_audit` 映射+白名单(否则带 risk flag,勘察 §4);**审计仍两笔不缺**;`workflow_audit.rs` 共用 helper 零碰。
3. **A3 批态旧件退场**:①说态卡 `JiaobanSayState` 死码收尸(定义+re-export+2 测试翻案);②【允许并开始】按钮四变体分支(`JiaobanAuthorizeStates.tsx:244-328`)收敛为单一渲染,**按钮文案零改**(感受件);③卡上修改框(`:222-233`)退场=只留常驻框 amendment 路由(双通道同源 `submitAmendmentText`,删 UI 不删函数);④`extractOpenQuestions` 开放问题正则止血件退场(07-16 `bb878d3`;P1-B 结构化 waiting_user 已替代;锁测试 `jiaoban-page-content-cleanup.test.tsx:280-324` 翻案);⑤「旧七态内容照常在」过渡断言(`jiaoban-conversation-center.test.tsx:484`)翻案。
4. **A4 验收路径**:①机器面:mock E2E=批准一下→自动经绑定/prepare/dispatch 到跑,中间零用户停点;审计序列打印(授权→prepare→dispatch 各笔在、语义真);②`cargo test --lib` 全量+**m5b/m5c 定向亲跑**(serde 前科,动审计文本必验对账);③四闸;④真机走查交用户:批一单看「批完直接跑,中间零弹窗零选择」。

## B·红线(违者停手报回)

1. S1 三支/guard 本体/`final_mark` 复核实证闸/写域锁定/path-lock/高危 5 条**零碰**——勘察已证退场不需要碰闸,diff 扫到闸文件=打回。
2. P1-C 刚验收的形态零碰:对话流/折叠/常驻框三通道路由/右区四视图;P3-C 占位件零碰;P1-E 殉葬品(`readonly_codex_consult` 调用族)零碰,退役候选清单不在本包执行。
3. 工作流页「说目标」表单(勘察 §3 件D)**不碰**,归属另议。
4. serde 双件前科:任何持久化字段增改 `default`+`skip_serializing_if` 齐;闸数字对应最终 diff,禁跑完闸再改码。
5. `ProjectJiaobanPanel.tsx` 现 1990/2000:本包属净删包,收口行数必须 <1990,报回注明前后行数。
6. 审计两笔「仍在且同笔进 DB+JSON」——改文本可以,改笔数/拆笔=打回。

## C·交付

1. 代码+mock E2E+翻案测试;确认点退场前后对照表(哪个停点/按钮/正则删了,替代语义是什么);
2. 10 项回传模板:cargo 全量+m5b/m5c 定向原文/typecheck 0/离线全绿(断言随形态)/gate 仓根 13/5/5 零净增/git status 前后;
3. 真机走查点清单(交用户批一单实测)。
