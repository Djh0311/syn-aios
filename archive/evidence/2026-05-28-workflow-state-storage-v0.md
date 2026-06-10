# 工作流事实层 v0 存储决策证据

## 结论先说

薄弱点：

- 阶段 3 最小工作流模型已定，但还没有本地事实层存储位置。依据：`product-line/handoffs/2026-05-28-codex-workflow-min-model-review.md` 明确“本地工作台状态存储位置还没定”。
- 当前桌面壳已能展示 `harness_resources`，但仍是只读候选展示，不是事实状态。依据：`product-line/handoffs/2026-05-28-desktop-shell-harness-resources-validation-review.md` 接受为真实窗口只读展示，并明确不接受为 harness 可运行、已验证或管理完成。
- 技术栈长期方向是 SQLite，但当前任务是 v0 事实层决策；直接上 SQLite 会增加当前阶段实现面。依据：`product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md` 写明 SQLite 是下一步稳定化方向，不要求立刻替换现有索引原型。

本轮结论：

- v0 本地事实层采用 JSON 文件，不采用 SQLite。
- v0 真实运行路径定为 Tauri 应用数据目录：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`。
- 本轮不创建真实运行状态文件；只在决策文档中给 schema 和最小样例。
- v0 JSON schema 必须按未来 SQLite 表结构设计，后续可以逐表迁移。

## 本轮读取范围

任务允许读取：

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

本轮写入：

- `product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`

本轮没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文
- 工具输出、命令输出、输入历史
- 记忆正文

本轮没有写入：

- `/Users/yoyi/.codex`
- 真实 Codex 状态库
- 前端代码
- Rust 代码
- 索引内核代码
- 真实运行中的工作流状态文件

## 当前索引事实

当前 `codex-index.json` 顶层字段：

- `generated_at`
- `memories`
- `plugins`
- `projects`
- `skills`
- `source_stats`
- `threads`
- `warnings`

当前项目字段已经包含：

- `harness_candidates`
- `harness_resources`

当前统计：

- 文件级 harness candidates：132
- 文件夹级 harness resources：14
- `source_stats.project_context.harness_resource_warning_count`：49

当前 `harness_resources` 字段：

- `root_path`
- `display_name`
- `harness_kind`
- `agent_type`
- `adapter_id`
- `source_kind`
- `capabilities`
- `manifest_path`
- `readme_path`
- `version`
- `entrypoints`
- `permission_level`
- `warnings`

结论：

- 索引派生层已能提供文件夹级 harness 候选。
- 这些仍是候选，不是“已验证”“可运行”“已启用”的事实。
- 是否启用、是否验证过、是否废弃、是否加强，必须进入本地事实层或 review / audit，不能写回索引。

## 已知

- 当前阶段只做 Codex 治理。依据：`product-line/STAGE_PLAN.md` 和 `product-line/README.md`。
- 模型必须高扩展，不写死 Codex-only。依据：`product-line/decisions/2026-05-28-extensible-first-development-rule.md`。
- 最小模型对象已确定。依据：`product-line/decisions/2026-05-28-codex-workflow-min-model.md`。
- 桌面壳已通过真实 Tauri 窗口 smoke，并能只读展示 `harness_resources`。依据：相关回收意见。

## 假设

- Tauri 应用可以通过 Rust 后端访问应用数据目录。依据：Tauri 技术底座已接受；具体 API 使用属于后续桌面应用线实现。
- v0 工作流状态规模较小，JSON 文件足够承载。依据：当前阶段目标是项目级事实层 v0，不是全文搜索、知识库或大规模事件库。
- 后续 SQLite 迁移会发生，但不应阻塞当前阶段落地事实层边界。

## 缺口

- 真实运行状态文件尚未创建。
- 读写命令尚未实现。
- JSON schema 尚未有自动校验器。
- 状态文件并发写入策略需要桌面应用线实现时处理。
- SQLite 迁移脚本尚未实现。

## 风险

- JSON 文件缺少数据库级约束；需要写入前校验 schema。
- JSON 文件并发写入容易损坏；需要单写入口、临时文件原子替换和备份。
- 如果路径放进项目目录，会污染业务项目；所以 v0 选择应用数据目录。
- 如果路径放进 `.codex`，会违反当前边界；所以明确禁止。
- 如果事实层混入索引派生字段，后续会分不清什么是事实、什么是候选；所以必须使用 `source_kind` 和 `permission_level`。
