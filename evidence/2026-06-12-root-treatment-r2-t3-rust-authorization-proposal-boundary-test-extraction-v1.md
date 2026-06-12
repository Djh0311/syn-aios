# Evidence: Root Treatment / R2-T3 Rust Authorization Proposal Boundary Test Extraction v1

日期：2026-06-12

状态：已完成，hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t3-rust-authorization-proposal-boundary-test-extraction-v1.md`

Planning baseline commit：`56e59569abbe7e3e160ac4f6229ef5ed1525649d`

Implementation commit：`e428c98c5e04f24282d5ae10cdb46d20b850e588`

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`bda6c096ec1ad6c5f653d3eb06ec778bf1fd78dc`

## 1. 本轮目标

承接新策略调整、R4-A50 shape gate 硬化、R2-T0 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决、R2-T1 / R2-T2 inline tests 迁移结果，迁移第三批低风险 Rust inline tests：C1-C3 方案授权、项目咨询方案、全局边界复核测试。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_authorization_proposal_boundary_tests.rs`
- `tasks/2026-06-12-root-treatment-r2-t3-rust-authorization-proposal-boundary-test-extraction-v1.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- K3-B runtime prompt guard 测试。
- `reads_real_static_index_summary`。
- C4 项目主管拆任务、prepared dispatch、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption、共享 stub runner / factory。
- Tauri command、DB schema、sidecar schema、workflow state JSON。
- 前端 UI / CSS / TS。
- 真实执行路径。

## 3. 迁移内容

从 `lib.rs` 的 `#[cfg(test)] mod tests` 中迁出 18 个测试：

- 方案授权 guard：8 个。
- 项目咨询方案 proposal：5 个。
- 全局边界复核：5 个。

迁移方式：

- 新增 `lib_authorization_proposal_boundary_tests.rs`，仍由 `#[cfg(test)] mod tests` 内 `include!("lib_authorization_proposal_boundary_tests.rs");` 引入。
- 测试仍处于同一个 crate-root test module 中，不新增 public API，不扩大生产函数可见性。
- 共享 fixture / helper 保留在 `lib.rs`，降低迁移风险。
- 未迁移 C4 `project_director_task_plan_rejects_without_active_c3_authorization` 及其后续测试。

## 4. 棘轮结果

行数：

```text
lib.rs: 12,699 -> 12,019
lib_authorization_proposal_boundary_tests.rs: 674
```

shape gate 水位线：

```text
prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 12,019
```

本轮将 R2-T3 的收益写入 `workbench-shape-gate.js`，使 `lib.rs` 新历史低点成为后续防回涨基线。

## 5. 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

```text
cargo test --lib plan_authorization
```

结果：通过，8 passed，479 filtered out。

```text
cargo test --lib project_consultation_proposal
```

结果：通过，5 passed，482 filtered out。

```text
cargo test --lib global_boundary_review
```

结果：通过，5 passed，482 filtered out。

```text
cargo test --lib workflow_authorization
```

结果：通过，1 passed，486 filtered out。

```text
cargo fmt -- --check
```

结果：通过。

```text
cargo test --lib
```

结果：通过，471 passed，16 ignored。

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，0 errors，0 warnings；`Ratchet policy: historical_lowest_closed_value`；`lib.rs: 12019/12019 (same)`。

```text
git diff --check
```

结果：通过，无输出。

既有 warning：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` never used。

说明：

- 本轮未运行 npm / build，因为未改前端产品代码。
- focused tests 中并行 cargo 曾出现短暂 `Blocking waiting for file lock on artifact directory`，随后正常通过；未产生失败或残留。

收尾过程偏差：

- 复查 `TBD` / review 状态残留时，第一次误把 Markdown 反引号放在 shell 双引号内，zsh 尝试执行 `TBD` 并返回 `command not found: TBD`；未触发真实 Codex、未读写 `/Users/yoyi/.codex`、未改文件。随后已用单引号安全重跑扫描，确认无“待复核”或 review `TBD` 残留；剩余 `TBD` 仅为 implementation / checkpoint hash 正常占位。

## 6. 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮不接受为：

- `lib.rs <= 3,000` 达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write 或 rollback production workflow。
- 多 agent 并行真实执行解锁。
- C4 项目主管拆任务迁移完成。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 7. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

结论：`STATUS: CLEAR`

- P0：无。
- P1：无。
- P2：无。
- Diff 范围符合 R2-T3：`lib.rs`、`workbench-shape-gate.js`、新增 test include，以及 task/evidence/handoff 三份文档。
- `lib.rs` 只在原位置保留 `include!("lib_authorization_proposal_boundary_tests.rs");`；C4 `project_director_task_plan_rejects_without_active_c3_authorization` 仍在 `lib.rs`。
- K3-B guard 和 `reads_real_static_index_summary` 仍在 `lib.rs`。
- 新增 `lib_authorization_proposal_boundary_tests.rs` 含 18 个 `#[test]`，函数名覆盖 C1-C3 plan authorization / project consultation proposal / global boundary review。
- 新 include 文件未发现 `std::process`、网络、真实 Codex、`.codex`、Tauri command、secret/env/keychain/OAuth/full transcript 访问。
- 新 include 文件中的文件写入仅限原测试语义的 temp fixture sidecar。
- Shape gate 将 `lib.rs` waterline 更新为 `12019`；当前 `wc -l lib.rs` 也是 `12019`。
- `git diff --check` 无输出。

Residual risk：只读复核未重跑 cargo/node 验证；结论基于主管线已报告通过结果，加复核线静态 `git diff` / `rg` / `wc` / `git diff --check` 检查。
