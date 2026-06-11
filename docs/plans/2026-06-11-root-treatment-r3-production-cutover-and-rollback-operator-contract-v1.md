# Root Treatment R3 Production Cutover And Rollback Operator Contract v1

日期：2026-06-11

状态：R3-A6 contract freeze。本文冻结 R3 从 fixture-only SQLite rehearsal 进入生产前置工作的 cutover、backup、rollback、allowed roots、denied paths、evidence 和后续任务拆分合同。本文不创建生产 DB、不迁移真实 JSON / sidecar、不切产品读写路径、不停写 JSON / sidecar、不解锁多 agent 并行真实执行。

## 0. 结论

R3-A1 到 R3-A5 已证明：

- SQLite schema / importer / apply / export / dual-write / read-cut / observation / rollback verification 可以在 fixture + temp DB + temp projection root 内稳定演练。
- 这些结果仍不是生产迁移，不足以直接切换真实工作台读写路径。

R3-A6 冻结的下一步原则：

```text
先做 production preflight，只读扫描真实工作台自有 JSON / sidecar 的 hash、schema、revision、backup readiness 和 rollback readiness。
再做 copied production snapshot 上的 temp DB 演练。
最后才允许单独任务包申请 production DB apply / read-cut / stop-write。
```

## 1. Production Data Roots

### 1.1 Allowed Roots

后续生产前置任务只能在任务包明确后访问：

| Root class | Contract | Default mode |
| --- | --- | --- |
| `product-line` repo | 源码、任务包、evidence、handoff、fixtures、scripts、git metadata | read/write by task scope |
| Workbench state root | 工作台自有 `workflow-state.v0.json` 和 sibling sidecars 所在目录 | R3-A7 read-only metadata/hash only |
| Workbench backup root | workflow state backups 和 sidecar backups | R3-A7 read-only inventory/hash only |
| Temp DB root | `std::env::temp_dir()` 下由任务包创建的 temp DB / projection / report | read/write |
| Evidence root | `evidence/**` 下的 report / manifest / verification matrix | read/write |
| Handoff root | `handoffs/**` | read/write |

### 1.2 Denied Paths

后续 R3 任务默认禁止：

- `/Users/yoyi/.codex`
- secret / token / `.env`
- keychain / OAuth / provider credential
- full transcript / rollout body
- 用户真实项目源代码内容，除非任务包明确只读取 hash / metadata 且写明 allowed root
- 任意未列入任务包 allowed roots 的路径

### 1.3 Production DB Location

默认生产 DB 仍未创建。后续任务包如需创建，必须先冻结：

| Item | Required contract |
| --- | --- |
| DB file name | 建议 `workbench-state.v1.sqlite`，但必须由 R3-A7 / R3-A8 再确认 |
| DB location | 必须位于 workbench state root 或明确的 workbench-owned storage root |
| Path storage | evidence 中记录 full path；DB 内只保存 path ref / path hash / redaction flag |
| Backup before create | 必须有 JSON / sidecar source root hash、git commit、backup manifest |
| Failure handling | create / migrate / import 失败不得修改 production JSON / sidecar |

## 2. Mode Contract

| Mode | Reads production JSON / sidecar | Writes production DB | Writes production JSON / sidecar | Product read path | Allowed in R3-A6 |
| --- | --- | --- | --- | --- | --- |
| `fixture_only` | no | no | no | unchanged | yes, already covered by A1-A5 |
| `dry_run` | no, fixture/temp only | no | no | unchanged | yes |
| `production_preflight` | metadata/hash/schema/revision only | no | no | unchanged | contract only |
| `copied_snapshot_apply` | copied snapshot only | temp DB only | no | unchanged | contract only |
| `production_apply` | yes | yes | no or manifest only | unchanged | no |
| `read_cut` | yes | yes | no | DB behind feature gate / fallback | no |
| `stop_write_json` | export / backup only | yes | no live writes | DB primary | no |
| `rollback_operator` | manifest / backup only | may disable DB read-cut | may restore projection only after supervisor decision | fallback/restored JSON | contract only |

Rules:

