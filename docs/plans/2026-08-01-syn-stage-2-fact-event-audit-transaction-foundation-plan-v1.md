# Syn Stage 2：事实、事件、审计与事务底座计划 v1

日期：2026-08-01<br>
阶段：`M2`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY；硬前置已满足（M1 于 2026-08-03 关闭，见 `decisions/2026-08-03-syn-m1-closure-acceptance-v1.md`），M1 残留项已划入（见 §0.4），激活待用户明确指示。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M2。<br>
硬前置：M1 退出门通过，M1 合同版本与迁移矩阵冻结。<br>
当前 active node / package：`NONE`；本计划不授权 schema、store、App、真实数据或产品代码写入。

## 0. 权威与现状口径

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → 2026-08-01 两份修订 → master → M1 → 本计划。现状只按当前 inventory 与直接源码核验结算；旧迁移计划和证据只能证明当时 fixture / dry-run，不证明当前 dirty tree 的 live store。

### 0.1 当前已经存在

- SQLite repository 已使用 WAL、启用 `foreign_keys` PRAGMA 和 `BEGIN IMMEDIATE`；这不表示现有表普遍已经声明完整 FK。部分 workflow / authorization / dispatch 路径能在一个 SQLite transaction 中写业务记录和 workflow audit。
- DB-primary bridge 会在 DB commit 后写 JSON projection；projection 失败时会冻结 DB-primary writer。当前另有 JSON-only fallback 路线；M2 必须正式裁决而不能假定它等同 fail closed。启动 reconcile 能识别 DB-leading、回放 projection，并对 JSON-leading / hash mismatch fail closed。
- 现有 importer、apply、export、reconcile、read-cut、dual-write、stop-write 和观察期模块可作为迁移素材。
- 这些是局部存储与迁移能力，不是全工作台统一 UoW、event ledger、audit ledger、outbox 或投影系统已经成立。

### 0.2 当前缺口

- `workbench_sqlite_schema.rs` 仍以大量 `record_json` 表为主；没有通用 typed event、command receipt、integration outbox、projection checkpoint / current snapshot 合同。
- 现有 workflow audit 不等于每条 command 都原子产生 domain state + event + audit + outbox。
- memory、knowledge、conversation、runtime、blackboard 等仍有多套 JSON / 文件 / 进程内状态；部分跨 store 写入存在明确半状态窗口。
- `AuditLedgerItem` 仍可返回 raw JSON；统一敏感 payload 边界尚未接入全部生产入口。
- 当前 DB blocked 后存在降级到 JSON-only 写的历史路线；若 M2 宣称 SQLite authoritative，必须明确选择“冻结业务写”“持久排队”或“可回收降级日志”，不得让 JSON 静默领先。

### 0.3 HOLD / UNKNOWN

- dirty tree 当前真实 DB-primary / JSON fallback / startup reconcile 结果；
- live store 的 count、key、canonical hash、unknown / corrupt / sensitive 数据分布；
- commit 成功但 receipt 丢失后的正式 replay 语义；
- outbox lease、超时、重领、外部 effect id 与结果回写合同；
- event payload ref 的物理存储与保留期；
- 当前测试、non-test build、真实 App 冷启动 / 强退 / 重启表现。

这些事项必须在获批任务中以 temp fixture 或经授权的隔离 profile 核验，计划不预填“通过”。

### 0.4 M1 残留项（2026-08-03 用户拍板划入本阶段范围）

以下来自 `decisions/2026-08-03-syn-m1-closure-acceptance-v1.md` 的残留清单，并入本阶段对应切片，不再是 M1 债务：

