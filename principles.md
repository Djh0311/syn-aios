# Principles

工作台项目的核心原则。不打算让任何工具自动加载；需要某个 agent 知道这些事时，让它先读这个文件和 `CURRENT.md`。

## 1. 不用旧文件反推当前方向

当前方向以 `CURRENT.md` 和 `tasks/README.md` 为准。`STAGE_PLAN.md` 是阶段说明，只有在它和 `CURRENT.md` 一致时才作为阶段计划使用。

旧任务包、旧 evidence、旧 handoff 只提供历史依据，不提供当前行动顺序。归档文件先查 `archive/README.md`，当前状态先看 `CURRENT.md` 和最新 review。

依据：本产品线已经从任务包管理器方向纠偏到 Codex 会话管理和 Codex 工作流编排。

## 2. 任务包是内部协议，不是产品中心

任务包可以继续用于：

- 内部协议。
- 审计。
- 导出。
- 交接。

任务包不应该继续主导主界面、主工作流或下一步任务顺序。

依据：`decisions/2026-05-29-codex-session-workflow-route-correction.md`。

## 3. 方案会变是默认状态

这个项目仍是探索型项目。目标不是让方案永远不变，而是让改方案的成本可控。

做法：

- 新功能先判断是模块内部实现还是模块间接缝。
- 接缝区先写 schema、状态机或协议，再写实现。
- spike 可以丑着跑通全链路，正式能力再加厚。
- 决策要写影响面，被推翻时能追踪哪些文件要复核。

## 4. 事实核心不能放在模型上下文里

LM 可以理解目标、拆任务、生成建议、写回收意见。

事实核心必须落在：

- 本地索引。
- 本地工作流状态。
- evidence。
- handoff。
- review。
- 审计记录。

不接受：

- 任务状态只存在聊天上下文里。
- LM 绕过状态机改状态。
- LM 绕过权限确认执行写入、删除或运行 harness。

依据：`decisions/2026-05-28-extensible-first-development-rule.md`。

## 5. 先单层跑通，再分层扩展

当前先保持 Codex 工作流闭环可解释、可审计、可回收，再选择下一阶段。不要因为最终蓝图很大，就把多 agent、多级组长、个人知识库或记忆层工程化提前包装成已完成事实。

后置内容：

- 多 agent 接入。
- 个人知识库。
- 向量搜索。
- 模型调度。
- Skill 自动安装和仓库化。
- Harness 自动运行。

判断依据：`CURRENT.md`、`tasks/README.md` 和最终蓝图；`STAGE_PLAN.md` 只作为同步后的阶段说明。

## 6. 安全边界先于便利性

默认边界：

- 不读取 `auth.json`、`.env`、密钥、token、授权文件内容。
- 不默认展开全部会话正文。
- 不把索引推断当成用户确认事实。
- 不把 safe probe 包装成真实业务自动执行。
- 不绕过用户确认写 `/Users/yoyi/.codex`。

所有真实 Codex 写入都必须有明确任务、明确 prompt、明确批准和 evidence。