- A mode cannot silently escalate. `production_preflight` cannot create DB. `copied_snapshot_apply` cannot write production root. `read_cut` cannot imply stop-write.
- Every mode must write `mode`, `allowed_roots`, `denied_paths`, `source_root_hash`, `db_path_hash` when applicable, and `production_restore_performed`.
- Failed / blocked / partial modes must not emit `stable_verified`, `completed`, `db_authoritative`, or `stop_write_ready`.

## 3. Backup And Recovery Contract

### 3.1 Required Before Production Apply

Before any future `production_apply`, evidence must include:

- git HEAD before task.
- clean or explicitly classified working tree.
- source root path and source root hash.
- list of JSON / sidecar files with file hash, schema version, revision and size.
- backup manifest hash.
- backup location.
- planned DB path and DB path hash.
- rollback manifest path.
- allowed roots / denied paths.
- dry-run import report.
- copied snapshot apply report.
- DB -> JSON export verification report.
- corruption / interruption failure injection result.

### 3.2 Backup Manifest

Backup manifest must include:

```json
{
  "schema_version": "workbench_sqlite_production_backup_manifest.v1",
  "mode": "production_preflight|production_apply|read_cut|stop_write|rollback",
  "source_root_hash": "...",
  "backup_root_ref": "...",
  "backup_manifest_hash": "...",
  "files": [
    {
      "path_ref": "workflow-state.v0.json",
      "path_hash": "...",
      "file_hash": "...",
      "schema_version": "...",
      "revision": "...",
      "size_bytes": 0,
      "redaction_status": "metadata_only"
    }
  ],
  "db_path_hash": "...",
  "created_by": "global_supervisor",
  "production_restore_performed": false
}
```

The manifest must not include prompt body, full transcript, secret/token/credential, provider credential or rollout body.

### 3.3 Recovery Order

If a production migration task fails:

1. Stop further writes for that task.
2. Preserve DB file for audit unless supervisor explicitly approves deletion.
3. Preserve source JSON / sidecar backups.
4. Disable read-cut feature flag if it was enabled.
5. Verify last known good JSON projection or source backup hash.
6. Write rollback evidence.
7. Do not retry automatically.
8. Require supervisor decision before any restore that writes production JSON / sidecar.

## 4. Transaction And Lock Contract

Future production apply must obey:

- Open SQLite transaction before inserting imported records.
- Hold workflow state / sidecar write lock before touching JSON / sidecar projection or migration marker.
- Never hold lock while executing external Codex or model calls.
- Source JSON revision must be rechecked immediately before commit.
- Corrupt JSON, revision conflict, lock busy or schema mismatch must abort without overwriting original files.
- Crash injection must cover before DB begin, after DB before manifest, after manifest before report, and after report before mode completion.

For cross-domain acceptance, at least one future R3 task must prove candidate adoption across memory + audit in one SQLite transaction:

- candidate consumed / adopted.
- formal memory record created.
- formal memory version created.
- memory audit event created.
- workflow or product command audit ref linked.
- rollback leaves no half-adopted state.

## 5. Read-Cut And Stop-Write Gates

### 5.1 Read-Cut Gate

Future read-cut task cannot start until all are true:

- R3-A6 contract accepted.
- production preflight report exists and is accepted.
- copied production snapshot apply report exists and is accepted.
- DB -> JSON export verification is stable for copied snapshot.
- fallback JSON projection hash matches source root hash or accepted projection hash.
- runtime log, product command, continuation, memory adoption and audit tables have count / hash parity.
- feature flag or explicit read-cut switch exists and defaults off.
- rollback operator dry-run accepted.

### 5.2 Stop-Write JSON Gate

Stop-write JSON cannot start until:

- read-cut has completed with observation period accepted.
- DB is authoritative for selected write path.
- DB -> JSON export can rebuild legacy JSON / sidecar projection.
- at least one rollback drill from DB to JSON projection has passed.
- supervisor writes explicit stop-write decision.
- user-facing / developer-facing docs explain JSON is export / backup / rollback projection, not live write source.

### 5.3 Never Bundle

The following must not be bundled in one task:

- production DB apply + read-cut.
- read-cut + stop-write.
- stop-write + rollback restore.
- SQLite migration + real Codex execution unlock.
- SQLite migration + Stage L / K3-B1 / K3-B2 recovery.

