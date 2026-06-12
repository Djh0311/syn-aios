# 复核结论：Root Treatment / R2-T13 Deferred Inline Tests Reassessment v1

日期：2026-06-12

Reviewer：Claude（claude-opus-4-8，复核线临时代班，依据 `handoffs/2026-06-12-review-line-temporary-takeover-claude-v1.md`）

复核对象：

- 任务包：`tasks/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`
- 实现 evidence：`evidence/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`
- 结果 handoff：`handoffs/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1-result.md`

复核基线：Planning baseline `dd39c8254f9ef9ed45e0e2e641e7bd81130322e4`（= 当前 git HEAD）；工作区仅 T13 三份文档（untracked），零代码改动。

性质：只读独立复核。本包是**评估包**（无实现 diff），复核对象为 deferred 11 个 inline tests 的分类裁决；故方法为"通读测试体 + 独立依赖扫描 + 产品函数控制流核对"，而非字节对账。本结论文件是复核线唯一产出；复核线不改产品代码/任务包/evidence/权威文档，不跑 `git commit`，发现问题只列不修，不派活、不预定 T14。

---

## STATUS: CLEAR

- P0：无
- P1：无
- P2：无

deferred 11 个测试的分类裁决（可迁 9 / 禁迁 +1 / deferred 维持 1）逐条经独立验证成立。最高风险的"同组一迁一禁"判断（director review 组）经产品函数控制流核对站得住。零代码改动属实。未发现任何 P 级问题。

---

## 1. 评估包完整性（零代码改动）

独立重跑基线门：

