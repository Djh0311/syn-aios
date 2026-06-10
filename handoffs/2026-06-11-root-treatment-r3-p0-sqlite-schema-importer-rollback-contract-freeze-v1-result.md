# Root Treatment R3-P0 SQLite Schema Importer Rollback Contract Freeze v1 Result

日期：2026-06-11

## STATUS

`DONE_WITH_CONCERNS`

本线完成 R3-P0 contract freeze，只写合同文档、evidence、handoff。未提交，未改产品源码，未建库建表，未迁移真实 JSON / sidecar。

## 读写文件

读取：

- `tasks/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`
- `AGENTS.md`、`codex-multi-agent-safe-collaboration.md`
- Root Treatment plan、R0 shape gate 文档、R2 code map
- R2 closing / R3 preflight review evidence / handoff
- R2-B10 supervisor evidence / handoff
- R3-P0 authority sync supervisor checkpoint
- `Cargo.toml`、`types.rs`、`workflow_state_store.rs`、workflow helpers、memory / observation / governance / runtime / continuation / product command store 文件、shape gate 脚本

写入：

- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`

未写可选 R3-A1 任务包草案；合同文档已包含下一任务建议，当前少写任务入口文件更稳。

## Schema v0 摘要

合同冻结了五组表域：

- metadata / source：import batch、source hash、export batch、rollback point。
- workflow：projects、workflows、nodes、edges、work_items、artifacts、reviews、audit_events、bindings、dispatches。
- memory / observation：formal memory、candidate、observation、capture、lint、entity relation、mature pattern、blackboard。
- workflow governance：proposal、authorization、scope、C5/C6 reviews / summaries。
- runtime / continuation / product command：product command、attempt、continuation、runtime log、readback。

敏感字段策略：prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body 不进 DB；只允许 summary/ref/hash/redaction refs 和当前 workbench 已有的脱敏摘要。

## Importer / Idempotency 摘要

- Required input：`workflow-state.v0.json`。
- Optional inputs：当前 shape gate 允许的 14 种 sidecar。
- 幂等键优先使用 existing natural ids；无稳定 id 时用 canonical record hash fallback。
- same source hash + same natural key：skip。
- same natural key + different record hash：conflict，不覆盖。
- missing optional sidecar：warning。
- missing primary / corrupt primary：batch reject。
- unknown sidecar：source reject，需要 supervisor decision。
- importer 不覆盖源 JSON / sidecar。

## Transaction / Rollback 摘要

单事务边界已冻结：

- candidate -> formal memory + memory audit。
- observation -> candidate。
- memory capture -> observation -> candidate。
- proposal confirmation -> authorization creation。
- global boundary review -> authorization activation。
- process fact decision -> observation。
- product command Phase A/B trace -> continuation -> runtime log。

Rollback / export：

- DB -> JSON export 必须重建 workflow state 和 canonical sidecars。
- `runtime-logs.v1.json` 是 canonical；`runtime-log.v1.json` 仅 legacy alias / ref label。
- DB 失败、import batch reject、export hash mismatch、outbox projection failure 或 DB integrity corrupt 时可回旧 JSON read path。

## Fixture / Verification 摘要

合同冻结最小 fixture：empty workflow、project/workflow/work item/artifact/review/audit、formal/candidate/observation adoption、capture chain、proposal/authorization、process fact observation、product command continuation runtime、runtime-log alias、corrupt/missing/duplicate/revision/unknown sidecar/sensitive field。

R3-A1 应先做 schema + dry-run importer + fixture，不切读写路径。

## R3-A1 建议

建议任务包：

`2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`

建议范围：

- 只做 schema file / temp DB initializer。
- 只做 dry-run importer。
- 只写 fixture。
- 不创建生产 DB，不迁移真实数据，不双写，不读切 DB，不访问 `.codex`。

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

已运行并通过 / 完成：

- `git status --short`：初始无输出。
- `git rev-parse HEAD`：`0287d995785df467baf3677e2b03f30165eb7b85`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- sidecar inventory `rg`：完成，覆盖 workflow-state、formal-memories、memory-candidates、observations、runtime-log(s)、plan-authorizations、project-proposals、real-execution-product-commands、session-continuations。
- store guard `rg`：完成，覆盖 `rusqlite|sqlite|StoreLock|revision|corrupt|backup|rename|\.v1\.json`。
- `git log --oneline -8`：确认当前 R3-P0 任务包 commit 位于 HEAD。

最终收尾检查已运行：

- `git diff --check`
- `git status --short`

结果：

- `git diff --check`：通过，无输出。
- `git status --short`：仅包含三份预期 untracked 文件：
  - `?? docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
  - `?? evidence/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1.md`
  - `?? handoffs/2026-06-11-root-treatment-r3-p0-sqlite-schema-importer-rollback-contract-freeze-v1-result.md`

可选未运行：

- `cargo test --lib workflow_state`
- `cargo test --lib`
- `cargo fmt -- --check`

原因：本轮只写 docs / evidence / handoff，未改产品源码；cargo/fmt 非任务包必跑项，且可能产生 build artifacts。本线不把未运行的 cargo/fmt 冒充为 fresh verify。

## P0 / P1 / P2

- P0：无。
- P1：R3-A1 不得切 DB 读写路径、迁移真实数据或启动双写。
- P1：R3-A1 必须验证 forbidden-field fixture，避免 prompt body / secret / full transcript 被导入。
- P1：Product Command / continuation / runtime log 进入双写前必须先有 crash injection transaction tests。
- P2：`runtime-log.v1.json` / `runtime-logs.v1.json` alias 命名债仍存在。
- P2：sidecar backup retention 策略不统一，R3 初期应先保留现状。
- P2：Product Command sidecar 写入保护弱于多数 sidecar store，R3 transaction 需要优先覆盖。
- P2：R2 inline tests 巨石仍未迁移；R3-A1 不应夹带测试巨石拆分。

## 边界确认

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
