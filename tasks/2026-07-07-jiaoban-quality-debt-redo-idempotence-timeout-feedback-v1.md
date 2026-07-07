# 实现任务包:质量债·redo 幂等(重拆喂已完成事实) + 超时反馈边(自动打回主管重拆一次)· 主导线 → 执行线 v1

日期:2026-07-07　性质:**轻档**(后端为主·集中在 director_agent;单线;前端目标 0-diff)。两债一包——共用同一根管子(「已完成事实」喂重拆)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**。**子线不 commit。** 全程中文。
- **两笔债的真机案情(2026-07-06·盘上有账)**:
  - **债一·redo 叠加**:删怪单第一轮 task1 完成(怪 3→2),task2 超时链停;[接着跑]重拆——**主管不知道 task1 已做过**,新计划又含删怪,又删一个(2→1);再一轮 1→0。口供(「已删除第三个巡逻怪」)明明落库,没人喂给重拆。
  - **债二·超时死等人**:task2 跑 600.7s 撞 worker 上限被杀,链 fail-stop(**停是对的**:timeout≠抽风,原地重试只会再等十分钟)——但唯一出路是用户回来点[接着跑]。中间版 §0.2 主管职责原文含「决定继续、返工」,这活本该主管干。
- **主导线已核的接缝(直接用)**:
  1. **重拆点** = `director_agent.rs:1440` 调 `finalize_stale_chain_for_replan(:474)`——只在 re-plan 分支(`approved_planned_tasks.is_none()` 守着,首跑「所批即所跑」路径不经过);
  2. **口供读法** = B1 已建全套(`global_supervisor_agent.rs:94` 口供投影 struct + ~188 按 `worker_structured_report_recorded` 过滤 + 时间窗圈轮)——**复用别重造**:可见性不够就 fn/struct 改 `pub(crate)`(仅此改动·语义 0-diff);
  3. **喂料槽位先例** = `ProjectContext`(`consultant_agent.rs:49`)的 `memory_summary:58`——加法字段照它长;**死锚纪律照刀B**:`load_project_context` 保持纯装配,新字段由**调用方**(director 重拆分支)用手里的真 path 填,防回潜断言照抄;
  4. **timeout 判别语义已三分**(`director_agent.rs:302/316`):早退才 retry、timeout/gate 不 retry——债二**不改这个**,是在 fail-stop **之后**加反馈边;
  5. **信任级依据**:[接着跑,不用重批] 是用户拍过的(fix3)——已确认方案+active 授权下重拆不需重批;债二的自动重拆 = 同一信任级,只是省了那一下人肉点击,且**只给 timeout**。

## 1. 拍板摘要

- **要做的事**:① 重拆时把本单授权内已完成任务的口供事实喂给主管(「这些干完了,别重做」);② 任务超时导致 fail-stop 后,自动打回主管重拆**一次**(带预算/审计/授权复查),再败才停等人。
- **为什么**:债一到真实项目是数据损坏级的雷(Phase E 前必清);债二让「等得安心」名副其实——超时不再死等人。
- **代价**:一轮,集中一文件 + 一处加法字段。

## 一句话判据

**「是不是只:re-plan 分支多喂一个『已完成事实』prompt 块 + timeout fail-stop 后带预算(=1)自动走一遍现成 re-plan 路——而首跑所批即所跑路径 0 触碰、人闸/fix9 守卫/prepare guard/判决体/闸/retry 三分语义全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 债一·redo 幂等(喂已完成事实)

- `ProjectContext` **加法**字段 `prior_completed_summary: Option<String>`(serde/构造默认 None·`load_project_context` 纯装配不填——照 memory_summary 死锚纪律,防回潜断言同款加一条);
- **收集**(director 重拆分支·调用方填):读本 workflow、**本单授权时间窗内**(授权 created_at 之后——覆盖多轮叠加:A 完成 X、B 完成 Y、C 重拆要同时看到 X+Y)的口供投影 + 链记录完成态,拼人话行:`「任务标题」— did 首行(status)`;完成但没交口供的任务 → `「标题」—(无自述·执行态 completed)`;0 条 → None(现状);读失败 → None 不挡重拆(增益不是闸,同记忆召回先例);
- **喂**:主管拆任务 prompt 加块「--- 本单已完成(**别重复执行这些动作**·以下是已归档的 worker 自述) ---」;只进 **re-plan 分支**,首跑与批前预拆**不喂**(预拆无前轮·首跑走所批即所跑);
- **词表**:喂的是「做完了什么」的事实摘要(did/status/产物**文件名**可以)——**不搬产物内容本体**(「产物喂下一步」用户明令另批,别越);
- 单测:**复刻双删案**——旧链 completed「删除巡逻怪」+口供在 → 重拆 prompt 含事实块与禁令(stub director 断言);多轮累计(A+B 都在);0 口供 →「无自述」行;approved graph 路径 prompt 0 触碰(Bomb 类桩)。

