# Root Treatment / R3-A11 Production Observation Export Verification Contract v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R3-A11 任务包，用于在 R3-A10 Level A limited read-cut contract / feature flag fallback 完成后，冻结并实现“production observation / export verification”合同的 Level A fixture / temp rehearsal。这里的 `production observation` 只表示生产切换前必须具备的合同语义和证据结构，不是授权真实生产观察动作。R3-A11 默认不执行真实 production observation，不读取真实 workbench state root，不创建真实 production DB，不切 app startup / Tauri command / UI / 产品全局读路径，不停写 JSON / sidecar，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

规划基线 commit：`eda6a4968839a9c470de03ab360f586a0a8060e1`

## 0. 全局主管理解

已知事实：

- R3-A5 已完成 fixture-only observation / export / rollback verification rehearsal，代码落点为 `workbench_sqlite_observation_period.rs`。
- R3-A9 Level A 已完成 fixture / temp production DB initializer + apply with backup manifest / export verification / rollback boundary，implementation commit 为 `52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`。
- R3-A10 Level A 已完成 `workflow_state_summary` 单一低风险 read model 的 fixture / temp limited read-cut contract、feature flag、verified JSON fallback、blocked matrix、recovery dry-run 和 A10 专用 projection path guard，implementation commit 为 `b18424c38bf0f36f8c9b8ee783a0010598ca9683`。
- R3-A9 / R3-A10 Level B 均未执行；真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，真实 production limited read-cut 未执行。
- R3-A11 不能假设真实 production DB 已存在，也不能把 R3-A10 Level A 冒充为真实 product read path 已切 DB。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

核心判断：

```text
R3-A11 先把“production observation / export verification”做成显式合同和 Level A rehearsal：`production` 在 Level A 仅指生产切换前合同语义，不指真实生产路径执行；只在 fixture / temp DB 上验证观察期、导出校验、fallback / rollback readiness 和 no-new-write-path safety flags；真实 production observation 或 read-cut Level B 必须另写 execution record。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- 可派发给既有 Stage R 开发线，思考程度 high / xhigh。
- 开发线不得提交；主管线负责 fresh verify、复核、入口同步和 commit。
- 复核线必须只读复核，不改文件。

Level split：

- Level A：fixture / temp production-observation contract implementation。必须先完成；只允许 repo fixture 或 temp DB / temp projection root / temp report root，不读取真实 workbench state root，不创建真实 production DB，不切真实产品读路径，不停写 JSON / sidecar。
- Level B：optional real workbench-owned production observation / export verification。只有 Level A 通过、任务包 evidence 追加 Level B execution record、真实 production DB 已由单独任务包创建且主管自审通过后才可执行。Level B 仍不得 stop-write，且必须证明可回退到 verified JSON fallback。

Fallback If Scope Expands：

- 如果实现需要 app startup hook、Tauri command、UI 接入、真实 product global read path、stop-write JSON、真实 Codex 执行、`.codex`、secret / full transcript、provider credential，立即停止并拆新任务包。

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
- `tasks/2026-06-11-root-treatment-r3-a5-fixture-only-observation-export-and-rollback-verification-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- A5 / A9 / A10 evidence 和 handoff。

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A11 Level A 必须完成：

- 新增或扩展 production observation contract helper，建议继续落在 `workbench_sqlite_observation_period.rs`，避免新增平行观察模块。
- Helper 必须显式接收：
  - `observation_mode`
  - `feature_flag_enabled`
  - `read_model_name`
  - `db_path`
  - `json_fallback_root`
  - `projection_root`
  - `observation_report_path`
  - `rollback_manifest_path`
  - `expected_db_hash`
  - `expected_fallback_hash`
  - `allowed_read_models`
  - `denied_path_markers`
  - optional failure injection point
- `observation_mode` Level A 默认只能是 `level_a_fixture_temp`，不得命名为真实 `production_live`。
- `read_model_name` Level A 默认只能承接 R3-A10 的低风险模型：`workflow_state_summary`。
- `feature_flag_enabled=false` 时必须直接记录 `feature_flag_disabled_json_fallback_observation`，不得打开 / 创建 DB。
- `feature_flag_enabled=true` 时只允许在 temp DB 上执行 limited observation / export verification；DB unavailable、schema mismatch、integrity failure 必须 degraded fallback；DB hash mismatch、fallback hash mismatch、export hash mismatch、projection missing / corrupt、rollback manifest missing / incomplete、observation drift 必须 blocked，且不写 stable completed report。
- Observation success 必须至少包括两次 deterministic samples：
  - sample 1 从 temp DB export dry-run 得到 export hash / per-file hash / record counts。
  - sample 2 在同一 fixture / DB / projection root 下重跑，验证 export hash、projection hash、counts 和 redaction policy 稳定。
