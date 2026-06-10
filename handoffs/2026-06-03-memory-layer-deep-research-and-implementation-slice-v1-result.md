# memory-layer deep research and implementation slice handoff v1

日期：2026-06-03

## 结论

已完成记忆层深度复核，并新增中间版本记忆层实施切片：

- `docs/plans/memory-layer-implementation-slice-v1.md`

接受为：

- 记忆层设计已按本地权威文档重新复核。
- `docs/memory-layer-design-v1.md` 被明确为最终工作台记忆层设计主依据。
- `docs/agent-memory-governance.md` 被明确降级为 harness / agent 工程治理参考，不能覆盖最终记忆层设计。
- 当前候选治理实现和正式记忆缺口已明确。
- 中间版本记忆层后续 M1 到 M13 执行批次已定义。
- 已明确 M1 到 M6 只是第一条真实闭环，不能宣称中间版本记忆层完成。
- 已补齐正式记忆生命周期、实体和关系治理、维护任务、成熟模式、跨项目记忆和最终验收。
- 下一步应写 M1 具体任务包。

不接受为：

- 正式记忆系统完成。
- 记忆层代码实现完成。
- 任务包记忆注入完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。

## 改动文件

- `docs/plans/memory-layer-implementation-slice-v1.md`
- `CURRENT.md`
- `evidence/2026-06-03-memory-layer-deep-research-and-implementation-slice-v1.md`
- `handoffs/2026-06-03-memory-layer-deep-research-and-implementation-slice-v1-result.md`

## 当前判断

当前 app 只完成记忆候选治理：

- `candidate_confirmed` 只表示候选确认保留。
- 候选不会自动写正式 `MemoryRecord`。
- UI 不应出现“已记住 / 正式记忆已写入”。

真正的记忆层闭环仍需继续做：

- ObservationStore
- FormalMemoryStore
- MemoryVersion
- MemoryAuditEvent
- MemoryConflict / MemoryLintFinding
- TaskMemoryPacketBuilder
- 工作流任务包注入

## 下一步任务

建议下一步写并执行：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`

M1 范围只做：

- 正式记忆受控存储。
- 正式记忆第一版 version。
- 记忆审计事件。
- 只读读模型。

M1 禁止：

- 不从候选自动升级。
- 不接向量库 / 图数据库 / Obsidian。
- 不注入任务包。
- 不执行真实 Codex。

## 验证

本轮只做文档验证，没有跑产品代码测试。

已只读确认新文档存在，并确认文档包含核心对象、闭环、M1 到 M13、Stop 条件和知识库边界。

## 边界

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 workflow state JSON。
- 未迁移数据库。
- 未写正式记忆。
