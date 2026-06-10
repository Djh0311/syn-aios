# 决策：Codex 会话中心与项目内 Agent 会话架构 v1

## 结论

工作台后续要把 Codex 做成一套统一的会话能力。

Agent 页面是全局 Codex 会话中心；项目页面可以打开项目内的单独 Agent 会话；工作流节点也可以绑定 Agent 会话。但这三处不能各做一套聊天系统，必须复用同一套 Codex 会话服务和同一套会话组件。

大白话：

- Agent 页管全部 Codex 会话。
- 项目页能在当前项目里打开或创建 Codex 会话。
- 工作流节点能引用这些会话去跑任务。
- 底层只有一套“打开会话、读正文、发消息、读回结果”的能力。

## 薄弱点

- 现在还没做到 Codex 原生体验。依据：桌面壳尚未接入 transcript viewer、composer、多轮发送和 resume。
- 现在还不能说“持续多轮聊天已可用”。依据：只验证了 `codex exec` 新建无业务测试会话，没有验证 `codex resume`。
- 当前会话读取模型没有实质参考 Codex++ 的源码实现。依据：`transcript_reader.py` 和相关 evidence 没有 Codex++ 实现引用。
- Codex++ 只能作为会话管理体验参考，不能直接照搬删除、移动、CDP 注入和供应商写入。依据：已有 UI 参考源决策已写明这些不吸收。
- 项目归属有两种来源：索引推断和用户绑定。不能把索引推断直接当成事实。依据：当前索引只提供 `project_root` / `cwd` / rollout 元数据，不能证明用户接受了项目归属。

## 已知

- 用户要求工作台能直接打开 Codex 每一个会话，也能创建会话。
- 用户要求项目里也能打开单独的 Agent 会话。
- 用户要求 Agent 页面最终能管理接入的 Agent，但当前先只做好 Codex。
- 用户要求项目界面打开项目后能编排工作流，并调用 Agent 能力。
- 单会话 transcript 读取 v1 已完成。
- 受控 `codex exec` 新建测试会话和读回 v2 已完成。
- 工作流运行模型 v1 dry-run 已完成。

## 未知

- `codex resume <session_id> <prompt>` 是否能稳定实现多轮聊天。
- 官方 Codex 桌面会话和 CLI session id 的对应关系是否长期稳定。
- 长任务下 `--json` 事件流是否足够表达执行状态。
- Codex++ 的内部会话管理实现细节是否有可直接吸收的读取逻辑。
- 项目页嵌入会话时，最终布局是右侧抽屉、中央面板，还是可拆分标签页，仍需要 UI 原型验证。

## 假设

- 当前只做 Codex，不做 OpenClaw、VS Code、OpenCode、Claude Code 的真实接入。
- 所有会话正文默认按用户选择或工作流授权读取，不默认全量展开。
- 工作台自己的状态只保存会话引用、绑定关系、运行摘要、审计事件；不默认复制完整 Codex transcript 到工作台状态。
- 会话删除、会话移动、归档和 provider 配置先不做。

## 页面结构

### 首页

首页继续只有四个入口：

- Agent
- 项目
- Skill 管理
- Harness 管理

首页不展示会话数量作为主入口。

### Agent 页面

Agent 页面是全局 Agent 控制台。

当前只启用 Codex；OpenClaw、VS Code、OpenCode、Claude Code 等保持未接入空白位。

Codex Agent 详情页分为四块：

1. 会话列表
   - 展示所有索引内 Codex 会话。
   - 支持按项目、时间、来源、是否可读、是否绑定工作流筛选。
   - 显示项目归属来源：索引推断、用户绑定、工作流绑定、未知。
   - 不在列表里展示正文。

2. 会话正文
   - 打开单个会话后显示完整时间线。
   - 用户消息、assistant 消息、工具调用、工具结果、命令输出分层展示。
   - 工具输出默认折叠，用户可展开。
   - 显示读取 warning，例如加密内容省略、未知事件、疑似敏感。

3. 会话详情
   - 显示 `thread_id`、rollout 路径、项目归属、创建/更新时间、模型、token、来源。
   - 显示绑定到哪些项目、哪些 workflow、哪些节点。
   - 提供“复制路径”“打开 rollout 所在位置”等需要确认的动作。

4. 对话输入区
   - 最终目标：能新建会话、持续聊天、向已有会话发送消息。
   - 当前状态：新建测试会话已验证；持续多轮和 resume 未验证，未验证前应显示为能力缺口，不假装可用。

### 项目页面

项目页面打开某个项目后，左侧仍是窄功能列表。

建议项目内一级功能：

- 总览
- 工作流
- Agent 会话
- Skill
- Harness
- Evidence / Handoff
- 诊断

项目里的 Agent 会话不是第二套 Agent 页面，而是当前项目范围的会话视图。

项目 Agent 会话页必须支持：

- 查看当前项目关联的 Codex 会话。
- 打开项目内单独会话。
- 创建“绑定到当前项目”的 Codex 会话。
- 绑定已有 Codex 会话到当前项目。
- 解除工作台自己的项目绑定，但不删除 Codex 原始会话。
- 从项目内会话跳转到 Agent 页面完整会话视图。

项目工作流页必须支持：

- 节点绑定 Codex 会话。
- 从节点打开对应会话。
- 节点显示会话状态和最后回传摘要。
- 节点引用 transcript reader 的读回结果。

## 统一组件

必须复用这些组件，不允许 Agent 页和项目页各自实现一套：

- `CodexSessionList`
- `CodexSessionTimeline`
- `CodexSessionDetails`
- `CodexChatComposer`
- `CodexSessionBindingPanel`
- `CodexWorkflowNodeSessionPanel`

组件可以按上下文改变布局：

