# Codex 工作台 UI 与信息架构重设计证据

## 结论先说

薄弱点：

- 当前产品化桌面壳一期已经能展示索引，但 UI 方向仍偏索引浏览器，不能继续只在旧 6 页结构上小修。依据：`product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md` 明确说现有首页、项目页、会话页、Skills / Plugins 页、任务线页、诊断页不能作为最终 UI 方向继续扩展。
- 当前 `codex-index.json` 能支撑项目、会话、skills、plugins、handoff / evidence、harness 候选展示，但不能完整支撑“最近打开/最近使用”、Skill 推荐关系、Harness 多仓库多版本来源。依据：本轮只读 `codex-index.json` 字段结构和统计。
- Agent 页只能把 Codex 写成当前可用编排对象，其他 agent 只能空白占位。依据：任务包、阶段计划和 UI 方向决策都明确当前只编排 Codex，不做多 agent 接入。
- 可视化工作流模型可以设计，但当前索引没有完整任务包状态机、review 节点和节点间边关系。依据：`codex-index.json` 顶层只有 `threads`、`projects`、`skills`、`plugins`、`memories`、`source_stats`、`warnings`。

可用结论：

- 新版首页必须只有四个入口：Agent、项目、Skill 管理、Harness 管理。入口下方显示最近打开或最近使用，不显示数量。依据：UI 方向决策和任务包。
- 项目打开后默认进入项目级可视化工作流，不再默认进入列表页。依据：UI 方向决策和阶段计划阶段 3。
- 项目内固定采用左侧窄功能列表、中间工作流画布、右侧详情面板。依据：UI 方向决策和任务包。
- Skill 管理要从目录树升级为关系看板，展示分类、agent 使用关系、项目使用关系、推荐关系和后置能力。依据：任务包和 UI 方向决策。
- Harness 管理要展示框架、版本、来源、功能、使用场景、适用项目、关联命令或验证入口，并预留多仓库、多版本来源。依据：任务包和 UI 方向决策。

## 本轮读取范围

任务允许读取的文件：

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`
- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-review.md`
- `product-line/prototypes/productized-desktop-shell/README.md`
- `product-line/prototypes/index-kernel/codex-index.json`

本轮写入：

- `product-line/evidence/2026-05-28-codex-workbench-ui-ia-redesign.md`
- `product-line/handoffs/2026-05-28-codex-workbench-ui-ia-redesign-result.md`

本轮没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文
- 工具输出、命令输出、输入历史
- 记忆正文

## 阶段边界依据

当前阶段仍以治理 Codex 为主，不接非 Codex agent，不做个人知识库正文入库，不做向量搜索，不做模型辅助调度，不做跨 agent 复杂画布编排，不写 Codex 状态库。

依据：`product-line/STAGE_PLAN.md` 总原则。

阶段 3 已把项目级可视化编排列为目标：

- 首页总览只提供 Agent、项目、Skill 管理、Harness 管理四个入口。
- 项目打开后进入项目级可视化工作流。
- 用可视化工作流表达 Director、Codex 会话、任务包、handoff、evidence、review 的流转。
- 保持当前只编排 Codex，不接入 OpenClaw、VS Code、OpenCode、Claude Code。

依据：`product-line/STAGE_PLAN.md` 阶段 3。

## 当前索引字段证据

本轮只读检查 `product-line/prototypes/index-kernel/codex-index.json`。

顶层字段：

- `generated_at`
- `memories`
- `plugins`
- `projects`
- `skills`
- `source_stats`
- `threads`
- `warnings`

当前统计：

- 项目数：30
- 会话数：296
- skills 数：50
- plugins 数：11
- 顶层 warnings 数：0
- harness 候选数：132

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

Skill 字段：

- `skill_id`
- `title`
- `description`
- `path`
- `source_type`
- `plugin_name`
- `plugin_version`
- `warnings`

Plugin 字段：

