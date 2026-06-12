# Root Treatment R3 Level B Execution Window Plan v1

日期：2026-06-13

状态：窗口计划已写；不执行 Level B。

性质：本文只规划 R3 SQLite Level B 执行窗口、前置清单、用户在场点、备份 / 回滚 / 中止 / fresh verify。本文不是 execution record，不授权读取真实 workbench state root，不创建 production DB，不切 read/write path，不停写 JSON / sidecar。

## 0. 当前事实

R3-A9 / A10 / A11 / A12 / A13 均只完成 Level A：

- A9：fixture / temp production DB initializer + apply with backup manifest；未读取真实 workbench state root，未创建真实 production DB，未 read-cut。
- A10：fixture / temp `workflow_state_summary` limited read-cut contract；未切 app startup / Tauri command / UI / 产品全局读路径。
- A11：fixture / temp production observation / export verification contract；未执行真实 observation。
- A12：fixture / temp stop-write JSON decision / rollback drill；未执行真实 stop-write。
- A13：fixture / temp transaction acceptance + cutover gap matrix；未完成真实 production cutover。

R3 Level B 必须按分窗执行，不得把 production DB apply、read-cut、observation、stop-write 和 final acceptance 打包成一个大动作。

## 1. 总体窗口策略

建议拆成 5 个独立窗口，每个窗口都需要单独 execution record、单独用户 / 主管确认、单独 fresh verify：

| 窗口 | 目标 | 预计时长 | 是否可写生产 DB | 是否可切读 | 是否可停写 JSON |
| --- | --- | --- | --- | --- | --- |
| B0 Preflight | 只读确认真实 workbench state root、路径分类、hash / backup readiness | 30-45 min | 否 | 否 | 否 |
| B1 Production Apply | 创建 workbench-owned production DB，导入 allowed JSON / sidecar，生成 backup / rollback manifest | 45-90 min | 是 | 否 | 否 |
| B2 Limited Read-Cut | 仅对 `workflow_state_summary` 启用受控 feature flag + JSON fallback 验证 | 30-60 min | 否，使用 B1 DB | 仅单 read model | 否 |
| B3 Observation | 对 B2 的单 read model 做观察、导出校验、双样本稳定性 | 45-75 min | 否，除 report | 不扩大 | 否 |
| B4 Stop-Write Decision | 只在 B1-B3 全部通过后，评估是否停写 JSON；默认建议先只做 ready/not-ready decision | 45-75 min | 否，除 decision/report | 不扩大 | 仅另行批准后 |
| B5 Final Matrix | 汇总 B1-B4 证据，决定 R3 是否可进入后续收口 | 30-45 min | 否 | 否 | 否 |

默认建议：第一次真实窗口只做 B0。B1 及以后必须在 B0 evidence 通过、路径清单冻结、备份位置确认后另行拍板。

## 2. Level B 前置清单

开始任何 Level B 前必须满足：

- 当前 git HEAD 已记录，工作树变更已分类；若有外部未提交变更，execution record 必须写清不触碰。
- `node scripts/harness/workbench-shape-gate.js --mode check` 通过。
- R3-A9/A10/A11/A12/A13 Level A evidence 与 handoff 可引用。
- 用户确认当前窗口编号：B0 / B1 / B2 / B3 / B4 / B5。
- 用户确认本窗口的 exact allowed roots / denied paths。
- 用户确认本窗口不触碰 `/Users/yoyi/.codex`、secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 用户确认中止条件和回滚策略。
- 本窗口 execution record 已写入 evidence，且包含 before hashes。

B1 及以后还必须满足：

- B0 preflight 已通过并冻结 `WORKBENCH_STATE_ROOT`、`WORKBENCH_STORAGE_ROOT`、`MIGRATION_WORK_ROOT`。
- `MIGRATION_WORK_ROOT` 不在 `WORKBENCH_STATE_ROOT` 内。
- production DB path 不在 source state root 内。
- backup / report / rollback manifest path 不在 source state root 内。
- backup manifest、rollback manifest、report path 均已预留。

## 3. 用户需在场步骤

以下步骤必须用户在场或显式确认：

- B0：确认真实 `WORKBENCH_STATE_ROOT` 和 `WORKBENCH_STORAGE_ROOT` 的路径分类。
- B0：确认是否允许只读读取 allowed JSON / sidecar 的 bytes 以计算 hash。
- B1：确认创建 production DB 的 exact path。
- B1：确认 backup root、rollback manifest path、report path。
- B1：确认 production apply 开始。
- B2：确认 feature flag scope 只限 `workflow_state_summary`。
- B2：确认启用 limited read-cut 前的 rollback / JSON fallback 可用。
- B3：确认 observation window 开始和结束。
- B4：确认 stop-write decision；默认不得实际 stop-write，除非另写 execution record 并再次确认。
- 任意窗口：确认任何会写 production JSON / sidecar 的 rollback / recovery。
- 任意窗口：确认删除、覆盖、清理真实用户数据；默认禁止。

