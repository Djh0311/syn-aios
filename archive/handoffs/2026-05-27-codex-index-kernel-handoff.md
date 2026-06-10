# Codex 索引内核线交接

## 任务来源

本交接来自 `product-line/tasks/2026-05-27-codex-data-inventory.md`。

上游 evidence：

- `product-line/evidence/2026-05-27-codex-local-data-inventory.md`

## 第一版读取原则

先泼冷水：

- 不要用 Codex 侧边栏状态做权威索引。依据：项目 README 已记录侧边栏不可靠，本轮 evidence 也显示全局状态只有 22 条线程到项目的提示，覆盖远低于 SQLite 的 289 条线程。
- 不要只读 `session_index.jsonl`。依据：它只有 232 个去重线程 ID，而 SQLite 有 289 个线程 ID。
- 不要浅层扫描 `sessions/`。依据：`maxdepth 3` 得到 0 个文件，`maxdepth 8` 得到 231 个文件。

建议的权威顺序：

1. `state_5.sqlite.threads`
2. `threads.rollout_path` 指向的 JSONL 会话文件
3. `.codex-global-state.json` 的项目列表和工作区提示
4. `session_index.jsonl` 轻量补充
5. `skills/`、`plugins/cache/`、`memories/` 作为独立资料入口

## 线程索引读取

主表：

- `/Users/yoyi/.codex/state_5.sqlite`
- 表：`threads`

必读字段：

- `id`
- `rollout_path`
- `created_at`
- `updated_at`
- `created_at_ms`
- `updated_at_ms`
- `cwd`
- `title`
- `archived`
- `archived_at`
- `thread_source`
- `model_provider`
- `model`
- `reasoning_effort`
- `sandbox_policy`
- `approval_mode`
- `tokens_used`
- `has_user_event`
- `preview`

可选字段：

- `source`
- `git_sha`
- `git_branch`
- `git_origin_url`
- `cli_version`
- `first_user_message`
- `agent_nickname`
- `agent_role`
- `memory_mode`

第一版输出结构建议：

```json
{
  "thread_id": "string",
  "title": "string",
  "project_root": "string",
  "rollout_path": "string",
  "created_at_ms": "number|null",
  "updated_at_ms": "number|null",
  "archived": "boolean",
  "archived_at_ms": "number|null",
  "thread_source": "user|subagent|unknown",
  "model": "string|null",
  "model_provider": "string|null",
  "reasoning_effort": "string|null",
  "sandbox_policy": "string|null",
  "approval_mode": "string|null",
  "tokens_used": "number",
  "has_user_event": "boolean",
  "preview": "string",
  "confidence": "high|medium|low",
  "warnings": ["string"]
}
```

字段处理建议：

- `project_root` 用 `threads.cwd`，不要用全局状态里的侧边栏归属覆盖它。
- `created_at_ms` 和 `updated_at_ms` 优先使用毫秒字段；为空时才降级到 `created_at`、`updated_at`。
- `thread_source` 只映射 `user`、`subagent`、空值。空值显示为 `unknown`。
- `preview` 默认截断，详情页按需打开。
- `first_user_message` 默认不进入列表页。
- `source` 单独解析，不要直接展示原始 JSON 字符串。

校验建议：

- 启动时检查 `threads` 表是否存在。
- 检查必读字段是否存在，缺字段就降级并记录 warning。
- 检查 `rollout_path` 文件是否存在；不存在时保留线程，但标记 `missing_rollout_file`。
- 读取 SQLite 必须只读打开。

## 会话 JSONL 读取

主入口：

- `threads.rollout_path`

备用扫描：

- `/Users/yoyi/.codex/sessions/**/*.jsonl`
- `/Users/yoyi/.codex/archived_sessions/*.jsonl`

不要只扫浅层目录。依据：`sessions/` 按年月日分层。

顶层字段：

- `timestamp`
- `type`
- `payload`

第一版建议只做这些能力：

- 打开原始会话文件。
- 统计事件类型。
- 提取 `session_meta`、`turn_context` 的少量上下文字段。
- 对正文、命令输出、工具输出做默认折叠。

默认不展示：

- `payload.content`
- `payload.message`
- `payload.output`
- `payload.stdout`
- `payload.stderr`
- `payload.arguments`
- `payload.encrypted_content`

原因：

- 这些字段可能包含用户输入、命令输出、业务细节或不可解析内容。依据见 evidence 的 JSONL 采样字段。

## 项目索引读取

主来源：

- `threads.cwd`

辅助来源：

- `.codex-global-state.json.electron-saved-workspace-roots`
- `.codex-global-state.json.project-order`
- `.codex-global-state.json.active-workspace-roots`

不要作为权威：

- `.codex-global-state.json.thread-workspace-root-hints`
- `.codex-global-state.json.projectless-thread-ids`
- 侧边栏折叠状态

第一版输出结构建议：