## 6. Rollback Operator Contract

Rollback operator is not an automatic repair daemon. It is a supervised recovery procedure.

| Field | Contract |
| --- | --- |
| Actor | `global_supervisor` with explicit task package / decision |
| Inputs | last verified JSON projection, source backup manifest, DB backup/ref, rollback manifest, export hash, source root hash |
| Outputs | rollback evidence, restored projection only if separately approved, disabled DB read-cut marker, preserved DB audit ref |
| Default restore | false |
| Data deletion | forbidden unless separate user approval and backup evidence exist |
| Retry | no automatic retry |

Required output flags:

- `would_disable_db_read_cut`
- `would_use_last_verified_json_projection`
- `would_preserve_db_for_audit`
- `would_require_supervisor_decision`
- `production_restore_performed`

Production restore can only set `production_restore_performed=true` in a future task that explicitly writes production JSON / sidecar and records user/supervisor approval.

## 7. Evidence And Handoff Contract

Every future R3 production task must record:

- task package path.
- start commit.
- implementation commit.
- checkpoint commit.
- allowed roots.
- denied paths.
- source root hash.
- DB path hash.
- backup manifest hash.
- rollback manifest hash.
- dry-run/apply mode.
- before / after file hashes for touched production files.
- tests and shape gate.
- failure injection result.
- P0/P1/P2.
- boundary confirmation.
- do-not-claim list.

Evidence must distinguish:

- `accepted`
- `accepted_with_p2`
- `done_with_concerns`
- `blocked`
- `failed_classified`
- `partial_evidence_only`

Blocked / failed / partial cannot be upgraded to completed by doc wording.

## 8. Future Task Split

Recommended next tasks:

| Task | Purpose | Writes production data? | Unlocks read-cut? |
| --- | --- | --- | --- |
| R3-A7 | production preflight scanner / report: metadata, hash, schema, backup readiness | no | no |
| R3-A8 | copied production snapshot temp DB apply and export verification | no | no |
| R3-A9 | production DB initializer + apply with backup manifest, no read-cut | yes, DB only | no |
| R3-A10 | limited read-cut for one low-risk read model behind feature flag / fallback | DB read only | limited |
| R3-A11 | production observation period and export verification | DB read/export | no new write path |
| R3-A12 | stop-write JSON decision and rollback drill | yes, if approved | yes |
| R3-A13 | transaction acceptance across memory + audit and R3 final acceptance | yes, scoped | possible final |

R3-A7, R3-A8, R3-A9 Level A, R3-A10 Level A, R3-A11 Level A, R3-A12 Level A, and R3-A13 Level A are complete. R3-A13 is accepted only as fixture / temp SQLite transaction acceptance and cutover gap matrix: memory candidate adoption, formal memory record, formal memory version, formal memory audit event, and workflow audit event are verified in one SQLite transaction, with rollback coverage for before-commit failures. R3-A9 / R3-A10 / R3-A11 / R3-A12 / R3-A13 Level B real workbench-owned production DB, production limited read-cut, production observation, JSON / sidecar stop-write, or final cutover has not executed. The next supervisor step is R4 read model / frontend slimming, or separately decide whether any R3 Level B execution is needed first; no path may read secret / `.codex` / full transcript, cut global product reads to DB, or stop JSON / sidecar writes without a new task package and rollback evidence.

## 9. Boundary Confirmation

This contract does not:

- 创建生产 DB。
- 写用户真实数据目录。
- 迁移真实 `workflow-state.v0.json` 或 sidecar。
- 修改任何真实 JSON / sidecar。
- 切任何产品读写路径到 DB。
- 让真实 app read model 读 DB。
- 停止 JSON / sidecar 写入。
- 把 JSON 降为生产 fallback。
- 在 app startup / Tauri command / UI 中接入生产 SQLite。
- 改 workflow state 顶层 schema。
- 新增 sidecar store 或 sidecar JSON 种类。
- 新增 Tauri command。
- 改真实 Codex runner。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 启动 Stage L / K3-B1 retry / K3-B2。
- 解冻 backlog 功能。

## 10. Do Not Claim

After R3-A6, do not claim:

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
