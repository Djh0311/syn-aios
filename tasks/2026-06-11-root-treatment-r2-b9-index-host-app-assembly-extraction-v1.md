# Root Treatment / R2-B9 Index Host App Assembly Extraction v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R2 第九批治理任务包，用于把 `src-tauri/src/lib.rs` 中 `software_key_of_session` 到 Tauri `run()` 之前的剩余尾段边界物理抽出到独立 helper 文件，继续推进小批次、行为不变、可回滚的 `lib.rs` 解体路径。

R2-B9 是行为不变的形状治理任务，不新增产品能力，不执行真实 Codex，不迁移 SQLite，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1 已完成并 checkpoint。
- R2-B1 已完成 command registry extraction，completion commit `13016917442070fc2f59a130b2748eb0cba06a34`。
- R2-B2 已完成 lib map and workflow state JSON helper extraction，completion commit `76ed0ef46d9b0a2a83f6e77ce533d6c8741c93cf`。
- R2-B3 已完成 workflow state lifecycle and task package chain extraction，completion commit `208fabaa4cae8aeda45cdce4c66cbe7f2cf8e6c3`。
- R2-B4 已完成 workflow run binding and legacy dispatch entrypoints extraction，completion commit `66a0cff5a4fb94101c1830a174dc908448ec8dba`。
- R2-B5 已完成 workflow read model dispatch summary and readback stats extraction，completion commit `35cacc22ec813152e9357a42bc82e7ef581d2509`。
- R2-B6 已完成 workflow execution control offline role and machine extraction，completion commit `2dd766be84e977d75e77f31ec2dbf9d463f45690`。
- R2-B7 已完成 memory command bridge and context guard extraction，completion commit `9cd10bb51fe828ae5b2b72501414b5cf025b77a9`。
- R2-B8 已完成 diagnostics provider continuation adapter boundary extraction，completion commit `9935dac822ab41bce2391b8f6a54d6b42eeb4f95`，supervisor checkpoint commit `68c7d4afc135b730eb94a4bbaa790bdb06a3bb6e`。
- 当前 `lib.rs` 为 17,042 行。
- 当前 `lib.rs` 中 R2-B9 可连续抽出的尾段为 `software_key_of_session` 到 `run()`，约 589 行。
- `read_index` / transcript sqlite fallback loader、C4-C6 自动化工作流治理、task package render helper、shared workflow utility、snapshot assembly、atomic path/time helper 和 inline tests 不属于本批次。

R2-B9 的核心判断：

```text
把 session/index parser、allowed path helper、host OS helper 和 Tauri app assembly 尾段从 lib.rs 移出；只搬位置，不改行为。
```

说明：默认采用保守 `include!` helper，例如 `index_host_app_entrypoints.rs`，让函数仍在 crate root 展开，避免一次性改大量可见性。如果开发线判断需要缩小范围，必须优先保留行为并在 evidence 中说明未搬部分；不得扩大到 C4-C6、SQLite、UI、真实执行、前端、workflow state schema 或 tests 巨石。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b8-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b8-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_transcript.rs`

## 2. 目标

R2-B9 必须完成：

- 新增 helper 文件，例如 `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`。
- 将 `lib.rs` 中 session/index parsing、allowed paths、host OS helper 和 Tauri app assembly 尾段函数物理移出。
- `lib.rs` 原位置保留一个 `include!("index_host_app_entrypoints.rs")` 或等价保守入口；若使用正式 `mod`，必须解释原因并证明行为不变。
- `lib.rs` 行数必须继续低于 17,042。
- 新增 Rust 文件必须低于 3,000 行。
- 不改任何函数语义、返回值、错误文案、公开 Tauri command 契约、workflow state schema 或 sidecar schema。
- 不新增 `#[tauri::command]`。
- 不新增 sidecar JSON 种类。
- 写 R2-B9 evidence / handoff。

建议优先覆盖的函数 / 区域：

