# Workflow And Task Package Design v1

状态：最终蓝图下的工作流和任务包设计草案。

本文从最终蓝图倒推工作流和任务包的完整形态，不替代 `CURRENT.md`、`README.md`、`decisions/**` 或最终蓝图。

当前实现状态：

- `workflow-task-package-design-v1` Task 0-12 已完成一轮保守工程落地：兼容读模型、运行前检查、任务包字段、任务包 stale 规则、工作流账本、子智能体汇报、审查结果、异常、状态机、项目主管完成闸门、接口边界、只读工作流画布和端到端验收场景展示。
- 已完成范围以 `evidence/2026-06-01-workflow-task-package-plan-baseline.md`、`evidence/2026-06-01-workflow-task-package-design-v1-execution.md`、`handoffs/2026-06-01-workflow-task-package-task4-6-user-test.md`、`handoffs/2026-06-01-workflow-task-package-design-v1-execution-result.md` 为依据。
- 不能把这轮落地说成真实业务自动编排完成，也不能说成真实 Tauri 窗口完整验收完成。Task 7-12 截图是 Chrome headless + 只读 mock。

最终蓝图来源：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

相关设计来源：

- `/Users/yoyi/workspace/product-line/docs/memory-layer-design-v1.md`
- `/Users/yoyi/workspace/product-line/docs/tooling-and-mcp-registry.md`

## 1. 当前结论

工作流不是普通待办列表。

工作流是项目主管组织多角色、多会话、多工具完成项目目标的执行编排。

任务包不是产品主界面。

任务包是项目主管派给子智能体的内部协议，负责限定目标、权限、资料、模型、工具、验收和汇报格式。

工作流负责：

- 把建议方案变成可执行流程。
- 记录每个节点的负责人、状态、权限、证据和风险。
- 帮项目主管派发任务包。
- 接住子智能体汇报和审查结果。
- 让用户看见进度、风险、待确认和最终汇报。

工作流不负责：

- 替代建议方案。
- 替代项目主管判断。
- 替代记忆层。
- 替代审计中心。
- 让子智能体直接找用户决策。
- 让工具调用结果全文进入工作流账本。

## 2. 硬边界

- 用户默认看建议方案、进度、风险和汇报，不默认看完整后台执行计划。
- 子智能体只接收项目主管派发的任务包。
- 子智能体不能直接向用户提建议。
- 子智能体不能自己标记任务完成。
- 子智能体汇报进入工作流账本，不直接进入项目记忆。
- 项目主管最终标记节点完成。
- 权限使用在项目主管派发任务包时明确规定。
- 审查智能体不查权限使用。
- harness 可以影响工作流，但 harness 不是普通工具节点。
- 工具调用结果不进入工作流账本全文，只保留摘要、状态和审计链接。

## 3. 核心对象

### 3.1 Workflow

项目工作流。

字段草案：

- `workflow_id`
- `project_id`
- `title`
- `source_proposal_id`
- `status`
- `view_mode`
- `created_by_role`
- `owner_role`
- `current_stage`
- `run_check_status`
- `risk_level`
- `created_at`
- `updated_at`

状态候选：

- `draft`
- `ready`
- `running`
- `paused`
- `waiting_decision`
- `completed`
- `failed`
- `archived`

### 3.2 WorkflowNode

工作流节点。

主节点类型：

- `consultation`
- `director`
- `subagent`
- `review`
- `report`

默认不作为主节点的内容：

- 人工确认。
- 知识库读取。
- 工具调用。
- 普通权限读取。

这些内容进入节点详情、账本、审计或任务包。

字段草案：

- `workflow_node_id`
- `workflow_id`
- `node_type`
- `title`
- `assigned_role`
- `assigned_session_id`
- `status`
- `task_package_id`
- `depends_on`
- `harness_requirements`
- `review_requirements`
- `acceptance_criteria`
- `created_at`
- `updated_at`

节点状态：

- `not_started`
- `waiting`
- `running`
- `waiting_permission`
- `waiting_decision`
- `reviewing`
- `passed`
- `returned`
- `failed`
- `skipped`
- `paused`

### 3.3 WorkflowRunCheck

运行性检查。

用途：

- 在工作流运行前检查是否可运行。
- 在运行中修改后重新检查。

