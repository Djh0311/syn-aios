# Root Treatment R3-P0 SQLite Schema Importer Rollback Contract Freeze v1 Evidence

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R3-P0 合同冻结已完成。本轮只新增 R3 SQLite schema / importer / transaction / rollback / fixture 合同文档，并写本 evidence 与 handoff；未改产品源码，未创建 SQLite schema 实现，未新增 Rust storage module，未新增 migration，未导入真实数据，未迁移 JSON / sidecar。

当前结论：

- R3 可以进入下一步最小 `schema file + dry-run importer + fixture` 准备，但不能声明 SQLite 迁移开始。
- 当前 `rusqlite` 依赖已存在，但扫描显示主要用于 `codex_db.rs` 只读读取 Codex 原生 sqlite；这不是工作台 workflow / sidecar 统一存储实现。
- 当前 shape gate 仍为 pass：`lib.rs` 13,949 行；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- 合同冻结了 `runtime-logs.v1.json` 为 canonical，`runtime-log.v1.json` 仅作为 legacy alias / ref label 处理，后续 importer 必须显式验证。

## Commit / Worktree

- start commit：`0287d995785df467baf3677e2b03f30165eb7b85`
- end commit：无。本任务包禁止 `git add` / `git commit`，提交由主管线完成。
- 初始 `git status --short`：无输出。

## 读取文件

权威 / 任务材料：

- `tasks/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1-result.md`
- `evidence/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1-result.md`
- `evidence/2026-06-11-root-treatment-r3-p0-task-package-authority-sync-supervisor-checkpoint-v1.md`

代码 / store 现状：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `scripts/harness/workbench-shape-gate.js`

## 写入文件

- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`

未创建可选 R3-A1 任务包草案。原因：任务包允许可选；本合同文档第 8 节已给出 R3-A1 建议任务包名称、读写范围、验证命令和不做项，当前不新增任务入口文件能降低入口文档并发风险。

## Schema v0 摘要

合同文档冻结了以下表域：

- metadata / source：`schema_migrations`、`import_batches`、`import_sources`、`source_records`、`export_batches`、`rollback_points`。
- workflow：`workflow_state_meta`、`projects`、`agent_adapters`、`workflows`、`workflow_nodes`、`workflow_edges`、`work_items`、`workflow_artifacts`、`workflow_reviews`、`workflow_audit_events`、`workflow_node_session_bindings`、`workflow_node_dispatches`、`capabilities`、`harness_resources`。
- memory / observation：`memory_scopes`、`memory_source_refs`、`formal_memory_records`、`formal_memory_versions`、`formal_memory_audit_events`、`memory_candidates`、`memory_candidate_events`、`observations`、`observation_events`、`memory_capture_events`、`memory_lint_runs`、`memory_lint_findings`、`memory_entity_relations`、`mature_pattern_candidates`、`mature_pattern_audit_events`、`blackboard_candidates`、`blackboard_candidate_audit_events`。
- workflow governance：`project_proposals`、`project_proposal_decisions`、`project_proposal_audit_events`、`plan_authorizations`、`plan_authorization_audit_events`、`authorized_execution_scopes`、`stage_c_reviews`、`stage_c_acceptance_summaries`。
- runtime / continuation / product command：`product_commands`、`product_command_previews`、`product_command_decisions`、`product_command_attempts`、`session_continuations`、`session_continuation_attempts`、`session_continuation_audit_events`、`runtime_log_entries`、`runtime_log_summaries`、`runtime_source_refs`、`readback_results`。

Natural key policy：

- Prefer existing stable ids: `workflow_id`、`node_id`、`work_item_id`、`artifact_id`、`event_id`、`candidate_key`、`memory_id`、`version_id`、`observation_key`、`event_key`、`authorization_id`、`proposal_id`、`product_command_id`、`attempt_id`、`continuation_id`、`entry_id`。
- If no stable id exists, use `(source_kind, array_name, canonical_record_hash)` only as importer fallback.
- `source_hash` and `record_hash` are mandatory for dry-run conflict reporting.

Sensitive fields policy：

- prompt body、full transcript、secret、token、credential、keychain、OAuth、provider credential、rollout body 不进 DB。
- 只允许 prompt summary / ref / hash、redaction_status、sensitive omissions、source refs 和 workbench 已有 redacted summaries。

## Importer / Idempotency 摘要

输入范围：

- Required primary input：`workflow-state.v0.json`。
- Optional sidecars：当前 shape gate 允许的 14 种 sidecar。
- Optional backups：可扫描为 rollback metadata，但默认不导入为 live facts。

幂等规则：

- 同一 `source_kind + source_path_hash + source_hash` 重复导入必须 skip。
- 同一 natural key + 同一 canonical record hash 必须 skip。
- 同一 natural key + 不同 record hash 必须 conflict，不允许静默覆盖。
- missing optional sidecar 是 warning；missing primary workflow state 是 batch reject。
- corrupt JSON reject source；primary corrupt reject batch。
- unknown sidecar reject source 并要求 supervisor decision。
- revision conflict 记录 conflict；apply mode 不得局部覆盖。

导入顺序：

1. workflow state validation。
2. batch / source metadata。
3. workflow metadata / projects / adapters / workflows / nodes / edges。
4. work items / artifacts / reviews / audit / bindings / dispatches。
5. plan authorizations / project proposals。
6. memory scopes / source refs。
7. memory / observation / capture / lint / relation / pattern / blackboard。
8. product commands / continuations / runtime logs / readback。
9. cross-ref validation。
10. dry-run report or apply commit。

Importer 不覆盖源 JSON / sidecar。

## Transaction / Rollback 摘要

单事务必须覆盖：

- candidate -> formal memory + memory audit。
- observation -> candidate。
- memory capture -> observation -> candidate。
- proposal confirmation -> authorization creation。
- global boundary review -> authorization activation。
- process fact decision -> observation。
- product command Phase A/B trace -> continuation -> runtime log。

双写期策略：

- DB transaction 先提交事实和 outbox，JSON / sidecar projection 通过 temp + rename 导出。
- DB commit 前失败：不写 JSON / sidecar。
- DB commit 后 JSON rename 失败：进入 degraded / pending export 状态，旧 JSON 仍是读路径权威，后续 repair 从 DB 重导 projection。

Export / rollback：

- DB -> JSON export 必须能重建 `workflow-state.v0.json` 和所有 canonical sidecar。
- Export manifest 必须记录每个文件 SHA-256、record counts 和 redaction omissions。
- DB open / schema migration / import batch / export hash / dual-write outbox / DB integrity 任一失败，可回旧 JSON read path。
- DB 损坏时保留 DB 文件用于 forensic diff，不自动删除。

## Fixture / Verification 摘要

合同冻结的最小 fixture：

- valid empty workflow state。
- project + workflow + work item + artifact + review + audit。
- formal / candidate / observation adoption chain。
- memory capture -> observation -> candidate。
- proposal / authorization C1-C3。
- process fact -> observation。
- product command + continuation + runtime log。
- runtime log singular/plural alias。
- corrupt primary / corrupt optional sidecar。
- duplicate same hash / duplicate different hash。
- revision conflict。
- unknown sidecar。
- forbidden sensitive field。

R3-A1 最小验证：

- shape gate check。
- schema temp DB 初始化。
- dry-run importer deterministic report。
- idempotent dry-run repeated twice。
- corrupt / missing / unknown / alias / sensitive fixture classification。
- `git diff --check`、`git status --short`。

## R3-A1 建议

建议任务包：

`2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`

范围：

- 只做 schema file / temp DB initializer。
- 只做 dry-run importer。
- 只写 fixture 和 evidence / handoff。
- 不切读写路径，不创建生产 DB，不迁移真实 JSON / sidecar，不双写，不访问 `.codex`。

建议验证：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_schema
cargo test --lib sqlite_importer_dry_run
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

## 运行检查

已运行：

```bash
git status --short
git rev-parse HEAD
node scripts/harness/workbench-shape-gate.js --mode check
rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations' prototypes/productized-desktop-shell/src-tauri/src
rg -n 'rusqlite|sqlite|StoreLock|revision|corrupt|backup|rename|\.v1\.json' prototypes/productized-desktop-shell/src-tauri/Cargo.toml prototypes/productized-desktop-shell/src-tauri/src
git log --oneline -8
```

结果：

- 初始 `git status --short`：无输出。
- HEAD：`0287d995785df467baf3677e2b03f30165eb7b85`。
- shape gate check：pass，0 errors / 0 warnings / 12 info。
- shape gate metrics：`lib.rs` 13,949 lines；`real_execution_command.rs` 8,763 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；ratchet files 12。
- sidecar scan：命中 workflow state、formal memories、memory candidates、observations、runtime log singular/plural、plan authorizations、project proposals、real execution product commands、session continuations 等 R3 输入域。
- store guard scan：确认多套 sidecar store 存在 StoreLock / revision / corrupt guard / backup / rename；同时确认 `real_execution_command.rs` 的 product command sidecar 有 revision checks 和 temp rename，但没有同其他 store 一致的 StoreLock / backup pattern。
- Cargo scan：`rusqlite = { version = "0.32", features = ["bundled"] }` 存在。
- `codex_db.rs` scan：`rusqlite` 当前用于只读读取 Codex 原生 sqlite 路径，不是工作台统一存储。

最终写入后已运行：

- `git diff --check`
- `git status --short`

结果：

- `git diff --check`：通过，无输出。
- `git status --short`：仅包含三份预期 untracked 文件：
  - `?? docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
  - `?? evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
  - `?? handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`

可选命令未运行：

- `cargo test --lib workflow_state`
- `cargo test --lib`
- `cargo fmt -- --check`

原因：本轮是 contract freeze 文档线，未改产品源码、未创建 schema、未新增 Rust module；cargo/fmt 不是任务包必跑项，且可能产生 build artifacts。本轮不把未运行的 cargo/fmt 冒充为 fresh verify。

## P0 / P1 / P2

- P0：无。
- P1：R3-A1 不能越过本合同直接切 DB 读写路径或迁移真实 JSON / sidecar。
- P1：R3-A1 必须实现 forbidden-field fixture，防止 prompt body / secret / full transcript 被 importer 接收。
- P1：Product Command / continuation / runtime log 链路在进入双写前必须有单事务和 crash injection 测试。
- P2：`runtime-log.v1.json` / `runtime-logs.v1.json` alias 仍是命名债；合同指定 plural 为 canonical。
- P2：sidecar backup retention 策略不统一；R3 初期只保留现状，不做清理。
- P2：`real-execution-product-commands.v1.json` 写入保护弱于多数 sidecar store，R3 transaction 设计需优先覆盖。
- P2：R2 inline tests 巨石未迁移，R3-A1 应避免同时做测试巨石拆分。

## 禁止项确认

- 未改产品源码。
- 未创建 SQLite schema 实现。
- 未新增 Rust storage module。
- 未新增 migration 文件。
- 未导入真实用户数据。
- 未迁移 JSON / sidecar。
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
- 未运行 `git add` / `git commit`。

## 不能声明完成

- 不能声明 R3 SQLite 迁移开始或完成。
- 不能声明 DB schema 实现完成。
- 不能声明 importer 实现完成。
- 不能声明双写期开始。
- 不能声明读切 DB 完成。
- 不能声明 JSON / sidecar 停写。
- 不能声明多 agent 并行真实执行已解锁。