- `software_key_of_session`
- `load_sessions`
- `load_sessions_from_sqlite_or_index`
- `overlay_project_thread_counts`
- `parse_projects`
- `parse_sessions`
- `parse_codex_transcript`
- `parse_codex_transcript_event`
- `parse_skills`
- `parse_plugins`
- `parse_file_candidates`
- `parse_harness_candidates`
- `parse_harness_resources`
- `parse_harness_entrypoints`
- `parse_tasks`
- `allowed_paths`
- `allowed_paths_with_sessions`
- `extend_allowed_rollouts_from_sqlite`
- `impl AllowedPaths`
- `array_len`
- `optional_string`
- `optional_string_from`
- `optional_i64_from`
- `string_array`
- `usize_value`
- `i64_value`
- `usize_map`
- `bool_value`
- `path_name`
- `copy_to_clipboard`
- `run_open`
- `run`

必须留在本批次外：

- `read_index`。
- `load_codex_session_transcript_for_index` / `load_codex_session_transcript_with_*` / `codex_home_from_index` 等前段 transcript fallback loader。
- C4-C6 自动化工作流治理。
- task package render / finder helper。
- shared workflow utility。
- workbench snapshot assembly。
- atomic path / time helper。
- SQLite migration。
- UI / TypeScript。
- worker_protocol / real_execution_command / project_workflow_automation 模块内部重构。
- inline tests 巨石整体迁移。

## 3. 允许读取

- 全部项目源码和文档。
- git 元数据。
- R0/R1/R2-B1/R2-B2/R2-B3/R2-B4/R2-B5/R2-B6/R2-B7/R2-B8 evidence / handoff / supervisor checkpoint。

## 4. 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs` 或同等命名的新 helper 文件
- `evidence/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1-result.md`

本线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口同步由主管线 checkpoint 统一做。

## 5. 禁止事项

R2-B9 禁止：

- 不改产品业务逻辑。
- 不新增 Tauri command。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不迁移 SQLite。
- 不改 workflow state 顶层 schema。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不顺手拆前段 transcript loader、C4-C6、task package render、shared workflow utility、snapshot assembly、atomic helper、SQLite、UI 或 tests 巨石。

## 6. 形状影响

- 任务类型：治理任务包。
- 新增代码落点：`src-tauri/src/index_host_app_entrypoints.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，目标是行数继续下降。
- 预计行数变化：`lib.rs` 预计减少约 550-650 行；新增 Rust 文件必须小于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`080907c41936d88bb028d8869cc02610038b47c2`。
- 本任务完成 commit：待完成后记录。

## 7. 验收标准

R2-B9 可接受为：

- 指定 session/index parser、allowed path helper、host OS helper 和 Tauri app assembly 尾段已从 `lib.rs` 物理抽出。
- `lib.rs` 行数低于 17,042。
- 新增 Rust 文件低于 3,000 行。
- Tauri command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。
- `cargo test --lib transcript` 通过，或无匹配时记录原因并用 `cargo test --lib codex_transcript` / `cargo test --lib` 覆盖。
- `cargo test --lib workbench_snapshot` 通过。
- `cargo test --lib workflow_state` 通过。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline` 和 `--mode check` 通过。
- `git diff --check` 通过。
- evidence / handoff 记录 start commit、end commit、前后行数、验证结果和 P2。

R2-B9 不接受为：

- R2 全部完成。
- `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成，除非实际达到并由主管线单独确认。
- transcript / rollout 全量读取产品化完成。
- session continuation 真实 send / resume 完成。
- host OS clipboard / open 能力扩权完成。
- Tauri UI / 截图验收完成。
- runtime log、worker protocol、real execution command 或 project workflow automation 模块内部重构完成。
- workflow state schema 迁移完成。
- SQLite 统一存储完成。
- 新真实执行授权或 Stage L 恢复。

## 8. 建议验证命令

必须跑：

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

如果某个 filter 因无匹配或环境问题失败，必须记录完整原因，并由更宽的相关测试或 `cargo test --lib` 覆盖；不得把失败冒充完成。

## 9. 必须回传

开发线回传必须包含：

1. 做了什么。
2. 改了哪些文件。
3. `lib.rs` 前后行数。
4. 新 helper 文件行数。
5. 抽出函数清单。
6. command 总量和 `lib.rs` 内 command 数量。
7. shape gate baseline / check 摘要。
8. Rust 测试和格式化结果。
9. start commit / end commit。
10. P0 / P1 / P2。
11. 是否触碰任何禁止项。
