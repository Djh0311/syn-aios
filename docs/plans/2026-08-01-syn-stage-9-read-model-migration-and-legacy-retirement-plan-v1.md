# Syn Stage 9：读模型迁移与旧路退役计划 v1

日期：2026-08-01<br>
阶段：`M9`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M9。<br>
硬前置：M1-M8 exit receipts；各目标 domain 有 authoritative source、projector、parity、rollback 与 replacement acceptance。<br>
当前 active node / package：`NONE`；本计划不授权主读切换、command unregister、archive、删除、App 或产品代码。

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → 当前 inventory → master → M1-M8 exit / HOLD receipts → 本计划。历史 migration fixture / dry-run 只作素材，不升级为 live cutover 事实。

## 0. 当前事实与未知

### 当前 read-model 事实

- 前端并行请求六个 page queries，却把每份 `snapshot_slice` 合并回 `WorkbenchSnapshot`；
- 后端每次 page query 仍先 `build_snapshot`，再取相应 slice；这是源码接线，不是独立 projector / query；
- 当前源码静态检索没有目标 typed `ObjectRef` / deep-link 领域实现；
- Audit、runtime、memory、workflow、notification 等仍由多份 sidecar / heuristic 聚合；raw JSON 边界不统一；
- command registry 同时注册 manual relay、agent / supervisor transport、workflow dispatch、已 blocked 的 workflow machine、旧 `knowledge_vault_*` 与新 native knowledge workspace APIs。

### 可复用迁移素材

- SQLite / JSON importer、export、reconcile、read-cut、dual-write、stop-write、observation period、backup / recovery 模块；
- 旧 command / store / UI 路径和现状 inventory 已有清单；
- M2 应提供 projector checkpoint / parity / recovery，M3-M8 应提供每域 replacement receipts。

### 尚无证据

- 没有全页 shadow read、new-primary、compatibility read-only、reference-zero、command unregister、rollback 或 retirement 完成证据；
- 历史 SQLite fixture / dry-run 不证明 current dirty tree live store；
- 当前 DB-primary / JSON fallback / reconcile 仍是 HOLD；
- 没有 typed ObjectRef / exact deep link 的 production 实现；
- 没有旧路物理删除或发布批准。

## 1. 阶段目标

1. 首页、项目、角色会话、成员、运行中、待办、日报、审计各有独立 projector / query；
2. UI 不再拼底层 sidecar，也不反复构建完整 snapshot；
3. 所有可见项带 typed `ObjectRef`、scope、source ref、精确 deep link 和 resolution receipt；
4. 每类迁移严格执行 `shadow → new primary → compatibility read-only → command unregister → archive/export`；
5. 逐页 / 逐对象 parity、rebuild、failure / rollback 可机械复现；
6. raw JSON 默认不出产品响应；legacy data 有 manifest、hash、export 和残留解释；
7. 一次只退役一个 capability / command family，物理删除继续另批。

## 2. 本阶段不做

- 不在 replacement 未验收时 unregister / 隐藏旧入口；
- 不把“命令已 blocked”当成“已退役”；
- 不一次删除多类 store / command / UI；
- 不把 compatibility read-only 继续当双主写；
- 不因 parity count 一致忽略语义、source ref、ACL、freshness 或敏感裁剪差异；
- 不把 ObjectRef 只做前端字符串；后端 resolver / scope / owner 必须同合同；
- 不在 M9 顺带做新业务功能、视觉重做、真实 connector 或发布；
- 不用 fixture / build / history 声称 live cutover 完成。

## 3. 退役清单与 owner

初始 candidate family：

1. 全量 snapshot build + frontend snapshot_slice merge；
2. 多份 heuristic notification / todo / audit aggregation；
3. Jiaoban / Agent Center frontend conversation cache；
4. manual relay 中经 MIG-001 精确判定为 `RETIRE` 的正常业务 symbols；保留 / 提取的诊断、ConversationTransportPort 或 AgentAdapter 不在 family unregister 范围；
5. old agent / supervisor / offline handoff 中经 MIG-001 精确判定为 `RETIRE` 的 symbols；M3 replacement / adapter 明确 `KEEP / EXTRACT`；
6. fixed Mario chain、resident / pilot loop、synthetic Phase A 产品入口；
7. `run_workflow_machine` 及经 MIG-001 精确判定为 `RETIRE` 的 legacy dispatch symbols；仍服务新 aggregate 的 command 明确 `KEEP / EXTRACT`；
8. workflow id `contains(slug)` owner compatibility；
9. legacy Canvas MCP execution tools（数据另迁保留）；
10. 旧 `knowledge_vault_*` single-layer note APIs；
11. legacy JSON / sidecar primary write paths；
12. 旧计划、旧按钮和隐藏 action 的执行语义。

每一项必须有：current owner、replacement owner、read / write references、data manifest、replacement evidence、cutover owner、rollback owner、unregister owner、archive / export path、residual HOLD。退役线是唯一 writer。

## 4. 任务切片

### SYN-MIG-001 — Read-model / retirement inventory freeze

冻结全部 page / command / store / UI reference graph、typed ObjectRef registry、candidate family、owner、opening hashes、acceptance source 和禁止删除项。每个 family 必须列 exact symbol / command / UI route / store，并逐项标 `KEEP / EXTRACT / RETIRE / HOLD`；在该清单冻结前任何 family unregister 不可激活。只读分析 / 文档。

### SYN-MIG-002 — Typed ObjectRef 与 deep-link resolver

消费已退出的 M1/M3 合同，实现后端 resolution、scope / owner / not-found / stale / denied receipt 与前端导航；MIG-002 前必须复跑 M1 跨 scope、伪造 owner、denied / stale 负例。禁止前端自行拼路径或跨 scope id。

