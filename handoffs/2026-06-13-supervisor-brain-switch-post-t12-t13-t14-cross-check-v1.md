# Handoff: Supervisor Brain Switch Post T12/T13/T14 Cross Check v1

日期：2026-06-13

状态：已完成，作为 Codex 回归主管线后的“换脑抽查”事后复检留档。

复检人：Codex 主管线回归脑。

输入依据：

- `handoffs/2026-06-12-supervisor-line-rotation-protocol-v1.md`
- `handoffs/2026-06-12-supervisor-line-takeover-duty-summary-claude-v1.md`
- `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`
- T12/T13/T14 三份任务包、三份 evidence、三份 handoff result、三份 `review-claude-v1` 复核结论。

## 1. 复检结论

`STATUS: CLEAR_WITH_P2_REVIEW_NOTE`

- P0：无。
- P1：无。
- P2：1 个复核表述问题，不阻断 T12/T13/T14 的实现接受。

结论：代班期间 T12/T13/T14 三包的核心事实成立。T12/T14 为行为保持型 Rust inline tests 纯搬运，T13 为零代码 deferred 复评；当前 `lib.rs` 为 5,567 行，shape gate waterline 为 5,567，T 系列剩余 35 个 inline tests = 禁迁 34 + deferred 1。没有发现产品代码越界、真实 Codex 执行、`.codex` 读写、UI/TS/CSS/DB/schema/sidecar 变更或测试丢失证据。

## 2. 独立复跑证据

本次复检重新执行：

- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings；ratchet policy 为 `historical_lowest_closed_value`；`lib.rs: 5567/5567 (same)`。
- `cargo test --lib`：471 passed，0 failed，16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。

当前磁盘核对：

- `wc -l src-tauri/src/lib.rs`：5,567。
- `lib_task_package_preview_binding_read_model_tests.rs`：539 行，11 个 `#[test]`。
- `lib_workflow_governance_boundary_tests.rs`：339 行，8 个 `#[test]`。
- `lib_director_review_rejection_tests.rs`：102 行，1 个 `#[test]`。
- 当前四文件合计 `#[test]` 数：55；其中 `lib.rs` 剩 35 个，与 T14 handoff 声明一致。

## 3. T12 复检

T12 声明：迁移 task package preview / workflow node session binding / read model 相关 11 个 inline tests，`lib.rs` 6,544 -> 6,006，新增 539 行 include 文件，T12 后剩余 44 个 inline tests。

复检结果：

- 迁移文件存在，`include!("lib_task_package_preview_binding_read_model_tests.rs")` 位于当前 `lib.rs`。
- 当前测试守恒链条成立：T12/T13/T14 收口后，三份 include 共 20 个测试，`lib.rs` 剩 35 个测试，合计 55 个。
- T12 任务包和 evidence 明确允许 helper 定义继续留在 `lib.rs`；当前磁盘也显示 helper 定义仍在 `lib.rs`。
- 没有发现 T12 迁移触碰 K3-B guard、workflow execution runner、workflow machine、ignored real-state、cross-store memory adoption、memory candidate adoption、formal memory adoption 或共享 stub runner / factory 定义。

P2 说明：

- T12 正式复核结论 `evidence/2026-06-12-root-treatment-r2-t12-rust-task-package-preview-binding-read-model-test-extraction-v1-review-claude-v1.md` 第 3 节写“禁迁/helper 扫描 ... `mark_task_package_fixture_ready`、`append_fixture_dispatch` ... 零命中”。
- 当前磁盘复扫发现 T12 include 文件中存在这些 helper 的调用：`mark_task_package_fixture_ready` 和 `append_fixture_dispatch`。这不等于迁移 helper 定义，也不等于触碰 runner / 真实执行路径；T12 任务包本身允许 helper 留在 `lib.rs` 并由迁出测试继续调用。
- 影响分类：复核报告表述过满，属于复核结论文案 P2，不是产品代码缺陷，不阻断 T12 接受。后续复核报告应区分“helper 定义被迁移”和“迁出的测试调用既有 helper”。

## 4. T13 复检

T13 声明：对 T12 deferred 的 11 个 inline tests 做逐测试复评，裁决为可迁 9、禁迁确认 1、deferred 维持 1；零代码改动。

复检结果：

- T13 任务包、evidence、handoff result 和复核结论一致：本包为评估包，不修改产品代码，不改变 `lib.rs` 行数或 waterline。
- 可迁 9 个与 T14 实际迁移清单一致。
- 禁迁确认的 `workflow_dispatch_director_review_records_completed_dispatch` 当前仍保留在 `lib.rs`，且测试体包含 `StubCodexResumeRunner` 和 `execute_workflow_node_dispatch_for_index_at`，禁迁理由成立。
- deferred 维持的 `compact_last_message_summary_preserves_workflow_machine_control_marker` 当前仍保留在 `lib.rs`，属于后续 R2 后段收口 decision 需要按 T0 原口径复评的唯一 deferred 项。

## 5. T14 复检

T14 声明：执行 T13 可迁 9 个测试迁移，`lib.rs` 6,006 -> 5,567，新增 governance 8 tests 和 director rejection 1 test 两个 include 文件，T 系列可迁切片到底。

复检结果：

- 两个 include 文件存在，测试数量 8 + 1。
- 禁迁的 `workflow_dispatch_director_review_records_completed_dispatch` 当前仍保留在 `lib.rs`。
- deferred 的 `compact_last_message_summary_preserves_workflow_machine_control_marker` 当前仍保留在 `lib.rs`。
- `cargo test --lib workflow_dispatch_director_review` 已由 T14 evidence 记录为 2 passed；本次全量 `cargo test --lib` 也通过。
- 当前 shape gate 已锁 `lib.rs` waterline 5,567；R4-A50 的 historical-low ratchet 语义已生效。

## 6. 当前边界确认

本次复检没有：

- 修改产品代码。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / screenshot。
- 修改 UI / CSS / TS / Rust 产品路径 / DB / sidecar schema / workflow state schema。

本次复检只新增本 handoff、后续 decision / 任务包准备文档，并更新 `CURRENT.md` 脑切回 checkpoint。

## 7. 后续交接

复检通过后按用户指定顺序准备：

1. 起草 R2 后段收口 decision：按“明确下降轨道 + 冻结 deferred”口径，把 `lib.rs` 5,567 行、禁迁 34、deferred 1 和 deferred 的 T0 原口径复评写清楚。该 decision 为草案，最终车道裁决仍待用户确认。
2. 准备 R4 硬目标首批任务包：`types.ts` 分域、`WorkbenchSnapshot` 按页查询先行。
3. 把 P2-1 / R3 Level B 窗口计划文档排进任务队列，只写计划，不执行 Level B。

不接受为：

- R2 已最终完成。
- R4 硬目标已执行。
- R3 Level B 已排期或已执行。
- 多 agent 并行真实执行解锁。
- 真实 Codex 执行或 `.codex` 接触已授权。