- JSON fallback observation 必须来自 verified fallback root 的 `workflow-state.v0.json` summary，并校验 `expected_fallback_hash`；不得把未校验 source JSON 当成 fallback success。
- Export verification 必须覆盖：
  - canonical `runtime-logs.v1.json` export，不输出 legacy singular `runtime-log.v1.json`。
  - 每个 projected file 的 path、hash、record_count、redaction_status。
  - source / fallback root hash。
  - db path hash。
  - db export hash。
  - projection hash。
  - export manifest hash。
  - fallback projection hash。
  - forbidden body omission policy。
- Report 必须记录：
  - schema version：建议 `workbench_sqlite_production_observation.v1`
  - mode：`production_observation_export_verification`
  - level：`level_a_fixture`
  - status：`stable_verified` / `fallback_degraded` / `feature_flag_disabled_json_fallback_observation` / `blocked` / `failed_classified`
  - read model name
  - feature flag state
  - observation source：`db_limited_observation` / `json_fallback`
  - fallback decision
  - degraded flag
  - DB path hash
  - fallback root hash
  - projection hash
  - export manifest hash
  - rollback manifest hash
  - expected / actual DB hash
  - expected / actual fallback hash
  - row / record counts
  - sample summaries
  - recovery dry-run
  - safety flags
  - failure point if any
- Safety flags 必须为：
  - `production_observation_enabled=true` only when feature flag true and DB observation succeeds for the selected model.
  - `product_global_read_path_changed=false`
  - `app_startup_reads_db=false`
  - `tauri_command_reads_db=false`
  - `ui_reads_db=false`
  - `stop_write_json=false`
  - `source_json_written=false`
  - `new_write_path_added=false`
  - `production_restore_performed=false`
  - `codex_home_touched=false`
- Recovery dry-run 必须说明：
  - would-disable limited read-cut / production observation before recovery。
  - would-use verified JSON fallback or last verified JSON projection。
  - would-preserve DB for audit。
  - would-require supervisor decision before production restore。
  - production_restore_performed=false。
- Failure injection 必须覆盖：
  - feature flag disabled fallback observation。
  - DB unavailable fallback。
  - DB schema mismatch fallback。
  - DB integrity failure fallback。
  - DB hash mismatch blocked。
  - fallback hash mismatch blocked。
  - export hash mismatch blocked。
  - projection file missing blocked。
  - projection file corrupt blocked。
  - observation drift between samples blocked。
  - rollback manifest missing blocked。
  - rollback manifest incomplete blocked。
  - after first sample before second sample leaves no stable report。
  - after fallback selected before report commit leaves no stable report。
  - after rollback selected before report commit leaves no stable report。
- Idempotent rerun：同一 fixture / temp DB / projection / fallback / manifest 的 report hash 稳定，或显式 already completed classification。
- Report / projection / manifest 不得包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body。

R3-A11 Level B 如执行，必须另写 execution record，并且至少满足：

