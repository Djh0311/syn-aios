# Root Treatment / R-U3 Rust Fs Ops Util Dedup v1

日期：2026-06-14

状态：已完成。

性质：R-U 后端 util 去重第 3 包。本包只把重复文件删除 helper 与同形状测试 fixture 路径 helper 收敛到 `src-tauri/src/utils/fs_ops.rs`；严格无行为变化。

Planning baseline：`df1fe61`。

## 0. 主管线理解

用户要求按合并正本进入 R-U3：

- 前置 U1、U2 已放行，U2 checkpoint commit 为 `df1fe61`。
- A 部分：6 个生产代码 `remove_file_if_exists(path: &Path) -> Result<(), String>` 完全同形、同文案，收敛到 `src-tauri/src/utils/fs_ops.rs`。
- B 部分：8 个同形状测试 fixture helper 收敛到 `fixture_dir(stage: &str, name: &str) -> PathBuf`，必须 `#[cfg(test)]` 门控。
- 4 个依赖 `*_fixture_root()` 或无 `name` 参数的 fixture helper 形状不同，本包不碰，记 deferred。
- 严格无行为变化，以聚焦测试和 `cargo test --lib` 全绿为铁证。
- 完成后交独立复核线 CLEAR，再 commit，再更新 checkpoint，停在 U3 复核点；不得进入 U4。

## 1. 当前扫描事实

### A. `remove_file_if_exists`

6 个生产定义均为同一函数体：

```rust
fn remove_file_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove file failed {}: {error}", path.display())),
    }
}
```

涉及文件：

- `workbench_sqlite_snapshot_apply.rs`
- `workbench_sqlite_production_apply.rs`
- `workbench_sqlite_observation_period.rs`
- `workbench_sqlite_stop_write.rs`
- `workbench_sqlite_dual_write.rs`
- `workbench_sqlite_read_cut.rs`

本包删除这 6 个本地定义，改为：

```rust
use crate::utils::fs_ops::remove_file_if_exists;
```

不保留 wrapper，理由是函数体字节级同形、报错文案无变体、外部入口不需要保持模块本地名字。

### B. `fixture_dir`

8 个同形状测试 helper 收敛到：

```rust
#[cfg(test)]
pub(crate) fn fixture_dir(stage: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(stage)
        .join(name)
}
```

允许并入映射：

- `workbench_sqlite_importer.rs::fixture_dir(name)` -> `stage = "r3-a1"`
- `workbench_sqlite_exporter.rs::fixture_dir(name)` -> `stage = "r3-a2"`
- `workbench_sqlite_apply.rs::fixture_dir(name)` -> `stage = "r3-a2"`
- `workbench_sqlite_dual_write.rs::fixture_dir(name)` -> `stage = "r3-a3"`
- `workbench_sqlite_read_cut.rs::fixture_dir(name)` -> `stage = "r3-a4"`
- `workbench_sqlite_observation_period.rs::fixture_dir(name)` -> `stage = "r3-a5"`
- `workbench_sqlite_read_cut.rs::fixture_dir_a10(name)` -> `stage = "r3-a10"`
- `workbench_sqlite_observation_period.rs::fixture_dir_r3_a11(name)` -> `stage = "r3-a11"`

注意：`workbench_sqlite_importer.rs` 当前使用 `PathBuf::from(env!("CARGO_MANIFEST_DIR"))`，其余 7 个使用 `Path::new(env!("CARGO_MANIFEST_DIR"))`；两者最终路径应保持一致，必须在验证 / 复核中单独确认。

### C. Deferred

以下 4 个 fixture helper 形状不同，本包不碰：

- `workbench_sqlite_snapshot_apply.rs::fixture_dir(name)`，委托 `manifest_r3_a8_fixture_root().join(name)`。
- `workbench_sqlite_production_apply.rs::fixture_dir(name)`，委托 `manifest_r3_a9_fixture_root().join(name)`。
- `workbench_sqlite_stop_write.rs::fixture_dir(name)`，委托 `r3_a12_fixture_root().join(name)`。
- `workbench_sqlite_transaction_acceptance.rs::fixture_dir()`，无 `name` 参数且硬编码 leaf `transaction-acceptance-core`。

Deferred 理由：强行合并会牵出 `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇，扩大到 U3 范围外，可能碰 fixture root 校验语义。

## 2. 目标

完成后：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs`。
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs` 增加 `pub(crate) mod fs_ops;`。
- 6 个生产 `remove_file_if_exists` 本地定义归零，只剩公共 helper。
- 8 个同形状测试 `fixture_dir` / `fixture_dir_a10` / `fixture_dir_r3_a11` 本地定义归零，只剩公共 `fixture_dir(stage, name)`。
- Deferred 的 4 个 fixture helper 和其 `*_fixture_root` 簇保持原地。
- 增加 fs_ops 单测：
  - `remove_file_if_exists`：NotFound -> Ok。
  - `remove_file_if_exists`：真实文件删除成功。
  - `remove_file_if_exists`：非文件路径等其他 IO 错误 -> Err，且保持 `remove file failed {path}: {error}` 前缀。
  - `fixture_dir`：断言 stage / name 拼出的尾部路径。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs`
