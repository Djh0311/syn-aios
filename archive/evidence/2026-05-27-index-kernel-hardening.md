# 索引内核路径注入与语义校验证据

## 结论先说

薄弱点：

- 当前真实 SQLite 线程数又变化了，本轮生成时是 295，不是上游 289，也不是上一轮索引内核证据里的 290。依据：`python3 product-line/prototypes/index-kernel/build_index.py --pretty` 输出 `thread_count=295`。
- `title_truncated` 仍然存在，本轮真实索引有 64 条线程标题被截断。依据：`--check --warning-summary` 输出 `{"title_truncated": 64}`。
- 本轮仍没有覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL。依据：任务目标是路径注入和 warning 语义校验，异常夹具仍是原 9 个加 3 个入口/语义测试。

可用结果：

- `build_index.py` 已新增正式数据源配置对象 `IndexSources`。
- CLI 已支持 `--codex-home <path>`。
- 测试不再覆盖模块全局路径常量，而是通过 `IndexSources.from_codex_home()` 注入临时假 Codex home。
- 线程数回归检查从硬编码 warning 改为 opt-in：只有传 `--expect-thread-count <n>` 才会生成 `sqlite_thread_count_differs_from_expected:<actual>`。
- `--check` 已支持 warning 语义校验：`--require-warning`、`--forbid-warning`、`--warning-summary`。

## 本轮读取范围

按任务包读取：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/evidence/2026-05-27-index-kernel-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-result.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-review.md`

验证真实环境生成索引时读取：

- `/Users/yoyi/.codex/state_5.sqlite`
- `/Users/yoyi/.codex/session_index.jsonl`
- `/Users/yoyi/.codex/.codex-global-state.json`
- `/Users/yoyi/.codex/skills/`
- `/Users/yoyi/.codex/plugins/cache/`
- `/Users/yoyi/.codex/memories/`
- `threads.rollout_path` 指向的会话文件存在性

本轮没有读取或打印 `auth.json`、`.env`、授权文件、密钥、令牌。没有写真实 `/Users/yoyi/.codex`。

## 本轮写入

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-index-kernel-hardening.md`
- `product-line/handoffs/2026-05-27-index-kernel-hardening-result.md`

验证过程中 Python 生成过 `__pycache__`，已清理；交付目录现在只保留脚本、索引 JSON 和测试文件。

## 新入口

默认真实环境生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

指定 Codex home：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --codex-home /Users/yoyi/.codex --output /private/tmp/codex-index-explicit-home.json
```

显式线程数回归检查：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty --output /private/tmp/codex-index-expected-289.json --expect-thread-count 289
```

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

warning 语义校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check /private/tmp/codex-index-expected-289.json --require-warning sqlite_thread_count_differs_from_expected
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --forbid-warning sqlite_thread_count_differs_from_expected
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --warning-summary
```

## 测试变化

修改测试文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`

保留原 9 个异常夹具：

- SQLite 文件不存在。
- SQLite 存在但没有 `threads` 表。
- `threads` 表缺少非关键字段。
- `threads` 表缺少 `id` 字段。
- `rollout_path` 指向不存在文件。
- `rollout_path` 指向允许目录外文件。
- `session_index.jsonl` 含坏 JSON 行。
- plugin manifest JSON 损坏。
- skill 文件编码异常。

新增 3 个测试：

- `test_cli_codex_home_uses_injected_source_root`：验证 `--codex-home` 使用临时假 Codex home。
- `test_expected_thread_count_warning_is_opt_in`：验证线程数 warning 默认关闭，传入 expected 才出现。
- `test_check_can_require_warning_semantics`：验证 `--check` 能要求和禁止指定 warning。

测试已改为：

- `self.sources = build_index.IndexSources.from_codex_home(self.codex_home)`
- `build_index.build_index(self.sources)`

测试不再做：

- 不再覆盖 `build_index.CODEX_HOME`
- 不再覆盖 `build_index.SQLITE_PATH`
- 不再覆盖 `build_index.SESSION_INDEX_PATH`
- 不再覆盖 `build_index.GLOBAL_STATE_PATH`
- 不再覆盖 `build_index.SESSIONS_DIR`
- 不再覆盖 `build_index.ARCHIVED_SESSIONS_DIR`
- 不再覆盖 `build_index.SKILLS_DIR`
- 不再覆盖 `build_index.PLUGIN_CACHE_DIR`
- 不再覆盖 `build_index.MEMORIES_DIR`

## 验证命令和结果

异常夹具和 hardening 测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 12 tests in 0.021s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：通过。

真实环境生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果摘要：

```json
{"memory_count": 11, "plugin_count": 11, "project_count": 30, "rollout_checked": 295, "rollout_existing": 295, "skill_count": 50, "thread_count": 295, "warning_count": 0}
```

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

禁止默认线程数 warning：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --forbid-warning sqlite_thread_count_differs_from_expected
```

结果：

```text
validation_ok
```

warning 汇总：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --warning-summary
```

结果：

```json
{"title_truncated": 64}
```

显式 289 回归检查：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty --output /private/tmp/codex-index-expected-289.json --expect-thread-count 289
python3 product-line/prototypes/index-kernel/build_index.py --check /private/tmp/codex-index-expected-289.json --require-warning sqlite_thread_count_differs_from_expected
```

结果：

- 生成摘要中 `thread_count=295`，`warning_count=1`。
- `/private/tmp/codex-index-expected-289.json` 顶层 warning 为 `sqlite_thread_count_differs_from_expected:295`。
- `--require-warning` 返回 `validation_ok`。

显式 Codex home：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --codex-home /Users/yoyi/.codex --output /private/tmp/codex-index-explicit-home.json
python3 product-line/prototypes/index-kernel/build_index.py --check /private/tmp/codex-index-explicit-home.json
```

结果：

- 生成摘要中 `thread_count=295`。
- `source_stats.codex_home.path=/Users/yoyi/.codex`。
- `--check` 返回 `validation_ok`。

## 当前索引统计

当前交付索引：

- `product-line/prototypes/index-kernel/codex-index.json`

当前统计：

- 生成时间：`2026-05-27T08:42:03Z`
- 线程数：295
- 项目数：30
- skills 总数：50
- 本地非插件 skills 数：7
- 插件内 skills 数：43
- plugins 数：11
- memories 元数据入口数：11
- rollout 存在率：295/295
- session_index 行数：276
- session_index 去重线程 ID：235
- session_index 有但 SQLite 没有：1
- SQLite 有但 session_index 没有：61
- 顶层 warnings：空
- warning 汇总：`title_truncated=64`

## 风险

- `--require-warning` 和 `--forbid-warning` 是按 warning code 或完整 warning 文本匹配，不能表达复杂条件，例如“必须正好出现 N 次”。
- `--warning-summary` 会统计顶层和对象级 warnings；它适合回收检查，不等于完整审计报告。
- `--codex-home` 允许指定任意目录，调用方仍需要遵守任务包读取边界。
- 当前真实索引线程数变化很快，不能把 295 写成稳定事实；只能作为本轮生成时的事实。