- 明确真实 production DB path、source JSON fallback root、feature flag scope、selected read model、rollback command / rollback evidence。
- 必须记录 before / after JSON / sidecar hashes，证明未停写、未改源文件。
- 必须记录 DB file hash、schema version、selected observation query hash、fallback projection hash、report hash。
- 必须提供立即回退到 JSON fallback 的操作记录或 dry-run 证据。
- 仍不得让 app startup / Tauri command / UI / 产品全局读路径默认读取 DB。
- 仍不得 stop-write JSON / sidecar。

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

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11/**`
- 可写测试用 temp DB / temp projection / temp fallback / temp report。
- `evidence/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1-result.md`
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

R3-A11 禁止：

- 不切全局产品读路径到 DB。
- 不让 app startup / Tauri command / UI 读取 SQLite。
- 不停写 JSON / sidecar。
- 不修改任何 source JSON / sidecar。
- 不创建真实 production DB。
- 不把 A9 / A10 Level A temp DB 当成真实 production DB。
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
- 不把 production observation contract 冒充为 production read-cut、JSON / sidecar stop-write、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。

## 7. 形状影响

- 任务类型：治理任务包 / production observation export verification contract。
- 代码落点：优先扩展 `workbench_sqlite_observation_period.rs`。
- 触碰棘轮文件：不得新增 `lib.rs` module declaration，除非主管线确认必须新增模块。
- 新文件上限：
  - Rust 新文件如不可避免，必须低于 3,000 行。
  - fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`eda6a4968839a9c470de03ab360f586a0a8060e1`
- 本任务完成 commit：待主管线回收后记录。

## 8. Fixture 矩阵

必须至少新增并测试：

- `production-observation-feature-flag-disabled-fallback`：flag off，直接 JSON fallback observation，不打开 DB。
- `production-observation-db-stable-success`：flag on，temp DB observation success，two-sample stable。
- `production-observation-db-unavailable-fallback`：DB missing，verified JSON fallback，degraded。
- `production-observation-schema-mismatch-fallback`：schema mismatch，verified JSON fallback，degraded。
- `production-observation-integrity-failure-fallback`：integrity failure，verified JSON fallback，degraded。
- `production-observation-db-hash-mismatch-blocked`：DB hash mismatch，blocked，无 stable report。
- `production-observation-fallback-hash-mismatch-blocked`：fallback hash mismatch，blocked，无 stable report。
- `production-observation-export-hash-mismatch-blocked`：export hash mismatch，blocked，无 stable report。
- `production-observation-projection-missing-blocked`：projection file missing，blocked。
- `production-observation-projection-corrupt-blocked`：projection file corrupt，blocked。
- `production-observation-drift-between-samples-blocked`：两次 sample drift，blocked。
- `production-observation-manifest-missing-blocked`：rollback manifest missing，blocked。
- `production-observation-manifest-incomplete-blocked`：rollback manifest incomplete，blocked。
- `production-observation-sensitive-redaction`：report / projection 不含 forbidden sensitive body classes。
- `production-observation-idempotent-rerun`：同输入重跑结果稳定。

## 9. 验收标准

R3-A11 Level A 可接受为：

- production observation / export verification contract helper 已实现或现有 observation helper 已扩展。
- 一个低风险 read model 在 feature flag 下完成 DB observation / JSON fallback / blocked 三类结果验证。
- feature flag off 不打开 DB，直接 fallback observation。
- DB unavailable / schema mismatch / integrity failure 能 fallback 且状态 degraded。
- DB hash mismatch / fallback hash mismatch / export hash mismatch / projection missing / projection corrupt / observation drift / manifest missing / manifest incomplete 均阻断或失败分类，不写 stable report。
- Report 安全 flags 明确保持全局产品读路径、startup、Tauri command、UI、stop-write、source JSON 写入、新写路径、production restore 为 false。
- Recovery dry-run 明确回退 JSON fallback / last verified projection，且不执行 production restore。
- 未创建真实 production DB。
- 未读取真实 workbench state root。
- 未新增 Tauri command / UI / startup hook。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 明确 Level B 是否执行；默认应记录未执行真实 production observation。

R3-A11 不接受为：

- production read-cut 完成。
- app 真实读取 SQLite。
- production observation Level B 完成。
- JSON / sidecar stop-write。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 恢复。

## 10. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_observation
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
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11
rg -n '(product_global_read_path_changed|app_startup_reads_db|tauri_command_reads_db|ui_reads_db|stop_write_json|source_json_written|new_write_path_added|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11
```

如果 fixture 文件尚未创建，扫描命令可改为实际存在的 R3-A11 文件清单，但 evidence 必须说明。

## 11. Evidence / Handoff 要求

Evidence 必须记录：

- planning baseline commit。
- implementation commit 或未提交状态。
- changed files。
- feature flag behavior。
- selected read model。
- DB observation success / fallback / blocked 矩阵。
- export verification summary。
- recovery dry-run。
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
- 下一步建议：R3-A12 stop-write JSON decision 是否仍需单独任务包，或是否需要另行决策 A11 Level B。

## 12. 复核线检查清单

复核线必须检查：

- 是否仍只是 Level A fixture / temp。
- 是否没有新增 Tauri command、startup hook、UI 或真实 product read path。
- feature flag off 是否不打开 / 创建 DB。
- DB unavailable / schema mismatch / integrity failure 是否 fallback degraded。
- hash / export / projection / manifest mismatch 是否 blocked 且不写 stable report。
- recovery dry-run 是否没有执行 restore。
- safety flags 是否没有越界 true。
- evidence / handoff 是否没有夸大为 production observation Level B、production read-cut、stop-write 或 R3 完成。
- 是否没有 `.codex`、secret、token、full transcript、rollout 越界。

## 13. 完成后仍禁止声明

完成 R3-A11 Level A 后仍不得声明：

- production read-cut 完成。
- production observation Level B 完成。
- app 真实 SQLite 读路径已启用。
- JSON / sidecar stop-write 完成。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
