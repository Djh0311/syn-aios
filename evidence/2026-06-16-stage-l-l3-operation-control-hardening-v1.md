# Stage L / L3 Operation Control Hardening Evidence v1

日期：2026-06-16

状态：实现与本地验证完成，独立复核线 Aquinas 复审 `STATUS: CLEAR`；提交前停止，不 `git add` / `git commit`。

## 1. 结论摘要

L3 将 retry / stop / restart / resume 四个运行控制操作做成产品化控制面：用户可在运行中工作流页发起风险确认，确认后只登记为 `confirmed_recorded` 决策提示，不调用 runner、不触发真实 Codex、不停止 / 重启真实进程、不解锁 K3-B2。

本包不接受为：自动 retry / stop / restart / resume 已实现、任一操作已经真实运行、K3-B1 retry 成功、K3-B2 可开始、通用真实执行授权已获得、Stage L 完成。

## 2. 实际改动范围

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/operation_control.rs`：L3 操作读模型、四操作契约、状态集合、decision guard、workflow-state audit 写入函数与单测。
- 更新 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`、`workbench_snapshot_types.rs`、`page_read_model.rs`：把 `operation_control` 接入 WorkbenchSnapshot、warning count、running/agents page snapshot slice 与 coverage。
- 更新 `commands.rs`、`command_registry.rs`、`src/lib/tauri.ts`：新增窄命令 `record_operation_control_decision`，仅向 workflow-state `audit_events` 追加 `operation_decision_recorded` 并返回 `WorkflowStateMutationResult`。
- 更新 `workflow_audit.rs`：新增 `operation_decision_recorded` 审计事件 helper，记录 actor / operation / risk acknowledgement / supervisor review，显式 `real_operation_executed=false`。
- 更新 `runtime_log_store.rs`：新增 `operation_decision_runtime_entry` helper，定义 operation kind/status/gate/audit ref 的安全 runtime 记录形状，显式不做 process control；本包确认动作实际持久化的是 workflow-state audit，不自动写 runtime sidecar。
- 更新 `memory_capture_bus.rs`：允许 `operation_control_decision` 作为 capture / observation / candidate 来源；测试证明该来源可生成候选但不写 FormalMemory，本包不把确认动作自动正式化为记忆。

前端：

- 更新类型：`types/execution.ts`、`types/workbenchSnapshot.ts`、`types/workflow.ts`、`emptySnapshot.ts`。
- 更新 `RunningWorkflowsView.tsx`：操作控制区从 K5 只读建议升级为 L3 四操作确认控件；按钮只产生 `record-operation-control-decision` PendingAction。
- 更新 `PermissionDialog.tsx`：展示 L3 操作、当前门、确认后状态、真执行写入面、读回边界、审计/runtime refs；确认文案为“确认记录决策”。
- 更新 `App.tsx`：确认 L3 action 时调用 `recordOperationControlDecision` 写 workflow-state audit，然后更新 workflow snapshot；不调用 runner、不运行 Codex、不做真实进程控制。
- 更新 offline fixtures/tests：base snapshot 含 `operation_control`，新增 `offlineL3OperationControlScenario.tsx` 承载运行页/L3 操作控件点击与弹窗断言，主测试文件降到 shape-gate 水线以下。

未改 / 未触碰：

- 未修改 `session_continuation_store.rs::run_real_resume_phase_b_with_runner()`。
- 未新增 `Command::new("codex")`、runner 调用、真实 Codex 调用、真实 kill / stop / restart / resume。
- 新增的唯一 Tauri 命令是 workflow-state audit 写入命令；shape-gate 因命令总数 97→98 给出 1 个 warning，已分类为 L3 任务包要求的“追加审计事件”入口，且命令不在 `lib.rs`。
- 未读写 `/Users/yoyi/.codex`，未读 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。
- 未自动写 runtime sidecar 或 FormalMemory；runtime / memory capture 为安全 helper 与候选边界，确认动作实际写入 workflow-state audit。
- 未改 UI/CSS/视觉风格；只升级现有运行页操作控制区的信息和确认入口。

## 3. L3 四操作状态

