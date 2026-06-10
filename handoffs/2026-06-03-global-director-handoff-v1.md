# Global Director Handoff v1

日期：2026-06-03

## 给下一任全局主管

这份交接的目的不是证明某一轮任务做完，而是防止你接手时被长上下文和历史记录带偏。

当前最重要的原则：

- 先读 `CURRENT.md`，再读 `tasks/README.md`。
- 单个 evidence / handoff 只能证明某轮发生过什么，不自动代表当前有效状态。
- 遇到 `superseded`、`撤回`、`目标错误`、`纠偏` 相关记录时，以最新入口和复核记录为准。
- 不要凭旧任务包标题判断任务真实目标。

## 第一优先阅读顺序

接手后先按这个顺序读：

1. `CURRENT.md`
2. `tasks/README.md`
3. `AUTHORITY.md`
4. `docs/middleware-version-development-plan-v1.md`
5. `docs/plans/middleware-version-stage-plan-v1.md`
6. `docs/plans/memory-layer-implementation-slice-v1.md`
7. `evidence/2026-06-03-recovery-withdrawal-and-m2-review-v1.md`
8. `handoffs/2026-06-03-recovery-withdrawal-and-m2-review-v1-result.md`

如果要继续记忆层，再读：

- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`

## 当前可信结论

当前主线：

- 中间版本继续围绕自动化工作流和记忆层落地。
- 最终工作台骨架已完成到 `final-skeleton-16`。
- 会话中心底座硬化已完成：sqlite 是会话目录主权威，`index.json` 只做缓存 / 兼容 / 辅助。
- 工作流派发 readback stats 主路径已迁到 Rust 原生 transcript parser。
- adapter 后端能力声明读模型已完成，但 Claude Code / OpenClaw / OpenCode 尚未正式接入。
- 记忆层 M1、M1.1、M2 已完成。
- 下一步方向是 M3：`ObservationStore` 和工作流观察入口。

M1 / M1.1 / M2 的可信状态：

- M1：正式记忆 sidecar、第一版 version、审计骨架已完成。
- M1.1：正式记忆写入前必须通过 `project_root` 推导的 `project_id` / `workflow_id` / scope 绑定校验。
- M2：`MemoryCandidate -> FormalMemoryStore` 受控采纳已完成，低风险本项目 `candidate_confirmed` 可由 `project_director` 采纳为正式记忆；必须用户确认的候选不能被项目主管、秘书、worker 或 system 绕过。

M2 不能宣称：

- 不能说完整记忆层完成。
- 不能说任务包召回完成。
- 不能说任务包注入完成。
- 不能说正式记忆生命周期完成。
- 不能说 Obsidian / 知识库 / 向量库 / 图数据库完成。

## 最大上下文风险

这个线程里发生过一次严重误解：

用户说的是 Codex 原生软件自己的旧对话列表有旧对话消失，不能被 Codex 识别。

错误理解成了工作台智能体页识别旧 Codex 会话，于是曾经写过并执行过错误方向的 `codex-software-conversation-recovery-v1`。

当前有效状态：

- 错误的工作台侧 recovery 实现已撤回。
- 产品代码中未发现 recovery Rust 模块、Tauri command、前端 recovery 类型 / props / 面板残留。
- `tasks/2026-06-03-codex-software-conversation-recovery-v1.md` 已标记为目标错误，不再派发。
- 真正待执行任务是 `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`。

注意：

- 旧 `codex-software-conversation-recovery-v1` 文件正文里仍保留了大量工作台恢复描述，不能按正文继续执行。
- 只要是 Codex 原生 app 旧对话修复，验收对象必须是 Codex 原生 app 会话列表，不是工作台智能体页。
- 写 `/Users/yoyi/.codex`、Codex sqlite、session index 或缓存前，必须另行取得用户文件级确认、备份和回滚方案。

## 当前不要做的事

不要：

- 不要把 evidence / handoff 里的历史记录当当前命令。
- 不要继续派发 `tasks/2026-06-03-codex-software-conversation-recovery-v1.md`。
- 不要把工作台 sidecar 当成 Codex 原生 app 修复。
- 不要读写 `/Users/yoyi/.codex`，除非用户明确批准具体范围。
- 不要执行真实 `codex exec` / `codex exec resume`，除非当前任务包和用户明确授权。
- 不要修改 `workflow-state.v0.json` 结构，除非有单独任务包。
- 不要把 observation、候选、知识库命中、LLM 摘要直接当正式记忆。
- 不要让秘书、worker、system 采纳正式记忆。
- 不要把 M3 直接做成任务包注入；那是后续 M4 / M6 方向。

## 下一步：M3 该怎么起步

当前没有发现已存在的 M3 任务包。

所以如果用户说“开始 M3”，推荐第一步是先写 M3 任务包，而不是直接改代码。

M3 目标来自 `docs/plans/memory-layer-implementation-slice-v1.md`：

- 新增 `ObservationStore`。
- 从 worker 汇报、项目主管确认、全局主管复核、方案采纳、结果验收中记录 observation。
- 让 observation 可以生成记忆候选。

M3 必须守住：

- observation 不是正式记忆。
- observation 必须带 source refs。
- observation 可以标记 `recorded`、`candidate_created`、`ignored`、`quarantined`。
- 不把 observation 直接注入任务包。
- 不把普通聊天自动做成 observation 后立即入记忆。

建议 M3 任务包命名：

- `tasks/2026-06-03-memory-layer-m3-observation-store-and-workflow-entry-v1.md`

M3 第一版建议范围：

- 后端新增 observation sidecar 或端口。
- 定义 `ObservationRecord`、`ObservationSourceRef`、`ObservationStatus`。
- 只从明确的工作流事件 / 汇报 / 主管确认生成 observation，不扫普通聊天。
- observation 可以生成 `MemoryCandidate`，但必须走现有候选 store。
- UI 只做只读摘要和候选生成入口，不做完整记忆中心。
- 测试覆盖 observation 不等于正式记忆、隔离 observation 不生成 candidate、candidate 仍不等于正式记忆。

## Codex 原生旧对话修复

如果用户要继续修 Codex 原生 app 旧对话列表，入口是：

- `tasks/2026-06-03-codex-native-app-conversation-list-repair-v1.md`

执行前必须确认：

- 用户说的是 Codex 原生 app，不是工作台。
- 是否允许只读访问 `/Users/yoyi/.codex` 元数据。
- 是否有 1 到 3 个消失旧对话的线索。
- 是否允许诊断后进入写入修复。只读诊断通过不等于允许写。

写入前必须有：

- 文件级写入清单。
- 备份路径。
- 回滚方案。
- 用户明确确认。

## 已知未完成项

- M3-M13 记忆层后续切片。
- TaskMemoryPacketBuilder。
- 正式记忆召回和任务包注入。
- 正式记忆编辑、废弃、冻结、合并、拆分。
- 冲突、过期、权限不满足时的排除逻辑。
- Obsidian-compatible 知识库集成。
- 向量库、图索引、理解地图等派生索引。
- Claude Code / OpenClaw / OpenCode adapter 正式接入。
- readback 失败可见化。
- 真实 Tauri UI 完整验收。
- 工作台运维日志系统。

## 接手工作方式

建议每次新任务都先做一个短复核：

```text
1. 读 CURRENT.md
2. 读 tasks/README.md
3. 读当前任务包
4. rg superseded / 撤回 / 目标错误 / 纠偏
5. 再决定能不能执行
```

如果用户要求“直接开发”，也不要跳过第 1 到 4 步。这个项目当前最大的风险不是代码难，而是历史记录互相覆盖、同名任务目标漂移、旧 evidence 被误当权威。

## 本交接边界

本交接只写文档：

- 未改产品代码。
- 未读写 `/Users/yoyi/.codex`。
- 未执行真实 Codex。
- 未改 workflow state。
- 未创建 M3 任务包。

本交接只告诉下一个全局主管如何接手，不替代具体任务包。