- `cargo test --lib`：`471 passed; 0 failed; 16 ignored`——与 R2-T12 收口基线逐字一致，证明本包未动任何代码。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs: 6006 lines`、waterline 6006 未变。
- `git diff --check`：干净（exit 0）。
- `git status --short`：tracked 文件零改动；工作区仅 T13 任务包 / evidence / result handoff 三份新增文档。
- `git log`：T12 已收口入库（HEAD `dd39c82`；实现 `a3fce1f`、复核清除 `bcb8864`）；T13 baseline = HEAD，一致。

## 2. 评估对象核对（11 个测试，全部落在 4478-5044 带内）

通读 `lib.rs` 4478-5044，确认该带恰好是 T12 §6 deferred 的 11 个测试（compact 1 + governance 带 8 + director review 组 2），无遗漏、无混入；紧随其后的 4046 起 `offline_role_orchestration_*` 属 T12 既定 33 禁迁的 offline role 组，不在本包评估范围。分类与 T12 deferred 集合一一对应。

独立依赖扫描（对整文件，按行号归带）：

- runner 关键词（`StubCodexResumeRunner|CodexResumeRunner|execute_workflow_node_dispatch|Runner\b`）：4478-5044 带内仅命中 **4871 / 4877**（均在禁迁测试 `...records_completed_dispatch` 体内）；可迁的两个区段 4490-4828 与 4943-5044 **零命中**。
- temp-dir / store / fs：4490-4828 区段零命中（4676 命中为 `fn workflow_state_transition...` 函数名内 `workflow_state` 子串的假阳性，非 store 操作）；4943-5044 命中 temp_dir + `append_fixture_dispatch`（store-local，见 §3）。

## 3. 逐桶裁决验证

### 3.1 可迁 9 个——成立

- **governance 带 8 个（4490-4828）**：通读确认全部为"`json!`/struct 构造入参 → 调纯派生/边界函数 → 断言"形态，零 runner、零 store、零 temp dir：
  - `workflow_ledger_derives_*`(4491)→`derive_workflow_ledger_entries`；`subagent_report_derives_*`(4533)→`derive_subagent_reports`；`review_result_cannot_directly_complete_node`(4577)→`derive_review_results`；`workflow_exception_detects_*`(4600)→`derive_workflow_exceptions`：均纯内存 JSON/struct 入参派生。
  - `workflow_state_transition_enforces_confirmed_table`(4676)→`workflow_transition_allowed`、`workflow_node_state_transition_enforces_actor_boundaries`(4694)→`workflow_node_transition_allowed`：纯转移表、入参为字符串字面量、无任何状态。
  - `director_completion_gate_requires_evidence_review_and_no_risk`(4740)→`director_completion_gate`：纯内存 struct gate。
  - `workflow_interfaces_keep_conservative_boundaries`(4806)→`workflow_interface_boundaries`：纯只读边界 descriptor。
  - 与 T0 优先级 2"纯只读 descriptor/summary 派生"及 T11/T12 已迁切片同构，判据客观可证伪。
- **director review 拒绝路径 1 个（4943-5044）**：见 §3.2 控制流核对，store-local、零 runner，成立。

### 3.2 高风险项核对：`workflow_dispatch_director_review_rejects_invalid_state_and_dispatch`（可迁）vs 同组 `..._records_completed_dispatch`（禁迁）

用户重点要求：确认拒绝路径在触发 runner 之前返回。核对结论——**比裁决声明更干净：所调产品函数全函数无 runner。**

- 该测试只调 `record_workflow_dispatch_director_review_at`（定义于 `workflow_execution_entrypoints.rs:819-918`）。读其本体：签名仅 `(path, request)`、不带 runner 参数；本体为纯 store 操作——读状态 → schema 校验 → `validate_director_review_work_item_state`(846) / dispatch 归属校验(849) / `normalize_director_review_decision`(855) / `validate_director_review`(858) 四道 Err 门 → 追加 review+audit → `write_validated_workflow_state`(908)。**全函数（含成功路径）不构造、不调用任何 `CodexResumeRunner` / `execute_workflow_node_dispatch` / codex exec**；audit reason 与返回 message 均明示"没有发送 Codex 消息"。
- 测试三条拒绝分别命中 846（"工作项当前状态不是待回收"）、858（"派发记录不是 completed"）、855（"未知总指导回收结论"），全部在 864 行写盘前返回 Err，末尾断言 `reviews` 长度为 0。**所有拒绝路径与 runner 无任何交集**。
- 测试用 `append_fixture_dispatch`（`lib.rs:5628`）直接构造 dispatch JSON 并 `write_validated_workflow_state` 写入 store——读其本体确认零 runner、零执行链——以此绕开真实执行，制造 prepared/completed dispatch 夹具。
- 对照禁迁的同组兄弟（4831-4941）：其 runner 依赖来自 **setup**——4871 构造 `StubCodexResumeRunner`、4877 调 `execute_workflow_node_dispatch_for_index_at`（`commands.rs:1074`，签名带 `&runner`）去造一个真完成的 dispatch，再记录 review。
- 故"同组一迁一禁"非名称裁剪而是依赖画像差异：records 需要真执行的 completed dispatch（runner 端到端，禁迁正确）；rejects 只需校验门覆盖（store-local 夹具，可迁正确）。所用 helper `fixture_director_review_request`(5613) / `append_fixture_dispatch`(5628) 均在迁移带外、留 `lib.rs`，与 T11/T12 既定模式一致。

### 3.3 禁迁 +1（33→34）——成立

- `workflow_dispatch_director_review_records_completed_dispatch`（4831-4941）：体内 4871 `StubCodexResumeRunner` + 4877 `execute_workflow_node_dispatch_for_index_at` 实锤，命中 T0"runner fixture 端到端组"与代班档案 §3"共享 stub runner"。归入既定禁迁清单正确，保守方向无风险。

### 3.4 deferred 维持 1 个——成立

- `compact_last_message_summary_preserves_workflow_machine_control_marker`（4479-4488）：纯字符串函数测试，调 `compact_last_message_summary` + 断言 `workflow_machine_final_acceptance`，无 runner / store / temp dir。客观上不命中 T0 runner 判据（理论上可迁）；但断言对象是 workflow machine 控制标记语义，落入代班档案 §3"workflow machine"无限定词收紧区，口径歧义。按用户指示①"拿不准一律 deferred 不赌"维持 deferred 是保守且政策一致的安全裁决——保守留置不造成任何危害，复评待 Codex 回归按原 T0 口径重判。

## 4. 计数一致性

- deferred 11 = 可迁 9（governance 8 + director rejection 1）+ 禁迁 1 + deferred 1。
- 禁迁 33 → 34；deferred 11 → 1。本包后 44 个 inline 分类为：可迁 9（待 T14）+ 禁迁 34 + deferred 1 = 44。
- T14 若迁 9 个后剩 44 − 9 = 35 = 禁迁 34 + deferred 1，与任务包 §4 一致。本包计数无 T12 §6 式笔误。

## 5. 边界与流程确认

- 工作区改动面仅 T13 三份文档，无 `.rs/.js/.ts`、无 schema/UI/shape gate 改动，无 `~/.codex` 写入；与"评估包零代码改动、无真实 Codex、无 prompt、无 secret 读取"边界声明一致。
- T12 既定 33 禁迁未被重开（本包仅对 deferred 11 裁决）。
- T14 迁移建议与车道选择属主管线前向规划，不在本复核裁决范围；复核仅认证 deferred 11 的分类裁决成立，不endorse、不预定、不派活。

## 6. 复核边界声明

- 本文件为复核线唯一产出。未改任何产品代码/任务包/evidence/权威文档，未跑 `git commit`。
- §1 基线门为复核线本地独立重跑结果；§2/§3 依赖扫描与控制流核对均为复核线独立通读源码所得，非转述 evidence。
- 结论仅针对 R2-T13 任务包 §3-§6 的评估口径；不接受为任务包 §6 列明的任何外延（任何迁移已开始/完成、`lib.rs` 行数或 waterline 变化、R2 完成、R3 Level B、真实 Codex 执行、后续车道选择已定等）。
- 本结论将作为 Codex 额度恢复后"换脑抽查"事后复检的输入之一。
