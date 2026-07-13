# Workflow-state 架构风险处置证据 v1

状态：`ARCHITECTURE_SLICE_VERIFIED__NO_SQLITE_CUTOVER__NOT_COMMITTED`

## 本轮边界

- 允许：修复主 JSON 的静默并发覆盖、收口 workflow-state 备份入口、限制备份增长、只读审计 SQLite 漂移。
- 不允许：写 production SQLite、切 read path、stop-write JSON、删除真实历史备份、commit。
- 3b 真实闭环仍是本轮优先目标；本文件记录锁屏等待期间完成的独立低风险架构切片。

## 已落 WIP

1. `workflow_state_store::write_validated` 在文件锁内重新读取当前 revision；stale snapshot 返回 `workflow_state_revision_conflict`，不覆盖首个 writer。
2. workflow-state 的手工 `fs::copy(path, &backup)` 已只剩中央 `workflow_state_store::backup_file` 一处。
3. 中央保留策略：最近 30 份 + 最近 30 个每日恢复点，集合去重后总量不超过 60。
4. 新主管运行材料已搬到 `runtime-artifacts/`；历史 txt 未迁移。

## 真实现状采样

采样时间：2026-07-13。

```text
workflow-state.v0.json bytes = 5,897,201
workflow-state.v0.json mtime = 2026-07-13 12:59:28
main-store backup count = 45
backups directory = 233,520 KiB（约 228 MiB）
workflow-state root regular files = 103（12 JSON + 91 historical txt）

JSON revision = 10
JSON projects/workflows/nodes/edges = 5/8/65/50
JSON dispatches = 363
JSON audit_events = 1473
JSON artifacts = 26
JSON work_items = 57
JSON bindings = 75
JSON execution_attempts = 148
JSON permission_requests = 1
JSON workflow_chain_runs = 37
JSON workflow_execution_controls = 148
JSON workflow_machine_runs = 10
```

旧真库：

```text
path = ~/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/
       b1-production-apply-20260615/workbench-state.v1.sqlite
mtime = 2026-06-15 16:00:49
bytes = 5,017,600

SQLite dispatches = 118
SQLite audit_events = 356
SQLite artifacts = 1
SQLite work_items = 12
SQLite bindings = 36
```

结论：旧库不是当前 JSON 的镜像，不能直接翻闸。

## SQLite 额外阻断项

- importer 白名单不含四个当前主管账本：
  - `global-supervisor-reviews.v1.json`
  - `supervisor-action-control.v1.json`
  - `supervisor-orchestrator.v1.json`
  - `exec-process-registry.v1.json`
- unknown JSON 会进入 `rejected_unknown`；preflight 同样拒绝 unknown JSON。
- 真实 workflow-state 根仍有历史主管 txt；preflight 拒绝非 JSON。

迁移完整性 P0：

- `execution_attempts`、`permission_requests`、`workflow_chain_runs`、`workflow_execution_controls`、`workflow_machine_runs` 五组当前真实数据不在 importer `WORKFLOW_ARRAYS`、apply `workflow_records`、SQLite schema 或 exporter；当前快照重导会丢数据。
- importer 白名单接受 memory-lint/entity/patterns/blackboard，但 apply 对部分来源返回空 records 或未知 record kind `Ok(0)`；当前真实根已有 `memory-lint.v1.json`，不是理论问题。
- exporter 只覆盖主 workflow、formal memory、runtime log、product command、continuation，无法完整回滚当前 sidecar。
- 三个主管持久账本需要正式入库；`exec-process-registry` 属于 OS 进程租约，应明确留在 runtime 层，不能把旧 entry 当历史事实恢复。

因此本轮不做 production apply/read-cut/stop-write，不声明 `ready for cutover`。

## 当前验证

```text
cargo test --lib workflow_state_store::tests:: --quiet
6 passed; 0 failed

cargo test --lib workflow_state_backup_retention_ --quiet
2 passed; 0 failed

rg 手工 workflow-state fs::copy
只剩 workflow_state_store.rs 中央 helper 一处

cargo test --lib --quiet
935 total; 892 passed; 0 failed; 43 ignored

npm run typecheck
通过

npm run test:offline-interaction
通过；offline interaction tests passed: 15

cargo check --offline
通过；569 warnings（既有基线）

cargo fmt --check
仅报历史 codex_db.rs / codex_local_runner.rs / mcp/storage.rs 漂移；未据此改动无关文件

git diff --check
通过
```

以上全库验证于 2026-07-13 以当时工作树重新执行。3b 随后已取得新运行的单 worker、零写根、`dispatch → inspect → finalize(pass) → report_user` 和项目前后逐字节一致证据，见 `evidence/2026-07-13-orchestrator-station3b-mario-test-readonly-real-run-v1.md`。本文件的全库回归数字将在最终收口时再次刷新。

## 未完成风险

- revision conflict 只会明确失败，还没有业务级重放。
- 锁文件没有 stale-lock 恢复。
- 备份与最终 CAS 写仍不是同一事务。
- 主 JSON 仍整本 parse/serialize/rename；CAS 与备份限额都不解决 UI 卡顿。
- 备份只有限份数、没有字节预算；按当前单份约 5.9MB，总量上界仍可能接近 350MB。
- SQLite 尚缺五组主状态数组、主管持久账本、已接受 sidecar 的完整 apply/export、历史 txt 分类迁移、当前 JSON 重新导入和全量对账。
