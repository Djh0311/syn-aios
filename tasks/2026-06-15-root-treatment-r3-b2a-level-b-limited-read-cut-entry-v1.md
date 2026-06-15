# Root Treatment / R3 B2a Level-B Limited Read-Cut Entry v1

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`，待提交前咨询线复扫。

性质：B2 enablement 代码任务包，不是 B2 limited read-cut execution record。本包只新增显式、窄口径的 Level-B confirmed-path limited read-cut 入口，并只用 fixture / temp 测试证明安全门；不读取真实 `WORKBENCH_STATE_ROOT`，不运行真实 B2 env runner，不切产品全局读路径。

## 0. 背景

B1 retry 已完成并创建真实 B1 DB，但当前 read-cut 代码仍是 Level-A only：

- `rehearse_limited_read_cut_level_a` 通过 `validate_limited_read_cut_paths` 强制 temp DB / temp 或 R3-A10 fixture fallback / projection。
- 真实 B1 DB 不在 temp，因此后续 B2 需要一条并列 Level-B confirmed-path 入口。

本包复用 B1 enablement 的策略：新增 Level-B 入口，不放宽 Level-A 旧门。

## 1. 目标

在 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs` 中新增：

- `SqliteLimitedReadCutLevelBConfig`
- `rehearse_limited_read_cut_level_b_workbench_owned_state`
- Level-B confirmed DB / fallback root / projection / manifest / report path 校验
- `#[ignore]` env-gated runner `r3_b2_limited_read_cut_confirmed_paths_requires_env_authorization`

Level-B 入口只放行调用方显式确认的：

- `confirmed_db_path`
- `confirmed_fallback_root`
- `confirmed_projection_root`
- `confirmed_rollback_manifest_path`
- `confirmed_read_cut_report_path`
- `confirmed_work_dir`

## 2. 硬边界

- 不改 Level-A 旧门，不把真实 B1 DB 加入 Level-A allowlist。
- 不运行 B2 env-gated runner；本包只新增 runner 代码。
- 不读取真实 source root。
- 不建 DB；测试只用 fixture/temp DB。
- 不改 app startup / Tauri command / UI / 产品全局读路径。
- 不停写 JSON / sidecar。
- 不执行真实 Codex。
- 不读取或写入 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。
- 不执行 `git add` / `git commit`。

## 3. 允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- 本任务包
- 后续如实现完成并通过复核，可新增对应 evidence / handoff / `CURRENT.md` checkpoint，但本包当前提交前停止。

## 4. 实现要求

实现必须满足：

- `rehearse_limited_read_cut_level_a` 对外行为保持，继续调用原 Level-A path validator。
- 共享私有核心承载 read-cut 主体，Level-A / Level-B 只传不同 path validator 与 report level。
- Level-B DB path 必须与 `confirmed_db_path` 完全一致，绝对、存在、canonical、非 denied，并通过 `verify_db_integrity`。
- Level-B fallback root 必须与 `confirmed_fallback_root` 完全一致，绝对、存在、canonical、非 denied。
- Level-B projection root / rollback manifest / report path 必须与 confirmed path 完全一致，absolute / canonical-or-parent / clean，位于 `confirmed_work_dir` 内，不在 fallback/source root 内，不在 B1 DB 目录内。
- Level-B 不要求 temp DB；这是本包唯一放开的点。
- `report.level` 和 rollback manifest `level` 对 Level-B 必须写 `level_b_workbench_owned_state`。
- feature flag 关闭时返回 `feature_flag_disabled_fallback` / `json_fallback`，证明默认关走 JSON fallback。
- DB 不可用 / schema / integrity 问题走 degraded fallback；DB hash mismatch 阻断且不写 report。
- DB limited 与 JSON fallback 的 `projection_hash` / `counts` 必须可比较，B2b runner 需断言 flag on/off 对 `workflow_state_summary` 等价。
- `write_limited_read_cut_report` 继续强制产品全局读路径、startup、Tauri、UI、stop-write、source write、restore、`.codex` safety flags 为 false。`limited_read_cut_enabled=true` 只代表本次 limited read-cut 局部 flag 生效，不代表产品全局读切。

## 5. 测试要求

按 TDD 新增 / 覆盖：

- Level-B confirmed-path 入口接受非 temp DB，flag on 成功 `db_limited`。
- Level-B flag off 走 `json_fallback`。
- Level-B flag on 的 `projection_hash` / `counts` 与 flag off fallback 等价。
- Level-B 拒绝非 confirmed `db_path`。
- Level-B 拒绝 projection / manifest / report 落在 fallback/source root 内。
- Level-B DB hash 不符阻断且不写 report。
- Level-B DB 不可用时 degraded fallback。
- Level-B source/fallback 文件 hash 前后不变。
- Level-A 继续拒绝非 temp DB，证明旧门未放宽。
- Env-gated ignored runner 存在但默认测试不执行。

## 6. 验证命令

必须运行并记录原始输出：

```bash
cargo test --lib sqlite_read_cut
cargo test --lib sqlite_production
cargo test --lib
cargo fmt -- --check
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
```

## 7. 独立复核要求

复核线只读核验，输出 `STATUS: CLEAR | CLEAR_WITH_P2 | NEEDS_FIXES`。复核重点：

- Level-A path guard 没被放宽。
- Level-B 是 explicit confirmed-path 入口，不自动发现真实 root。
- B2a 未运行真实 env runner，未读取真实 `WORKBENCH_STATE_ROOT`。
- DB path 放开 temp 限制只发生在 Level-B。
- 输出路径被限制在 confirmed work dir，且不在 fallback/source root 或 DB 目录内。
- flag off / on 对 `workflow_state_summary` 等价断言存在。
- safety flags 没把 app startup / Tauri / UI / stop-write / source write / `.codex` 置 true。
- 未修改 stop-write、UI、Tauri、startup、产品全局读路径。

## 8. 停止线

- 任一验证失败且无法快速分类修复。
- Level-A 守卫被迫放宽。
- 需要读取真实 source root 或运行 env-gated runner。
- 需要改 `workbench_sqlite_stop_write.rs`、UI、Tauri、startup、产品全局读路径。
- 复核线出现 P0 / P1。

到达停止线时记录 blocked / failed_classified，不把 partial 说成 completed。

## 9. 不接受为

- B2 limited read-cut 已真实执行。
- 真实 B1 DB 已被读取验证。
- read-cut 已接入 app startup / Tauri / UI / 产品全局读路径。
- stop-write 已执行。
- 完整存储迁移完成。
- R3 Level B 完成。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触发生。
