# 任务包：桌面壳工作流节点绑定 Codex 会话 v1

## 所属开发线

桌面应用线。

## 关联口径来源

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`

## 后续验证

本任务完成后另派验证线。验证线不是本任务的共同执行线。

## 背景

工作流状态流转 v1 已完成后，项目工作流已经能表达工作项状态推进，但节点还不知道自己对应哪个 Codex 会话。

当前要补的是“工作流节点和 Codex 会话的绑定关系”。这不是自动执行 Codex，而是为后续自动派发、读回、回收建立稳定对象。

依据：

- 用户要求“项目里也能打开单独的 agent 会话”。
- 用户要求工作流最终能由总指导派发，开发线执行，完成后反馈给总指导。
- 现有决策要求 Agent 页、项目页、工作流节点复用同一套 Codex 会话能力。

## 薄弱点

- 本任务不启动真实 Codex，所以完成后仍不能自动开发。依据：`codex resume` 多轮控制仍未验证。
- 当前项目会话归属主要来自索引推断，不等于用户确认绑定。依据：已有决策明确 `index_inferred` 不能直接当作用户事实。
- 如果绑定关系只存在前端 state，后续派发会丢失；所以本任务必须写入工作台自己的状态文件。
- 如果把绑定做成会话管理功能，会偏离当前重点；本任务只服务工作流节点。

## 已知、未知和假设

已知：

- Agent 页面已有只读 Codex 会话中心。
- 项目页面已有 `Agent 会话` 入口，并能按索引推断过滤项目会话。
- 工作流事实层 v0 可写入工作台自己的 JSON 状态。
- 工作项状态流转已经有审计事件。

未知：

- 真实执行阶段，每个节点最终是新建会话、复用已有会话，还是两者都支持。
- 一个工作流节点是否允许绑定多个会话。
- 总指导节点是否必须绑定真实 Codex 会话，还是可以先由用户当前会话承担。

假设：

- v1 只支持“选择已有 Codex 会话并绑定到节点”。
- v1 一个节点只绑定一个主会话。
- v1 不创建新业务会话。
- v1 不发送消息、不 resume、不运行 Codex CLI。
- v1 绑定关系只写工作台状态，不写 Codex 状态库。

## 目标

在项目工作流里实现节点绑定 Codex 会话：

1. 用户能在工作流节点上选择一个已有 Codex 会话。
2. 绑定关系写入工作台自己的状态文件。
3. 节点显示绑定的会话标题、更新时间、项目归属来源、读取状态。
4. 当前工作项能显示自己当前关联的节点会话。
5. 用户能从工作流节点跳转到项目 Agent 会话视图并打开该会话。
6. 用户能解除工作台自己的节点绑定，但不删除 Codex 原始会话。
7. 每次绑定、变更、解除都追加审计事件。

大白话目标：

让工作流里的“总指导 / 开发线 / 回收”节点知道自己对应哪条 Codex 会话。先把对象接上，下一步才谈自动发消息和读回。

## 非目标

- 不启动 Codex CLI。
- 不执行 `codex resume`。
- 不创建真实 Codex 业务会话。
- 不向 Codex 会话发送消息。
- 不自动读取业务会话正文。
- 不运行 harness。
- 不做会话删除、移动、归档。
- 不做 Agent 会话中心 UI v2 精修。
- 不做多会话并发绑定。
- 不做自动派发执行。
- 不写 `/Users/yoyi/.codex`。

## 建议数据模型

在工作台状态里新增或复用工作流节点会话绑定结构。

建议字段：

- `binding_id`
- `project_id`
- `workflow_id`
- `node_id`
- `work_item_id`
- `agent_type`
- `adapter_id`
- `native_thread_id`
- `native_rollout_path`
- `session_title`
- `project_binding_source`
- `binding_source`
- `binding_mode`
- `lifecycle`
- `created_at_ms`
- `updated_at_ms`
- `warnings`

建议枚举：

- `agent_type = codex`
- `adapter_id = codex-local`
- `binding_source = user_bound | workflow_bound | index_inferred`
- `binding_mode = select_existing_session`
- `lifecycle = active | detached`

注意：

- 索引推断只能作为候选来源。
- 用户点击绑定后，才可以写成 `user_bound` 或 `workflow_bound`。
- 不要把完整 transcript 保存进工作台状态。

## 建议后端命令

在 Tauri Rust 后端新增或扩展：

- `bind_workflow_node_codex_session`
- `unbind_workflow_node_codex_session`
- `load_project_workflow_session_bindings`

要求：

- 非索引项目拒绝。
- 缺 workflow 拒绝。
- 缺 node 拒绝。
- 缺 work item 时，如果绑定的是节点级关系可以允许；如果绑定的是工作项关系则拒绝。
- 非索引内会话拒绝。
- 会话 rollout 缺失时允许作为不可读候选展示，但绑定时必须写 warning。
- 写入前备份状态文件。
- 临时文件写入后原子替换。
- 写入后重新读取校验。
- 每次绑定、改绑、解绑写审计事件。

建议审计事件：

- `workflow_node_session_bound`
- `workflow_node_session_rebound`
- `workflow_node_session_unbound`

## 建议前端改动

项目详情页 `工作流` 视图：

- 在节点详情区显示“绑定 Codex 会话”。
- 展示当前绑定状态。
- 提供“选择会话”入口。
- 候选会话优先显示当前项目索引推断会话。
- 允许切换到全部 Codex 会话列表，但必须标出项目归属来源。
- 绑定前走确认弹层。
- 解绑前走确认弹层。
- 绑定后节点卡片显示会话摘要。
- 提供“打开会话”跳转到项目 Agent 会话视图。

项目 Agent 会话视图：

- 如果从工作流节点跳转过来，应自动选中或高亮对应会话。
- 不新增发送消息入口。
- 不新增创建业务会话入口。

确认弹层必须说明：

- 写入的是工作台自己的 workflow state。
- 不写 Codex 状态库。
- 不启动 Codex。
- 不发送消息。
- 不读取完整会话正文，除非用户打开会话。

## 允许读取

允许读取：

- `product-line/decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `product-line/decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `product-line/decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`
- `product-line/tasks/2026-05-29-desktop-shell-workflow-orchestration-min-loop-v1.md`
- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/index-kernel/codex-index.json`

