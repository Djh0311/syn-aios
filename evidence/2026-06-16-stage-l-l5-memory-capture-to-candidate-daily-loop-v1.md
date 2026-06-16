# Stage L / L5 Memory Capture To Candidate Daily Loop Evidence v1

日期：2026-06-16

状态：主线实现与本地验证已完成；独立复核线 Aquinas 复审 `STATUS: CLEAR_WITH_NOTE`，P0 / P1 / P2 / P3 均无；提交前停止，不 `git add` / `git commit`。

## 1. 结论摘要

L5 将“日常操作 -> capture -> observation -> candidate -> 用户确认 -> FormalMemory”的第一条日常可见闭环接到产品面：运行中工作流页新增“日常记忆候选收件箱”，候选可在日常流中看到，并通过现有 M2 `adopt_memory_candidate_to_formal_memory` 确认门单条或批量采纳。L3 operation-control 决策确认后会携带 `captureMemoryEvent` 输入，App 在记录 L3 决策后调用既有 capture 入口生成 observation / candidate。

本包不改 R3 记忆 schema / 17 表，不新增 Tauri 命令，不新增真实执行，不读写 `/Users/yoyi/.codex`，不自动写 FormalMemory，不绕过 M2 用户确认门。

一句话判据：如果运行中日常流能看到待确认候选，采纳动作仍经 PermissionDialog 和 M2 采纳链路，并且操作控制 capture 只生成 observation / candidate、不写 FormalMemory、不新增真实执行，则 L5 可进入复核。

## 2. 实际改动范围

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/memory_daily_loop.rs`：L5 operation-control capture 输入构造 helper、`capture_daily_memory_event` 薄封装与 2 个单测。
- 更新 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：只注册 `mod memory_daily_loop;`，并移除 1 个空行保持 `lib.rs` 水线。
- 更新 `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`：K3 Level-A 过程事实 capture policy 从 `audit_only` 改为 `observation_only`；仍不生成 candidate / FormalMemory，测试断言 `observation_id` 存在且 `candidate_key=None`。

前端：

- 新增 `prototypes/productized-desktop-shell/src/lib/memoryDailyLoop.ts`：日常候选收件箱读模型、单条/批量 M2 采纳 pending action、日常候选暂不处理 / 拒绝 pending action、operation-control capture 输入构造。
- 新增 `prototypes/productized-desktop-shell/src/components/DailyMemoryCandidateInbox.tsx`：运行中页候选收件箱，可见待确认数量、来源、风险、单条/批量采纳、暂不处理、拒绝候选入口和候选边界文案。
- 更新 `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`：接入收件箱；L3 operation-control pending action 携带 `memoryCaptureEvent`。
- 更新 `prototypes/productized-desktop-shell/src/App.tsx`：批量采纳在用户确认后逐条调用 `adoptMemoryCandidateToFormalMemory`；operation-control 决策记录后调用 `captureMemoryEvent` 并 reload candidate stores。
- 更新 `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`：单条 / 批量采纳显示 M2 门、候选清单和确认文案。
- 更新 `prototypes/productized-desktop-shell/src/lib/types/workflow.ts`：新增批量采纳 action 和 `memoryCaptureEvent` pending action 字段。
- 更新 `prototypes/productized-desktop-shell/src/lib/types/memory.ts` / `src/lib/knowledgeBase.ts`：补 `operation_control_decision` source type / label。
- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineL5MemoryDailyLoopScenario.tsx`，并接入 `tests/offline-permission-dialog.test.tsx`。

任务包：

- 新增 `tasks/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md`。

## 3. Coverage / Deferred

本包已接：

- `operation_control_decision`：L3 retry / stop / restart / resume 用户确认后，pending action 携带 capture input；App 记录 operation decision 后调用 `captureMemoryEvent`，policy 为 `candidate_allowed`，生成待确认候选。
- `process_fact_decision`：K3 Level-A 过程事实 capture 从 `audit_only` 调整为 `observation_only`；仍无 candidate / FormalMemory。
- 日常确认面：运行中页“日常记忆候选收件箱”显示待确认候选数、列表、来源、风险，提供单条采纳 / 批量采纳 / 暂不处理 / 拒绝候选入口。

Deferred，未在本包硬合：

- 用户确认方案/计划采纳 -> capture。
- 全局主管最终复核签字 -> capture。
- worker report 更广覆盖 -> capture。

理由：本包先修复“前端 `captureMemoryEvent` 零调用”和日常候选可见性硬缺口；其余来源涉及更多工作流落账点和业务语义，后续应另包接线，不能为达标强合。

## 4. 边界确认

