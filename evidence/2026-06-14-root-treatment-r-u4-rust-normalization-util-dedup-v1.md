# Root Treatment / R-U4 Rust Normalization Util Dedup v1 Evidence

日期：2026-06-14

状态：实现与本地验证完成，独立复核待回收。

Planning baseline：`0a27b91`

Task package commit：`b5964b6 docs: add r-u4 normalization util dedup package`

## 1. 本包目标

本包只把规则完全相同的 Rust normalization helper 收敛到 `src-tauri/src/utils/normalization.rs`，不强合规则不同或涉及业务语义的 normalization。

完成内容：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs`。
- 新增公共 helper `normalize_slash_lowercase(value)`，规则为 `value.trim().replace('\\', "/").to_lowercase()`。
- `utils/mod.rs` 注册 `normalization` 模块。
- 10 个同形 `fn normalize(value: &str)` 本地定义改为公共 helper alias。
- `control_core.rs::normalize_symbol` 保留函数名，仅 wrapper 到公共 helper。

## 2. 修改范围

代码文件：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`

未修改：

- 未修改 JSON / sidecar / workflow state schema。
- 未修改 store 业务规则、状态机、权限、runner、Codex 执行参数。
- 未迁 SQLite，未进入 R3 Level B。
- 未实现或接入 U-Gate。
- 未启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未解冻 backlog。

## 3. Deferred 清单

本包未合并以下 normalization：

- `memory_capture_bus.rs::normalize`：规则为 `value.trim().to_ascii_lowercase()`，不做 slash normalize，且 ASCII lowercase；保留原地。
- `mature_pattern_governance.rs::normalize`：trim + lowercase 后保留 ascii alphanumeric / whitespace / 非 ASCII，用于 mature pattern key；保留原地。
- `c4_c6_workflow_governance_entrypoints.rs::normalize_c4_symbol`：额外把 `-` 替换为 `_`；保留原地。
- `workflow_execution_entrypoints.rs::normalize_director_review_decision`：带业务枚举校验；保留原地。
- `codex_transcript.rs` 的 path canonical / normalized path：路径处理；保留原地。
- `control_core.rs::normalized_absolute_path`：路径解析和校验；保留原地。
- 敏感路径检测中的局部 lowercase：安全检测逻辑；保留原地。
- `alias_key`、tokenize、candidate key、pattern key 等特化逻辑：只共享其底层同形 normalize，不抽业务特化。

## 4. 扫描记录

命令：

```bash
rg -n "^fn normalize\\(value: &str\\)|^fn normalize_symbol|normalize_slash_lowercase" prototypes/productized-desktop-shell/src-tauri/src --glob '*.rs'
```

输出：

```text
prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/control_core.rs:5:use crate::utils::normalization::normalize_slash_lowercase;
prototypes/productized-desktop-shell/src-tauri/src/control_core.rs:626:fn normalize_symbol(value: &str) -> String {
prototypes/productized-desktop-shell/src-tauri/src/control_core.rs:627:    normalize_slash_lowercase(value)
prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs:957:fn normalize(value: &str) -> String {
prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs:540:fn normalize(value: &str) -> String {
prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs:1:pub(crate) fn normalize_slash_lowercase(value: &str) -> String {
prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs:10:    fn normalize_slash_lowercase_trims_slashes_and_lowercases() {
prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs:12:            normalize_slash_lowercase("  Foo\\Bar\\BAZ  "),
prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs:18:    fn normalize_slash_lowercase_preserves_inner_whitespace() {
prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs:19:        assert_eq!(normalize_slash_lowercase(" A  B "), "a  b");
prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs:2:use crate::utils::normalization::normalize_slash_lowercase as normalize;
```

说明：剩余本地 `fn normalize(value: &str)` 仅为 deferred 的 `memory_capture_bus.rs` 与 `mature_pattern_governance.rs`。`control_core::normalize_symbol` 是保留语义名称的 wrapper。

## 5. 验证记录

### 5.1 `cargo fmt -- --check`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

输出为空。

结果：通过。

### 5.2 `cargo test --lib memory_candidate_store`

```text
running 1 test
test tests::memory_candidate_store_keeps_candidates_out_of_formal_memory ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 497 filtered out; finished in 0.05s
```

### 5.3 `cargo test --lib formal_memory_store`

```text
running 6 tests
test tests::formal_memory_store_rejects_candidate_status ... ok
test tests::formal_memory_store_rejects_missing_source_refs ... ok
test tests::formal_memory_store_damaged_json_is_not_overwritten ... ok
test tests::formal_memory_store_creates_record_version_and_audit ... ok
test tests::formal_memory_store_revision_conflict_is_rejected ... ok
test tests::formal_memory_store_keeps_candidate_store_separate ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 492 filtered out; finished in 0.04s
```

### 5.4 `cargo test --lib session_continuation_store`

```text
running 20 tests
test session_continuation_store::tests::h2_phase_b_real_mario_test_probe_requires_env_authorization ... ignored, requires explicit H2 Phase B real codex resume authorization
test session_continuation_store::tests::h3_b_real_new_session_fixture_probe_requires_env_authorization ... ignored, requires explicit H3-B real codex new-session authorization
test session_continuation_store::tests::h5_level_b1_real_mario_test_project_workflow_dispatch_requires_env_authorization ... ignored, requires explicit H5-Level-B1 real project workflow dispatch authorization
test session_continuation_store::tests::h5_level_b2_real_mario_test_project_workflow_write_probe_requires_env_authorization ... ignored, requires explicit H5-Level-B2 real project workflow write-probe authorization
test result: ok. 16 passed; 0 failed; 4 ignored; 0 measured; 478 filtered out; finished in 0.32s
```