状态：

- `runnable`
- `warning`
- `blocked`

检查内容：

- 是否有负责人。
- 是否有可用会话。
- 是否指定模型。
- 是否缺少权限。
- 是否违反项目策略。
- 是否缺少 harness 要求。
- 是否缺少验收标准。
- 是否有未处理冲突。

### 3.4 TaskPackage

任务包。

用途：

- 给子智能体提供最小必要上下文和明确执行边界。

字段草案：

- `task_package_id`
- `workflow_id`
- `workflow_node_id`
- `project_id`
- `target_session_id`
- `target_role`
- `task_goal`
- `allowed_read_scope`
- `allowed_write_scope`
- `available_skills`
- `available_knowledge_refs`
- `available_memory_refs`
- `forbidden_actions`
- `acceptance_criteria`
- `report_format`
- `timeout_policy`
- `failure_policy`
- `callable_tool_capabilities`
- `model_id`
- `harness_requirements`
- `created_by`
- `created_at`

任务包可以包含：

- 任务目标。
- 所属项目。
- 负责会话。
- 允许读写范围。
- 可用技能。
- 可用知识库。
- 禁止事项。
- 验收标准。
- 汇报格式。
- 超时或失败处理。
- 可调用工具能力。
- 指定模型。

### 3.5 WorkflowLedgerEntry

工作流账本记录。

用途：

- 保存工作流执行过程的结构化事实。
- 作为项目主管总结、审查和记忆候选的来源。

字段草案：

- `ledger_entry_id`
- `workflow_id`
- `workflow_node_id`
- `entry_type`
- `actor_role`
- `actor_session_id`
- `summary`
- `source_refs`
- `tool_call_refs`
- `audit_refs`
- `risk_flags`
- `created_at`

账本记录类型：

- `task_package_created`
- `subagent_started`
- `permission_requested`
- `permission_granted`
- `permission_denied`
- `tool_call_summary`
- `subagent_report`
- `review_result`
- `node_returned`
- `node_failed`
- `node_passed`
- `director_summary`
- `user_decision`

硬规则：

- 工作流账本不是审计中心。
- 工作流账本不保存工具调用全文。
- 工作流账本可以引用审计记录。
- 工作流账本可以生成记忆候选，但不直接写正式记忆。

### 3.6 SubagentReport

子智能体结构化汇报。

必须包含：

- 执行了什么。
- 改了什么。
- 证据是什么。
- 遇到什么问题。
- 是否需要更多权限或资料。
- 是否认为方向可能错误。
- 后续建议。
- 是否满足验收标准。

字段草案：

- `subagent_report_id`
- `task_package_id`
- `workflow_node_id`
- `session_id`
- `completion_claim`
- `changes_summary`
- `evidence_refs`
- `open_issues`
- `permission_requests`
- `direction_risk`
- `follow_up_suggestions`
- `acceptance_status`
- `created_at`

### 3.7 ReviewResult

审查结果。

字段草案：

- `review_result_id`
- `workflow_node_id`
- `reviewer_session_id`
- `status`
- `findings`
- `evidence_refs`
- `return_reason`
- `created_at`

状态候选：

- `passed`
- `returned`
- `failed`
- `not_required`

### 3.8 WorkflowException

异常通知来源。

触发：

- 子智能体超时。
- 子智能体失败。
- 审查反复不通过。
- 权限长期等待。
- 方向风险未处理。
- harness 阻断。

字段草案：

- `exception_id`
- `workflow_id`
- `workflow_node_id`
- `exception_type`
- `summary`
- `severity`
- `suggested_action`
- `created_at`

## 4. 工作流生命周期

### 4.1 从建议方案生成工作流

流程：

1. 用户采纳建议方案。
2. 全局主管或项目主管拆分后续动作。
3. 生成工作流草案。
4. 项目主管补齐节点、负责人、验收标准、权限和 harness 要求。
5. 运行运行性检查。
6. 可运行后进入 `ready`。

### 4.2 运行前检查

运行前必须检查：

- 节点是否有负责人。
- 会话是否可用。
- 模型是否可用。
- 权限是否满足。
- 任务包是否完整。
- 验收标准是否存在。
- harness 要求是否满足。
- 是否有待确认风险。

