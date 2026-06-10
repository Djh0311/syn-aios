# memory-layer deep research and implementation slice evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮没有改产品代码，也没有实现正式记忆写入。
- 本轮没有运行代码测试，因为只新增和更新文档。
- 本轮没有接 Obsidian、向量库、图数据库或 SQLite 迁移。
- 当前实现仍只到候选治理最小闭环；正式 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent` 和 `TaskMemoryPacketBuilder` 都未实现。

## 研究依据

最终设计权威：

- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `CURRENT.md`
- `tasks/README.md`

历史 / 实现参考：

- `docs/agent-memory-governance.md`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`
- `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`

## 关键结论

记忆层不是普通聊天摘要、向量库、图谱、Obsidian vault、Markdown 文件夹或当前 `memory-candidates.v1.json`。

`docs/agent-memory-governance.md` 只能作为 harness / agent 工程治理参考，不是最终工作台记忆层的产品权威。最终功能和设计以 `docs/memory-layer-design-v1.md` 和最终蓝图为准。

记忆层是会影响 agent 行为的确认事实系统，必须包含：

- ObservationStore
- MemoryCandidate
- MemoryRecord / MemoryPage
- MemoryVersion
- MemorySourceRef
- MemoryConflict / MemoryLintFinding
- MemoryAuditEvent
- TaskMemoryPacket / TaskMemoryPacketBuilder

当前代码只完成：

- 记忆候选 sidecar。
- 候选创建和候选状态变更。
- 候选不能变正式记忆的控制核心校验。
- UI 文案避免“已记住 / 正式记忆已写入”误导。

当前代码没有完成：

- 正式记忆写入。
- 记忆版本。
- 正式记忆审计。
- 记忆冲突检测。
- 任务包召回和注入。
- 权限撤回影响未来召回。
- 知识库 / Obsidian 集成。

## 新增文档

- `docs/plans/memory-layer-implementation-slice-v1.md`

最终审核后，该文档把中间版本记忆层落地拆成 M1 到 M13：

- M1：正式记忆受控存储和审计骨架。
- M2：候选到正式记忆的受控采纳。
- M3：ObservationStore 和工作流观察入口。
- M4：任务记忆包生成器和预览。
- M5：冲突和记忆 lint 最小阻断。
- M6：工作流任务包注入和端到端闭环。
- M7：记忆管理 UI 最小入口。
- M8：知识库和 Obsidian 接口占位。
- M9：正式记忆生命周期操作。
- M10：实体和关系治理。
- M11：维护任务和记忆 lint。
- M12：成熟模式、跨项目记忆和完整验收。
- M13：中间版本记忆系统最终验收。

M1 到 M6 只能证明第一条真实闭环完成，不能宣称中间版本记忆层完成。M7 到 M12 补齐中间版本完整记忆系统，M13 做总验收。

## 验证

已做只读验证：

- `docs/plans/memory-layer-implementation-slice-v1.md` 已存在。
- 文档中包含 `ObservationStore`、`MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`、`TaskMemoryPacketBuilder`、`candidate_confirmed`、`Obsidian`、`Stop 条件`、`M1`、`M6`、`M13`。
- `CURRENT.md` 已更新下一步建议。

未跑：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

原因：本轮没有改产品代码。

## 边界确认

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 workflow state JSON。
- 未迁移数据库。
- 未写正式记忆。
- 未接知识库 / Obsidian / 向量库 / 图数据库。

## 下一步

如果用户接受 `docs/plans/memory-layer-implementation-slice-v1.md`，下一步写：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`

该任务只做 M1：正式记忆受控存储和审计骨架。
