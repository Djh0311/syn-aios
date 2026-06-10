# Root Treatment R2 Closing / R3 Preflight Review v1 Evidence

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

R2-B10 后的只读复核已完成。结论是：

- `lib.rs` 当前 13,949 行，已低于 R2 第一阶段 `<= 15,000` 水位线，但仍不是 R2 全部完成。
- `lib.rs` 剩余主代码已经收缩到若干明确区块：transcript/index loader、task package render/finder、shared workflow utility、atomic/path/time helper、workbench snapshot assembly 和 inline tests 巨石。
- inline tests 仍是最大风险：`#[cfg(test)] mod tests` 位于 `lib.rs:1703-13949`，约 12,247 行，当前静态统计有 213 个 `#[test]`。
- R3 SQLite 可以进入前置冻结和 importer 设计，但不应直接开始写迁移。当前 JSON/sidecar 写入已分散在多套 store，StoreLock、revision guard、corrupt guard 和 backup 策略不完全一致。
- 推荐下一任务包先做 `R3-P0 SQLite schema / importer / rollback contract freeze`，只做 schema、导入器输入、fixture、回滚和验证矩阵冻结；不直接迁移写路径。

本轮只写 evidence / handoff，不改产品源码，不迁移 SQLite，不提交。

## Commit / Worktree

- 任务包记录的 R3 preflight 基线 commit：`b7c7276`
- 本复核线 start HEAD：`489b18f36e217bf10f761118bf303a8b92c057ed`
- 本复核线 completion commit：无。本线按任务包禁止 `git add` / `git commit`，提交由主管线完成。
- 初始 `git status --short`：无输出。

## 读取文件

必读文件已读取或按任务包聚焦复核：

- `tasks/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
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
- `evidence/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1-result.md`
- `tasks/2026-06-11-root-treatment-r2-b10-c4-c6-automation-workflow-governance-extraction-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/c4_c6_workflow_governance_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- store / runtime 相关文件：`formal_memory_store.rs`、`memory_candidate_store.rs`、`observation_store.rs`、`memory_lint_store.rs`、`memory_entity_relation_store.rs`、`mature_pattern_store.rs`、`blackboard_candidate_store.rs`、`memory_capture_bus.rs`、`plan_authorization_store.rs`、`project_consultation_proposal_store.rs`、`runtime_log_store.rs`、`session_continuation_store.rs`、`real_execution_command.rs`、`project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`

