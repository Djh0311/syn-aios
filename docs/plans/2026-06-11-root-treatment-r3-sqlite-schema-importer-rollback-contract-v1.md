# Root Treatment R3 SQLite Schema Importer Rollback Contract v1

日期：2026-06-11

状态：R3-P0 contract freeze。本文只冻结 SQLite schema v0、JSON / sidecar importer、transaction boundary、export / rollback 和 fixture 合同；不创建 SQLite schema 实现，不迁移真实数据，不改产品源码，不改 workflow state JSON，不新增 sidecar，不解锁多 agent 并行真实执行。

## 0. 结论

R3 的第一步不是直接建表或双写，而是把工作台自有事实库合同冻结为：

```text
SQLite 是新的工作台事实库目标；JSON / sidecar 在观察期内仍是可导出、可回滚的兼容格式。
R3-A1 只能做 schema file + dry-run importer + fixture，不切真实读写路径。
```

本合同基于 R2 closing / R3 preflight review、R2-B10 supervisor checkpoint、当前 store 扫描和 workbench shape gate。当前 `rusqlite` 依赖已经存在，但主要用于只读读取 Codex 原生 sqlite，不等于工作台 workflow / sidecar 统一存储已经实现。

## 1. R3 不变量

| 不变量 | 合同 |
| --- | --- |
| 事实库归属 | R3 SQLite 是工作台自有事实库，不是 Codex 原生 `state_*.sqlite`，也不是 `/Users/yoyi/.codex` 的替代写入器。 |
| Codex 数据边界 | 不读取、不导入、不持久化 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout 或 prompt body。 |
| Prompt policy | DB 只允许保存 prompt summary、prompt ref、prompt hash、permission envelope、redaction status 和 runtime / audit refs；prompt body 不进 DB。 |
| JSON / sidecar 观察期 | R3 双写前、双写期、读切 DB 后的观察期都必须保留 DB -> JSON / sidecar export 与旧 JSON 读路径回滚条件。 |
| schema 兼容 | `workflow-state.v0.json` 顶层 schema 在 R3-P0 / R3-A1 不改变；SQLite schema v0 是并行目标合同。 |
| sidecar 种类 | R3-P0 / R3-A1 不新增 sidecar JSON 种类。当前 shape gate 检测 14 种 allowed sidecar，未知种类为 0。 |
| command surface | 不新增 Tauri command，不把 command wrapper 或 registry 当作 R3 范围。 |
| 多 agent 真实执行 | R3 未收口前不解锁多 agent 并行真实执行；R3-P0 不授权真实 `codex exec` / `codex exec resume`。 |
| 迁移顺序 | 先 dry-run importer 和 fixture，再 schema implementation，再双写，再读切 DB，再观察停写 JSON，最后回滚演练。 |
| 完成声明 | 不能把 schema 合同、建表、导入器 dry-run、`rusqlite` 依赖存在、或单次 fixture 通过说成 R3 SQLite 迁移完成。 |

## 2. Store Inventory