- retry：`available` → 确认后 `confirmed_recorded`；真执行仍需独立授权窗口。
- stop：`available` → 确认后 `confirmed_recorded`；当前门为 `blocked_no_runtime_handle`，不 kill 真实进程。
- restart：`available` → 确认后 `confirmed_recorded`；当前门为 `blocked_restart_semantics_not_defined`，不新建会话、不 resume、不重跑任务。
- resume：`available` → 确认后 `confirmed_recorded`；当前门为 `gated_real_resume_mario_test_only`，不进入 real-resume phase B，不放宽既有门。

共同不变量：

- `does_execute_in_l3=false`
- `status_after_confirmation=confirmed_recorded`
- `readback_result_count=null`
- `true_operation_available=false`
- `k3_b2_unlocked=false`
- `requires_separate_authorized_window=true`

## 4. 验证输出

`node scripts/harness/capability-scan.js --target .`

```text
Harness capability scan: /Users/yoyi/workspace/product-line

PASS (7)
WARN (10)
FAIL (0)
```

`node scripts/harness/guard-state-files.js --target .`

```text
Harness state-file guard: /Users/yoyi/workspace/product-line

PASS (19)
envFiles: []
```

`cargo test --lib operation_control`

```text
running 6 tests
test operation_control::tests::l3_duplicate_and_blocked_operations_do_not_auto_route ... ok
test operation_control::tests::l3_confirmed_recorded_is_a_recoverable_decision_not_success ... ok
test operation_control::tests::l3_operation_contract_covers_four_controls_without_execution ... ok
test operation_control::tests::l3_operation_decision_rejects_execution_claims_before_write ... ok
test operation_control::tests::l3_operation_decision_writes_audit_without_real_execution_or_zero_readback ... ok
test memory_capture_bus::tests::operation_control_decision_can_be_captured_as_candidate_without_formal_memory ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 531 filtered out; finished in 0.12s
```

`cargo test --lib workflow_audit`

```text
running 4 tests
test tests::workflow_audit_helper_preserves_work_item_state_changed_fields ... ok
test tests::workflow_audit_helper_preserves_permission_decision_fields ... ok
test workflow_audit::tests::k3_b1_recovery_audit_event_records_choice_without_sensitive_payloads ... ok
test workflow_audit::tests::operation_decision_audit_event_records_decision_without_execution_or_sensitive_payloads ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 533 filtered out; finished in 0.00s
```

`cargo test --lib runtime_log_store`

```text
running 2 tests
test runtime_log_store::tests::operation_decision_runtime_entry_records_status_without_process_control_or_sensitive_payloads ... ok
test runtime_log_store::tests::runtime_log_store_redacts_runtime_records_and_keeps_audit_as_refs ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 535 filtered out; finished in 0.00s
```

`cargo test --lib memory_capture_bus`

```text
running 8 tests
test memory_capture_bus::tests::memory_capture_rejects_secret_candidate_path ... ok
test memory_capture_bus::tests::memory_capture_rejects_prompt_body_text ... ok
test memory_capture_bus::tests::memory_capture_corrupt_json_is_rejected_without_overwrite ... ok
test memory_capture_bus::tests::memory_capture_revision_conflict_does_not_overwrite_store ... ok
test memory_capture_bus::tests::memory_capture_duplicate_event_is_rejected_without_append ... ok
test memory_capture_bus::tests::memory_capture_audit_only_writes_no_observation_or_candidate ... ok
test memory_capture_bus::tests::operation_control_decision_can_be_captured_as_candidate_without_formal_memory ... ok
test memory_capture_bus::tests::memory_capture_candidate_allowed_creates_observation_and_candidate_only ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 529 filtered out; finished in 0.05s
```

`cargo test --lib page_read_model`

```text
running 7 tests
test page_read_model::tests::page_read_model_schema_catalog_defines_batch_one_six_pages ... ok
test page_read_model::tests::page_read_model_query_rejects_unknown_or_empty_page ... ok
test page_read_model::tests::page_read_model_schema_catalog_covers_workbench_snapshot_fields ... ok
test page_read_model::tests::page_read_model_query_returns_selector_contract_for_known_page ... ok
test page_read_model::tests::page_read_model_inventory_freezes_r4_a1_contracts_only ... ok
test page_read_model::tests::page_read_model_query_with_snapshot_keeps_non_batch_pages_contract_only ... ok
test page_read_model::tests::page_read_model_query_with_snapshot_returns_payload_for_batch_one_pages ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 530 filtered out; finished in 0.00s
```

