# Handoff: Root Treatment / R2-T0 Inline Tests Migration Reassessment After R3-A13 v1 Result

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t0-inline-tests-migration-reassessment-after-r3-a13-v1.md`

Evidence：`evidence/2026-06-12-root-treatment-r2-t0-inline-tests-migration-reassessment-after-r3-a13-v1.md`

Planning baseline commit：`329b2d9bda1adcd6b67356a6fe752d8cca472817`

Implementation commit：待回填。

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填。

## 1. 完成内容

完成 R2 后段 inline tests 迁移复评。

裁决：

```text
PARTIALLY_UNLOCKED_WITH_GUARDS
```

含义：

- 不允许继续无限期默认挂起 inline tests 迁移。
- 可以启动 R2-T1 inline tests migration。
- 只能先迁低耦合、能降低 `lib.rs` 棘轮指标的测试切片。
- 不允许全量搬迁、不允许夹带存储语义改动、不允许触碰真实执行冻结语义。

## 2. 当前基线

当前 `lib.rs`：

- 13,965 行。
- `#[cfg(test)] mod tests` 从 `1720` 行开始。
- `#[test]` 静态统计：213。

R3-A13 影响：

- 已部分解除跨 store transaction 未验证的问题。
- 未解除 production DB / read-cut / stop-write / real state root 未执行的问题。
- 未解除共享 Rust fixture / stub runner 底座未拆的问题。

## 3. 推荐下一任务

下一任务建议：

```text
R2-T1 Rust Inline Transcript / Readback Test Extraction
```

边界：

- 迁 `lib.rs` 中 transcript catalog / dispatch readback stats 相关测试和局部 fixture。
- 推荐先新增 `src-tauri/src/lib_transcript_readback_tests.rs`，由 crate-root `#[cfg(test)] mod tests` 内 `include!` 引入。
- 不改产品函数签名，不新增 public API，不改测试断言含义。
- 不迁 cross-store memory adoption、workflow execution runner、workflow machine、ignored real-state test。

验收：

- `lib.rs` 行数下降约 350-500 行。
- 新测试文件低于 Rust 3,000 行阈值。
- `cargo test --lib transcript`
- `cargo test --lib dispatch_readback_stats`
- `cargo test --lib`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 4. 验证结果

通过：

- `node scripts/harness/workbench-shape-gate.js --mode check`：0 errors，0 warnings。
- `cargo test --lib sqlite_transaction_acceptance`：5 passed。
- `cargo test --lib workflow_state`：11 passed。
- `cargo test --lib`：471 passed，16 ignored。

说明：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` never used。
- 未运行 `cargo fmt -- --check`、npm、build，因为本轮未改 Rust/前端产品源码。

## 5. 边界确认

本轮没有修改产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema 或真实执行路径。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot，没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮不接受为 inline tests 迁移已开始或完成、`lib.rs <= 3,000`、R2 全部完成、R3 Level B 执行、生产 SQLite 迁移、read-cut、stop-write、多 agent 并行真实执行解锁、UI / 产品行为修改或 backlog 功能解冻。

## 6. 复核回收

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

结论：`STATUS: CLEAR`

- P0：无。
- P1：无。
- P2：无。
- 复核确认 R2-T0 只有三份文档变更，没有产品代码、UI/CSS、Rust/Tauri 产品路径、DB/schema、真实执行、`.codex` 或 secret 越界。
- 复核确认 `PARTIALLY_UNLOCKED_WITH_GUARDS` 没有越过 R3-A13 Level A 和 A50 棘轮收益边界。
- 复核残余风险：R2-T1 真正实施时仍需严控不扩大产品函数可见性、不改变测试断言语义。
