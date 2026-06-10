# 决策：阶段 3 工作流事实层 v0 存储

## 结论

v0 本地工作流事实层采用 JSON 文件。

真实运行路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

备份路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json
```

本轮不创建真实运行状态文件。后续桌面应用线实现时，才由 Tauri / Rust 在应用数据目录创建和写入。

选择 JSON 的原因：

- 当前阶段事实层规模小，主要保存工作流、节点、边、review、audit 等轻量事实。
- 当前任务目标是先落存储边界和 schema，不是引入完整数据库迁移。
- JSON 可读、可备份、易审计，适合 v0。
- schema 按表结构设计，后续可迁移到 SQLite。

不选择 SQLite 作为 v0 的原因：

- 技术栈长期方向仍是 SQLite，但当前还没有读写命令、迁移脚本、状态表和验证任务。
- 直接上 SQLite 会扩大本轮实现面，容易把“存储决策”推进成“数据库实现”。
- 当前禁止改 Rust / 前端 / 索引内核代码。

## 边界

v0 可以写：

- Tauri 应用数据目录下的 `workflow-state.v0.json`。
- 同目录下 `backups/` 备份文件。
- 写入必须由后续实现的 Rust 后端统一执行。

v0 不能写：

- `/Users/yoyi/.codex`
- Codex 真实状态库
- 项目业务目录
- `product-line/` 产物目录，除非是任务文档、样例或测试夹具
- `.env`、授权文件、密钥文件

## v0 schema

顶层字段：

- `schema_version`
- `workflow_version`
- `workspace_id`
- `created_at`
- `updated_at`
- `source_kind`
- `permission_level`
- `projects`
- `agent_adapters`
- `workflows`
- `nodes`
- `edges`
- `work_items`
- `artifacts`
- `reviews`
- `audit_events`
- `capabilities`
- `harness_resources`

### schema_version

当前值：

- `workflow_state_v0`

用途：

- 标识状态文件 schema。
- 后续迁移到 `workflow_state_v1` 或 SQLite 时使用。

### workflow_version

当前值：

- `1`

用途：

- 标识工作流语义版本，不等同于文件 schema 版本。

### workspace_id

v0 规则：

- 由工作台根据当前工作区根路径生成稳定 ID。
- 不直接等同于 Codex thread 或 Codex cwd。
- 建议格式：`workspace:<normalized-root-hash>`。

当前 `/Users/yoyi/workspace` 示例：

- `workspace:yoyi-workspace`

说明：

- 示例不是最终 hash 算法。
- 后续实现需固定生成规则。

### projects

保存本地项目事实，不复制完整索引项目对象。

最小字段：

- `project_id`
- `display_name`
- `root_path`
- `source_kind`
- `permission_level`
- `created_at`
- `updated_at`
- `warnings`

### agent_adapters

保存可用 agent adapter。

当前只允许 Codex：

- `adapter_id = codex-local`
- `agent_type = codex`
- `provider = local-codex-index`
- `status = available`
- `permission_level = read_only`

未接入 agent 不写入可用 adapter 表。

### workflows

保存项目工作流事实。

最小字段：

- `workflow_id`
- `workflow_version`
- `project_id`
- `title`
- `state`
- `source_kind`
- `permission_level`
- `model_policy`
- `created_at`
- `updated_at`

### nodes

保存工作流节点事实或已确认的派生节点。

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

### edges

保存节点关系。

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

### work_items

保存任务包或工作项。

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

### artifacts

保存工作流引用的产物元数据，不保存正文。

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

禁止：

- 不存 Codex 会话正文。
- 不存工具输出。
- 不存命令输出。
- 不存密钥或授权内容。

### reviews

保存回收判断。

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

### audit_events

保存写入、确认、状态转换审计。

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

规则：

- 所有用户确认写入都必须追加 audit event。
- 所有状态转换都必须追加 audit event。
- 所有失败写入也应记录失败事件，不能静默吞掉。

### capabilities

保存项目能力。

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

### harness_resources

保存已被本地事实层确认或登记的 harness resource。

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
- `status`
- `warnings`

说明：

- 索引里的 `harness_resources` 是候选。
- 本地状态里的 `harness_resources` 是用户确认、登记或工作台管理过的事实。
- 候选不能自动升级成事实。

## 最小样例

此样例只用于说明 schema，不是本轮创建的真实状态文件。

```json
{
  "schema_version": "workflow_state_v0",
  "workflow_version": 1,
  "workspace_id": "workspace:yoyi-workspace",
  "created_at": "2026-05-28T00:00:00Z",
  "updated_at": "2026-05-28T00:00:00Z",
  "source_kind": "workspace_state",
  "permission_level": "read_only",
  "projects": [
    {
      "project_id": "project:workspace",
      "display_name": "workspace",
      "root_path": "/Users/yoyi/workspace",
      "source_kind": "codex_index",
      "permission_level": "read_only",
      "created_at": "2026-05-28T00:00:00Z",
      "updated_at": "2026-05-28T00:00:00Z",
      "warnings": []
    }
  ],
  "agent_adapters": [
    {
      "adapter_id": "codex-local",
      "agent_type": "codex",
      "agent_id": "local-codex",
      "display_name": "Codex",
      "provider": "local-codex-index",
      "capabilities": ["read_index", "open_project_path", "reveal_rollout_path", "copy_path"],
      "status": "available",
      "permission_level": "read_only",
      "source_kind": "workspace_state"
    }
  ],
  "workflows": [
    {
      "workflow_id": "workflow:project-workspace:default",
      "workflow_version": 1,
      "project_id": "project:workspace",
      "title": "默认工作流",
      "state": "draft",
      "source_kind": "workspace_state",
      "permission_level": "read_only",
      "model_policy": "none",
      "created_at": "2026-05-28T00:00:00Z",
      "updated_at": "2026-05-28T00:00:00Z"
    }
  ],
  "nodes": [],
  "edges": [],
  "work_items": [],
  "artifacts": [],
  "reviews": [],
  "audit_events": [],
  "capabilities": [],
  "harness_resources": []
}
```

## 只读索引派生层和本地事实层合并规则

读取顺序：

1. 读取 `codex-index.json`。
2. 读取 `workflow-state.v0.json`。
3. 校验 `schema_version` 和 `workflow_version`。
4. 用本地事实层补充 UI 状态。
5. 用索引派生层刷新只读候选。

合并原则：

- 索引派生层提供候选，不直接改事实状态。
- 本地事实层保存用户确认、review、状态转换、审计记录。
- 同一个对象如果同时存在索引派生和本地事实，UI 要显示来源差异。
- 本地事实引用的索引来源消失时，不删除事实，只显示 `source_missing` warning。
- 索引新增会话、handoff、evidence、harness resource 时，只生成候选，不自动生成已确认节点。
- 用户确认采用候选时，才写入本地事实层并追加 audit event。

## source_kind 填值规则

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

v0 规则：

- 来自 `codex-index.json` 的候选用 `codex_index`。
- 写入状态文件的事实用 `workspace_state`。
- 从任务包文件解析的用 `task_package`。
- 从 handoff / evidence / review 文件登记的用对应值。
- 用户手动创建或确认的用 `user_input`。
- LM 输出如果未来出现，只能用 `lm_suggestion`，不能直接改事实。
- 程序从可追溯来源推导的用 `derived`，必须带 `source_ref`。

## permission_level 填值规则

允许值：

- `read_only`
- `user_confirmed_write`
- `user_confirmed_run`
- `blocked`

v0 默认：

- `read_only`

需要用户确认：

- 创建或修改 workflow。
- 创建或修改 node / edge。
- 创建或修改 work_item。
- 登记 review。
- 接受候选 harness resource 成为事实。
- 状态转换。
- 删除、废弃、覆盖任何本地事实。

当前禁止：

- `user_confirmed_run` 不进入 v0 实现。
- 不自动运行 harness。
- 不写 Codex 状态库。

## 写入流程

后续实现必须按这个流程：

1. 前端发起明确动作。
2. 显示确认弹层，说明目标、路径、source_kind、permission_level。
3. Rust 后端读取当前状态文件。
4. 校验 schema。
5. 写入前复制当前文件到 `backups/`。
6. 生成 audit event。
7. 写临时文件。
8. 原子替换 `workflow-state.v0.json`。
9. 重新读取校验。
10. UI 显示成功或失败。

失败规则：

- 备份失败则不写。
- schema 校验失败则不写。
- 原子替换失败则保留备份并报告错误。
- 不允许部分写入后静默成功。

## 迁移到 SQLite 的口径

v0 JSON 的每个数组都对应未来 SQLite 表：

- `projects` -> `projects`
- `agent_adapters` -> `agent_adapters`
- `workflows` -> `workflows`
- `nodes` -> `workflow_nodes`
- `edges` -> `workflow_edges`
- `work_items` -> `work_items`
- `artifacts` -> `artifacts`
- `reviews` -> `reviews`
- `audit_events` -> `audit_events`
- `capabilities` -> `project_capabilities`
- `harness_resources` -> `harness_resources`

迁移规则：

- ID 字段保持稳定。
- `schema_version` 决定迁移脚本。
- `workflow_version` 保留为业务语义版本。
- `source_kind`、`permission_level`、`model_policy` 原样迁移。
- JSON 中未知字段迁移到 SQLite 时先进入 `extra_json` 或迁移日志，不直接丢弃。

迁移风险：

- JSON 没有外键约束，迁移前需要完整引用校验。
- JSON 时间字段格式必须统一，否则 SQLite 查询会混乱。
- 如果 v0 写入时不保留稳定 ID，迁移后 UI 会丢失节点关系。

## 后置内容

当前不做：

- 自动运行 harness。
- 多 agent 接入。
- 个人知识库。
- 向量搜索。
- LM 调度。
- release 打包。
- 写 Codex 状态库。

长期预留：

- SQLite 事实库。
- React Flow 节点坐标和交互。
- LM Director 建议层。
- 多 agent adapter。
- harness 多来源、多版本。