## 4. Root 和路径模板

执行前必须把以下占位符替换为真实路径并写入 execution record：

| 占位符 | 含义 | 约束 |
| --- | --- | --- |
| `WORKBENCH_STATE_ROOT` | 工作台自有 `workflow-state.v0.json` 与 allowed sidecars 所在目录 | B0 只读确认；不得是 `/Users/yoyi/.codex`；不得包含 secret / token / `.env` |
| `WORKBENCH_STORAGE_ROOT` | 工作台自有 SQLite storage root | 必须与 source state root 分离或经合同明确允许；默认不得写用户项目源码目录 |
| `MIGRATION_WORK_ROOT` | 本次 Level B 备份 / 报告 / manifest 工作目录 | 不得位于 source state root 内 |
| `PRODUCTION_DB_PATH` | `workbench-state.v1.sqlite` 目标路径 | B1 才可创建；不得位于 source state root 内 |
| `BACKUP_ROOT` | source JSON / sidecar backup 与 manifest 目录 | 不得位于 source state root 内 |
| `REPORT_ROOT` | execution report 输出目录 | 建议同时写入 `evidence/r3-level-b/<window-id>/` 索引 |
| `ROLLBACK_MANIFEST_PATH` | rollback manifest 文件 | 不得位于 source state root 内 |

Denied paths 固定包含：

- `/Users/yoyi/.codex`
- secret / token / `.env`
- keychain / OAuth / provider credential
- full transcript / rollout body / prompt body
- 未列入 execution record 的任意用户项目源码目录
- 任意未列入 allowed roots 的路径

## 5. Before / After Hash 规则

每个窗口开始前必须记录：

- git HEAD。
- `WORKBENCH_STATE_ROOT` path hash。
- allowed JSON / sidecar 文件清单。
- 每个 allowed 文件的 SHA-256、size、schema version、revision。
- source root aggregate hash：按 path 排序后拼接 path hash + file hash 计算。
- production DB file hash：若不存在，记录 `missing_before=true`。
- backup manifest hash：若尚未生成，记录 `pending_before=true`。
- rollback manifest hash：若尚未生成，记录 `pending_before=true`。

每个窗口结束后必须记录：

- source JSON / sidecar after hash，并与 before 对比。
- production DB hash。
- report hash。
- backup manifest hash。
- rollback manifest hash。
- export verification hash。
- feature flag / read-cut / stop-write 状态。

B1 允许 production DB 从 missing 变为 created，但 source JSON / sidecar after hash 必须等于 before hash。B2-B4 默认不得修改 source JSON / sidecar。

## 6. Execution Record 格式

每个窗口必须在对应 evidence 中追加如下记录：

```json
{
  "schema_version": "root_treatment_r3_level_b_execution_record.v1",
  "window_id": "r3-level-b-b0-preflight-YYYYMMDD-HHMM",
  "phase": "B0_preflight|B1_production_apply|B2_limited_read_cut|B3_observation|B4_stop_write_decision|B5_final_matrix",
  "status": "planned|running|completed|blocked|failed_classified|rolled_back",
  "supervisor": "codex-global-supervisor",
  "user_present_required": true,
  "user_confirmation_ref": "manual-confirmation-or-thread-ref",
  "git_head_before": "",
  "git_head_after": "",
  "allowed_roots": [],
  "denied_paths": [
    "/Users/yoyi/.codex",
    "secret",
    "token",
    ".env",
    "keychain",
    "OAuth",
    "provider credential",
    "full transcript",
    "rollout body",
    "prompt body"
  ],
  "source_root_hash_before": "",
  "source_root_hash_after": "",
  "production_db_path_hash": "",
  "production_db_hash_before": "",
  "production_db_hash_after": "",
  "backup_manifest_hash": "",
  "rollback_manifest_hash": "",
  "report_hash": "",
  "read_cut_scope": "none|workflow_state_summary",
  "stop_write_json": false,
  "source_json_written": false,
  "sidecar_written": false,
  "codex_home_touched": false,
  "product_global_read_path_changed": false,
  "product_global_write_path_changed": false,
  "rollback_or_recovery_performed": false,
  "fresh_verify": [],
  "do_not_claim": []
}
```

## 7. Fresh Verify 清单

B0 必须通过：

- Path classification check：allowed roots / denied paths 无冲突。
- Source inventory hash report。
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

B1 必须通过：

