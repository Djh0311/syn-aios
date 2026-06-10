# Root Treatment R2 Closing / R3 Preflight Review v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

本线完成 R2-B10 后的只读架构/治理复核，只写本 handoff 和对应 evidence。未改产品源码，未迁移 SQLite，未提交。

## 读写文件

读取：

- 当前权威入口：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`
- 协作规则：`AGENTS.md`、`codex-multi-agent-safe-collaboration.md`
- R 计划和 R0/R2 文档：`docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`、`docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`、`docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- R2-B10 回收材料：`evidence/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1.md`、`handoffs/2026-06-11-root-treatment-r2-b10-supervisor-checkpoint-v1-result.md`
- 相关 Rust 源码和 store 文件，重点是 `lib.rs`、R2 helpers、`workflow_state_store.rs`、各 `*store.rs`、`runtime_log_store.rs`、`session_continuation_store.rs`、`real_execution_command.rs`

写入：

- `evidence/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1-result.md`

## 矩阵摘要

`lib.rs` 剩余结构：

- `lib.rs` 当前 13,949 行，Tauri commands 96 total / `lib.rs` 内 0。
- 仍建议后续治理的主区块：
  - `119-243` index / transcript loader，可抽到 transcript catalog 边界。
  - `254-866` task package render / finder helper，建议 R2 后段优先。
  - `872-1210` shared workflow utility，需要先分类，避免变成新的杂物间。
  - `1215-1298` dispatch id / safe probe / path / time / atomic helper，可小批次抽出。
  - `1299-1695` workbench snapshot assembly，可作为 R4 按页读模型前置整理。
  - `1703-13949` inline tests 巨石，必须单独迁移。

inline tests：

- 主测试模块：`lib.rs:1703-13949`。
- 静态统计：213 个 `#[test]`。
- 优先迁移建议：
  - transcript/readback tests。
  - task package render/readiness/file generation tests。
  - C4-C6 tests 到 R2-B10 helper local tests。
  - workflow state lifecycle / workflow run dispatch store-local tests。
- 不建议现在迁移：
  - 共享 fixture / stub runner 底座未拆前，不做整段批量搬迁。
  - candidate adoption、observation -> candidate、real execution stub runner 等跨 store / 跨执行语义测试，等 R3 transaction / test support 设计后再迁。

R3 SQLite preflight：

- shape gate 当前检测 sidecar JSON kinds：14 detected / 0 unknown。
- `workflow_state_store.rs` 已有 StoreLock、corrupt guard、temp + rename、backup retention。
- 多数 sidecar store 有 lock/corrupt/revision/atomic write，但策略不统一；`real-execution-product-commands.v1.json` 相关写入和 runtime/continuation 跨文件写入尤其需要 R3 transaction 设计。
- 当前 `rusqlite` 已在 `Cargo.toml` 存在，但主要用于只读读取 Codex 原生 sqlite，不等于工作台 workflow/sidecar 统一存储已存在。
- R3 必须先冻结 schema、importer 输入、idempotency key、rollback/export 和 crash injection fixture，不能直接把建表当 R3 完成。

## 推荐下一任务包

建议下一任务包：

`2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`

建议执行模式：

- 单线治理任务。
- 只做 schema/importer/rollback contract freeze，不改产品写路径。
- R3 完成前不解锁多 agent 并行真实执行。

建议读写范围：

- 读：`src-tauri/src/*store.rs`、`workflow_state_*`、`real_execution_command.rs`、`runtime_log_store.rs`、`session_continuation_store.rs`、`project_workflow_automation.rs`、R0/R1/R2 evidence/handoff、shape gate。
- 写：R3 schema/importer contract 文档、对应 evidence/handoff、可选下一步 R3-A1 任务包草案。
- 不写：产品源码、SQLite schema 实现、migration、workflow state schema、sidecar、UI、入口文档。

建议验证：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- `rg` sidecar/store scan
- `git diff --check`
- `git status --short`
- 若新增 fixture，再加 fixture lint / importer dry-run 设计检查

风险和授权：

- R3-P0 文档/contract freeze 为中风险，不需要新的用户授权。
- 后续实际 SQLite schema/importer、双写、停写 JSON、真实数据迁移或 sidecar 清理是高风险，必须单独任务包和主管复核。

## 验证结果

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`
  - Status `pass`
  - 0 errors / 0 warnings
  - `lib.rs` 13,949 lines
  - Tauri commands 96 total / 0 in `lib.rs`
  - Sidecar JSON kinds 14 detected / 0 unknown
- `git status --short`
  - 初始无输出
- 建议只读命令已运行：
  - `rg -n "include!|#\\[cfg\\(test\\)\\]|fn test_|#\\[test\\]" prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - `wc -l prototypes/productized-desktop-shell/src-tauri/src/lib.rs prototypes/productized-desktop-shell/src-tauri/src/*.rs`
  - `git log --oneline -12`

未运行可选：

- 未运行 `cargo test --lib workflow_state`
- 未运行 `cargo test --lib`
- 未运行 `cargo fmt -- --check`

原因：本线是只读复核任务，只允许写 evidence/handoff；cargo/fmt 不是本轮必须命令，且本轮未改产品源码。R2-B10 supervisor checkpoint 已 fresh-run 并记录 cargo/fmt 通过，但本线不把它冒充为本轮 fresh verify。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：本轮为静态只读复核，未 fresh-run cargo/fmt。
- P2：行号和测试统计基于 HEAD `489b18f36e217bf10f761118bf303a8b92c057ed`，后续任务开始前需重算。
- P2：inline tests 仍在 `lib.rs`，R2 后段需专项迁移。
- P2：R3 schema/importer/rollback 尚未冻结，本轮仅推荐下一任务包。

## 边界确认

- 未改产品源码。
- 未迁移 SQLite。
- 未建 schema。
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