- R3 记忆 schema / 17 表：未改；未改 SQLite schema / migration / importer。
- FormalMemory：未自动写入；只有用户确认 PermissionDialog 后，单条或批量采纳才逐条调用 M2 `adopt_memory_candidate_to_formal_memory`。
- 候选：候选仍是待确认对象，UI 明示“候选不是正式记忆，采纳前必须确认”。
- 暂不 / 拒绝：复用既有 `record-memory-candidate-decision` 路径；`candidate_confirmed` 可“暂不处理”到 `candidate_discarded`，`candidate_needs_review` 可“拒绝候选”到 `candidate_rejected`。两者只写候选 sidecar，不写正式记忆。
- 真实执行：未新增 `Command::new("codex")`、runner 调用、`codex exec` / `codex exec resume` 路径。
- `.codex`：未读写；新增 L5 代码只有否定断言 / 边界文案。
- 普通聊天：未自动 capture。
- 敏感材料：沿用 `memory_capture_bus` prompt body / secret 拦截测试；L5 新输入不包含 prompt body、secret、full transcript 或 `.codex` 内容。
- UI 平台边界：只改桌面 Tauri 工作台现有运行中页局部面板；不做手机端 UI / mobile-first。

## 5. 验证原始输出摘录

Preflight：

```text
node scripts/harness/capability-scan.js --target .
PASS (7)
WARN (10)
FAIL (0)
```

```text
node scripts/harness/guard-state-files.js --target .
Harness state-file guard: /Users/yoyi/workspace/product-line

PASS (19)
envFiles: []
```

前端：

```text
npm run typecheck
> codex-governance-workbench@0.1.0 typecheck
> tsc --noEmit
```

```text
npm run test:offline-interaction
> codex-governance-workbench@0.1.0 test:offline-interaction
> node scripts/run-offline-interaction-test.mjs

offline interaction tests passed: 15
r4 page read model settings test passed
r4 page read model query contract test passed
r4 page read model runtime test passed
r4 page selectors test passed
```

```text
npm run build
> codex-governance-workbench@0.1.0 build
> tsc --noEmit && vite build

vite v7.3.3 building client environment for production...
transforming...
✓ 254 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.59 kB │ gzip:   0.42 kB
dist/assets/index-Cq18P1uG.css  145.61 kB │ gzip:  24.83 kB
dist/assets/index-t-P-45kX.js   1,001.13 kB │ gzip: 272.97 kB

(!) Some chunks are larger than 500 kB after minification.
✓ built in 1.31s
```

Rust 聚焦：

```text
cargo test --lib memory_daily_loop
running 2 tests
test memory_daily_loop::tests::l5_operation_control_capture_input_is_candidate_allowed_with_source_refs ... ok
test memory_daily_loop::tests::l5_daily_capture_creates_observation_and_candidate_without_formal_memory ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 537 filtered out
```

```text
cargo test --lib memory_capture_bus
running 8 tests
...
test memory_capture_bus::tests::memory_capture_candidate_allowed_creates_observation_and_candidate_only ... ok
test memory_capture_bus::tests::operation_control_decision_can_be_captured_as_candidate_without_formal_memory ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 531 filtered out
```

```text
cargo test --lib observation_store
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 538 filtered out
```

```text
cargo test --lib memory_candidate_store
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 538 filtered out
```

```text
cargo test --lib formal_memory_lifecycle
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 532 filtered out
```

```text
cargo test --lib page_read_model
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 532 filtered out
```

```text
cargo test --lib project_workflow_automation
test result: ok. 15 passed; 0 failed; 4 ignored; 0 measured; 520 filtered out
```

```text
cargo test --lib
running 539 tests
...
test result: ok. 518 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out; finished in 11.91s
```

```text
cargo fmt -- --check
<no output; exit 0>
```

Gates：

```text
node scripts/harness/workbench-shape-gate.js --mode check
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Workbench shape gate is read-only; it does not execute Codex, send prompts, read/write /Users/yoyi/.codex, start Tauri, or inspect secrets.
Status: pass
Errors: 0
Warnings: 1
Info: 9
Git HEAD: 4d5c9581f8ec59684ba2afb52b7792ba8ed0b9ae
...
Findings:
- [warn] tauri_command_total_increased: Tauri command total increased; confirm task package shape impact and non-lib.rs placement. {"current":98,"baseline":97}
```

```text
git diff --check
<no output; exit 0>
```

```text
node scripts/harness/checkpoint-audit.js --package stage-l-l5-memory-capture-to-candidate-daily-loop --review evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md --allow-dirty --skip-gates
checkpoint-audit: /Users/yoyi/workspace/product-line
Package: stage-l-l5-memory-capture-to-candidate-daily-loop
Boundary: MECHANICAL facts only (commit reachable / tree clean / files in allow-list / review+STATUS present / gates green / evidence hash-field format). Does NOT judge behavior-change or pitfalls — human review still required.

Resolved claims:
- impl commit:   (none)
- task commit:   (none)
- review file:   evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md
- review STATUS: (parsed at check)
- allow-list:    (none)
- record files:  (none)
- CURRENT.md block found: false

Checks:
- [NA] commits_reachable: no commit claimed (pass --commit/--task-commit or use --package with a CURRENT.md block)
- [WARN] tree_clean: {"declared_dirty":true,"entries":[" M prototypes/productized-desktop-shell/src-tauri/src/lib.rs"," M prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs"," M prototypes/productized-desktop-shell/src/App.tsx"," M prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx"," M prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts"," M prototypes/productized-desktop-shell/src/lib/types/memory.ts"," M prototypes/productized-desktop-shell/src/lib/types/workflow.ts"," M prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx"," M prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx","?? evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md","?? evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md","?? handoffs/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1-result.md","?? prototypes/productized-desktop-shell/src-tauri/src/memory_daily_loop.rs","?? prototypes/productized-desktop-shell/src/components/DailyMemoryCandidateInbox.tsx","?? prototypes/productized-desktop-shell/src/lib/memoryDailyLoop.ts","?? prototypes/productized-desktop-shell/tests/helpers/offlineL5MemoryDailyLoopScenario.tsx","?? tasks/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md"]}
- [PASS] review_status: {"file":"evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md","status":"CLEAR"}
- [NA] current_md_refs: package not referenced in CURRENT.md top (e.g. harness-line package)
- [NA] files_within_allow: no impl commit to inspect
- [NA] gates_green: skipped (--skip-gates)
- [NA] evidence_hash_format: no --record; evidence JSON hash fields not inspected

VERDICT: PASS
```

