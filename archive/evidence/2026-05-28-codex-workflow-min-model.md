# Codex 项目级工作流最小数据模型证据

## 结论先说

薄弱点：

- 当前新版 UI 骨架已经能展示工作流样子，但没有正式工作流数据模型。依据：`product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md` 明确 Director、Review、边关系、状态机仍缺数据模型。
- 当前 `codex-index.json` 能提供项目、会话、handoff、evidence、harness 候选等只读来源，但不能提供完整工作流事实。依据：本轮只读检查 `codex-index.json` 字段，顶层只有 `generated_at`、`memories`、`plugins`、`projects`、`skills`、`source_stats`、`threads`、`warnings`。
- 当前 harness 候选索引是文件级候选，不足以表达用户补充的“Codex 专用、文件夹形式 harness”。依据：当前 harness 候选字段只有 `entry_type`、`name`、`path`、`source`、`size_bytes`、`updated_at_ms`、`warnings`，缺少文件夹根、manifest、版本、能力、适配 agent、命令入口。
- 模型不能写死为 Codex-only。依据：`product-line/decisions/2026-05-28-extensible-first-development-rule.md` 要求模型和接口优先使用通用概念，当前可以只填 Codex，但 schema 不应写成 `codex_task`、`codex_session_node`、`codex_harness_only`。

可用结论：

- 阶段 3 最小工作流模型应分成三层：只读索引派生层、本地工作台事实层、未来智能建议层。
- 事实状态必须落在本地工作流模型、evidence、handoff、review 或审计记录里，不能只存在 LM 上下文。依据：高扩展开发规则。
- 当前 Codex-only 实例应使用通用字段填值，例如 `agent_type=codex`、`adapter_id=codex-local`、`source_kind=codex_index`、`workflow_version=1`。
- 文件夹式 harness 应建模为项目能力和验证资源，不是单个脚本候选，也不是 Codex 专属硬编码字段。

## 本轮读取范围

任务允许读取：

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-redesign-review.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation-review.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-redesign.md`
- `product-line/evidence/2026-05-28-codex-workbench-ui-shell-tauri-smoke-validation.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`

本轮写入：

- `product-line/evidence/2026-05-28-codex-workflow-min-model.md`
- `product-line/handoffs/2026-05-28-codex-workflow-min-model-result.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`

本轮没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文
- 工具输出、命令输出、输入历史
- 记忆正文

## 当前索引事实

`codex-index.json` 生成时间：

- `2026-05-27T10:23:52Z`

当前统计：

- 项目：30
- 会话：296
- skills：50
- plugins：11
- harness 候选：132

项目字段：

- `project_root`
- `thread_count`
- `active_thread_count`
- `archived_thread_count`
- `latest_updated_at_ms`
- `from_saved_workspace_roots`
- `active_hint`
- `order_hint`
- `authority_files`
- `handoff_files`
- `evidence_files`
- `harness_candidates`
- `context_warnings`
- `warnings`

会话字段：

- `thread_id`
- `title`
- `project_root`
- `rollout_path`
- `rollout_exists`
- `created_at_ms`
- `updated_at_ms`
- `archived`
- `thread_source`
- `model`
- `model_provider`
- `reasoning_effort`
- `tokens_used`
- `has_user_event`
- `warnings`

当前 harness 候选字段：

- `entry_type`
- `name`
- `path`
- `source`
- `size_bytes`
- `updated_at_ms`
- `warnings`

## 当前前端事实

`src/lib/types.ts` 现有类型只覆盖：

- `ProjectRecord`
- `SessionRecord`
- `SkillRecord`
- `PluginRecord`
- `HarnessCandidate`
- `TaskEntry`
- `WorkbenchSnapshot`
- 路径动作确认类型

没有正式类型：

- `Workflow`
- `WorkflowNode`
- `WorkflowEdge`
- `WorkflowState`
- `Actor`
- `Artifact`
- `Review`
- `AuditEvent`
- `Capability`

`ProjectsView.tsx` 当前工作流画布是从现有索引临时拼出的 UI 骨架：

- 项目中心来自 `projects.project_root`
- Codex 会话来自 `threads` 元数据
- Handoff 来自 `handoff_files`
- Evidence 来自 `evidence_files`
- Harness 来自 `harness_candidates`
- Director 和 Review 显示为缺少数据
- 画布提示缺少节点边关系、坐标、任务包状态机、Director 拆解结果和 Review 结论

`HarnessBoardView.tsx` 当前只按 `harness_candidates.entry_type`、路径和来源目录做候选展示，明确缺少版本、来源仓库、功能说明、使用场景、关联命令语义和最近验证状态。

## 事实、假设、缺口

事实：

- 当前只交付 Codex 治理，不接入非 Codex agent。
- 模型必须高扩展预留，不能写死 Codex-only schema。
- 当前 UI 骨架不是可编辑自动化工作流。
- 当前 harness 索引不是完整文件夹式 harness 支持。

假设：

- 阶段 3 本地工作台状态文件可以放在项目线或工作台数据目录中；具体路径需后续任务决定。
- `workflow_version=1` 可以作为第一版模型迁移口径。
- 当前 Codex-only 实例的默认 adapter 可以命名为 `codex-local`。

缺口：

- 工作流本地状态文件尚未定义存储路径和格式。
- 任务包、review、状态转换、审计记录尚未进入前端类型。
- 文件夹式 harness 的目录结构和 manifest 规则尚未扫描验证。
- LM Director 仍是未来预留，不是当前可用事实来源。

## 风险

- 过度抽象会拖慢当前阶段；因此模型只定义最小对象和必要扩展字段。
- 抽象不足会导致后续接入其他 agent、LM Director 或 harness 仓库化时重构。
- 如果继续沿用当前 `harness_candidates` 文件级模型，会漏掉文件夹式 harness 的整体能力、版本和适配关系。
- 如果把 LM 输出当状态事实，会导致 UI 看起来“智能”，但没有可审计依据。
