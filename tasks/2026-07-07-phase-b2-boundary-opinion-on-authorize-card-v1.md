# 实现任务包:B2·全局主管批前边界意见(授权卡上「这方案对不对得上你的目标」)· 主导线 → 执行线 v1

日期:2026-07-07　性质:**轻档**(B1 同族扩展:agent 模块加半边 + store 加法 + 批脸小块;单线双面·文件边界 §2.5;死线 0-diff)。Phase B 第二片,正本 `decisions/2026-07-07-phase-b-advisory-supervisor-and-secretary-v1.md`。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(单线双面)。**子线不 commit。** 全程中文。
- **背景**:B1 已落全局主管**跑后**结果复核(d1ff9f0 前一发·真机过);B2 = 它的另半边钩点——**批前**边界意见:方案出来、用户批之前,全局主管读方案给一句「范围/目标对不对得上」的人话意见,上授权卡。中间版 §0.6 第 3 步(全局主管复核方案边界)advisory 形态落地。**活广告刚发生**:07-07 纯建议方案事故,批前扫一眼就能点破「你要动手,这方案不改任何文件」——fix9 是确定性堵(已落),B2 是智能提醒层。
- **意见不是闸(定稿第 1 条,照 B1 同款)**:不拦批、不驱动;缺席不挡批;词表禁「审批」。
- **主导线已核的接缝(直接用,全是 B1 现成家底)**:
  1. agent 家 = `global_supervisor_agent.rs`(readonly consult 通道/契约提取/保守归一化/provider 失败分类/幂等骨架全在——**照抄结构加半边**,别另起炉灶);
  2. store 家 = `global_supervisor_review_store.rs`(sidecar·原子写/备份/损坏跳过)——**加法扩展**:新 `boundary_reviews` 集合按 `proposal_id` 存取(schema 版本加法·旧 `reviews` 集合语义 0-diff·loader 对缺字段容忍);
  3. 方案数据 = proposal store 现成只读(goal/summary/proposed_steps/scope_draft/risks 全在记录里);
  4. 前端缓存先例 = worksmap 预拆「按 proposal_id 缓存」(fix7/刀2-UI);invoke 封装照 `lib/tauri.ts` 家族。

## 1. 拍板摘要

- **要做的事**:批脸自动出一份边界意见(async·不挡批·按方案幂等),点破「目标 vs 方案」错配、越界苗头、风险漏报。
- **为什么**:第 3 步 canon 落地;纯建议事故的智能层;「少懵」——批之前有人替你看一眼。
- **代价**:一轮。后端半边 agent + store 加法 + 一条命令;前端批脸一小块。

## 一句话判据

**「是不是只:agent 模块加批前半边(读盘→只读 consult→意见落 boundary_reviews+审计)+ 批脸意见小块(async·缺席不挡批)——而 B1 结果复核路/合流守卫/分流/档位/闸全 0-diff、意见不驱动任何状态?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 后端·`global_supervisor_agent.rs` 加批前半边

- **边界档案(新 prompt 常量·独立于结果复核档案)**:角色=全局主管·批前边界复核(不是审批者);输入=用户目标 + 方案要点(proposed_steps)+ scope_draft(写根/角色/工具/checks)+ risks;检查四件:① **目标与方案对不对得上**(用户要动手 vs 方案纯建议/写根空 = 07-07 事故形态,必须点破)② 范围苗头(方案步骤里出现测试项目之外的路径/写 `~/.codex`/push/删除不可逆等字样 → 点名)③ 步骤与验收齐不齐 ④ 风险漏报。保守:拿不准说拿不准;全中文人话、短。
- **输出 schema**(serde 全 default):`{ verdict: "looks_ok"|"mismatch"|"caution", points: ["短句"...], summary }`;归一化保守向(未知/审批腔 → caution)。
- **命令 `run_global_supervisor_boundary_review`**(registry +1):入参 project_root(+state_path 惯例)/proposal_id/force;**幂等**:该 proposal_id 已有记录(含 unavailable)且 !force → 直接返回不 consult;流程=盘上读 proposal → 组 prompt → `readonly_codex_consult` → 提取 json → 落 `boundary_reviews` + store 内嵌审计事件 `global_supervisor_boundary_review_recorded`(照 B1 内嵌先例——**升档给闸权那天回归正典 audit_events 的前置已记档,B2 沿用不改辙**);失败三分照 B1(供给类人话/解析失败落 unavailable 可重试/任何失败不 Err 断面板);记录带 model/profile_version。
- 测试进模块自己的 `#[cfg(test)] mod`(照 B1):见 §4。

