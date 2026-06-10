# 索引内核异常夹具验证交接

## 状态

验证线任务已完成，可回收。

## 做了什么

- 为索引内核补了异常夹具测试。
- 验证坏 schema、缺字段、缺文件、坏 manifest、坏 session_index JSONL、skill 编码异常。
- 验证测试命令离线可跑，不依赖网络，不使用真实授权文件。
- 验证测试前后真实 `/Users/yoyi/.codex` 目录 mtime 不变。

## 改了哪些文件

- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/evidence/2026-05-27-index-kernel-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-result.md`

运行测试时生成过 Python 缓存：

- `product-line/prototypes/index-kernel/tests/__pycache__/test_build_index_failures.cpython-313.pyc`

## 新增了哪些测试或夹具

新增 9 个 `unittest` 场景：

- SQLite 文件不存在。
- SQLite 存在但没有 `threads` 表。
- `threads` 表缺少非关键字段。
- `threads` 表缺少 `id` 字段。
- `rollout_path` 指向不存在的文件。
- `rollout_path` 指向允许目录外文件。
- `session_index.jsonl` 含坏 JSON 行。
- plugin manifest JSON 损坏。
- skill 文件编码异常。

夹具都在 `tempfile.TemporaryDirectory()` 里生成，并把 `build_index` 的 Codex 路径常量改指向临时假目录。

## 验证命令

异常夹具测试：

```bash
python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 9 tests in 0.018s
OK
```

禁用字节码缓存复跑：

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

结果：通过。

既有索引结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

```text
validation_ok
```

## 哪些异常场景通过

全部 9 个新增异常场景通过。

其中关键降级结果：

- 缺 SQLite、缺 `threads` 表、缺 `id` 字段都不会崩。
- 缺非关键字段仍能输出线程，并记录字段缺失 warning。
- rollout 缺文件和越界路径都不会误判为存在。
- 坏 JSONL 行和坏 manifest 都会记录 warning，能继续处理可读部分。
- skill 编码异常不会中断索引构建。

## 哪些异常场景暴露了问题

没有出现测试失败。

但暴露出两个设计问题：

- `build_index.py` 没有正式的测试注入入口。当前测试只能导入模块后覆盖全局路径常量。
- `validate_index()` 只做结构校验，不校验 warning 语义。异常是否明确暴露，靠本轮单元测试直接断言 `build_index()` 结果。

## 是否建议修改索引内核

建议修改，但本轮没改。依据：任务包允许写入不包含 `build_index.py`。

建议：

- 增加 `--codex-home` 或配置对象，让测试和桌面应用能显式指定数据源。
- 让 `sqlite_thread_count_differs_from_inventory` 只在真实盘点回归模式启用，避免夹具里固定出现 `:0`。
- 扩展 `validate_index()` 的可选语义检查，支持验证指定 warning。
- 增加 SQLite 损坏文件、权限拒绝、符号链接绕过和超大 JSONL 场景。

## 风险和下一步建议

风险：

- 本轮是夹具级验证，不代表覆盖所有未来 Codex schema 变化。
- 权限拒绝类场景没有稳定模拟，只覆盖了编码异常和坏 JSON。
- 测试会导入生产脚本并覆盖全局常量，后续重构路径常量时测试也要跟着改。

下一步：

- 回收线先接受本轮 evidence。
- 若允许进入修复线，先给索引内核补路径依赖注入，再把测试从 monkey patch 改为正式 CLI 或配置入口。
- 将本轮测试命令加入索引内核固定验收清单。
