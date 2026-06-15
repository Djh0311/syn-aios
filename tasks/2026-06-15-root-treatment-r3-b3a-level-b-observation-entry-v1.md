# Root Treatment / R3 B3a Level-B Observation Entry v1

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`，待提交前咨询线复扫。

性质：B3 observation enablement 代码任务包，不是 B3b 真实 observation execution record。本包只新增显式、窄口径的 Level-B confirmed-path production observation 入口，并只用 fixture / temp 测试证明安全门；不读取真实 `WORKBENCH_STATE_ROOT`，不运行真实 B3 env runner，不切产品全局读路径，不停写 JSON / sidecar。

## 0. 背景

B2b 已证明 limited read-cut 可以在用户在场窗口中只读验证真实 B1 DB 与 JSON fallback 的 `workflow_state_summary` 等价。下一步 B3 observation 仍需要先补代码入口，因为现有 production observation 只有 Level-A 路径：

- `rehearse_production_observation_level_a` 仍通过 `validate_production_observation_paths` 强制 temp / fixture 口径。
- 真实 B1 DB 不应被加入 Level-A allowlist。
- B3b 后续若要观察真实 B1 DB，需要一条并列 Level-B confirmed-path 入口。

本包复用 B1 enablement 与 B2a read-cut enablement 的策略：新增 Level-B 入口，不放宽 Level-A 旧门。

## 1. 目标

在 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs` 中新增：

- `SqliteObservationLevelBConfig`
- `rehearse_production_observation_level_b_workbench_owned_state`
- Level-B confirmed DB / fallback root / work dir / projection / rollback manifest / observation report path 校验
- `#[ignore]` env-gated runner `r3_b3_observation_confirmed_paths_requires_env_authorization`
- Level-B fixture-only 正向 / 负向测试

Level-B 入口只放行调用方显式确认的：

- `confirmed_db_path`
- `confirmed_fallback_root`
- `confirmed_work_dir`
- `confirmed_projection_root`
- `confirmed_rollback_manifest_path`
- `confirmed_observation_report_path`

## 2. 硬边界

- 不改 Level-A 旧门，不把真实 B1 DB 加入 Level-A allowlist。
- 不运行 B3 env-gated runner；本包只新增 runner 代码。
- 不读取真实 source root。
- 不建真实 DB；测试只用 fixture / temp DB。
- 不改 app startup / Tauri command / UI / 产品全局读路径。
- 不停写 JSON / sidecar。
- 不执行真实 Codex。
- 不读取或写入 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。
- 不执行 `git add` / `git commit`，提交需等待咨询线复扫放行。

## 3. 允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs`
- 本任务包
- 对应 evidence / review / `CURRENT.md` checkpoint

除上述文件外，不改其它产品代码、harness、UI、Tauri command、startup 或真实数据路径。

## 4. 实现要求

实现必须满足：

- `rehearse_production_observation_level_a` 对外行为保持，继续调用原 Level-A path validator。
- 共享私有核心承载 observation 主体，Level-A / Level-B 只传不同 path validator 与 DB 读取路径。
- Level-B DB path 必须与 `confirmed_db_path` 完全一致，绝对、存在、canonical、非 denied，并通过 DB integrity / schema guard。
- Level-B fallback root 必须与 `confirmed_fallback_root` 完全一致，绝对、存在、canonical、非 denied。
- Level-B projection root / rollback manifest / observation report path 必须与 confirmed path 完全一致，absolute / clean / canonical-or-parent，位于 `confirmed_work_dir` 内，不在 fallback/source root 内，不在 B1 DB 目录内。
- Level-B 不要求 temp DB；这是本包唯一放开的点。
- `report.level` 与 rollback manifest level 对 Level-B 写 `level_b_workbench_owned_state`。
- observation 必须采两份样本，`verify_sample_stability` 必须比较两次 `export_hash` 与 `projection_hash`。
- DB hash mismatch、输出路径越界、样本漂移、export hash mismatch 均必须阻断且不写 stable report。
- safety flags 继续保持 false，不把本包解释为产品全局 observation / read path 已上线。

## 5. 测试要求

按 TDD 新增 / 覆盖：

- Level-B confirmed-path 入口接受 confirmed 非 temp DB，并与 JSON fallback 匹配。
- Level-B 拒绝非 confirmed DB path。
- Level-B 拒绝 fallback / projection / rollback / report 路径落入 source root 或 DB 目录。
- Level-B DB hash 不符阻断且不写 report。
- Level-B 注入样本漂移被 `verify_sample_stability` 抓住。
- Level-A 继续拒绝非 temp DB，证明旧门未放宽。
- Env-gated ignored runner 存在但默认测试不执行。

## 6. 体积门处理

shape gate 曾发现 `workbench_sqlite_observation_period.rs` 超过新文件上限。不得通过压缩注释、合并行或删除测试绕过数字。

正确处理方式是结构性拆分测试：

- 父文件保留实现与 `#[cfg(test)] mod tests;`。
- 测试体搬到同名子模块 `src/workbench_sqlite_observation_period/tests.rs`。
- 子模块使用 `use super::*;` 访问父模块私有函数。
- 不为了测试搬运把私有函数改成 `pub` / `pub(crate)`。

## 7. 验证命令

必须运行并记录原始输出：

```bash
cargo test --lib sqlite_observation
cargo test --lib
cargo fmt -- --check
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
```

## 8. 独立复核要求

复核线只读核验，输出 `STATUS: CLEAR | CLEAR_WITH_P2 | NEEDS_FIXES`。复核重点：

- Level-A path guard 没被放宽。
- Level-B 是 explicit confirmed-path 入口，不自动发现真实 root。
- B3a 未运行真实 env runner，未读取真实 `WORKBENCH_STATE_ROOT`。
- DB path 放开 temp 限制只发生在 Level-B。
- 输出路径被限制在 confirmed work dir，且不在 fallback/source root 或 DB 目录内。
- 双样本稳定断言存在。
- 测试模块拆分是纯搬运，没有删测试、没有改可见性。
- 文件行数均低于 3000，shape gate 通过。

## 9. 停止线

- 任一验证失败且无法快速分类修复。
- Level-A 守卫被迫放宽。
- 需要读取真实 source root 或运行 env-gated runner。
- 需要改 UI、Tauri、startup、产品全局读路径或 stop-write。
- 复核线出现 P0 / P1。

到达停止线时记录 blocked / failed_classified，不把 partial 说成 completed。

## 10. 不接受为

- B3 observation 已真实执行。
- 真实 B1 DB 已被 observation runner 读取。
- observation 已接入 app startup / Tauri / UI / 产品全局读路径。
- stop-write 已执行。
- 完整存储迁移完成。
- R3 Level B 完成。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触发生。
