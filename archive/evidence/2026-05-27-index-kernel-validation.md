# 索引内核异常夹具验证证据

## 结论先说

薄弱点：

- 本轮没有修改 `build_index.py`。依据：任务包“允许写入”没有列出该文件，因此本轮只补测试和验证文档。
- 当前测试通过了坏 schema、坏文件和坏 JSON 场景，但这些是单元夹具，不等于证明 Codex 后续内部结构变化全部可承受。依据：测试只覆盖任务包列出的 9 个离线场景。
- `validate_index()` 对 warning 的存在性没有强约束。依据：本轮异常场景主要直接断言 `build_index()` 结果里的 warning，而不是通过 `--check` 检查异常语义。

可用结果：

- 新增异常夹具测试：`product-line/prototypes/index-kernel/tests/test_build_index_failures.py`。
- 测试命令离线可运行，不依赖网络，不读取真实授权文件。
- 测试使用临时目录里的假 Codex home，不把夹具放进 `/Users/yoyi/.codex`。
- 顺序验证中，运行测试前后真实 `/Users/yoyi/.codex` 目录 mtime 都是 `1779847486`。依据见“不写真实 Codex 目录验证”。

## 本轮读取范围

按任务包读取：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-codex-index-kernel.md`
- `product-line/handoffs/2026-05-27-codex-index-kernel-result.md`

补充说明：

- 曾执行一次上级目录 `AGENTS.md` 路径搜索，输出只包含路径名，没有读取 `.codex` 文件内容；后续验证已收回到任务包范围内。
- 本轮没有读取或打印 `auth.json`、`.env`、密钥、令牌。

## 本轮写入

- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/evidence/2026-05-27-index-kernel-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-result.md`

Python 运行测试时生成过：

- `product-line/prototypes/index-kernel/tests/__pycache__/test_build_index_failures.cpython-313.pyc`

该文件是解释器缓存，不是手写夹具。

## 新增测试夹具

测试文件：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

测试做法：

- 用 `tempfile.TemporaryDirectory()` 创建临时假 Codex home。
- 将 `build_index` 模块里的 `CODEX_HOME`、`SQLITE_PATH`、`SESSION_INDEX_PATH`、`GLOBAL_STATE_PATH`、`SESSIONS_DIR`、`ARCHIVED_SESSIONS_DIR`、`SKILLS_DIR`、`PLUGIN_CACHE_DIR`、`MEMORIES_DIR` 全部指向临时目录。
- SQLite、session_index、plugin manifest、skill 文件都在临时目录内生成。

## 覆盖的异常场景

已覆盖并通过：

- SQLite 文件不存在：返回空线程，顶层 warning 包含 `missing_sqlite:<path>`。
- SQLite 存在但没有 `threads` 表：返回空线程，顶层 warning 包含 `missing_table:threads`。
- `threads` 表缺少非关键字段：仍生成线程，顶层 warning 包含 `missing_threads_field:<field>`。
- `threads` 表缺少 `id` 字段：返回空线程，顶层 warning 包含 `missing_threads_id_field`。
- `rollout_path` 指向不存在文件：线程 warning 包含 `missing_rollout_file`，统计 `missing=1`。
- `rollout_path` 指向允许目录外文件：不检查文件存在性，线程 warning 包含 `rollout_path_outside_allowed_session_dirs`。
- `session_index.jsonl` 含坏 JSON 行：记录 `session_index_invalid_json_line:2`，好行继续计数。
- plugin manifest JSON 损坏：插件保留，插件 warning 包含 `manifest_unreadable_or_invalid`，顶层 warning 包含 `invalid_json:<path>:1`。
- skill 文件编码异常：skill 保留，skill warning 包含 `skill_read_decode_failed`。

## 验证命令和结果

异常夹具测试：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 9 tests in 0.018s
OK
```

禁用字节码缓存后复跑：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 9 tests in 0.017s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：通过，命令退出码为 0。

既有索引结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

## 不写真实 Codex 目录验证

顺序验证命令：

```bash
stat -f '%m %N' /Users/yoyi/.codex
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
stat -f '%m %N' /Users/yoyi/.codex
```

结果：

```text
1779847486 /Users/yoyi/.codex
Ran 9 tests in 0.017s
OK
1779847486 /Users/yoyi/.codex
```

结论：

- 真实 `/Users/yoyi/.codex` 目录 mtime 在测试前后未变化。
- 这个证据只能证明目录本身没有新增、删除、重命名一类写入；测试不写真实 `.codex` 的主要依据仍是测试将所有 Codex 路径常量指向临时假目录。

## 暴露的问题

明确问题：

- `validate_index()` 仍偏结构校验，不验证异常 warning 是否存在。依据：本轮测试直接调用 `build_index()` 并断言 warning；`--check` 只能确认索引顶层和对象字段完整。

设计风险：

- `build_index.py` 目前没有正式的 `--codex-home` 或依赖注入入口。依据：测试只能通过导入模块后覆盖全局路径常量来注入临时目录。
- `sqlite_thread_count_differs_from_inventory:0` 会在空夹具或异常夹具中出现。依据：`build_index()` 固定把线程数和 289 比较；这适合真实盘点回归，不适合所有测试环境。

## 是否建议修改索引内核

建议改，但不是本轮越权修改。

建议项：

- 给 `build_index.py` 增加显式 `--codex-home` 或 `IndexSources` 配置对象，让测试和未来桌面应用不用覆盖全局常量。
- 把 `sqlite_thread_count_differs_from_inventory` 做成真实环境检查，或允许测试/夹具模式关闭。
- 扩展 `validate_index()`，让它可以选择性检查 warning 语义，比如要求异常索引必须带指定 warning。
- 考虑在 CLI 上增加只读验证模式，输出实际读取路径和拒绝写入路径，方便后续回收线复核。

## 风险和下一步

风险：

- 当前夹具覆盖的是任务包列出的异常，不覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL、manifest 字段类型异常等情况。
- skill 读取失败目前只能稳定模拟编码异常；权限拒绝在本地不同系统权限模型下不稳定，本轮没有强造。
- 真实 Codex 内部 schema 后续变化仍可能绕过当前字段集合，需要继续保持 warning-first 的验证习惯。

下一步建议：

- 若允许修改内核，先做路径依赖注入，再把本轮 monkey patch 式测试改为正式测试入口。
- 增加 SQLite 损坏文件和权限异常测试。
- 将本轮测试命令加入后续桌面应用线或回收线的固定验收清单。
