# 工作流事实层 v0 存储决策总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-workflow-state-storage-v0.md`
- 开发线：信息架构线
- Evidence：`product-line/evidence/2026-05-28-workflow-state-storage-v0.md`
- Handoff：`product-line/handoffs/2026-05-28-workflow-state-storage-v0-result.md`
- Decision：`product-line/decisions/2026-05-28-workflow-state-storage-v0.md`

## 结论

接受为“阶段 3 本地工作流事实层 v0 存储决策”。

不接受为“读写实现完成”，不接受为“真实状态文件已创建”，也不接受为“可编辑自动化工作流完成”。

依据：

- 决策明确 v0 使用 JSON 文件，不直接上 SQLite。
- 决策明确真实运行路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`。
- 决策明确备份路径：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json`。
- 决策明确 v0 schema 顶层字段，包括 `schema_version`、`workflow_version`、`workspace_id`、`projects`、`agent_adapters`、`workflows`、`nodes`、`edges`、`work_items`、`artifacts`、`reviews`、`audit_events`、`capabilities`、`harness_resources`。
- 决策明确索引派生层只产生候选，本地事实层保存用户确认后的 workflow、node、edge、review、audit、能力状态和已登记 harness。
- 决策明确写入必须用户确认、备份、schema 校验、追加 audit event、临时文件写入、原子替换、重新读取校验。
- 决策明确迁移到 SQLite 的口径：JSON 数组对应未来表，ID、`source_kind`、`permission_level`、`workflow_version` 保持稳定。
- 总指导线只读检查真实运行状态文件不存在，符合本轮“不创建真实运行状态文件”的边界。

## 先说薄弱点

- v0 JSON 不是最终事实库形态。依据：技术栈长期方向仍是 SQLite + FTS，决策也明确后续迁移。
- JSON 没有数据库级约束。依据：handoff 明确后续实现必须做 schema 校验、引用校验、备份和原子替换。
- 本轮没有实现读写命令。依据：任务禁止改前端、Rust、索引内核代码。
- 本轮没有创建真实运行状态文件。依据：任务禁止创建真实运行中的工作流状态文件，总指导线 `test -e` 检查返回不存在。
- 并发写入、状态文件损坏恢复、`workspace_id` 最终 hash 算法仍未定。依据：handoff “仍不确定”部分。
- `harness_resources` 仍是候选资源，不能因为进入 schema 就变成可运行事实。依据：决策明确索引候选不能自动升级成本地事实。

## 接受内容

接受 v0 存储选择：

- 当前用 JSON 文件。
- 后续迁移 SQLite。

接受真实运行路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json
```

接受备份路径：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.<timestamp>.json
```

接受 v0 顶层 schema：

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

接受当前 Codex-only 默认：

- `adapter_id = codex-local`
- `agent_type = codex`
- `provider = local-codex-index`
- `permission_level = read_only`
- `model_policy = none`

接受写入边界：

- 可写 Tauri 应用数据目录下的状态文件和 backups。
- 不写 `/Users/yoyi/.codex`。
- 不写 Codex 真实状态库。
- 不写项目业务目录。
- 不写 `.env`、授权文件、密钥文件。
- 运行状态不写 `product-line/`，除非是任务文档、样例或测试夹具。

接受合并规则：

- 索引派生层提供候选，不直接改事实状态。
- 本地事实层保存用户确认、review、状态转换、审计记录。
- 索引来源消失时保留本地事实，并显示 `source_missing` warning。
- 候选被用户确认采用后，才写入本地事实层并追加 audit event。

## 总指导线复核

只读存在性检查：

```bash
test -e '/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json'
```

结果：

- 返回码 1，无输出。
- 判断为真实运行状态文件未创建。

说明：

- 总指导线没有读取应用数据目录中的状态文件内容，因为文件不存在。
- 总指导线没有写应用数据目录。
- 总指导线没有写 `/Users/yoyi/.codex`。

## 安全和范围判断

接受当前安全边界。

依据：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未改前端、Rust 或索引内核代码。
- 未创建真实运行状态文件。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、LM 调度或 release 打包。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 阶段 3 本地工作流事实层 v0 存储方式、路径、schema、写入边界和迁移口径已定。

仍不能说：

- 工作流事实层读写已实现。
- 真实状态文件已创建。
- 可编辑自动化工作流已完成。
- SQLite 事实库已完成。
- 多 agent、知识库、向量搜索或 LM 调度已接入。

## 下一步

下一步派给桌面应用线：实现工作流事实层 v0 的最小读写。

约束：

- 缺状态文件时只返回空状态和 `exists=false`，不自动创建。
- 初始化必须用户确认。
- 写入必须由 Rust 后端统一执行。
- 写入前必须备份；新文件无旧文件时要记录无备份原因。
- 写入必须追加 audit event。
- 写入必须临时文件 + 原子替换 + 重新读取校验。
- 前端只做初始化和只读展示，不做复杂画布编辑。
- 不写 `.codex`，不写项目业务目录，不自动运行 harness。
