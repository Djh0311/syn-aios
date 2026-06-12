# Root Treatment / R2-T3 Rust Authorization Proposal Boundary Test Extraction v1

日期：2026-06-12

状态：复核通过，hash 待回填。

Planning baseline commit：`56e59569abbe7e3e160ac4f6229ef5ed1525649d`

Implementation commit：`TBD`

Review result：`CLEAR`，P0/P1/P2 无；复核线程继续复用 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`TBD`

本文是 Root Treatment / Stage R 的 R2-T3 任务包，承接新策略调整、R4-A50 shape gate 硬化、R2-T0 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决、R2-T1 / R2-T2 inline tests 迁移结果，继续迁移低风险 Rust inline tests，并把 `lib.rs` 历史最低水位线继续下压。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 C1-C3 方案授权、项目咨询方案、全局边界复核相关 inline tests 迁出到 crate-root test include 文件，降低 `lib.rs` 棘轮指标。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_authorization_proposal_boundary_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

## 2. 允许范围

允许：

- 迁移以下 `lib.rs` inline tests：
  - `workflow_authorization_plan_authorization_guard_blocks_without_authorization`
  - `plan_authorization_guard_needs_review_before_user_and_global_approval`
  - `plan_authorization_guard_authorizes_matching_active_scope`
  - `plan_authorization_guard_blocks_write_scope_escape`
  - `plan_authorization_guard_blocks_role_and_agent_escape`
  - `plan_authorization_guard_blocks_revoked_paused_and_expired`
  - `plan_authorization_guard_needs_review_when_stop_condition_requires_user`
  - `plan_authorization_inspect_writes_auto_dispatch_scope_checked_audit`
  - `project_consultation_proposal_create_writes_revision_and_read_model`
  - `project_consultation_proposal_rejects_missing_required_fields`
  - `project_consultation_proposal_confirm_creates_user_confirmed_authorization_not_active`
  - `project_consultation_proposal_request_changes_and_reject_do_not_create_authorization`
  - `project_consultation_proposal_rejects_repeated_confirmation`
  - `global_boundary_review_approved_activates_authorization_and_guard_still_checks_scope`
  - `global_boundary_review_rejects_missing_user_confirmation`
  - `global_boundary_review_rejects_proposal_authorization_mismatch`
  - `global_boundary_review_rejects_incomplete_checklist_and_blocking_finding_for_approved`
  - `global_boundary_review_needs_changes_and_blocked_do_not_activate_authorization`
- 在 `#[cfg(test)] mod tests` 内新增 `include!("lib_authorization_proposal_boundary_tests.rs");`。
- 保持测试仍在同一个 crate-root `tests` module 中，避免扩大生产函数可见性。
- 共享 fixture / helper 默认留在 `lib.rs`，除非编译要求必须跟随迁移；如必须迁移，也只能迁移纯测试 fixture，不得迁移产品逻辑。
- 更新 shape gate 中 `lib.rs` waterline 到迁移后的实际行数。

禁止：

- 迁移 K3-B runtime prompt guard 测试。
- 迁移 `reads_real_static_index_summary`。
- 迁移 C4 项目主管拆任务、prepared dispatch、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- 修改产品函数签名、可见性或行为。
- 新增 public API。
- 修改测试断言含义。
- 修改 DB schema、sidecar schema、workflow state JSON、Tauri command、UI、CSS、前端代码或真实执行路径。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 3. 形状影响

预计影响：

- `lib.rs` 预计下降约 650-750 行。
- 新增测试 include 文件预计 650-750 行，低于 Rust 新文件 3,000 行上限。
- 不新增 Tauri command。
- 不新增 sidecar JSON 种类。
- 不新增生产模块，只新增测试 include 文件。

棘轮目标：

```text
prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 12,699 -> 新历史低点
```

本任务符合新策略要求：它必须降低 `lib.rs` 棘轮指标，不属于零下降 helper 拆分包。

## 4. Implementation Plan

1. 新增 `lib_authorization_proposal_boundary_tests.rs`，把目标测试原样移入。
2. 在 `lib.rs` 原位置保留一个 crate-root test include。
3. 更新 `workbench-shape-gate.js` 的 `lib.rs` waterline 到迁移后的实际行数。
4. 运行 focused cargo tests、全量 cargo lib、format check、shape gate 和 diff check。
5. 写 evidence / handoff。
6. 交给既有复核线只读审查。
7. 复核 `CLEAR` 或仅 P2 后提交 implementation commit，再做 checkpoint 和 hash backfill。

## 5. 验收标准

必须满足：

- `lib.rs` 行数下降，且 shape gate 通过。
- 新测试文件低于 Rust 3,000 行阈值。
- focused tests 通过：
  - `cargo test --lib plan_authorization`
  - `cargo test --lib project_consultation_proposal`
  - `cargo test --lib global_boundary_review`
  - `cargo test --lib workflow_authorization`
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode check` 通过。
- `git diff --check` 通过。
- 复核线 `STATUS: CLEAR` 或仅 P2。

## 6. 不接受为

本任务不接受为：

- `lib.rs <= 3,000` 达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write 或 rollback production workflow。
- 多 agent 并行真实执行解锁。
- C4 项目主管拆任务迁移完成。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 7. Review Result

只读复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 结论：`STATUS: CLEAR`。

复核结论：

- P0：无。
- P1：无。
- P2：无。
- 复核确认 diff 范围符合 R2-T3：`lib.rs`、`workbench-shape-gate.js`、新增 test include，以及 task/evidence/handoff 三份文档。
- 复核确认 `lib.rs` 原位置只保留 `include!("lib_authorization_proposal_boundary_tests.rs");`，C4 `project_director_task_plan_rejects_without_active_c3_authorization` 仍留在 `lib.rs`。
- 复核确认 K3-B guard 和 `reads_real_static_index_summary` 仍留在 `lib.rs`。
- 复核确认新增 `lib_authorization_proposal_boundary_tests.rs` 含 18 个 `#[test]`，函数名覆盖 C1-C3 plan authorization / project consultation proposal / global boundary review。
- 复核确认新 include 文件未发现 `std::process`、网络、真实 Codex、`.codex`、Tauri command、secret/env/keychain/OAuth/full transcript 访问。
- 复核确认新 include 文件中的文件写入仅限原测试语义的 temp fixture sidecar。
- 复核确认 shape gate `lib.rs` waterline 更新为 `12019`，当前 `wc -l lib.rs` 也是 `12019`。
- `git diff --check` 无输出。
