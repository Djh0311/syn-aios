# Codex 项目级工作流最小数据模型交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-codex-workflow-min-model.md`
- 开发线：信息架构线
- Evidence：`product-line/evidence/2026-05-28-codex-workflow-min-model.md`
- Decision：`product-line/decisions/2026-05-28-codex-workflow-min-model.md`

## 结论

建议接受为阶段 3 最小工作流模型。

这不是代码实现，不是可编辑自动化工作流完成，也不是多 agent 接入。它定义的是下一步索引内核线、桌面应用线和验证线可以共同依赖的数据模型口径。

## 先说薄弱点

- 当前 UI 骨架只有临时工作流展示，没有正式 `WorkflowNode`、`WorkflowEdge`、`ReviewRecord` 或状态机类型。
- 当前 `codex-index.json` 只能支撑只读派生，不足以保存事实状态。
- 当前 harness 索引是文件级候选，不能完整表达 Codex 专用文件夹式 harness。
- 模型按最终形态预留扩展，但当前功能仍只能交付 Codex 治理。不能把预留字段解释成多 agent 已接入。
- LM 是未来智能核心预留，不是事实核心；如果没有本地状态、handoff、evidence、review 或审计记录，就不能把 LM 输出当事实。

## 做了什么

- 定义阶段 3 最小工作流对象。
- 定义每个对象的最小字段。
- 区分只读索引派生对象和本地工作台状态对象。
- 定义节点类型、边类型、状态枚举和需要用户确认的状态转换。
- 定义 Director、Coder、Reviewer、Tester 在当前 Codex-only 版本里的表达方式。
- 定义任务包、handoff、evidence、review 的关系。
- 明确 Codex 专用文件夹式 harness 应建模为项目能力和验证资源。
- 明确当前索引字段缺口。
- 输出后续索引内核线、桌面应用线、验证线任务建议。

## 最小工作流模型

模型三层：

- 只读索引派生层：从 `codex-index.json` 派生项目、会话、handoff、evidence、harness 候选。
- 本地工作台事实层：保存 workflow、node、edge、state、review、audit、project capability。
- 未来智能建议层：LM 可以提出建议，但不能直接成为事实状态。

最小对象：

- `WorkspaceProject`
- `AgentAdapter`
- `Workflow`
- `WorkflowNode`
- `WorkflowEdge`
- `ActorRole`
- `WorkItem`
- `Artifact`
- `ReviewRecord`
- `ProjectCapability`
- `HarnessResource`
- `AuditEvent`

必须保留或等价表达的扩展字段：

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

当前 Codex-only 填值：

- `agent_type = codex`
- `adapter_id = codex-local`
- `provider = local-codex-index`
- `source_kind = codex_index | workspace_state | project_file | user_input | derived`
- `permission_level = read_only`
- `workflow_version = 1`
- `model_policy = none`

## 对象字段

`WorkspaceProject`：

- `project_id`
- `display_name`
- `root_path`
- `source_kind`
- `permission_level`
- `created_at`
- `updated_at`
- `warnings`

`AgentAdapter`：

- `adapter_id`
- `agent_type`
- `agent_id`
- `display_name`
- `provider`
- `capabilities`
- `status`
- `permission_level`
- `source_kind`

`Workflow`：

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

`WorkflowNode`：

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

`WorkflowEdge`：

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

`ActorRole`：

- `role_id`
- `role_type`
- `display_name`
- `agent_type`
- `adapter_id`
- `source_kind`
- `permission_level`

`WorkItem`：

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

`Artifact`：

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

`ReviewRecord`：

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

`ProjectCapability`：

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

`HarnessResource`：

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

`AuditEvent`：

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

## 节点类型

- `director`
- `actor`
- `session`
- `task`
- `artifact`
- `review`
- `capability`
- `validation`

当前 Codex-only 展示：

- Director：当前由用户/总指导线承担，未来可接 LM 建议。
- Coder：Codex 会话，`agent_type=codex`。
- Reviewer：总指导回收意见或本地 `ReviewRecord`。
- Tester：验证线或 harness 验证入口；当前不自动运行。

## 边类型

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

## 状态枚举

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

UI 映射：

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
- `pending_dispatch -> in_progress`，如果会绑定 agent adapter 或打开执行入口。
- `in_progress -> waiting_user`，如果需要用户决策。
- `pending_review -> accepted`。
- `pending_review -> needs_changes`。
- `pending_review -> paused`。
- 任意状态 -> `superseded`。
- 任意状态 -> `historical`。
- 任意会导致写项目配置、运行 harness、删除、迁移、覆盖文件的转换。

只读派生、不需要确认：

- 会话节点更新时间变化。
- handoff / evidence 候选出现。
- harness 候选出现。
- rollout 路径存在性变化。

但这些只能生成候选或更新只读展示，不能直接改事实状态。

## 任务包、handoff、evidence、review 关系

关系规则：

- Director 或用户创建 `WorkItem`。
- `WorkItem` 可以引用 `Artifact(artifact_type=task_package)`。
- `WorkItem` 通过 `assigned_to` 边指向 Coder / Codex 会话。
- Codex 会话通过 `produces` 边产生 `Artifact(artifact_type=handoff)` 和 `Artifact(artifact_type=evidence)`。
- `ReviewRecord` 通过 `reviews` 边指向 WorkItem 或 Artifact。
- Review 决策通过 `accepts`、`requests_changes`、`pauses` 等边改变后续流转。
- Review 决策必须有本地记录或 review artifact，不能只来自 LM 总结。

