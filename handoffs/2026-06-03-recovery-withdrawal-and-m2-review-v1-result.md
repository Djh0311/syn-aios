# Handoff：recovery withdrawal and M2 review v1

日期：2026-06-03

## 结论

本轮复核通过：

- 错误方向的工作台侧 conversation recovery 实现已撤回，产品代码残留未发现。
- Codex 旧对话真正待执行任务是 `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`，不是 `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`。
- M2 候选到正式记忆受控采纳可以接受为完成。
- 当前下一步可以进入 M3：ObservationStore 和工作流观察入口。

## 复核依据

检查过：

- `CURRENT.md`
- `tasks/README.md`
- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`
- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`
- M2 关键代码和前端入口。

## 验证

通过：

- `cargo test --lib memory_candidate_adoption -- --nocapture`
- `npm run typecheck`

Recovery 残留扫描无产品代码命中：

- `codex_conversation_recovery.rs` 不存在。
- `diagnose_codex_conversation_catalog` / `write_codex_conversation_recovery_catalog` / `CodexConversationRecoveryDiagnostic` / `recoveryDiagnostic` / `onPersistRecoveryCatalog` 在产品代码和测试中无命中。

## 已更新

- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-recovery-withdrawal-and-m2-review-v1.md`
- `handoffs/2026-06-03-recovery-withdrawal-and-m2-review-v1-result.md`

## 边界

- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 Codex。
- 未改 workflow state 结构。
- 未把 M2 宣称为完整记忆层完成。

## 下一步

进入 M3 时必须守住：

- observation 只是观察记录，不是正式记忆。
- observation 不能直接注入任务包。
- 后续正式记忆仍必须经过候选、采纳、来源、权限、审计和上下文绑定。