| 残留项 | 承接切片 | 说明 |
|---|---|---|
| grant 校验仅为格式级：无 grant store，活路径 `grant_id = dispatch_id`，`verify_grant` 跑自铸通配 grant | DAT-002（schema/ports）+ DAT-003（vertical slice） | M2 建真 grant 持久化与 mint/load/verify；在接上之前，任何规划不得把 grant 当作真实防御 |
| FND-006 场景 3/4（伪造 report/grant 全链运行时验证） | DAT-008（隔离 App 验收） | 需 fake runner 全链夹具；届时 consume 路径的 grant 拒绝可拿运行时证据 |
| FND-006 场景 5（Station 3b 写入拒绝运行时验证） | DAT-008；若 supervisor 会话机制不在 M2 建成，则顺延至 M3 并在 M2 退出时显式标注 | 需 supervisor 会话夹具 |
| `sqlite_production_preflight_blocked_creates_no_db_or_report` 稳定失败（M1 前既有） | DAT-002 期间定性修复 | preflight 期望拦截、实际 completed 且建库；属本阶段存储域 |
| 进程夹具族环境性失败（codex_local_runner / obsidian / manual_relay 轮流翻） | DAT-002 期间合并排查 | 07-26 R3B 浏览器抖动、07-27 Rust 抖动同族并案 |
| code-map advisory（`MAP_UPDATE_REQUIRED`：新模块无能力映射、`index.json` invalid domain path） | 随首个 DAT 提交批处理 | 非阻断但持续告警 |
| FND-001 合同 commit 未进 integration main（HOLD） | M2 激活前由指导线决定集成路径 | 不涉及产品代码 |

## 1. 阶段目标

建立全工作台可复用、但不吞并各域业务真源的事务机制：

1. 每条已迁移 command 有稳定 identity、幂等键、policy receipt 和可重放结果；其余入口保留精确 migration state，不把代表性切片冒充全覆盖；
2. 同一事务原子提交 domain state、typed event、scrubbed audit 和待执行 outbox；
3. 外部副作用只在 commit 后领取，结果通过新 command 回写；
4. projector 确定性、可重建、有 checkpoint、有失败 receipt；
5. 旧 store 通过 adapter、shadow、parity、reconcile 和 rollback 逐域迁移；
6. unknown / corrupt / sensitive 数据有显式 disposition，不以 warning 冒充零损失；
7. 为 M3-M9 提供稳定的 repository / UoW / event / audit / outbox ports。

## 2. 本阶段不做

- 不一次迁完所有 domain，不把 M2 写成“大一统数据库重构”；
- 不把 event ledger 变成跨域业务事实池，也不做全量 event sourcing；
- 不接真实 provider、凭据、邮件、日历或外部写 action；
- 不复制 raw transcript、prompt、tool output、secret 或完整 provider response；
- 不在 parity 之前切新主读，不在回滚前关闭旧读，不物理删除旧 store；
- 不借 schema migration 顺带改首页、角色、项目流程、记忆政策或连接器产品语义；
- 不用表存在、单测、fixture 或 debug build 声称真实数据切换完成。

## 3. 冻结输入与阶段合同

进入 `SYN-DAT-001` 前必须具备：

- M1 的 `identity-scope-v1`、`command-v1`、`event-audit-outbox-v1` 和敏感字段矩阵；
- 全入口 inventory 与 `migrated / guarded-legacy / blocked / not-in-scope` 清单；
- store / table / sidecar owner、natural key、revision、join key、projection、读写入口、数据等级；
- 每个拟迁 domain 的静态 inventory / fixture schema、预期 natural key 和 manifest 规格；真实路径、权限、count、key、canonical hash、unknown / corrupt / sensitive 分类只有在独立只读 live-manifest preflight 获批后才成为输入；
- opening HEAD / status / target hashes、单写者、允许写面、rollback owner。

### 3.1 对象与 owner

| 对象 | 唯一 owner / 真源 | 强制字段 / 约束 |
|---|---|---|
| `CommandReceipt` | application command / receipt ledger；SQLite receipt repository 是物理权威 | command_id、idempotency_key、actor/scope/object、policy result、commit/result hash、status；aggregate 只拥有 domain state，并在成功 UoW 中原子追加 receipt |
| domain state | 对应 aggregate | revision / optimistic precondition；event 不反向拥有业务状态 |
| `WorkbenchEventEnvelope` | event ledger repository；EventWriter 是唯一 mutation port | correlation / causation、schema、source、sensitivity、summary/ref/hash；禁止 raw secret |
| `AuditRecord` | audit ledger repository；AuditWriter 是唯一 mutation port | allowed / denied / committed / degraded、actor、scope、reason、scrub result、source refs |
| `OutboxItem` | outbox repository；OutboxWriter / claimer 分权 | effect id、capability、payload ref/hash、lease、attempt、next retry、result command |
| `CurrentSnapshot` | 对应 domain 的 authoritative current-state projection repository | object / revision / source watermark / hash；更新规则确定、可由 authoritative state 重建，不反向拥有业务事实 |
| `ProjectionCheckpoint` | 每个 projector | projector version、last event / watermark、status、error receipt |
| read model | 对应 projector | 可丢弃 / 重建；只读，禁止反写业务事实 |
| migration manifest | migration owner | before/after、disposition、parity、rollback、残留说明 |

