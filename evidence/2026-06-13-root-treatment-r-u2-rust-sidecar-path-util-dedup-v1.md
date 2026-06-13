# Root Treatment / R-U2 Rust Sidecar Path Util Dedup Evidence v1

日期：2026-06-13

状态：已完成。

## 1. 实现摘要

本包只做 Rust 后端 sidecar path helper 去重：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs`。
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs` 增加 `pub(crate) mod store_paths;`。
- 12 个 store 文件保留原 `pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String>` wrapper，内部改为调用公共 helper。
- 12 个 store 文件的 `SIDECAR_NAME` 常量均保留在原文件，原值不变。
- 12 个 store label 逐店传入公共 helper，保留原父目录缺失报错文案。

本包没有把 sidecar 文件名集中搬到 utils；这是刻意边界，用来避免文件名误接导致数据写错位置。

## 2. 文件变化

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`

精确边界：上述 12 个 store 文件只改了 `use crate::utils::store_paths;` 与 `sidecar_path` wrapper 函数体；未修改 `load_store` / `empty_store` / `validate_store` / write / lock / backup / atomic replace 业务逻辑。

## 3. Sidecar 文件名核对

扫描命令：

```text
rg -n "const SIDECAR_NAME" prototypes/productized-desktop-shell/src-tauri/src/{memory_lint_store.rs,memory_entity_relation_store.rs,session_continuation_store.rs,plan_authorization_store.rs,memory_candidate_store.rs,observation_store.rs,formal_memory_store.rs,project_consultation_proposal_store.rs,memory_capture_bus.rs,mature_pattern_store.rs,blackboard_candidate_store.rs,runtime_log_store.rs}
```

原始输出：

```text
prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs:11:const SIDECAR_NAME: &str = "memory-entity-relations.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs:13:const SIDECAR_NAME: &str = "formal-memories.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs:20:const SIDECAR_NAME: &str = "plan-authorizations.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs:14:const SIDECAR_NAME: &str = "memory-lint.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs:8:const SIDECAR_NAME: &str = "memory-patterns.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs:14:const SIDECAR_NAME: &str = "memory-candidates.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs:21:const SIDECAR_NAME: &str = "project-proposals.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs:14:const SIDECAR_NAME: &str = "runtime-logs.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs:11:const SIDECAR_NAME: &str = "memory-capture-events.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs:16:const SIDECAR_NAME: &str = "blackboard-candidates.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs:14:const SIDECAR_NAME: &str = "observations.v1.json";
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:30:const SIDECAR_NAME: &str = "session-continuations.v1.json";
```

## 4. Helper 扫描

扫描命令：

```text
rg -n "fn sidecar_path\\(" prototypes/productized-desktop-shell/src-tauri/src
```

原始输出摘要：

```text
prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs:14:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:33:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs:14:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs:19:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs:23:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs:17:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs:16:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs:11:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs:17:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs:24:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs:17:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs:17:pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs:3:pub(crate) fn sidecar_path(
```

解释：12 个 store wrapper 按任务包要求保留，外部调用入口不变；公共重复逻辑集中到 `utils/store_paths.rs`。

## 5. 禁止路径核对

命令：

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs
```

原始输出：无输出。

敏感 / 真实执行关键词扫描命令：

```text
rg -n "Command::new\\(\"codex\"\\)|codex exec|exec resume|/Users/yoyi/.codex" prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs prototypes/productized-desktop-shell/src-tauri/src/{memory_lint_store.rs,memory_entity_relation_store.rs,session_continuation_store.rs,plan_authorization_store.rs,memory_candidate_store.rs,observation_store.rs,formal_memory_store.rs,project_consultation_proposal_store.rs,memory_capture_bus.rs,mature_pattern_store.rs,blackboard_candidate_store.rs,runtime_log_store.rs}
```

原始输出命中均为既有测试 fixture / command preview 文案，非本包新增执行路径：

```text
prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs:868:            command_preview: "codex exec resume sk-test-secret".to_string(),
prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs:893:            command_preview: "codex exec resume sk-test-secret".to_string(),
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:1948:            "H3.1 preview only: codex exec --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <stdin:workbench-managed-prompt>",
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:1954:            "Level A preview only: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} {}",
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:2846:        "H2 preflight only: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <session:{}>",
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:2855:        "Controlled real resume command: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <session:{}>",
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:3302:        assert!(!serialized_runtime.contains("codex exec resume"));
prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:4225:                confirmation_reason: "用户授权 H3-B 隔离 fixture、allowed write roots、Codex home 最小新会话副作用和一次真实 codex exec new_session probe。".to_string(),
```

## 6. 验证记录

在 `prototypes/productized-desktop-shell/src-tauri` 执行。

### cargo fmt

命令：

```text
cargo fmt -- --check
```

原始输出：无输出，exit code 0。

### 聚焦测试

原始尾部输出：

```text
cargo test --lib memory_lint
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 483 filtered out; finished in 0.20s

cargo test --lib memory_entity_relation
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 487 filtered out; finished in 0.08s

cargo test --lib session_continuation
test result: ok. 17 passed; 0 failed; 4 ignored; 0 measured; 471 filtered out; finished in 0.31s

cargo test --lib plan_authorization
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 484 filtered out; finished in 0.05s

cargo test --lib memory_candidate
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 483 filtered out; finished in 0.23s

cargo test --lib observation
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 452 filtered out; finished in 1.46s

cargo test --lib formal_memory
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 463 filtered out; finished in 0.58s

cargo test --lib project_consultation
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 487 filtered out; finished in 0.13s

cargo test --lib memory_capture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 485 filtered out; finished in 0.03s

cargo test --lib mature_pattern
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 487 filtered out; finished in 0.35s

cargo test --lib blackboard
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 489 filtered out; finished in 0.06s

cargo test --lib runtime_log
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 485 filtered out; finished in 0.13s
```

说明：以上 Rust 测试均保留既有 warning：

```text
warning: associated function `invalid_params` is never used
```

### cargo test --lib

命令：

```text
cargo test --lib
```

原始尾部输出：

```text
test result: ok. 476 passed; 0 failed; 16 ignored; 0 measured; 0 filtered out; finished in 7.12s
```

### shape gate

命令：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

原始关键输出：

```text
Status: pass
Errors: 0
Warnings: 0
Git HEAD: ef869902d0039e92d39d8e02f6ab5b53bc4317b0
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- session_continuation_store.rs: 5221/5237 (decreased)
```

### git diff --check

命令：

```text
git diff --check
```

原始输出：无输出，exit code 0。

## 7. 当前 git 实物

`git log --oneline -6` 原始输出：

```text
ef86990 docs: add r-u2 sidecar path util dedup package
6fca242 docs: checkpoint r-u1 hash util dedup
e6325e8 refactor: deduplicate rust hash helpers
5a295e0 docs: add r-u1 hash util dedup package
c8b3a1e docs: checkpoint r4 h3 completion
5b3bb80 refactor: split project workflow side panels
```

`git status --short` 原始输出：

```text
 M prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs
 M prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs
 M tasks/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1.md
?? evidence/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1.md
?? handoffs/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1-result.md
?? prototypes/productized-desktop-shell/src-tauri/src/utils/store_paths.rs
```

## 8. 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮具体修改了 12 个 store 文件的 `sidecar_path` wrapper 内部实现；没有修改这些 store 的 `load_store` / `empty_store` / `validate_store` / write / lock / backup / atomic replace 业务逻辑。没有改 `workbench_sqlite_schema.rs`、`workflow_state_store.rs`、`workflow_state_json_helpers.rs`，没有迁 SQLite。

## 9. 独立复核结果

独立复核 agent `Poincare`（`019ec19f-6366-7cf0-9a17-ecb07722429e`）回交 `STATUS: CLEAR_WITH_P2`，P0/P1 无；唯一 P2 为本 evidence 的 `git status --short` 记录漏写 task/evidence/handoff 文件。该 P2 已在本文件第 7 节补齐，记录见 `evidence/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1-review-poincare-v1.md`。

复核确认：

- 12 个 store wrapper 仍保留，外部调用入口不变。
- 公共 helper 只做 parent error + join sidecar_name。
- 12 个 `SIDECAR_NAME` 原值逐店零变化。
- 12 个 store label 保持原文，父目录缺失报错文案保持。
- 未改 store 读写业务语义、JSON / sidecar schema、状态机、SQLite schema / migration。
- 验证记录可信。
