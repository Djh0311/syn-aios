# Root Treatment / R3-A10 Limited Read-Cut Planning And Feature Flag Fallback v1

日期：2026-06-11

状态：Level A 已完成；Level B 未执行。本文是 Root Treatment / Stage R 的 R3-A10 任务包，用于在 R3-A9 Level A production DB initializer + apply with backup manifest / no read-cut 之后，冻结并实现“有限读切”合同的 Level A fixture / temp rehearsal。R3-A10 默认不执行真实 production read-cut，不停写 JSON / sidecar，不让真实 app startup / Tauri command / UI 读取 SQLite，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

完成记录：

- evidence：`evidence/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1-result.md`
- planning baseline commit：`9196b04ad861b01e56132fba9b2f5fc170661ccc`
- task package freeze commit：`b8ecc19885f6fd2cef79c907db95fab4a76053d5`
- implementation commit：`b18424c38bf0f36f8c9b8ee783a0010598ca9683`

## 0. 全局主管理解

已知事实：

- R3-A4 已完成 fixture-only read-cut DB / JSON fallback / rollback recovery dry-run，代码落点为 `workbench_sqlite_read_cut.rs`。
- R3-A9 Level A 已完成 fixture / temp production DB initializer + apply with backup manifest / export verification / rollback boundary，implementation commit 为 `52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`。
- R3-A9 Level B 未执行，真实 workbench state root 未读取，真实 workbench-owned production DB 未创建。
- R3-A10 不能假设生产 DB 已存在，也不能把 A9 Level A 冒充为真实 production apply。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

核心判断：

