# 决策：知识库与对话底座双线并行 v1

> **状态校正（2026-08-09）：HISTORICAL / NO ACTIVE WORKSTREAM。** “双线同时活跃”只属于 2026-07-23 当时现场；当前没有活动工程阶段或任务。知识基础设施方向看 `../docs/product/knowledge-infrastructure-canon-v1.md`，知识施工计划当前停放；对话与全工作台方向看产品正本和 2026-08-01 总开发计划。

- 日期：2026-07-23
- 状态：**ACCEPTED**
- 决策人：用户

## 决策

从本日起，以下两个方向同时保持 active，不再把其中一条解释为另一条完成前的暂停项：

1. **知识库线**：完成 Syn 原生知识工作区、`knowledge_open` host relay 安全闭锁与真实 App 十二项验收。
2. **对话底座线**：完成共享 Conversation Transport、主管可信 binding、MCP 工具时序和真实 App 三句替代性验收。

## 并行边界

- 两线使用独立任务包、独立 evidence、独立 Codex 任务和独立验收结论；一条线的离线绿不能替另一条线结算。
- 文档、只读审计和不重叠代码可并行。
- `commands.rs`、`manual_relay.rs`、`manual_relay/conversation_transport.rs`、`exec_process_registry.rs` 等共享承重文件同时只允许一条线写；另一条线先只读，或在指导线确认的隔离工作树中实施。
- 同一真实 store 上的 Syn/Codex/MCP 运行验收不得并发；每次只允许一个有明确包、前后 manifest 和停止合同的真实 App 验收。
- 知识 relay 安全返工和对话线只读恢复审计均已完成并经指导验收。后续知识 N2R 离线施工可与对话线只读准备并行；对话真实 App 重验仍须新授权，且不得与知识线代码写入、构建或任何真实 App 验收并发。

## 不改变的边界

- 主管仍为 `read-only + 空写根`，MCP 仍为 Syn 的统一能力层。
- 不恢复 resident/private-home 旧主路线。
- 不因并行放宽真实 store、卡、chain/worker、任意 shell/filesystem、stage/commit/push 或其他高危权限。
- Obsidian 真嵌入、Electron 迁移和插件生态复刻仍停止；知识库主路线仍是 Syn 原生工作区。

## 当前入口

- 知识库线：`tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md`（已形成，待用户 kickoff）
- 对话底座线：`tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md`（合同已冻结，真实运行未授权）
