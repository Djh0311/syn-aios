# 任务包：Codex 只读索引内核

## 所属开发线

索引内核线。

## 背景

Codex 数据盘点线已回收并接受。第一版不能用 `session_index.jsonl` 或侧边栏状态做权威，必须以 `state_5.sqlite.threads` 为主索引。

依据：

- `product-line/evidence/2026-05-27-codex-local-data-inventory.md`
- `product-line/handoffs/2026-05-27-codex-index-kernel-handoff.md`
- `product-line/handoffs/2026-05-27-codex-data-inventory-review.md`

## 目标

- 实现一个只读索引内核原型。
- 从 `state_5.sqlite.threads` 读取线程。
- 按 `threads.cwd` 聚合项目。
- 用 `threads.rollout_path` 校验原始会话文件存在性。
- 扫描本地 skills 和插件 skills。
- 输出一个本地索引 JSON 文件。
- 对 schema 缺失、文件缺失、格式变化做降级和 warning。

## 允许读取

- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/session_index.jsonl`
- `/Users/yoyi/.codex/sessions/`
- `/Users/yoyi/.codex/archived_sessions/`
- `/Users/yoyi/.codex/.codex-global-state.json`
- `/Users/yoyi/.codex/skills/`
- `/Users/yoyi/.codex/plugins/`
- `/Users/yoyi/.codex/memories/`
- `/Users/yoyi/workspace/product-line/`

## 允许写入

- `/Users/yoyi/workspace/product-line/evidence/`
- `/Users/yoyi/workspace/product-line/handoffs/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改 `state_5.sqlite`。
- 不移动、删除、格式化 Codex 文件。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不默认展示 JSONL 正文、命令输出、工具输出、输入历史或记忆正文。
- 不把 `.codex-global-state.json.thread-workspace-root-hints` 覆盖到 `threads.cwd`。
- 不把 `session_index.jsonl` 当主索引。

## 最小输出结构

索引 JSON 至少包含：

- `generated_at`
- `warnings`
- `threads`
- `projects`
- `skills`
- `plugins`
- `memories`
- `source_stats`

线程对象至少包含：

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

项目对象至少包含：

- `project_root`
- `thread_count`
- `active_thread_count`
- `archived_thread_count`
- `latest_updated_at_ms`
- `from_saved_workspace_roots`
- `active_hint`
- `order_hint`
- `warnings`

## 验收标准

- 能读出当前 SQLite 线程数，若不是 289，必须在 evidence 中解释当前数量。
- 能报告 `rollout_path` 存在率。
- 能按 `cwd` 聚合项目。
- 能列出本地非插件 skill。
- 能列出插件 manifest 和插件内 skill。
- 能把 `session_index.jsonl` 标为辅助来源。
- 能把 `.codex-global-state.json` 标为 UI 状态和项目提示来源。
- SQLite 必须只读打开。
- 不读取或打印授权文件。
- 有自动或半自动验证命令。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些 evidence / handoff
4. 输出索引文件在哪里
5. 当前线程数、项目数、skills 数、plugins 数是多少
6. 哪些字段降级或跳过了
7. 风险和下一步建议
