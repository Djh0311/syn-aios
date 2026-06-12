# Root Treatment / R2-T13 Deferred Inline Tests Reassessment v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：评估完成并通过复核线 `STATUS: CLEAR`；用户已放行 commit 序列，序列执行中；本包零代码改动。

Planning baseline commit：`dd39c8254f9ef9ed45e0e2e641e7bd81130322e4`

Task package commit：`82d30751771b77aa99d16fb21d2ef02e8eeb5ada`

Evidence commit：`f3ad6e3779b16c01643213eedc612c85014699f9`

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无。复核线另以产品函数控制流核对确认：`record_workflow_dispatch_director_review_at` 全函数（含成功路径）无 runner 构造与调用，director review 组"一迁一禁"为依赖画像差异而非名称裁剪

本文是 Root Treatment / Stage R 的 R2-T13 评估任务包，落实 R2-T12 §6 的复评触发点：对 T12 标记为 deferred 的 11 个 `lib.rs` inline tests 逐测试核对 runner fixture / store / 冻结语义依赖，写成显式裁决。

用户开包指示（2026-06-12，三点）：

1. T13 是评估包，不受"必须有降幅"约束，全部判 deferred、零迁移也是合格产出；拿不准一律 deferred 不赌。
2. 若评估结论是 T 系列可迁切片已到底，不找新活，按接管档案写代班清单收尾并停下。
3. 后续车道选择（R2 后段按"明确下降轨道 + 冻结 deferred"收口、还是转 R4 硬目标）等用户和 Codex 回归后定。

## 0. 全局主管理解

已知事实：

- R2-T12 收口后 `lib.rs` = 6,006 行，剩余 44 个 inline `#[test]`，T12 §6 分类为禁迁 33 / deferred 11。
- 禁迁 33 个为既定清单命中（K3-B guard、real-state、adoption 带、dispatch execute 带、workflow machine、offline role），本包不重新裁决。
- 本包评估对象只有 deferred 的 11 个：workflow machine marker 1 个 + workflow governance boundary 带 8 个 + director review 组 2 个。
- 评估判据沿用 R2-T0：runner fixture 依赖（命中即归 T0 暂缓组）、存储/事务语义变更风险、真实执行边界 / K3-B / Stage L/K 冻结语义纠缠、代班档案 §3 收紧清单（workflow machine 等无限定词条目从严）。

## 1. Execution Mode

Execution Mode：Supervisor-led reassessment, no product code change。

- 主管线只读扫描测试体与依赖，写本任务包、evidence、handoff。
- 不迁移任何测试，不改任何产品代码 / 测试代码 / shape gate / schema / UI。
- 复核线只读复核本评估结论。

## 2. Scope

允许：

- 读取 `lib.rs` deferred 带测试体（4478-5044 行，按 planning baseline 行号）。
- 运行只读扫描与基线验证命令（cargo test / shape gate / git diff --check）。
- 写本任务包、evidence、handoff result。
- checkpoint 同步当前入口文档（放行后随 commit 序列）。

禁止：

- 迁移任何 inline test 源码；修改任何 `.rs` / `.js` / `.ts` / UI / schema 文件。
- 重新裁决 T12 已定的 33 个禁迁测试。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- R3 Level B、解冻 backlog、开新车道。

## 3. 逐测试裁决（行号按 planning baseline `dd39c82`）

证据方法：通读全部 11 个测试体（4478-5044）；对 4490-4828（governance 带）跑 `StubCodexResumeRunner|CodexResumeRunner|runner|stub|temp_dir|fs::|bootstrap_project_workflow|execute_workflow_node_dispatch` 扫描，零命中；对 4943-5044（director review 拒绝路径）跑 runner 类扫描，零命中；4830-4941（director review 回收路径）runner 类扫描命中 2 处。

### 3.1 可迁（9 个，判据字面不命中任何禁迁/暂缓条目）

workflow governance boundary 带 8 个（4490-4828，连续区段，纯内存派生 / 边界表测试，零 runner、零 store、零 temp dir，非端到端）：

| 测试 | 行号 | 被测函数 | 形态 |
| --- | --- | --- | --- |
| `workflow_ledger_derives_summary_entries_without_tool_output_fulltext` | 4491 | `derive_workflow_ledger_entries` | 纯内存 JSON 入参派生 |
| `subagent_report_derives_required_fields_and_direction_risk` | 4533 | `derive_subagent_reports` | 纯内存 JSON 入参派生 |
| `review_result_cannot_directly_complete_node` | 4577 | `derive_review_results` | 纯内存派生；断言 review 不能直接完成节点的保守边界 |
| `workflow_exception_detects_timeout_permission_review_direction_and_harness` | 4600 | `derive_workflow_exceptions` | 纯内存 JSON / struct 入参派生 |
| `workflow_state_transition_enforces_confirmed_table` | 4676 | `workflow_transition_allowed` | 纯转移表测试 |
| `workflow_node_state_transition_enforces_actor_boundaries` | 4694 | `workflow_node_transition_allowed` | 纯转移表测试 |
| `director_completion_gate_requires_evidence_review_and_no_risk` | 4740 | `director_completion_gate` | 纯内存 struct 入参 gate 测试，非端到端、无 runner |
| `workflow_interfaces_keep_conservative_boundaries` | 4806 | `workflow_interface_boundaries` | 纯只读边界描述测试 |

