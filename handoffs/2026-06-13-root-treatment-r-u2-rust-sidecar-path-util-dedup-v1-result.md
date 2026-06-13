# Root Treatment / R-U2 Rust Sidecar Path Util Dedup Handoff v1

日期：2026-06-13

状态：已完成。

## 1. 主管线结论

R-U2 已完成实现侧闭环：12 个重复 `sidecar_path` helper 的 parent / join 逻辑已迁入 `src-tauri/src/utils/store_paths.rs`，12 个 store 文件保留同名 wrapper、原 `SIDECAR_NAME` 常量和原 store label，外部调用入口不变。

本包只改 wrapper 内部实现，不改 store 读写业务语义、JSON / sidecar schema、状态机或 SQLite schema / migration。

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

证据：

- `evidence/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1.md`

## 3. 形状结果

- `utils/store_paths.rs` 新增公共 helper。
- 12 个 store wrapper 保留。
- `lib.rs` 保持 5567 行。
- `session_continuation_store.rs` 从 5237 行降到 5221 行。
- shape gate 通过，0 errors / 0 warnings。

## 4. 验证

已通过：

- `cargo fmt -- --check`
- `cargo test --lib memory_lint`
- `cargo test --lib memory_entity_relation`
- `cargo test --lib session_continuation`
- `cargo test --lib plan_authorization`
- `cargo test --lib memory_candidate`
- `cargo test --lib observation`
- `cargo test --lib formal_memory`
- `cargo test --lib project_consultation`
- `cargo test --lib memory_capture`
- `cargo test --lib mature_pattern`
- `cargo test --lib blackboard`
- `cargo test --lib runtime_log`
- `cargo test --lib`，`476 passed / 16 ignored`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 5. 扫描结论

- 12 个 `SIDECAR_NAME` 原值仍在原文件。
- `rg -n "fn sidecar_path\\("` 显示 12 个 store wrapper + 1 个公共 helper；这是任务包设计内结果。
- `workbench_sqlite_schema.rs`、`workflow_state_store.rs`、`workflow_state_json_helpers.rs` 无 diff。
- 敏感关键词命中均为既有 fixture / preview 文案，非本包新增真实执行路径。

## 6. 独立复核结果

独立复核 agent `Poincare`（`019ec19f-6366-7cf0-9a17-ecb07722429e`）回交 `STATUS: CLEAR_WITH_P2`，P0/P1 无；唯一 P2 为 evidence 的 `git status --short` 记录漏写 task/evidence/handoff 文件。该 P2 已补齐，不影响代码行为或提交放行；记录见 `evidence/2026-06-13-root-treatment-r-u2-rust-sidecar-path-util-dedup-v1-review-poincare-v1.md`。

复核确认：

- 12 个 store wrapper 是否仍保留且外部调用入口不变。
- 公共 helper 是否只做 parent error + join sidecar_name。
- 12 个 `SIDECAR_NAME` 原值是否逐店零变化。
- 12 个 store label 是否逐店保持原文。
- 是否没有改 `load_store` / `empty_store` / `validate_store` / write / lock / backup / atomic replace 业务语义。
- 是否没有改 JSON / sidecar schema、workflow state schema、状态机或 SQLite schema / migration。
- 验证记录是否可信。

## 7. 停止线

复核已 `CLEAR_WITH_P2` 且 P2 已修正，主管线可提交 implementation commit，并停在 R-U2 复核点。

不得顺手进入 U3 / U4 / U5 / U-Gate、R3 Level B 或 backlog 解冻。