## 4. 任务切片

### SYN-DAT-001 — 机制合同与逐域迁移清单

只写合同 / schema design / migration matrix，不改生产 schema。消费 M1 固定的外部接口 / 禁止字段，由 M2 单一 owner 冻结其持久化和运行时状态机、FK / unique / index、receipt 丢失、lease、quarantine、重建和 rollback。语义变化必须回到 M1 version review。另冻结一个具名 `reference_slice_id`、aggregate 和 command，DAT-003—006 必须使用同一 reference slice。

DAT-001 还必须冻结安全 payload storage、payload-ref 完整性、retention / GC 和 scrub 规则；在这些决定完成前 DAT-002 / DAT-004 保持 HOLD。policy-denied command 使用独立、幂等、append-only 的 scrubbed denial receipt / audit transaction，且零 domain / event / outbox mutation。

验收：至少用 conversation、workflow、memory、knowledge 四类现状路径走纸面追踪；每类都能回答 owner、事务边界、外部 effect、失败残留和恢复动作。

### SYN-DAT-001B — 经授权的只读 live-manifest preflight（条件包）

只有需要真实 parity / cutover 的 domain 才激活。包内必须列 exact roots、数据等级、只读方法、允许保留的 value-free count/key/hash、敏感 material 停止路线和零 mutation 证明。它不是 DAT-001 文档包的前置，也不授权 migration。

### SYN-DAT-002 — Additive schema 与 repository ports

仅新增版本化 migration、typed DTO、repository / UoW ports 和 temp DB 测试；不切写路。migration 必须可重复检查，旧 binary 行为与 rollback 窗口写清。

### SYN-DAT-003 — 无外部副作用的首个完整 vertical slice

使用 DAT-001 冻结的同一 `reference_slice_id`、aggregate 和 command，选择 owner 清楚、无真实外部动作、数据量可控的路径，接通：policy → UoW → domain state → event → audit → receipt → current snapshot / projector。它证明该已迁切片，不代表其他入口已经迁移。

### SYN-DAT-004 — Transactional outbox 与结果 command

继续使用同一 reference slice 实现 claim / lease / expiry / retry / poison / cancellation、稳定 effect id、单消费者语义和结果 command。fake adapter 故障注入必须覆盖 commit 前、commit 后未执行、执行成功 receipt 丢失、结果回写失败和重复消费。

### SYN-DAT-005 — Deterministic projector 与 shadow / parity

定义 canonical normalization；对同一 reference slice / domain 做 shadow write/read、current snapshot、checkpoint、重建、count/key/hash/semantic parity 和 degraded receipt。差异必须分类为 bug、批准差异、legacy corrupt 或 UNKNOWN。若 parity 使用真实 store，`DAT-005(domain X)` 必须先完成同一 domain 的 DAT-001B live-manifest preflight。

### SYN-DAT-006 — Legacy adapter、quarantine 与恢复

为同一 reference slice 的 JSON / sidecar / file owner 建明确 adapter；unknown、corrupt、敏感与无法精确 join 的记录进入 manifest / quarantine，不静默丢弃，也不把原始 secret 复制进普通 SQLite。

### SYN-DAT-007 — 逐域切换包

每次只切一个 domain：shadow → parity → new primary → compatibility read-only。`DAT-007(domain X)` 硬依赖 `DAT-001B(domain X)`；每个 domain 单独 active package、观察窗、before / after evidence、回切动作；M2 不预先授权任何真实数据切换。

### SYN-DAT-008 — 隔离 App 崩溃与恢复验收

依赖 DAT-004 + DAT-005 + DAT-006；若场景包含真实 cutover，再依赖对应 DAT-007。隔离 profile + scratch store 覆盖冷启动、写一笔、commit 前强退、commit 后 receipt 丢失、投影失败、重启恢复、重复 command、DB busy/corrupt、JSON-leading 和 outbox retry。真实 store 另行授权。

