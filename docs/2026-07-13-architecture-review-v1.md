# 系统性架构评审 v1（2026-07-13，处置中）

> 资料状态（2026-08-09）：日期和版本受限的历史评审，正文中的“处置中”和未提交状态只反映当时现场。当前架构看 `docs/workbench-system-architecture-v1.md`，当前实现看 `docs/current-state.md`、源码和新鲜验证。

> 方法：六域并行映射（读遍 197 文件/137,777 行·全部成功）+ 总指导亲手核关键高危项。初稿严格区分**【已核】总指导亲验**与**【map声称】映射发现但未对抗核实**；07-13 后续处置与实数复核见 §七。
> 当前状态：`WIP_NOT_COMMITTED`。Tauri 命令实测 **137**（非摸底所报 235）；3b 真跑后主 JSON 已增至 **5,897,201 bytes**；主备份 45 份，`backups/` **233,520 KiB（约 228 MiB）**。

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

### A. 单 JSON 热状态文档 —— 辐射存储域一半 smell【已核·部分处置中】
主 store `workflow-state.v0.json` 已由初审时 4.7MB 增长到 5,897,201 bytes，仍是单文件全量读改写。初审确认它**读无锁、写只锁 rename 瞬间、无 revision CAS**；07-13 WIP 已在锁内增加 revision CAS，把静默覆盖改成明确 `workflow_state_revision_conflict`，但它仍同时是：
- **并发冲突源（原 P1 静默丢写已止血）**：旧快照不再覆盖新快照，但调用方还没有业务级自动重放，真实并发会转成显式失败；
- **UI 卡顿源**：每任务约 8 次约 5.5MiB parse/serialize，108 个命令全同步签名；
- **备份爆炸源**：audit_events（现 1473 条）+ dispatches（现 363 条）内嵌热文档、只增不减；真实备份目录约 228MiB。07-13 WIP 已把剩余 9 个手工 `fs::copy` 入口归一到中央 helper，并给“最近 30 份 + 每日恢复点”增加最多 30 个每日点的上限；但只有份数上限，没有字节预算，按当前单份约 5.9MB 仍可能逼近 350MB；
- **R3 迟切的税**:正确解法你库里有,主 store 却用最弱保护。
- **修法方向**：CAS 只解决并发静默覆盖；备份归一和限额只控制增长。全量 parse/serialize、UI 卡顿和正文膨胀仍需 audit/dispatch 正文外置或 SQLite 产品切换，不能把三件事说成“一次 CAS 一起解决”。

### B. 安全谓词/契约/常量靠人肉同步 —— 每处都是未来失守缝【部分已核】
契约文本双正本(1条测试兜底)、3b 闸 4 处 copy-paste、审批绕过黑名单 2 份、根路径常量 ≥3 处写死、前后端类型 ~6800 行手工镜像、行为分支押在后端中文文案上。全靠"测试断言"或"人肉纪律"防漂移。**已吃过一次亏**:弱版 C3 授权入口(record_global_boundary_review 弱版仍注册为命令,map称是授权链旁路)"改严版忘弱版"发生过。
- **修法方向**:安全谓词单一真源(谓词函数收一处、多处调用);契约/常量 codegen 或 include 单源;前端行为别读后端文案,读结构化 code。

## 四、CONFIRMED 高危(总指导亲核)

