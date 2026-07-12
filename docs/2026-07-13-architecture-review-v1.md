# 系统性架构评审 v1(2026-07-13)

> 方法:六域并行映射(读遍 197 文件/137,777 行·全部成功)+ 总指导亲手核关键高危项。批评/对抗核实阶段因额度中断,故本报告严格区分**【已核】总指导亲验**与**【map声称】映射发现但未对抗核实**。评审只读,评审后 WIP 指纹核对无越界。
> 底数纠正:Tauri 命令实测 **137**(非摸底所报 235);store 线上 236MB(其中 ~221MB 是备份)。

## 一、总评(诚实版)

**安全纪律极扎实,但背着两笔在滚雪球的结构债,且 god file 开始侵蚀"核实物"本身。**

- 护栏是真的、多层的、测试钉死的:path-lock 单常量闸 + 每节点 S1 强闸 + runner 拒缺沙箱 + argv 双层沙箱 + 授权每跳回链核验 + fail-closed 进程回收 + 原子写/备份/回读校验。比多数团队项目更严。终标"确定性优先零 LM、断供保守停而非假过"、retry"预算1不空烧"——失败姿态诚实,这是这套东西最可信的地方。
- 但两个 **root pattern** 反复辐射出全库一半的 smell(见 §三)。
- 而且 director_agent.rs 7470 行 / ProjectJiaobanPanel.tsx 3845 行已逼近"主导线逐行核实物"的人力上限——**核实物是这个项目质量的命根子,god file 侵蚀的正是这个机制本身**。

## 二、真实优点(不奉承,择硬列)

1. 终标确定性优先 + 断供保守停(director_agent.rs:2818-2822, 3066-3135)。
2. retry 纪律统一:三处重试全预算1、不循环、显式排除供给失败。
3. 纵深防御层层独立,单点被绕仍有下层(执行域+主管编排域双证)。
4. 正确的并发模式**库内已存在**:新 sidecar 有 revision CAS + 文件锁 + 损坏拒覆盖(plan_authorization_store.rs:41-47)——只是主 store 没回填。
5. 主管编排"权限不经模型"做穿了:MCP 面无副作用工具、allowed_write 宿主自取、fresh-session 三重一致校验(supervisor_orchestrator.rs:356-473)。
6. 记忆"不自动转正"是四处独立机器强制,不是约定;R3 17 表已建好并与 JSON 对账过,切库是翻闸非重写。

## 三、两个 root pattern(比任何单条 smell 都重要)

### A. 单 JSON 热状态文档 —— 辐射存储域一半 smell【已核】
主 store `workflow-state.v0.json` 4.7MB 单文件全量读改写、**读无锁、写只锁 rename 瞬间、无 revision CAS**(workflow_state_store.rs:13/103-125,validate 无冲突检查),而 app 与主管 MCP 子进程双进程同写(supervisor_orchestrator.rs:611-666)。它同时是:
- **并发丢写源(P1)**:后写者带旧快照整本覆盖,前一笔 binding/audit 无声丢失——主管试点(UI 操作与 supervisor 步循环并行)正是触发场景;
- **UI 卡顿源**:每任务约 8 次 4.7MB parse/serialize,108 个命令全同步签名;
- **备份爆炸源**:audit_events(1447条)+dispatches 内嵌热文档、只增不减,每写整本 copy,221MB/236MB 是备份,半年按曲线进 GB;
- **R3 迟切的税**:正确解法你库里有,主 store 却用最弱保护。
- **修法方向**:主 store 回填 sidecar 已有的 revision CAS + 文件锁;audit/dispatch 正文外置进 artifacts 只留引用;这一条修了,存储域大半 smell 一起消。

### B. 安全谓词/契约/常量靠人肉同步 —— 每处都是未来失守缝【部分已核】
契约文本双正本(1条测试兜底)、3b 闸 4 处 copy-paste、审批绕过黑名单 2 份、根路径常量 ≥3 处写死、前后端类型 ~6800 行手工镜像、行为分支押在后端中文文案上。全靠"测试断言"或"人肉纪律"防漂移。**已吃过一次亏**:弱版 C3 授权入口(record_global_boundary_review 弱版仍注册为命令,map称是授权链旁路)"改严版忘弱版"发生过。
- **修法方向**:安全谓词单一真源(谓词函数收一处、多处调用);契约/常量 codegen 或 include 单源;前端行为别读后端文案,读结构化 code。

## 四、CONFIRMED 高危(总指导亲核)

| 严重度 | 发现 | 证据 | 爆炸半径 |
|---|---|---|---|
| **P1** | 主 store 无锁无 CAS 并发丢写 | workflow_state_store.rs:13,103-125 | 主管试点触发丢 binding/audit |
| **P1** | launcher 裸 txt 污染 store 根,与 R3 preflight 拒非 json 打架 | supervisor_session_launcher.rs:1113→store 根;workbench_sqlite_preflight.rs:178,270 | 今天真跑 R3 切换 preflight_blocked;文件名带冒号不可移植 |
| ~~catch~~ 撤回 | h5:44 fail-open 债**已还为 fail-closed** | h5_project_dispatch_bridge.rs:44 明文注释 | **无漂移**:CURRENT §二已正确标"站3a 前置修成 fail-closed"。评审初判"文档仍挂待修"基于摸底旧 FACTS,核实物纠正——记为「核实物抓评审自己假 catch」 |
| **catch** | Tauri 命令实测 137 非 235 | rg #[tauri::command] 137(commands 108+mcp 10+agents 19) | 治理覆盖率结论按 137 修;摸底命令算错 |

## 五、Map 声称、未对抗核实(诚实标注·待验)

以下来自映射,总指导**未亲核**,涉 WIP 或需深读,列清单不下定论:

- **【最该先验】follow_up_worker 追问结果读不回来**(主管编排P1,supervisor_orchestrator.rs:314-330,959-1013):map 称追问输出写进无人消费的 txt、read_worker_report 命中旧缓存、v7 零 follow-up 从未真跑证伪。**若成立,主管编排的核心卖点(追问)是坏的**——涉 WIP 文件,应作为**站 2 试点第一验**:专门跑"派工→追问→读回追问结果"一单。
- 弱版 C3 授权入口平行旁路(治理P1,plan_authorization_store.rs:225-312)。
- 一次派发横跨 3+ 无事务 JSON 账本,一致性靠调用顺序(主管编排P2)。
- binding_id 截断碰撞是模式病、同类生成器仍在新代码用(主管编排P2,3a 已修一处但 stable_id 截断仍广泛用于 event_id/reservation_id)。
- 前端交办组件 46 useState/相位机巨石,与"简单性原则"最相悖(前端P1);离线测试对编排组件结构性盲区(前端P1)。
- 记忆双召回面闸门不等价:top5 召回绕过 lint/过期/冲突(记忆P2);候选值密度无过滤,真用必淤积模板候选(记忆P2)。
- 中文 token 预算低估 4 倍、召回无排序无 recency(记忆P3)——评"切库收益"时勿高估(sqlite 也是 blob,翻闸不自带检索升级)。

## 六、给主人的三句战略话

1. **修 root pattern A 比修任何单条都值**:一次回填 CAS,消掉并发丢写+备份爆炸+一半卡顿,还给 R3 切换扫清 preflight 障碍。
2. **站 2 试点第一件事,验 follow_up 读回**——别带着可能坏掉的核心卖点往 3b 走。
3. **god file 拆分不是洁癖,是护命根子**:director/Jiaoban 大到人核不动那天,"核实物"就名存实亡了;拆到可核尺寸,是给你自己的质量机制续命。
