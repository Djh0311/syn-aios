# adapter recovery and memory M1 task package evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮只做 adapter 回收核对和 M1 任务包编写，没有改产品代码。
- 本轮没有重新跑 `npm` / `cargo` 验证；adapter 任务验证结果引用既有 evidence。
- M1 只是任务包，不是正式记忆实现。

## Adapter 回收核对

已核对：

- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `CURRENT.md`
- `tasks/README.md`

回收结论：

- 接受为后端 `WorkbenchSnapshot.agent_adapters[]` 读模型完成。
- 接受为 Agent 页和秘书读模型优先使用后端 descriptor。
- 不接受为 Claude Code / OpenClaw / OpenCode 已接入。
- 不接受为真实能力重新验证。
- 不接受为真实 Codex 执行语义已修改。

## M1 任务包

已新增：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`

任务边界：

- 只做正式记忆受控存储、第一版 version、审计事件和只读读模型。
- 不做候选采纳。
- 不做任务包召回或注入。
- 不做正式记忆生命周期操作。
- 不接 Obsidian / 知识库 / 向量库 / 图数据库。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。

## 文档更新

已更新：

- `CURRENT.md`
- `tasks/README.md`

## 验证

只读验证：

- adapter evidence / handoff 存在，并写明边界。
- M1 任务包存在。
- `CURRENT.md` 和 `tasks/README.md` 已指向 M1 任务包。

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
- 未改 `workflow-state.v0.json`。
- 未迁移数据库。
- 未写正式事实或正式记忆。
