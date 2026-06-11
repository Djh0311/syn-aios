# Root Treatment / R3-A7 Production Preflight Scanner And Report v1

日期：2026-06-11

状态：已完成。本文是 Root Treatment / Stage R 的 R3-A7 任务包，用于在 R3-A6 production cutover / rollback operator contract freeze 基础上，实现 production preflight scanner / report 的只读工具和 fixture 验证。R3-A7 默认只实现 scanner 模块和测试，不默认扫描真实生产 root；如要扫描真实工作台 state root，必须由主管线在本任务内明确 allowed root、report path、读取字段和不读取项。

完成记录：

- evidence：`evidence/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1-result.md`
- implementation commit：`7949253c91c8e688dc48e03c47a952f00fcd6fda`

## 0. 全局主管理解

已知事实：

- R3-A6 已冻结 production cutover contract、rollback operator contract、allowed roots / denied paths、backup / recovery 和 dry-run / apply 分界。
- R3-A7 的推荐目标是只读 production preflight scanner / report：读取工作台自有 JSON / sidecar metadata、hash、schema、revision 和 backup readiness。
- R3-A7 不创建 production DB，不写 production root，不迁移 JSON / sidecar，不切读写路径，不读写 `/Users/yoyi/.codex`。

本任务核心判断：

```text
R3-A7 先实现 scanner 能力并用 fixture/temp root 验证；真实 production root 扫描必须是显式 allowed root 的单独步骤，不得默认发生。
```

## 1. Execution Mode

Execution Mode：Single implementation line, supervisor integrated。

Multi-Agent Policy：

- 本任务可以由主管线直接实现，避免过细分线导致文档维护成本。
- 若执行真实 production root preflight，必须先写 execution record 并由主管线自审 allowed root；默认不执行。

Fallback If Scope Expands：

- 若需要写 production root、创建 DB、迁移 JSON / sidecar、读取 secret / transcript 或接 UI/Tauri command，停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A7 必须完成：

- 新增只读 scanner module，建议 `workbench_sqlite_preflight.rs`。
- `lib.rs` 只允许新增 module declaration；不得新增 Tauri command、startup hook、UI 或产品路径调用。
- Scanner 显式接收：
  - `source_root`
  - optional `report_path`
  - expected allowed sidecar list
  - denied path substrings / denied file names
- Scanner 只读取 file metadata、raw file hash、JSON schema_version / revision / top-level shape / record counts；不得输出 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。
- Scanner report 必须包含：
  - source root ref / hash
  - file count / accepted / missing optional / rejected / warnings
  - per-file path ref、path hash、file hash、size、schema_version、revision、top-level keys、record count estimate、redaction_status
  - backup readiness：是否存在 `backups/`、workflow state backup count、latest backup hash / timestamp
  - sidecar readiness：canonical sidecar present / optional missing / unknown sidecar rejected
  - denied path / sensitive name hits as blocked
  - `production_db_created=false`
  - `production_root_written=false`
  - `read_cut_enabled=false`
  - `stop_write_json=false`
  - `codex_home_touched=false`
- Scanner must reject:
  - `/Users/yoyi/.codex`
  - paths containing `.env`, `token`, `secret`, `credential`, `keychain`, `oauth`
  - unknown JSON files not in allowed sidecar / workflow-state / backup list
  - non-JSON files in source root except lock/temp/report files explicitly ignored
- Scanner tests must use temp / fixture roots only.
- Evidence / handoff must record no real production root scan unless explicitly executed.

## 4. 允许读取

允许读取：

- `product-line` 内源码、任务包、docs、evidence、handoff、fixtures、git metadata。
- Temp / fixture roots created by tests.
- 如果主管线执行真实 preflight：只能读取显式 allowed workbench state root 的 metadata/hash/schema/revision/top-level counts，不读取正文输出，不读 `.codex`。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目源码内容。

## 5. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅新增 module declaration。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a7/**`
- `evidence/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1-result.md`

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 6. 禁止事项

R3-A7 禁止：

- 不创建 production DB。
- 不写 production root。
- 不迁移真实 JSON / sidecar。
- 不修改任何真实 JSON / sidecar。
- 不切任何产品读写路径到 DB。
- 不在 app startup / Tauri command / UI 中接入 scanner。
- 不停止 JSON / sidecar 写入。
- 不读取 `/Users/yoyi/.codex`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不执行真实 Codex。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。

## 7. 形状影响

- 任务类型：治理任务包 / production preflight scanner。
- 新增代码落点：`workbench_sqlite_preflight.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1 行 module declaration。
- 新文件上限：Rust 新文件必须低于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`330ec20937209633cd823bdf4bd69e04f95e01f6`。
- 本任务 implementation commit：`7949253c91c8e688dc48e03c47a952f00fcd6fda`。
- 本任务 checkpoint / review-fix commit：以主管线最终回交为准。

## 8. 验收标准

R3-A7 可接受为：

- production preflight scanner module 已实现。
- scanner 只读 metadata/hash/schema/revision/top-level counts。
- fixture tests 覆盖 valid root、missing optional sidecars、unknown sidecar rejected、denied sensitive name rejected、backup readiness、report flags false。
- `lib.rs` 只新增 module declaration。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过。
- evidence / handoff 记录是否执行真实 production root scan；默认应记录未执行。

R3-A7 不接受为：

- production DB 创建。
- production apply。
- production read-cut。
- JSON / sidecar stop-write。
- rollback production workflow。
- R3 完成。
- 多 agent 并行真实执行解锁。

## 9. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_preflight
cargo test --lib sqlite_schema
cargo test --lib sqlite_observation
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

如执行真实 production root preflight，还必须记录 exact command / allowed root / report path / source root hash，并确认 report 不含 forbidden body。

## 10. Evidence / Handoff 结构

Evidence 必须包含：

1. STATUS。
2. READ / WRITE SCOPE。
3. SCANNER SUMMARY。
4. FIXTURE COVERAGE。
5. REAL PRODUCTION PREFLIGHT STATUS。
6. CHECKS RUN。
7. P0 / P1 / P2。
8. BOUNDARY CONFIRMATION。
9. DO NOT CLAIM。

## 11. Do Not Claim

完成 R3-A7 后仍不得声明：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
