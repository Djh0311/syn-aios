# Root Treatment / R3-A10 Limited Read-Cut Planning And Feature Flag Fallback v1

日期：2026-06-11

## STATUS

`DONE`

R3-A10 Level A 已完成：limited read-cut contract / feature flag / JSON fallback / blocked matrix 的 fixture + temp rehearsal。

## Scope

本轮接受为：

- `workflow_state_summary` 单一低风险 read model 的 Level A limited read-cut 合同。
- `feature_flag_enabled=false` 时不打开 DB，直接走 JSON fallback。
- `feature_flag_enabled=true` 时只在 temp DB 上尝试 DB limited read。
- DB unavailable / schema mismatch / integrity failure 时 verified JSON fallback，状态 degraded。
- DB hash mismatch / fallback hash mismatch / projection hash mismatch / missing rollback manifest / incomplete rollback manifest 时 blocked，且不写 completed report。
- report safety flags 明确保持产品全局读路径、startup、Tauri command、UI、stop-write、source JSON 写入、production restore、Codex home touched 为 false。
- R3-A10 fixture：`prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10/limited-read-cut-workflow-summary/workflow-state.v0.json`。

本轮不接受为：

- production read-cut。
- app 真实 SQLite 读路径启用。
- production DB 创建。
- JSON / sidecar stop-write。
- rollback production workflow。
- R3 完成。
- 多 agent 并行真实执行解锁。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10/limited-read-cut-workflow-summary/workflow-state.v0.json`
- `tasks/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a10-limited-read-cut-planning-and-feature-flag-fallback-v1-result.md`

## Implementation Summary

新增 `rehearse_limited_read_cut_level_a(...)`：

- 显式接收 read model、feature flag、DB path、JSON fallback root、projection root、rollback manifest path、report path、expected DB hash、expected fallback hash、allowed read models、denied path markers 和 failure point。
- 只允许 `workflow_state_summary`。
- 路径限制为 temp / R3-A10 fixture；拒绝 denied markers。
- feature flag 关闭时不创建也不打开 DB。
- feature flag 开启时使用 temp DB export projection 生成 summary。
- fallback 读取 verified JSON fallback root 的 `workflow-state.v0.json`，并校验 expected fallback hash。
- report 中的 `fallback_root_hash` 使用 fallback root 文件内容 manifest hash，不使用明文路径。
- report 使用 `workbench_sqlite_limited_read_cut.v1` schema。
- recovery dry-run 对 DB success / fallback 都明确：would disable limited read-cut、would use verified JSON fallback、would preserve DB for audit、would require supervisor decision、`production_restore_performed=false`。
- A10 projection / report path 只允许 temp 或 R3-A10 fixture root，明确拒绝 R3-A4 fixture root。

新增 focused tests：

- feature flag disabled fallback。
- DB limited success。
- DB limited success recovery dry-run。
- R3-A4 projection root rejection。
- DB unavailable fallback。
- DB schema mismatch fallback。
- DB integrity failure fallback。
- DB hash mismatch blocked。
- fallback hash mismatch blocked。
- projection hash mismatch blocked。
- missing rollback manifest blocked。
- incomplete rollback manifest blocked。
- after DB read before report commit no report。
- after fallback selected before report commit no report。
- sensitive redaction and idempotent report。

## Verification

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_read_cut`：PASS，26 passed。
- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，438 passed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。

Known warning：Cargo 仍提示既有 `JsonRpcError::invalid_params` unused；非 R3-A10 引入。

## Scan Results

Sensitive / real execution scan:

- 命中 `tasks/...R3-A10...md` 的禁止项和扫描命令文本。
- 命中 `workbench_sqlite_read_cut.rs` 的 denied markers、redaction policy、测试断言字符串和既有 A4 read source 文案。
- 未发现 R3-A10 新增真实 `Command::new("codex")`、`codex exec`、`codex exec resume`、`.codex` 访问、secret/token/provider credential/full transcript/rollout 读取路径。

Forbidden true-flag scan:

- `rg -n '(product_global_read_path_changed|app_startup_reads_db|tauri_command_reads_db|ui_reads_db|stop_write_json|source_json_written|production_restore_performed|codex_home_touched)"?\s*[:=]\s*true' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a10`
- PASS，无命中。

Review fix notes:

- 只读复核线曾指出 P1：DB success 报告的 recovery dry-run 不应因为 `read_source=db_limited` 而显示“不关闭 limited read-cut / 不使用 fallback”。已修复并由 `sqlite_limited_read_cut_db_authoritative_success` 断言覆盖。
- 只读复核线曾指出 P2：A10 不能复用 R3-A4 projection path validation。已新增 A10 专用 projection path guard，并由 `sqlite_limited_read_cut_rejects_r3_a4_projection_root` 覆盖。

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

## Next

主管线已完成只读复核、fresh verify 和 implementation commit 记录。下一步可进入 R3-A11 production observation / export verification 任务包准备；如要执行 A10 Level B，必须另写 execution record。不得直接进入 stop-write JSON。
