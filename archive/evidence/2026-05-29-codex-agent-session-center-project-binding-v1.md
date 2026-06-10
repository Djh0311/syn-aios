# Codex 会话中心与项目内 Agent 会话架构 v1 证据

## 薄弱点

- 本轮只定义信息架构和数据关系，没有改前端、后端或索引代码。
- 没有重新审计 Codex++ 源码。依据：本轮只读取本地已保存的参考源决策和路线纠偏文档。
- 会话持续聊天仍未验证。依据：已有探针只验证 `codex exec` 新建测试会话，没有验证 `resume`。
- 项目内会话具体布局还需要 UI 原型验证。依据：本轮只定“复用同一套会话组件”，没有做交互稿或实现。

## 做了什么

- 明确 Agent 页面是全局 Codex 会话中心。
- 明确项目页面也能打开单独 Agent 会话。
- 明确项目页不能另做第二套聊天系统，必须复用 Agent 页同一套会话能力。
- 明确工作流节点通过会话绑定引用 Codex 会话。
- 定义 `AgentAdapter`、`AgentSession`、`ProjectAgentSessionBinding`、`WorkflowNodeSessionBinding`、`ConversationEvent` 的字段建议。
- 明确项目归属来源分为用户绑定、工作流绑定、索引推断、未知。
- 明确当前能力分级：可读、可新建测试会话、resume 待验证、删除移动禁用。

## 依据

- 用户要求：“项目里也能打开单独的agent会话”。
- 用户要求：“Agent 页面要做到和 Codex 的原生体验”。
- 用户要求：“项目界面也能调用到工作台的 agent 功能”。
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md` 已明确当前主线是 Codex 会话管理和工作流编排，不是任务包管理器。
- `product-line/decisions/2026-05-29-ui-reference-sources.md` 已明确 Codex++ 作为 Codex 会话管理参考源，但不照搬删除、provider 写入和 CDP 注入路线。
- `product-line/handoffs/2026-05-29-codex-workflow-orchestration-runtime-model-v1-result.md` 显示当前工作流运行模型只做到 dry-run，不适合直接进入真实自动派发 UI。
- `product-line/handoffs/2026-05-29-codex-controlled-real-session-write-probe-v2-review.md` 显示新建测试会话和读回链路已打通，但 resume / 多轮 / 并发未验证。

## 关键判断

### 不能做两套聊天系统

项目页要能打开 Agent 会话，但不能在项目页重新实现一套独立聊天系统。

依据：

- Agent 页和项目页如果分别实现打开、发送、读回，会出现状态分叉。
- 工作流节点后续也要调用会话能力，如果每个页面各自实现，节点绑定会很难维护。

结论：

- 底层统一为 `AgentSessionService`。
- UI 统一复用 Codex 会话组件。
- 项目页只是加上项目过滤、项目绑定和节点引用上下文。

### 项目归属必须显示来源

会话属于项目有不同依据。

结论：

- 用户手动绑定是事实。
- 工作流节点创建或引用是事实。
- 索引根据 cwd / project_root 推断只是候选。
- 无法判断就是未知。

UI 必须显示来源，不能把候选归属显示成已确认事实。

### Codex++ 只作为体验参考

Codex++ 对会话列表、时间线、导出、项目归属有参考价值。

但当前不能直接照搬：

- 会话删除。
- provider 写入。
- 用户脚本注入。
- CDP 注入作为唯一路线。

依据：已有参考源决策已经记录这些边界。

## 产物

新增正式决策：

- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`

新增证据：

- `product-line/evidence/2026-05-29-codex-agent-session-center-project-binding-v1.md`

新增交接：

- `product-line/handoffs/2026-05-29-codex-agent-session-center-project-binding-v1-result.md`

## 未做

- 没有实现桌面壳 UI。
- 没有新增 Rust / React 代码。
- 没有读取新的 Codex 会话正文。
- 没有写 `/Users/yoyi/.codex`。
- 没有执行 Codex CLI。
- 没有运行 harness。
- 没有改任务队列。

## 后续建议

下一步建议写任务包并派给桌面应用线：

```text
Codex 会话中心只读 UI v1
```

目标：

- 桌面壳 Agent 页显示 Codex 会话列表。
- 可以打开单个会话 transcript。
- 展示用户消息、assistant 消息、工具调用、工具结果、命令输出和 warning。
- 不做发送消息。
- 不做删除、移动、归档。

再下一步：

```text
项目内 Agent 会话入口 v1
```

目标：

- 项目页显示该项目关联会话。
- 项目页可以打开单独会话。
- 支持绑定已有会话到项目。
- 工作流节点能引用会话。
