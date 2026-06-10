# 任务包：阶段 3 工作流事实层 v0 存储决策

## 任务名

定义 Codex 工作台阶段 3 的本地工作流事实层 v0。

## 所属开发线

信息架构线。

当前先派给信息架构线，因为任务核心是存储口径和 schema 决策。后续如果工作流事实层需要长期独立推进，可以按开发线治理规则新增或拆分开发线。

## 背景

阶段 3 最小工作流模型已接受，但仍缺本地事实层落地口径。

已知：

- UI 骨架已经实现并通过真实 Tauri 窗口 smoke。
- 桌面壳已只读展示文件夹级 `harness_resources`，并通过真实 Tauri 窗口 smoke。
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md` 已定义 `Workflow`、`WorkflowNode`、`WorkflowEdge`、`WorkItem`、`Artifact`、`ReviewRecord`、`AuditEvent` 等对象。
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md` 要求 schema 不写死 Codex，但当前功能仍只交付 Codex 治理。

未知：

- v0 本地状态存储用 JSON 还是 SQLite。
- v0 状态文件或数据库放在哪里。
- v0 是否写入，写入范围如何确认。
- 从索引派生的节点和用户确认后的事实如何区分。
- 后续迁移到 SQLite 时如何避免重做 UI。

## 目标

- 决定阶段 3 v0 本地工作流事实层的存储形式和路径。
- 定义 v0 schema，至少覆盖：
  - `schema_version`
  - `workspace_id`
  - `projects`
  - `workflows`
  - `nodes`
  - `edges`
  - `work_items`
  - `artifacts`
  - `reviews`
  - `audit_events`
  - `capabilities`
- 定义只读索引派生层和本地事实层的合并规则。
- 定义 `source_kind`、`permission_level`、`agent_type`、`adapter_id`、`workflow_version` 的 v0 填值规则。
- 定义 v0 写入边界：
  - 可以写哪里。
  - 不能写哪里。
  - 哪些写入必须用户确认。
  - 是否需要备份。
- 定义 v0 迁移口径：
  - 如果 v0 用 JSON，后续怎么迁 SQLite。
  - 如果 v0 用 SQLite，当前 Tauri/Rust 最小需要哪些读写命令。
- 定义下一条桌面应用线实现任务需要的输入、输出和验收标准。
- 明确哪些内容仍后置：自动运行 harness、多 agent、知识库、向量搜索、LM 调度。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/handoffs/2026-05-28-codex-workflow-min-model-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-review.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-review.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-validation-review.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/README.md`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`
- `product-line/decisions/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不改前端、Rust、索引内核代码。
- 不创建真实运行中的工作流状态文件，除非任务结论明确说明只是样例并放在 `evidence/` 或 `decisions/`。
- 不自动运行 harness。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。

## 验收标准

- 有 evidence、handoff、decision。
- 明确选择 v0 存储方式和路径，不能只说“以后再定”。
- 明确 v0 schema 字段和最小样例。
- 明确只读索引派生层与本地事实层的边界。
- 明确写入权限、备份和审计要求。
- 明确后续迁移到 SQLite 或继续使用 SQLite 的理由。
- 明确下一条桌面应用线实现任务应做什么。
- 不把未实现的可编辑工作流写成已完成。
- 不扩大到多 agent、知识库、向量搜索、LM 调度。

## 必须回传

1. 做了什么
2. 读了哪些文件
3. 新增了哪些 evidence / handoff / decision
4. v0 存储方式和路径是什么
5. v0 schema 是什么
6. 哪些数据来自索引派生，哪些属于本地事实
7. 写入边界和审计要求是什么
8. 后续迁移风险是什么
9. 下一条桌面应用线任务建议是什么
10. 哪些仍不确定

## 总指导回收重点

回收时必须判断：

- 是否真的解决了本地状态存储位置和 schema。
- 是否符合高扩展开发规则。
- 是否没有写死 Codex-only。
- 是否没有把 LM 建议当作事实状态。
- 是否没有越过当前 Codex 治理阶段。
- 是否能直接派给桌面应用线实现。