检查结果：

- `runnable`：可以运行。
- `warning`：可运行但有风险。
- `blocked`：不可运行。

### 4.3 派发任务包

流程：

1. 项目主管选择节点。
2. 系统生成任务包草案。
3. 系统执行权限、记忆、知识库、模型、工具检查。
4. 项目主管确认任务包。
5. 任务包派发给目标会话。
6. 工作流账本记录任务包创建。
7. 审计记录任务包派发和权限范围。

### 4.4 子智能体执行

规则：

- 子智能体只能按任务包执行。
- 子智能体需要更多权限或资料时，请求发给项目主管。
- 子智能体发现方向可能错误时，反馈给项目主管。
- 子智能体不能直接找用户。
- 子智能体不能自己标记完成。

### 4.5 待决策

触发：

- 子智能体反馈方向可能错误。
- 任务目标和新证据冲突。
- 权限或私密内容风险超出项目主管权限。
- 需要改变工作流方向。

流程：

1. 相关节点进入 `waiting_decision`。
2. 暂停该节点后续执行。
3. 不暂停整个项目。
4. 项目主管生成建议方案。
5. 用户或对应主管确认后继续。

### 4.6 审查

审查智能体可以选择性创建。

可不创建审查智能体的情况：

- 项目启用的 harness 已经覆盖足够验证。
- 当前节点风险低。
- 项目主管判断不需要额外审查。

审查规则：

- harness 只能替代一部分审查。
- harness 不能完全替代项目主管判断。
- 审查智能体不查权限使用。
- 审查不合格后退回原子智能体。
- 反复不合格时项目主管介入。

### 4.7 完成判定

子智能体、审查智能体、项目主管三者都启用时：

1. 子智能体汇报完成。
2. 审查智能体通过并汇报完成。
3. 项目主管最终标记完成。

项目主管必须检查：

- 任务目标是否完成。
- 验收标准是否满足。
- 证据是否足够。
- 审查或 harness 是否通过。
- 是否有未处理风险。
- 是否需要生成记忆候选。
- 是否需要生成用户汇报。

### 4.8 工作流结束

工作流结束后：

1. 项目主管做总结汇报给用户。
2. 工作流账本保留结构化记录。
3. 生成必要的记忆候选。
4. 生成异常通知或待办。
5. 关键动作写审计。

一轮工作结束的定义：

- 跑完用户定的工作目标。

## 5. 状态流转

### 5.1 Workflow 状态流转

允许流转：

- `draft -> ready`
- `ready -> running`
- `running -> paused`
- `paused -> running`
- `running -> waiting_decision`
- `waiting_decision -> running`
- `running -> completed`
- `running -> failed`
- `completed -> archived`
- `failed -> archived`

### 5.2 WorkflowNode 状态流转

允许流转：

- `not_started -> waiting`
- `waiting -> running`
- `running -> waiting_permission`
- `waiting_permission -> running`
- `running -> waiting_decision`
- `waiting_decision -> running`
- `running -> reviewing`
- `reviewing -> passed`
- `reviewing -> returned`
- `returned -> running`
- `running -> failed`
- `running -> paused`
- `paused -> running`
- `waiting -> skipped`

硬规则：

- `passed` 不等于工作流完成。
- 子智能体不能把节点改成 `passed`。
- `waiting_decision` 不能自动恢复。
- `failed` 后由项目主管选择重试、退回、换会话或结束。

## 6. 画布和界面

画布是工作流的可视化界面。

画布风格是工程编排图，不做装饰型脑图。

### 6.1 视图模式

画布至少支持：

- 方案视图。
- 运行状态视图。

规则：

- 默认可以先看方案视图。
- 运行后可以切到运行状态视图。
- 用户可以切回方案视图。

### 6.2 手动编排

手动编排支持：

- 拖拽节点。
- 步骤列表细调。

运行中可以暂停后修改。

修改后必须重新运行运行性检查。

### 6.3 主节点

主节点包括：

- 咨询。
- 主管。
- 子智能体。
- 审查。
- 汇报。

默认不显示成主节点：

- 人工确认。
- 知识库读取。
- 工具调用。
- 普通权限读取。

### 6.4 项目规则状态条

