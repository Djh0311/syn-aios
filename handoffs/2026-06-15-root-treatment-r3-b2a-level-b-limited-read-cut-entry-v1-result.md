# Root Treatment / R3 B2a Level-B Limited Read-Cut Entry v1 Result

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`。提交前仍需咨询线复扫放行。

Task package：`tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`

Evidence：`evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`

复核线：Arendt (`019ec9db-8def-73d0-85a8-7cd8ff0ee74b`)

## 1. 完成内容

本包只完成 B2a read-cut enablement，不执行 B2 limited read-cut。

新增显式 Level-B confirmed-path limited read-cut 入口：

- `SqliteLimitedReadCutLevelBConfig`
- `rehearse_limited_read_cut_level_b_workbench_owned_state`
- Level-B confirmed DB / fallback root / work dir / projection / rollback manifest / read-cut report path 校验
- ignored env runner `r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization`

Level-A 旧入口保持原安全语义：

- `rehearse_limited_read_cut_level_a`
- `validate_limited_read_cut_paths`
- temp DB / R3-A10 fixture 守卫

## 2. 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `tasks/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-review-arendt-v1.md`
- `handoffs/2026-06-15-root-treatment-r3-b2a-level-b-limited-read-cut-entry-v1-result.md`
- `CURRENT.md`

## 3. 验证

主管线通过：

- `cargo test --lib sqlite_read_cut`：`29 passed / 1 ignored`
- `cargo test --lib`：`495 passed / 19 ignored`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`：`Status: pass`，`Errors: 0`，`Warnings: 0`
- `git diff --check`：输出为空

## 4. 独立复核

Arendt 复核：`STATUS: CLEAR`。

P0：无。

P1：无。复核线明确确认 `read_cut_report_path` P1 regression fixed。

P2：无阻断项。备注为 Level-B success 当前使用 `for_fallback()`，使 `limited_read_cut_enabled` 也保持 false，比任务包文字更保守。

复核确认：

- Level-A 旧门未放宽。
- Level-B 是显式 confirmed-path 入口。
- `read_cut_report_path` 和 projection / rollback 一样受 confirmed work dir、fallback root deny、DB dir deny、confirmed exact/canonical 校验。
- flag off / on 对 `workflow_state_summary` 的 `projection_hash` / `counts` 等价断言存在。
- source hash 前后不变断言存在。
- safety flags 未把 app startup / Tauri / UI / stop-write / source write / `.codex` 置 true。
- 本轮未运行真实 env-gated runner。

## 5. 下一窗 B2b 前置

B2b limited read-cut 仍未执行。下一窗必须另行用户在场确认并显式传入：

1. `R3_B2_DB_PATH`：真实 B1 DB path。
2. `R3_B2_EXPECTED_DB_HASH`：真实 B1 DB hash。
3. `R3_B2_FALLBACK_ROOT`：真实 source fallback root。
4. `R3_B2_EXPECTED_FALLBACK_HASH`：真实 fallback root canonical hash。
5. `R3_B2_WORK_DIR`：B2 work dir，必须在 source root 与 DB dir 外。
6. `R3_B2_READ_CUT_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_15` 或后续窗口指定的新确认串。

B2b 只允许运行 ignored env-gated runner；不得把 B2a 代码入口启用当成 B2 read-cut 已执行。

## 6. 边界确认

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

## 7. 不接受为

本包不接受为 B2 limited read-cut 已真实执行、read-cut 已切产品全局读路径、stop-write 已执行、完整迁移完成、R3 Level B 完成、多 agent 并行真实执行解锁、真实 Codex 执行或 `.codex` 接触。
