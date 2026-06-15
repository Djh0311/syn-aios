# R3 B3a Level-B Observation Entry Review - Poincare v1

日期：2026-06-15

复核线：Poincare (`019ecbae-c624-7431-bbc7-633e356b4f48`)

任务包：`tasks/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`

实现证据：`evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`

## 1. 复核结论

状态：`STATUS: CLEAR`

P0：无。

P1：无。

P2 / P3：复核线未发现需要升级的 P2 / P3。

## 2. 复核范围

复核范围为：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs`

复核线只读核验，未修改文件、未提交、未运行 ignored env-gated runner、未读取真实 `WORKBENCH_STATE_ROOT`、未访问 `/Users/yoyi/.codex`、未执行真实 Codex。

## 3. 关键核验点

- Level-A 旧门未放宽：`rehearse_production_observation_level_a` 仍走 `validate_production_observation_paths`，`validate_temp_db_path` 仍拒绝非 temp DB。
- Level-B 是显式 confirmed-path 入口：新增 config 携带 confirmed `db / fallback / work / projection / rollback / report` 路径，校验 canonical 后逐项匹配，并阻止输出落入 fallback root 或 DB 目录。
- 双样本稳定断言存在：实现采样两次，`verify_sample_stability` 同时比较 `export_hash` 和 `projection_hash`；测试也断言两次 hash 相等。
- `r3_b3_observation_confirmed_paths_requires_env_authorization` 仍为 `#[ignore]`，普通测试不会运行真实 env runner。
- 体积门处理是结构性测试子模块拆分：父文件末尾为 `#[cfg(test)] mod tests;`，子模块以 `use super::*;` 访问父模块私有项。
- 未为了测试搬运把私有函数改成 `pub` / `pub(crate)`；复核未发现可见性放宽。
- 父文件 1858 行，子模块 1179 行，均低于 3000。

## 4. 复核线只读证据

Poincare 复核线记录：

```text
STATUS: CLEAR
No P0/P1 found in the reviewed scope. I also did not find any P2/P3 issues that would change the conclusion.
```

关键证据摘录：

```text
Level-A old gate is still intact: rehearse_production_observation_level_a still routes through validate_production_observation_paths, and validate_temp_db_path still rejects non-temp DBs with temp_db_path_required.
Level-B is an explicit confirmed-path entry: the new config carries confirmed db/fallback/work/projection/rollback/report paths, validation canonicalizes and matches them, and output paths are blocked from the fallback root and DB dir.
Sample stability is still double-checked: the implementation samples twice and verify_sample_stability compares both export_hash and projection_hash; the tests also assert both hashes are equal.
r3_b3_observation_confirmed_paths_requires_env_authorization remains #[ignore].
The shape-gate fix is structural and complete: the parent file now ends with #[cfg(test)] mod tests;, the child starts with use super::*;, and both files are under 3000 lines.
```

复核线只读命令范围：`git diff`、`rg`、`sed`、`tail`、`nl`、`wc -l`。

## 5. 边界

本复核不接受为 B3 observation 已真实执行、产品全局 read path / observation path 已切换、stop-write 已执行、R3 Level B 完成、多 agent 并行真实执行解锁、真实 Codex 执行或 `/Users/yoyi/.codex` 接触。
