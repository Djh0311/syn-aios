# R3 B4a Level-B Stop-Write Decision Entry

日期：2026-06-16

状态：review_clear_pending_consultation_commit

## 目标

新增显式 Level-B confirmed-path stop-write decision 入口，使后续 B4b 能在用户在场窗口用真实 B1 DB、真实源目录、B3b projection / observation report 做只读决策验证。

本包不是 B4b，不运行真实 env runner，不执行真实停写。

## 写入范围

- 允许：`prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- 如 shape gate 因测试体积超过 3000 行失败，允许纯搬运测试模块到 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write/tests.rs`，使用 `use super::*;`，不得删测试、不得改私有可见性。

## 硬边界

- Level-A `rehearse_stop_write_decision_level_a` 的 temp / R3-A12 fixture 守卫原样保留，不为了真实 DB 放宽旧门。
- Level-B 是并列 confirmed-path 入口，DB / fallback root / projection root / observation report / work dir / stop-write report / rollback manifest 均由调用方逐项确认。
- B4 决策函数永不真停写。`approve_stop_write` 的最好结果仍是 `ready_but_not_executed`。
- 不新增任何实际停写 JSON、删除 sidecar、切产品全局写路径、改 UI / Tauri / startup 的代码路径。
- Safety flags 只允许 `stop_write_decision_recorded=true`；`stop_write_json`、`source_json_written`、`sidecar_written`、产品全局读写路径等实际动作标志必须保持 false。
- 回滚仍是 dry-run。

## TDD 行为

先写失败测试，再实现：

- Level-B confirmed-path `approve_stop_write` 在 DB / fallback / projection / observation report hash 匹配且 B1/B2b/B3b evidence 全 true 时，返回 `ready_but_not_executed`，level 为 `level_b_workbench_owned_state`，不真停写。
- 非 confirmed DB path 被拒。
- 输出路径落入 source root / DB 目录 / projection root 或不在 confirmed work dir 内被拒。
- `prepare_only` 返回 `not_ready`。
- 缺任一 Level-B evidence 标志或 hash 不符时，`approve_stop_write` 被 `preconditions_not_met` 阻断。
- Level-A 仍拒绝非 temp / R3-A12 fixture DB。

## 验证

- `cargo test --lib sqlite_stop_write`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 复核判据

独立复核线必须确认：

- Level-A 旧门未放宽。
- Level-B 仅放开 confirmed-path DB，不放开未确认路径。
- 无任何实际停写路径；决策最高只到 `ready_but_not_executed`。
- Ignored env runner 未在本包真实运行。
- 所有新增测试为 fixture/temp，未触碰真实 state root。

## 不接受为

- B4b 已执行。
- stop-write 已执行。
- JSON / sidecar 停写已完成。
- 产品全局读写路径已切换。
- R3 Level B 完成。
- 真实 Codex 执行完成。
- `/Users/yoyi/.codex` 已接触。

## 收口记录

- 实现已完成：新增并列 Level-B confirmed-path stop-write decision 入口，Level-A 入口与 temp / R3-A12 fixture 守卫保持不放宽。
- 决策边界已保持：`approve_stop_write` 最好状态为 `ready_but_not_executed`，未新增任何实际 stop-write JSON / sidecar 删除 / 产品全局读写路径切换。
- Ignored env runner 已加入但本包未真实运行：`r3_b4_stop_write_decision_confirmed_paths_requires_env_authorization`。
- 文件体积检查：`workbench_sqlite_stop_write.rs` 为 2017 行，未触发 3000 行拆测试门。
- 验证通过：`cargo test --lib sqlite_stop_write` 22 passed / 1 ignored；`cargo test --lib` 503 passed / 21 ignored；`cargo fmt -- --check` 通过；shape gate 0 errors / 0 warnings；`git diff --check` 为空。
- 独立复核：Parfit，`STATUS: CLEAR`，无 P0 / P1 / P2 / P3。书面复核见 `evidence/2026-06-16-root-treatment-r3-b4a-level-b-stop-write-decision-entry-review-parfit-v1.md`。
- 当前停止点：提交前停下，交咨询线复扫；未执行 `git add` / `git commit`。
