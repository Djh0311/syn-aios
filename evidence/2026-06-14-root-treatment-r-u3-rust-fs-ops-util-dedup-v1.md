# Root Treatment / R-U3 Rust Fs Ops Util Dedup Evidence v1

日期：2026-06-14

状态：已完成。

## 1. 实现摘要

本包只做 Rust 后端 fs ops helper 去重：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs`。
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs` 增加 `pub(crate) mod fs_ops;`。
- 6 个生产代码 `remove_file_if_exists(path: &Path) -> Result<(), String>` 本地定义删除，改为直接 import `crate::utils::fs_ops::remove_file_if_exists`。
- 8 个同形状测试 fixture helper 改用 `crate::utils::fs_ops::fixture_dir(stage, name)`。
- `fixture_dir(stage, name)` 使用 `#[cfg(test)]` 门控，仅测试构建可见。
- Deferred 的 4 个 fixture helper 和 `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇保持原地。

本包没有修改 SQLite schema / migration / production apply / read-cut / stop-write 的业务决策；但确实修改了 6 个 `workbench_sqlite_*` 模块中的生产删除 helper 引用来源，以及 6 个 `workbench_sqlite_*` 模块中的测试 fixture helper 调用方式。

## 2. 文件变化

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`

精确边界：

- `workbench_sqlite_snapshot_apply.rs` / `production_apply.rs` / `observation_period.rs` / `stop_write.rs` / `dual_write.rs` / `read_cut.rs`：删除本地 `remove_file_if_exists` 函数体，增加公共 helper import；不改调用点业务顺序。
- `workbench_sqlite_importer.rs` / `exporter.rs` / `apply.rs` / `dual_write.rs` / `read_cut.rs` / `observation_period.rs`：测试模块改用 `fixture_dir(stage, name)`；不改测试断言。
- Deferred helper 所在的 `snapshot_apply.rs` / `production_apply.rs` / `stop_write.rs` / `transaction_acceptance.rs` 仍保留各自 `fixture_dir`。

## 3. Helper 扫描

扫描命令：

```text
rg -n "fn remove_file_if_exists\\(" prototypes/productized-desktop-shell/src-tauri/src
```

原始输出：

```text
prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs:7:pub(crate) fn remove_file_if_exists(path: &Path) -> Result<(), String> {
```

扫描命令：

```text
rg -n "fn fixture_dir\\(|fn fixture_dir_a10\\(|fn fixture_dir_r3_a11\\(" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_* prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs
```

原始输出：

```text
prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs:1166:    fn fixture_dir(name: &str) -> PathBuf {
prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs:1220:    fn fixture_dir(name: &str) -> PathBuf {
prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs:1320:    fn fixture_dir(name: &str) -> PathBuf {
prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs:16:pub(crate) fn fixture_dir(stage: &str, name: &str) -> PathBuf {
prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_transaction_acceptance.rs:634:    fn fixture_dir() -> PathBuf {
```

解释：剩余 4 个 `workbench_sqlite_*` fixture helper 均为任务包 deferred 项；8 个同形状 helper 已归零。

## 4. Fixture Stage 映射

本包实际映射：

- `workbench_sqlite_importer.rs` -> `fixture_dir("r3-a1", name)`；原 `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join("r3-a1").join(name)` 与新公共 helper 最终路径一致。
- `workbench_sqlite_exporter.rs` -> `fixture_dir("r3-a2", name)`。
- `workbench_sqlite_apply.rs` -> `fixture_dir("r3-a2", name)`。
- `workbench_sqlite_dual_write.rs` -> `fixture_dir("r3-a3", name)`。
- `workbench_sqlite_read_cut.rs` 原 `fixture_dir(name)` -> `fixture_dir("r3-a4", name)`。
- `workbench_sqlite_read_cut.rs` 原 `fixture_dir_a10(name)` -> `fixture_dir("r3-a10", name)`。
- `workbench_sqlite_observation_period.rs` 原 `fixture_dir(name)` -> `fixture_dir("r3-a5", name)`。
- `workbench_sqlite_observation_period.rs` 原 `fixture_dir_r3_a11(name)` -> `fixture_dir("r3-a11", name)`。

Deferred 保留：

