# M3 — SQLite 迁移·临时副本演练证据 v1

日期:2026-07-13 · 基线 `d952a7d` · 配套 M0 `docs/2026-07-13-sqlite-migration-completeness-contract-m0-v1.md`
演练走 `production_apply.rs` **Level-B** rehearse(`rehearse_production_db_apply_level_b_workbench_owned_state`),临时副本+temp DB,**真实根只读**。

## 一、两个演练测试(均已跑过)

| 测试 | 位置 | 档 | 结果 |
|---|---|---|---|
| `sqlite_m3_synthetic_live_scale_level_b_round_trip` | `workbench_sqlite_production_apply.rs` | 常规套件 | **PASS** |
| `sqlite_m3_live_snapshot_level_b_rehearsal_and_reconcile` | 同上 | `#[ignore]`·需 `WORKBENCH_M3_LIVE_ROOT` | **PASS**(记录 blocker) |

## 二、合成·live-scale Level-B 往返(PASS)

合成主 store(5 数组 + `revision:11`,代表 live 形状)走 Level-B:
- `report.status == "completed"`;`safety_flags.source_json_written == false`;`before_source_hashes == after_source_hashes`(源未被写)。
- **落表计数对账**:execution_attempts=24 · permission_requests=1 · workflow_chain_runs=7 · workflow_execution_controls=24 · workflow_machine_runs=3 —— 源计数 == DB 计数,逐项相等。
- **revision 保真**:导出投影 `revision == 11`(**未被打回 1**;§六①修复生效)。
- **二次导入零新增**:re-apply 后 execution_attempts 仍=24。
- **事务前崩溃**:注入 `TransactionRollbackBeforeCommit` → `Err`(原子回滚,无半成品)。

> 六处一致 + 全源往返另由 `sqlite_apply_export_m1_completeness_round_trips_new_sources`(常规套件·PASS)覆盖:4 层 a/b/c + 3 账本落表 + 导出 + revision=7 保真 + 幂等。

## 三、真实 live 快照演练(PASS·但发现 blocker)

对**真实 live 根只读拷贝**(11 白名单文件:primary + 10 sidecar,剪掉 91 txt + exec-registry + backups/):

```
[M3] staged 11 files: [workflow-state.v0.json, formal-memories, memory-candidates,
     memory-capture-events, memory-lint, observations, plan-authorizations,
     project-proposals, global-supervisor-reviews, supervisor-action-control,
     supervisor-orchestrator]
[M3] dry_run batch_status=rejected_sensitive conflicts=0
     inventory=[("workflow_state","rejected_sensitive")]
```

**⚠ Headline 发现:live 主 store 被 importer 敏感谓词判 `rejected_sensitive`,无法整体往返。**
- 根因:`contains_sensitive_value`(`workbench_sqlite_importer.rs:965`)对 `SENSITIVE_KEY_PARTS` 的 `"token"` 做**子串键匹配**,误命中良性键 `estimated_tokens`/`max_estimated_tokens`(LLM token **计数**元数据,非凭据)。
- 核实:真凭据类标记(secret/credential/prompt_body/keychain/oauth…)在主 store **不作为键出现**;其余 "token" 仅在字符串**值**里,而值只对另一组 `SENSITIVE_STRING_MARKERS`(不含 token)匹配 → **纯子串误报**。
- 影响:**若真翻闸(M5/M6),迁移会静默把整个主 store 判敏感拒收**——这正是 M3「翻闸前演练」要抓的。
- 处置:`contains_sensitive_value` 是**安全谓词**,红线 #3 禁改 → **停下报,不擅改、不绕行、不改 live 数据**。M3 测试在此路径**显式记录 finding 并早退**,不篡改数据凑过。
- 建议(需用户单独授权、另开包):该键匹配改**词界/整键**,或对 `*_tokens` 计数字段 allowlist。

## 四、真实根 0 改动(红线 #1/#2 亲核)

- M3 测试在读取前/后对**整个 live 树递归算 SHA-256**:`live_hash_before == live_hash_after`,值 = `3d8f962eb4dd323b76fc32c578c8fdd3c78683c9a6bf8b23f76ffa5fa7b73b3f`(阻断路径亦核过一致)。
- live 根 `git` 不适用(在 Application Support 外)。主 store 文件 hash = `bf3e6f47…`(M2 清单),演练前后一致。
- 全程只 copy-out / shasum / ls,零写零删零移。

## 五、结论

- **迁移机器完整正确性**:六处一致 + 全源往返 + revision 保真 + 幂等 + 事务原子性 = **代码级证明(合成+fixture 均 PASS)**。
- **真实 live 快照往返**:被**既有安全谓词误报**卡住(非本包 M1 改动引入;M1 前后此谓词行为不变)→ 记 finding,不擅改安全闸。
- **真实根**:0 改动,hash 前后一致亲核。
- **未翻闸**:无 production apply 到真实 DB、无 read-cut/stop-write、无 flag 改动。
