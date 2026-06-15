# Root Treatment / R3 B3a Level-B Observation Entry v1 Evidence

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`。提交前仍需咨询线复扫放行。

Planning baseline：`9edc2a7 Root Treatment / R3 B2b 受控 limited read-cut 治理账本收口`

Task package：`tasks/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`

复核记录：`evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-review-poincare-v1.md`

## 1. 本包目标

本包只做 B3a observation enablement，不执行 B3b 真实 observation。

目标是在不放宽 Level-A 旧门的前提下，为后续 B3b 用户在场窗口新增一条显式、窄口径的 Level-B confirmed-path production observation 入口。它允许后续用真实 B1 DB 作为 confirmed DB path 做受控 observation，但本包自身只用 fixture / temp 测试证明安全门。

本包不读取真实 `WORKBENCH_STATE_ROOT`，不运行真实 B3 env-gated runner，不切产品全局 read path，不停写 JSON / sidecar。

## 2. 完成内容

### 2.1 保留 Level-A 旧门

`rehearse_production_observation_level_a` 对外入口保留，继续通过 `validate_production_observation_paths` 校验。Level-A 仍只接受 temp / fixture 口径，不把真实 B1 DB 加入 allowlist。

测试继续覆盖 Level-A 非 temp DB 拒绝。

### 2.2 新增显式 Level-B confirmed-path 入口

新增：

- `SqliteObservationLevelBConfig`
- `rehearse_production_observation_level_b_workbench_owned_state`
- Level-B confirmed path validation
- confirmed DB export / sample path
- env-gated ignored runner `r3_b3_observation_confirmed_paths_requires_env_authorization`

Level-B 要求调用方显式确认：

- `confirmed_db_path`
- `confirmed_fallback_root`
- `confirmed_work_dir`
- `confirmed_projection_root`
- `confirmed_rollback_manifest_path`
- `confirmed_observation_report_path`

任何实际路径与 confirmed path 不一致均拒绝。输出路径必须在 confirmed work dir 内，且不在 fallback/source root 或 DB 目录内。

### 2.3 双样本 observation 稳定性

observation 仍采两份样本：

- sample 1
- sample 2

`verify_sample_stability` 比较两份样本的 `export_hash` 与 `projection_hash`。若出现漂移，则阻断且不写 stable report。

### 2.4 负向测试与 ignored runner

新增 / 覆盖：

- Level-B 接受 confirmed 非 temp DB，并与 fallback 匹配。
- Level-B 拒绝非 confirmed inputs。
- Level-B 拒绝输出路径越界。
- Level-B DB hash / projection / manifest / drift failure 均阻断。
- ignored env runner `r3_b3_observation_confirmed_paths_requires_env_authorization` 存在，但默认测试不运行。

### 2.5 体积门结构修正

shape gate 曾发现 `workbench_sqlite_observation_period.rs` 超过 3000 行新文件上限。本轮没有压缩注释、合并行或删除测试，而是将测试模块拆成同名子模块：

- 父文件：`prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`，1858 行。
- 子模块：`prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs`，1179 行。

父文件只保留：

```rust
#[cfg(test)]
mod tests;
```

子模块开头使用：

```rust
use super::*;
```

未为了测试搬运改动私有函数可见性。

## 3. 修改范围

代码文件：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs`

治理文件：

- `tasks/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-review-poincare-v1.md`
- `CURRENT.md`

未修改：

- 未修改 UI / CSS / app startup / Tauri command / 产品全局读写路径。
- 未新增 sidecar JSON kind。
- 未修改真实 workbench state root。
- 未读取或写入 `/Users/yoyi/.codex`。

## 4. 验证记录

### 4.1 `cargo test --lib sqlite_observation`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
warning: associated function `invalid_params` is never used
warning: `codex-governance-workbench` (lib test) generated 1 warning
running 27 tests
test workbench_sqlite_observation_period::tests::r3_b3_observation_confirmed_paths_requires_env_authorization ... ignored, requires explicit R3 B3 observation authorization and confirmed paths
test result: ok. 26 passed; 0 failed; 1 ignored; 0 measured; 490 filtered out; finished in 2.51s
```

结果：通过。

### 4.2 `cargo test --lib`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
warning: associated function `invalid_params` is never used
warning: `codex-governance-workbench` (lib test) generated 1 warning
running 517 tests
test workbench_sqlite_observation_period::tests::r3_b3_observation_confirmed_paths_requires_env_authorization ... ignored, requires explicit R3 B3 observation authorization and confirmed paths
test result: ok. 497 passed; 0 failed; 20 ignored; 0 measured; 0 filtered out; finished in 10.85s
```

结果：通过。

### 4.3 `cargo fmt -- --check`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
exit code 0
```

结果：通过。

### 4.4 Shape gate

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
Git HEAD: 9edc2a7c1eb4e93cd3c4b32ed9e2cb1256b0ee29
Tauri commands: 97 total; 0 in lib.rs
Sidecar JSON kinds: 14 detected; 0 unknown
Converged-helper dups outside utils/: 0 (12 deferred-whitelisted)
```

结果：通过。

### 4.5 `git diff --check`

执行目录：仓库根 `/Users/yoyi/workspace/product-line`

```text
exit code 0
```

输出为空。

结果：通过。

## 5. 当前 git 实物

治理文件写入前的状态为：

```text
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs
?? prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs
```

治理文件写入前行数：

```text
1858 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs
1179 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period/tests.rs
```

## 6. 独立复核

复核线：Poincare (`019ecbae-c624-7431-bbc7-633e356b4f48`)

复核结论：`STATUS: CLEAR`

P0：无。

P1：无。

P2 / P3：复核线未发现需要升级的问题。

复核记录见：

- `evidence/2026-06-15-root-treatment-r3-b3a-level-b-observation-entry-review-poincare-v1.md`

## 7. 边界确认

本包没有：

- 运行 B3 env-gated runner。
- 执行真实 B3 observation。
- 读取真实 `WORKBENCH_STATE_ROOT`。
- 修改真实 JSON / sidecar。
- 切产品全局 read path 或 observation path。
- 停写 JSON / sidecar。
- 新增 Tauri command。
- 修改 UI / CSS / app startup / 产品全局读写路径。
- 执行真实 Codex。
- 读取或写入 `/Users/yoyi/.codex`。

## 8. 不接受为

本包不接受为 B3 observation 已真实执行、observation 已接入产品全局读路径 / 界面 / Tauri / 启动、stop-write 已执行、完整存储迁移完成、R3 Level B 完成、多 agent 并行真实执行解锁、真实 Codex 执行或 `.codex` 接触。

## 9. 下一步

提交前先交咨询线复扫本 evidence、review、task package 与 `CURRENT.md` 是否如实；放行后再提交。

B3b 真实 observation 窗口需另行用户在场确认并运行 ignored env-gated runner；本包不进入 B3b。
