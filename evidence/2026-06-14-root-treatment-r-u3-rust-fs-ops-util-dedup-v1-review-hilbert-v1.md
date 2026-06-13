# Root Treatment / R-U3 Rust Fs Ops Util Dedup Review - Hilbert v1

日期：2026-06-14

状态：`STATUS: CLEAR_WITH_P2`

复核线：独立复核 agent `Hilbert`，id `019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`。

## 1. Findings

- P0：无。
- P1：无。
- P2：`evidence/2026-06-14-root-treatment-r-u3-rust-fs-ops-util-dedup-v1.md` 的验证记录原先写“在 `prototypes/productized-desktop-shell/src-tauri` 执行”，但 `node scripts/harness/workbench-shape-gate.js --mode check` 这个相对路径实际只在仓库根 `/Users/yoyi/workspace/product-line` 可复现；`src-tauri/scripts/...` 不存在。

P2 处理：已在 U3 evidence 第 6 节拆清执行目录：Rust 命令在 `src-tauri`，shape gate / git 命令在仓库根。该问题是记录 cwd 精确性问题，不影响代码行为。

## 2. 复核证据摘要

Hilbert 回交确认：

- `remove_file_if_exists` 只剩 `utils/fs_ops.rs` 一份。
- 删除语义保持：NotFound -> Ok，其他错误 -> Err。
- 错误文案前缀保持 `remove file failed {path}: {error}`。
- 8 个同形状测试 fixture helper 已归零，只剩 `utils/fs_ops.rs` 的 `#[cfg(test)] fixture_dir(stage, name)`。
- stage 映射逐项正确：importer `r3-a1`、exporter `r3-a2`、apply `r3-a2`、dual_write `r3-a3`、read_cut `r3-a4`、read_cut A10 `r3-a10`、observation_period `r3-a5`、observation_period A11 `r3-a11`。
- importer 原 `PathBuf::from(env!(...))` 到公共 helper `Path::new(env!(...))` 最终路径一致。
- Deferred 4 个 helper 保留：snapshot_apply A8、production_apply A9、stop_write A12、transaction_acceptance A13。
- `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇未迁移。
- 未发现 schema / store / JSON / 状态机 / 真实 Codex 路径相关 diff。

## 3. 复核线实际复跑

复跑通过：

- `cargo fmt -- --check`
- `git diff --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`

未复跑：

- `cargo test`

原因：复核线按只读复核避免进入测试执行 / target 写入路径；已核对主管线 evidence 中 `cargo test --lib 480 passed / 16 ignored` 与新增 4 个 `fs_ops` 测试后的计数一致，记录可信。

## 4. 边界确认

Hilbert 回交确认：

- 只读复核。
- 未修改文件。
- 未提交。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未执行真实 Codex。
- 未读取 `/Users/yoyi/.codex`。

## 5. 放行结论

代码侧 CLEAR；P2 已修正后，可由主管线进入 implementation commit / checkpoint。