- Agent 页面：全局管理视图。
- 项目页面：项目过滤视图。
- 工作流节点：节点绑定视图。

但数据来源和动作入口必须一致。

## 数据模型

### AgentAdapter

表示一个 Agent 接入。

字段建议：

- `adapter_id`
- `agent_type`
- `display_name`
- `status`
- `capabilities`
- `version`
- `last_checked_at_ms`
- `warnings`

Codex 当前建议：

```json
{
  "adapter_id": "codex-local",
  "agent_type": "codex",
  "display_name": "Codex",
  "status": "available",
  "capabilities": {
    "list_sessions": "supported",
    "read_transcript": "supported",
    "create_session": "probe_supported",
    "send_initial_prompt": "probe_supported",
    "resume_session": "unknown",
    "multi_turn_chat": "unknown",
    "delete_session": "disabled",
    "move_session": "disabled"
  }
}
```

### AgentSession

表示工作台认识的一条 Agent 会话。

字段建议：

- `agent_session_id`
- `agent_type`
- `adapter_id`
- `native_thread_id`
- `native_rollout_path`
- `title`
- `project_id`
- `project_binding_source`
- `cwd`
- `created_at_ms`
- `updated_at_ms`
- `source`
- `read_status`
- `control_status`
- `transcript_warning_count`
- `workflow_binding_count`
- `tags`
- `warnings`

说明：

- `native_thread_id` 对 Codex 来说就是 `thread_id`。
- `project_binding_source` 只能是 `index_inferred`、`user_bound`、`workflow_bound`、`unknown` 之一。
- 索引推断不能直接改成用户事实。

### ProjectAgentSessionBinding

表示项目和会话的绑定关系。

字段建议：

- `binding_id`
- `project_id`
- `agent_session_id`
- `binding_source`
- `relationship`
- `label`
- `created_at_ms`
- `created_by`
- `status`
- `warnings`

`relationship` 建议：

- `general`
- `director`
- `developer`
- `reviewer`
- `validator`
- `handoff`

### WorkflowNodeSessionBinding

表示工作流节点和会话的绑定关系。

字段建议：

- `binding_id`
- `workflow_id`
- `node_id`
- `agent_session_id`
- `role_id`
- `binding_mode`
- `lifecycle`
- `last_dispatch_event_id`
- `last_readback_event_id`
- `warnings`

`binding_mode` 建议：

- `create_new_session`
- `select_existing_session`
- `read_only_reference`

`lifecycle` 建议：

- `planned`
- `created`
- `dispatched`
- `running`
- `reported`
- `read_back`
- `reviewed`
- `accepted`
- `blocked`

### ConversationEvent

会话正文事件默认从 Codex rollout 按需读取，不默认持久化到工作台事实层。

UI 使用字段建议：

- `event_id`
- `native_event_id`
- `event_type`
- `actor`
- `role`
- `timestamp`
- `text`
- `tool_name`
- `arguments`
- `output`
- `stdout`
- `stderr`
- `exit_code`
- `warnings`
- `metadata`

## 项目归属规则

项目归属分四级：

1. 用户绑定：用户明确把会话绑定到项目。
2. 工作流绑定：工作流节点创建或引用该会话。
3. 索引推断：会话 cwd / project_root 与项目路径匹配。
4. 未知：无法判断。

UI 必须显示来源。

不能把索引推断直接写成用户绑定。

## 工作流调用规则

工作流节点调用 Agent 会话时，统一走 `AgentSessionService`。

最小流程：

```text
节点生成任务指令
-> 选择或创建 Agent 会话
-> 发送任务
-> 等待结果
-> 读回 transcript
-> 形成节点回传
-> 总指导回收
```

当前已验证的是新建测试会话的初始 prompt，不是 resume 多轮。

所以 v1 UI 里必须区分：

- 可读：已支持。
- 新建测试会话：已探针支持。
- 持续聊天 / resume：待验证。
- 删除 / 移动 / provider：禁用。

## 权限与安全

- 打开会话正文必须由用户选择或工作流授权触发。
- 默认不全量展开所有会话正文。
- 工具输出默认折叠。
- 加密内容继续省略。
- 疑似敏感内容必须 warning。
- 删除、移动、归档会话不进入 v1。
- 向已有业务会话发送消息必须另做能力探针和用户确认。
- 真实自动编排必须先让用户审核阶段方案。

## 与 Codex++ 的关系

吸收：

- 会话列表管理。
- 会话时间线。
- Markdown 导出思路。
- 项目归属和移动的概念提醒。
- Codex 外部增强管理工具的边界意识。

不吸收：

- CDP 注入作为默认路线。
- 会话删除。
- Provider 配置写入。
- 用户脚本注入。
- 直接改 Codex 原始安装目录。

## 实施顺序

建议下一步顺序：

1. 桌面壳只读 Codex 会话中心 v1：会话列表、打开 transcript、时间线、warning。
2. 项目内 Agent 会话入口 v1：项目过滤、绑定已有会话、项目内打开会话。
3. Codex resume / 多轮聊天探针：验证持续对话能力。
4. Codex 新建会话 UI：从 Agent 页和项目页创建项目绑定会话。
5. 工作流节点绑定会话：节点引用会话、读回结果、显示回收摘要。

不建议下一步先做：

- 任务包 ready 流程。
- 删除 / 移动 Codex 会话。
- 多 agent。
- 知识库。
- 复杂画布自动执行。

## 当前权威影响

本决策补充并细化：

- `2026-05-29-codex-session-workflow-route-correction.md`
- `2026-05-29-ui-reference-sources.md`

本决策压低以下方向的优先级：

- dry-run `run.json` 直接做展示 UI。
- 继续做任务包字段和 ready 文件流程。

原因：用户当前要先把 Codex 会话管理和项目内会话入口做清楚。
