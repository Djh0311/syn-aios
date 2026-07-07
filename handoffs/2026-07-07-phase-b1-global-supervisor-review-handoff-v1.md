# 回交:B1·全局主管读口供出复核意见(advisory)· 执行线(单线双面)→ 主导线 v1

日期:2026-07-07 · 包:`tasks/2026-07-07-phase-b1-global-supervisor-review-on-reports-v1.md` · 决策:`decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md`。**子线未 commit。** 轻档。

## 一句话结论

交货后自动出一份全局主管复核意见(每任务点评·黄牌必评·总判+建议动作·async 不挡交货·按轮幂等),真跑一次通过且意见 grounded——**正好把包 §1 点名的那单「无法启动浏览器·未完成手动验收」假完成复核成 issue,还抓到「4 节点只有 3 份口供」**。链/闸/判决体/口供登记机器/workflow state 全 0-diff;复核不驱动任何状态。剩真机验收(§6)。

## 0. §0.3 口供落库点核查实答(stop-gate·包要求先做)

- **落库点 = workflow state 本体(`workflow-state.v0.json`)的 `audit_events` 数组**,event_type=`worker_structured_report_recorded`(c4_c6:404 push;链路径经 `worker_report.rs:104` best-effort 落)。**不是 observation_store**——那是「过程事实确认」环节用户确认后才写观察的另一家店(c4_c6:485+)。
- **现成只读口(有,故没停)**:`read_workflow_state_value`(全 crate state loader)+ 既有数组投影——过滤逻辑与 c4_c6 自家 `has_worker_report_for_workflow`(1101)/`evidence_refs_for_c5`(1476)同构;链轮 = `workflow_chain_runs` 数组按 (workflow_id, project_id, started_at) 精确匹配(controller `latest_chain_run_for` 同构、不用"最新一条"防串轮);任务节点 = 顶层 `nodes` 的 `{wf}:node:task:` 前缀(刀2 落的)。「本轮口供」圈法 = workflow_id + `created_at ∈ [链 started_at, ended_at]`(两者同源 `unix_timestamp_string()` 毫秒串,parse i64 可比)。**未自造第二套存取。**

## 1. 建了什么(落点清单)

**后端**(两新文件 + registry 两处加法):
- `global_supervisor_agent.rs`(新):档案常量(角色=复核最终结果·不是审批;黄牌必评;保守判 needs_human_check;全中文)+ 契约段(照 `WORKER_REPORT_CONTRACT_TEXT` 风格·最后仅输出一个 json 块)| 输出 schema serde 全 default + **保守归一化**(overall 未知→needs_human_check、action 未知→none 不给错按钮、verdict 未知→issue)| `load_review_input` 读盘组输入(方案=proposal store 该 workflow 最新 UserConfirmed;链轮精确匹配;口供时间窗;任务节点计数——**prompt 里写明「N 节点 M 口供,数对不上=有任务没交」**)| 单字段 clip 截断防 prompt 爆炸 | `run_global_supervisor_review_core`(consult 可注入·单测 stub 计次)| 命令 `run_global_supervisor_review`(spawn_blocking·consult=现成 `readonly_codex_consult` 420s)。
- `global_supervisor_review_store.rs`(新·sidecar `global-supervisor-reviews.v1.json`):按 (workflow_id, chain_started_at) 存取;原子写(tmp+rename+sync)/写前备份+prune/revision 递增全照 `plan_authorization_store` 先例;**损坏跳过** = load 损坏→空店+人话 warning(不断面板),写盘时坏文件先进 backups/(尸体保留);记录带 **model/profile_version**(§10-1 零成本半边)。
- `command_registry.rs`:`mod` 声明 ×2(照 worker_report 借道先例·lib.rs 0-diff)+ handler 注册 1 行。
- **失败三分**:供给类→`readonly_codex_consult` 内建 fix8 前缀,剥 `codex_provider_unavailable:` 直取人话;consult 失败/没按契约→落 status="unavailable" 记录(可重试);链轮不存在→unavailable 但**不落盘**(键不对落了是垃圾);**全部路径不 Err 断面板**(返回结构带 status)。
- **幂等(成本护栏)**:同键已有记录(**含 unavailable**)且非 force → 直接返回不 consult;[重试]/[重新复核] 才 force=true。

**前端**(Panel + css + tauri.ts + types 加法):
- 触发:交货翻脸(phase==="done")→ 自动 invoke(fire-and-forget);定位键=本轮链 `started_at`,**ran 直翻 done 轮询已停时用现成 `getProjectWorkflowChainStatus` 补拉一次并按 fix6-v2 同口径校验(≥runStartedAtMs)防拿旧轮**;拿不到键→区块零渲染。结果态缓存进 JiaobanRunCache(重挂载先读缓存/无则按幂等键补拉·后端秒回);新一轮开跑(authorizeAndStart/continueRun 进门)清上轮复核。
- 展示:`JiaobanSupervisorReviewSection`(纯展示·无 hooks·export 供离线 DOM)四态——loading「全局主管复核中…(约 2-7 分钟,不影响交货)」/ 意见到(总判行 pass绿|建议打回|建议亲验 + 每任务点评 issue 带⚠ + `replan`→[按建议打回重拆]走现成 backToSay、`human_verify`→显 human_note、`none`→无按钮 + 常驻[重新复核]小链接)/ 不可用(人话+[重试复核] force·绝不零出路)/ 没起→零渲染。
- 词表:「全局主管意见/复核意见」,全文无「审批」(离线测试断言);不露 thread_id/store 黑话。
- CSS:`jiaoban-supervisor*` 纯加法;配色用**双主题真变量** `--accent`/`--warning`(核过 styles.css 两主题都有;`--warn`/`--ok` 单边或不存在,没用——fix7 幽灵变量教训)。
- **前端不转述内容**:invoke 只传 project_root/workflow_id/chain_started_at/force 四个定位参。

