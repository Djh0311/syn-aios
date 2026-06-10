# 决策：保留 Codex 会话开发方案，优先推进工作流

## 结论

Codex 会话开发方案保留，但当前实现顺序调整为先做工作流。

依据：

- 用户明确说“保留会话开发方案，优先做工作流”。
- 现有工作台目标已经从任务包管理器纠偏为 Codex 会话管理和工作流编排。
- 会话能力仍是后续工作流自动执行的底座，但当前最卡的是工作流还不能被用户清楚编排和流转。

大白话：

- 会话线不废。
- 现在先不继续深挖会话页美化、多轮聊天和 Codex++ 体验对齐。
- 下一步先让项目里的工作流能被创建、看懂、分派、流转、回收。

## 薄弱点

- 如果暂缓会话控制，下一版工作流仍不能真正自动驱动 Codex 开发线。依据：`codex resume` 多轮控制仍未验证。
- 如果只做工作流 UI，不做状态机，仍会是画布壳。依据：当前项目工作流草稿只有默认节点和边，缺少真实流转命令。
- 如果现在继续做会话 UI，会改善浏览体验，但不能解决“总指导派发、执行线反馈、总指导回收”的主问题。依据：用户当前要求优先工作流。
- 当前会话读取不是全格式完备。依据：`transcript_reader.py` 对未知事件只保留诊断并打 warning。

## 已知

- 单会话 transcript 读取 v1 已完成，可以按索引内 `thread_id` 读取一条会话的结构化时间线。
- 受控真实会话写入探针 v2 已完成，证明 `codex exec` 可以新建无业务测试会话并读回。
- Agent 页面已有只读会话中心雏形。
- 项目页面已有 `Agent 会话` 入口，按索引推断过滤当前项目会话。
- 工作流事实层 v0 已有 JSON 存储底座。
- 项目默认工作流草稿初始化 v1 已有。
- 任务包相关能力保留为内部协议、审计和导出能力，不再作为主界面中心。

依据：

- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/archive/handoffs/2026-05-29-codex-session-full-transcript-v1-result.md`
- `product-line/archive/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-result.md`
- `product-line/archive/handoffs/2026-05-29-desktop-shell-project-agent-session-entry-v1-result.md`

## 未知

- `codex resume <session_id> <prompt>` 是否能稳定支持持续多轮聊天。
- 工作台内直接聊天最终应走 Codex CLI、官方接口、还是类似 Codex++ 的外部增强路线。
- 长任务运行时，Codex JSON 事件流是否足够支撑实时进度展示。
- 项目和会话的归属关系，哪些应由索引推断，哪些必须由用户确认绑定。
- 工作流自动执行时，失败重试、暂停、人工介入和权限确认的边界还没定。

这些未知不能用猜的方式补齐。

## 假设

- 当前阶段仍只治理 Codex，不接入 OpenClaw、OpenCode、Claude Code、VS Code 等其他 agent。
- 工作流 v1 先做工作台自己的状态流转，不默认启动真实 Codex CLI。
- 会话正文读取仍以用户选择或工作流授权为前提，不默认全量展开。
- 工作台状态只保存会话引用、绑定关系、摘要和审计，不默认复制完整 transcript。
- 任务包文件继续作为可导出交接物，不作为主操作界面。

## 保留的会话开发方案

后续会话线仍按这条路线走：

- 全局 Agent 页是 Codex 会话中心。
- 项目页能打开项目范围内的 Codex 会话。
- 工作流节点能绑定 Codex 会话。
- 同一套会话能力复用于 Agent 页、项目页和工作流节点，不做三套聊天系统。
- 会话时间线支持用户消息、assistant 消息、工具调用、工具输出、命令输出、系统事件。
- 系统内容、工具调用和命令输出默认折叠。
- 后续支持创建会话、恢复会话、多轮聊天和读回结果。

保留但后置：

- Agent 会话中心 UI v2 精修。
- Codex++ 式完整会话管理体验。
- `codex resume` 多轮控制探针。
- 会话删除、移动、归档。
- Provider 配置、脚本注入、CDP 增强。

## 当前优先工作流

下一步优先做“项目工作流最小编排闭环”：

- 项目内能看到清楚的工作流。
- 工作流能创建工作项。
- 工作项能绑定角色和节点。
- 状态能从草稿流转到待派发、执行中、待回收、已接受或需修改。
- 每次流转写入工作台自己的审计事件。
- UI 以工作流为中心，不以任务包文件为中心。

这一步不要求真实启动 Codex 会话。

## 对既有任务包能力的处理

继续保留：

- 任务包草稿登记。
- Markdown 预览。
- 字段编辑。
- 文件生成。
- ready 检查。

但这些能力只作为：

- 工作流节点之间的内部消息格式。
- 总指导和执行线之间的可导出交接物。
- 审计和复盘材料。

它们不再决定主界面方向。

## 当前权威影响

本决策补充并收窄以下文档：

- `2026-05-29-codex-session-workflow-route-correction.md`
- `2026-05-29-codex-agent-session-center-project-binding-v1.md`

本决策没有废弃会话线，只是把实现顺序改成：

1. 工作流最小编排闭环。
2. 工作流节点绑定会话。
3. Codex resume / 多轮聊天探针。
4. Agent 会话中心体验补齐。