```text
R3-A10 先把“有限读切”做成显式 feature flag + fallback + report 合同；Level A 只在 fixture / temp DB 中验证一个低风险读模型的 DB-read / JSON-fallback 行为，真实产品读路径保持不变。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- 可派发给既有 Stage R 开发线，思考程度 high / xhigh。
- 开发线不得提交；主管线负责 fresh verify、复核、入口同步和 commit。
- 复核线必须只读复核，不改文件。

Level split：

- Level A：fixture / temp limited read-cut contract implementation。必须先完成；只允许 repo fixture 或 temp DB / temp projection root，不读取真实 workbench state root，不创建真实 production DB，不切真实产品读路径。
- Level B：optional real workbench-owned production DB limited read-cut rehearsal。只有 Level A 通过、任务包 evidence 追加 Level B execution record、真实 production DB 已由单独任务包创建且主管自审通过后才可执行。Level B 仍不得停写 JSON / sidecar，且必须可一键回退到 JSON fallback。

Fallback If Scope Expands：

- 如果实现需要 app startup hook、Tauri command、UI 接入、真实 product read path、stop-write JSON、真实 Codex 执行、`.codex`、secret / full transcript、provider credential，立即停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- A4 / A9 evidence 和 handoff。

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A10 Level A 必须完成：

- 新增或扩展 read-cut contract helper，建议继续落在 `workbench_sqlite_read_cut.rs`，避免新增平行读切模块。
- Helper 必须显式接收：
  - `read_model_name`
  - `feature_flag_enabled`
  - `db_path`
  - `json_fallback_root`
  - `projection_root`
  - `rollback_manifest_path`
  - `read_cut_report_path`
  - `expected_db_hash`
  - `expected_fallback_hash`
  - `allowed_read_models`
  - `denied_path_markers`
  - optional failure injection point
- `read_model_name` Level A 默认只能是一个低风险读模型，建议 `workflow_state_summary` 或 `runtime_log_summary`；不得一次覆盖所有 read model。
- `feature_flag_enabled=false` 时必须直接选择 JSON fallback，状态为 `feature_flag_disabled_fallback`，不得打开 DB。
- `feature_flag_enabled=true` 时允许先读 DB projection；DB unavailable、schema mismatch、integrity failure、hash mismatch、rollback manifest missing / incomplete 必须降级或阻断，不能冒充 DB success。
- 成功 DB read 必须记录 `read_source=db_limited`，fallback 必须记录 `read_source=json_fallback`，阻断必须返回错误且不写 completed report。
- JSON fallback 必须来自已验证 projection / fallback root，并校验 `expected_fallback_hash`；不得把未校验 source JSON 当成 fallback success。
- Report 必须记录：
  - schema version
  - mode：`limited_read_cut`
  - level：`level_a_fixture`
  - status：`completed` / `fallback_degraded` / `blocked` / `failed_classified`
  - read model name
  - feature flag state
  - read source
  - fallback decision
  - degraded flag
  - DB path hash
  - fallback root hash
  - projection hash
  - rollback manifest hash
  - expected / actual DB hash
  - expected / actual fallback hash
  - row / record counts
  - recovery dry-run
  - safety flags
  - failure point if any
- Safety flags 必须为：
  - `limited_read_cut_enabled=true` only when feature flag true and DB read succeeds for the selected model.
  - `product_global_read_path_changed=false`
  - `app_startup_reads_db=false`
  - `tauri_command_reads_db=false`
  - `ui_reads_db=false`
  - `stop_write_json=false`
  - `source_json_written=false`
  - `production_restore_performed=false`
  - `codex_home_touched=false`
- Recovery dry-run 必须说明：would-disable limited read-cut、would-use JSON fallback、would-preserve DB for audit、would-require supervisor decision、production_restore_performed=false。
- Failure injection 必须覆盖：
  - feature flag disabled fallback。
  - DB unavailable fallback。
  - DB schema mismatch fallback。
  - DB integrity failure fallback。
  - DB hash mismatch blocked。
  - fallback hash mismatch blocked。
  - projection hash mismatch blocked。
  - missing rollback manifest blocked。
  - incomplete rollback manifest blocked。
  - after DB read before report commit leaves no completed report。
  - after fallback selected before report commit leaves no completed report。
- Idempotent rerun：同一 fixture / temp DB / projection / fallback / manifest 的 report hash 稳定，或显式 already completed classification。
- Report / projection 不得包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body。

R3-A10 Level B 如执行，必须另写 execution record，并且至少满足：

- 明确真实 production DB path、source JSON fallback root、feature flag scope、selected read model、rollback command / rollback evidence。
- 只允许一个低风险 read model；不得全局切所有读取。
- 必须记录 before / after JSON / sidecar hashes，证明未停写、未改源文件。
- 必须记录 DB file hash、schema version、selected read query hash、fallback projection hash、report hash。
- 必须提供立即回退到 JSON fallback 的操作记录或 dry-run 证据。

## 4. 允许读取

Level A 允许读取：

- `product-line` 内源码、任务包、docs、evidence、handoff、fixtures、git metadata。
- R3 fixtures 和测试创建的 temp roots。

Level B 可选允许读取：

- 任务包 evidence 中明确记录的 workbench-owned production DB path 和 JSON fallback root。
- 只读 metadata/hash/schema/revision/top-level counts 和所选 read model 必需字段。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout、prompt body。
- 用户真实项目源码内容。
- 任意未列入任务包 allowed roots 的路径。

## 5. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10/**`
- 可写测试用 temp DB / temp projection / temp fallback / temp report。
- `evidence/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1-result.md`
- 本任务包状态。

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 6. 禁止事项

R3-A10 禁止：

- 不切全局产品读路径到 DB。
- 不让 app startup / Tauri command / UI 读取 SQLite。
- 不停写 JSON / sidecar。
- 不修改任何 source JSON / sidecar。
- 不创建真实 production DB。
- 不把 A9 Level A temp DB 当成真实 production DB。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不新增 Tauri command。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不改 workflow state 顶层 schema。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不把 limited read-cut contract 冒充为 production read-cut、JSON / sidecar stop-write、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。

## 7. 形状影响

- 任务类型：治理任务包 / limited read-cut contract。
- 代码落点：优先扩展 `workbench_sqlite_read_cut.rs`。
- 触碰棘轮文件：不得新增 `lib.rs` module declaration，除非主管线确认必须新增模块。
- 新文件上限：
  - Rust 新文件如不可避免，必须低于 3,000 行。
  - fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`9196b04ad861b01e56132fba9b2f5fc170661ccc`
- 本任务完成 commit：待主管线回收后记录。

