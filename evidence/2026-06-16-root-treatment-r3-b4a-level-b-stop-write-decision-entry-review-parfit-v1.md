# R3 B4a Level-B Stop-Write Decision Entry Review Parfit v1

日期：2026-06-16

review_line: Parfit
agent_id: 019ecc10-98a6-7eb2-aeaf-4729a2830f06

STATUS: CLEAR

无 P0 / P1 / P2 / P3。

## Scope Reviewed

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `tasks/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-v1.md`
- 当前未提交 diff 范围

本复核只读核验 B4a 代码与测试入口。未运行 ignored env runner，未触碰真实 `WORKBENCH_STATE_ROOT` / B1 DB，未读取或写入 `/Users/yoyi/.codex`，未读取 secret / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。

## Findings

- P0: none.
- P1: none.
- P2: none.
- P3: none.

## Evidence Checked

- 当前 HEAD 为 `11beb3bf8bafaf78819b1d534deb185603168e89`，短 hash `11beb3b`。
- 未提交产品代码变更仅在 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`；另有本任务文档未跟踪。
- Level-A `rehearse_stop_write_decision_level_a` 仍调用原 `validate_paths`，`validate_paths` 仍要求所有路径为 absolute、不得命中 denied markers，且必须在 temp 或 `fixtures/r3-a12` 下；新增测试 `sqlite_stop_write_level_a_still_rejects_non_temp_db_path` 覆盖非 temp DB 仍被 `level_a_temp_or_fixture_path_required` 拒绝。
- Level-B 新增并列入口 `rehearse_stop_write_decision_level_b_workbench_owned_state` 与 `SqliteStopWriteLevelBConfig`，要求 DB、fallback root、projection root、observation report、work dir、stop-write report、rollback manifest 均逐项 confirmed path 匹配。
- Level-B 仅通过 `export_confirmed_db_to_json_dry_run` 放开 confirmed DB 的 temp 限制；非 confirmed DB path 被测试覆盖并拒绝。
- Level-B 输出路径校验要求 stop-write report / rollback manifest 在 confirmed work dir 内，且不得落入 source root、DB 目录或 projection root；相关测试覆盖 source / projection / DB dir 三类拒绝。
- denied markers 仍由默认敏感 marker 与调用方 marker 合并后生效，Level-B confirmed existing path 与 output path 均调用 marker 拒绝逻辑。
- `approve_stop_write` 的最好状态仍为 `ready_but_not_executed`；缺 evidence 或 hash 不匹配时返回 `preconditions_not_met` 且清理 report / rollback manifest。
- Safety flags 仍只允许 `stop_write_decision_recorded=true`；`write_report` 会拒绝任何实际动作 flag 为 true，包括 stop-write JSON、source JSON 写入、sidecar 写入、产品全局读写路径、startup、Tauri、UI、restore、`.codex`。
- 回滚仍是 `rollback_drill_only`，`production_restore_performed=false`；Level-B 写出的 rollback manifest 是 dry-run decision 产物，不执行 restore。
- 未发现 UI / Tauri / startup / 产品全局读写路径接入；`rehearse_stop_write_decision_level_b_workbench_owned_state` 只在本文件测试与 ignored env runner 中出现。
- Ignored runner `r3_b4_stop_write_decision_confirmed_paths_requires_env_authorization` 存在，带 `#[ignore]` 与显式 env authorization；本复核未运行该 runner。
- 普通测试辅助路径使用 temp root 与 `fixtures/r3-a12`，未触碰真实 state root / B1 DB。

## Verification

本复核复跑：

- `cargo test --lib sqlite_stop_write` -> pass: 22 passed / 1 ignored.
- `cargo fmt -- --check` -> pass.
- `git diff --check` -> pass.

本复核未复跑完整 `cargo test --lib`、shape gate；采信主管线记录：

- `cargo test --lib` -> 503 passed / 21 ignored.
- `node scripts/harness/workbench-shape-gate.js --mode check` -> 0 errors / 0 warnings.

## Boundaries

本包只接受为 B4a Level-B confirmed-path stop-write decision entry 完成。不得声明 B4b 已执行、stop-write 已执行、JSON / sidecar 停写已完成、产品全局读写路径已切换、R3 Level B 完成、真实 Codex 执行完成或 `/Users/yoyi/.codex` 已接触。