### 5.5 `cargo test --lib codex_local_runner`

```text
running 12 tests
test codex_local_runner::tests::codex_local_guard_blocks_new_session_without_work_item_binding ... ok
test codex_local_runner::tests::codex_local_guard_blocks_secret_paths_and_prompt_hash_gap ... ok
test codex_local_runner::tests::codex_local_guard_blocks_path_escape_and_sensitive_readback_refs ... ok
test codex_local_runner::tests::codex_local_guard_blocks_planned_adapter_duplicate_and_missing_confirmation ... ok
test codex_local_runner::tests::codex_local_guard_allows_new_session_noop_without_existing_session ... ok
test codex_local_runner::tests::codex_local_guard_allows_confirmed_structured_dry_run_only ... ok
test codex_local_runner::tests::h4_unknown_result_statuses_keep_result_count_null ... ok
test codex_local_runner::tests::h2_phase_a_noop_runner_records_no_real_execution ... ok
test codex_local_runner::tests::h2_phase_b_fake_runner_keeps_failed_readback_count_unknown ... ok
test codex_local_runner::tests::h2_phase_b_classifies_codex_state_readonly_without_zero_results ... ok
test codex_local_runner::tests::h2_phase_b_fake_runner_records_real_execution_flags_and_readback ... ok
test codex_local_runner::tests::h2_phase_a_runner_classifies_timeout_and_readback_failed_without_zero_results ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 486 filtered out; finished in 0.00s
```

### 5.6 `cargo test --lib control_core`

```text
running 2 tests
test worker_protocol::tests::worker_protocol_i5_cli_parity_requires_control_core_permission_and_audit ... ok
test tests::workflow_permission_decision_records_audit_through_control_core ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 496 filtered out; finished in 0.03s
```

### 5.7 `cargo test --lib`

```text
test utils::normalization::tests::normalize_slash_lowercase_preserves_inner_whitespace ... ok
test utils::normalization::tests::normalize_slash_lowercase_trims_slashes_and_lowercases ... ok

test result: ok. 482 passed; 0 failed; 16 ignored; 0 measured; 0 filtered out; finished in 10.59s
```

### 5.8 Shape gate

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

输出摘要：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 9
Git HEAD: b5964b6b1faf077342626d662fee28db41ab5352

Key metrics:
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- real_execution_command.rs: 8754 lines (prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs)
- ProjectsView.tsx: 378 lines (prototypes/productized-desktop-shell/src/views/ProjectsView.tsx)
- AgentView.tsx: 285 lines (prototypes/productized-desktop-shell/src/views/AgentView.tsx)
- types.rs: 5229 lines (prototypes/productized-desktop-shell/src-tauri/src/types.rs)
- types.ts: 43 lines (prototypes/productized-desktop-shell/src/lib/types.ts)
- styles.css: 8464 lines (prototypes/productized-desktop-shell/src/styles.css)
- offline-permission-dialog.test.tsx: 3404 lines (prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx)
- Tauri commands: 97 total; 0 in lib.rs
- Sidecar JSON kinds: 14 detected; 0 unknown
```

复核线 Hilbert 复跑 shape gate 时确认 `session_continuation_store.rs` 为 `5218/5237 (decreased)`；该数值为本包最终记录值。

### 5.9 `git diff --check`

命令：

```bash
git diff --check
```

输出为空。

结果：通过。

## 6. 当前 git 实物

### 6.1 `git status --short`

```text
 M prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs
 M prototypes/productized-desktop-shell/src-tauri/src/control_core.rs
 M prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs
 M prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs
 M prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs
 M prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs
?? prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs
```

### 6.2 `git diff --stat`

```text
 .../src-tauri/src/blackboard_candidate_store.rs                      | 5 +----
 .../productized-desktop-shell/src-tauri/src/codex_local_runner.rs    | 5 +----
 prototypes/productized-desktop-shell/src-tauri/src/control_core.rs   | 3 ++-
 .../src-tauri/src/formal_memory_lifecycle.rs                         | 5 +----
 .../productized-desktop-shell/src-tauri/src/formal_memory_store.rs   | 5 +----
 .../src-tauri/src/memory_candidate_store.rs                          | 5 +----
 .../src-tauri/src/memory_entity_relation_governance.rs               | 5 +----
 .../productized-desktop-shell/src-tauri/src/memory_lint_engine.rs    | 5 +----
 .../productized-desktop-shell/src-tauri/src/observation_store.rs     | 5 +----
 .../src-tauri/src/session_continuation_store.rs                      | 5 +----
 .../src-tauri/src/task_memory_packet_builder.rs                      | 5 +----
 prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs      | 1 +
 12 files changed, 13 insertions(+), 41 deletions(-)
```

注：`git diff --stat` 不显示未跟踪的新文件；新文件见 `git status --short`。

## 7. 待复核

独立复核线需要确认：

- 公共 helper 行为等于原同形规则。
- 同形本地 normalize 归一，剩余本地 normalize 均在 deferred 范围内。
- `control_core::normalize_symbol` 只保留语义名 wrapper，行为不变。
- 未改 store 业务 / JSON / schema / 状态机 / runner / SQLite 迁移。
- 验证记录可信。

## 8. 不接受为

本包不接受为 R-U 全部完成、U-Gate 完成、查重门实现、R3 Level B 执行、SQLite 真实切换、真实 Codex 执行、`.codex` 读写、backlog 解冻或所有 normalization 规则强制统一完成。
