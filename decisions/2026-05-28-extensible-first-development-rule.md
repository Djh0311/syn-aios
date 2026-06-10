# 决策：按最终形态预留扩展的开发规则

## 结论

后续开发规则调整为：一切以最终版本的状态去设计架构和数据模型，保持高扩展性；当前功能交付仍按阶段目标收敛。

换成大白话：

- 数据模型、接口、权限边界、状态机不要写死成 Codex-only。
- 当前实现和验收仍只做 Codex 治理。
- 不因为暂时只做 Codex，就把未来多 agent、LM Director、知识库、harness 仓库化等方向堵死。
- 也不因为要预留最终形态，就提前实现多 agent、知识库、向量搜索或模型调度。

## 依据

- 用户明确要求：“确定开发规则，一切以最终版本的状态去开发，保持高扩展性”。
- 当前产品长期方向是本地 agent 管理工作台，并可能演进为个人知识库和超级工作台。
- 当前阶段目标仍是治理 Codex，不能被长期范围反向污染。
- 当前已进入阶段 3 最小工作流模型设计，如果此时写死 Codex，后续接入 LM、多 agent、知识库或 harness 仓库化会产生较高返工。

## 规则

### 必须高扩展预留

后续模型和接口必须优先使用通用概念：

- `agent_type`
- `agent_id`
- `adapter_id`
- `role`
- `capability`
- `artifact_type`
- `source_kind`
- `permission_level`
- `workflow_version`
- `node_type`
- `edge_type`
- `state`
- `provider`
- `model_policy`

当前可以只填 Codex：

- `agent_type = codex`
- `adapter_id = codex-local`
- `provider = local-codex-index`

但 schema 不应写死为：

- `codex_task`
- `codex_session_node`
- `codex_harness_only`
- `codex_director_state`

### LM 的定位

最终形态可以以 LM 作为智能核心，但不能以 LM 作为事实核心。

接受：

- LM 负责理解目标。
- LM 负责拆任务。
- LM 负责生成任务包。
- LM 负责总结 handoff / evidence。
- LM 负责提出 review 建议。
- LM 负责提示风险和缺口。

不接受：

- 任务状态只存在 LM 上下文里。
- LM 直接绕过工作流状态机改状态。
- LM 直接绕过权限确认执行写入、删除、运行 harness。
- 用聊天窗口替代可视化工作流和本地事实记录。

事实核心必须是：

- 本地索引。
- 本地工作流状态。
- evidence。
- handoff。
- review。
- 审计记录。

### 当前阶段边界

当前阶段仍不做：

- 非 Codex agent 接入。
- 个人知识库正文入库。
- 向量搜索。
- 模型辅助调度。
- 自动运行 harness。
- 写 Codex 状态库。
- release 打包。

但是当前阶段必须预留：

- 未来多个 agent 共存。
- 未来 LM Director 接入。
- 未来不同 agent adapter 接入。
- 未来 harness 多来源、多版本、文件夹式结构。
- 未来知识库作为 artifact / context provider 接入。

## 对阶段 3 最小工作流模型的影响

`product-line/archive/tasks/2026-05-28-codex-workflow-min-model.md` 已按这个规则执行：

- 模型命名要 agent-agnostic。
- 字段要能表达 Codex-only 当前实例。
- 字段也要能扩展到未来 agent。
- Codex 专用文件夹式 harness 不能建模成只属于 Codex 的特殊硬编码字段。
- Harness 应建模为项目能力 / 验证资源 / artifact 引用，再通过 `agent_type`、`adapter_id`、`capability` 表达当前适配 Codex。

## 风险

- 过度抽象会拖慢当前阶段。
- 抽象不足会导致后期重构成本高。
- 需要在每个任务包里明确“预留接口”和“当前不实现”的边界。

## 执行口径

后续总指导回收时必须检查：

- 是否写死 Codex。
- 是否把未实现的多 agent 写成已完成。
- 是否把 LM 建议当作事实状态。
- 是否有 `source_kind` 或同等来源字段。
- 是否有权限边界。
- 是否有版本字段或迁移口径。

如果任务为了当前速度把模型写死，必须退回修改。
