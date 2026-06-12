# Root Treatment / R2-T4 Rust Memory Lint Mature Pattern Test Extraction v1

日期：2026-06-12

状态：复核通过，hash 待回填。

Planning baseline commit：`fe1b69c70058e3316c7a6f3020552279757f7397`

Implementation commit：`4d35b3b3e042cd738c0ebd92267508ed5e93fe31`

Review result：`CLEAR`，P0/P1/P2 无；复核线程继续复用 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`TBD`

本文是 Root Treatment / Stage R 的 R2-T4 任务包，承接 R4-A50 新策略和 R2-T1 / R2-T2 / R2-T3 inline tests 迁移结果，继续迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 memory lint、maintenance 和 mature pattern 相关 inline tests 迁出到 crate-root test include 文件，降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 1,000 行以上。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests 及仅这些 tests 内部专用 helper：

- `memory_lint_blocks_conflicting_candidate_adoption`
- `memory_lint_allows_non_conflicting_candidate_adoption`
- `memory_lint_duplicate_claim_generates_finding`
- `memory_lint_authority_superseded_does_not_mutate_formal_memory`
- `memory_lint_revoked_source_excludes_task_packet_memory`
- `memory_lint_open_blocking_finding_excludes_task_packet_memory`
- `memory_lint_maintenance_run_is_readonly_for_formal_memory`
- `memory_lint_damaged_json_is_not_overwritten`
- `memory_lint_revision_conflict_is_rejected`
- `memory_maintenance_run_reports_source_secret_and_index_findings_readonly`
- `memory_maintenance_run_reports_entity_drift_and_relation_revoked_readonly`
- `memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory`
- `fixture_m12_preview_input`
- `prepare_m12_repeated_candidate_fixture`
- `mature_pattern_preview_derives_candidates_and_memory_cluster_reports_readonly`
- `memory_cluster_report_and_unconfirmed_mature_pattern_do_not_enter_task_packet`
- `mature_pattern_user_confirmation_writes_formal_memory_and_task_packet_can_recall`
- `mature_pattern_reject_quarantine_revision_and_damaged_json_do_not_mutate_formal_memory`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_memory_lint_mature_pattern_tests.rs");`
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption、共享 stub runner / factory。
- 迁移 blackboard、observation、task memory packet、workflow state bootstrap / task draft / work item state tests。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib memory_lint`
- `cargo test --lib mature_pattern`
- `cargo test --lib task_memory_packet`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- 复核线只读审查，结论不得有 P0/P1。

## 5. 不接受为

本任务不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- UI / 产品行为修改
- backlog 功能解冻
