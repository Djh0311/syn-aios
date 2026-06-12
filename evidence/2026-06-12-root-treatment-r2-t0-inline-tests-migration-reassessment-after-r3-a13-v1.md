# Evidence: Root Treatment / R2-T0 Inline Tests Migration Reassessment After R3-A13 v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t0-inline-tests-migration-reassessment-after-r3-a13-v1.md`

Planning baseline commit：`329b2d9bda1adcd6b67356a6fe752d8cca472817`

Implementation commit：待回填。

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填。

## 1. 本轮目标

落实 R4-A50 后的新策略要求，重开 R2 后段评估：以 R3-A13 Level A 完成为输入，重新判定 `lib.rs` inline tests 迁移是否解锁，并输出显式决定、可迁范围、暂缓范围和下一任务建议。

## 2. 读取依据

读取并使用：

- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md`
- `evidence/2026-06-11-root-treatment-r2-closing-r3-preflight-review-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a13-transaction-acceptance-and-cutover-gap-matrix-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

关键依据：

- R2 closing 记录：`lib.rs` inline tests 位于 `1703-13949`，约 12,247 行，静态统计 213 个 `#[test]`。
- R2 closing 建议：单独 `R2-T1 inline tests migration`，按领域分批迁到 helper / module-local tests，共享 fixtures 先抽 `test_support`。
- R3-A13 记录：fixture / temp SQLite transaction acceptance 已完成，memory candidate adoption + formal memory + memory audit + workflow audit 可同事务提交；Level B 未执行。
- A50 策略：后续任务必须降低棘轮指标，不降低指标的 helper 拆分不得立项。

## 3. 当前态扫描

工作树：

- `git status --short`：无输出。

当前行数：

```text
lib.rs: 13,965
offline-permission-dialog.test.tsx: 3,404
```

当前 inline tests：

```text
rg -c "#\\[test\\]" src-tauri/src/lib.rs
213
```

当前结构：

- `#[cfg(test)] mod tests` 仍从 `lib.rs:1720` 开始。
- transcript / readback 测试集中在 `2520-2894`。
- shared fixture helpers 从 `2914` 起集中出现。
- workflow / memory / task package / dispatch / runner fixture 仍大量共享 `test_temp_dir`、`fixture_project`、`fixture_task_draft_request`、`StubCodexResumeRunner` 等底座。
- `13216` 的 `real_task_package_file_generation_confirmation_v1` 仍是 ignored real-state confirmation test，不应在治理期顺手迁移或重新激活。

## 4. 验证命令

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，0 errors，0 warnings；`Ratchet policy: historical_lowest_closed_value`。

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

```text
cargo test --lib sqlite_transaction_acceptance
```

结果：通过，5 passed。

```text
cargo test --lib workflow_state
```

结果：通过，11 passed。

```text
cargo test --lib
```

结果：通过，471 passed，16 ignored。

说明：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` never used。
- 本轮没有运行 `cargo fmt -- --check`，因为未改 Rust 源码。
- 本轮没有运行 npm / build，因为未改前端或产品代码。

## 5. Reassessment Decision

结论：

```text
PARTIALLY_UNLOCKED_WITH_GUARDS
```

理由：

- R3-A13 已经解除“跨 store transaction 完全未验证”的一部分前置风险，足以支持低风险测试迁移专项启动。
- 但 R3-A13 只是 Level A；生产 DB、production read-cut、stop-write JSON / sidecar 和真实 workbench state root 均未执行。
- `lib.rs` 内仍存在共享 fixture / stub runner 底座，尤其 `StubCodexResumeRunner`、`WorkflowMachineStubRunner`、`FailingCodexResumeRunner` 支撑多个执行域，不能直接整段搬迁。
- 所以 inline tests 迁移不能无限期挂起，但也不能全量解锁。

## 6. 可迁范围

优先迁：

1. transcript catalog / dispatch readback stats：`2520-2894`。
2. diagnostics / provider / session continuation / adapter boundary read-model tests 中不涉及 K3-B runtime prompt guard 的子集。
3. workflow_state store-local / lifecycle tests 中不改 JSON shape 的子集。
4. memory lint / maintenance / mature pattern store-local / preview tests。

推荐第一包：

```text
R2-T1 Rust Inline Transcript / Readback Test Extraction
```

建议第一包只迁 transcript / readback tests 和局部 fixture，通过 crate-root test include 过渡，不改产品函数可见性。

## 7. 暂缓范围

暂缓：

- memory candidate adoption 跨 candidate + formal store 端到端组。
- workflow node dispatch execute / readback / user reviewed instruction / failure / timeout 组。
- workflow machine / director review / offline role 中依赖 runner fixture 的端到端组。
- ignored real-state confirmation test。
- 共享 stub runner / fixture factories 的单独迁移。

暂缓不是无限期挂起；触发下一次复评的条件：

- R2-T1 / T2 完成 test support 和 transcript/readback 低风险迁移后。
- R3 Level B window plan 完成后。
- 任何迁移包发现必须扩大生产可见性或改变测试语义时。

## 8. 边界确认

本轮没有修改产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema 或真实执行路径。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot，没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮不接受为：

- inline tests 迁移已开始或完成。
- `lib.rs <= 3,000` 已达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write、rollback production workflow 或多 agent 并行真实执行解锁。
- UI / 产品行为修改或 backlog 功能解冻。

## 9. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

结论：`STATUS: CLEAR`

复核摘要：

- P0：无。
- P1：无。
- P2：无。
- 工作树范围符合预期：只有三份 R2-T0 文档未跟踪，tracked diff 为空。
- 复核确认复评结论由 R2 closing / R3-A13 / A50 证据支撑。
- 复核确认未把 R3-A13 Level A 冒充为 R3 全量完成、production DB/read-cut/stop-write 解锁或多 agent 并行真实执行解锁。
- 复核确认推荐 R2-T1 符合 A50 棘轮收益规则，暂缓范围保持 real execution / stub runner / ignored real-state 测试冻结。

Residual risk：本复核未重跑主管线的 cargo/node 验证，只做静态只读审查；R2-T1 实施时仍需严控不扩大产品函数可见性、不改变测试断言语义。
