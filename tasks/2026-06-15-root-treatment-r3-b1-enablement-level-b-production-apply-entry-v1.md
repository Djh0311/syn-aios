# Root Treatment / R3 B1 Enablement Level-B Production Apply Entry v1

日期：2026-06-15

状态：已完成

性质：B1 enablement 代码任务包，不是 B1 production apply execution record。本包只新增显式、窄口径的 Level-B production apply 入口，并只用 fixture / temp 测试证明安全门；不读取真实 `WORKBENCH_STATE_ROOT`，不创建真实 production DB，不执行 B1 apply。

## 0. 背景

B0 已提交，commit `688108f`，冻结：

- `WORKBENCH_STATE_ROOT=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`
- `source_root_hash_before=2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53`

B1 当前不能直接执行，原因是现有 production apply 代码仍是 Level-A only：

- `validate_level_a_source_root` 只允许 temp / R3-A9 fixture source root。
- `apply_fixture_dir_to_temp_db` 只允许 temp / R3 fixture DB path。

本包的目标是补一个并列入口，让后续 B1 可以在用户逐点确认路径后受控执行。

## 1. 目标

新增显式 Level-B production apply 入口，例如：

- `SqliteProductionApplyLevelBConfig`
- `rehearse_production_db_apply_level_b_workbench_owned_state`
- 对应的 Level-B source / output path validation
- 对应底层 DB init / apply / export path guard

该入口只放行：

- 调用方显式传入并确认的 `confirmed_source_state_root`。
- 调用方显式传入并确认的 `confirmed_production_db_path`。
- 调用方显式传入并确认的 `confirmed_backup_root`。
- 调用方显式传入并确认的 `confirmed_report_path`。
- 调用方显式传入并确认的 `confirmed_rollback_manifest_path`。

输出路径必须全部在 source state root 之外。后续真实 B1 才能把 B0 冻结的 root 和用户确认路径填入该配置。

## 2. 硬边界

- 不改 Level-A 旧门：`validate_level_a_source_root`、`rehearse_production_db_apply_level_a`、`apply_fixture_dir_to_temp_db` 的安全语义保持。
- 不把真实 root 加进 Level-A allowlist。
- 不读取 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`。
- 不创建真实 production DB。
- 不写真实 backup / report / rollback manifest。
- 不切 read path。
- 不停写 JSON / sidecar。
- 不改 app startup / Tauri command / UI / 产品全局读写路径。
- 不执行真实 Codex。
- 不读取或写入 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。

## 3. 允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- 必要的测试 fixture / 测试辅助代码
- 本任务包、evidence、handoff、`CURRENT.md` checkpoint

默认不新增 Tauri command，不新增 sidecar JSON kind，不改 product UI。

## 4. 实现要求

Level-B 入口必须：

- 要求 `confirmed_source_state_root` 与实际 `source_state_root` 完全一致，否则拒绝。
- 要求 DB / backup / report / rollback 四个输出路径与对应 confirmed path 完全一致，否则拒绝。
- Level-B source root 和输出路径必须是 canonical / clean path；拒绝 `.` / `..` 语义路径、canonical 后文本不一致的别名路径，以及 canonical 后落入 source root 内的输出路径。
- 对所有路径执行 denied marker guard。
- 拒绝任一输出路径位于 source root 内。
- 保留 source before / after hash invariant；apply 后 source JSON / sidecar hash 必须不变。
- 缺失 optional sidecar 必须是 warning / accepted-with-rejections 语义，不是 failure。
- report `level` 必须为 `level_b_workbench_owned_state`，不能复用 `level_a_fixture`。
- Safety flags 必须保持：`read_cut_enabled=false`、`stop_write_json=false`、`production_restore_performed=false`、`codex_home_touched=false`、`product_read_path_changed=false`、`source_json_written=false`。
- DB export verification 仍只允许 canonical `runtime-logs.v1.json`，不得输出 legacy `runtime-log.v1.json`。

## 5. 测试要求

必须新增 / 覆盖以下测试：

- Level-B 入口接受一个测试确认 root，成功完成 temp / fixture apply。
- Level-B 入口拒绝非 confirmed source root。
- Level-B 入口拒绝非 confirmed DB / backup / report / rollback path。
- Level-B 入口拒绝 denied path marker。
- Level-B 入口拒绝输出路径落在 source root 内。
- Level-B 入口拒绝 `..` / non-clean 输出路径语义逃逸。
- Level-B 入口 apply 后 source before / after hashes 不变。
- Level-B 入口对缺失 optional sidecar 只产生 warning，不失败。
- Level-A 旧入口仍拒绝非 temp / non-fixture source root 或 DB path，证明旧门未被放宽。

## 6. 验证命令

必须运行并记录原始输出：

```bash
cargo fmt -- --check
cargo test --lib sqlite_production
cargo test --lib sqlite_apply
cargo test --lib sqlite_export
cargo test --lib
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
```

## 7. 独立复核要求

本包会打开后续真实数据入口，复核必须 thorough。复核线需要确认：

- Level-A 旧门未放宽。
- Level-B 是显式配置入口，不自动发现真实 root。
- fixture-only，本包没有读取真实 `WORKBENCH_STATE_ROOT`。
- denied path guard 和 output outside source root guard 有负向测试。
- source hash invariant 有测试。
- missing optional sidecar 是 warning 非 failure。
- 未新增 read-cut / stop-write / Tauri command / UI / `.codex` 接触。

## 8. 不接受为

- B1 apply 已执行。
- 真实 production DB 已创建。
- 真实数据已迁移。
- Level-A 被放宽。
- read-cut 已执行。
- stop-write 已执行。
- R3 完成。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触已发生。
