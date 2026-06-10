# Codex 会话中心与项目内 Agent 会话架构 v1 结果

## 薄弱点先说

- 这轮只是定细节，没有做实现。
- 没有验证 resume 和持续多轮聊天。
- 没有重新审计 Codex++ 源码。
- 项目内会话布局还没有真实 UI 原型。

## 这轮做了什么

明确了后续 Codex 工作台的会话架构：

- Agent 页面是全局 Codex 会话中心。
- 项目页面也能打开项目内的单独 Agent 会话。
- 工作流节点能绑定并调用 Codex 会话。
- 三者必须复用同一套 Codex 会话服务和组件。
- 任务包继续降级为内部协议和审计产物，不作为主界面中心。

## 新增文件

- `/Users/yoyi/workspace/product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-29-codex-agent-session-center-project-binding-v1-result.md`

## 页面细节

Agent 页面：

- 全局 Codex 会话列表。
- 打开单个会话完整时间线。
- 查看会话详情和绑定关系。
- 最终提供新建会话和持续聊天。
- 当前持续聊天要等 resume 探针，不假装已支持。

项目页面：

- 左侧保持项目功能列表。
- 新增或明确 `Agent 会话` 项。
- 展示当前项目关联的 Codex 会话。
- 可以打开项目内单独会话。
- 可以创建绑定到当前项目的会话。
- 可以绑定已有会话到当前项目。
- 可以从项目会话跳转到 Agent 全局会话视图。

工作流页面：

- 节点可以绑定会话。
- 节点可以打开对应会话。
- 节点显示会话状态和最后回传摘要。
- 节点读回 transcript 后交给总指导回收。

## 数据关系

定义了这些模型：

- `AgentAdapter`
- `AgentSession`
- `ProjectAgentSessionBinding`
- `WorkflowNodeSessionBinding`
- `ConversationEvent`

关键口径：

- Codex 的 `thread_id` 是 `native_thread_id`。
- rollout 路径是 `native_rollout_path`。
- 项目归属来源必须标明是用户绑定、工作流绑定、索引推断还是未知。
- transcript 默认按需读取，不默认复制进工作台状态。

## 能力边界

当前支持：

- 会话列表元数据。
- 单会话 transcript 读取。
- 新建无业务测试会话和读回。
- dry-run 工作流状态流。

当前未支持：

- 持续多轮聊天。
- resume 已有会话。
- 并发调度。
- 删除、移动、归档会话。
- provider 配置。
- 真实自动派发业务任务。

## 后续建议

下一步建议派发：

```text
Codex 会话中心只读 UI v1
```

先做 Agent 页：

- 会话列表。
- 打开 transcript。
- 时间线展示。
- 工具调用和工具结果折叠。
- warning 展示。
- 不做发送消息。

之后再做：

```text
项目内 Agent 会话入口 v1
```

让项目页复用同一套会话组件，支持项目过滤、绑定和打开会话。

## 禁止事项复核

- 没有执行 Codex CLI。
- 没有创建新会话。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取授权、密钥或新的业务会话正文。
- 没有改前端、后端或索引代码。
- 没有运行 harness。