画布顶部显示：

- 当前项目使用的 harness。
- 运行性检查状态。
- 违反规则数量。
- 证据完整度。

harness 通过项目规则状态条和运行性检查影响画布。

### 6.5 节点详情

点击节点后，右侧打开详情面板。

不使用弹窗打断画布操作。

节点详情显示：

- 知识库权限。
- 工具权限。
- 模型。
- 技能。
- 验收标准。
- 审查要求。
- harness 要求。
- 任务包。
- 汇报。
- 账本记录。
- 审计链接。

## 7. 和其他系统的边界

### 7.1 和建议方案

建议方案是用户决策界面。

工作流是执行编排。

规则：

- 高风险方向变化先生成建议方案。
- 用户采纳建议后，可以生成或修改工作流。
- 工作流不能绕过建议方案直接改变用户确认过的方向。

### 7.2 和待办

待办是用户和角色需要跟进的事项。

工作流是多节点执行过程。

关系：

- 轻待办可以不生成工作流。
- 执行待办可以生成工作流。
- 跟踪待办可以绑定已有工作流或项目阶段。

### 7.3 和记忆层

工作流账本是记忆候选来源。

规则：

- 子智能体汇报先进入工作流账本。
- 审查或项目主管总结后，再生成记忆候选。
- 工作流不会直接写正式记忆。
- 任务包可以引用正式记忆。
- 任务包结束后不自动变成长期记忆。

### 7.4 和知识库

知识库是材料和思考空间。

工作流只引用任务包允许的知识库材料。

规则：

- 知识库读取不作为主画布节点。
- 知识库权限放在节点详情和任务包中。
- agent 需要更多资料时，向项目主管请求。

### 7.5 和审计

审计中心是事实账本。

工作流账本是执行过程记录。

必须写审计：

- 工作流创建。
- 运行性检查。
- 任务包派发。
- 权限变更。
- 节点完成。
- 节点失败。
- 节点退回。
- harness 阻断。
- 方向变化建议。
- 用户确认。

### 7.6 和 harness

harness 是项目级工作协议。

工作流不能把 harness 当普通工具节点。

harness 可以：

- 提供任务包模板。
- 提供验证要求。
- 提供汇报格式。
- 影响运行性检查。
- 影响完成判定。
- 生成审查或证据要求。

未定：

- 多个 harness 冲突如何处理。
- harness 失败是阻断、警告还是建议。
- harness 输出在界面里如何呈现。
- harness 是否以及如何访问项目知识库。

### 7.7 和工具

工具是可调用能力。

规则：

- 工具按具体能力授权。
- 工具权限进入任务包。
- 工具调用结果不进工作流账本全文。
- 节点详情可以保留调用摘要、失败状态和审计链接。
- 工具调用失败后交给项目主管判断。

## 8. 权限规则

权限采用白名单。

任务包必须明确：

- 允许读什么。
- 允许写什么。
- 可用知识库材料。
- 可用工具能力。
- 可用技能。
- 可用模型。
- 禁止事项。

规则：

- 没写允许，就是不允许。
- 子智能体请求权限必须找项目主管。
- 项目主管只能在自己权限范围内批准。
- 项目主管不能临时放宽项目策略。
- 超出项目主管权限时，生成建议方案或用户确认项。

## 9. 失败和异常处理

### 9.1 子智能体失败

项目主管可以：

- 重试。
- 退回重做。
- 换新会话。
- 生成建议方案。
- 标记节点失败。

### 9.2 超时

超时后：

- 节点进入异常或失败状态。
- 项目主管收到通知。
- 旧会话保留为异常会话。
- 项目主管可换新会话处理低风险问题。

### 9.3 方向可能错误

子智能体反馈方向可能错误后：

- 项目主管不能直接改方向。
- 相关节点进入 `waiting_decision`。
- 项目主管生成建议方案。
- 用户或对应主管确认后继续。

### 9.4 审查反复不合格

如果反复不合格：

- 项目主管介入。
- 可以退回重做。
- 可以换会话。
- 可以暂停工作流。
- 可以生成建议方案。

## 10. 验收场景

### 10.1 子智能体不能越过项目主管

场景：

- 子智能体发现方向可能错误。

正确表现：