`cargo test --lib`

```text
running 537 tests
...
test workflow_audit::tests::operation_decision_audit_event_records_decision_without_execution_or_sensitive_payloads ... ok
...
test result: ok. 516 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out; finished in 9.60s
```

`cargo fmt -- --check`

```text
<no output; exit 0>
```

`npm run typecheck`

```text
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

`npm run test:offline-interaction`

```text
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

`npm run build`

```text
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 252 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.59 kB │ gzip:   0.42 kB
dist/assets/index-Cq18P1uG.css  145.61 kB │ gzip:  24.83 kB
dist/assets/index-CLg4PDEN.js   989.58 kB │ gzip: 270.25 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 1.40s
```

备注：build 仍有既有 Vite chunk-size warning，不是 L3 新失败。

真实浏览器 smoke：

```text
npm run dev
error when starting dev server:
Error: listen EPERM: operation not permitted 127.0.0.1:5173
```

沙箱阻止本地监听后，经用户权限机制批准提升启动 Vite：

```text
VITE v7.3.3  ready in 143 ms
Local:   http://127.0.0.1:5173/
```

Playwright 浏览器自动化尝试：

```text
playwright chromium: Executable doesn't exist at /Users/yoyi/Library/Caches/ms-playwright/chromium_headless_shell-1200/chrome-headless-shell-mac-arm64/chrome-headless-shell
system Google Chrome headless: process did exit: exitCode=null, signal=SIGABRT
```

结论：真实浏览器/Tauri 可视化验收未完成，原因是当前环境缺 Playwright browser binary 且系统 Chrome headless SIGABRT；本包以 offline React render + typecheck + build 覆盖主路径，浏览器验收缺口记为主管线 residual note，结转后续 L4 / 真实 Tauri 可视化验收。

`node scripts/harness/workbench-shape-gate.js --mode check`

```text
Status: pass
Errors: 0
Warnings: 1
Key metrics:
- lib.rs: 5567 lines (prototypes/productized-desktop-shell/src-tauri/src/lib.rs)
- offline-permission-dialog.test.tsx: 3395 lines (prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx)
- Tauri commands: 98 total; 0 in lib.rs
Ratchet waterlines:
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 5567/5567 (same)
- prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx: 3395/3404 (decreased)
- prototypes/productized-desktop-shell/src-tauri/src/types.rs: 5230/5386 (decreased)
- prototypes/productized-desktop-shell/src/views/ProjectsView.tsx: 337/378 (decreased)
Findings:
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":98,"baseline":97}
```

`git diff --check`

```text
<no output; exit 0>
```

`node scripts/harness/checkpoint-audit.js --package stage-l-l3-operation-control-hardening --allow-dirty --skip-gates --review evidence/2026-06-16-stage-l-l3-operation-control-hardening-review-aquinas-v1.md`

```text
Resolved claims:
- impl commit:   (none)
- task commit:   (none)
- review file:   evidence/2026-06-16-stage-l-l3-operation-control-hardening-review-aquinas-v1.md
- CURRENT.md block found: false

Checks:
- [NA] commits_reachable: no commit claimed
- [WARN] tree_clean: declared_dirty=true
- [PASS] review_status: {"file":"evidence/2026-06-16-stage-l-l3-operation-control-hardening-review-aquinas-v1.md","status":"CLEAR"}
- [NA] current_md_refs: package not referenced in CURRENT.md top
- [NA] files_within_allow: no impl commit to inspect
- [NA] gates_green: skipped (--skip-gates)
- [NA] evidence_hash_format: no --record

VERDICT: PASS
```

说明：本包按用户指令停在提交前，且 `CURRENT.md` 不在子线写入范围内，因此 checkpoint-audit 的 commit / CURRENT / allow-list 项为 NA，dirty tree 为预期 warning；review STATUS 解析为 `CLEAR`。