- `workbench_sqlite_snapshot_apply.rs::fixture_dir(name)` -> `manifest_r3_a8_fixture_root().join(name)`。
- `workbench_sqlite_production_apply.rs::fixture_dir(name)` -> `manifest_r3_a9_fixture_root().join(name)`。
- `workbench_sqlite_stop_write.rs::fixture_dir(name)` -> `r3_a12_fixture_root().join(name)`。
- `workbench_sqlite_transaction_acceptance.rs::fixture_dir()` -> `manifest_r3_a13_fixture_root().join("transaction-acceptance-core")`。

## 5. 禁止路径核对

命令：

```text
git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs
```

原始输出：无输出。

`manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 扫描仍显示既有 root 簇留在原文件；本包未迁移 root 簇。

## 6. 验证记录

Rust 命令在 `prototypes/productized-desktop-shell/src-tauri` 执行；仓库门禁命令在 `/Users/yoyi/workspace/product-line` 执行。

### cargo fmt

命令：

```text
cargo fmt -- --check
```

原始输出：无输出，exit code 0。

### 聚焦测试

原始尾部输出：

```text
cargo test --lib fs_ops
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 492 filtered out; finished in 0.00s

cargo test --lib workbench_sqlite_importer
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.01s

cargo test --lib workbench_sqlite_exporter
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 493 filtered out; finished in 0.04s

cargo test --lib workbench_sqlite_apply
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 490 filtered out; finished in 0.11s

cargo test --lib workbench_sqlite_dual_write
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 486 filtered out; finished in 0.19s

cargo test --lib workbench_sqlite_read_cut
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 470 filtered out; finished in 0.43s

cargo test --lib workbench_sqlite_observation_period
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 472 filtered out; finished in 1.27s

cargo test --lib workbench_sqlite_snapshot_apply
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 483 filtered out; finished in 0.22s

cargo test --lib workbench_sqlite_production_apply
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 484 filtered out; finished in 0.22s

cargo test --lib workbench_sqlite_stop_write
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 480 filtered out; finished in 0.36s

cargo test --lib workbench_sqlite_transaction_acceptance
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 491 filtered out; finished in 0.16s
```

说明：以上 Rust 测试均保留既有 warning：

```text
warning: associated function `invalid_params` is never used
```

### cargo test --lib

命令：

```text
cargo test --lib
```

原始尾部输出：

```text
test result: ok. 480 passed; 0 failed; 16 ignored; 0 measured; 0 filtered out; finished in 7.09s
```

### shape gate

执行目录：`/Users/yoyi/workspace/product-line`。

命令：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

原始关键输出：

```text
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 0d23ddf72544663a7b6b6b65e260f2251f42d238
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- Tauri commands: 97 total; 0 in lib.rs
- Sidecar JSON kinds: 14 detected; 0 unknown
```

### git diff --check

执行目录：`/Users/yoyi/workspace/product-line`。

命令：

```text
git diff --check
```

原始输出：无输出，exit code 0。

## 7. 当前 git 实物

`git log --oneline -6` 原始输出：

```text
0d23ddf docs: add r-u3 fs ops util dedup package
df1fe61 docs: checkpoint r-u2 sidecar path util dedup
1ba8f01 refactor: deduplicate rust sidecar path helpers
ef86990 docs: add r-u2 sidecar path util dedup package
6fca242 docs: checkpoint r-u1 hash util dedup
e6325e8 refactor: deduplicate rust hash helpers
```

`git status --short` 原始输出：

```text
 M prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs
?? prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs
```

## 8. 边界确认

本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，未启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本包不接受为 R-U 全部完成、U4/U5/U-Gate 完成、fixture root 簇收敛完成、store 模式合并完成、R3 Level B、SQLite 真实切换、真实 Codex 执行或 backlog 解冻。

## 9. 独立复核结果

独立复核 agent `Hilbert`（`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`）回交 `STATUS: CLEAR_WITH_P2`。

- P0：无。
- P1：无。
- P2：本 evidence 原先写“在 `prototypes/productized-desktop-shell/src-tauri` 执行”，但 `node scripts/harness/workbench-shape-gate.js --mode check` 相对路径实际应在仓库根 `/Users/yoyi/workspace/product-line` 执行；复核线已在仓库根复跑 shape gate，结果 pass，关键指标一致。

P2 处理：已将第 6 节验证记录修正为 Rust 命令在 `src-tauri` 执行、shape gate / git 命令在仓库根执行。该 P2 为记录 cwd 精确性问题，不影响代码行为或放行。
