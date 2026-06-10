# Root Treatment R3-A1 SQLite Schema And Idempotent Importer Dry Run v1 Evidence

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-A1 已完成最小 SQLite schema module、显式临时 / fixture DB initializer、离线 idempotent dry-run importer 和 R3-A1 fixture 矩阵。本轮不提交，提交由主管线回收；未创建生产 DB，未迁移真实 JSON / sidecar，未双写，未切产品读路径，未新增 Tauri command。

## Commit / Worktree

- start commit：`183f30e40c1a89071942e26f486c2396eba4a0b3`
- end commit：无。本开发线按任务包要求不运行 `git add` / `git commit`。
- 初始 `git status --short`：无输出。
- 当前变更仅限 R3-A1 允许写入范围，最终 `git status --short` 见本文件末尾。

## 读取文件

- `tasks/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- R3-P0 evidence / handoff / supervisor checkpoint / supervisor handoff
- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- 相关 store / sidecar / runtime / product command 文件和 `scripts/harness/workbench-shape-gate.js`

## 写入文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a1/**`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1-result.md`

未写入口文档，未写 Root Treatment 官方计划，未写可选 fixture helper。

## Schema Module Summary

新增 `workbench_sqlite_schema.rs`：

- `WORKBENCH_SQLITE_SCHEMA_VERSION = "workbench_sqlite_schema_v0"`。
- `WORKBENCH_SQLITE_SCHEMA_DDL` 覆盖 R3-P0 五组表域：
  - metadata / source / export / rollback。
  - workflow core。
  - memory / observation。
  - workflow governance。
  - runtime / continuation / product command / readback。
- schema 使用 `CREATE TABLE IF NOT EXISTS`，只作为 R3-A1 最小 v0 DDL 常量和测试初始化路径。
- 不新增 migration 文件，不创建生产 DB，不接 app startup。

## Temp DB Initializer Summary

`initialize_temp_workbench_sqlite_db(path: &Path)`：

- 只接受绝对路径。
- 只允许 `std::env::temp_dir()` 或仓库内 `src-tauri/fixtures/r3-a1` 下路径。
- 对其他路径返回 `temp_or_fixture_path_required`。
- 显式打开调用方传入路径，启用 foreign keys，执行 DDL，并写入 `schema_migrations`。
- 不推导生产数据目录，不读取真实 workflow state，不写用户真实目录。

## Dry-Run Importer / Idempotency Summary

新增 `workbench_sqlite_importer.rs`：

- `dry_run_import_fixture_dir(root)` 读取 fixture 目录内 `workflow-state.v0.json` 与允许 sidecar。
- `dry_run_import_fixture_dir_with_previous(root, previous)` 支持第二次 dry-run 的幂等比较。
- report 字段包含：
  - `batch_id`、`mode = dry_run`、`batch_status`、`importer_version`。
  - `source_root_ref`、`source_root_hash`。
  - source inventory：`accepted` / `missing_optional` / `rejected_corrupt` / `rejected_unknown` / `rejected_sensitive`。
  - record summaries：source kind、record kind、natural key、record hash、classification。
  - counts：files / proposed inserts / skips / conflicts / warnings。
  - conflicts / warnings / runtime log alias policy。
- 幂等规则：
  - 同一 natural key + same record hash：`skipped_duplicate`。
  - 同一 natural key + different record hash：`conflict`。
  - second pass with previous report：相同 source kind / record kind / natural key / record hash 标为 `skipped_duplicate`。
  - `expected_revision != revision` 标为 `conflict`。
- importer 不写 DB，不写源 JSON / sidecar，不覆盖 fixture。

## Fixture Coverage Matrix

| 要求 | fixture |
| --- | --- |
| valid-empty-workflow | `fixtures/r3-a1/valid-empty-workflow/workflow-state.v0.json` |
| valid-workflow-core | `fixtures/r3-a1/valid-workflow-core/workflow-state.v0.json` |
| memory-adoption-chain | `fixtures/r3-a1/memory-adoption-chain/{workflow-state.v0.json,formal-memories.v1.json,memory-candidates.v1.json,observations.v1.json}` |
| memory-capture-chain | `fixtures/r3-a1/memory-capture-chain/{workflow-state.v0.json,memory-capture-events.v1.json,observations.v1.json,memory-candidates.v1.json}` |
| proposal-authorization-chain | `fixtures/r3-a1/proposal-authorization-chain/{workflow-state.v0.json,project-proposals.v1.json,plan-authorizations.v1.json}` |
| process-fact-observation | `fixtures/r3-a1/process-fact-observation/{workflow-state.v0.json,observations.v1.json}` |
| product-command-runtime-chain | `fixtures/r3-a1/product-command-runtime-chain/{workflow-state.v0.json,real-execution-product-commands.v1.json,session-continuations.v1.json,runtime-logs.v1.json}` |
| runtime-log-alias | `fixtures/r3-a1/runtime-log-alias/{workflow-state.v0.json,runtime-logs.v1.json,runtime-log.v1.json}` |
| corrupt-primary | `fixtures/r3-a1/corrupt-primary/workflow-state.v0.json` |
| corrupt-optional-sidecar | `fixtures/r3-a1/corrupt-optional-sidecar/{workflow-state.v0.json,memory-candidates.v1.json}` |
| duplicate-same-hash | `fixtures/r3-a1/duplicate-same-hash/workflow-state.v0.json` |
| duplicate-different-hash | `fixtures/r3-a1/duplicate-different-hash/workflow-state.v0.json` |
| revision-conflict | `fixtures/r3-a1/revision-conflict/workflow-state.v0.json` |
| unknown-sidecar | `fixtures/r3-a1/unknown-sidecar/{workflow-state.v0.json,custom-unknown.v1.json}` |
| forbidden-sensitive-field | `fixtures/r3-a1/forbidden-sensitive-field/workflow-state.v0.json` |

`sqlite_importer_dry_run_accepts_contract_fixture_matrix_domains` 覆盖主矩阵有效链路；其余 focused tests 覆盖 corrupt / duplicate / revision / unknown / sensitive / alias。

## Forbidden Sensitive Field Handling

Importer 递归扫描 JSON key 和字符串标记：

- key class：`prompt_body`、secret、token、credential、keychain、OAuth、provider credential、full transcript、transcript body、rollout body。
- string marker：provider credential、full transcript、rollout body、`prompt_body`。

命中后 source 分类为 `rejected_sensitive`，batch 对 forbidden fixture 返回 `rejected_sensitive`，`proposed_inserts = 0`。本轮没有读取外部 secret/token/auth 文件；敏感扫描命中的 `prompt_body` 来自 R3-A1 forbidden fixture。

## Runtime-Log Alias Handling

- `runtime-logs.v1.json` 作为 canonical source kind：`runtime_log`。
- `runtime-log.v1.json` 作为 legacy alias / ref label source kind：`runtime_log_legacy_alias`。
- report 中 `runtime_log_alias` 记录 canonical / legacy 是否存在和 policy。
- R3-A1 不导出 JSON，不新增 sidecar 种类，不把 legacy alias 当 canonical export 目标。

## Shape / Line Metrics

- `lib.rs`：13951 行，只新增 2 行 module declaration。
- `workbench_sqlite_schema.rs`：242 行。
- `workbench_sqlite_importer.rs`：1236 行。
- R3-A1 fixture 文件：31 个。
- Tauri command 总量：96；`lib.rs` 内 command 数量：0。
- shape gate sidecar：14 detected / 0 unknown。

## Evidence / Checks Run

TDD red / green：

- 初始 `cargo test --lib sqlite_schema`：2 tests failed as expected on `workbench_sqlite_schema_unimplemented` / missing `temp_or_fixture_path_required`。
- 初始 `cargo test --lib sqlite_importer_dry_run`：2 tests failed as expected on `workbench_sqlite_importer_unimplemented`。
- 实现后 focused tests passed。

最终验证：

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 13951 行，Tauri commands 96 total / 0 in `lib.rs`，sidecar 14 detected / 0 unknown。
- `cargo test --lib sqlite_schema`：pass，2 passed / 0 failed / 358 filtered。
- `cargo test --lib sqlite_importer_dry_run`：pass，6 passed / 0 failed / 354 filtered。
- `cargo test --lib workflow_state`：pass，11 passed / 0 failed / 349 filtered。
- `cargo test --lib`：pass，344 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：pass，无输出。
- `git diff --check`：pass，无输出。
- 敏感 / 禁止项扫描：完成。命中 `plan_authorization(s)` 表名 / sidecar 名、importer 敏感字段拒绝清单，以及 forbidden fixture 的 `prompt_body`；另行精确扫描 `Command::new("codex")|codex exec|codex exec resume|/Users/yoyi/.codex` 无输出。
- sidecar / workflow source 扫描：完成，命中 importer 允许清单与 R3-A1 fixtures 中的 `workflow-state`、`formal-memories`、`memory-candidates`、`observations`、`runtime-log(s)`、`plan-authorizations`、`project-proposals`、`real-execution-product-commands`、`session-continuations`。
- `git status --short`：见下方。

Filtered cargo tests 均有匹配测试；未发生 no-match。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R3-A1 仅实现 dry-run importer，不实现 apply importer、双写、读切 DB、DB -> JSON export / rollback。
- P2：schema v0 是最小合同表域，字段仍偏 coarse，R3-A2 / 后续 schema implementation 需要细化约束和索引。
- P2：unknown sidecar 只做 dry-run reject / warning；是否纳入 future schema 仍需主管线决策。
- P2：Product Command / continuation / runtime log 单事务仍未实现，本轮只做 dry-run chain fixture 和表域预留。

## Boundary Confirmation

- 未创建生产 DB。
- 未写用户真实数据目录。
- 未迁移真实 `workflow-state.v0.json` 或 sidecar。
- 未修改任何真实 JSON / sidecar。
- 未双写 DB + JSON。
- 未切任何产品读路径到 DB。
- 未在 app startup / Tauri command / UI 中接入 DB initializer 或 importer。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未迁移 R2 inline tests 巨石。
- 未做 R4 前端按页读模型或 UI 瘦身。
- 未运行 `git add` / `git commit`。

## 不能声明完成

- 不能声明 R3 SQLite 迁移开始或完成。
- 不能声明 apply importer 完成。
- 不能声明双写期开始。
- 不能声明读切 DB 完成。
- 不能声明 JSON / sidecar 停写。
- 不能声明 production DB 创建完成。
- 不能声明 transaction boundary 全部实现完成。
- 不能声明 DB -> JSON export / rollback 实现完成。
- 不能声明多 agent 并行真实执行解锁。
- 不能声明 Stage L / K3-B1 / K3-B2 已恢复。
