# Root Treatment / R2-B8 Diagnostics Provider Continuation Adapter Boundary Extraction v1

日期：2026-06-11

状态：已完成，经主管线回收为 `accepted_with_p2`。本文是 Root Treatment / Stage R 的 R2 第八批治理任务包，用于把 `src-tauri/src/lib.rs` 中 diagnostics、provider availability、session continuation preview / guard、agent adapter descriptors 和 session operation descriptors 相关边界物理抽出到独立 helper 文件，继续推进小批次、行为不变、可回滚的 `lib.rs` 解体路径。

R2-B8 是行为不变的形状治理任务，不新增产品能力，不执行真实 Codex，不迁移 SQLite，不读写 `/Users/yoyi/.codex`。

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
- 当前 `lib.rs` 为 18,932 行。
- 当前 `lib.rs` 中 diagnostics / provider / continuation / adapter / operation descriptor 实际连续块为 `derive_diagnostic_summary` 到 `session_operation_descriptor_for_adapter`。
- `build_snapshot` / `build_snapshot_with_session_source`、session/index parsing、host OS helper、Tauri app assembly 和 inline tests 不属于本批次。

R2-B8 的核心判断：

```text
把 diagnostics / provider / continuation / adapter / session operation descriptor 边界从 lib.rs 移出；只搬位置，不改行为。
```

说明：默认采用保守 `include!` helper，例如 `diagnostics_provider_session_entrypoints.rs`，让函数仍在 crate root 展开，避免一次性改大量可见性。如果开发线判断需要缩小范围，必须先搬连续低风险块并在 evidence 中说明未搬部分；不得扩大到 snapshot assembly、index parser、host OS helper、app assembly、SQLite、UI 或 tests 巨石。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b7-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b7-supervisor-checkpoint-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostic.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`

## 2. 目标

R2-B8 必须完成：

- 新增 helper 文件，例如 `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs`。
- 将 `lib.rs` 中 diagnostics、store integrity、provider availability、session continuation preview / guard、agent adapter descriptor 和 session operation descriptor 相关函数物理移出。
- `lib.rs` 原位置保留一个 `include!("diagnostics_provider_session_entrypoints.rs")` 或等价保守入口；若使用正式 `mod`，必须解释原因并证明行为不变。
- `lib.rs` 行数必须继续低于 18,932。
- 新增 Rust 文件必须低于 3,000 行。
- 不改任何函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 不新增 `#[tauri::command]`。
- 不新增 sidecar JSON 种类。
- 写 R2-B8 evidence / handoff。

建议优先覆盖的函数 / 区域：

- `derive_diagnostic_summary`
- `workflow_state_integrity`
- `json_file_integrity`
- `text_file_integrity`
- `sidecar_integrity`
- `derived_store_integrity_findings`
- `derive_provider_availability_summaries`
- `provider_availability_for_adapter`
- `provider_kind_for_adapter`
- `derive_session_continuation_previews`
- `active_session_bindings_for_adapter`
- `session_continuation_preview_for_binding`
- `continuation_prompt_source_kind`
- `continuation_prompt_summary`
- `continuation_readback_expectation`
- `continuation_failure_boundary`
- `continuation_audit_impact`
- `inspect_session_continuation_guard`
- `sensitive_path_like`
- `path_within_scope`
- `derive_agent_adapter_descriptors`
- `planned_agent_adapter_descriptors`
- `planned_agent_adapter_descriptor`
- `adapter_capability`
- `derive_session_operation_descriptors`
- `session_operation_specs`
- `session_operation_descriptor_for_adapter`

必须留在本批次外：

- `build_snapshot` / `build_snapshot_with_session_source`。
- `load_sessions`、`parse_projects`、`parse_sessions`、`parse_tasks` 和 index parser。
- `copy_to_clipboard`、`run_open` 和 Tauri app assembly。
- SQLite migration。
- UI / TypeScript。
- worker_protocol / real_execution_command / project_workflow_automation 模块内部重构。
- inline tests 巨石整体迁移。

## 3. 允许读取

- 全部项目源码和文档。
- git 元数据。
- R0/R1/R2-B1/R2-B2/R2-B3/R2-B4/R2-B5/R2-B6/R2-B7 evidence / handoff / supervisor checkpoint。

## 4. 允许写入

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/diagnostics_provider_session_entrypoints.rs` 或同等命名的新 helper 文件
- `evidence/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b8-diagnostics-provider-continuation-adapter-boundary-extraction-v1-result.md`

本线默认不更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`；入口同步由主管线 checkpoint 统一做。

## 5. 禁止事项

R2-B8 禁止：

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
- 不顺手拆 snapshot assembly、index parser、host OS helper、Tauri app assembly、SQLite、UI 或 tests 巨石。

## 6. 形状影响

- 任务类型：治理任务包。
- 新增代码落点：`src-tauri/src/diagnostics_provider_session_entrypoints.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，目标是行数继续下降。
- 预计行数变化：`lib.rs` 预计减少约 1,800-2,200 行；新增 Rust 文件必须小于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`204ab39fa2dadb5e3f16cb506a97fe8c5b2e8615`。
- 本任务完成 commit：待完成后记录。

## 7. 验收标准

R2-B8 可接受为：

- 指定 diagnostics / provider / continuation / adapter / session operation descriptor helper 已从 `lib.rs` 物理抽出。
- `lib.rs` 行数低于 18,932。
- 新增 Rust 文件低于 3,000 行。
- Tauri command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。
- `cargo test --lib diagnostic` 通过。
- `cargo test --lib provider_availability` 通过。
- `cargo test --lib session_continuation` 通过。
- `cargo test --lib agent_adapter` 通过。
- `cargo test --lib session_operation` 通过。
- `cargo test --lib workbench_snapshot` 通过。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline` 和 `--mode check` 通过。
- `git diff --check` 通过。
- evidence / handoff 记录 start commit、end commit、前后行数、验证结果和 P2。

R2-B8 不接受为：

- R2 全部完成。
- `lib.rs <= 15,000` 第一阶段目标完成，除非实际达到并由主管线单独确认。
- diagnostics 自动修复完成。
- provider credential / model verification 完成。
- planned adapters 真实接入。
- session continuation 真实 send / resume 新能力完成。
- runtime log、worker protocol、real execution command 或 project workflow automation 模块内部重构完成。
- workflow state schema 迁移完成。
- 新真实执行授权或 Stage L 恢复。

## 8. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib diagnostic
cargo test --lib provider_availability
cargo test --lib session_continuation
cargo test --lib agent_adapter
cargo test --lib session_operation
cargo test --lib workbench_snapshot
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

## 10. 总指导回收动作

总指导回收时必须判断：

- `accepted`
- `accepted_with_p2`
- `needs_changes`
- `blocked`

P0/P1 示例：

- helper 抽出后编译失败。
- command 总量变化或 `lib.rs` 出现新的 `#[tauri::command]`。
- helper 中新增真实 Codex 执行、`.codex` 访问、SQLite migration 或 provider credential 读取。
- 把 snapshot assembly / index parser / app assembly / tests 巨石混进本批次。

P2 示例：

- 仍使用 `include!` 过渡。
- 相关 tests 仍留在 `lib.rs` inline tests。
- R2-B8 完成后仍未达到 R2 水位线。
