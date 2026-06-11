# Root Treatment / R3-A11 Production Observation Export Verification Contract v1

日期：2026-06-11

## STATUS

`DONE_PENDING_IMPLEMENTATION_COMMIT_HASH`

R3-A11 Level A 已实现并通过复核：production observation / export verification contract 的 fixture + temp rehearsal。Level B 未执行。

## Scope

本轮接受为：

- `workflow_state_summary` 单一低风险 read model 的 Level A production observation contract。
- `feature_flag_enabled=false` 时不打开 / 不创建 DB，直接记录 verified JSON fallback observation。
- `feature_flag_enabled=true` 时只在 temp DB 上执行 limited observation / export verification。
- DB unavailable / schema mismatch / integrity failure 时使用 verified JSON fallback，状态 degraded。
- DB hash mismatch / fallback hash mismatch / export hash mismatch / projection missing / projection corrupt / observation drift / rollback manifest missing / rollback manifest incomplete 时 blocked，且不写 stable report。
- two-sample deterministic DB observation：sample 1 / sample 2 比对 export hash、projection hash、projected files、row counts 和 redaction policy。
- export verification 覆盖 canonical `runtime-logs.v1.json`，不输出 legacy singular `runtime-log.v1.json`。
- report safety flags 明确保持产品全局读路径、startup、Tauri command、UI、stop-write、source JSON 写入、新写路径、production restore、Codex home touched 为 false。
- recovery dry-run 明确 would disable limited read-cut / production observation、would use verified JSON fallback or last verified JSON projection、would preserve DB for audit、would require supervisor decision、`production_restore_performed=false`。

本轮不接受为：

- production observation Level B。
- production read-cut。
- app 真实 SQLite 读路径启用。
- 真实 workbench-owned production DB 创建。
- JSON / sidecar stop-write。
- rollback production workflow。
- R3 完成。
- 多 agent 并行真实执行解锁。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11/production-observation-workflow-summary/workflow-state.v0.json`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11/production-observation-workflow-summary/runtime-logs.v1.json`
- `evidence/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a11-production-observation-export-verification-contract-v1-result.md`

## Implementation Summary

新增 `rehearse_production_observation_level_a(...)`：

- 显式接收 observation mode、feature flag、read model、DB path、JSON fallback root、projection root、report path、rollback manifest path、expected DB hash、expected fallback hash、allowed read models、denied path markers 和 failure point。
- 只允许 `level_a_fixture_temp` observation mode。
- 只允许 `workflow_state_summary` read model，并要求 caller 显式列入 allowed read models。
- 路径限制为 temp / R3-A11 fixture；拒绝 `.codex`、`.env`、token、secret、credential、keychain、OAuth、provider credential、full transcript、rollout、prompt body 等 denied markers。
- fallback 从 verified JSON fallback root 的 `workflow-state.v0.json` 生成 summary，并校验 expected fallback hash。
- DB success 通过 temp SQLite dry-run export 两次采样，校验稳定性后写 projection 和 rollback manifest。
- fallback / blocked / mid-commit failure 都不会写 stable report。
- report schema 为 `workbench_sqlite_production_observation.v1`。
- projection/report/manifest 不包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential 或 rollout body。

新增 R3-A11 fixture：

- `workflow-state.v0.json`：最小工作流摘要 fixture。
- `runtime-logs.v1.json`：canonical runtime log fixture，用于验证 export 不输出 legacy singular alias。

新增 focused tests：

- feature flag disabled fallback 不打开 DB。
- DB stable success two-sample verified。
- DB unavailable fallback degraded。
- schema mismatch fallback degraded。
- integrity failure fallback degraded。
- DB hash mismatch blocked。
- fallback hash mismatch blocked。
- export hash mismatch blocked。
- projection missing blocked。
- projection corrupt blocked。
- observation drift blocked。
- rollback manifest missing blocked。
- rollback manifest incomplete blocked。
- after first sample / after fallback selected / after rollback selected before report commit 均无 stable report。
- sensitive redaction。
- idempotent rerun。

## Verification

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_observation`：PASS，24 passed。
- `cargo test --lib sqlite_read_cut`：PASS，26 passed。
- `cargo test --lib sqlite_production`：PASS，21 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，447 passed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。

Known warning：Cargo 仍提示既有 `JsonRpcError::invalid_params` unused；非 R3-A11 引入。

## Scan Results

Sensitive / real execution scan:

- `rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11`
- 命中均为 A11 denied markers、redaction policy 和测试断言字符串。
- 未发现 R3-A11 新增真实 `Command::new("codex")`、`codex exec`、`codex exec resume`、`.codex` 访问、secret/token/provider credential/full transcript/rollout 读取路径。

Forbidden true-flag scan:

- `rg -n '(product_global_read_path_changed|app_startup_reads_db|tauri_command_reads_db|ui_reads_db|stop_write_json|source_json_written|new_write_path_added|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a11`
- PASS，无命中。

## Boundary Confirmation

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / screenshot。
- 未读取真实 workbench state root。
- 未创建真实 workbench-owned production DB。
- 未切产品读路径到 DB。
- 未停写 JSON / sidecar。
- 未新增 Tauri command、UI 或 startup hook。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## Review Notes

- 当前 `workbench_sqlite_observation_period.rs` 从约 1,045 行增至 2,405 行，主要增量是 A11 合同 helper 和测试矩阵。任务包允许优先扩展该文件以避免新增平行 observation 模块，但复核线应检查是否需要在后续 R3/R5 做结构校准。
- 复核线结论：`CLEAR`，P0/P1/P2 均无，可提交。
- 当前 implementation commit 尚未写入；等待主管线提交并回填。

## Next

等待复核线只读审查。若 clear，主管线提交 R3-A11 implementation 并更新任务包状态 / checkpoint 文档。R3-A11 Level B 如需执行必须另写 execution record；不得直接进入 stop-write JSON。
