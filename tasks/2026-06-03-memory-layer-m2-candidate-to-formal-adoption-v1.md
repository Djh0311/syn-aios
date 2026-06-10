# Task Package：Memory Layer M2 Candidate To Formal Adoption v1

状态：已完成，记录见 `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md` 与 `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`。  
用途：实现中间版本记忆层 M2：候选到正式记忆的受控采纳。  
执行方式：一个中等批次完成，最终统一验收；不要拆成十几个治理小任务。

## 1. 先说薄弱点

- M1 只完成正式记忆 store / version / audit 骨架。
- M1.1 上下文绑定校验已完成；M2 已在该 guard 之后执行。
- 当前 `memory-candidates.v1.json` 的 `candidate_confirmed` 只表示候选保留，不代表正式记忆。
- M2 的重点不是“自动记住”，而是“谁在什么权限下，把哪个候选采纳成哪条正式记忆，并留下来源、版本和审计”。

## 2. 任务目标

实现受控采纳链路：

```text
MemoryCandidate
-> 采纳请求
-> 控制核心确认权限 / 风险 / 作用域 / 来源 / 冲突
-> FormalMemoryStore 创建 MemoryRecord
-> MemoryVersion 创建第一版
-> MemoryAuditEvent 记录 memory_candidate_adopted_to_formal_memory
-> MemoryCandidateStore 保留候选历史并标记 adopted link
-> UI / 读模型能看到候选和正式记忆的关系
```

M2 完成后可以说：

- 低风险项目候选可以受控采纳为项目正式记忆。
- 必须用户确认的候选不会被项目主管、秘书、worker 或系统绕过。

M2 完成后仍不能说：

- 任务包召回完成。
- 任务包注入完成。
- 完整记忆生命周期完成。
- 中间版本记忆层完成。

## 3. 前置条件

必须已完成：

- M1：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- M1.1：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`

M2 开始前必须复核：

- M1.1 已能拒绝 project_id / workflow_id / scope 与 project_root 不匹配的正式记忆写入。
- M1 原有 record / version / audit / candidate separate 测试仍通过。

## 4. 必须先读

- `CURRENT.md`
- `tasks/README.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端命令，例如 `adopt_memory_candidate_to_formal_memory`。
- 新增输入 / 输出类型：
  - `AdoptMemoryCandidateInput`
  - `AdoptMemoryCandidateOutput`
- 新增控制核心 helper：
  - `validate_memory_candidate_adoption(...)`
- 扩展 `MemoryCandidateStore` 记录候选被采纳后的 link，例如：
  - `adopted_memory_id`
  - `adopted_version_id`
  - `adopted_audit_event_id`
  - `adopted_at`
  - `adopted_by_role`
- 复用 `formal_memory_store::create_record` 或新增内部 helper，确保采纳时仍生成 record / version / audit。
- 扩展前端类型和 Tauri 包装。
- 在记忆入口或项目页候选治理详情显示“已采纳为正式记忆”的正式记忆 ID、version、audit。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

禁止：

- 不自动采纳所有 `candidate_confirmed`。
- 不让秘书采纳正式记忆。
- 不让 worker 写正式记忆。
- 不让黑板候选直接变正式记忆。
- 不把知识库命中直接变正式记忆。
- 不把 LLM 摘要直接变正式记忆。
- 不接 `TaskMemoryPacketBuilder`；那是 M4。
- 不把正式记忆注入 worker 任务包；那是 M6。
- 不做正式记忆编辑、废弃、冻结、合并、拆分；那是 M9。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不扫描知识库。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。

## 6. 建议数据对象

```ts
type AdoptMemoryCandidateInput = {
  project_root: string;
  candidate_key: string;
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director";
  adoption_reason: string;
  expected_candidate_store_revision?: number;
  expected_formal_store_revision?: number;
};
```

```ts
type AdoptMemoryCandidateOutput = {
  candidate_key: string;
  candidate_status: "candidate_confirmed";
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  candidate_store_revision: number;
  formal_store_revision: number;
  warnings: string[];
};
```

候选 store 建议扩展字段：

```ts
type MemoryCandidateAdoptionRef = {
  adopted_memory_id: string;
  adopted_version_id: string;
  adopted_audit_event_id: string;
  adopted_at: string;
  adopted_by_role: "user" | "project_director" | "global_director";
  adoption_reason: string;
};
```

如果不想直接改候选 record 结构，也可以新增 `candidate_adoption_events[]` sidecar 内事件列表，但必须能从候选查到正式记忆 ID。

## 7. 控制核心采纳规则

候选必须满足：

- candidate 存在。
- candidate 当前状态必须是 `candidate_confirmed` 或 `candidate_needs_review`。建议第一版只允许 `candidate_confirmed`，避免把 review 流程糊掉。
- candidate 必须有 `source_refs`。
- candidate 不能是 quarantined / rejected / discarded / superseded。
- candidate 不能已经被采纳。

角色规则：

- `user` 可以采纳任何需要用户确认的候选，只要来源、作用域和安全边界通过。
- `project_director` 只能采纳低风险、本项目、非跨项目、非用户偏好、非全局蓝图、非成熟模式候选。
- `global_director` 在 M2 默认不采纳正式全局记忆；如果要开放，必须只允许用户已经确认的全局内容，并在 evidence 说明。
- `secretary` 不在 actor_role 白名单内。
- `worker` 不在 actor_role 白名单内。
- `system` 不允许采纳正式记忆。

