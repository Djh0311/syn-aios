# Root Treatment R2-B9 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B9 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 index parsing、allowed paths、host OS helper 和 Tauri app assembly 尾段已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/index_host_app_entrypoints.rs`。
- `lib.rs` 通过 `include!("index_host_app_entrypoints.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 17,042 行降到 16,457 行，低于 R0 水位线。
- 新 helper 文件为 586 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 transcript / rollout 全量读取产品化完成。
- 不接受为 session continuation 真实 send / resume、host OS clipboard / open 扩权、runtime log、worker protocol、real execution command 或 project workflow automation 模块内部重构完成。
- 不接受为 R3 SQLite 迁移、R4 UI / 按页读模型、Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B9 start commit：`d100d73c39ddb014372c48ea5a7eaa643fd15bf7`
- R2-B9 completion commit：`bd63d7f5a12a29443d4d0c97713c1c6b1921cf20`
- 本 supervisor checkpoint 提交：`5e3f281df9574a61520f8995dc6539e61020dd56`。

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib transcript
cargo test --lib workbench_snapshot
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 16,457 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `16,457 / 25,925`，status `decreased`。
- `cargo test --lib transcript`：通过，16 passed / 0 failed / 336 filtered out；filter 有匹配，无需 fallback；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib workbench_snapshot`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；ignored 均为显式真实执行授权测试；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：代码验收点通过，无输出；随后仅新增/更新本 supervisor checkpoint 与入口同步文档。

## 主管边界复核

已核对：

- R2-B9 completion commit 只包含 4 个文件：`lib.rs`、`index_host_app_entrypoints.rs`、R2-B9 evidence、R2-B9 handoff。
- 抽出范围从 `software_key_of_session` 到 `run()`，覆盖 session/index parser、allowed path helper、host OS helper 和 Tauri app assembly 尾段。
- `read_index`、前段 transcript fallback loader、C4-C6 自动化工作流治理、task package render / finder helper、shared workflow utility、workbench snapshot assembly、atomic path/time helper 和 inline tests 仍留在 `lib.rs` 或本批次外。
- helper 内没有新增 `#[tauri::command]`、sidecar store、SQLite schema、UI、planned adapter 真实接入、provider credential 读取、真实 Codex 调用或 `.codex` 访问。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B9 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B9 只抽出 index parsing / allowed paths / host OS helper / Tauri app assembly 尾段，不代表 transcript / rollout 全量读取产品化、host OS 扩权、session continuation 真实 send / resume、SQLite、UI 或 tests 巨石拆分完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## 下一步

继续 R2 小批次治理。下一步应先创建 R2-B10 任务包，建议限定为 C4-C6 自动化工作流治理抽出；不顺手做 R3 SQLite、R4 UI/按页读模型、Stage L/K3 恢复或真实 Codex 执行。
