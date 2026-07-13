# 任务包:M4 产品级 SQLite repository + 行级事务(休眠机器·不翻闸)v1

日期:2026-07-13 · 档位:**轻档**(全程 temp/演练 DB+休眠模块·产品行为不变=JSON 仍是活库)· 基线 commit `6dd0d33`。
上承:M0-M3 迁移完整性收口 `1ec3544`(六处一致·往返可证明);排布正本=架构评审 `docs/2026-07-13-architecture-review-v1.md` §八 M4。
勘察正本:文末「勘察坐标 & 坑」,2026-07-13 只读勘察(HEAD `6dd0d33`·全坐标真实读到)。

> **状态门**:M4 造的是**休眠的产品存储引擎**——repository 存在、WAL/busy/事务语义被证明、行级写被证明,但**产品命令零接线、JSON 仍是活库**。接线/翻闸=M5(拍板锁着·用户授权那一下+维护窗口)。敏感谓词误报修复=另一重档包,不在本包。

## 所属开发线

执行线(架构/存储线)。总指导写包+核实物;执行线一次做完;**不 commit**。

## 背景

评审 §八 M4:唯一产品存储入口、连接/WAL/busy 策略与有界重试;业务事实+审计+授权/主管动作**同一事务**;外部 Codex 副作用只用稳定 idempotency key 对齐、**禁盲重试**;验收=普通工作项只改相关行(不再 serialize 5.9MB)、并发不丢数据、failure injection 只留整旧或整新。

勘察实况(纠正三个想当然):
- **WAL/busy 从零建**:全部 workbench_sqlite_* 今天零 WAL、零 busy_timeout、零 Immediate;仅有的两个事务都是默认 Deferred(apply.rs:124 整批导入、transaction_acceptance.rs:102)。rusqlite 0.32.1(bundled)API 齐:`busy_timeout`/`transaction_with_behavior(Immediate)`/`pragma_update(journal_mode=WAL)`/savepoint。
- **dual_write 不是接缝**:批式 fixture 演练桩(dual_write.rs:41),无 per-write API;可搬的只有失败点枚举(:12-18)、tmp+rename 提交(:134-142)、incomplete 标记、rollback manifest 姿势。
- **库内并存两种 busy 语义**:主 store 锁单次不重试(workflow_state_store.rs:186-208)vs sidecar 锁 5×100ms(proposal/plan_auth store)。本包基准**拍死**:见目标 R1。

## 目标(交付物)

### R1 repository 模块(新文件 `workbench_sqlite_repository.rs`,<3000 行,大了按 observation_period 的 `mod tests;`+子目录拆)
- 单一入口:连接构造统一走 repository(打开即 `journal_mode=WAL` + `busy_timeout` 显式毫秒数 + `foreign_keys=ON`);**写事务一律 `transaction_with_behavior(Immediate)`**。
- **有界重试基准(拍死)**:Busy 冲突重试预算 **≤1 次、不循环**(对齐全库「retry 预算 1」纪律);busy_timeout 数值与预算写进代码注释+证据。
- 事务包裹器:业务事实行 + audit 行 + 授权/主管动作行**同笔提交**;commit 前任意失败=整体回滚零半行。

### R2 六个代表流的行级 mutation(样板=今天 JSON 侧写入口,坐标见附录 §4;JSON 原文件零碰)
| 流 | JSON 样板 | 行级要点 |
|---|---|---|
| ① proposal 落店 | proposal_store.rs:57 | INSERT proposal 行+audit 行,同笔 |
| ② authorization | plan_authorization_store.rs:51 | 行级 CAS:**强制**版本校验(不 opt-in——JSON 侧 expected=None 跳过的坑不带过来) |
| ③ dispatch 预留+记录 | workflow_execution_entrypoints.rs:288/:354 | dispatch 行+audit 行+work_item/node 状态行,同笔;不再整本读写 |
| ④ 主管动作 reserve/complete | supervisor_action_controller.rs:813/:874 | 两相包夹外部副作用;reserved 未回写**绝不自动重放**(照 :1025-1063 语义)→ 标 waiting 状态行 |
| ⑤ audit 追加 | 无公共 helper(全库内联 push) | repository 出**唯一** append_audit 入口,各流复用 |
| ⑥ work_item 状态变更 | workflow_run_dispatch_entrypoints.rs:440 | 复用 `control_core::validate_work_item_state_transition` 判合法,再改行+audit 同笔 |