## 8. Fixture 矩阵

必须至少新增并测试：

- `limited-read-cut-feature-flag-disabled-fallback`：flag off，直接 JSON fallback，不打开 DB。
- `limited-read-cut-db-authoritative-success`：flag on，DB read success，read source 为 DB limited。
- `limited-read-cut-db-unavailable-fallback`：DB missing，verified JSON fallback，degraded。
- `limited-read-cut-schema-mismatch-fallback`：schema mismatch，verified JSON fallback，degraded。
- `limited-read-cut-db-hash-mismatch-blocked`：DB hash mismatch，blocked，无 completed report。
- `limited-read-cut-fallback-hash-mismatch-blocked`：fallback hash mismatch，blocked，无 completed report。
- `limited-read-cut-projection-hash-mismatch-blocked`：projection hash mismatch，blocked，无 completed report。
- `limited-read-cut-missing-manifest-blocked`：rollback manifest missing，blocked。
- `limited-read-cut-incomplete-manifest-blocked`：rollback manifest incomplete，blocked。
- `limited-read-cut-sensitive-redaction`：report / projection 不含 forbidden sensitive body classes。
- `limited-read-cut-idempotent-rerun`：同输入重跑结果稳定。

## 9. 验收标准

R3-A10 Level A 可接受为：

- limited read-cut contract helper 已实现或现有 read-cut helper 已扩展。
- 一个低风险 read model 在 feature flag 下完成 DB-read / JSON-fallback / blocked 三类结果验证。
- feature flag off 不打开 DB，直接 fallback。
- DB unavailable / schema mismatch / integrity failure 能 fallback 且状态 degraded。
- DB hash mismatch / fallback hash mismatch / projection mismatch / manifest missing / manifest incomplete 均阻断或失败分类，不写 completed report。
- Report 安全 flags 明确保持全局产品读路径、startup、Tauri command、UI、stop-write 和 source JSON 写入为 false。
- 未创建真实 production DB。
- 未读取真实 workbench state root。
- 未新增 Tauri command / UI / startup hook。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 明确 Level B 是否执行；默认应记录未执行真实 production limited read-cut。

R3-A10 不接受为：

- production read-cut 完成。
- app 真实读取 SQLite。
- JSON / sidecar stop-write。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 恢复。

## 10. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_read_cut
cargo test --lib sqlite_production
cargo test --lib sqlite_export
cargo test --lib sqlite_apply
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
```

建议扫描：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10
rg -n '(product_global_read_path_changed|app_startup_reads_db|tauri_command_reads_db|ui_reads_db|stop_write_json|source_json_written|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10
```

如果 fixture 文件尚未创建，扫描命令可改为实际存在的 R3-A10 文件清单，但 evidence 必须说明。

## 11. Evidence / Handoff 要求

Evidence 必须记录：

- planning baseline commit。
- implementation commit 或未提交状态。
- changed files。
- feature flag behavior。
- selected read model。
- DB success / fallback / blocked 矩阵。
- safety flags。
- verification commands。
- scan results。
- boundary confirmation。
- do-not-claim list。

Handoff 必须记录：

- 本轮完成范围。
- Level A / Level B 是否执行。
- 如何验证 fallback。
- 哪些路径没有接入真实产品读。
- 下一步建议：R3-A11 observation 或 A10 Level B 是否需要另行任务包。

## 12. 复核线检查清单

复核线必须检查：

- changed files 是否只在 A10 允许范围。
- 是否没有新增 Tauri command、startup hook、UI、真实 product read path。
- feature flag off 是否不打开 DB。
- DB success / fallback / blocked 状态是否可区分。
- safety flags 是否没有 true 越界项。
- 是否没有读取真实 workbench state root、没有创建真实 production DB。
- 是否没有 `.codex`、secret、token、full transcript、rollout 越界。
- evidence / handoff 是否没有把 A10 Level A 夸大为 production read-cut 或 R3 完成。

## 13. 完成后仍不得声明

完成 R3-A10 Level A 后仍不得声明：

- production read-cut 完成。
- app 真实 SQLite 读路径已启用。
- JSON / sidecar stop-write 完成。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
