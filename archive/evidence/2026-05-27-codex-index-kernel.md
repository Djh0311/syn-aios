# Codex 只读索引内核证据

## 结论先说

薄弱点：

- 当前 `state_5.sqlite.threads` 已经不是上游盘点时的 289 条，而是 290 条。依据：索引器只读打开 SQLite 后得到 `thread_count=290`；独立命令 `sqlite3 'file:/Users/yoyi/.codex/state_5.sqlite?mode=ro' 'select count(*) from threads;'` 也返回 290。
- `threads.title` 不总是短标题，部分记录会包含长任务包正文。依据：初版索引生成后发现长标题会把索引文件放大到约 1.5M，并携带过多会话上下文；已改为 160 字以内单行展示，61 条线程记录 `title_truncated`。
- SQLite 是 Codex 内部状态库，schema 没有公开稳定契约。依据：上游回收意见已记录该风险，本轮实现也只把现有字段作为当前可读结构。

可用结果：

- 已实现只读索引内核原型：`product-line/prototypes/index-kernel/build_index.py`。
- 已生成本地索引：`product-line/prototypes/index-kernel/codex-index.json`。
- SQLite 用 `file:...?...mode=ro` 打开，并执行 `PRAGMA query_only = ON`。依据：索引 `source_stats.sqlite.opened_readonly=true`，`query_only_enabled=true`。
- `rollout_path` 文件存在率为 290/290。依据：索引 `source_stats.sqlite.rollout_files.checked=290`，`existing=290`，`missing=0`，`existence_rate=1.0`。

## 本轮读取范围

本轮读取：

- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/session_index.jsonl`
- `/Users/yoyi/.codex/sessions/`
- `/Users/yoyi/.codex/archived_sessions/`
- `/Users/yoyi/.codex/.codex-global-state.json`
- `/Users/yoyi/.codex/skills/`
- `/Users/yoyi/.codex/plugins/`
- `/Users/yoyi/.codex/memories/`
- `/Users/yoyi/workspace/product-line/`

本轮写入：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-codex-index-kernel.md`
- `product-line/handoffs/2026-05-27-codex-index-kernel-result.md`

本轮没有读取或打印 `auth.json`、`.env`、授权文件、密钥或令牌。依据：脚本没有访问这些路径；会话 JSONL 正文没有被读取，只检查 `threads.rollout_path` 指向文件是否存在。

## 原型说明

入口：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

输出：

```text
/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json
```

验证：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py
```

验证结果：

- `--check` 返回 `validation_ok`。
- `py_compile` 通过。

## 输出结构

索引顶层字段：

- `generated_at`
- `warnings`
- `threads`
- `projects`
- `skills`
- `plugins`
- `memories`
- `source_stats`

线程对象字段：

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

项目对象字段：

- `project_root`
- `thread_count`
- `active_thread_count`
- `archived_thread_count`
- `latest_updated_at_ms`
- `from_saved_workspace_roots`
- `active_hint`
- `order_hint`
- `warnings`

依据：`jq 'keys'` 和对象 `keys` 检查均符合任务包最小结构。

## 当前统计

索引生成时间：

- `2026-05-27T01:20:06Z`

核心数量：

- SQLite 线程数：290
- 项目数：30
- skills 总数：50
- 本地非插件 skills 数：7
- 插件内 skills 数：43
- 插件 manifest 数：11
- memories 元数据入口数：11

线程状态：

- active 线程：232
- archived 线程：58
- `thread_source=subagent`：153
- `thread_source=user`：42
- `thread_source=unknown`：95

rollout 文件：

- checked：290
- existing：290
- missing：0
- outside_allowed_session_dirs：0
- existence_rate：1.0

`session_index.jsonl`：

- role：`auxiliary_thread_list`
- 行数：271
- 解析数：271
- 去重线程 ID：233
- session_index 有但 SQLite 没有：1
- SQLite 有但 session_index 没有：58

`.codex-global-state.json`：

- role：`ui_state_and_project_hint_source`
- saved workspace roots：9
- project order：9
- active workspace roots：1
- thread workspace root hints：22
- `used_to_override_thread_cwd=false`

## 降级和跳过

已降级：

- 当前 SQLite 线程数不是上游记录的 289，索引顶层 warning 记录为 `sqlite_thread_count_differs_from_inventory:290`。依据：本轮实测 SQLite 为 290。
- 61 条线程标题被截断，线程级 warning 记录为 `title_truncated`。依据：`jq '[.threads[].warnings[]] | group_by(.) ...'`。
- `thread_source` 只接受 `user` 和 `subagent`，空值或不认识的值输出为 `unknown`。
- `created_at_ms`、`updated_at_ms` 若缺失才从文本时间降级派生；本轮 SQLite 必读字段没有缺失。

主动跳过：

- 不读取 JSONL 正文，不统计正文消息，不展示 `payload.content`、`payload.message`、`payload.output`、`stdout`、`stderr` 或工具参数。
- 不读取 `first_user_message`、`preview` 进入索引输出。
- 不解析 `threads.source`。依据：上游 handoff 记录该字段混有字符串和 JSON 子任务来源，需要单独解析器。
- memories 只输出路径、类型、行数和更新时间，不读取正文进索引。
- `.codex-global-state.json.thread-workspace-root-hints` 不用于覆盖 `threads.cwd`。

## 验收对照

- 能读出当前 SQLite 线程数：通过，当前为 290；不是 289 的原因是本地 Codex 状态已经新增线程，证据见本轮只读统计。
- 能报告 `rollout_path` 存在率：通过，290/290。
- 能按 `cwd` 聚合项目：通过，项目数 30。
- 能列出本地非插件 skill：通过，7 个。
- 能列出插件 manifest 和插件内 skill：通过，11 个 manifest，43 个插件 skill。
- 能把 `session_index.jsonl` 标为辅助来源：通过，role 为 `auxiliary_thread_list`。
- 能把 `.codex-global-state.json` 标为 UI 状态和项目提示来源：通过，role 为 `ui_state_and_project_hint_source`。
- SQLite 必须只读打开：通过，`opened_readonly=true` 且 `query_only_enabled=true`。
- 不读取或打印授权文件：通过，本轮没有访问授权路径。
- 有自动或半自动验证命令：通过，`--check` 和 `py_compile`。

## 风险

- 真实索引仍包含线程标题、项目路径、rollout 路径、模型和 token 统计，这些不是密钥，但属于本机工作上下文；后续界面默认展示范围需要继续收紧。
- `title` 截断只能降低泄漏面，不能证明标题完全不含敏感信息。
- 插件 manifest 字段没有稳定统一命名，`has_mcp_servers`、`has_apps` 只是按常见字段存在性判断。
- 没有做坏 schema 夹具测试；目前只在当前真实 SQLite 上验证了 schema 检查和缺字段 warning 逻辑。
