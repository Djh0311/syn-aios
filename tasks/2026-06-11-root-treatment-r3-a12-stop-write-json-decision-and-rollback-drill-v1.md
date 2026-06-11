# Root Treatment / R3-A12 Stop-Write JSON Decision And Rollback Drill v1

日期：2026-06-11

状态：Level A 已完成，Level B 未执行。本文是 Root Treatment / Stage R 的 R3-A12 任务包，用于在 R3-A11 Level A production observation / export verification contract 完成后，先把 stop-write JSON / sidecar 做成显式 supervisor decision contract 和 Level A fixture rollback drill。A12 默认不执行真实 stop-write，不读取真实 workbench state root，不创建真实 workbench-owned production DB，不切 app startup / Tauri command / UI / 产品全局读写路径，不修改任何 source JSON / sidecar，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

规划基线 commit：`be8dac4430066705b5c400d255830f3f31887d60`

## 0. 全局主管理解

已知事实：

- R3-A9 Level A 已完成 fixture / temp production DB initializer + apply with backup manifest / no read-cut，implementation commit 为 `52d6b4b73dcb49e4ffc582dac500d9ad6a8ee4df`；Level B 未执行。
- R3-A10 Level A 已完成 `workflow_state_summary` 单一低风险 read model 的 fixture / temp limited read-cut contract，implementation commit 为 `b18424c38bf0f36f8c9b8ee783a0010598ca9683`；Level B 未执行。
- R3-A11 Level A 已完成 production observation / export verification fixture / temp contract，implementation commit 为 `a7d715c49888b9d3ec67c36c3e431f07e14af12a`；Level B 未执行。
- 真实 workbench state root 未读取，真实 workbench-owned production DB 未创建，真实 production limited read-cut / production observation 未执行。
- 因此 A12 不能默认执行真实 stop-write JSON / sidecar；只能先做 Level A decision contract / rollback drill。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

核心判断：