### 2.2 债二·超时反馈边(自动打回重拆一次)

- 链 fail-stop 且停因 = **任务超时**(按现有 timed_out 执行态/停因判别·别新造分类)→ 若本次 auto_advance 的 `timeout_auto_replan_budget`(**=1,写死**)未用:
  1. 审计(`role_loop_auto_advance` 现有事件族·reason 写明「任务 X 超时,自动打回主管重拆(1/1)」——**不新开事件族**);
  2. **复查授权仍 active**(照起链双点复查先例)+ stop_requested 检查(用户点过停就不自动续);
  3. 走**现成** re-plan 路(finalize 旧链→重拆[自然带上 2.1 的已完成事实+超时事实一句:「任务 X 上轮超时被杀,考虑拆细」]→prepare→起新链)——**不复制路径,就是调它**;
  4. 新一轮**再** fail-stop(任何原因,含再超时)→ 照现有停·人话前缀「已自动重拆过 1 次」——预算耗尽,回到人(卡住脸按钮照旧·永不冻)。
- **只给 timeout**:供给类(额度死)/gate 拒/普通 failed **不走**反馈边(额度死自动重拆=白烧;gate 拒=该人看)——单测钉死;
- outcome/warnings 记「自动重拆 1 次」让交货/卡住脸自然显示(现有 warnings 渲染承载·**前端目标 0-diff**,实测需微调则报回)。

### 2.3 明确不做(§7 同)

产物内容喂下一步(另批);跨方案/跨授权去重(新方案重做=合法语义);timeout 之外任何失败的自动重拆;改 retry 三分语义或 worker timeout 上限;预算参数化(写死 1·要改另拍)。

### 2.4 文件边界(越界即停)

- 允许:`director_agent.rs`(重拆喂料+反馈边+自测 mod)/ `consultant_agent.rs`(**仅** ProjectContext 加法字段+防回潜断言·分流/档位/prompt/召回 0-diff)/ `global_supervisor_agent.rs`(**仅**口供投影 reader 可见性 `pub(crate)`·若不需要则 0-diff)/ `tests/` 若加离线断言 + 跑器 1 行;
- **0-diff**:c4_c6(prepare guard/登记机器)/ controller 本体(finalize 是 director 侧函数·controller 照旧只调)/ commands / runner / control_core / worker_report / secretary_agent / global_supervisor 两命令本体 / manual_relay / lib.rs / **前端全部**(目标)。

## 3. 安全死线

- 人闸/fix9 双守卫/prepare 逐任务钳与拒/path-lock/四护栏(runaway·可中断·审计·回滚)**一字不动**——反馈边在它们**下游**,每次自动重拆产生的新链照样全套过闸;
- 预算写死 1、复查授权、尊重 stop——**不许出现无人值守循环**;
- 喂料只事实不指令、只摘要不产物本体;fmt skip_children。

## 4. 验收

- **单测**(director 自 mod):① 双删案复刻(事实块+禁令进 prompt)② 多轮累计 ③ timeout→自动重拆 1 次→ran(stub 链)④ 重拆再败→停·人话含「已自动重拆过 1 次」⑤ 预算=1 不循环(两连超时只重拆一次)⑥ 非 timeout 失败(供给类/gate/普通 failed)不触发 ⑦ approved graph 首跑路径两件全 0 触碰 ⑧ stop_requested 时不自动续;
- **真跑**:timeout 难真造——stub 级为主如实标注;真机 = 用户日常使用中**自然遇到**超时看自动接续(交货/卡住脸见「自动重拆过 1 次」字样),不硬造;
- 三闸绿 + §2.4 0-diff 自证(前端 0-diff 用 `git status` 自证)+ 计数不降。

## 5. 回交

- §4 证据 + 落点清单 + 前端是否真 0-diff 实答 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 反馈边给了 timeout 之外的失败 / 预算可循环 / 绕过或复制 re-plan 路径另写一条 / 喂产物内容本体 / 动 retry 三分或 timeout 上限 / load_project_context 里填新字段(死锚家族!)/ 前端悄悄改了不报。
