# Root Treatment / R3-A12 Stop-Write JSON Decision And Rollback Drill v1

日期：2026-06-11

## STATUS

`DONE_LEVEL_A`

R3-A12 Level A 已实现：stop-write JSON / sidecar decision contract 和 fixture / temp rollback drill。Level B 未执行。

Planning baseline commit：`be8dac4430066705b5c400d255830f3f31887d60`

Implementation commit：待回填

## Scope

本轮接受为：

- `workflow_state_summary` 单一低风险 read model 的 Level A stop-write decision contract。
- supervisor decision schema：`prepare_only`、`reject_stop_write`、`approve_stop_write`。
- decision actor gate：只允许 `global_supervisor` / `supervisor_user`。
- 缺少 A9/A10/A11 Level B evidence 时，`approve_stop_write` blocked，且不写 completed report。
- 在 fixture 模拟全部前置 evidence 齐全时，`approve_stop_write` 只输出 `ready_but_not_executed`，仍不真实 stop-write。
- rollback drill 明确 would disable stop-write、would re-enable JSON / sidecar write path、would use last verified JSON projection、would preserve DB for audit、would require supervisor decision、`production_restore_performed=false`。
- safety flags 保持真实 stop-write、source JSON 写入、sidecar 写入、产品全局读写路径、startup、Tauri command、UI、production restore、Codex home touched 为 false。
- before / after source hashes 用于证明 source JSON / sidecar 未被修改。

本轮不接受为：

- JSON / sidecar stop-write 完成。
- production read-cut 完成。
- app 真实 SQLite 读写路径启用。
- production observation Level B 完成。
- rollback production workflow 完成。
- R3 完成。
- 多 agent 并行真实执行解锁。

## Changed Files

- `tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12/stop-write-workflow-summary/workflow-state.v0.json`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12/stop-write-workflow-summary/runtime-logs.v1.json`
- `evidence/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1-result.md`

## Implementation Summary

新增 `workbench_sqlite_stop_write.rs`：

- 新增 `rehearse_stop_write_decision_level_a(...)`。
- 显式接收 decision mode、decision actor、supervisor decision、read model、DB path、JSON fallback root、last verified projection root、report path、rollback manifest path、observation report path、expected hashes、allowed read models、denied markers、Level B evidence 和 failure point。
- 只允许 `level_a_fixture_stop_write_decision`。
- 只允许 `workflow_state_summary` read model，并要求 caller 显式列入 allowed read models。
- 只允许 temp path 或 R3-A12 fixture path。
- 合并默认 denied markers：`.codex`、`.env`、token、secret、credential、keychain、OAuth、provider credential、full transcript、rollout、prompt body。
- `prepare_only` 输出 `not_ready` report。
- `reject_stop_write` 输出 `rejected_by_supervisor` report。
- `approve_stop_write` 在 preconditions 不满足时 blocked，不写 completed report。
- fixture 模拟 preconditions 全满足时只输出 `ready_but_not_executed` report，不真实 stop-write。
- `lib.rs` 只新增 `mod workbench_sqlite_stop_write;`，未新增 Tauri command、startup hook、UI 或产品全局读写路径。

新增 R3-A12 fixture：

- `workflow-state.v0.json`：最小 workflow summary fixture。
- `runtime-logs.v1.json`：canonical runtime log fixture。

新增 focused tests：

- prepare only not ready。
- rejected by supervisor。
- approve without Level B evidence blocked。
- ready but not executed with fixture evidence。
- DB missing blocked。
- DB hash mismatch blocked。
- fallback hash mismatch blocked。
- projection hash mismatch blocked。
- observation report missing / mismatch blocked。
- rollback manifest missing / incomplete blocked。
- source mutation detected blocked。
- denied path marker blocked。
- rollback restore performed blocked。
- after preconditions before report commit leaves no report。
- sensitive redaction。
- idempotent rerun。
- missing supervisor / invalid decision / non-supervisor actor blocked。

## Verification

执行时间：2026-06-11 12:59 CST 前后。

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_stop_write`：PASS，16 passed。
- `cargo test --lib sqlite_observation`：PASS，24 passed。
- `cargo test --lib sqlite_read_cut`：PASS，26 passed。
- `cargo test --lib sqlite_production`：PASS，21 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，463 passed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。

Known warning：Cargo 仍提示既有 `JsonRpcError::invalid_params` unused；非 R3-A12 引入。

## Scan Results

Sensitive / real execution scan:

- `rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12 tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- 命中均为任务包禁止项、denied markers 和测试断言字符串。
- 未发现 R3-A12 新增真实 `Command::new("codex")`、`codex exec`、`codex exec resume`、`.codex` 访问、secret/token/provider credential/full transcript/rollout 读取路径。

Forbidden true-flag scan:

- `rg -n '(stop_write_json|source_json_written|sidecar_written|product_global_write_path_changed|product_global_read_path_changed|app_startup_writes_db|tauri_command_writes_db|ui_writes_db|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a12 tasks/2026-06-11-root-treatment-r3-a12-stop-write-json-decision-and-rollback-drill-v1.md`
- 唯一命中为 `sqlite_stop_write_rollback_restore_performed_blocks` 测试中故意构造 `production_restore_performed=true`，用于验证必须 blocked。
- 产品成功路径没有 forbidden true flag。

## Boundary Confirmation

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 未启动 Tauri / Browser / Chrome / Vite / screenshot。
- 未读取真实 workbench state root。
- 未创建真实 workbench-owned production DB。
- 未切产品读写路径到 DB。
- 未停写 JSON / sidecar。
- 未修改 source JSON / sidecar。
- 未新增 Tauri command、UI 或 startup hook。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## Review Notes

- 主管线自查：当前 source mutation failure point 主要在 `approve_stop_write` 路径 blocked；prepare/reject 可记录 precondition false。已提交复核线重点复核是否需要收紧。
- 主管线自查：rollback manifest complete 判定为 `status=completed` 且 `production_restore_performed=false` 的最小合同。已提交复核线重点复核是否需要更多字段。
- 复核线结论：`CLEAR_WITH_P2`。
- P0：无。
- P1：无。
- P2：`verify_rollback_manifest` 当前只以 `status=completed` 且 `production_restore_performed=false` 判定 complete；Level A fixture / temp 可接受。若进入 Level B，建议补 schema/version、rollback boundary 字段完整性、dry-run/source/projection/decision 绑定校验。
- P2：denied marker 主要校验传入 path；`source_file_hashes` 会 hash allowed root 下的直接文件名。当前 R3-A12 fixture 干净且无真实入口，非提交阻断。后续若扩大 caller，建议对子文件名也套 denied marker，避免 `.env`/secret/token 文件被 hash。
- 复核线建议：可以提交；P2 作为后续 Level B 前结构加固。

## Next

R3-A12 Level B 如需执行，必须先补齐 A9/A10/A11 Level B 或等价真实 production evidence，并另写 execution record、allowed roots、rollback strategy 和 fresh verify。

默认下一步建议：不要直接真实 stop-write；应进入 R3-A13 final acceptance / cutover gap matrix，或由主管线单独决策是否先执行 A9/A10/A11/A12 Level B。
