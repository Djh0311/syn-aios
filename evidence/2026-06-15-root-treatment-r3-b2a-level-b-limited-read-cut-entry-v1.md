# Root Treatment / R3 B2a Level-B Limited Read-Cut Entry v1 Evidence

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`。提交前仍需咨询线复扫放行。

Planning baseline：`75e1a1f docs: checkpoint harness C4 checkpoint-audit + evidence_hash_format hardening merged to main`

Task package：`tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`

复核记录：`evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-review-arendt-v1.md`

## 1. 本包目标

本包只做 B2a read-cut enablement，不执行 B2 limited read-cut。

目标是在不放宽 Level-A 旧门的前提下，为后续 B2b 用户在场窗口新增一条显式、窄口径的 Level-B confirmed-path limited read-cut 入口。它允许后续用真实 B1 DB 作为 confirmed DB path 做受控读切演练，但本包自身只用 fixture / temp 测试证明安全门。

本包不读取真实 `WORKBENCH_STATE_ROOT`，不运行真实 B2 env-gated runner，不切产品全局 read path，不停写 JSON / sidecar。

## 2. 完成内容

### 2.1 保留 Level-A 旧门

`rehearse_limited_read_cut_level_a` 对外入口保留，继续通过 `validate_limited_read_cut_paths` 校验。Level-A 仍只接受 temp DB / R3-A10 fixture 口径，不把真实 B1 DB 加入 allowlist。

新增测试 `sqlite_limited_read_cut_level_a_still_rejects_non_temp_db_path` 证明非 temp DB 仍被拒绝。

### 2.2 新增显式 Level-B confirmed-path 入口

新增：

- `SqliteLimitedReadCutLevelBConfig`
- `rehearse_limited_read_cut_level_b_workbench_owned_state`
- `validate_level_b_limited_read_cut_paths`
- `read_limited_confirmed_db_model`
- env-gated ignored runner `r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization`

Level-B 要求调用方显式确认：

- `confirmed_db_path`
- `confirmed_fallback_root`
- `confirmed_work_dir`
- `confirmed_projection_root`
- `confirmed_rollback_manifest_path`
- `confirmed_read_cut_report_path`

任何实际路径与 confirmed path 不一致均拒绝。

### 2.3 P1 修复点

复核前曾发现 `read_cut_report_path` 因 special-case early return 跳过 confirmed work dir containment。

本轮已修：`read_cut_report_path` 现在和 `projection_root` / `rollback_manifest_path` 一样进入 `validate_level_b_output_path`，必须：

- 与 confirmed path 精确一致。
- 是 absolute / clean path。
- canonical-or-parent 后仍与 confirmed path 一致。
- 不在 fallback/source root 内。
- 不在 B1 DB 目录内。
- 位于 confirmed work dir 内。

新增回归测试覆盖 report path outside confirmed work dir 被拒。

### 2.4 默认关闭与等价性

Level-B `feature_flag_enabled=false` 返回 `feature_flag_disabled_fallback` / `json_fallback`，证明默认关闭走 JSON fallback。

Level-B `feature_flag_enabled=true` 成功时返回 `db_limited`，并断言 `workflow_state_summary` 的 `projection_hash` / `counts` 与 flag-off fallback 等价。

source / fallback 文件 hash 前后不变断言存在。

### 2.5 安全标志

本包不切产品全局 read path。测试 helper `assert_limited_safety_flags_all_false` 覆盖：

- `limited_read_cut_enabled`
- `product_global_read_path_changed`
- `app_startup_reads_db`
- `tauri_command_reads_db`
- `ui_reads_db`
- `stop_write_json`
- `source_json_written`
- `production_restore_performed`
- `codex_home_touched`

Arendt 复核备注：Level-B success 当前使用 `for_fallback()`，使 `limited_read_cut_enabled` 也保持 false，比任务包文字更保守。

## 3. 修改范围

代码文件：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`

治理文件：

- `tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-review-arendt-v1.md`
- `handoffs/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1-result.md`
- `CURRENT.md`

未修改：

- 未修改 `workbench_sqlite_stop_write.rs`。
- 未修改 UI / CSS / app startup / Tauri command / 产品全局读写路径。
- 未新增 sidecar JSON kind。
- 未修改真实 workbench state root。
- 未读取或写入 `/Users/yoyi/.codex`。

## 4. 测试覆盖

新增 / 覆盖的关键测试：

