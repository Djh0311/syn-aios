# Evidence：recovery withdrawal and M2 review v1

日期：2026-06-03

## 复核目标

本轮复核两件事：

1. 错误方向的工作台侧 Codex conversation recovery 实现是否已撤回。
2. 记忆层 M2 候选到正式记忆受控采纳是否可以接受为完成。

## Recovery 撤回复核

产品代码扫描：

```text
rg -n "diagnose_codex_conversation_catalog|write_codex_conversation_recovery_catalog|codex-conversation-recovery\\.v1\\.json|CodexConversationRecoveryDiagnostic|recoveryDiagnostic|onPersistRecoveryCatalog" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

结果：无命中。

文件检查：

```text
test ! -f prototypes/productized-desktop-shell/src-tauri/src/codex_conversation_recovery.rs
```

结果：`codex_conversation_recovery.rs absent`。

结论：

- recovery 产品实现残留未发现。
- 旧 `tasks/2026-06-03-codex-software-conversation-recovery-v1.md` 已标为“已撤回实现，任务目标错误，已被 native 任务 superseded”。
- `CURRENT.md` 和 `tasks/README.md` 已改为指向 `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`。

## M2 复核

已检查关键实现：

- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

确认：

- `adopt_memory_candidate_to_formal_memory` Tauri command 存在。
- `MemoryCandidateAdoptionRef` / `AdoptMemoryCandidateInput` / `AdoptMemoryCandidateOutput` 存在。
- 候选采纳只允许 `candidate_confirmed`。
- secretary / worker / system 不能采纳正式记忆。
- global_director 在 M2 默认拒绝。
- project_director 只能采纳低风险、本项目、project/workflow/session scope 的项目类记忆。
- user_preference / global_blueprint / mature_pattern / medium/high risk / private/secret / user/global scope 必须 user 采纳。
- secret 来源或候选必须 `model_export_policy = blocked`。
- 采纳正式记忆时通过 `create_formal_memory_record_at(...)` 复用 M1.1 上下文绑定 guard。
- 正式记忆审计事件类型为 `memory_candidate_adopted_to_formal_memory`。
- 候选 sidecar 保留 adoption 回链，能反查 memory / version / audit id。

## 验证命令

通过：

```text
cargo test --lib memory_candidate_adoption -- --nocapture
# 7 passed

npm run typecheck
# passed
```

保留 warning：

- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。

## 边界

- 本轮未读取 `/Users/yoyi/.codex`。
- 本轮未写 `/Users/yoyi/.codex`。
- 本轮未执行真实 Codex。
- 本轮未改 `workflow-state.v0.json` 结构。
- M2 不接受为完整记忆层完成、任务包召回完成、任务包注入完成、Obsidian / 知识库 / 向量库 / 图数据库完成。

## 结论

- Recovery 撤回状态可以接受：产品代码残留未发现，文档入口已纠偏到 native app 修复任务。
- M2 可以接受为完成：候选到正式记忆的受控采纳链路已实现并通过聚焦测试。
- 当前下一步可以进入 M3：ObservationStore 和工作流观察入口。