### 2.2 前端·批脸「全局主管意见(边界)」小块(`ProjectJiaobanPanel.tsx`)

- **触发**:批脸挂载/方案切换时**自动** invoke——**只对「今天生成的 pending 方案」触发**(stale 方案不触发=省额度;纯建议方案照常触发,它点破 mismatch 与 fix9 警条互证);结果按 proposal_id 缓存(照 worksmap 先例·重挂载先读缓存);
- **四态**:loading 小字「全局主管正在看边界…(意见没到也可以先批——它不拦事)」→ 意见(verdict 人话行 + points 列表·mismatch 用告警色调·looks_ok 一行绿)→ 不可用(一行小字 + [重试]=force)→ 无方案零渲染;
- **位置**:方案要点之后、按钮区之前;**不挡不拦**:按钮区行为一概不变(fix9 的纯建议改道/正常主按钮全原样);
- **词表**:「全局主管意见/边界意见」,禁「审批」;不露 proposal_id/黑话;
- **并发注**:批脸可能同时在跑 worksmap 预拆(另一条 tier-1)——两者都是只读 consult、互不依赖,并行属预期;若真机观察到 runner 层互踩,如实报回别硬修。
- invoke 封装 `lib/tauri.ts`、类型 `lib/types/*`(加法)。

### 2.3 明确不做(§7 同)

意见驱动任何行为(拦批/自动改方案);结果复核路(B1)任何语义改动;秘书面(B3);对 stale 方案自动烧额度;把边界意见喂给主管拆任务(另议)。

### 2.5 文件边界(越界即停)

- 允许:`global_supervisor_agent.rs`(加半边)/ `global_supervisor_review_store.rs`(**加法**:boundary_reviews+审计,旧集合语义 0-diff)/ `command_registry.rs`(+1 注册)/ `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `lib/tauri.ts` / `lib/types/*`(加法)/ `tests/` 新离线 DOM 文件 + 跑器 1 行;
- **0-diff**:director_agent(含 fix9 守卫/合流/auto_advance)/ consultant_agent(分流/档位/prompt)/ c4_c6 / controller / commands / codex_local_runner / control_core / worker_report / manual_relay / 两执行 store / lib.rs。

## 3. 安全死线

- 批前意见**结构性只读**(readonly consult);唯一写=自己 store 的 boundary_reviews+内嵌审计;**批的手永远是用户**(按钮区 0-diff 是硬线);
- 渲染类真机过;fmt skip_children。

## 4. 验收

- **单测**(模块 mod):schema 三态+保守归一化;boundary_reviews 往返+旧 reviews 集合不受扰(加法自证);幂等命中不重跑(stub 计次=1)/force=2;供给类人话;
- **真跑**(`#[ignore]`·额度在):对着盘上真方案出 grounded 边界意见——**最佳夹具=16:53 那份 user_confirmed 纯建议方案**(目标要动手 vs 写根空),意见应点破 mismatch(=事故的智能层复现,B2 的 money shot;LM 没点破就如实记,别硬造);
- **离线 DOM**:四态断言 + stale 不触发 + 词表无「审批」+ 按钮区两态(fix9 改道/正常)不受意见块影响;
- **真机(用户)**:出一份方案 → 批脸自动见「正在看边界…」→ 意见上卡(先批了也不碍事);重进批脸不重烧;
- 三闸绿 + §2.5 0-diff 自证 + 计数不降。

## 5. 回交

- §4 证据(真跑意见原文必带)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 意见拦批/改按钮行为 / 动 B1 结果复核语义或旧 reviews 集合 / stale 方案也自动烧 / 不幂等 / 词表「审批」/ 失败 Err 断批脸 / 另起第二套 consult 通道。
