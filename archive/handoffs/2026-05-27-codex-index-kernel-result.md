# Codex 只读索引内核交接

## 状态

原型已完成，可进入回收。

## 产物

- 原型脚本：`product-line/prototypes/index-kernel/build_index.py`
- 生成索引：`product-line/prototypes/index-kernel/codex-index.json`
- evidence：`product-line/evidence/2026-05-27-codex-index-kernel.md`

## 怎么运行

生成索引：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

校验已有索引：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py
```

## 当前输出位置

```text
/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json
```

## 当前统计

- 线程数：290
- 项目数：30
- skills 数：50
- 本地非插件 skills 数：7
- 插件内 skills 数：43
- plugins 数：11
- memories 元数据入口数：11
- rollout 存在率：290/290

线程数不是上游盘点的 289。依据：本轮只读 SQLite 统计为 290，索引顶层 warning 已记录 `sqlite_thread_count_differs_from_inventory:290`。

## 已满足的任务要求

- 以 `/Users/yoyi/.codex/state_5.sqlite` 的 `threads` 表作为主索引。
- 用 `threads.cwd` 聚合项目。
- 用 `threads.rollout_path` 校验原始会话文件存在性。
- 扫描 `/Users/yoyi/.codex/skills/` 本地 skills。
- 扫描 `/Users/yoyi/.codex/plugins/cache/**/.codex-plugin/plugin.json` 和插件内 `skills/*/SKILL.md`。
- 输出本地索引 JSON。
- 对 schema 缺失、文件缺失、格式变化预留 warning 路径。
- `session_index.jsonl` 标为 `auxiliary_thread_list`。
- `.codex-global-state.json` 标为 `ui_state_and_project_hint_source`，没有覆盖 `threads.cwd`。
- SQLite 只读打开，并启用 `PRAGMA query_only = ON`。

## 字段降级和跳过

降级：

- `thread_source` 只保留 `user`、`subagent`，其余输出 `unknown`。
- `title` 限制为 160 字以内单行展示，超长记录 `title_truncated`；本轮有 61 条。
- `created_at_ms`、`updated_at_ms` 缺失时才从文本时间派生。

跳过：

- 不读取 JSONL 正文。
- 不输出 `first_user_message` 和 `preview`。
- 不解析 `threads.source`。
- memories 不读取正文，只输出路径、类型、行数、更新时间。
- 不读取或展示授权文件、`.env`、密钥、令牌。

## 给回收线的检查点

- 运行 `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json`，期望输出 `validation_ok`。
- 检查 `source_stats.sqlite.opened_readonly=true` 和 `source_stats.sqlite.query_only_enabled=true`。
- 检查 `source_stats.sqlite.rollout_files.missing=0`。
- 检查 `source_stats.session_index.role=auxiliary_thread_list`。
- 检查 `source_stats.global_state.used_to_override_thread_cwd=false`。

## 风险和下一步

- 风险：索引 JSON 仍含项目路径、会话路径、线程标题、模型和 token 统计，属于本机工作上下文。建议桌面应用线默认只展示必要字段，并对标题继续做折叠或搜索后显示。
- 风险：当前只对真实环境验证，没有做坏 schema 夹具测试。建议验证线补 SQLite 缺表、缺字段、rollout 缺文件、manifest 损坏、skill 读取失败的夹具。
- 风险：`threads.source` 未结构化解析。建议单独开小任务做来源解析，不要混进本轮回收。
- 下一步：由回收线确认索引内核 evidence，若接受，可把 `codex-index.json` 作为桌面应用线的只读样例输入。
