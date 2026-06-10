# 索引内核路径注入与语义校验交接

## 状态

修复已完成，可进入回收。

## 做了什么

- 给索引内核增加 `IndexSources` 数据源配置对象。
- 给 CLI 增加 `--codex-home <path>`。
- 把测试从覆盖模块全局路径常量改为传入正式配置对象。
- 把线程数回归 warning 改成显式 opt-in：`--expect-thread-count <n>`。
- 扩展 `--check`，支持 warning 语义校验和 warning 汇总。
- 重新生成真实环境索引。

## 改了哪些文件

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-index-kernel-hardening.md`

## 新增或修改了哪些测试

原 9 个异常夹具保留并通过。

新增 3 个测试：

- `test_cli_codex_home_uses_injected_source_root`
- `test_expected_thread_count_warning_is_opt_in`
- `test_check_can_require_warning_semantics`

当前测试数：12。

## 新入口怎么使用

默认生成真实索引：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

指定 Codex home：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --codex-home /path/to/codex-home --output /tmp/codex-index.json
```

显式线程数回归检查：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --expect-thread-count 289 --output /tmp/codex-index.json
```

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

warning 语义校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check /tmp/codex-index.json --require-warning missing_table
python3 product-line/prototypes/index-kernel/build_index.py --check /tmp/codex-index.json --forbid-warning sqlite_thread_count_differs_from_expected
python3 product-line/prototypes/index-kernel/build_index.py --check /tmp/codex-index.json --warning-summary
```

## 哪些验证通过

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
  - 结果：12 tests OK。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
  - 结果：通过。
- `python3 product-line/prototypes/index-kernel/build_index.py --pretty`
  - 结果：真实索引生成成功，当前线程数 295，rollout 295/295。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json`
  - 结果：`validation_ok`。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --forbid-warning sqlite_thread_count_differs_from_expected`
  - 结果：`validation_ok`。
- `python3 product-line/prototypes/index-kernel/build_index.py --check /private/tmp/codex-index-expected-289.json --require-warning sqlite_thread_count_differs_from_expected`
  - 结果：`validation_ok`。

## 当前统计

当前交付索引：

- `product-line/prototypes/index-kernel/codex-index.json`

统计：

- 线程数：295
- 项目数：30
- skills 数：50
- 本地非插件 skills：7
- 插件内 skills：43
- plugins 数：11
- memories 元数据入口：11
- rollout 存在率：295/295
- 顶层 warnings：空
- warning 汇总：`title_truncated=64`

## 是否仍有未修问题

仍未修：

- 没有覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL。
- `threads.source` 仍未结构化解析。
- warning 语义校验只能检查“有/无某类 warning”，不能校验次数、比例或字段级关联。

已修：

- 测试不再 monkey patch 模块全局路径常量。
- `--codex-home` 可用。
- 线程数 warning 不再在夹具模式默认出现。
- `--check` 不再只是结构校验，已能选择性检查 warning。

## 风险和下一步建议

- 风险：`--codex-home` 是强入口，调用者传错路径会读取错误数据源。建议桌面应用线调用时显式记录数据源路径并展示在诊断页。
- 风险：真实线程数在本轮从 290 变为 295，说明本地 Codex 状态变化快。建议回收线只验证“当前实测值”和 rollout 存在率，不把某个线程数当长期固定。
- 下一步：验证线补 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL；桌面应用线可开始使用 `--codex-home` 和 `codex-index.json` 做只读输入。