| Store / JSON | 当前角色 | R3 归属 | Canonical / alias 策略 |
| --- | --- | --- | --- |
| `workflow-state.v0.json` | workflow v0 主事实层，含 projects、workflows、nodes、edges、work_items、artifacts、reviews、audit_events、capabilities、harness_resources、workflow_node_session_bindings、workflow_node_dispatches 等 | workflow domain、audit domain、task package refs、dispatch refs | 保持 JSON export 语义稳定；schema_version `workflow_state_v0` 和 workflow_version `1` 在 export 中保留。 |
| `formal-memories.v1.json` | 正式记忆记录、版本、审计 | memory domain | 导入 records / versions / audit_events；保留 source refs 和 lifecycle 状态。 |
| `memory-candidates.v1.json` | 记忆候选、候选事件、采纳回链 | memory domain | `candidate_key` 是 natural key，adoption refs 指向 formal memory 表。 |
| `observations.v1.json` | observation 记录、audit events、candidate 回链 | observation / memory domain | `observation_key` 是 natural key；`candidate_key` 是可空引用。 |
| `memory-lint.v1.json` | memory lint runs / findings / store revision guard | memory governance domain | lint findings 不自动成为事实；作为治理检查表导入。 |
| `memory-entity-relations.v1.json` | 记忆实体 / 关系候选和治理结果 | memory graph domain | 仍是关系治理，不等于向量库或 GraphRAG。 |
| `memory-patterns.v1.json` | mature pattern candidates / decisions / audit | memory pattern domain | confirmed pattern 可引用 formal memory output，但 R3 不自动 formalize。 |
| `memory-capture-events.v1.json` | capture bus event，可能派生 observation / candidate | memory capture domain | `event_key` 是 natural key；body / source summary 需按 sensitive policy 过滤。 |
| `blackboard-candidates.v1.json` | 项目黑板候选和决策 | blackboard domain | candidate state machine 单独表化；不能导入为正式事实。 |
| `plan-authorizations.v1.json` | C1/C3 authorization、guard、audit | workflow governance domain | `authorization_id` 是 natural key；scope fingerprint 可作为派生字段。 |
| `project-proposals.v1.json` | C2 proposal、user decision、audit | workflow governance domain | `proposal_id` 是 natural key；authorization 回链可空。 |
| `real-execution-product-commands.v1.json` | Product Command request / preview / decision / attempts | product command domain | `product_command_id`、`attempt_id` 是 natural key；prompt body forbidden。 |
| `session-continuations.v1.json` | continuation、attempt、audit event | continuation domain | `continuation_id`、`attempt_id` 是 natural key；real execution remains frozen unless separately authorized. |
| `runtime-logs.v1.json` | runtime log entries / summaries | runtime domain | canonical file name for persisted runtime log sidecar. |
| `runtime-log.v1.json` | legacy singular ref / descriptor name detected by shape gate | legacy alias only | Importer must treat singular as legacy alias or ref label. R3 export emits only canonical `runtime-logs.v1.json` unless supervisor explicitly approves alias export. |

## 3. Schema v0 Proposal

### 3.1 Metadata And Source Tables

| Table | Purpose | Natural key / uniqueness |
| --- | --- | --- |
| `schema_migrations` | workbench DB schema version and applied_at | `version` |
| `import_batches` | one dry-run or apply import attempt | `batch_id`; also unique `(source_root_hash, importer_version, mode, created_at)` for audit only |
| `import_sources` | one source JSON / sidecar file per batch | unique `(batch_id, source_kind, source_path_hash, source_hash)` |
| `source_records` | normalized source item refs for traceability | unique `(source_id, record_kind, natural_key)` |
| `export_batches` | DB -> JSON / sidecar export attempts | `export_id`; include export_hash and target root |
| `rollback_points` | pre-import / pre-dual-write / pre-read-cut rollback anchors | `rollback_id`; include source backup refs and DB checkpoint refs |

Rules:

- `source_path` can be stored as workspace-relative when possible; absolute paths may be stored only for workbench state files, never for secret/provider credential paths.
- `source_hash` uses SHA-256 over raw file bytes for importer input; `record_hash` uses canonical JSON for source item.
- Dry-run output must include proposed inserts / updates / skips / rejects without writing DB or source JSON.

### 3.2 Workflow Domain

| Table | Source | Natural key / FK policy |
| --- | --- | --- |
| `workflow_state_meta` | workflow state top-level metadata | one row per workspace_id + source root |
| `projects` | `projects[]` | `project_id`; project_root stored as path ref with redaction flag |
| `agent_adapters` | `agent_adapters[]` | `adapter_id`; does not imply credential or live capability verification |
| `workflows` | `workflows[]` | `workflow_id`; FK project_id nullable until importer can resolve |
| `workflow_nodes` | `nodes[]` | `node_id`; FK workflow_id |
| `workflow_edges` | `edges[]` | `edge_id` or `(workflow_id, source_node_id, target_node_id, edge_type)` |
| `work_items` | `work_items[]` | `work_item_id`; FK workflow_id / node_id nullable if legacy |
| `workflow_artifacts` | `artifacts[]` | `artifact_id`; FK work_item_id nullable |
| `workflow_reviews` | `reviews[]` | `review_id`; FK workflow_id / work_item_id nullable |
| `workflow_audit_events` | `audit_events[]` | `event_id`; FK target refs by typed string, not strict for v0 |
| `workflow_node_session_bindings` | `workflow_node_session_bindings[]` | natural `(workflow_id, node_id, work_item_id, lifecycle, session_id)` |
| `workflow_node_dispatches` | `workflow_node_dispatches[]` | `dispatch_id` |
| `capabilities` | `capabilities[]` | `capability_id` or canonical hash |
| `harness_resources` | `harness_resources[]` | `resource_id` or canonical hash |

Contract:

- `workflow_state_meta.workflow_version` remains the JSON v0 value; DB schema version is separate.
- Arrays missing in legacy JSON are importer rejects unless the current JSON validator accepts them as optional compatibility fields.
- `audit_events` remain audit records, not runtime logs and not formal memory.

### 3.3 Memory And Observation Domain

| Table | Source | Natural key / FK policy |
| --- | --- | --- |
| `memory_scopes` | embedded `MemoryScope` | `scope_id`; references project/workflow/session when present |
| `memory_source_refs` | embedded source refs | `source_ref_id` or source hash fallback |
| `formal_memory_records` | formal store records | `memory_id`; FK scope_id |
| `formal_memory_versions` | formal store versions | `version_id`; FK memory_id |
| `formal_memory_audit_events` | formal store audit_events | `audit_event_id`; FK memory_id nullable for split/merge |
| `memory_candidates` | candidate store candidates | `candidate_key`; candidate_id also unique |
| `memory_candidate_events` | candidate events | `audit_ref_id` |
| `observations` | observation records | `observation_key`; observation_id also unique |
| `observation_events` | observation events | `audit_ref_id` |
| `memory_capture_events` | capture bus events | `event_key`; capture_event_id also unique |
| `memory_lint_runs` | lint store | `lint_run_id` or canonical hash if absent |
| `memory_lint_findings` | lint findings | `finding_id` or `(lint_run_id, source_id, rule_id)` |
| `memory_entity_relations` | entity relation store | `relation_id` or canonical hash |
| `mature_pattern_candidates` | mature pattern store | `candidate_id` |
| `mature_pattern_audit_events` | mature pattern audit | `audit_event_id` |
| `blackboard_candidates` | blackboard store records | record id / target key / canonical hash |
| `blackboard_candidate_audit_events` | blackboard audit | `audit_event_id` |

Contract:

- Candidate adoption must link `memory_candidates.candidate_key` to `formal_memory_records.memory_id`, `formal_memory_versions.version_id`, and `formal_memory_audit_events.audit_event_id`.
- Observation -> candidate must link `observations.observation_key` to `memory_candidates.candidate_key`.
- Sensitive summaries may be stored only when current sidecar already stores them and they are not prompt body / secret / credential / full transcript.

### 3.4 Workflow Governance Domain

| Table | Source | Natural key / FK policy |
| --- | --- | --- |
| `project_proposals` | `project-proposals.v1.json` proposals | `proposal_id`; FK project_id/workflow_id |
| `project_proposal_decisions` | proposal decisions | `decision_id`; FK proposal_id |
| `project_proposal_audit_events` | proposal audit | `audit_event_id`; FK proposal_id nullable |
| `plan_authorizations` | `plan-authorizations.v1.json` authorizations | `authorization_id`; FK source_proposal_id nullable |
| `plan_authorization_audit_events` | authorization audit | `audit_event_id`; FK authorization_id nullable |
| `authorized_execution_scopes` | embedded authorization scope | `authorization_id`; one-to-one |
| `stage_c_reviews` | C5/C6 workflow reviews / artifacts | `review_id` / artifact id |
| `stage_c_acceptance_summaries` | generated C6 artifacts | `artifact_id` |

Contract:

- C2 proposal confirmation + authorization creation is one transaction in DB.
- C3 global boundary review + authorization activation is one transaction in DB.
- C5 process fact decision + observation creation is one transaction in DB.
- C6 final review / user decision / acceptance summary remain workflow governance facts, not automatic formal memory.

### 3.5 Runtime, Continuation, Product Command Domain

| Table | Source | Natural key / FK policy |
| --- | --- | --- |
| `product_commands` | Product Command requests | `product_command_id` |
| `product_command_previews` | Product Command previews | `preview_id` or `(product_command_id, preview_hash)` |
| `product_command_decisions` | Product Command decisions | `decision_id` |
| `product_command_attempts` | Product Command attempts | `attempt_id`; FK product_command_id |
| `session_continuations` | continuation store continuations | `continuation_id`; FK product_command_id nullable through refs |
| `session_continuation_attempts` | continuation attempts | `attempt_id`; FK continuation_id |
| `session_continuation_audit_events` | continuation audit | `event_id` |
| `runtime_log_entries` | runtime log entries | `entry_id` |
| `runtime_log_summaries` | runtime summaries | unique `(batch_id, category, status, severity)` or canonical hash |
| `runtime_source_refs` | runtime entry source refs | unique `(entry_id, source_kind, source_id)` |
| `readback_results` | readback summaries from attempts / runtime | `readback_id` or canonical `(attempt_id, source_kind)` |