```text
R3-A12 Level A 的目标是证明“什么时候不得停写”和“如果未来停写失败如何回滚”，不是现在真的停写。真实 stop-write Level B 只能在真实 production DB、真实 read-cut / observation、rollback manifest 和主管 stop-write decision 全部具备后另写 execution record。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- 可复用既有 Stage R 开发线；思考程度 high / xhigh。
- 复核线只读复核，不改文件、不提交。
- 主管线负责 fresh verify、入口同步和 commit。

Level split：

- Level A：fixture / temp stop-write decision contract + rollback drill。必须完成；只允许 repo fixture 或 temp DB / temp projection root / temp report root，不读取真实 workbench state root，不创建真实 production DB，不切真实产品读写路径，不停写 JSON / sidecar。
- Level B：optional real stop-write JSON / sidecar decision。只有 A9/A10/A11 Level B 或等价真实 production evidence 完成、A12 Level A 通过、主管自审 execution record 完整、备份/回滚点存在后才允许另行执行。

Fallback If Scope Expands：

- 如果实现需要真实 workbench state root、真实 production DB、app startup hook、Tauri command、UI 接入、真实 product global read/write path、真实 stop-write、真实 Codex 执行、`.codex`、secret / full transcript、provider credential，立即停止并拆新任务包。

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
- `tasks/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`
- A9 / A10 / A11 evidence 和 handoff。

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A12 Level A 必须完成：

- 新增 stop-write decision contract helper，建议落在新文件 `workbench_sqlite_stop_write.rs`，避免继续膨胀 `workbench_sqlite_observation_period.rs`。
- `lib.rs` 只允许新增 module declaration，不新增 Tauri command、不接 startup、不接 UI。
- Helper 必须显式接收：
  - `decision_mode`
  - `decision_actor`
  - `supervisor_decision`
  - `read_model_name`
  - `db_path`
  - `json_fallback_root`
  - `last_verified_projection_root`
  - `stop_write_report_path`
  - `rollback_manifest_path`
  - `expected_db_hash`
  - `expected_fallback_hash`
  - `expected_projection_hash`
  - `expected_observation_report_hash`
  - `allowed_read_models`
  - `denied_path_markers`
  - optional failure injection point
- `decision_mode` Level A 默认只能是 `level_a_fixture_stop_write_decision`。
- `supervisor_decision` Level A 允许：
  - `prepare_only`
  - `reject_stop_write`
  - `approve_stop_write`
- `decision_actor` Level A 只能是 `global_supervisor` / `supervisor_user`，非 supervisor actor 必须 blocked。
- 在 A9/A10/A11 Level B 未执行的默认事实下，`approve_stop_write` 必须 blocked，不能写 completed report。
- `read_model_name` Level A 默认只能承接 A10/A11 的低风险模型：`workflow_state_summary`。
- `prepare_only` 必须输出 `not_ready` / `blocked_preconditions` 合同，不改变任何 source JSON / sidecar。
- `reject_stop_write` 必须输出 `rejected_by_supervisor` 合同，不改变任何 source JSON / sidecar。
- `approve_stop_write` 只有在所有 preconditions 为真时才允许生成 `ready_but_not_executed` report；Level A 默认 fixture 必须证明缺少真实 Level B evidence 时会 blocked。
- Preconditions 必须至少包含：
  - production DB exists / DB hash matches。
  - selected read model allowed。
  - verified JSON fallback hash matches。
  - last verified projection hash matches。
  - observation report hash matches。
  - rollback manifest exists / complete / dry-run only。
  - no source JSON / sidecar mutation before decision。
  - no app startup / Tauri command / UI / product global path cutover。
- Rollback drill 必须说明：
  - would disable stop-write mode。
  - would re-enable JSON / sidecar write path。
  - would use last verified JSON projection。
  - would preserve DB for audit。
  - would require supervisor decision。
  - production restore performed = false。
- Safety flags 必须为：
  - `stop_write_decision_recorded=true` only for completed prepare/reject/ready reports。
  - `stop_write_json=false`。
  - `source_json_written=false`。
  - `sidecar_written=false`。
  - `product_global_write_path_changed=false`。
  - `product_global_read_path_changed=false`。
  - `app_startup_writes_db=false`。
  - `tauri_command_writes_db=false`。
  - `ui_writes_db=false`。
  - `production_restore_performed=false`。
  - `codex_home_touched=false`。
- Report 必须记录：
  - schema version：建议 `workbench_sqlite_stop_write_decision.v1`
  - mode：`stop_write_json_decision`
  - level：`level_a_fixture`
  - status：`not_ready` / `rejected_by_supervisor` / `ready_but_not_executed` / `blocked` / `failed_classified`
  - supervisor decision
  - read model name
  - precondition matrix
  - DB / fallback / projection / observation / rollback hashes
  - rollback drill
  - safety flags
  - before / after source file hashes
  - failure point if any
  - do-not-claim list
- Failure injection 必须覆盖：
  - missing supervisor decision blocked。
  - non-user / non-supervisor decision blocked。
  - approve without A9/A10/A11 Level B evidence blocked。
  - DB missing blocked。
  - DB hash mismatch blocked。
  - fallback hash mismatch blocked。
  - projection hash mismatch blocked。
  - observation report hash missing / mismatch blocked。
  - rollback manifest missing / incomplete blocked。
  - source JSON / sidecar mutation detected blocked。
  - denied path marker blocked。
  - rollback drill says restore performed blocked。
  - after preconditions before report commit leaves no completed report。
  - sensitive redaction。
  - idempotent rerun。

R3-A12 Level B 如执行，必须另写 execution record，并且至少满足：

- 真实 production DB path、source JSON fallback root、last verified projection root、feature flag scope、selected read model、rollback command / rollback evidence 明确。
- A9/A10/A11 Level B 或等价真实 production evidence 已完成。
- before / after JSON / sidecar hashes 记录完整。
- stop-write decision actor、decision id、prompt/ref/hash、allowed roots、denied paths、runtime log、audit、readback、rollback / recovery 记录完整。
- 不触碰 `/Users/yoyi/.codex`，除非另有计划内真实 Codex 执行点并单独授权。
- 不自动删除真实用户数据。

## 4. 允许读取

Level A 允许读取：

- `product-line` 内源码、任务包、docs、evidence、handoff、fixtures、git metadata。
- R3 fixtures 和测试创建的 temp roots。

Level B 可选允许读取：

- 任务包 evidence 中明确记录的 workbench-owned production DB path、JSON fallback root 和 last verified projection root。
- 只读 metadata/hash/schema/revision/top-level counts 和所选 read model 必需字段。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout、prompt body。
- 用户真实项目源码内容。
- 任意未列入任务包 allowed roots 的路径。

## 5. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` module declaration only。
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12/**`
- 可写测试用 temp DB / temp projection / temp fallback / temp report。
- `evidence/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1-result.md`
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

R3-A12 禁止：

- 不真实 stop-write JSON / sidecar。
- 不切全局产品读写路径到 DB。
- 不让 app startup / Tauri command / UI 读取或写入 SQLite。
- 不修改任何 source JSON / sidecar。
- 不创建真实 production DB。
- 不把 A9 / A10 / A11 Level A temp DB 当成真实 production DB。
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
- 不把 stop-write decision contract 冒充为真实 stop-write、production read-cut、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。

## 7. 形状影响

- 任务类型：治理任务包 / stop-write decision contract。
- 代码落点：新增 `workbench_sqlite_stop_write.rs`，避免继续膨胀 `workbench_sqlite_observation_period.rs`。
- 触碰棘轮文件：`lib.rs` 仅新增 module declaration。
- 新文件上限：
  - `workbench_sqlite_stop_write.rs` 必须低于 3,000 行。
  - fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`be8dac4430066705b5c400d255830f3f31887d60`
- 本任务完成 commit：`eacfad7c4a916f1307e633a37a6084a9fc2927e6`

## 8. Fixture 矩阵

必须至少新增并测试：

- `stop-write-prepare-only-not-ready`：prepare only，输出 not ready / blocked preconditions，不停写。
- `stop-write-rejected-by-supervisor`：主管拒绝 stop-write，输出 rejected，不停写。
- `stop-write-approve-without-level-b-blocked`：没有真实 Level B evidence 时 approve 被 blocked。
- `stop-write-ready-but-not-executed`：fixture 满足全部 fake production evidence 时输出 ready_but_not_executed，仍不真实 stop-write。
- `stop-write-db-missing-blocked`
- `stop-write-db-hash-mismatch-blocked`
- `stop-write-fallback-hash-mismatch-blocked`
- `stop-write-projection-hash-mismatch-blocked`
- `stop-write-observation-report-missing-blocked`
- `stop-write-observation-report-hash-mismatch-blocked`
- `stop-write-rollback-manifest-missing-blocked`
- `stop-write-rollback-manifest-incomplete-blocked`
- `stop-write-source-mutation-detected-blocked`
- `stop-write-denied-path-marker-blocked`
- `stop-write-rollback-restore-performed-blocked`
- `stop-write-after-preconditions-before-report-commit-no-report`
- `stop-write-sensitive-redaction`
- `stop-write-idempotent-rerun`

## 9. 验收标准

R3-A12 Level A 可接受为：

- stop-write decision contract helper 已实现。
- 缺少 A9/A10/A11 Level B 真实 evidence 时 approve stop-write 会 blocked。
- prepare / reject / ready-but-not-executed 三类 Level A 决策结果可区分。
- rollback drill 明确恢复 JSON / sidecar 写路径、禁用 stop-write、保留 DB for audit、需要 supervisor decision，且不执行 production restore。
- Report 安全 flags 明确保持真实 stop-write、source JSON 写入、sidecar 写入、产品全局读写路径、startup、Tauri command、UI、production restore、Codex home touched 为 false。
- before / after source hashes 证明 source JSON / sidecar 未被修改。
- mismatch / missing / mutation / denied path / rollback restore true 均 blocked，且不写 completed report。
- 未创建真实 production DB。
- 未读取真实 workbench state root。
- 未新增 Tauri command / UI / startup hook。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 明确 Level B 是否执行；默认应记录未执行真实 stop-write。

R3-A12 不接受为：

- JSON / sidecar stop-write 完成。
- production read-cut 完成。
- app 真实 SQLite 读写路径已启用。
- production observation Level B 完成。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 恢复。

## 10. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_stop_write
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
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12
rg -n '(stop_write_json|source_json_written|sidecar_written|product_global_write_path_changed|product_global_read_path_changed|app_startup_writes_db|tauri_command_writes_db|ui_writes_db|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12
```

## 11. Evidence / Handoff 要求

Evidence 必须记录：

- planning baseline commit。
- implementation commit 或未提交状态。
- changed files。
- selected read model。
- supervisor decision matrix。
- precondition matrix。
- rollback drill。
- safety flags。
- before / after source hash verification。
- verification commands。
- scan results。
- boundary confirmation。
- do-not-claim list。

Handoff 必须记录：

- 本轮完成范围。
- Level A / Level B 是否执行。
- 为什么没有真实 stop-write。
- 哪些路径没有接入真实产品读写。
- 下一步建议：是否需要 A11 Level B / A12 Level B，或进入 R3-A13 final acceptance 前还缺什么。

## 12. 复核线检查清单

复核线必须检查：

- 是否仍只是 Level A fixture / temp。
- 是否没有新增 Tauri command、startup hook、UI 或真实 product read/write path。
- approve stop-write 在缺少真实 Level B evidence 时是否 blocked。
- ready_but_not_executed 是否仍不真实 stop-write。
- rollback drill 是否没有执行 restore。
- safety flags 是否没有越界 true。
- source hashes 是否证明 source JSON / sidecar 未改。
- evidence / handoff 是否没有夸大为 stop-write、production read-cut、rollback production workflow 或 R3 完成。
- 是否没有 `.codex`、secret、token、full transcript、rollout 越界。

## 13. 完成后仍禁止声明

完成 R3-A12 Level A 后仍不得声明：

- JSON / sidecar stop-write 完成。
- production read-cut 完成。
- app 真实 SQLite 读写路径已启用。
- production observation Level B 完成。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。