### R3 idempotency(行级强制)
- key 生成对齐既有 `action_idempotency_key`(supervisor_action_controller.rs:1125-1167)。
- **先核后加约束**:检查 live 30 条 supervisor_actions 的 idempotency_key 有无重复→无重复则 DDL 加 `UNIQUE INDEX`(schema.rs:137 现无约束);有重复则改用 Immediate 事务内 SELECT-then-INSERT 并在回传记明原因。同 key 重放=零新行。

### R4 失败注入 + 并发证明
- 六流各注入 commit 前失败→整旧无半行;commit 后报告失败→可识别"已提交"(照 transaction_acceptance 分类姿势,结构可搬代码不可搬——它 fixture id 全写死 r3-a13)。
- 双线程同 temp DB 并发 mutation(WAL+busy+Immediate)→零丢写、计数精确。

### R5 revision 双面小修(坐标附录 §7)
- exporter.rs:486 `ORDER BY meta_json LIMIT 1` 字典序取行→改确定性选行(按 import 批次新旧);exporter.rs:291-295 `source_import_meta` 无 ORDER BY→同修。
- apply.rs:664-676 同键 `DO NOTHING` 留旧 meta 行→重导更新语义(upsert 或按批次键);回归测试两面:两 root 两批→取新;同 root 重导 revision 变→不留旧。
- workspace_id 写死 `'fixture-workspace'`(apply.rs:666):能一并修就修;fixture hash 断言阻力大则**记账残留**,不硬改。

### R6 休眠 gating(拍死:取最强层)
- **无产品调用者层**(照 workbench_sqlite 全家现状):零 Tauri command、commands.rs/lib.rs(除 mod 声明)/main/index_host_app_entrypoints 零引用;真写只对 temp/演练 DB 开(路径闸照 dual_write.rs:166-209 / schema.rs:227-234 姿势)。
- 不採 read_cut 的 flag-off-fallback 层(那层代码已可进产品,休眠强度低一档)。

## 允许读取

src/ 全部;架构评审;M0 合同;live 根只读采数(hash/计数/查 idempotency_key 重复,严禁写)。

## 允许写入

- **新**:`src/workbench_sqlite_repository.rs`(+必要子模块/tests 子文件)
- **增量**:workbench_sqlite_{schema,apply,exporter,production_apply,dual_write}.rs(DDL 加索引/接口暴露/R5 小修);各 `#[cfg(test)]`
- 临时 DB 全落 temp_dir;新证据文档 evidence/

## 禁止事项(红线)