## 2. §4 机器证据

- **单测**(两新模块自己的 `#[cfg(test)] mod`·不进 lib.rs):10/10 绿——schema 三态(合法/缺字段 default/坏 json→不可用记录)|store 往返+同键替换+revision 递增+created_at 保留|损坏跳过+坏文件备份尸检|缺幂等键拒|**幂等命中 stub consult 计次=1·force=2**|供给类人话(前缀剥净)+unavailable 幂等+force 恢复覆盖|坏回包落记录/错轮不落盘|读盘时间窗圈轮(窗外旧口供不进)+prompt 素材断言。
- **全量**:`cargo test --lib` = **690 passed / 0 failed / 40 ignored**(接手基线 680/0/39 + 本包 10 单测 + 1 ignored 真跑;计数不降)。
- **离线 DOM**(新 `tests/global-supervisor-review-section.test.tsx`·跑器+1 行):6 组全过——loading/意见到(含 replan 按钮**点击回调直调验证**)/不可用+重试回调/pass 绿行+human_verify 注/零渲染/**词表断言(无「审批」·无 store 黑话)**。offline 全套 15 passed。
- **三闸**:tsc 绿 / offline 绿 / build ✓。fmt:本包 3 个 rs 文件 `--config skip_children=true` 净(录得手动跑过、check 复核 CLEAN)。

## 3. §4 真跑证据(`global_supervisor_review_real_run`·22.76s·可按名重跑)

命令:`cargo test --lib global_supervisor_review_real_run -- --ignored --nocapture`

- 对真 store 最近一轮已收尾链(07-06 减怪物那单·workflow `…mario-test:default`·started 1783395278635)真 consult 复核:**status=ready,overall=needs_human_check,action=human_verify**。
- **意见 grounded(逐条对得上真口供)**:①「basePatrols…只有 1 个怪物」= 真口供原词 ②减怪物单判 ok ③**浏览器验收单判 issue**——正是包 §1 点名的「无法启动浏览器·未完成手动验收」假完成,现在「有人读了、给了说法」④ **「任务列表 4 个节点只收到 3 份口供」**——没交口供的黄牌被点名(prompt 计数设计生效)。总评保守不装确定。
- **实物独立核过(不信测试自报)**:sidecar 真建(12:27·schema v1·revision 1·记录字段全·model=codex-cli-default·profile=global-supervisor-profile.v1·内嵌审计 1 条 `global_supervisor_review_recorded`);`.codex/auth.json` mtime 仍 Jun 3(凭据没碰);**workflow state 本体 mtime 11:41 < 复核时刻 12:27 = 复核全程没写它一个字节**(「唯一写=自家 store」硬证)。

## 4. 0-diff 自证(§2.5 全名单)

`git status` 改动面 = 允许名单精确吻合:新 2 rs + `command_registry.rs`(mod×2+注册 1)+ `ProjectJiaobanPanel.tsx` + `projectWorkflowSidePanel.css` + `lib/tauri.ts` + `lib/types/workflow.ts`(尾部纯加)+ 新离线测试 + 跑器 1 行。死线逐一 `git diff --stat` 空:**c4_c6 / workflow_chain_controller / commands / codex_local_runner / control_core / director_agent / consultant_agent / worker_report / manual_relay / lib.rs 全 0-diff**。复核不驱动:后端无任何 verdict→状态写路径(唯一写=自家 sidecar,运行时 state 文件 mtime 为证);前端按钮全走现成动作(backToSay/重试)。

## 5. 设计取舍报备(2 条·主导线可否决)

1. **审计放 store 内嵌**(照 plan_authorization_store 先例)而非 workflow state 的 audit_events:包字面「落新 store + 审计事件」两者都满足,内嵌版让复核运行时对 state 文件零写入(mtime 硬证)、比借 director_agent 的 append helper(source_kind 写死 role_loop 语义不对)干净。审计事件名照包 `global_supervisor_review_recorded`。
2. **「所批方案」取该 workflow 最新一条 UserConfirmed**(链记录里没存 proposal_id,不碰 controller 加字段):交办产线「每单一条工作流+fix4 re-plan supersede」下同轮方案即最新已确认;万一没找到,prompt 老实注明「缺方案对照」照常复核。

## 6. 真机待验(§4·用户·我做不了)

1. 跑一单到交货 → 交货脸下方**自动**出现「全局主管复核中…(约 2-7 分钟,不影响交货)」→ 数分钟后意见上脸(总判+每任务点评);
2. 造/遇一单黄牌 → 黄牌任务有点评;若建议 replan → [按建议打回重拆] 点击真回「说」面;若建议亲验 → 显一句「建议你亲验:…」;
3. 切 tab 走再回来 → 意见还在(缓存/幂等补拉·不重烧);[重新复核] 才真重跑;
4. 断网/额度死一次 → 「复核不可用(人话)」+[重试复核],交货内容不受影响;
5. 词表肉眼扫:无「审批」、无 thread_id。

## 7. 回交动作

§4 证据(单测+真跑+实物)+ §0.3 stop-gate 实答 + 0-diff 自证 + 取舍报备如上 → 主导线核实物。**子线不 commit。**