- 6 个含重复 `remove_file_if_exists` 的 `workbench_sqlite_*` Rust 文件。
- 8 个含同形状测试 fixture helper 的 `workbench_sqlite_*` Rust 文件。
- 本任务包。
- 对应 evidence / handoff / review evidence。
- 必要 checkpoint 入口文档。

允许的代码变化仅限：

- 增加 `crate::utils::fs_ops::remove_file_if_exists` import。
- 删除 6 个本地 `remove_file_if_exists` 定义。
- 在测试模块内增加 `use crate::utils::fs_ops::fixture_dir;`。
- 将 8 个测试 helper 调用改为 `fixture_dir("<stage>", name)` 或直接调用公共 helper。
- 删除因 helper 迁移变成 unused 的 import。
- 增加公共 helper 自身单测。

## 4. 禁止范围

禁止：

- 修改 `remove_file_if_exists` 的 NotFound 吞掉、其他错误上抛行为。
- 修改 `remove_file_if_exists` 的错误文案前缀。
- 修改任一 fixture 最终路径。
- 把 `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇并入本包。
- 迁移或修改 deferred 的 4 个 fixture helper。
- 修改 store 业务逻辑、JSON / sidecar schema、workflow state schema、状态机语义。
- 修改 SQLite schema / migration / production apply / read-cut / stop-write 业务决策。
- 修改真实 Codex runner / command 参数。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 解冻 backlog。

## 5. 停止线

若抽取牵连以下任一情况，必须停止：

- 任一 `remove_file_if_exists` 调用行为或错误文案发生变化。
- 任一 fixture 最终路径发生变化，尤其 `workbench_sqlite_importer.rs` 的 `r3-a1` 路径。
- 需要移动 `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇才能继续。
- 需要修改 JSON / sidecar schema、workflow state schema、状态机或 SQLite 迁移语义。
- 需要真实 Codex 执行或读取 `/Users/yoyi/.codex` 才能验证。

发生停止时，相关 helper 留原地并在 evidence 记为 deferred，不硬合。

## 6. 验证

必须通过并在 evidence 粘贴原始尾部输出：

- `cargo fmt -- --check`
- 聚焦测试：
  - `cargo test --lib fs_ops`
  - `cargo test --lib workbench_sqlite_importer`
  - `cargo test --lib workbench_sqlite_exporter`
  - `cargo test --lib workbench_sqlite_apply`
  - `cargo test --lib workbench_sqlite_dual_write`
  - `cargo test --lib workbench_sqlite_read_cut`
  - `cargo test --lib workbench_sqlite_observation_period`
  - `cargo test --lib workbench_sqlite_snapshot_apply`
  - `cargo test --lib workbench_sqlite_production_apply`
  - `cargo test --lib workbench_sqlite_stop_write`
  - `cargo test --lib workbench_sqlite_transaction_acceptance`
- `cargo test --lib`，U2 基线为 `476 passed / 16 ignored`；新增 fs_ops 测试后应不低于 477 passed。
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

必须扫描：

- `rg -n "fn remove_file_if_exists\\(" prototypes/productized-desktop-shell/src-tauri/src`
- `rg -n "fn fixture_dir\\(|fn fixture_dir_a10\\(|fn fixture_dir_r3_a11\\(" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_*`
- `rg -n "manifest_r3_a[0-9]+_fixture_root|r3_a[0-9]+_fixture_root" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_*`
- `git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`

## 7. 复核判据

独立复核线必须确认：

- 6 个 `remove_file_if_exists` 本地定义归零，只剩 `utils/fs_ops.rs` 一份公共生产 helper。
- 8 个同形状测试 `fixture_dir` / `fixture_dir_a10` / `fixture_dir_r3_a11` 本地定义归零，只剩 `utils/fs_ops.rs` 一份 `#[cfg(test)]` 公共 helper。
- `remove_file_if_exists` 删除行为与错误文案保持。
- 8 个 fixture 最终路径逐 stage 字节一致，含 importer 的 `r3-a1`。
- Deferred 的 4 个 fixture helper 和 `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇未动。
- 未修改 store 业务逻辑、JSON / sidecar schema、workflow state schema、状态机语义。
- 未修改 SQLite schema / migration / production apply / read-cut / stop-write 业务决策。
- 未新增真实 Codex 执行路径、`.codex` 访问或 runner 参数变更。
- 验证记录可信。

## 8. 不接受为

本包不接受为：

- R-U 全部完成。
- U4 / U5 / U-Gate 完成。
- `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇收敛完成。
- store 模式合并完成。
- R3 Level B 执行。
- SQLite 真实切换。
- 真实 Codex 执行。
- backlog 解冻。

## 9. 停止点

任务包提交后进入实现；实现经独立复核 CLEAR、implementation commit 和 checkpoint commit 后，停在 U3 复核点，不得顺手进入 U4。

## 10. 复核结果

独立复核 agent `Hilbert`（`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`）回交 `STATUS: CLEAR_WITH_P2`。P0/P1 无；P2 为 evidence 验证执行目录记录不够精确，shape gate 应从仓库根执行而不是 `src-tauri`。该 P2 已在 evidence 中修正，不影响代码行为或放行。