## 5. 顺序、并行与写所有权

```text
DAT-001 → DAT-002 → DAT-003 → DAT-004
                         └→ DAT-005 → DAT-006 → DAT-008
                                           └→ DAT-007（逐域真实切换时）
DAT-001B(domain X) ───────→ DAT-005 / DAT-007（使用真实 store 时）
```

- schema / migration / UoW / event / outbox 承重文件由平台数据线单写；
- domain adapter 只能在 ports 和 schema version 冻结后并行，且不得改同一 table、registry、AppState 或 migration 文件；
- Rust producer 与 React consumer 可在 DTO 冻结后并行，但 UI 只消费 read model；
- DAT-003 后只能收集 M3 / M7 的下游消费需求、做合同评审或不修改产品代码的隔离 fixture；不得激活 M3/M7 实现包、建立 RoleSession / memory 真源或改变阶段路由。任何实现必须等 M2 exit 与显式激活；
- 每个任务包必须记录 opening hashes、write_surface、single_writer、migration/rollback owner 与现有 WIP 归属。

## 6. 迁移、兼容与回滚

- additive schema 先行；任何 destructive migration 都不属于 M2 默认权限；
- 旧 store 在观察窗内保持可读和可导出，新系统不得先删 / 改无法回切的旧记录；
- DB-primary blocked 时的行为必须合同化并 fail closed；默认不得让无日志 JSON 写继续成为新事实；
- command replay 以 command_id / idempotency_key 和结果 hash 返回同一 receipt；
- outbox 外部动作以 effect id 去重，外部执行结果只能经 result command 改 domain；
- projector 可从 authoritative state / events 重建，checkpoint 可丢弃；
- rollback 只切回已验证的旧读主 / 旧 adapter，不回退安全 scrubber，不恢复双主写；
- unknown / corrupt / sensitive 记录保留 value-free manifest / hash 与可控原件位置，处置未决即 HOLD。

## 7. 验证与证据上限

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / schema lint | owner、FK、状态、禁止字段、migration 顺序一致 | repository 已正确实现 |
| Unit / property | UoW、幂等、scrub、lease、projector 确定性 | 生产入口全接入 |
| Temp SQLite / fixture | rollback、crash point、parity、quarantine、重建 | live store 已迁移 |
| Non-test build | production path 可构建 | App 行为正确 |
| Isolated Tauri | scratch store 冷启动 / 强退 / 重启 / 恢复可见 | 真实数据、provider 或发布通过 |
| 经授权 live migration | 精确 domain 的真实 before/after/parity/rollback | 其他 domain 或全工作台已切换 |

关键机械断言：commit 前任一点失败全部回滚；commit 后重试不重复外部动作；投影失败有 durable receipt；raw JSON 默认不进入产品 DTO；旧 / 新 count、key、canonical hash 可解释。

## 8. 独立授权与停止条件

以下必须各自 task package：生产 schema migration、只读 live-manifest preflight、真实 store shadow、每个 domain 主读 / 主写切换、DB/JSON reconcile、真实 App 强退、任何外部 adapter、旧写路关闭。真实凭据、真实 connector、生产项目写入、Git 写入均不由本阶段授予。

遇到任一情况立即停：owner / natural key 不唯一；migration 缺 before/after/幂等/回滚；unknown 数据被静默排除；双主写；JSON 无日志领先；敏感原文进入 event/audit；outbox 可能重复副作用；公共写面撞 WIP；只能用 fixture 声称 live cutover。

## 9. 阶段退出与交给 M3

全部满足才允许将 M3 设为 current：

- 同一具名 reference slice / domain 完整通过 UoW、denial audit、current snapshot、outbox、projector、shadow、parity、recovery；不同样本各自通过不得拼成这项结论；
- 公共 ports、schema、receipt 和禁止字段冻结，所有消费方版本可追踪；
- 每个已触及 domain 有 exact migration state，其余明确 `not-migrated / HOLD`；
- 隔离 App 崩溃 / 重启证据通过，结论未越级到真实 store；
- 旧数据未被物理删除，rollback / export 可执行；
- CURRENT 回写实际完成、证据、HOLD 和下一阶段；
- 用户显式激活 M3 前不得自动进入角色会话实现。
