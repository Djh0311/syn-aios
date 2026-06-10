# adapter recovery and memory M1 task package handoff v1

日期：2026-06-03

## 本轮完成

按用户要求先回收 adapter，再写 M1。

已完成：

1. 回收核对 `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`。
2. 新增 M1 任务包：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`。
3. 更新 `CURRENT.md` 和 `tasks/README.md`。
4. 新增本 handoff 和对应 evidence。

## Adapter 回收结论

接受为：

- 后端 `WorkbenchSnapshot.agent_adapters[]` 读模型完成。
- Agent 页优先展示后端 descriptor。
- 秘书只读模型优先读取后端 adapter warnings。

不接受为：

- Claude Code / OpenClaw / OpenCode 已接入。
- adapter 能力已重新真实验证。
- 真实 Codex 执行语义已修改。

## M1 任务包结论

M1 任务包路径：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`

M1 只允许实现：

- `FormalMemoryStore` / `MemoryVersionStore` / `MemoryAuditStore`。
- 显式正式记忆创建命令。
- 创建 record 时同步创建 version 和 audit。
- 最小只读读模型。
- 测试和 evidence / handoff。

M1 禁止：

- 候选采纳。
- 任务包召回和注入。
- 生命周期操作。
- Obsidian / 知识库。
- 向量库 / 图数据库。
- 真实 Codex。
- `/Users/yoyi/.codex`。

## 验证

本轮只做文档验证，没有跑产品代码测试。

已确认：

- adapter evidence / handoff / task package 均存在。
- M1 任务包已存在。
- `CURRENT.md` 和 `tasks/README.md` 已更新。

## 下一步

把 M1 任务包交给其他对话执行即可。

执行后必须新增：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`