## 验证命令

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
git status --short
rg -n "include!|#\\[cfg\\(test\\)\\]|fn test_|#\\[test\\]" prototypes/productized-desktop-shell/src-tauri/src/lib.rs
wc -l prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/*.rs
git log --oneline -12
rg -c "#\\[test\\]" prototypes/productized-desktop-shell/src-tauri/src/lib.rs
```

结果：

- `workbench-shape-gate --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- shape gate metrics：`lib.rs` 13,949 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；ratchet `lib.rs` 13,949 / 25,925，status `decreased`。
- `git status --short`：初始无输出。
- `rg include/test`：确认 `lib.rs` 仍包含 crate-root `include!` 汇合点和 inline tests；主测试模块从 `1703` 行开始。
- `wc -l`：确认 `lib.rs` 13,949 行，新 R2 helpers 均低于 Rust 3,000 行阈值；`workflow_execution_entrypoints.rs` 2,068 行，`workflow_read_model_entrypoints.rs` 2,066 行，`c4_c6_workflow_governance_entrypoints.rs` 2,509 行。
- `git log --oneline -12`：当前 HEAD 为 `489b18f docs: add r2 closing r3 preflight review task`，前序包括 R2-B10 checkpoint/backfill 和 completion。
- `rg -c "#\\[test\\]" lib.rs`：213。

可选命令未运行：

- 未运行 `cargo test --lib workflow_state`、`cargo test --lib`、`cargo fmt -- --check`。
- 原因：本任务是只读复核线，只允许写两个报告文件；cargo/fmt 会触发 build artifacts 或格式检查流程，且本轮不改产品源码。R2-B10 supervisor checkpoint 已 fresh-run 并记录这些命令通过，本复核结论不把未重跑的 cargo/fmt 冒充为本轮 fresh verify。

## lib.rs 剩余结构矩阵

| 区块名 | 当前行号范围 | 职责 | 是否适合 R2 后段抽出 | 推荐拆分方式 | 风险 | 推荐测试 |
| --- | --- | --- | --- | --- | --- | --- |
| crate 装配 / AppState / legacy product command boundary | `1-117` | 模块声明、`AppState`、`types.rs` / `commands.rs` / `command_registry.rs` include、legacy product command blocking wrapper | 部分适合；最终应只保留装配 | 暂留 crate root；legacy boundary 可随真实执行 command surface 后续收敛 | 牵涉 command wrappers 和 registry，过早正式 `mod` 会扩散可见性 | `node scripts/harness/workbench-shape-gate.js --mode check`、`cargo test --lib legacy_real_execution_entrypoints_are_blocked_for_product_routing`、`cargo test --lib` |
| index / transcript loader | `119-243` | index JSON 读取、Codex sqlite/index transcript fallback、rollout catalog source 标记 | 适合小批次抽出 | 正式 `mod transcript_catalog_entrypoints` 更合适，复用 `codex_db.rs` / `codex_transcript.rs`；也可先 `include!` | 涉及 `.codex` 路径推导和 transcript fallback，必须保持只读和错误分类不变 | `cargo test --lib transcript`、`cargo test --lib codex_transcript`、`cargo test --lib dispatch_readback_stats`、`cargo test --lib workbench_snapshot` |
| R2 include 汇合点 | `244-252`、`867-870`、`1213`、`1698-1701` | R2-B2 到 R2-B10 已抽出的 crate-root include | 不作为业务抽出对象 | 后续单独任务评估从 `include!` 收敛正式 `mod` | 正式 `mod` 需要大规模可见性修改；容易改变私有 helper 边界 | 对应每个 helper 的既有 focused filters 加 `cargo test --lib` |
| task package render / finder helper | `254-866` | work item / permission / artifact finder、任务包字段归一、任务包 Markdown 渲染、readiness warning/blocking、用户审核指令预览 | 适合 R2 后段优先抽出 | 建议 `include!("task_package_render_entrypoints.rs")` 先搬位置；后续再正式 `mod` | 与 memory packet input、task package artifact schema、dispatch readiness 紧耦合；错误文案必须不变 | `cargo test --lib task_package_preview`、`cargo test --lib task_package_fields_update`、`cargo test --lib task_package_file_generation`、`cargo test --lib task_package_dispatch_readiness`、`cargo test --lib task_memory_injection` |
| shared workflow utility | `872-1210` | work item state transition labels、node state update、index project/thread lookup、stable IDs、blackboard enum names、task package output path、atomic new text file | 适合，但要先分类 | 拆成 `workflow_shared_helpers.rs` 或分成 `workflow_identity_helpers.rs` + `task_package_file_helpers.rs`；先 `include!` | 这是多域共享层，直接正式模块化会引入可见性和循环依赖；`current_date_prefix` 受 env 测试影响 | `cargo test --lib workflow_state_transition`、`cargo test --lib workflow_node_state_transition`、`cargo test --lib task_package_file_generation`、`cargo test --lib work_item_state_update` |
| dispatch ID / safe probe / atomic path / time helpers | `1215-1298` | dispatch id、safe probe marker、last message compact summary、default workflow dispatch output dir、`atomic_write_json`、default workflow state path、workspace/time helpers | 适合小批次抽出，但要避开真实执行语义 | 建议 `include!("workflow_runtime_path_time_helpers.rs")`；`safe_probe_*` 可等待 workflow execution 正式模块化 | 默认路径和时间 helper 被多处测试依赖；safe probe 字符串不可改；`HOME` 推导不可变更 | `cargo test --lib workflow_node_dispatch`、`cargo test --lib workflow_machine`、`cargo test --lib workflow_state` |
| workbench snapshot assembly | `1299-1695` | `WorkbenchSnapshot` 组装、session source overlay、adapter/provider/continuation/runtime log/worker protocol/project automation/diagnostics 聚合 | 适合 R2 后段或 R4 前置抽出 | 建议 `include!("workbench_snapshot_assembly.rs")`，不要在 R3 前重写读模型 | 牵涉 18 个顶层 snapshot 字段和多 sidecar 读取；R4 会按页读模型，R3 前只搬不改 | `cargo test --lib workbench_snapshot`、`cargo test --lib diagnostic`、`cargo test --lib provider_availability`、`cargo test --lib session_continuation`、`cargo test --lib agent_adapter` |
| inline tests 巨石 | `1703-13949` | 213 个 `#[test]`，覆盖 transcript、diagnostics、adapter/provider/session、C1-C6、memory、workflow state、task package、dispatch、workflow machine 等 | 适合 R2 后段专项迁移，不建议夹进业务抽出 | 单独 `R2-T1 inline tests migration`，按领域分批迁到 helper/module-local tests；共享 fixtures 先抽 `test_support` | 最大风险是 fixture 共享、crate root 私有函数依赖、迁移后 filter 名称变化导致任务包验证失真 | 每批迁移前后跑对应 focused filters + `cargo test --lib`；shape gate check |
| test-only task package path helper | `1091-1114` | `next_available_task_package_path` 仅测试用 | 可随 task package tests 迁移 | 放入 task package test support 或 task package helper tests | 只在测试中使用，低行为风险；迁移时避免改日期 env fallback | `cargo test --lib task_package_file_generation` |

## inline tests 矩阵

| 测试模块位置 | 主要测试域 | 优先迁移测试组 | 目标文件建议 | 不建议现在迁移的组 |
| --- | --- | --- | --- | --- |
| `1703-2490` | K3-B command guard、path whitelist、snapshot metadata、G2 diagnostics、agent adapter/provider/session continuation boundary | snapshot/diagnostic/provider/session descriptor tests 可先迁 | `diagnostics_provider_session_entrypoints.rs` 或 `workbench_snapshot_assembly.rs` 的 local tests | K3-B guard 牵涉 Stage L/K 冻结语义，建议保持到真实执行治理恢复前 |
| `2505-2878` | transcript catalog、sqlite/index fallback、dispatch readback stats | transcript/readback tests 优先迁，收益高且边界清晰 | `codex_transcript.rs`、`codex_db.rs`、`index_host_app_entrypoints.rs` 或新 `transcript_catalog_entrypoints.rs` | 需要真实 `.codex` 的测试不得新增；保持 temp sqlite/rollout fixture |
| `2898-3445` | 跨域 fixtures：transcript、memory、plan authorization、C4/C5/C6、observation、task packet | 先抽共享 test support，不直接搬业务测试 | `src-tauri/src/test_support.rs` gated by `#[cfg(test)]` 或每个 helper 内局部 fixture | 不建议在没有 fixture 分层前批量迁移，容易造成循环依赖和可见性扩散 |
| `3457-4914` | C1-C6 authorization/proposal/global boundary/project director/C5/C6 | R2-B10 helper 已抽出，C4-C6 tests 可作为第二优先迁移；C1-C3 可等 plan/proposal store边界确认 | `c4_c6_workflow_governance_entrypoints.rs` local tests；C1-C3 可迁到 `plan_authorization_store.rs` / `project_consultation_proposal_store.rs` | 不建议和 R3 SQLite 同批迁移，避免测试迁移与存储语义变更叠加 |
| `5709-6640` | memory lint、maintenance、mature pattern | memory lint / mature pattern store tests 可分批迁 | `memory_lint_store.rs`、`mature_pattern_store.rs`、`mature_pattern_governance.rs` | 跨 formal/candidate/observation 的端到端记忆链先等 R3 transaction schema 冻结 |
| `6787-7277` | workflow state init/bootstrap/task draft/work item state/audit helper | workflow_state_store helper tests 和 lifecycle tests 可迁 | `workflow_state_store.rs`、`workflow_state_lifecycle_task_package.rs`、`workflow_run_dispatch_entrypoints.rs` | workflow top-level schema 迁移前，不改测试断言的 JSON shape |
| `7304-9284` | blackboard candidate、observation、task memory packet、entity relation、formal memory、candidate adoption | store-local corrupt/revision/context guard tests 可迁 | `blackboard_candidate_store.rs`、`observation_store.rs`、`task_memory_packet_builder.rs`、`memory_entity_relation_store.rs`、`formal_memory_store.rs`、`memory_candidate_store.rs` | candidate adoption 跨 candidate + formal store，建议作为 R3 transaction fixture 重点，不宜先拆散 |
| `9348-11320` | node session binding、task package preview/readiness/file generation/field correction/task memory injection | task package render/finder helper若先抽，相关 tests 同批迁 | 新 `task_package_render_entrypoints.rs` + `workflow_state_lifecycle_task_package.rs` | 真实任务包文件生成测试写临时文件，迁移时须保留 overwrite guard 和 path fixture |
| `11359-12290` | workflow node dispatch prepare/execute/readback/user reviewed instruction/timeout/failure | 等 workflow execution helper 正式边界或 test support 抽出后迁 | `workflow_execution_entrypoints.rs` local tests | 不建议现在迁移执行 stub runner 组，涉及 `CodexResumeRunner` trait 和真实执行禁区语义 |
| `12422-13149` | workflow ledger、exception、interfaces、dispatch director review、offline role | 可按 R2-B5/B6 helper 分批迁 | `workflow_read_model_entrypoints.rs`、`workflow_execution_entrypoints.rs` | 与 workflow machine 端到端 fixture 共用较多，先抽小组，不做整段搬迁 |
| `13200-13949` | 任务包真实文件生成确认、fixture factories、stub runners、read_json_file | 先拆 test support，再拆具体测试 | `test_support` 或每个领域 local fixture | 不建议单独迁移 runner fixtures；它们支撑多个测试域 |

## R3 SQLite Preflight 矩阵

| JSON / sidecar store | StoreLock / corrupt / revision guard 现状 | 表候选 | 导入器输入 | 双写风险 | 回滚 / 导出策略 | fixture 需求 |
| --- | --- | --- | --- | --- | --- | --- |
| `workflow-state.v0.json` | `workflow_state_store.rs` 有 `.workflow-state.v0.lock`、写前读取防 corrupt overwrite、temp + rename、备份保留最近 30 + 每日 1；revision guard 通过上层 validate callback，不是统一 DB transaction | `workflow_state_meta`、`projects`、`agent_adapters`、`workflows`、`workflow_nodes`、`workflow_edges`、`work_items`、`artifacts`、`reviews`、`audit_events`、`capabilities`、`harness_resources`、`permission_requests` | 当前 `workflow-state.v0.json` 及 `backups/workflow-state.v0.*.json`；导入需记录 source hash、schema_version、workflow_version | 多个入口 read-modify-write 同一个 JSON；R3 双写期若 DB 写成功但 JSON 写失败，会出现 authority 分裂 | DB 每次迁移前备份；提供 DB -> JSON export，导出必须可重建现有 v0 shape；导入幂等以 source hash + natural keys 去重 | missing file、corrupt JSON、revision conflict、lock busy、重复导入、导入后 JSON export byte-stable 或 semantically stable |
| memory stores：`formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json`、`memory-entity-relations.v1.json`、`memory-patterns.v1.json`、`memory-capture-events.v1.json` | 各 store 基本有 sidecar lock、corrupt guard、atomic write 和 revision；但 backup retention 多为最近 20，策略不完全统一；跨 store 操作靠顺序写 | `memory_records`、`memory_versions`、`memory_audit_events`、`memory_candidates`、`observations`、`memory_lint_runs`、`memory_lint_findings`、`memory_entity_relations`、`memory_patterns`、`memory_capture_events` | sibling sidecars + workflow state project/workflow context；需保留 revision 和 audit refs | candidate -> formal adoption、observation -> candidate、lint blocking 与 task packet recall 现在跨 JSON 文件，双写期最容易半写 | 单事务覆盖跨 store 写入；DB -> sidecar export 支持逐 store 回滚；保留 old sidecar read-only fallback 一个观察期 | candidate adoption 半写注入、observation candidate creation 半写、lint corrupt JSON、entity relation revision conflict、mature pattern confirmation |
| workflow governance stores：`plan-authorizations.v1.json`、`project-proposals.v1.json`、C4-C6 写入 `workflow-state.v0.json` | plan/proposal store 有 lock、corrupt guard、expected revision；C4-C6 仍写 workflow state arrays | `plan_authorizations`、`authorization_reviews`、`project_proposals`、`proposal_decisions`、`stage_c_reviews`、`stage_c_acceptance_summaries` | plan/proposal sidecars + workflow state reviews/artifacts/audit_events | C2/C3/C4/C5/C6 跨 proposal、authorization、workflow state、observation sidecar；双写会扩大不一致面 | R3 应先定义 transaction boundary：proposal confirmation + authorization creation、global review + authorization activation、process fact decision + observation | C1-C6 happy path、missing user confirmation、proposal/auth mismatch、C5 observation write fail rollback、C6 summary export |
| runtime / execution stores：`session-continuations.v1.json`、`runtime-logs.v1.json`、`real-execution-product-commands.v1.json` | continuation/runtime log 有 lock、corrupt guard、atomic write、backup；product command store 有 revision checks和 temp rename，但未看到同等 StoreLock/backup 统一策略 | `product_commands`、`product_command_decisions`、`session_continuations`、`continuation_attempts`、`runtime_log_entries`、`runtime_log_summaries`、`readback_results` | three sibling sidecars + workflow state refs + redacted prompt summary/ref/hash | Phase A/B 可能写 product command + continuation + runtime log，JSON 顺序写很难保证跨文件原子性 | 真实执行仍冻结；R3 先用 stub/fixture transaction 测试；DB export redacts prompt body，保持不持久化正文 | product command revision conflict、continuation corrupt blocks before partial write、runtime log corrupt preflight、duplicate dispatch blocked |
| blackboard / candidate sidecar：`blackboard-candidates.v1.json` | 有 lock、corrupt guard、expected revision、atomic write、backup | `blackboard_candidates`、`blackboard_candidate_decisions` | sidecar + workflow state context | 与 workflow fact / memory candidate promotion边界有关，错误双写会把 candidate 当事实 | DB 内保留 candidate state machine，export 仍可恢复 sidecar | direct promotion rejected、candidate decision revision conflict、damaged JSON |
| legacy / naming debt：`runtime-log.v1.json` 与 `runtime-logs.v1.json` | shape gate 同时检测到两种名称；代码主常量为 `runtime-logs.v1.json`，部分 descriptor / refs 仍出现 singular 名称 | 迁移前必须冻结 canonical store name 和 legacy alias policy | 全仓扫描 + shape gate sidecar list | alias 未收敛会导致 importer 漏读或误判新增 sidecar | importer 明确 singular 是 legacy alias / ref name 还是历史文件；导出只输出 canonical | fixture 同时存在 singular/plural，验证 importer 行为 |

R3 关键判断：

- 依赖层面已有 `rusqlite = { version = "0.32", features = ["bundled"] }`，但当前主要用于只读读取 Codex 原生 sqlite；这不等于工作台 workflow / sidecar 统一存储已经存在。
- R3 不应从“建表”直接开工。先冻结 schema、importer input、idempotency keys、rollback/export 和 crash injection fixture。
- R3 完成标准必须包含至少一次跨记忆 + 审计的单事务写入，以及中断注入不产生半写状态；不能把建库建表冒充 R3 完成。

## 下一任务建议

| 项 | 建议 |
| --- | --- |
| 任务包名称 | `2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md` |
| 执行模式 | 单线治理任务。R3 完成前不建议多 agent 并行真实执行，也不建议多线同时改存储写路径。 |
| 目标 | 冻结 SQLite schema v0、JSON/sidecar importer 输入包、idempotency / conflict 策略、DB -> JSON export / rollback 策略、fixture 列表和后续 R3-A1 最小 importer 验证矩阵。 |
| 允许读 | `src-tauri/src/*store.rs`、`workflow_state_*`、`real_execution_command.rs`、`runtime_log_store.rs`、`session_continuation_store.rs`、`project_workflow_automation.rs`、`types.rs`、R0/R1/R2 evidence/handoff、shape gate。 |
| 允许写 | 建议仅写 `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-contract-v1.md`、对应 evidence/handoff、可选 R3-A1 任务包草案。不改产品源码。 |
| 验证命令 | `node scripts/harness/workbench-shape-gate.js --mode check`、`rg` sidecar/store scan、`git diff --check`、`git status --short`；如写 schema fixtures 再加 JSON schema/fixture lint。 |
| 是否高风险 | 中风险。文档/contract freeze 低风险；一旦进入 actual schema/importer 和双写就是高风险。 |
| 是否需要用户再授权 | R3-P0 文档/contract freeze 不需要新的用户授权。实际迁移真实用户 workflow state、清理 sidecar、停写 JSON 或触碰 `.codex` 都必须另起任务并按边界重新确认。 |

备选但不推荐作为下一步：

- 直接 R3-A1 建 schema / importer：过早，当前 rollback/export/fixture 尚未冻结。
- 继续 R2-B11 抽 snapshot assembly：可降低 `lib.rs`，但 R3 是多 agent 并行真实执行硬门槛，优先级低于 R3-P0。
- 立即 R2-T1 inline tests migration：必要但不应抢在 R3-P0 前；建议在 R3-P0 后并行准备或作为 R3-A1 前的辅助治理。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮是静态只读复核，未 fresh-run cargo/fmt；复核结论不声称本轮 cargo/fmt 通过。
- P2：行号基于 HEAD `489b18f36e217bf10f761118bf303a8b92c057ed`；后续任务开始前需重新确认。
- P2：R3 schema、importer、transaction boundary 和 rollback/export 仍未冻结；本轮只给推荐，不代表 R3 开始或完成。
- P2：inline tests 仍留在 `lib.rs`，R2 后段仍需要测试迁移专项。
- P2：`include!` 仍是 R2 保守过渡，后续正式 `mod` 需要可见性审查。

## 边界确认

- 未改产品源码。
- 未迁移 SQLite。
- 未创建 schema。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未新增 Tauri command。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未运行 `git add` / `git commit`。

## 不能声明完成

- 不能声明 R2 全部完成。
- 不能声明 R3 SQLite 迁移开始或完成。
- 不能声明多 agent 并行真实执行已解锁。
- 不能声明 Stage L / K3-B1 / K3-B2 恢复。
- 不能声明真实 Codex 执行授权。
- 不能声明 inline tests 巨石已拆完。
