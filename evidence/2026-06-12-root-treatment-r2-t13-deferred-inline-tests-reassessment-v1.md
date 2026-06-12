# Evidence: Root Treatment / R2-T13 Deferred Inline Tests Reassessment v1

日期：2026-06-12

Supervisor：Claude（claude-fable-5，临时代班，依据 `handoffs/2026-06-12-supervisor-line-temporary-takeover-codex-to-claude-v1.md`）

状态：已完成并复核通过，checkpoint 已同步；本包零代码改动。

任务包：`tasks/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`

Planning baseline commit：`dd39c8254f9ef9ed45e0e2e641e7bd81130322e4`

Task package commit：`82d30751771b77aa99d16fb21d2ef02e8eeb5ada`

复核清除 commit：`0f99690c2aee50b3eed8cb59adbb54c3b62abc03`

Authority sync commit：`0895f56a9906509071951a685e673e527d6d174f`

Review result：`CLEAR`；复核结论文件 `evidence/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）；P0/P1/P2 无；详见第 6 节

## 1. 本轮目标

落实 R2-T12 §6 复评触发点：对 T12 deferred 的 11 个 `lib.rs` inline tests 逐测试核对 runner fixture / store / 冻结语义依赖，写成显式裁决。按用户开包指示，本包为评估包，不受降幅约束，拿不准一律 deferred。

## 2. 改动范围

新增（仅文档）：

- `tasks/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`
- `evidence/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1-result.md`

零修改：`.rs` / `.js` / `.ts` / UI / CSS / schema / shape gate / workflow state JSON 一概未动。

## 3. 评估方法与证据

- 通读 deferred 带全部 11 个测试体（`lib.rs` 4478-5044，行号按 planning baseline）。
- 依赖扫描（grep，记录于任务包 §3）：
  - governance 带 4490-4828：`StubCodexResumeRunner|CodexResumeRunner|runner|stub|temp_dir|fs::|bootstrap_project_workflow|execute_workflow_node_dispatch` 零命中——纯内存派生 / 边界表测试。
  - director review 拒绝路径 4943-5044：runner 类关键词零命中——store-local 拒绝测试，与 T11/T12 已迁形态同款。
  - director review 回收路径 4830-4941：`StubCodexResumeRunner` + `execute_workflow_node_dispatch_for_index_at` 命中 2 处——runner fixture 端到端实锤。
- helper 定位：`fixture_director_review_request` 在 5613 行，处于建议迁移带之外，按既定模式留 `lib.rs`。

## 4. 裁决结果

- **可迁 9 个**：workflow governance boundary 带 8 个（4490-4828，连续）+ `workflow_dispatch_director_review_rejects_invalid_state_and_dispatch`（4943-5044）。判据：零 runner、非端到端、不触碰冻结语义；其中 8 个为 T0 优先级 2 "纯只读派生" 同构，1 个为 T11/T12 已迁 store-local 拒绝形态同构。
- **禁迁确认 1 个**：`workflow_dispatch_director_review_records_completed_dispatch`——StubCodexResumeRunner + dispatch execute 链路，T0 暂缓组与代班档案"共享 stub runner"双重命中，归入既定禁迁清单（33 → 34）。
- **deferred 维持 1 个**：`compact_last_message_summary_preserves_workflow_machine_control_marker`——纯函数但断言 workflow machine 控制标记语义，代班收紧口径下属"拿不准"，按用户指示①不赌。
- **结论：T 系列可迁切片未到底**。建议 T14 迁移 9 个（两非连续区段 339 + 102 行，预计 `lib.rs` 6,006 → 约 5,567）；T14 后剩余 35 个（禁迁 34 + deferred 1），届时 T 系列到底，写代班清单收尾。

## 5. 验证（零改动基线）

- `cargo test --lib`：471 passed / 0 failed / 16 ignored——与 R2-T12 收口基线一致，证明本包未动代码。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors。
- `git diff --check`：通过。
- `git status`：tracked 文件零改动，工作区仅本包文档。

## 6. 复核结论

复核线只读复核已通过，用户已放行：

- 复核结论文件：`evidence/2026-06-12-root-treatment-r2-t13-deferred-inline-tests-reassessment-v1-review-claude-v1.md`（Reviewer：claude-opus-4-8，复核线临时代班）。
- 最终结论：`STATUS: CLEAR`；P0/P1/P2：无。
- 复核线独立重跑基线门（471/0/16、shape gate 0 errors、`git diff --check` 干净、工作区零代码改动）并独立通读源码复扫依赖：可迁两区段 runner 关键词零命中；禁迁项 4871/4877 runner 实锤。
- 复核线另做产品函数控制流核对：`record_workflow_dispatch_director_review_at`（`workflow_execution_entrypoints.rs:819-918`）全函数无 runner 构造与调用，三条拒绝路径均在写盘前返回 Err——director review 组"一迁一禁"为依赖画像差异，比本包裁决声明更干净。
- 复核确认计数守恒：deferred 11 = 9 + 1 + 1；本包后 44 = 可迁 9 + 禁迁 34 + deferred 1；T14 后剩 35，与任务包 §4 一致。

## 7. 边界确认

本轮没有：

- 迁移任何测试、修改任何代码。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 重开 T12 既定 33 个禁迁裁决。
- 开新车道或预定后续车道选择（属用户与 Codex 回归后决策）。

## 8. 不接受为

- 任何 inline test 迁移已开始或完成（T14 未立项执行）。
- `lib.rs` 行数或 waterline 变化。
- R2 全部完成、R3 Level B 执行、真实 Codex 执行、产品语义变更、UI 修改或 backlog 解冻。
- 后续车道选择已定。
