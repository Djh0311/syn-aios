# Evidence：Memory Layer M2 Candidate To Formal Adoption v1

日期：2026-06-03  
任务包：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`  
状态：已完成

## 完成内容

- 后端新增受控采纳链路：
  - `AdoptMemoryCandidateInput`
  - `AdoptMemoryCandidateOutput`
  - `MemoryCandidateAdoptionRef`
  - `adopt_memory_candidate_to_formal_memory`
- `MemoryCandidate` 新增 `adoption` 回链，保留候选历史，不把候选 record 改成正式 record。
- 控制核心新增 `validate_memory_candidate_adoption(...)`，限制：
  - 只允许 `candidate_confirmed` 被采纳。
  - `secret` 来源或候选必须 `model_export_policy = blocked`。
  - `secretary` / `worker` / `system` 不在采纳角色白名单内。
  - `global_director` 在 M2 默认拒绝。
  - `project_director` 只可采纳低风险、本项目、project/workflow/session scope 的项目类记忆。
  - `user_preference` / `global_blueprint` / `mature_pattern` / medium/high risk / private/secret / user/global scope 必须由 `user` 采纳。
- 采纳时复用 M1.1 的 `create_formal_memory_record_at(...)`，正式写入继续执行 `project_root` 推导出的 `project_id` / `workflow_id` / scope 绑定校验。
- 正式记忆审计事件使用 `memory_candidate_adopted_to_formal_memory`，并写入第一版 `MemoryVersion`。
- 前端新增采纳类型、Tauri wrapper、权限确认动作、项目候选治理卡按钮和 adoption ID 展示：
  - `adopted_memory_id`
  - `adopted_version_id`
  - `adopted_audit_event_id`
- `candidateGovernance` 摘要新增 adopted count / first adoption，并在正式记忆最近审计为采纳事件时显示“候选受控采纳审计已记录”。

## 跨 sidecar 一致性策略

M2 采用“candidate lock + formal then candidate link”的非数据库事务策略：

1. 获取 `memory-candidates.v1.json` sidecar lock。
2. 校验 candidate store revision。
3. 校验候选状态、来源、风险、作用域和角色权限。
4. 调用正式记忆写入，写 `formal-memories.v1.json` record / version / audit。
5. 写回 candidate adoption ref 和 candidate audit ref。

如果 formal 写入成功但 candidate adoption ref 写回失败，后端返回：

```text
formal_memory_written_candidate_adoption_link_failed: ...
```

同时成功路径 warnings 包含：

- `memory_candidate_adopted_to_formal_memory`
- `candidate_history_retained_with_adoption_link`
- `cross_sidecar_write_formal_then_candidate_link`

残余风险：当前 sidecar 不是数据库事务；极端文件写入失败下可能留下 orphan formal memory。M2 已把该风险显式暴露为错误 / warning，后续 M7+ 生命周期和维护任务应补 orphan 检测或补偿流程。

## 验证

通过：

```text
cargo test --lib memory_candidate_adoption -- --nocapture
# 7 passed

cargo test --lib memory_candidate
# 9 passed

cargo test --lib formal_memory
# 14 passed

cargo test --lib
# 136 passed; 1 ignored

rustfmt --check src/memory_candidate_store.rs src/formal_memory_store.rs src/control_core.rs src/lib.rs src/commands.rs
# passed

npm run typecheck
# passed

npm run test:offline-interaction
# offline interaction tests passed: 9

npm run build
# passed
```

已知非阻断 warning：

- `cargo test --lib` 仍有既有 warning：`JsonRpcError::invalid_params` unused。
- `npm run build` 仍有既有 Vite chunk > 500 kB warning。

## 边界确认

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未修改 `workflow-state.v0.json` 结构。
- 未迁移数据库。
- 未接 Obsidian / 向量库 / 图数据库 / 知识库扫描。
- 未实现任务包召回或任务包注入；该范围仍属于 M4 / M6。
- 未实现正式记忆编辑、废弃、冻结、合并、拆分；该范围仍属于 M9。

## 额外兼容修复（已被后续撤回 superseded）

`npm run build` 曾暴露前端 `AgentView` / `WorkbenchSnapshot` 的 conversation recovery 类型和 props 漂移。为保证构建链路通过，本轮做了最小兼容修复：

- 恢复 TS 侧 `CodexConversationRecoveryDiagnostic` / `CodexConversationRecoveryCatalogWriteResult` 类型。
- 恢复 `writeCodexConversationRecoveryCatalog` / `diagnoseCodexConversationCatalog` Tauri wrapper。
- `AgentView` 接受可选 `recoveryDiagnostic` / `onPersistRecoveryCatalog`，有值时显示只读恢复诊断面板。

该修复不改变 M2 记忆采纳语义，不写 Codex sqlite，不读写 `/Users/yoyi/.codex`。

后续纠偏说明：

- 上述 conversation recovery 兼容修复属于错误方向的工作台侧 recovery 残留，不属于 M2 记忆采纳能力。
- 该 recovery 实现后来已被撤回，当前产品代码中 `codex_conversation_recovery.rs`、`CodexConversationRecoveryDiagnostic`、`writeCodexConversationRecoveryCatalog`、`diagnoseCodexConversationCatalog`、`recoveryDiagnostic`、`onPersistRecoveryCatalog` 均已扫描无残留。
- 当前有效复核记录见 `evidence/2026-06-03-recovery-withdrawal-and-m2-review-v1.md` 与 `handoffs/2026-06-03-recovery-withdrawal-and-m2-review-v1-result.md`。
- 阅读本 M2 evidence 时，不能把“额外兼容修复”理解为当前仍存在的代码，也不能把它当作 M2 的组成部分。