必须用户确认的候选：

- `user_preference`
- `global_blueprint`
- `mature_pattern`
- 跨项目记忆
- 高风险记忆
- 敏感或 secret 记忆
- 会改变安全边界、自动化程度、权限策略、用户偏好的记忆

作用域规则：

- 采纳为 project / workflow / session 正式记忆时，必须通过 M1.1 上下文绑定校验。
- project_director 只能采纳本项目 scope。
- 任何跨项目或 global scope 都不能由 project_director 采纳。

安全规则：

- `secret` 来源或敏感内容必须 `model_export_policy = blocked`。
- 权限不足、冲突未处理、来源不足、上下文绑定不通过时，必须拒绝采纳。

## 8. 正式记忆生成规则

从候选生成 `CreateFormalMemoryRecordInput` 时：

- `claim` 来自 candidate.claim。
- `body` 来自 candidate.body。
- `memory_type` 来自 candidate.memory_type。
- `scope` 来自 candidate.scope。
- `source_refs` 必须完整复制 candidate.source_refs。
- `actor_id` / `actor_role` 来自采纳请求。
- `reason` 使用 adoption_reason，并附加 candidate_key。
- `expected_store_revision` 使用 `expected_formal_store_revision`。

审计事件：

- 可以继续使用 `memory_record_created`，但必须能看出来源是 candidate adoption。
- 更推荐新增 event_type：`memory_candidate_adopted_to_formal_memory`。
- audit reason 必须包含 candidate_key。
- candidate store 也要记录 adoption event 或 adoption ref。

原子性要求：

- 不能出现正式记忆写入成功但候选 adoption ref 丢失的半状态。
- 如果当前 sidecar 无法跨文件事务，必须至少采用安全顺序和补偿策略：
  - 先校验两个 store revision。
  - 写 formal store。
  - 写 candidate adoption ref。
  - 如果第二步成功、第三步失败，必须返回明确 warning，并能通过后续修复任务发现 orphan formal memory。
- 更推荐在 M2 里实现 `adoption_pending` / `adoption_committed` 事件，降低跨 sidecar 半写风险。

M2 必须在 evidence 里说明采用了哪种跨 sidecar 一致性策略。

## 9. UI / 读模型要求

M2 不做完整记忆管理页面。

最小 UI 要求：

- 候选治理详情里能显示：
  - candidate_key
  - 当前候选状态
  - 是否已采纳
  - adopted_memory_id
  - adopted_version_id
  - adopted_audit_event_id
- 正式记忆摘要能显示：
  - 正式记忆数量增加
  - 最近审计是候选采纳
  - 来源候选 key

文案必须避免：

- “AI 自动记住”
- “候选已记住”
- “秘书已批准”
- “worker 已写入正式记忆”
- “完整记忆层完成”

允许文案：

- “候选已受控采纳为正式记忆”
- “采纳时写入 version 和 audit”
- “仍未注入任务包”

## 10. 测试要求

Rust 必须覆盖：

1. `memory_candidate_adoption_project_director_low_risk_project_memory`
   - 低风险本项目 candidate_confirmed 由 project_director 采纳成功。
   - formal store 增加 record / version / audit。
   - candidate store 保留候选并记录 adoption link。

2. `memory_candidate_adoption_rejects_user_preference_without_user`
   - user_preference 候选由 project_director 采纳，拒绝。

3. `memory_candidate_adoption_rejects_secret_without_blocked_export`
   - secret 或敏感内容 model_export_policy 不是 blocked，拒绝。

4. `memory_candidate_adoption_rejects_cross_project_project_director`
   - project_director 采纳其他项目候选，拒绝。

5. `memory_candidate_adoption_rejects_rejected_or_discarded_candidate`
   - rejected / discarded / quarantined / superseded 候选不能采纳。

6. `memory_candidate_adoption_rejects_already_adopted_candidate`
   - 已采纳候选不能重复采纳。

7. `memory_candidate_adoption_rejects_context_binding_mismatch`
   - project_root 与 candidate scope 不匹配，拒绝。

8. `memory_candidate_rejection_does_not_create_formal_memory`
   - 拒绝候选不会生成正式记忆。

前端离线测试必须覆盖：

- 候选采纳后 UI 能看到正式记忆 ID / version / audit。
- 候选确认保留仍不等于正式记忆。
- 越界文案不出现。

## 11. 验证命令

至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib memory_candidate
cargo test --lib formal_memory
cargo test --lib
rustfmt --check src/memory_candidate_store.rs src/formal_memory_store.rs src/control_core.rs
```

如 `cargo fmt --check` 因既有格式债失败，不要批量格式化无关 `src/mcp/**`；在 evidence 说明。

## 12. 验收标准

接受为完成：

- M1.1 已完成并通过。
- 低风险本项目记忆候选可以由项目主管受控采纳为正式记忆。
- 必须用户确认的候选不能被项目主管、秘书、worker 或 system 采纳。
- 采纳后正式记忆有 record / version / audit。
- 候选 store 保留历史，并能反查正式记忆 ID。
- 拒绝 / 废弃 / 隔离候选不会生成正式记忆。
- UI / 读模型不把普通 candidate_confirmed 文案说成“已记住”。

不接受为完成：

- 自动采纳完成。
- 任务包召回完成。
- 任务包注入完成。
- 正式记忆生命周期完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。
- 中间版本记忆层完成。

## 13. 回收记录要求

完成后新增：

- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md` 如发现 M2 定义需要补充边界。