### SYN-MIG-003 — 独立 domain projector / query

按页面一次一个：home、project、role sessions、members、running、attention/todo、daily、audit。每个 query 有 version / watermark / pagination / ACL / degraded state，禁止 fallback 构建整份 snapshot 后伪装。

### SYN-MIG-004 — Shadow read 与逐页 parity

同时读取 old / new，比较 count、keys、canonical values、source refs、ACL、freshness、ordering、empty/error states。差异逐项批准或修复，不以总数遮蔽语义差异。

### SYN-MIG-005 — New primary / compatibility read-only

每次只切一页 / object family；观察期记录 latency、errors、fallback use、stale、user-visible diff。旧路径只读，禁止继续产生新事实。

### SYN-MIG-006 — Command family unregister

在 replacement 真实验收、reference-zero 和 rollback 通过后，一次 unregister 一个 command family；先 backend deny / telemetry，再移 UI / registry reference。每次需独立高风险 package。

### SYN-MIG-005A — Unregister 前 manifest / export / recovery proof

在 MIG-006 之前为当前 family 生成只读 manifest、count/key/hash、schema、permissions 和可恢复 export；在隔离副本实际做 restore / replacement rebuild / rollback drill。此处只验证恢复依据，不执行 archive，也不关闭入口。

### SYN-MIG-007 — Unregister 后 archive / export 封存

MIG-006 完成并通过观察窗后，将已验证的 manifest / export / restore instructions 封存到 archive，补记 unregister receipt、最终 reference telemetry 和 residual HOLD。archive 不等于删除，也不替代 MIG-005A 的事前恢复证明。

### SYN-MIG-008 — Legacy UI / plan semantics retirement

移除已无 command / source 的旧按钮、隐藏 action 和 current wording；旧文档标历史并保留证据链接，不把正文“下一步”恢复为 current。

### SYN-MIG-009 — 物理删除候选清单

只形成 candidate + evidence，不在 M9 默认执行。逐对象列 reference-zero、retention、legal/user need、backup、restore、exact delete target 和单独用户授权。

## 5. 强制顺序与写所有权

```text
MIG-001 → MIG-002 → MIG-003 → MIG-004 → MIG-005 → MIG-005A
                                                   ↓
                                              MIG-006 → MIG-007 → MIG-008 → MIG-009(candidate only)
```

- retirement / cutover 线单写；domain owner 只提交 replacement receipt，不自行 unregister；
- 一次只迁一页或一个 command family；公共 registry / App assembly / routing / schema 不允许多 writer；
- UI visual polish 与 data source cutover 分包；
- connector、memory、workflow、conversation 等不同 family 可以做只读审计，但切换必须串行，便于归因 / rollback；
- active package 必须列 exact references / commands / stores / UI paths / hashes；dirty WIP 冲突即停。

## 6. 迁移、回滚和不可逆边界

- old / new shadow 期间只允许一个 authoritative writer；
- new primary 切换必须有 feature-independent rollback command / procedure，不靠放宽安全 guard；
- compatibility read-only 期间记录所有访问，达到 reference-zero 前不得 unregister；
- unregister 前必须已验证 data export / restore / rollback；unregister 后仍保留 adapter restore 窗口并封存最终 archive；
- projector 可从 authoritative source 重建，checkpoint 丢失不丢业务事实；
- raw legacy blob 只在受限 archive / export，不进入普通产品 DTO；
- archive/export 与 physical delete 分开；删除只能针对显式、验证后的 exact target，另获用户批准；
- 每次失败只回滚当前 family，不同时恢复多个旧主写路径。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Static inventory / reference graph | command/store/UI 引用完整、owner 明确 | replacement 可用 |
| Unit / projector | ObjectRef、query、pagination、rebuild、ACL | App parity |
| Temp / shadow fixtures | old/new semantic parity、failure / rollback | live cutover |
| Non-test build | unregister candidate 后 production path 可构建 | 用户路径通过 |
| Isolated Tauri | 每页 deep link、errors、rollback、old read-only 可见 | 真实数据完成 |
| 经授权 live cutover | 精确 family 的 before/after/parity/reference-zero/rollback | 其他 family 或删除通过 |

每个 family 的最低 evidence：replacement real-App acceptance（若属用户路径）、old/new parity、fallback / error count、reference search + runtime telemetry、export/restore、rollback drill、CURRENT 状态。

## 8. 授权与停止条件

每个 new-primary、旧写关闭、command unregister、archive/export、物理删除分别建包；真实 store、真实项目、真实 connector 与真实 App 也按对象授权。M9 不授权发布、外部 action、Git 或批量清理。

立即停止：replacement 未验收；owner / source 不唯一；shadow 变双主写；parity 有未解释差异；ObjectRef 可跨 scope；reference-zero 未成立；rollback 未演练；archive 无 export；需要一次删多类旧源；WIP 冲突；fixture 被表述成 live cutover。

## 9. 阶段退出与 M10 输入

全部满足才进入 M10：

- 所有目标页面使用独立 projector / query，不再反复构建全量 snapshot；
- typed ObjectRef / deep link / ACL / stale / denied 路径通过；
- §3 每个 candidate family 状态为 `retired / compatibility-read-only / explicit HOLD`，无模糊“差不多”；
- 已退役 family 有 parity、reference-zero、unregister、export、rollback 证据；
- raw JSON 默认不出产品响应；
- 旧 store 未未经授权物理删除；
- 给 M10 提供 release candidate 路由、HOLD、recovery / rollback scripts 和全日场景 truth sources；
- CURRENT 回写实际完成 / HOLD / 下一步，M10 未激活不得续跑。