Contract:

- Runtime log stores redacted operational facts; it never stores prompt body, full transcript, secrets, provider credential, token, or rollout body.
- Product command Phase A/B trace -> continuation -> runtime log is one DB transaction once real migration begins.
- R3-P0 / R3-A1 remains non-executing. Existing historical real execution records may be imported only from workbench sidecars, not by running Codex.

### 3.6 Redaction And Sensitive Fields Policy

| Field class | DB policy |
| --- | --- |
| Prompt body | Forbidden. Store summary/ref/hash only. |
| Full transcript | Forbidden. Store transcript source id, readback marker hash, or redacted summary only. |
| Secret / token / credential / keychain / OAuth / provider credential | Forbidden. Importer must reject files or fields classified as such. |
| Rollout body | Forbidden. Keep source refs or availability status only. |
| Project paths | Allowed only as workbench path refs; include path_hash for joins and display path only where current JSON already stores it. |
| Memory body / claim | Allowed only for existing workbench memory records and candidates; preserve sensitive_level and model_export_policy. |
| Runtime detail | Allowed only when redaction_status is present or can be set to `redacted_by_importer`; sensitive omissions must be explicit. |

## 4. Importer Contract

### 4.1 Inputs

R3 importer input is a directory containing a workbench workflow state file and sibling sidecars:

- Required primary input: `workflow-state.v0.json`.
- Optional sidecars: all 14 allowed sidecar names listed in Store Inventory.
- Optional backups: `backups/workflow-state.v0.*.json` and sidecar backup files may be scanned for rollback metadata but are not imported as live facts by default.

Importer must not read:

- `/Users/yoyi/.codex`.
- secret / token / `.env` / keychain / OAuth / provider credential files.
- full transcript or rollout content.

### 4.2 Source Hash And Batch Output

Each run produces an `import_batch`:

- `batch_id`
- `mode`: `dry_run` or `apply`
- `source_root`
- `source_root_hash`
- `importer_version`
- `started_at`, `finished_at`
- counts: files_seen, files_accepted, files_rejected, records_inserted, records_updated, records_skipped, conflicts, warnings
- `dry_run_report_json`: for dry run only, no DB writes required in R3-A1 unless the schema itself stores dry-run reports.

Each input file produces an `import_source`:

- `source_kind`
- `source_path_hash`
- `source_hash`
- `source_schema_version`
- `detected_revision`
- `status`: accepted / missing_optional / rejected_corrupt / rejected_unknown / rejected_sensitive / skipped_duplicate

### 4.3 Idempotency Keys

| Domain | Idempotency key |
| --- | --- |
| import file | `(source_kind, source_path_hash, source_hash)` |
| workflow arrays | source natural id when present; otherwise `(source_kind, array_name, canonical_record_hash)` |
| audit / runtime events | event_id / entry_id |
| memory candidate | candidate_key |
| formal memory record | memory_id |
| formal memory version | version_id |
| observation | observation_key |
| capture event | event_key |
| plan authorization | authorization_id |
| project proposal | proposal_id |
| product command | product_command_id |
| product command attempt | attempt_id |
| continuation | continuation_id |
| continuation attempt | attempt_id + continuation_id if needed |
| runtime source ref | `(entry_id, source_kind, source_id)` |

Duplicate import behavior:

- Same source hash + same natural key -> skip as idempotent.
- Same natural key + different record hash -> conflict, no overwrite in dry-run; apply mode requires explicit conflict policy.
- Missing optional sidecar -> warning, not failure.
- Missing required `workflow-state.v0.json` -> reject batch.
- Corrupt JSON -> reject that source; if primary workflow state is corrupt, reject batch.
- Unknown sidecar JSON kind -> reject source and require supervisor decision before importer includes it.
- Revision conflict -> record conflict and do not mutate target tables for that source in apply mode.

### 4.4 Import Order

1. Read and validate `workflow-state.v0.json`.
2. Create import batch and source metadata.
3. Import workflow metadata, projects, adapters, workflows, nodes, edges.
4. Import work items, artifacts, reviews, audit events, bindings, dispatches.
5. Import plan authorizations and project proposals.
6. Import memory scopes and source refs.
7. Import formal memories, candidates, observations, capture events, lint, entity relations, mature patterns, blackboard candidates.
8. Import product commands, continuations, runtime logs, readback summaries.
9. Validate cross refs and unresolved refs.
10. Produce dry-run report or apply commit.

