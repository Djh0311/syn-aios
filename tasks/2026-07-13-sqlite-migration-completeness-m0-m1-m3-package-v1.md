# 任务包:SQLite 迁移完整性 —— M0 合同冻结 + M1 六处补全 + M3 临时副本演练(+M2 只出计划)v1

日期:2026-07-13 · 档位:**轻档为主**(设计+代码+临时副本·产品行为不变=JSON 仍是活库·只补**休眠**的迁移机器)。**明确不碰高危清单**:不进真实 workflow-state 目录写、不翻闸、不改安全闸/沙箱/审批逻辑。
勘察正本:本包坐标全部来自 2026-07-13 只读勘察(交叉核过 live JSON + app-data 旧库),见文末「勘察坐标 & 修正」。

> **状态门**:本包只把 SQLite 迁移做到「可证明完整正确」,**不做** production apply / read-cut / stop-write。M4(产品级 repository/行级事务)是本包核清后的下一包;**M5-M6(真切换/停写)拍板锁着、需用户单独授权那一下,不在本包**。

## 所属开发线

执行线(架构/存储线)。总指导写包+核实物;执行线一次实现 M0/M1/M3 与 M2 计划件;**不 commit**(总指导收口)。

## 背景

正本 `docs/2026-07-13-architecture-review-v1.md` §四/§八 指认三条 P0,当前 SQLite 迁移链一翻闸即静默丢数据。勘察把它精确化为:

1. **主 store 数组漏合同**:`execution_attempts(148)`/`permission_requests(1)`/`workflow_chain_runs(37)`/`workflow_execution_controls(148)` 四组**活**数组 + `workflow_machine_runs(10)` 一组**死**数组(无 writer·key 已封),五个顶层 key 在迁移四文件 grep 全零。
2. **"接受但丢弃"是两层+一潜伏**(不是评审说的一处):(a) importer↔apply 落表缺口(memory-lint 等 4 源:apply `records_for_source` 无 arm→`apply.rs:454 Vec::new()`、`source_kind_for_file`→`apply.rs:907 unknown_sidecar` 被主循环跳过);(b) apply↔exporter 导出缺口(memory-candidates 等 5 源已落表却不在 `exporter.rs:74-108` 投影);(c) 潜伏 `apply.rs:872 Ok(0)` 未来 kind 不匹配静默吞。
3. **三主管账本无 schema/import/export**;`exec-process-registry` 是 OS 进程租约(pid+started_at+cmdline),**归档都不该导**(导入=错误复活死进程租约,评审判断成立)。

评审明写:**这三条没清之前禁止翻闸**。本包按 §八 M0→M1→M3 顺序清干净并为 M2 出计划件。

## 目标(交付物)

- **M0** 迁移完整性合同(落 `docs/` 或 `decisions/`):真实根 12 JSON + 91 txt + 主 JSON 顶层字段逐项分类=**持久事实 / 兼容投影 / runtime 临时件 / 历史归档**;未知项 **fail-closed**。`workflow_machine_runs`、91 txt、`exec-process-registry` 均归「历史归档/runtime 租约」,**不当活领域事实导入**。
- **M1** 六处一致补全(见「勘察坐标」§3 的六处):四组活数组 + 三主管账本 + 当前已接受 sidecar 的完整**落表+导出**;消除 (a)(b) 两层不对称;给 (c) `Ok(0)` 潜伏点改成**未知 kind fail-closed 报错**而非静默吞;修 `exporter.rs:166` `revision unwrap_or(1)` 隐患(见修正 6)。`workflow_machine_runs` 按**归档表**落(只入不当活数组接线)。
- **M3** 冻结快照在**临时副本**全量重建演练:从**当前 live 根的只读拷贝**建 DB,`JSON→SQLite→JSON` 对账 + rollback + 失败注入,**走 `production_apply.rs` Level-B rehearse**(核心四是 fixture-gated,不能只跑其 fixture 测)。
- **M2(只出计划)** 91 txt 分类 + 搬迁计划 + hash 清单当交付物;**不执行真实搬迁**。

## 允许读取

- `prototypes/productized-desktop-shell/src-tauri/src/**`,重点:`workbench_sqlite_{importer,apply,schema,exporter,production_apply,snapshot_apply,transaction_acceptance,preflight,read_cut,dual_write,stop_write,observation_period}.rs`、`workflow_state_store.rs`、`workflow_execution_entrypoints.rs`、`global_supervisor_review_store.rs`、`supervisor_action_controller.rs`、`supervisor_session_launcher.rs`、`exec_process_registry.rs`、`types.rs`
- `docs/2026-07-13-architecture-review-v1.md`、`evidence/2026-07-13-workflow-state-architecture-risk-remediation-v1.md`、`evidence/r3-level-b/**`
- **只读采数**(只 copy-out / 算 hash / 计数,严禁写):live 根 `~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/`;旧库参照 `~/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/`

