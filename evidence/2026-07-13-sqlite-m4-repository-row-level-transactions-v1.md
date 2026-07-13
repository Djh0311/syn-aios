# M4 — SQLite repository row-level transaction evidence v1

日期: 2026-07-13
任务包: `tasks/2026-07-13-sqlite-m4-repository-row-level-transactions-package-v1.md`
起点/终点 commit: `6dd0d337f4942dec92f7e4469799cd7d2ba5e816` / `6dd0d337f4942dec92f7e4469799cd7d2ba5e816`
状态: **DORMANT_ENGINE_READY** — JSON / sidecar 仍是唯一 live path；本包未接 Tauri command、UI、sidecar、startup、runner 或 read-cut。

## 1. 本包完成内容

- 新增内部 `WorkbenchSqliteRepository`，仅可打开临时路径；连接显式启用 `WAL`、`busy_timeout=100ms`、`foreign_keys=ON`。
- 每次写入使用 `BEGIN IMMEDIATE`；busy 仅首试加一次重试，`MAX_BUSY_RETRIES=1`，无无限重试。
- 六个常数行数事务流: audit append；proposal + audit；authorization CAS；dispatch + work-item/node + audit；standalone work-item/node + audit；supervisor reserve/complete/recover-to-waiting。
- supervisor action 幂等键在 live sidecar 只读核实为 `30` 个非空、`0` 个重复后，添加非空 partial unique index。重放返回已有 action，不重派外部 worker。
- commit 前注入失败全部回滚；commit 后、报告前注入失败返回 `committed_but_report_failed`，不暗示外部动作可安全重放。
- R5 导入/导出收敛: source identity 纳入 root hash；workspace identity 保持稳定；同 root 相同 hash 零新增；root 刷新替换 metadata；导出和 memory-candidate read-model 均按最新 import batch 选取。

## 2. 修改文件

| 文件 | 改动 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_repository.rs` | 新增 dormant repository 与 6 个回归测试 |
| `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs` | supervisor action 的非空幂等 unique index |
| `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs` | root-aware source identity 与 metadata 刷新替换 |
| `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs` | 最新 batch metadata 投影 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 仅 module 声明，无命令注册 |

未修改红线文件: `workflow_state_store.rs`、`project_consultation_proposal_store.rs`、`plan_authorization_store.rs`、`supervisor_action_controller.rs`、`global_supervisor_review_store.rs`、`workflow_execution_entrypoints.rs`、`workflow_run_dispatch_entrypoints.rs`、`codex_db.rs`、`workbench_sqlite_read_cut.rs`。

## 3. 验证原样结果

| 命令/核查 | 原样结果 |
|---|---|
| `cargo test --lib` | `test result: ok. 903 passed; 0 failed; 44 ignored; 0 measured; 0 filtered out; finished in 22.82s` |
| `cargo test --lib workbench_sqlite_repository -- --nocapture` | `6 passed; 0 failed` |
| `cargo test --lib sqlite_apply_export_uses_latest_batch_and_replaces_refreshed_root_meta` | `1 passed; 0 failed` |
| `cargo test --lib sqlite_apply_is_idempotent_for_same_root` | `1 passed; 0 failed` |
| `cargo check --offline` | exit `0`；现有 broad warnings `605`，无 fatal |
| `npm run typecheck` | exit `0` |
| `npm run test:offline-interaction` | exit `0`；`15 tests` |
| `git diff --check` | exit `0` |
| `node scripts/harness/workbench-shape-gate.js --mode baseline` | exit `0`；`Status: pass` |
| `node scripts/harness/workbench-shape-gate.js --mode check` | exit `1`；既有 `14 errors / 5 warnings / 5 info`，与 baseline 相同，零 M4 新增缺陷 |
| `cargo fmt --check` | exit `1`；仅既有漂移 `src/codex_db.rs`、`src/codex_local_runner.rs`、`src/mcp/storage.rs`，新增块已人工比对格式 |

shape gate 的 command 核查为 `134 total; 0 in lib.rs`；本包没有 sidecar 写入、没有 Tauri command、没有 ratchet 放宽。

## 4. 事务与并发证据

- 六流测试逐一断言 commit 前失败无半行；commit 后报告失败保留已提交 rows，并对 supervisor reserve/complete 的 terminal state 与 audit 数计数。
- CAS 必须带 expected revision，且仅允许 `expected + 1`；非法 work-item transition 复用 `control_core::validate_work_item_state_transition`。
- 并发 append 回归断言最终精确 `40` rows；busy retry 由常量上界为 `1`。
- 未完成的 reserved supervisor action 仅恢复为 `waiting`，不触发外部 replay。
- 文件长度 `1250` 行，低于 shape gate 的 `3000` 上限。

## 5. R5 最新批次与根刷新证据

真实 `apply_fixture_dir_to_temp_db` 覆盖 root A revision `3`、root B revision `9`、重写 root A revision `12`：

- exporter 先选 revision `9`，A 重导后选 revision `12`；
- memory-candidate sidecar 的 `source_import_meta` 同步先为 `9`、后为 `12`；
- root A 重导后 metadata 总行数仍为 `2`，即每 root 只保留当前 metadata；
- 同 root 相同 hash 的重复导入零新增。

## 6. 真实根只读与边界

M3 算法对 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state` 整棵树递归 SHA-256：M4 开工后与收尾复核均为 `201` files、hash `3d8f962eb4dd323b76fc32c578c8fdd3c78683c9a6bf8b23f76ffa5fa7b73b3f`。全程没有在真实根、生产 DB 或 Application Support 写入。

## 7. 被闸拦过的事件

- 初版 R5 metadata 刷新会使「同 root 重复导入零新增」回归失败；已改为相同 root hash 直接 `Ok(0)`，刷新 root 才替换 metadata，相关回归已通过。
- `cargo fmt --check` 拦出三处历史漂移；未运行写入式 fmt，未格式化非本包文件。
- shape gate check 仍报告既有 `14/5/5`；baseline 与 check 数量一致，未把历史 debt 误报为本包通过。

## 8. 已知未决与下一步

- repository 尚未接到任何产品入口，故未声称 SQLite runtime 生效、JSON read-cut 或 stop-write 完成。
- M3 真实快照仍受既有 sensitive predicate 的 `token` 子串误报阻断；本包未动该安全谓词。
- 后续只有在独立授权包中，才可评估产品接线、双写/影子读、读切换或停写决策。