| 严重度 | 发现 | 证据 | 爆炸半径 |
|---|---|---|---|
| ~~P1~~ 处置中 | 主 store 无 CAS 并发丢写 | 初审证据 workflow_state_store.rs:13,103-125；07-13 WIP 增加锁内 revision CAS + stale snapshot 回归 | 静默覆盖已改成显式 conflict；业务级重放仍未建 |
| ~~P1~~ 处置中 | launcher 裸 txt 污染 store 根,与 R3 preflight 拒非 json 打架 | 07-13 WIP 已将新主管运行材料搬到 `runtime-artifacts/` | 未来污染已止；历史 txt 尚未迁移，SQLite preflight 仍会被旧材料阻断 |
| **P0** 未处置 | SQLite 迁移链完全漏掉当前主 store 五组真实数组 | 当前 `execution_attempts=148`、`permission_requests=1`、`workflow_chain_runs=37`、`workflow_execution_controls=148`、`workflow_machine_runs=10`；均不在 importer `WORKFLOW_ARRAYS`、apply `workflow_records`、schema、exporter | 当前快照重导会静默丢执行、权限与链路事实，禁止翻闸 |
| **P0** 未处置 | importer/apply/exporter 合同不一致 | importer 白名单接受 memory-lint/entity/pattern/blackboard，apply 对部分来源返回空 records 或未知 record kind `Ok(0)`；真实根已有 `memory-lint.v1.json`；exporter 又不覆盖多数 sidecar | 文件可被“接受”却不落表，回导也无法恢复，迁移成功口径失真 |
| **P0** 未处置 | 三个主管持久账本无 schema/import/export；exec registry 未分类 | `global-supervisor-reviews`、`supervisor-action-control`、`supervisor-orchestrator` 不在旧迁移合同；`exec-process-registry` 是 OS 进程租约，不能当历史事实导入 | 切库会丢主管审计，或错误复活已死亡进程租约 |
| ~~catch~~ 撤回 | h5:44 fail-open 债**已还为 fail-closed** | h5_project_dispatch_bridge.rs:44 明文注释 | **无漂移**:CURRENT §二已正确标"站3a 前置修成 fail-closed"。评审初判"文档仍挂待修"基于摸底旧 FACTS,核实物纠正——记为「核实物抓评审自己假 catch」 |
| **catch** | Tauri 命令实测 137 非 235 | rg #[tauri::command] 137(commands 108+mcp 10+agents 19) | 治理覆盖率结论按 137 修;摸底命令算错 |

## 五、Map 声称、未对抗核实(诚实标注·待验)

以下来自映射,总指导**未亲核**,涉 WIP 或需深读,列清单不下定论:

- **follow_up_worker 代际问题【代码已核并修，仍缺一次真实追问单】**：追问现在回读并持久化新报告，追问开始即使旧报告/旧 inspect 失效，失败不回退旧报告，inspect/终标幂等键绑定报告代际。站 3b PASS 单 `follow_up_count=0`，因此证明了普通 inspect 闭环，没有替代真实“派工→追问→读回新报告”验证。
- 弱版 C3 授权入口平行旁路(治理P1,plan_authorization_store.rs:225-312)。
- 一次派发横跨 3+ 无事务 JSON 账本,一致性靠调用顺序(主管编排P2)。
- binding_id 截断碰撞是模式病、同类生成器仍在新代码用(主管编排P2,3a 已修一处但 stable_id 截断仍广泛用于 event_id/reservation_id)。
- 前端交办组件 46 useState/相位机巨石,与"简单性原则"最相悖(前端P1);离线测试对编排组件结构性盲区(前端P1)。
- 记忆双召回面闸门不等价:top5 召回绕过 lint/过期/冲突(记忆P2);候选值密度无过滤,真用必淤积模板候选(记忆P2)。
- 中文 token 预算低估 4 倍、召回无排序无 recency(记忆P3)——评"切库收益"时勿高估(sqlite 也是 blob,翻闸不自带检索升级)。

## 六、给主人的三句战略话

1. **修 root pattern A 比修任何单条都值，但要分责**：CAS 防静默丢写；统一备份入口和保留上限控制磁盘增长；正文外置/SQLite 才处理整本改写与卡顿。三者不能互相冒领功劳。
2. **SQLite 先冻结迁移完整性合同，再补 schema/import/apply/export**——现在最大的风险不是“切得慢”，而是旧工具会把丢数据说成成功。
3. **god file 拆分不是洁癖,是护命根子**:director/Jiaoban 大到人核不动那天,"核实物"就名存实亡了;拆到可核尺寸,是给你自己的质量机制续命。

## 七、07-13 实数复核与处置边界【已核】

### 7.1 本轮已落 WIP（未 commit）

- 主 store 写入在同一文件锁内重新读取当前 revision；旧快照被拒绝，首个 writer 的字段和 revision 保留。
- 新主管运行材料改到 `runtime-artifacts/`；只解决未来新材料，未删除历史文件。
- workflow-state 备份入口已从 9 处手工 copy 收回中央 `backup_file`；中央策略保持最近 30 份，并只再保留最近 30 个每日恢复点，总量上界 60。
- 以上不等于“4.7MB 全量改写已解决”：3b 真跑后主 JSON 实测已经是 5,897,201 bytes，仍会整本 parse/serialize/rename。

### 7.2 SQLite 现在不能直接切

旧真库 `/r3-migration-work/b1-production-apply-20260615/workbench-state.v1.sqlite` 修改于 06-15；当前 JSON 修改于 07-13。两边实数：

