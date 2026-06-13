# Root Treatment / R-U4 Rust Normalization Util Dedup v1 Result

日期：2026-06-14

状态：完成，独立复核 `STATUS: CLEAR_WITH_P2`；P2 已补正，不阻断。

Planning baseline：`0a27b91`

Task package commit：`b5964b6 docs: add r-u4 normalization util dedup package`

复核线：Hilbert (`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`)

## 1. 完成内容

本包只做 Rust normalization helper 去重：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/normalization.rs`。
- 新增公共 helper `normalize_slash_lowercase(value)`，规则保持 `value.trim().replace('\\', "/").to_lowercase()`。
- `utils/mod.rs` 注册 `normalization` 模块。
- 10 个同形本地 `fn normalize(value: &str)` 改为公共 helper alias。
- `control_core.rs::normalize_symbol` 保留函数名，仅 wrapper 到公共 helper，调用语义不变。
- 增加公共 helper 行为单测。

## 2. Deferred

本包没有强合规则不同或业务特化 normalization：

- `memory_capture_bus.rs::normalize`：ASCII lowercase，不做 slash normalize。
- `mature_pattern_governance.rs::normalize`：mature pattern key 特化规则。
- `c4_c6_workflow_governance_entrypoints.rs::normalize_c4_symbol`：`-` 到 `_` 的 C4 symbol 特化。
- `workflow_execution_entrypoints.rs::normalize_director_review_decision`：业务枚举校验。
- `codex_transcript.rs` path normalization、`control_core.rs::normalized_absolute_path`、敏感路径检测局部 lowercase：路径 / 安全语义，保留原地。
- `alias_key`、tokenize、candidate key、pattern key 等特化逻辑：只共享底层同形 normalize，不抽特化规则。

## 3. 验证

主管线通过：

- `cargo fmt -- --check`
- `cargo test --lib memory_candidate_store`：1 passed
- `cargo test --lib formal_memory_store`：6 passed
- `cargo test --lib session_continuation_store`：16 passed / 4 ignored
- `cargo test --lib codex_local_runner`：12 passed
- `cargo test --lib control_core`：2 passed
- `cargo test --lib`：482 passed / 16 ignored
- `node scripts/harness/workbench-shape-gate.js --mode check`：`Status: pass`，`Errors: 0`，`Warnings: 0`
- `git diff --check`

复核线 Hilbert 通过：

- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 4. 复核结论

Hilbert 结论：`STATUS: CLEAR_WITH_P2`

P0：无。

P1：无。

P2：evidence 的 shape gate 摘要里 `session_continuation_store.rs` 行数记录不够精确；Hilbert 复跑显示 `5218/5237 (decreased)`。主管线已补正 evidence，不阻断。

## 5. 边界确认

本包改动精确范围：

- 改了 `utils/normalization.rs`、`utils/mod.rs` 和 11 个使用同形 normalize 的 Rust 文件 import / helper 定义。
- 没有改 JSON / sidecar / workflow state schema。
- 没有改状态机迁移规则、权限语义、runner 调用参数、真实执行入口或 SQLite migration / read-cut / stop-write 决策。
- 没有启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 没有执行真实 `codex exec` / `codex exec resume`。
- 没有读写 `/Users/yoyi/.codex`。
- 没有解冻 backlog。

## 6. 不接受为

本包不接受为 R-U 全部完成、U-Gate 完成、查重门实现、R3 Level B 执行、SQLite 真实切换、真实 Codex 执行、`.codex` 读写、backlog 解冻或所有 normalization 规则强制统一完成。

## 7. 下一步

按用户夜间目标，U4 收口后继续 U-Gate 草稿：只写 2-3 种查重门形态 + 推荐，不实现、不接入 harness / CI。