- 子智能体反馈给项目主管。
- 项目主管生成建议方案。
- 用户或对应主管确认后继续。

失败表现：

- 子智能体直接找用户改方向。
- 子智能体直接改工作流方向。

### 10.2 任务包限制上下文

场景：

- 项目主管派发任务给子智能体。

正确表现：

- 子智能体只看到任务包允许的记忆、知识库、工具、技能和模型。
- 任务包记录允许读写范围和禁止事项。

失败表现：

- 子智能体能扫整个项目资料。
- 子智能体能调用未授权工具。

### 10.3 汇报不直接进记忆

场景：

- 子智能体完成任务并汇报。

正确表现：

- 汇报进入工作流账本。
- 项目主管总结后生成记忆候选。
- 正式记忆仍需确认流程。

失败表现：

- 子智能体汇报直接写入项目记忆。

### 10.4 审查不能替代项目主管

场景：

- 审查智能体通过。

正确表现：

- 项目主管仍需最终标记完成。

失败表现：

- 审查通过后节点自动完成。

### 10.5 harness 不是普通节点

场景：

- 项目启用 harness。

正确表现：

- harness 影响运行性检查、任务包模板和完成判定。
- 默认不作为主画布节点。

失败表现：

- harness 被当作普通工具调用节点。

## 11. 落地状态和仍缺什么

本节已从原始“全部需要补”更新为当前状态说明。

已落地的是保守读模型、测试和只读 UI 草案；仍缺的是正式产品能力、真实运行闭环和跨模块完整接口。

### 11.1 数据 schema

已补保守读模型和前端类型：

- `Workflow`
- `WorkflowNode`
- `WorkflowRunCheck`
- `TaskPackage`
- `WorkflowLedgerEntry`
- `SubagentReport`
- `ReviewResult`
- `WorkflowException`

仍缺：

- 正式持久化 schema。
- SQLite 迁移。
- 和审计中心、知识库、记忆层、工具注册中心的真实表结构和写入流程。

### 11.2 任务包生成算法

已补保守规则和展示：

- 输入。
- 权限过滤。
- 记忆引用占位。
- 知识库引用占位。
- 工具能力选择。
- 模型选择。
- harness 模板应用。
- 输出格式。

仍缺：

- 真实记忆召回算法。
- 真实知识库权限接口。
- 真实工具注册中心。
- token 超限裁剪。
- 入选和排除原因审计。
- 真实派发前的可操作确认流。

### 11.3 状态机实现

已补纯函数和测试：

- Workflow 状态机。
- WorkflowNode 状态机。
- 项目主管完成闸门。

仍缺：

- 会话状态和节点状态的完整实时映射。
- 异常恢复 UI。
- 真实多角色运行中的暂停、重试、取消、权限等待和超时处理。

### 11.4 UI 详细规格

已补只读草案展示：

- 方案视图。
- 运行状态视图。
- 节点详情。
- 任务包预览。
- 账本面板。
- 审查结果面板。
- 异常通知。

仍缺：

- 真实 Tauri 窗口完整验收。
- 可编辑画布正式交互。
- 与当前水墨 UI 的最终布局一致性。
- 右侧通知中心、待办中心、运行中工作流和节点详情的完整产品化交互。

### 11.5 和其他模块的接口

已补保守 stub 或边界说明：

- 建议方案接口。
- 记忆层接口。
- 知识库权限接口。
- 工具注册中心接口。
- 模型池接口。
- harness 接口。
- 审计中心接口。

仍缺：

- 真实建议方案对象。
- 真实记忆候选写入和正式记忆确认。
- 真实知识库/Obsidian-compatible 集成。
- 真实工具注册中心和模型池。
- harness 真实运行和输出展示。
- 审计中心正式查询和回滚建议方案。

## 12. 当前未定

- 多个 harness 冲突处理。
- harness 失败是阻断、警告还是建议。
- harness 输出在界面里的详细呈现。
- harness 是否以及如何访问项目知识库。
- 工具调用摘要在节点详情中保留多久。
- 任务包是否允许用户手动编辑。
- 任务包模板和 harness 模板如何合并。
- 工作流修改后，已派发任务包是否自动失效。
- 工作流账本和审计中心的引用粒度。
