# Stage L / L5 Memory Capture To Candidate Daily Loop Review - Aquinas v1

日期：2026-06-16

复核线：Aquinas  
agent_id：`019ece6b-4b39-7830-9553-86b979ec322c`  
范围：只读复核；未修改文件，未运行 `git add` / `git commit`，未读取 `/Users/yoyi/.codex`。

## STATUS: CLEAR_WITH_NOTE

P0：none  
P1：none  
P2：none  
P3：none

## 复核范围

已只读核验：

- `tasks/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md`
- `evidence/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1.md`
- `handoffs/2026-06-16-stage-l-l5-memory-capture-to-candidate-daily-loop-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_daily_loop.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/components/DailyMemoryCandidateInbox.tsx`
- `prototypes/productized-desktop-shell/src/lib/memoryDailyLoop.ts`
- `prototypes/productized-desktop-shell/src/lib/types/memory.ts`
- `prototypes/productized-desktop-shell/src/lib/types/workflow.ts`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/tests/helpers/offlineL5MemoryDailyLoopScenario.tsx`

## 初审

初审结论：`STATUS: CLEAR_WITH_P2`

P2：

- 日常收件箱缺少任务包要求的“暂不 / 拒绝”动作。任务包 §5 要求复核动作包括单条采纳 / 批量采纳 / 暂不 / 拒绝；初版 `DailyMemoryCandidateInbox.tsx` 只暴露批量采纳与单条采纳 / 先审查候选状态。

其余结论：

- 未发现 R3 schema / SQLite migration / 17 表变更。
- 未发现 L5 新增真实执行路径、`Command::new("codex")`、runner 调用、`codex exec` / `codex exec resume` 或 `.codex` 读写。
- operation-control capture 只生成 observation / candidate；FormalMemory 仍在 PermissionDialog + M2 adoption 后。
- K3 Level-A 为 `observation_only`，candidate 仍为 `None`，FormalMemory sidecar 不创建。

## 修复核验

已核修复：

- `memoryDailyLoop.ts` 新增 `buildDailyMemoryCandidateDecisionAction`，生成 `record-memory-candidate-decision` pending action，只带 `memoryCandidateDecision`，目标状态限定为 `candidate_discarded` / `candidate_rejected`。
- `DailyMemoryCandidateInbox.tsx` 每条候选新增“暂不处理”和“拒绝候选”；前者写 `candidate_discarded`，后者写 `candidate_rejected`。
- `App.tsx` 中 `record-memory-candidate-decision` 仍走既有 `recordMemoryCandidateDecision`，notice 明确只写候选 sidecar、未写正式长期记忆。
- `offlineL5MemoryDailyLoopScenario.tsx` 断言暂不处理为 `candidate_discarded`，拒绝为 `candidate_rejected`。
- evidence / handoff 已记录初审 P2、修复方式和“暂不处理映射 candidate_discarded”的残余语义。

复审结论：

- 初审 P2 已关闭。
- 日常收件箱现在覆盖“暂不 / 拒绝”，没有绕过 M2，没有写 FormalMemory，没有新增真实执行路径。

## Note

“暂不处理”当前语义是 `candidate_discarded`，会移出待办；如果以后需要“稍后再看但保留在收件箱”的 snooze 状态，需要另包设计。该残余已在 handoff 说明，不构成 P2。

## 复核未做

- 复核线未重跑测试；测试结果以主线报告为证。
- 复核线未做真实浏览器 / Tauri 可视化验收；该残余仍结转 L4。