## 允许写入

- `src/workbench_sqlite_*`(importer/apply/schema/exporter/production_apply 及相邻)+ 其 `#[cfg(test)]`/rehearse 脚手架
- 临时副本目录(M3):`/tmp/**` 或 scratchpad —— **不得**写真实 workflow-state 根
- 新文档:M0 合同、M2 搬迁计划+hash 清单(`docs/` 或 `evidence/`)

## 禁止事项(红线)

1. **不写/不搬/不删真实 workflow-state 根任何文件**(M0/M3 只读采数+临时副本;M2 只出计划)。
2. **不翻闸**:不 production apply 到真实 DB、不 read-cut、不 stop-write、不改任何"哪个是活库"的开关或 flag。
3. **不改安全闸/沙箱/审批逻辑**(高危#3);碰到安全谓词就停下报。
4. **默认禁新增 sidecar JSON 种类**;真需要先落 `decisions/`+用户确认。
5. **`#[tauri::command]` 不得写进 `lib.rs`**;本包预期**零新增 command**。
6. **不删历史**:91 txt / 旧库 / 旧备份一个不删(M2 只归类+留 hash)。
7. **不动主 store 活路径**:`workflow_state_store.rs` 的 read/write/atomic 活库路径本包不碰;若 M1 逼你动它 = 越界到活库,**停下报**。
8. 卡住/歧义/发现勘察或评审说错 → **停下报**,不擅自扩权。

## 变更辐射面

- **改了什么假设**:迁移链从"部分源静默漏"→"全字段六处一致的完整合同"。
- **谁依赖旧假设**:preflight 的 unknown 拒绝、importer 白名单、exporter round-trip、既有 `workbench_sqlite_*` rehearse。逐个在验收里核往返一致。
- **主 store 无 struct**:五组是 `serde_json::Value` 顶层 key(`types.rs:2864-2866` 是 read-model 不是容器)——别去改 struct 字段,改错地方无效。

## 形状影响

- 任务类型:**功能+治理混合**(补休眠迁移机器=功能;产品行为不变+往返一致=治理口径)。
- 新增代码落点:`workbench_sqlite_*` 六处 + schema DDL + 各 rehearse 测试;M0/M2 文档。
- 棘轮文件:预期**不碰** `lib.rs`/`ProjectJiaobanPanel.tsx`;若 record 类型镜像逼动 `types.rs`,列进回传。
- 预计行数:schema/import/apply/export 各 +〔执行线估〕;测试 +〔估〕。
- 新增 Tauri command:**否**。 新增 sidecar 种类:**否**。 shape gate 豁免:**不需要**。
- 本任务基线 commit:`d952a7d`。 完成 commit:〔总指导收口填〕。

## 验收标准(预写死)

- **M0**:12 JSON + 91 txt + 主 JSON 顶层字段全部且仅分类一次;每个 accepted source 有明确落表+导出策略;未知项 fail-closed(测试或明单证明);`workflow_machine_runs`/exec-registry/91txt 明确归档、非活事实。
- **M1**:六处一致(importer 白名单 · `records_for_source` · `insert_domain_record` · `source_kind_for_file` · schema DDL · exporter projection)——grep 证明 `apply.rs:454 Vec::new()` / `:907 unknown_sidecar` 缺口已消或列明保留处+理由;`apply.rs:872 Ok(0)` 改未知 kind fail-closed;`exporter.rs:166 revision` 不再 `unwrap_or(1)` 丢真值。
- **M3**(临时副本·走 `production_apply.rs` Level-B):当前冻结快照 `JSON→SQLite→JSON` 顶层字段/四活数组计数/natural key/record hash 语义一致;**revision 保真(live=10 不被打回 1)**;二次导入零新增零冲突;事务前崩溃无半成品、事务后"已提交但报告失败"能识别;**真实根 0 改动**(git status + 真实根 hash 前后不变为证)。
- **M2**:91 txt 精确分类覆盖 + hash 清单;搬迁计划写清搬到哪/怎么回滚/preflight v2 只忽略明确 runtime;**真实根未动**。
- **通用**:`node scripts/harness/workbench-shape-gate.js --mode baseline` 与 `--mode check` 摘要;`git diff --check` 过;`cargo test --lib` 全绿(基线 893/0/43·只增不减)+ 新增迁移测试计数;`cargo fmt --check` 不新增漂移(历史三漂移照旧)。

## 必须回传(10 项)

1 做了什么 · 2 改了哪些文件 · 3 新增测试/证据 · 4 哪些结论有依据 · 5 哪些仍不确定 · 6 风险+下一步 · 7 shape gate baseline/check 摘要 · 8 start/end commit(执行线无 git 则标 `no_git_blocked_for_r2_r3`)· 9 是否新增 command/sidecar/碰棘轮文件 · 10 **被闸拦过的事**(勘察硬闸/fixture-gate/path 拒/锁竞态…无也写"无")。

## 总指导回收动作

核实物:临时副本往返对账亲验 + 真实根 hash 前后不变亲核 + 重跑 `cargo test --lib` + 扫 diff 确认没碰活库路径/安全闸/真实状态目录 → 接受/需改/暂停/废弃 + 依据。核清后 commit(问一次);M4 另开包。

---

## 勘察坐标 & 修正(2026-07-13 只读勘察·执行线照此定位)

**四文件(真名属实·核心四 fixture-gated·真入口 `production_apply.rs` delegate 到四者):**
- importer `workbench_sqlite_importer.rs`:`dry_run_import_fixture_dir` :124;白名单 `WORKFLOW_ARRAYS` :28-45(13 组)、`OPTIONAL_SIDECARS` :12-27(14 个)
- apply `workbench_sqlite_apply.rs`:`apply_source_root_to_db` :89;`records_for_source` :310(缺 arm→:454 `Vec::new()`);`source_kind_for_file` →:907 `unknown_sidecar`;`insert_domain_record` :534(兜底 :872 `Ok(0)`);fixture 硬闸 :272
- schema `workbench_sqlite_schema.rs`:`WORKBENCH_SQLITE_SCHEMA_DDL` :7;已有表 :99-105(memory_lint/entity_relations/patterns/blackboard 表已建但 apply 不落);fixture 硬闸 :205
- exporter `workbench_sqlite_exporter.rs`:`export_*_dry_run` :29/:42;`workflow_state_projection` :160(revision :166 `unwrap_or(1)` 隐患);只投影 5 文件 :74-108;fixture 硬闸 :136

**五组数组(主 store 顶层 key·无 struct·`serde_json::Value`):** execution_attempts=148(`workflow_execution_entrypoints.rs:587,730`,`WorkflowExecutionAttemptRecord` types.rs:4980)· permission_requests=1(`lib.rs:14280`,types.rs:4964)· workflow_chain_runs=37 · workflow_execution_controls=148(types.rs:4946)· **workflow_machine_runs=10=死数据**。

**三主管账本 + registry:** global-supervisor-reviews `global_supervisor_review_store.rs`(store :159/record :41/audit :81/boundary :108)· supervisor-action-control `supervisor_action_controller.rs`(`SupervisorActionRecordV1` :365)· supervisor-orchestrator **无独立 store·内联** `supervisor_session_launcher.rs:2013,2278`· exec-process-registry `exec_process_registry.rs`(`ExecProcessRegistryStore` :38 / `RegisteredProcess` :18)=OS 租约不导。

**可复用 rehearse(fixtures/r3-a2):** `transaction_acceptance.rs`(单事务原子+失败分类:83/:495/:531/:572)· `snapshot_apply.rs`(拷贝快照 import→export hash 等价+rollback:191/:478/:548)· `production_apply.rs`(Level-B:213/:240 delegate 核心四)· `preflight.rs`(:113/:124)。

**修正(勘察纠评审):** ①`workflow_machine_runs` 死数据(无 writer·`run_workflow_machine_at` `workflow_execution_entrypoints.rs:1496` 已封)→归档非活数组。 ②主 store 无 struct,改 `types.rs:2864` read-model 无效。 ③病灶两层六处(见 §背景 2),补合同漏一处即新不对称。 ④live 根现只 `memory-lint.v1.json`(844B)真命中漏,余为潜伏。 ⑤核心四 fixture-gated,验收走 production_apply Level-B。 ⑥`exporter.rs:166 revision unwrap_or(1)` vs live=10:`workflow_state_meta` 缺失会把 revision 打回 1→假对账。 ⑦旧库 06-15 非基线,M3 从当前冻结快照建。
