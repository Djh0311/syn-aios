# 工作流事实层 v0 存储决策交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-workflow-state-storage-v0.md`
- 开发线：信息架构线
- Evidence：`product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- Decision：`product-line/decisions/2026-05-28-workflow-state-storage-v0.md`

## 结论

建议接受为阶段 3 本地工作流事实层 v0 存储决策。

本轮没有创建真实运行状态文件，没有改前端、Rust 或索引内核代码。它只定义后续桌面应用线可以实现的存储方式、路径、schema、合并规则、写入边界和迁移口径。

## 先说薄弱点

- v0 选 JSON，不是最终事实库形态。长期仍应迁移到 SQLite。
- JSON 没有数据库级约束，后续实现必须做 schema 校验、备份和原子替换。
- 本轮没有实现读写命令，也没有真实写入状态文件。
- 当前 `harness_resources` 仍是索引候选，不是已验证或可运行事实。
- 如果后续实现绕过 Rust 后端直接写文件，会破坏权限边界。

## 做了什么

- 决定 v0 本地事实层用 JSON 文件。
- 明确真实运行路径和备份路径。
- 定义 v0 schema。
- 给出最小 JSON 样例。
- 定义只读索引派生层和本地事实层合并规则。
- 定义 `source_kind`、`permission_level`、`agent_type`、`adapter_id`、`workflow_version` 的填值规则。
- 定义写入边界、备份和审计要求。
- 定义后续迁移到 SQLite 的口径。
- 定义下一条桌面应用线实现任务建议。

## 读了哪些文件

- `product-line/tasks/2026-05-28-workflow-state-storage-v0.md`
- `product-line/tasks/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
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

## 新增文件

- `product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- `product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- `product-line/decisions/2026-05-28-workflow-state-storage-v0.md`

## v0 存储方式和路径

存储方式：

- JSON 文件。

真实运行路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

备份路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json
```

本轮没有创建这些文件。

选择理由：

- 当前事实层规模小。
- JSON 易读、易备份、易审计。
- 当前任务禁止改代码，不能直接实现 SQLite。
- schema 可按 SQLite 表结构设计，后续平滑迁移。

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

核心规则：

- `schema_version = workflow_state_v0`
- `workflow_version = 1`
- `permission_level` 默认 `read_only`
- `model_policy` 当前默认 `none`
- 当前只注册 `codex-local` adapter
- 未接入 agent 不写入可用 adapter 表

## 索引派生和本地事实边界

来自索引派生：

- 项目候选。
- 会话元数据。
- handoff 候选。
- evidence 候选。
- authority 候选。
- 文件级 harness candidates。
- 文件夹级 harness resources。

属于本地事实：

- 用户确认后的 workflow。
- 用户确认后的 node / edge。
- 工作项状态。
- review 判断。
- audit event。
- 项目能力启用、暂停、废弃等状态。
- 已登记或已确认的 harness resource。
- 最近打开 / 最近使用事件。

合并规则：

- 索引只生成候选，不自动改事实状态。
- 本地事实补充 UI 状态。
- 候选被用户确认后，才写入本地事实并追加 audit event。
- 本地事实引用的索引来源消失时，保留事实并显示 `source_missing` warning。

## 写入边界和审计要求

可以写：

- Tauri 应用数据目录中的 `workflow-state.v0.json`。
- 同目录 `backups/` 里的备份文件。

不能写：

- `/Users/yoyi/.codex`
- Codex 真实状态库
- 项目业务目录
- `.env`、授权文件、密钥文件
- `product-line/` 产物目录中的运行状态

必须用户确认：

- 创建或修改 workflow。
- 创建或修改 node / edge。
- 创建或修改 work_item。
- 登记 review。
- 接受候选 harness resource 成为事实。
- 状态转换。
- 删除、废弃、覆盖本地事实。

写入必须：

- 先备份。
- 校验 schema。
- 追加 audit event。
- 写临时文件。
- 原子替换。
- 重新读取校验。

## 迁移风险

- JSON 没有外键约束，需要迁移前做引用校验。
- JSON 时间字段必须统一。
- ID 必须稳定，否则迁移到 SQLite 后节点关系会断。
- 未知字段不能直接丢弃，迁移时应进入 `extra_json` 或迁移日志。
- 如果 v0 没有严格区分 `source_kind`，后续会混淆候选和事实。

## 下一条桌面应用线任务建议

建议任务名：

- `2026-05-28-desktop-shell-workflow-state-v0.md`

建议目标：

- 在 Rust 后端实现 v0 JSON 状态文件读写。
- 首次启动时如果状态文件不存在，只返回空状态，不自动创建，除非用户确认初始化。
- 实现 schema 校验。
- 实现备份和原子写入。
- 实现 audit event 追加。
- 前端只读展示本地事实层和索引派生层合并结果。
- 支持用户确认“初始化工作流事实层”。

验收建议：

- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不写项目业务目录。
- 初始化必须有确认。
- 写入前有备份。
- 写入后能重新读取。
- `source_kind`、`permission_level`、`workflow_version` 可见或可调试。
- 索引候选不会自动变事实。
- 不自动运行 harness。

## 仍不确定

- `workspace_id` 最终 hash 算法。
- JSON schema 校验器用 Rust 手写还是引入 schema 文件。
- 状态文件损坏时 UI 的恢复流程。
- 多窗口并发写入如何锁定。
- 何时从 JSON 迁移到 SQLite。
- SQLite 迁移时是否保留 JSON 作为导出格式。

## 后置能力

当前仍不做：

- 自动运行 harness。
- 多 agent 接入。
- 个人知识库。
- 向量搜索。
- LM 调度。
- release 打包。
- 写 Codex 状态库。

## 验收对照

- 有 evidence、handoff、decision：已完成。
- 明确选择 v0 存储方式和路径：已完成。
- 明确 v0 schema 字段和最小样例：已完成。
- 明确只读索引派生层与本地事实层边界：已完成。
- 明确写入权限、备份和审计要求：已完成。
- 明确后续迁移到 SQLite 的理由和风险：已完成。
- 明确下一条桌面应用线实现任务：已完成。
- 不把未实现的可编辑工作流写成已完成：已遵守。
- 不扩大到多 agent、知识库、向量搜索、LM 调度：已遵守。