director review 拒绝路径 1 个（4943-5044）：

- `workflow_dispatch_director_review_rejects_invalid_state_and_dispatch`：store-local 拒绝路径测试（temp dir + workflow state JSON + `append_fixture_dispatch`），零 runner 依赖。T0 暂缓限定词是"依赖 runner fixture 的端到端组"，本测试不命中；形态与 T11/T12 已迁并 CLEAR 的 store-local 拒绝测试同款。所用 helper `fixture_director_review_request`（5613 行）在迁移带外，按既定模式留 `lib.rs`。

裁决说明：这 9 个不是"拿不准"——runner 依赖可 grep 证伪、形态与 T0 优先级 2"纯只读 descriptor / summary 派生"及 T11/T12 已迁切片同构、不触碰任何冻结语义。判 deferred 反而是把客观判据让位给名称联想。

### 3.2 禁迁确认（1 个）

- `workflow_dispatch_director_review_records_completed_dispatch`（4831-4941）：测试体内直接构造 `StubCodexResumeRunner` 并调用 `execute_workflow_node_dispatch_for_index_at` 走 dispatch execute 链路——T0 暂缓组"依赖 runner fixture 的端到端组"与代班档案 §3"共享 stub runner"双重命中。归入既定禁迁清单（33 → 34），与 dispatch execute 带同组。

### 3.3 deferred 维持（1 个）

- `compact_last_message_summary_preserves_workflow_machine_control_marker`（4479-4488）：纯函数测试、无 runner 无 store，但断言对象是 `workflow_machine_final_acceptance` 的控制标记语义。T0 暂缓限定词（runner fixture 端到端）字面不命中，但代班档案 §3 收紧清单中"workflow machine"无限定词；该测试是否属"workflow machine 测试"存在口径歧义——按用户指示①"拿不准一律 deferred 不赌"维持 deferred。复评触发点：Codex 回归后按原 T0 口径重判，或冻结清单修订。

## 4. 结论与下一任务建议

结论：**T 系列可迁切片未到底**。

```text
R2-T14 Rust Workflow Governance Boundary And Director Review Rejection Test Extraction
```

建议边界：

- 迁移 §3.1 的 9 个 tests，分两个非连续区段：governance 带（4490-4828，339 行）与 director review 拒绝路径（4943-5044，102 行）。
- 因区段非连续，建议新增两个 include 文件各自字节级对应一个区段：`lib_workflow_governance_boundary_tests.rs`（约 339 行）与 `lib_director_review_rejection_tests.rs`（约 102 行），各在原位置替换为一行 `include!`。
- 预计 `lib.rs` 下降约 439 行（6,006 → 约 5,567），waterline 收口后锁新低。
- 共享 helper（`fixture_director_review_request` 等）照旧留 `lib.rs`；不碰 §3.2/§3.3 两个测试及其余 33 个禁迁测试。
- 验收建议：`cargo test --lib workflow_ledger`、`cargo test --lib subagent_report`、`cargo test --lib review_result`、`cargo test --lib workflow_exception`、`cargo test --lib workflow_state_transition`、`cargo test --lib workflow_node_state_transition`、`cargo test --lib director_completion_gate`、`cargo test --lib workflow_interfaces`、`cargo test --lib workflow_dispatch_director_review`、`cargo test --lib`、`cargo fmt -- --check`、shape gate、`git diff --check`、复核线只读复核。

T14 收口后 T 系列剩余：44 − 9 = 35 个 inline tests（禁迁 34 + deferred 1），届时 T 系列可迁切片到底，按接管档案 §6 写代班清单收尾；R2 后段如何收口（"明确下降轨道 + 冻结 deferred"）或转 R4 硬目标，等用户和 Codex 回归后定（用户指示③）。

## 5. 验收

- `cargo test --lib`：471 passed / 0 failed / 16 ignored（基线不变，证明零代码改动）。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors。
- `git diff --check`：通过。
- 工作区除本包三份文档外零改动。
- 复核线只读复核，结论不得有 P0/P1。

## 6. 不接受为

本任务不接受为：

- 任何 inline test 迁移已开始或完成（T14 未立项执行）。
- `lib.rs` 行数变化或 waterline 变化。
- T12 既定 33 个禁迁裁决的重开。
- R2 全部完成、R3 Level B 执行、生产 SQLite 迁移 / read-cut / stop-write。
- 多 agent 并行真实执行解锁、真实 Codex 执行。
- 产品语义变更、UI / 产品行为修改、backlog 功能解冻。
- 后续车道选择已定（属用户与 Codex 回归后决策）。
