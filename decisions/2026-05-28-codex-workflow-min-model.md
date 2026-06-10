# 决策：阶段 3 最小工作流数据模型

## 结论

阶段 3 最小工作流模型采用 agent-agnostic schema，当前实例只填 Codex。

模型分三层：

1. 只读索引派生层：从 `codex-index.json` 派生项目、会话、handoff、evidence、harness 候选。
2. 本地工作台事实层：保存 workflow、node、edge、state、review、audit、project capability 等事实。
3. 未来智能建议层：LM 可以提出任务拆解、review 建议和风险提示，但不能作为事实核心。

必须保留这些通用字段或等价字段：

- `workflow_version`
- `source_kind`
- `permission_level`
- `agent_type`
- `agent_id`
- `adapter_id`
- `artifact_type`
- `node_type`
- `edge_type`
- `model_policy`

当前 Codex-only 填值规则：

- `agent_type = codex`
- `adapter_id = codex-local`
- `source_kind = codex_index | workspace_state | project_file | user_input | derived`
- `permission_level = read_only | user_confirmed_write | blocked`
- `workflow_version = 1`
- `model_policy = none`，除非后续任务显式接入 LM 建议层

## 依据

- `product-line/STAGE_PLAN.md` 阶段 3 要求项目级可视化编排。
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md` 要求模型和接口按最终形态预留扩展，但当前功能仍 Codex-only。
- `product-line/archive/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md` 明确当前 UI 骨架缺 Director、Review、边关系和状态机数据模型。
- 用户补充当前自制 harness 基本针对 Codex，且以文件夹形式存在；当前 harness 索引模式可能有问题。

## 最小对象

### WorkspaceProject

含义：

- 工作台里的项目，不等同于 Codex 内部 project。

最小字段：

- `project_id`
- `display_name`
- `root_path`
- `source_kind`
- `permission_level`
- `created_at`
- `updated_at`
- `warnings`

当前来源：

- `codex-index.json.projects[].project_root`
- `projects[].latest_updated_at_ms`
- `projects[].warnings`

需要本地状态：

- `project_id`
- `display_name` 人工别名
- `permission_level`

### AgentAdapter

含义：

- 一个可被工作台识别的 agent 接入适配器。

最小字段：

- `adapter_id`
- `agent_type`
- `agent_id`
- `display_name`
- `provider`
- `capabilities`
- `status`
- `permission_level`
- `source_kind`

当前 Codex-only：

- `adapter_id = codex-local`
- `agent_type = codex`
- `display_name = Codex`
- `provider = local-codex-index`
- `status = available`

未接入 agent：

- 只能作为 UI 空白位或未来 registry 候选，不进入可用 adapter 表。

### Workflow

含义：

- 一个项目内的可视化工作流实例。

最小字段：

- `workflow_id`
- `workflow_version`
- `project_id`
- `title`
- `state`
- `source_kind`
- `permission_level`
- `created_at`
- `updated_at`
- `model_policy`

当前规则：

- 一个项目可以先有一个默认 workflow。
- `model_policy = none`。
- 状态事实来自本地工作台状态，不来自 Codex 内部状态库。

### WorkflowNode

含义：

- 画布节点。

最小字段：

- `node_id`
- `workflow_id`
- `node_type`
- `title`
- `state`
- `source_kind`
- `source_ref`
- `agent_type`
- `adapter_id`
- `artifact_type`
- `permission_level`
- `position`
- `warnings`

节点类型：

- `director`
- `actor`
- `session`
- `task`
- `artifact`
- `review`
- `capability`
- `validation`

当前 UI 对应：

- Director 节点：`node_type=director`，当前缺真实拆解数据。
- Codex 会话节点：`node_type=session`，`agent_type=codex`，`source_kind=codex_index`。
- 任务包节点：`node_type=task`，当前需要后续解析任务包文件或本地状态。
- Handoff / Evidence 节点：`node_type=artifact`。
- Review 节点：`node_type=review`，当前需要本地状态。
- Harness 文件夹：优先建为 `node_type=capability`，其验证入口再建 `node_type=validation`。

### WorkflowEdge

含义：

- 节点之间的关系和流转。

最小字段：

- `edge_id`
- `workflow_id`
- `from_node_id`
- `to_node_id`
- `edge_type`
- `state`
- `source_kind`
- `permission_level`
- `created_at`
- `updated_at`
- `warnings`

边类型：

- `decomposes_to`
- `assigned_to`
- `produces`
- `supports`
- `validates`
- `reviews`
- `accepts`
- `requests_changes`
- `pauses`
- `blocks`
- `depends_on`

### ActorRole

含义：

- 工作流中的职责角色，不绑定具体 agent。

最小字段：

- `role_id`
- `role_type`
- `display_name`
- `agent_type`
- `adapter_id`
- `source_kind`
- `permission_level`

角色类型：

- `director`
- `coder`
- `reviewer`
- `tester`

当前 Codex-only 表达：

- Director：当前由用户/总指导线承担，未来可由 LM Director 建议，但状态仍落本地。
- Coder：Codex 会话。
- Reviewer：总指导回收意见或本地 review 记录。
- Tester：验证线或 harness 验证入口；当前不自动运行。

### WorkItem

含义：

- 任务包或工作单元。

最小字段：

- `work_item_id`
- `project_id`
- `workflow_id`
- `title`
- `state`
- `source_kind`
- `source_ref`
- `assigned_role_id`
- `agent_type`
- `adapter_id`
- `permission_level`
- `created_at`
- `updated_at`

当前来源：

- 可从 `product-line/tasks/*.md` 候选解析。
- 当前 `codex-index.json` 不提供完整任务包状态机。

### Artifact

含义：

- handoff、evidence、review、任务包、日志元数据、harness manifest 等可引用产物。

最小字段：

- `artifact_id`
- `artifact_type`
- `project_id`
- `path`
- `title`
- `source_kind`
- `source_ref`
- `permission_level`
- `created_at`
- `updated_at`
- `warnings`

Artifact 类型：

- `task_package`
- `handoff`
- `evidence`
- `review`
- `session_log_metadata`
- `harness_manifest`
- `harness_entry`
- `authority_file`

当前来源：

- `projects[].handoff_files`
- `projects[].evidence_files`
- `projects[].authority_files`
- `threads[].rollout_path` 只能作为日志元数据路径，不读取正文。

### ReviewRecord

含义：

- 对工作项或产物的回收判断。

最小字段：

- `review_id`
- `target_ref`
- `decision`
- `state`
- `reviewer_role_id`
- `source_kind`
- `artifact_ref`
- `permission_level`
- `created_at`
- `updated_at`

决策值：

- `accepted`
- `needs_changes`
- `paused`
- `rejected`
- `unknown`

### ProjectCapability

含义：

- 项目拥有的能力或资源，harness 应优先落在这里。

最小字段：

- `capability_id`
- `project_id`
- `capability_type`
- `display_name`
- `source_kind`
- `source_ref`
- `agent_type`
- `adapter_id`
- `permission_level`
- `version`
- `status`
- `warnings`

能力类型：

- `harness`
- `skill`
- `validation`
- `build`
- `test`
- `runtime`

### HarnessResource

含义：

- 文件夹式或文件式 harness 的规范化资源。

最小字段：

- `harness_id`
- `project_id`
- `root_path`
- `display_name`
- `harness_kind`
- `version`
- `source_kind`
- `source_ref`
- `agent_type`
- `adapter_id`
- `capabilities`
- `entrypoints`
- `permission_level`
- `warnings`

建模结论：

- Codex 专用 harness 是项目能力节点和验证资源，不是单纯资源节点，也不是单纯验证节点。
- 文件夹形式 harness 应以文件夹根为 `root_path`，内部脚本、配置、README、manifest 作为 entrypoints / artifact。
- 当前如果没有 manifest，只能标为 `source_kind=project_file` 或 `derived` 的候选，不能标为完整支持。

### AuditEvent

含义：

- 本地事实变化和用户确认记录。

最小字段：

- `event_id`
- `event_type`
- `target_ref`
- `actor_ref`
- `source_kind`
- `permission_level`
- `before_state`
- `after_state`
- `created_at`
- `reason`

用途：

- 状态转换、用户确认、写入预览、写入执行、拒绝操作都要留审计记录。

## 状态枚举

最小状态：

- `draft`
- `pending_dispatch`
- `in_progress`
- `waiting_user`
- `pending_review`
- `accepted`
- `needs_changes`
- `paused`
- `blocked`
- `superseded`
- `historical`
- `unknown`

对应 UI 口径：

- 待派发：`pending_dispatch`
- 执行中：`in_progress`
- 等待用户：`waiting_user`
- 待回收：`pending_review`
- 已接受：`accepted`
- 需修改：`needs_changes`
- 暂停：`paused`

## 需要用户确认的状态转换

必须确认：

- `draft -> pending_dispatch`，如果会生成或登记任务包。
- `pending_dispatch -> in_progress`，如果会绑定具体 agent adapter 或打开执行入口。
- `in_progress -> waiting_user`，如果需要用户决策。
- `pending_review -> accepted`。
- `pending_review -> needs_changes`。
- `pending_review -> paused`。
- 任意状态 -> `superseded`。
- 任意状态 -> `historical`。
- 任意会导致写项目配置、运行 harness、删除、迁移、覆盖文件的转换。

可以由索引只读派生、不需要确认：

- 会话节点最近更新时间变化。
- handoff / evidence 候选出现。
- harness 候选出现。
- rollout 路径存在性变化。

但这些派生变化只能更新只读展示或产生候选，不应直接改工作流事实状态。

## source_kind

允许值：

- `codex_index`
- `workspace_state`
- `project_file`
- `task_package`
- `handoff`
- `evidence`
- `review`
- `user_input`
- `lm_suggestion`
- `derived`

规则：

- `codex_index` 只读，不写回。
- `workspace_state` 是本地工作台事实状态。
- `lm_suggestion` 只能生成建议，不改变事实状态。
- `derived` 必须能追溯到来源字段。

## permission_level

允许值：

- `read_only`
- `user_confirmed_write`
- `user_confirmed_run`
- `blocked`

当前阶段默认：

- `read_only`

阶段 4 或后续才允许：

- `user_confirmed_write`
- `user_confirmed_run`

禁止：

- 不经确认直接写入。
- 不经确认运行 harness。
- 写 Codex 内部状态库。

## model_policy

允许值：

- `none`
- `suggest_only`
- `propose_state_change`

当前阶段：

- `none`

未来 LM Director：

- 可以进入 `suggest_only` 或 `propose_state_change`。
- 但事实状态必须由本地记录和用户确认落地。

## 当前 Codex-only 映射

当前实例：

- Project：来自 `codex-index.json.projects[]`
- AgentAdapter：只注册 `codex-local`
- Session Node：来自 `codex-index.json.threads[]`
- Artifact：来自 `authority_files`、`handoff_files`、`evidence_files`、rollout 元数据
- HarnessResource：从 `harness_candidates` 临时派生，标记缺字段
- ReviewRecord：来自后续本地状态或 review 文件，不从 Codex 内部状态库推断

不允许：

- 从 Codex 会话正文推断状态。
- 从 LM 总结直接改状态。
- 从 Codex 内部状态库写回项目工作流。

## 文件夹式 harness 决策

当前建模：

- 文件夹式 harness 是 `ProjectCapability(capability_type=harness)`。
- 其文件夹根是 `HarnessResource.root_path`。
- 文件夹内脚本、配置、README、manifest 是 `Artifact` 或 `entrypoints`。
- 与 Codex 的关系通过 `agent_type=codex`、`adapter_id=codex-local`、`capabilities` 表达。

不能建模为：

- 单个 `harness_candidate` 文件。
- Codex 专属字段。
- 已完整支持的仓库。

后续索引内核需要补：

- 识别 harness 文件夹根。
- 识别 manifest 或 README。
- 识别版本。
- 识别入口脚本和命令。
- 识别适配 agent / adapter。
- 识别功能和使用场景的候选来源。

## 长期预留但当前不实现

长期预留：

- 多 agent adapter。
- LM Director。
- 知识库作为 artifact / context provider。
- harness 多仓库、多版本。
- 模型辅助 review 建议。
- 可控写入和可控运行。

当前不实现：

- 非 Codex agent 接入。
- 个人知识库正文入库。
- 向量搜索。
- 模型调度。
- 自动运行 harness。
- 写 Codex 状态库。
- release 打包。