- `cargo test --lib sqlite_production`
- `cargo test --lib sqlite_export`
- `cargo test --lib sqlite_apply`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- source before / after hashes unchanged。
- production DB exists, DB hash recorded, backup / rollback manifest hash recorded。

B2 必须通过：

- `cargo test --lib sqlite_read_cut`
- `cargo test --lib sqlite_production`
- `cargo test --lib`
- JSON fallback proof for `workflow_state_summary`。
- feature flag defaults to off unless explicitly enabled for the selected model.
- no app startup / Tauri command / UI product path cutover.

B3 必须通过：

- `cargo test --lib sqlite_observation`
- export verification two-sample stability.
- source hashes unchanged.
- fallback still available.

B4 必须通过：

- `cargo test --lib sqlite_stop_write`
- stop-write decision matrix.
- rollback drill dry-run.
- if actual stop-write is requested, a separate execution record and user confirmation are mandatory before action.

B5 必须通过：

- R3 gap matrix updated.
- No P0/P1 from review line.
- Explicit accepted / deferred / blocked / not-executed classification.

敏感扫描每个窗口都必须包含：

- `.codex` 命中分类。
- `secret|token|\\.env|keychain|OAuth|provider credential|full transcript|rollout|prompt body` 命中分类。
- `codex exec|codex exec resume` 命中分类。

## 8. 中止条件

任意命中以下条件必须立即停止当前窗口：

- 用户不在场或未确认本窗口 exact scope。
- allowed roots / denied paths 无法写清。
- 发现 source root 或 backup root 位于 `/Users/yoyi/.codex`。
- 发现 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout body、prompt body。
- before hash 无法计算或存在不明变更。
- source JSON / sidecar 在非授权窗口发生 hash 变化。
- backup manifest 或 rollback manifest 无法生成。
- DB schema mismatch、integrity failure、export hash mismatch、fallback hash mismatch。
- read-cut 范围超过 `workflow_state_summary`。
- stop-write 被要求与 production apply / read-cut 同窗口执行。
- 任何任务需要改 app startup、Tauri command、UI、产品全局读写路径，但当前窗口未授权。
- `cargo test --lib`、shape gate 或 `git diff --check` 失败且无法分类。
- 复核线发现 P0/P1。

停止后必须写 `failed_classified` 或 `blocked`，不得把 partial 说成 completed。

## 9. Rollback / Recovery 序列

Rollback 不是自动修复。默认只做 dry-run，真实恢复必须另行用户确认。

标准序列：

1. 停止当前窗口，不再写新文件。
2. 记录 failure point、git HEAD、source before / after hashes、DB hash、manifest hash。
3. 禁用 limited read-cut feature flag（如曾启用）。
4. 保留 production DB 作为 audit 证据，默认不删除。
5. 校验 last verified JSON projection 或 source backup hash。
6. 生成 rollback evidence。
7. 如需恢复写 production JSON / sidecar，必须另写 execution record 并取得用户确认。
8. 恢复后重新计算 source root hash、DB hash、report hash。
9. 复跑 fresh verify。

禁止：

- 自动删除 production DB。
- 自动覆盖 source JSON / sidecar。
- 在没有 manifest 的情况下恢复。
- 把 rollback dry-run 说成真实 recovery 已完成。

## 10. 必须再次拍板的动作

以下动作不能仅凭本文执行：

- B1 production DB apply。
- B2 limited read-cut enable。
- B3 production observation window。
- B4 actual stop-write JSON / sidecar。
- 任何真实 rollback / recovery 写 source JSON / sidecar。
- 删除、覆盖、清理真实用户数据。
- 将 app startup / Tauri command / UI / 产品全局读写路径接入 DB。
- 将 R3 结论升级为 complete。
- 解锁多 agent 并行真实执行。
- 恢复 Stage L / K3-B1 / K3-B2。

## 11. 建议排期

建议不要同日连续执行 B1-B4。推荐节奏：

1. 第一次窗口：B0 preflight，只读路径和 hash，结束后给用户看报告。
2. 第二次窗口：B1 production DB apply，不 read-cut。
3. 第三次窗口：B2 limited read-cut，只限 `workflow_state_summary`。
4. 第四次窗口：B3 observation / export verification。
5. 第五次窗口：B4 stop-write decision，优先输出 `ready_but_not_executed` 或 `not_ready`。
6. 第六次窗口：B5 final matrix。

如果任一窗口失败，不顺延下一窗口；先分类、写 evidence、复核、再决定是否 retry。

## 12. 不接受为

本文不接受为：

- R3 Level B 已执行。
- 真实 workbench state root 已读取。
- production DB 已创建。
- production apply 已执行。
- production read-cut 已执行。
- production observation 已执行。
- JSON / sidecar 已停写。
- rollback / recovery 已验证于真实数据。
- R3 已完成。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触已授权。