说明：按用户指令，本包停在提交前，尚无 L5 implementation commit / CURRENT.md checkpoint，因此 checkpoint-audit 只能做提交前 dirty 状态、复核文件与 STATUS 的机械核验；commit / CURRENT.md / commit file-boundary 项需提交后再以实际 L5 commit 重跑。

## 6. 扫描分类

扫描命令按 `git status --short` 中 modified / untracked 文件显式列出执行：

```text
rg -n --fixed-strings \
  -e '已自动记入正式记忆' -e '记忆已自动正式化' -e '无需确认' -e '已执行真实操作' \
  -e 'codex exec' -e 'codex exec resume' -e 'Command::new' -e '/Users/yoyi/.codex' \
  -- <changed/untracked files + L5 evidence/handoff>
```

分类结果：

- L5 新增命中：
  - `memory_daily_loop.rs`：断言 summary 不包含 `/Users/yoyi/.codex`。
  - `offlineL5MemoryDailyLoopScenario.tsx`：反向断言 PermissionDialog 不包含“已自动记入正式记忆”。
  - L5 任务包：边界 / 禁止项 / 扫描清单。
- 既有文件命中：
  - `PermissionDialog.tsx`、`lib.rs`、`offline-permission-dialog.test.tsx` 的 `codex exec` / `.codex` 命中来自既有真实执行权限文案、历史 guard 或否定声明；L5 diff 没有新增真实执行调用。
  - `project_workflow_automation.rs` 的 `.codex` 命中来自既有 K3 fixture/boundary 字段；L5 diff 只改 capture policy 和断言。
- 未发现 L5 新增 `Command::new`、runner 调用、真实 `codex exec` / `codex exec resume` 路径、自动正式化正向文案或“已执行真实操作”正向文案。

## 7. TDD / Verify-After 说明

本续包接手时已有前序实现和测试文件，当前阶段不能诚实证明所有生产代码严格 test-first。因此本记录不声称 L5 红绿 TDD 完整链路；本轮以主线 verify-after、聚焦测试、全量测试、离线交互和独立复核补强收口。

已存在并通过的行为覆盖：

- L5 operation-control capture input 带 source_refs / candidate_allowed / requires_user_confirmation。
- L5 daily capture 生成 observation + candidate，但不创建 FormalMemory sidecar。
- 日常收件箱显示候选数量、候选边界、单条/批量采纳入口。
- 单条采纳进入既有 M2 PermissionDialog；批量采纳形成显式批量 action 且逐条复用 M2 输入。
- operation-control 确认 action 携带 `memoryCaptureEvent`。

## 8. 复核状态

独立复核文件：

- `evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-review-aquinas-v1.md`

- 复核线：Aquinas（`019ece6b-4b39-7830-9553-86b979ec322c`）。
- 初审 STATUS：`CLEAR_WITH_P2`。
- 初审 P0 / P1：none。
- 初审 P2：日常收件箱缺少“暂不 / 拒绝”动作；任务包 §5 要求复核动作包括单条采纳 / 批量采纳 / 暂不 / 拒绝。
- 修复：新增“暂不处理”和“拒绝候选”按钮，均复用既有 `record-memory-candidate-decision`；offline L5 场景断言 `candidate_discarded` / `candidate_rejected` pending action。
- 修复后复验：`npm run typecheck` 通过；`npm run test:offline-interaction` 通过；`npm run build` 通过，仅既有 chunk-size warning。
- 复审 STATUS：`CLEAR_WITH_NOTE`。
- 复审 P0 / P1 / P2 / P3：none。
- Note：“暂不处理”当前语义是 `candidate_discarded`，会移出待办；如果以后需要“稍后再看但保留在收件箱”的 snooze 状态，需要另包设计。

复核重点：

- 是否改了 R3 17 表 schema 或新增真实执行路径。
- L3 operation-control capture 是否只进入 observation / candidate，不写 FormalMemory。
- 日常收件箱是否可见、可达，且采纳仍走 M2 用户确认门。
- 批量采纳是否逐条复用 M2，不绕过确认门。
- §4 未接的 plan adoption / final review / worker report 是否如实 deferred。
- 扫描分类是否覆盖 untracked 新文件。