- `sqlite_limited_read_cut_level_b_accepts_confirmed_non_temp_db_and_matches_fallback`
- `sqlite_limited_read_cut_level_b_rejects_invalid_confirmed_inputs`
- `sqlite_limited_read_cut_level_a_still_rejects_non_temp_db_path`
- ignored env runner `r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization`

这些测试只使用 fixture / temp / target 下测试数据，不读取真实 `WORKBENCH_STATE_ROOT`。

## 5. 验证记录

### 5.1 `cargo test --lib sqlite_read_cut`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
warning: associated function `invalid_params` is never used
warning: `codex-governance-workbench` (lib test) generated 1 warning
running 30 tests
test workbench_sqlite_read_cut::tests::r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization ... ignored, requires explicit R3 B2 limited read-cut authorization and confirmed paths
test workbench_sqlite_read_cut::tests::sqlite_limited_read_cut_level_a_still_rejects_non_temp_db_path ... ok
test workbench_sqlite_read_cut::tests::sqlite_limited_read_cut_level_b_accepts_confirmed_non_temp_db_and_matches_fallback ... ok
test workbench_sqlite_read_cut::tests::sqlite_limited_read_cut_level_b_rejects_invalid_confirmed_inputs ... ok
test result: ok. 29 passed; 0 failed; 1 ignored; 0 measured; 484 filtered out; finished in 1.68s
```

结果：通过。

### 5.2 `cargo test --lib`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
warning: associated function `invalid_params` is never used
warning: `codex-governance-workbench` (lib test) generated 1 warning
running 514 tests
test workbench_sqlite_read_cut::tests::r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization ... ignored, requires explicit R3 B2 limited read-cut authorization and confirmed paths
test result: ok. 495 passed; 0 failed; 19 ignored; 0 measured; 0 filtered out; finished in 8.92s
```

结果：通过。

### 5.3 `cargo fmt -- --check`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
exit code 0
```

结果：通过。

### 5.4 Shape gate

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

执行目录：仓库根 `/Users/yoyi/workspace/product-line`

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 75e1a1fdc2cc97224415d9490622a3abd08f722f
Tauri commands: 97 total; 0 in lib.rs
Sidecar JSON kinds: 14 detected; 0 unknown
Converged-helper dups outside utils/: 0 (12 deferred-whitelisted)
```

结果：通过。

### 5.5 `git diff --check`

执行目录：仓库根 `/Users/yoyi/workspace/product-line`

```text
exit code 0
```

输出为空。

结果：通过。

## 6. 当前 git 实物

### 6.1 `git status --short`

提交前应包含 B2a 代码、任务包、review / evidence / handoff 与 `CURRENT.md`。本记录写入前的状态为：

```text
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs
?? tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md
```

### 6.2 `git diff --stat`

治理文件写入前代码 diff 摘要：

```text
.../src-tauri/src/workbench_sqlite_read_cut.rs | 814 ++++++++++++++++++++-
1 file changed, 805 insertions(+), 9 deletions(-)
```

## 7. 独立复核

复核线：Arendt (`019ec9db-8def-73d0-85a8-7cd8ff0ee74b`)

复核结论：`STATUS: CLEAR`

P0：无。

P1：无。复核线明确确认 `read_cut_report_path` P1 regression fixed。

P2：无阻断项。备注为 Level-B success 当前 safety flags 更保守。

复核记录见：

- `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-review-arendt-v1.md`

## 8. 边界确认

本包没有：

- 运行 B2 env-gated runner。
- 执行真实 B2 limited read-cut。
- 读取真实 `WORKBENCH_STATE_ROOT`。
- 修改真实 JSON / sidecar。
- 切产品全局 read path。
- 停写 JSON / sidecar。
- 新增 Tauri command。
- 修改 UI / CSS / app startup / 产品全局读写路径。
- 执行真实 Codex。
- 读取或写入 `/Users/yoyi/.codex`。

## 9. 不接受为

本包不接受为 B2 limited read-cut 已真实执行、read-cut 已切产品全局读路径、stop-write 已执行、完整迁移完成、R3 Level B 完成、多 agent 并行真实执行解锁、真实 Codex 执行或 `.codex` 接触。

## 10. 下一步

提交前先交咨询线复扫本 evidence、review、handoff 与 `CURRENT.md` 是否如实；放行后再提交。

B2b 真实 limited read-cut 窗口需另行用户在场确认并运行 ignored env-gated runner；本包不进入 B2b。
