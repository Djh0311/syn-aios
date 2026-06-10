# Task Package：final-skeleton-15 秘书核心只读模型 v1

状态：待执行。  
用途：在候选治理最小闭环之后，建立秘书核心协作层的第一版只读模型。  
对应总包：`2026-06-01-final-workbench-skeleton-execution-package-v1.md` 的 Skeleton-15。

## 1. 先说薄弱点

- 秘书是核心协作角色，不是一个固定页面。
- 这一轮不能做秘书聊天、自动派发、自动写事实或自动写正式记忆。
- 如果只在某个页面硬塞一个“秘书面板”，会和架构冲突。
- 如果让秘书建议直接变成 `PendingAction` 或 workflow state 写入，也会越过控制核心。
- 这一轮的价值不是“秘书已经能干活”，而是先让工作台能稳定生成秘书只读上下文、风险、建议和候选。

一句话边界：

```text
秘书只读模型 = 帮用户看清状态和待确认事项。
秘书建议 = 建议或候选。
秘书建议不等于事实变更、任务派发、权限批准或正式记忆写入。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`

架构依据：

- `docs/workbench-system-architecture-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`

前置完成依据：

- `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`
- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`
- `handoffs/2026-06-03-final-skeleton-12-adapter-capability-registry-v1-result.md`
- `handoffs/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1-result.md`

建议实现参考：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 3. 已知事实

- 当前 `WorkbenchSnapshot` 只包含索引、项目、会话、技能、插件、任务和诊断。
- 当前 `WorkflowStateSnapshot` 已包含 `project_workflows` 和 `project_blackboards`。
- 黑板候选 sidecar 已是 `blackboard-candidates.v1.json`。
- 记忆候选 sidecar 已是 `memory-candidates.v1.json`。
- 项目页已经有候选治理条。
- 适配器能力声明目前是前端只读读模型，不是后端正式 `agent_adapters[]`。
- 首页 UI 内容已经确定，本任务不能重做首页。

## 4. 未知和假设

未知：

- 秘书最终长期入口会在哪里。
- 秘书未来是否有低风险自动整理权限。
- 正式记忆写入和 Obsidian / 知识库集成何时开始。

本任务采用的假设：

- 第一版只做前端纯读模型，不新增后端命令，不新增状态文件。
- UI 第一落点使用“可复用只读组件”，可以放在全局壳或已有右侧区域，但不能把秘书定义成某个固定页面。
- 不改首页主视觉和已有首页内容。

如果执行时发现必须写后端状态、正式记忆、workflow state、真实 Codex 或 `/Users/yoyi/.codex`，立即停止并回传。

## 5. 总目标

实现一个可复用的秘书只读模型：

- `SecretaryContext`
- `SecretarySuggestion`
- `SecretaryRiskSignal`
- `SecretaryMemoryCandidate`
- `SecretaryActionProposal`

并从现有只读数据派生：

- 全局状态摘要。
- 项目运行状态。
- 候选治理待处理项。
- 权限、失败、超时、诊断风险。
- 记忆候选和记忆整理提示。
- 需要用户确认的下一步建议。

## 6. 全局禁止

- 不做秘书聊天。
- 不调用 LLM。
- 不接本地模型或云模型。
- 不接 Claude / OpenClaw / OpenCode。
- 不执行真实 Codex。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读完整 transcript。
- 不写 workflow state JSON。
- 不改 `workflow-state.v0.json` 结构。
- 不写正式事实。
- 不写正式 `MemoryRecord`。
- 不写正式长期记忆。
- 不创建或更新 Obsidian vault。
- 不接向量库。
- 不接图数据库。
- 不运行 MCP canvas run。
- 不运行 harness。
- 不写真实业务项目目录。
- 不把秘书固定成首页、项目页、画布页或右侧栏的附属功能。
- 不把 `SecretaryActionProposal` 直接转成 `PendingAction` 自动执行。
- 不批量格式化 `src/lib.rs` 或 `src/mcp/**`。

## 7. 建议改动范围

优先新增：

- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`

可按需新增：

- `prototypes/productized-desktop-shell/src/components/SecretaryBrief.tsx`

可按需小改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

不建议改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`

如果确实需要改 Rust，必须说明为什么前端纯读模型不够，并额外跑 Rust 验证。

## 8. 数据模型要求

建议在 `secretaryReadModel.ts` 中定义第一版类型，避免污染后端事实模型：

```ts
export type SecretaryContext = {
  context_id: string;
  source_kind: "derived_read_model";
  generated_at_label: string;
  global_summary: SecretaryGlobalSummary;
  project_summaries: SecretaryProjectSummary[];
  risk_signals: SecretaryRiskSignal[];
  suggestions: SecretarySuggestion[];
  memory_candidates: SecretaryMemoryCandidate[];
  action_proposals: SecretaryActionProposal[];
  warnings: string[];
};
```

必须包含：

- `source_kind: "derived_read_model"`。
- `warnings`，至少包含 `secretary_context_is_read_only`。
- 来源引用字段，能追到 project、workflow、candidate、permission 或 diagnostic。

### 8.1 SecretarySuggestion

建议字段：

```ts
export type SecretarySuggestion = {
  suggestion_id: string;
  kind:
    | "review_candidate"
    | "review_permission"
    | "inspect_failed_workflow"
    | "inspect_stale_session"
    | "review_memory_candidate"
    | "read_project_status";
  title: string;
  summary: string;
  priority: "low" | "medium" | "high";
  source_refs: SecretarySourceRef[];
  requires_user_confirmation: true;
  is_fact_change: false;
};
```

规则：

- 建议只能引导用户查看或确认。
- `is_fact_change` 必须是 `false`。
- `requires_user_confirmation` 必须是 `true`。

### 8.2 SecretaryRiskSignal

风险来源至少覆盖：

- `workflowStateError`。
- diagnostics warnings。
- pending permission requests。
- failed / timed_out execution attempts。
- pending blackboard candidates。
- pending memory candidates。
- adapter descriptor warnings。

风险只能提示，不允许自动处理。

### 8.3 SecretaryMemoryCandidate

这一轮只做只读候选展示或临时候选建议：

- 可以引用 `memory-candidates.v1.json` 中已有候选。
- 可以从项目黑板的 `memory_candidate` entry 派生“秘书建议候选”。
- 不能调用 `createMemoryCandidate` 自动落 sidecar。
- 不能创建正式 `MemoryRecord`。

必须显示边界：

```text
候选不等于工作台已经长期记住。
```

### 8.4 SecretaryActionProposal

建议字段：

```ts
export type SecretaryActionProposal = {
  proposal_id: string;
  kind:
    | "open_project"
    | "open_agent_session"
    | "open_candidate_governance"
    | "open_memory_review"
    | "open_audit_review";
  title: string;
  target_ref: SecretarySourceRef;
  requires_user_confirmation: true;
  executable_now: false;
  blocked_reason: string;
};
```

规则：

- `executable_now` 第一版必须是 `false`。
- 不把 proposal 直接塞进 `PendingAction`。
- 如果要加点击动作，只能是导航或展开 UI，不能写事实。

## 9. 派生函数要求

新增纯函数：

```ts
export function deriveSecretaryContext(input: {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  workflowStateError?: string | null;
}): SecretaryContext;
```

派生规则：

1. 从 `snapshot.summary` 生成全局数量摘要。
2. 从 `workflowState.project_workflows` 汇总运行中、失败、超时、待 review 工作项。
3. 从 `workflowState.project_blackboards` 汇总风险、权限请求、记忆候选、知识引用。
4. 从 `blackboardCandidateStore.records` 汇总 pending / confirmed / rejected / deferred / discarded。
5. 从 `memoryCandidateStore.candidates` 汇总 pending / confirmed / rejected / quarantined / discarded。
6. 从 diagnostics 和 adapter warnings 生成风险信号。
7. 生成最多 5 条高信号建议，避免 UI 变成噪音列表。

## 10. UI 要求

必须是只读。

允许：

- 新增可复用 `SecretaryBrief` 组件。
- 在全局壳的已有右侧区域、通知/待办/审计/运行区域，放一个很小的只读摘要。
- 在项目页或记忆页以后复用同一个组件。
- 显示建议、风险、候选数量和来源。

禁止：

- 新增固定“秘书页面”并把秘书定义为该页面。
- 改首页主视觉。
- 把秘书塞进项目画布右侧栏当成新的主面板。
- 显示“秘书已处理”“秘书已执行”“秘书已记住”。
- 增加会触发事实写入的按钮。

首版 UI 文案建议：

- “秘书只读摘要”
- “需要你确认”
- “候选，不是正式记忆”
- “建议，不是事实变更”

## 11. 测试要求

必须补离线测试，至少覆盖：

1. 有 pending permission request 时生成 `review_permission` 风险/建议。
2. 有 failed 或 timed_out attempt 时生成风险信号。
3. 有 pending blackboard candidate 时生成候选治理建议。
4. 有 pending memory candidate 时显示“候选不是正式记忆”边界。
5. 所有 `SecretaryActionProposal` 都是 `executable_now: false`。
6. 所有 `SecretarySuggestion` 都是 `is_fact_change: false`。
7. UI 渲染不出现“已记住”“已执行”“正式事实已写入”等越界文案。

## 12. 验证命令

在：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
```

必须跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

如果本轮改了 Rust，再额外跑：

```text
cargo test --lib
```

如果只新增/修改 Rust 小文件，再额外跑对应：

```text
rustfmt --check <changed-rust-files>
```

不要因为既有 `src/lib.rs` 或 `src/mcp/**` 格式债而批量格式化。

## 13. 验收标准

接受为：

- 秘书只读模型第一版完成。
- 可从现有 snapshot、workflow state、黑板候选 sidecar、记忆候选 sidecar 派生秘书上下文。
- 能展示风险、建议、候选和下一步确认提示。
- UI 文案明确秘书建议不是事实变更。
- UI 文案明确记忆候选不是正式长期记忆。
- 测试证明秘书 proposal 不会自动执行。
- 没有写 workflow state。
- 没有写正式事实。
- 没有写正式记忆。

不接受为：

- 秘书聊天完成。
- 秘书自动执行完成。
- 秘书能直接派发任务。
- 秘书能直接批准权限。
- 秘书能直接写正式记忆。
- 记忆管理界面完成。
- Obsidian / 知识库集成完成。
- Claude / OpenClaw / OpenCode 接入完成。

## 14. 必须输出

执行完成后必须新增：

- `evidence/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`
- `handoffs/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`

如果执行中发现需要越过边界，必须停止，并在 handoff 写清楚：

- 卡在哪里。
- 为什么只读模型不够。
- 需要用户确认的下一步是什么。

## 15. 完成后

普通情况下继续 Skeleton-16：项目工作流页最终收敛。

但如果 UI 因秘书摘要变乱，先不要做 Skeleton-16，实现者需要把 UI 风险写入 handoff，并建议单开 UI 收敛任务。