Importer must never overwrite source JSON / sidecar files.

## 5. Transaction Boundary

### 5.1 Single-Transaction Boundaries

The following operations must become single SQLite transactions before DB write path is accepted:

| Operation | Tables in one transaction | Current JSON risk |
| --- | --- | --- |
| candidate -> formal memory + memory audit | `memory_candidates`, `formal_memory_records`, `formal_memory_versions`, `formal_memory_audit_events`, source refs | Current flow writes candidate and formal memory sidecars sequentially. |
| observation -> candidate | `observations`, `observation_events`, `memory_candidates`, `memory_candidate_events` | Current flow can update observation before candidate sidecar. |
| memory capture -> observation -> candidate | `memory_capture_events`, `observations`, `memory_candidates` | Capture bus can span three stores. |
| proposal confirmation -> authorization creation | `project_proposals`, `project_proposal_decisions`, `project_proposal_audit_events`, `plan_authorizations`, `plan_authorization_audit_events` | Current proposal and authorization sidecars are separate. |
| global boundary review -> authorization activation | `plan_authorizations`, `plan_authorization_audit_events` plus workflow audit refs | Current authorizations and workflow state reviews can diverge. |
| process fact decision -> observation | `workflow_reviews` / audit event, `observations`, `observation_events` | Current C5 writes workflow state and observation sidecar separately. |
| product command Phase A/B trace -> continuation -> runtime log | `product_commands`, `product_command_attempts`, `session_continuations`, `session_continuation_attempts`, `runtime_log_entries` | Current Product Command store lacks same StoreLock/backup strategy as other sidecars and spans continuation/runtime stores. |

### 5.2 Dual-Write Consistency Strategy

R3 dual-write phase may begin only after R3-A1 dry-run and a later schema implementation task pass fixture gates.

Dual-write order:

1. Validate input and expected revisions.
2. Begin DB transaction.
3. Write DB rows and operation outbox record.
4. Export affected JSON / sidecar projection to temp file.
5. Commit DB transaction.
6. Atomically rename JSON / sidecar temp file.
7. Mark outbox projection status exported.

If step 6 fails after DB commit:

- Product must surface degraded state.
- JSON remains old authority for read path until explicit read-cut task.
- Outbox records pending JSON projection.
- Next repair command can retry export from DB.

If DB transaction fails before commit:

- No JSON / sidecar write is allowed.
- Existing JSON remains authoritative.

### 5.3 Crash Injection Points

Fixture tests must inject failure:

- before DB begin
- after DB begin before first insert
- after first table insert before cross table insert
- after all inserts before commit
- after commit before JSON temp write
- after JSON temp write before rename
- after rename before export status update
- during corrupt JSON read
- during revision conflict
- during duplicate natural key with different hash

## 6. Export / Rollback

### 6.1 DB -> JSON / Sidecar Export

Exporter must support:

- Full `workflow-state.v0.json` reconstruction with existing top-level arrays and metadata.
- Per-sidecar export for every canonical sidecar in Store Inventory.
- `runtime-logs.v1.json` canonical export; singular `runtime-log.v1.json` only as legacy alias if supervisor separately approves.
- Export hash manifest with per-file SHA-256 and record counts.
- Redaction manifest listing omitted forbidden fields.

Exporter must not:

- Emit prompt body.
- Emit full transcript.
- Emit secret / credential / token / rollout content.
- Invent new sidecar kinds.

### 6.2 Rollback Conditions

Return to old JSON read path when:

- DB open or schema migration fails.
- Import batch is rejected.
- Dual-write outbox has unresolved JSON projection failures.
- Export hash verification fails.
- Runtime diagnostics mark DB integrity as corrupt.
- Supervisor chooses rollback during observation period.

Rollback flow:

1. Stop DB read-cut feature flag or equivalent config.
2. Confirm latest exported JSON manifest matches DB export hash.
3. If export unavailable, restore pre-import JSON backups.
4. Mark DB as degraded / read-only for investigation.
5. Keep DB file for forensic diff unless user explicitly authorizes deletion.

### 6.3 Backup Retention

- Preserve current workflow state backup rule: recent 30 + daily 1 for workflow state JSON.
- Sidecar backup retention remains existing store behavior until a later harmonization task; R3 must not silently delete historical backups.
- DB checkpoints must be named by import batch / schema version and have hash manifest.