- `plugin_name`
- `plugin_version`
- `description`
- `homepage`
- `manifest_path`
- `skill_paths`
- `has_apps`
- `has_mcp_servers`
- `warnings`

Harness 候选字段：

- `entry_type`
- `name`
- `path`
- `source`
- `size_bytes`
- `updated_at_ms`
- `warnings`

## 当前索引能直接支持的 UI 字段

首页：

- 四入口本身来自 IA 决策，不来自索引。
- 入口下方最近项目可用 `projects.latest_updated_at_ms` 近似。
- 入口下方最近 Skill 可用 `skills` 的静态列表和 `plugin_version`，但没有真实最近使用事件。
- 入口下方最近 Harness 可用 `harness_candidates.updated_at_ms` 近似。
- Agent 入口当前只能显示 Codex，依据来自阶段决策，不来自索引。

项目：

- 项目列表、项目路径、最近活跃时间、会话数、authority / handoff / evidence、harness 候选可由当前索引支持。
- 项目类型、项目别名、项目最近打开事件、项目工作流节点位置需要后续新增。

工作流：

- Codex 会话节点可由 `threads` 支持。
- Handoff / Evidence 节点可由 `projects[].handoff_files` 和 `projects[].evidence_files` 支持。
- Harness 候选节点可由 `projects[].harness_candidates` 支持。
- Director、任务包、Review、状态流转、节点边关系需要后续新增。

Skill 管理：

- skill 名称、描述、路径、来源、插件名、插件版本可由 `skills` 支持。
- 被哪个 agent 使用、能在哪个 agent 使用、被哪些项目使用、推荐关系、加载状态需要后续新增。

Harness 管理：

- 候选名称、入口类型、路径、来源目录、更新时间可由 `harness_candidates` 支持。
- 框架、版本、来源仓库、功能说明、使用场景、适用项目、关联命令语义、最近验证状态、多仓库和多版本来源需要后续新增。

## 对旧产品化桌面壳的判断

保留：

- Tauri 2 + Rust + React + TypeScript + Vite 技术底座。
- 只读索引读取。
- 路径白名单。
- 打开目录、复制路径、定位日志前的权限确认模式。
- 不展示正文、密钥、授权信息的安全边界。

不继续作为 UI 方向：

- 旧首页总览。
- 旧项目页列表式索引。
- 旧会话页作为主入口。
- 旧 Skills / Plugins 目录展示。
- 旧任务线 / evidence / handoff 页的平铺方式。
- 诊断页作为主导航入口。

依据：UI 方向决策明确旧 UI 可作技术底座，不能作为最终信息架构继续扩展。

## 已知

- 当前只编排 Codex。依据：阶段计划、README、任务包、UI 方向决策。
- 首页四入口已确认。依据：UI 方向决策。
- 项目默认进入可视化工作流已确认。依据：UI 方向决策。
- 未接入 agent 必须空白显示。依据：任务包和 UI 方向决策。
- Skill 删除、编辑、选择 agent 加载后置。依据：任务包和 UI 方向决策。
- Harness 多仓库、多版本来源后置。依据：任务包和 UI 方向决策。

## 未知

- “最近打开/最近使用”是否已有事件存储。当前索引没有专门字段。
- Director 节点是否对应总指导线文件，还是 UI 内抽象角色。
- 任务包状态机如何从 `tasks/README.md`、任务包文件、handoff / review 文件可靠计算。
- Skill 推荐关系由人工标注、项目类型规则还是后续模型辅助产生。
- Harness 框架和版本如何识别，当前候选只有入口类型、路径、来源目录和更新时间。
- 工作流节点坐标、分组、边关系是否写入项目级配置；这属于后续写入能力，不在本任务实现。

## 不作为事实的内容

- 不声称 OpenClaw、VS Code、OpenCode、Claude Code 已可用。
- 不声称当前能自动删除、编辑、加载 skills。
- 不声称当前能管理多个 harness 仓库和多版本。
- 不声称当前有向量搜索、个人知识库、模型调度。
- 不声称当前能自动判断 harness 是否有用。