```json
{
  "project_root": "string",
  "thread_count": "number",
  "active_thread_count": "number",
  "archived_thread_count": "number",
  "latest_updated_at_ms": "number|null",
  "from_saved_workspace_roots": "boolean",
  "active_hint": "boolean",
  "order_hint": "number|null",
  "confidence": "high|medium|low"
}
```

合并规则：

- 先按 `threads.cwd` 聚合项目。
- 再把 `electron-saved-workspace-roots` 里没有线程的项目补进来，标记 `thread_count=0`。
- `active_workspace_roots` 只加提示，不改变项目归属。
- 如果 `cwd` 不存在于磁盘，仍保留记录，标记 `missing_project_root`。

## skill 索引读取

个人和系统 skill：

- `/Users/yoyi/.codex/skills/**/SKILL.md`

插件内 skill：

- `/Users/yoyi/.codex/plugins/cache/**/skills/*/SKILL.md`

字段建议：

```json
{
  "skill_id": "string",
  "source_type": "system|user|plugin",
  "title": "string",
  "description": "string",
  "path": "string",
  "plugin_name": "string|null",
  "plugin_version": "string|null"
}
```

解析建议：

- `title` 取第一个一级标题。
- `description` 取 front matter 或 `description:` 行。
- 不在列表页加载全文。
- `.system` 下的 skill 标为 `system`。
- `/plugins/cache/**` 下的 skill 标为 `plugin`。

依据：

- 本轮在 `/Users/yoyi/.codex/skills/` 找到 7 个 `SKILL.md`。
- 本轮在插件缓存里找到 43 个 `skills/*/SKILL.md`。

## plugin 索引读取

manifest：

- `/Users/yoyi/.codex/plugins/cache/**/.codex-plugin/plugin.json`

字段建议：

```json
{
  "plugin_name": "string",
  "plugin_version": "string",
  "manifest_path": "string",
  "description": "string",
  "homepage": "string|null",
  "has_mcp_servers": "boolean",
  "has_apps": "boolean",
  "skill_paths": ["string"]
}
```

注意：

- 不要只信 manifest 的 `skills` 字段。依据：本轮 manifest 中 `skills` 计数为 0，但目录扫描发现 43 个插件 skill 文件。
- 插件缓存目录版本可能变化，索引器每次启动应重新扫描。

## memories 读取

建议只做资料入口：

- `/Users/yoyi/.codex/memories/MEMORY.md`
- `/Users/yoyi/.codex/memories/memory_summary.md`
- `/Users/yoyi/.codex/memories/raw_memories.md`
- `/Users/yoyi/.codex/memories/rollout_summaries/*.md`
- `/Users/yoyi/.codex/memories/.omx/logs/*.jsonl`

不要做：

- 不把 memory 正文当项目事实。
- 不把 `.omx` 日志混入 Codex 会话主索引。
- 不默认展示全文。
- 不索引 `.git/`。

第一版输出结构建议：

```json
{
  "memory_path": "string",
  "kind": "memory|summary|raw|rollout_summary|omx_log",
  "line_count": "number",
  "updated_at_ms": "number|null",
  "confidence": "low"
}
```

## session_index 读取

路径：

- `/Users/yoyi/.codex/session_index.jsonl`

字段：

- `id`
- `thread_name`
- `updated_at`

建议用途：

- 用于兼容或快速列表补充。
- 可检查 SQLite 是否漏掉轻量索引中的线程。

不要做：

- 不把它当主索引。
- 不用它判断项目归属。

原因：

- 字段太少。
- 与 SQLite 数量不一致。

## 安全规则

索引内核必须遵守：

- 只读打开 `/Users/yoyi/.codex`。
- 不读取 `auth.json`。
- 不读取 `.env`。
- 不打印密钥、令牌、授权信息。
- 不写回 `state_5.sqlite`。
- 不移动、删除、格式化 Codex 文件。
- 不把 UI 侧边栏状态当权威。

建议实现保护：

- 文件读取白名单。
- 敏感文件名黑名单：`auth.json`、`.env`、包含 `token`、`secret`、`key` 的授权类文件。
- SQLite 用只读连接。
- 会话正文默认不进列表页。
- 大字段读取按需、截断、可关闭。

## 第一版最小验收

索引内核线完成后至少应证明：

- 能从 `state_5.sqlite.threads` 读出 289 条线程或合理解释数量变化。
- 能验证 `rollout_path` 存在率并报告缺失文件。
- 能按 `cwd` 聚合项目。
- 能列出本地 7 个非插件 skill。
- 能列出插件 manifest 和插件内 skill。
- 能把 `session_index.jsonl` 标为辅助来源。
- 能把 `.codex-global-state.json` 标为 UI 状态和项目提示来源。
- 不读取或打印授权文件。

## 未解决问题

- `source` 字段需要单独解析器。当前只确认它混有字符串和 JSON 子任务来源。
- `created_at` 与 `created_at_ms` 的单位换算还没在本轮验证。
- 归档会话除 `threads.rollout_path` 指向的文件外，是否还有未入库文件，本轮未全量验证。
- Codex 后续版本可能改 SQLite schema，索引内核需要迁移检查。