## 7. Fixture / Tests

### 7.1 Minimal Fixture Set

| Fixture | Purpose |
| --- | --- |
| `minimal_workflow_state_empty` | valid workflow v0 with no workflows beyond adapter baseline |
| `workflow_with_project_work_item_artifact_review_audit` | workflow domain table coverage |
| `formal_candidate_observation_adoption_chain` | candidate -> formal memory + observation links |
| `memory_capture_observation_candidate_chain` | capture bus transaction chain |
| `plan_proposal_authorization_c1_c3` | proposal confirmation and authorization activation |
| `c5_process_fact_to_observation` | workflow review + observation transaction |
| `product_command_continuation_runtime_chain` | Product Command + continuation + runtime log transaction |
| `runtime_log_alias_singular_plural` | canonical / legacy alias behavior |
| `corrupt_primary_workflow_state` | primary JSON reject |
| `corrupt_optional_sidecar` | sidecar reject without primary import loss |
| `duplicate_same_hash` | idempotent skip |
| `duplicate_different_hash` | conflict |
| `revision_conflict` | stale expected revision reject |
| `unknown_sidecar` | reject and require decision |
| `forbidden_sensitive_field` | prompt body / secret field reject |

### 7.2 Required Test Groups

R3-A1 minimum:

- schema file parses / migrates in empty temp DB
- dry-run importer reads fixtures and produces deterministic report
- idempotent dry-run repeated twice yields same planned operations
- corrupt / missing / unknown sidecar cases classified without source mutation
- alias fixture proves `runtime-logs.v1.json` canonical behavior
- `git diff --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`

Later R3 implementation:

- `cargo test --lib workflow_state`
- `cargo test --lib formal_memory`
- `cargo test --lib memory_candidate`
- `cargo test --lib observation`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib real_execution_product_command`
- `cargo test --lib`
- `cargo fmt -- --check`

### 7.3 Acceptance Matrix

| Stage | Must pass | Cannot claim |
| --- | --- | --- |
| R3-P0 | contract doc + evidence/handoff + shape gate + store scans | schema implemented, migration started |
| R3-A1 | schema file + dry-run importer + fixture reports, no read/write path cut | dual-write, DB read path, JSON stopped |
| R3-A2 | idempotent apply importer into temp DB, no real user data mutation | production migration |
| R3-A3 | dual-write behind explicit flag / fixture only | read-cut DB |
| R3-A4 | read-cut DB with rollback rehearsal | JSON permanently removed |
| R3-A5 | observation period + rollback/export verified | multi agent real execution unlocked unless supervisor says so |

## 8. R3-A1 Suggested Task Package

Suggested name:

```text
2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md
```

Execution mode:

- Single-line governance / implementation prep.
- No product read/write path cut.
- No real user data migration.
- No `.codex` access.

Allowed writes:

- New SQLite schema file or Rust module containing schema DDL constants and temp DB initializer.
- New dry-run importer module.
- Test fixtures under a dedicated fixture directory.
- R3-A1 evidence / handoff.

Forbidden:

- Do not create production DB in user data directory.
- Do not add migration runner to app startup.
- Do not write or modify `workflow-state.v0.json` or sidecars.
- Do not change workflow state top-level schema.
- Do not run real Codex.
- Do not switch reads to DB.
- Do not start dual-write.

Required verification:

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

If filters have no matches, the worker must record the exact no-match result and cover with broader related tests; it must not claim filtered tests passed.

## 9. P0 / P1 / P2 For R3 Entry

- P0: none in R3-P0 contract itself.
- P1: R3-A1 must not start without frozen forbidden-field policy and rollback/export manifest shape.
- P1: Product Command / continuation / runtime log chain needs explicit transaction design before any dual-write.
- P2: `runtime-log.v1.json` vs `runtime-logs.v1.json` alias remains a naming debt; canonical export should be plural.
- P2: Store backup retention is not uniform across sidecars; R3 should preserve existing behavior first, harmonize later.
- P2: `real-execution-product-commands.v1.json` currently has atomic temp + rename and revision checks but not the same StoreLock / backup pattern as many other sidecars; DB transaction work should account for this risk.
- P2: R2 inline tests still live in `lib.rs`; R3-A1 should keep tests focused and avoid mixing a broad inline-test migration into schema/importer work.