## 5. 危险串扫描分类

扫描命令按 `git status --short` 当前 modified / untracked 文件显式列出，匹配：

```text
自动重试已启用|安全审查已绕过|已执行|已成功|result_count: 0|codex exec|codex exec resume|Command::new|/Users/yoyi/.codex
```

分类：

- `workflow_audit.rs`、`runtime_log_store.rs` 命中 `/Users/yoyi/.codex/state`、`codex exec resume`：均为反向测试 / 历史 runtime redaction fixture，断言敏感或真实执行片段不会泄漏。
- `operation_control.rs` 命中 `/Users/yoyi/.codex/state` / `result_count: 0`：新增负向测试 forbidden fragments，断言 audit event 不泄漏敏感路径、不把未知 readback 写成 0。
- `PermissionDialog.tsx` 命中多处 `/Users/yoyi/.codex` / `codex exec resume`：既有历史真实执行 / 离线角色 / K3-B1 文案分支；L3 新增分支只说明“记录 L3 操作控制决策”，明确不调用 runner、不停止或重启真实进程。
- `offlineExecutionRunQueueTextFixtures.ts` 命中禁止文案与 `codex exec resume`：L3 禁止文案断言列表和既有运行队列边界 fixture，用来证明弹窗不出现 forbidden claim。
- `offline-permission-dialog.test.tsx` 命中“仍未启动工作者”“不应暴露裸 codex exec 命令”：既有反向断言。
- `offlineWorkbenchBaseFixtures.ts` 命中 `/Users/yoyi/.codex`：K3-B1 风险说明 fixture，不是 L3 执行路径。
- `handoff` / `evidence` / `task` 命中 forbidden 词：均为边界说明、禁止项、扫描 pattern 或不可声称清单，不是产品正向状态。
- `commands.rs`、`command_registry.rs`、`tauri.ts` 未命中 `Command::new` / `codex exec` / `/Users/yoyi/.codex`；新增命令只写 workflow-state audit。
- `lib.rs` 命中 `/Users/yoyi/.codex` / `codex exec resume`：既有真实执行 guard / 历史任务包 preview 测试内容，L3 未新增真实执行入口。
- `tasks/2026-06-16-stage-l-l3-operation-control-hardening-v1.md` 命中所有边界词：任务包自身的硬边界和禁止项。
- 未命中 `Command::new` 新增路径。

结论：扫描命中均为任务包边界、禁止文案、既有历史/fixture/guard 或反向测试；未发现 L3 新增真实执行路径。

## 6. 复核状态

独立复核文件：

- `evidence/2026-06-16-stage-l-l3-operation-control-hardening-review-aquinas-v1.md`

Aquinas 初审结论为 `STATUS: FINDINGS`：

- P1：确认 `record-operation-control-decision` 后只写 workflow-state audit 与 `setWorkflowState(result.snapshot)`，没有刷新 `snapshot.operation_control`；运行页可能继续显示旧的 `available` 状态。已修：`App.tsx` 在写 audit 后重新调用 `loadWorkbenchSnapshotFromPageQueries(queryWorkbenchPageReadModel)` 并 `setSnapshot(nextSnapshot)`；`offlineL3OperationControlScenario.tsx` 新增 confirmed snapshot 断言，确认运行页显示 `决策已登记`、仍提示 `仍未执行真实操作`，且 `已记录 重试 决策` 按钮 disabled。

复审结论为 `STATUS: CLEAR`：

- P0：none。
- P1：none；初审 P1 已关闭。
- P2：none。
- P3：none。

复核线确认：

- 未发现 L3 新增真实执行路径。
- `record_operation_control_decision` 是窄 workflow-state audit 写入，不调用 runner / process-control / real Codex。
- 未发现新增 `Command::new("codex")`、真实 `codex exec` / `codex exec resume`、真实 stop / restart / kill、`.codex` 读写或 K3-B2 解锁。
- `confirmed_recorded` 仍是“决策已登记”，不是已执行 / 已成功。
- Level-B / real-resume 既有门未放宽。