## Harness 文件夹问题

结论：

- Codex 专用 harness 应建模为 `ProjectCapability(capability_type=harness)` 和 `HarnessResource`。
- 它既是项目能力，也是验证资源；不应只建成单个资源节点或单个验证节点。
- 文件夹形式 harness 用 `root_path` 表示文件夹根，内部脚本、配置、README、manifest 用 `entrypoints` 和 Artifact 表示。
- 与 Codex 的关系通过 `agent_type=codex`、`adapter_id=codex-local`、`capabilities` 表达。

当前索引风险：

- 当前 `harness_candidates` 是文件级候选。
- 它没有文件夹根。
- 它没有 manifest。
- 它没有版本。
- 它没有来源仓库。
- 它没有功能说明。
- 它没有适配 agent / adapter。
- 它没有命令入口语义。
- 它没有最近验证状态。

因此不能说当前索引已经完整支持文件夹式 Codex harness。

## 字段来源

来自当前 `codex-index.json`：

- 项目根路径：`projects[].project_root`
- 项目更新时间：`projects[].latest_updated_at_ms`
- 项目 warning：`projects[].warnings`、`context_warnings`
- 会话节点：`threads[]`
- 会话标题、编号、项目路径、rollout 路径、更新时间、归档状态：`threads[]`
- Handoff 候选：`projects[].handoff_files[]`
- Evidence 候选：`projects[].evidence_files[]`
- Authority 候选：`projects[].authority_files[]`
- Harness 文件级候选：`projects[].harness_candidates[]`

需要新增本地工作台状态：

- `project_id`
- `workflow_id`
- `workflow_version`
- `node_id`
- `edge_id`
- 节点位置
- 工作流事实状态
- ReviewRecord
- AuditEvent
- 用户确认记录
- AgentAdapter 可用表
- ActorRole
- WorkItem
- ProjectCapability
- HarnessResource 文件夹根和 manifest
- 最近打开 / 最近使用事件
- `model_policy`

不能来自 Codex 内部状态库：

- 工作流事实状态。
- 用户确认写入或运行记录。
- Review 决策。
- ProjectCapability 的人工启用/禁用状态。
- Harness 是否有用、废弃或加强。
- 多 agent adapter 可用性。

## 仍不确定

- 本地工作台状态文件放在哪里。
- 状态文件格式用 JSON、SQLite，还是先 JSON 后 SQLite。
- 文件夹式 harness 的 manifest 规范。
- harness 文件夹扫描的安全边界和最大深度。
- ReviewRecord 是否从 review 文件解析，还是先由本地状态登记。
- LM Director 何时接入建议层。

## 后续任务建议

建议派给索引内核线：

- 补文件夹式 harness 扫描。
- 识别 harness root、manifest、README、entrypoints、version、capabilities、agent_type、adapter_id。
- 保留现有文件级候选，但新增 folder-level `harness_resources` 或等价结构。
- 增加坏 manifest、缺 README、目录过深、权限失败的 warning。

建议派给桌面应用线：

- 接入最小工作流模型类型。
- 项目页从临时拼节点改为读取 `Workflow`、`WorkflowNode`、`WorkflowEdge`。
- 只做 read-only 展示和缺口提示。
- 显示 `source_kind`、`permission_level`、`workflow_version`。
- Harness 看板区分文件级候选和文件夹式 HarnessResource。

建议派给验证线：

- 验证模型展示不读取会话正文。
- 验证不写 `/Users/yoyi/.codex`。
- 验证 `source_kind=codex_index` 的节点只读。
- 验证未接入 agent 不显示可用能力。
- 验证文件夹式 harness 缺字段时显示 warning，而不是显示为完整支持。
- 验证 LM 建议如果出现，只能是建议状态，不能直接改事实状态。

## 风险

- 过度抽象会拖慢当前阶段。
- 继续用当前文件级 harness 候选会漏掉文件夹式 harness。
- 如果没有审计记录，状态转换会变成不可追溯。
- 如果没有 `source_kind` 和 `permission_level`，后续很难证明哪些是只读派生、哪些是用户确认写入。
- 如果把 LM 输出当事实，会破坏工作台可信度。

## 如何满足最终形态预留扩展

- 使用 agent-agnostic 对象名和字段名。
- 当前只填 Codex adapter，不注册其他可用 agent。
- 用 `agent_type`、`agent_id`、`adapter_id` 表达当前和未来 agent。
- 用 `source_kind` 追踪事实来源。
- 用 `permission_level` 表达权限边界。
- 用 `workflow_version` 支持迁移。
- 用 `model_policy` 为未来 LM Director 预留位置，但当前设为 `none`。
- 用 `ProjectCapability` 和 `HarnessResource` 表达 harness，不写死 Codex-only。

## 验收对照

- 有 evidence 和 handoff：已完成。
- 形成正式决策并写入 `product-line/decisions/`：已完成。
- 明确区分事实、假设、缺口和后续任务：已完成。
- 明确说明当前 harness 索引模式风险：已完成。
- 不扩大到多 agent、知识库、向量搜索、模型调度：已遵守。
- 模型按最终形态预留扩展字段：已完成。
- 明确 `source_kind`、`permission_level`、`workflow_version`：已完成。
- 明确 LM 是未来智能核心，不是事实核心：已完成。
- 不要求本轮实现代码：已遵守。
- 不把当前 UI 骨架包装成可编辑工作流完成：已遵守。
- 不把当前 harness 索引包装成已完整支持文件夹式 Codex harness：已遵守。