1. **不翻闸**:repository 零接线产品路径;不改"哪个是活库";不 read-cut/stop-write/production apply 真库。
2. **live JSON 写路径零碰**(它们是样板不是靶子):workflow_state_store.rs、project_consultation_proposal_store.rs、plan_authorization_store.rs、supervisor_action_controller.rs、global_supervisor_review_store.rs、workflow_execution_entrypoints.rs、workflow_run_dispatch_entrypoints.rs——**一行 diff 都不许有**。
3. **codex_db.rs 零碰**(那是读 Codex 自家库的,WAL 策略别误配上去)。
4. **read_cut.rs 零加行**(2996/3000,加 4 行即撞 shape gate error);production_apply(2425)/stop_write(2017)偏满,能落新模块就别往里塞。
5. 不改安全闸/沙箱/审批/敏感谓词(高危#3)。零新 tauri command(command 基线 97 的 warn 也不许新增);不进 lib.rs(mod 声明除外);零新 sidecar 种类;棘轮文件不碰。
6. 既有表 DDL **只加不改语义**(新索引/新表要回传列明并保六处一致或明确标"repository 专用非迁移面");既有测试一条不放松。
7. 不写真实 workflow-state 根。卡住/歧义/发现勘察说错→停下报。

## 变更辐射面

- 改了什么假设:「SQLite 只有整快照批式 import/export」→「存在行级事务写引擎(休眠)」;「meta 行选择字典序」→「确定性选行」。
- 依赖旧假设的:M3 往返/snapshot_apply/observation 的 hash 对账(R5 改 meta 语义会动部分 fixture 断言——逐个改并回传列明);schema 六处一致(新 DDL 同步或标注非迁移面);exporter 消费者。
- 五态旅程:**不涉及**(纯后端休眠件,UI 零变化)。

## 形状影响

- 类型:功能(休眠引擎)+治理(mutation 路径零整本 serialize 的前后证明)。
- 新增落点:workbench_sqlite_repository.rs(.rs 新文件上限 3000,gate :20-24;RATCHET_WATERLINES 不含 workbench_sqlite 家族,超限即 error)。
- 新增 command:0。新 sidecar:0。棘轮:不碰。shape gate 豁免:不需要。
- 基线 commit:`6dd0d33`。完成 commit:〔总指导收口填〕。

## 验收标准(预写死)

- **事务**:六流各=单 Immediate 事务;commit 前注入失败→整旧(表计数与注入前逐项等);commit 后报告失败→分类可识别。
- **行级**:mutation 路径零整本 serialize——测试断言单 mutation 触碰行数常数级 + grep 证明 repository mutation 路径不调用 workflow_state_projection/export_*。
- **并发**:双线程并发 N 次 mutation→最终计数精确无丢写;同 idempotency key 重放→零新行(UNIQUE 或事务内查重,按 R3 拍板路径)。
- **CAS**:②流版本冲突(旧版本写新行)→显式拒绝,非静默覆盖。
- **休眠**:grep 证明产品四入口(commands/lib[除 mod]/main/index_host_app_entrypoints)对 repository 零引用;零新 command。
- **R5**:两批不同 root→exporter 取新批 meta;同 root 重导→meta 更新不留旧;source_import_meta 确定性。
- **通用**:`cargo test --lib` 基线 **896/0/44 只增不减**;shape gate baseline/check 零净增(read_cut 行数不变!);`git diff --check` 过;fmt 仅历史三漂移;真实根 0 改动(hash 前后一致,照 M3 姿势留证)。

## 必须回传(10 项)

1 做了什么 · 2 改了哪些文件 · 3 新增测试/证据 · 4 哪些结论有依据 · 5 哪些仍不确定 · 6 风险+下一步 · 7 shape gate baseline/check 摘要 · 8 start/end commit · 9 是否新增 command/sidecar/碰棘轮 · 10 **被闸拦过的事**(无也写"无")。

## 总指导回收动作

核实物:亲跑全库测试+并发/失败注入定向;扫 diff 核红线(红线 2 的七个文件零 diff、codex_db 零 diff、read_cut 行数不变、产品四入口零引用);真实根 hash 亲核 → 接受/需改/暂停/废弃。核清 commit(问一次)。M5 仍锁。

---

## 勘察坐标 & 坑(2026-07-13 只读勘察·执行线照此定位)

**§1 dual_write/read_cut**:dual_write.rs:41 唯一入口(批式演练);失败点枚举 :12-18;tmp+rename :134-142;路径闸 :166-209。read_cut flag-off fallback :318-341;禁旗自检拒写 :1098-1120;allowlist :620-635。
**§2 连接/事务**:全家零 WAL/busy/Immediate;pragma 仅 foreign_keys(schema.rs:209/apply.rs:121/transaction_acceptance.rs:100);每操作 `Connection::open` 即开即用;唯一 flags 先例=codex_db.rs:125(READ_ONLY|URI,别学到写库上)。事务仅 apply.rs:124-126 与 transaction_acceptance.rs:102-104(均 Deferred)。transaction_acceptance.rs:83 五写一事务证明(:106-183,commit :202;注入 :242-251)——**结构可搬,代码写死 r3-a13 不可参数化复用**。
**§3 schema**:68 表+18 索引(DDL schema.rs:7-159);58 张带 record_hash+record_json;workflow_state_meta PK(workspace_id,source_root_hash)(:66-75);supervisor_actions 有 idempotency_key 列**无 UNIQUE**(:137)。
**§4 六写入口样板**:①proposal_store.rs:57(锁 5×100ms :969-1007·CAS :565·原子写+备份 :757)②plan_authorization_store.rs:51(锁 :73·CAS :75+:838-847 **opt-in,expected=None 跳过**·写 :905-953 含全库唯一父目录 fsync :949)③workflow_execution_entrypoints.rs:288/:354(备份→push dispatch+audit+状态→write_validated 整本 5.9MB;底层 workflow_state_store.rs:69,锁**单次不重试** :186-208,强制 CAS :89-99)④supervisor_action_controller.rs:813 reserve/:874 complete 包夹 adapter.execute :476;update_store :1282-1310(**无备份步**)⑤audit 无公共 helper(workflow_audit.rs 只是 4 个构造器;全库内联 push)⑥workflow_run_dispatch_entrypoints.rs:440(转移合法性 control_core :472;command 在 commands.rs:2069)。
**§5 idempotency**:生成 :1125-1167(sha256 五元组+按动作取材 :1129-1153);查重/恢复 :1025-1063(**reserved 未回写→waiting_user 绝不自动重放**);B1 复核幂等键 global_supervisor_review_store.rs:228-247;导入幂等=natural_key+ON CONFLICT DO NOTHING(apply.rs:136-147)。lib.rs:1081 `stable_id` 只是 slug 化器,内容指纹走 utils::hash::sha256_hex。
**§6 休眠 gating 四层**:无产品调用者(最强,全家现状)/路径闸(Level A temp/fixture 白名单;Level B 确认路径 config 逐字段等值+canonicalize 双向 read_cut.rs:742-796)/禁词路径标记(stop_write.rs:19-34)/决策闸(approve 也只 `ready_but_not_executed` stop_write.rs:345-372,真跑还要 #[ignore]+env 确认串 :1702-1710)。
**§7 revision 双面坑**:读 exporter.rs:213→:486(`ORDER BY {column} LIMIT 1`=字典序);写 apply.rs:664-676(ON CONFLICT DO NOTHING 留旧行;workspace_id 写死 'fixture-workspace' :666);姊妹 exporter.rs:291-295(source_import_meta 连 ORDER BY 都没有)。
**§8 形状**:importer 1321/apply 1751/schema 348/exporter 658/production_apply 2425/dual_write 565/**read_cut 2996(距上限 4 行!)**/stop_write 2017/snapshot_apply 1244/observation_period 1858(+tests.rs 1179 子文件拆法先例)/preflight 955/transaction_acceptance 645;gate NEW_FILE_LIMITS .rs=3000(gate 脚本 :20-24);COMMAND_BASELINE_TOTAL=97(:11)。
**§9 rusqlite**:0.32.1 bundled(Cargo.toml:26);busy_timeout/busy_handler/transaction_with_behavior(Deferred/Immediate/Exclusive)/unchecked_transaction/savepoint/pragma_update 全可用。
**坑清单**:①dual_write 名不符实别当接缝 ②WAL/busy 全从零、别写"沿用现有" ③评审 plan_auth:41-47 坐标≠CAS 本体(散在 :73/:75+:838/:905);两种 busy 语义并存,基准已拍(R1 预算≤1) ④supervisor_actions.idempotency_key 无 UNIQUE,先核 live 30 条重复再定路径 ⑤sidecar 写姿势不齐(有无备份步/fsync 各异),没有"通用姿势"可照 ⑥休眠两层强度,已拍最强层 ⑦最易越界=红线 2 那七个活写文件+codex_db ⑧read_cut 2996/3000 零加行 ⑨transaction_acceptance 是证明不是零件 ⑩revision 坑双面+workspace_id 写死,meta 写语义要一并定义 ⑪家族实为 12 个 .rs+1 tests 子文件,别照抄"5+2"。