| 数据 | 旧 SQLite | 当前 JSON |
|---|---:|---:|
| projects | 5 | 5 |
| workflows | 5 | 8 |
| nodes | 35 | 65 |
| edges | 32 | 50 |
| dispatches | 118 | 363 |
| audit events | 356 | 1473 |
| artifacts | 1 | 26 |
| work items | 12 | 57 |
| bindings | 36 | 75 |

当前新增的 `global-supervisor-reviews.v1.json`、`supervisor-action-control.v1.json`、`supervisor-orchestrator.v1.json`、`exec-process-registry.v1.json` 也不在 importer 白名单；unknown JSON 会被 importer/preflight 拒绝。真实根当前有 12 个 JSON、91 个历史主管 txt，preflight 会同时报 unknown JSON 与 non-JSON。更严重的是，当前主 JSON 的五组执行/权限数组完全不在旧 schema/import/apply/export 合同。因此本窗口只完成漂移审计与止血，不做 production apply、read-cut 或 stop-write，也不得声称 `ready for cutover`。

### 7.3 仍挂账

- CAS 冲突的业务级重放/重试策略。
- stale lock 恢复；异常退出留下锁文件时当前会永久阻断。
- 备份与最终 CAS 写入尚不是同一事务，备份未必严格对应本次成功提交的直接前态。
- audit/dispatch 正文外置与真正消除全量改写。
- SQLite 新 sidecar schema/importer、历史 txt 迁移、当前 JSON 全量重新导入与对账。

## 八、SQLite 切换最小有序站点（当前建议）

### M0 迁移完整性合同冻结

- 把真实根 12 个 JSON、91 个 txt、主 JSON 全部顶层字段逐项归类为：持久事实 / 兼容投影 / runtime 临时件 / 历史归档。
- 三个主管账本是持久事实；`exec-process-registry` 是 runtime 租约，不能把旧 entry 导入；历史 txt 只归档，不当领域事实。
- 验收：103 个文件全部且仅分类一次；任何 accepted source 都必须有明确落表与导出策略；未知项 fail-closed。

### M1 schema/import/apply/export 补全

- 补五组主状态数组、三个主管持久账本和当前已接受 sidecar 的完整落表/导出。
- 消灭 `records_for_source => Vec::new()` 与 `insert_domain_record => Ok(0)` 这种“接受但丢弃”。
- 验收：当前快照 `JSON → SQLite → JSON` 顶层字段、数组计数、natural key、record hash 语义一致；二次导入零新增、零冲突。

### M2 根目录治理与 preflight v2

- 91 个 txt 搬到明确的 legacy runtime 目录，保留 hash 清单，不删除；preflight 只忽略明确 runtime 分类，不能泛化忽略。
- 该步会改真实状态目录，执行前仍需单独维护窗口授权。
- 验收：12 个 JSON hash 不变；91 个 txt 精确覆盖；preflight 0 unknown、0 non-json、0 sensitive rejected。

### M3 当前快照全量重建演练

- 不升级 06-15 旧库；从冻结快照新建 DB，在临时副本完成 import/export/rollback/失败注入。
- 验收：按执行时最新实数逐项对账；source snapshot 字节不变；事务前崩溃无半成品，事务后报告失败能识别“已提交但报告失败”。

### M4 产品级 repository 与行级事务

- 建唯一产品存储入口、连接/WAL/busy 策略与有界重试；业务事实、审计、授权/主管动作进入同一事务。
- 外部 Codex 副作用只能用稳定 operation/idempotency key 对齐，禁止通用盲重试。
- 验收：普通工作项只改相关行，不再 serialize 5.9MB JSON；并发 mutation 不丢数据；failure injection 只能留下整笔旧状态或整笔新状态。

### M5 有限真实切换

- 独立维护窗口停 app、确认无 writer、最终快照、导入新 DB、逐表对账；先 DB 主写并保留 lag=0 JSON 投影，再做有限 read-cut。
- 验收：flag on/off 读模型一致；proposal、authorization、单 worker dispatch、inspect/final 代表路径都落 DB；rollback drill 可恢复一致状态。

### M6 观察与停写 JSON

- 观察无漂移后再单独批准 stop-write；JSON 改成低频 checkpoint/export，SQLite 使用一致性备份并同时设置份数与字节预算。
- 验收：多次业务操作只推进 DB 行与 revision，主 JSON mtime/hash 不变；从 SQLite 备份恢复后代表性工作流通过。

只有 M6 验收后，才可声称“全量改写、备份膨胀、SQLite 产品切换”完成。当前 WIP 只完成 CAS 止血、备份入口归一与份数上限，距离真实切换仍是 M0-M6 的顺序工作。