允许读取工作台状态文件的必要结构：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

禁止读取：

- `/Users/yoyi/.codex/auth.json`
- `.env`
- 密钥、token、授权文件内容
- 与本任务无关的业务会话正文

## 允许写入

允许写入项目内：

- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/evidence/2026-05-29-desktop-shell-workflow-node-session-binding-v1.md`
- `product-line/handoffs/2026-05-29-desktop-shell-workflow-node-session-binding-v1-result.md`

允许在用户通过 UI 确认时写入工作台自己的状态文件：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/`

测试优先使用临时目录或夹具，不默认写真实状态文件。

## 禁止事项

- 禁止写 `/Users/yoyi/.codex`。
- 禁止改 Codex 状态库。
- 禁止运行 Codex CLI。
- 禁止运行 `codex resume`。
- 禁止创建真实 Codex 业务会话。
- 禁止向真实 Codex 会话发送消息。
- 禁止自动读取业务会话正文。
- 禁止运行 harness。
- 禁止把索引推断项目归属写成用户确认事实。
- 禁止保存完整 transcript 到工作台状态。
- 禁止删除、移动、归档 Codex 原始会话。

## 验收标准

必须满足：

- 工作流节点能显示绑定状态。
- 能从候选 Codex 会话中绑定一个已有会话到节点。
- 绑定关系写入工作台状态，并有审计事件。
- 能改绑会话，并有审计事件。
- 能解绑会话，并有审计事件。
- 非索引项目、缺 workflow、缺 node、非索引会话会被拒绝。
- 节点显示会话标题、更新时间、项目归属来源、读取状态。
- 能从节点跳到项目 Agent 会话视图并定位到绑定会话。
- 不发送消息、不创建会话、不 resume、不运行 Codex。
- 不写 `.codex` 或 Codex 状态库。

验证命令：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --offline
```

如果改了 Rust，必须跑 Rust 测试。  
如果没有改 Rust，说明为什么没跑。

## 必须回传

回传时必须说明：

- 做了什么。
- 改了哪些文件。
- 新增了哪些 evidence / handoff。
- 是否写了真实 workflow state。
- 如果写了，写入了哪些绑定字段，不要打印完整状态正文。
- 是否写了 `/Users/yoyi/.codex`，答案应为没有。
- 是否运行 Codex CLI，答案应为没有。
- 是否读取业务会话正文，答案应为没有，除非用户打开会话。
- 测试命令和结果。
- 当前仍不能自动执行 Codex 的缺口。

## 总指导回收重点

回收时重点看：

- 绑定关系是否真的落到工作台状态，不只是前端 state。
- 索引推断和用户绑定是否区分清楚。
- 工作流节点是否能打开对应会话。
- 是否保持不发送、不 resume、不启动 Codex 的边界。
- 是否为下一步“工作流节点派发 Codex 指令”留下清楚接口。

