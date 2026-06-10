# 索引内核边界异常补测交接

## 状态

验证线任务已完成，可回收。

## 做了什么

- 增加索引内核边界异常测试。
- 覆盖 SQLite 损坏文件。
- 覆盖权限拒绝场景。
- 覆盖 rollout_path 符号链接绕过。
- 覆盖大 rollout JSONL 正文不被默认读取。
- 覆盖大 session_index JSONL 的正文类字段不进入索引输出。

## 改了哪些文件

- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/evidence/2026-05-27-index-kernel-edge-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-edge-validation-result.md`

未改：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/codex-index.json`

## 新增了哪些测试

新增 5 个测试：

- `test_corrupt_sqlite_file_degrades_with_sqlite_warning`
- `test_unreadable_global_state_records_warning_or_skips_when_permissions_are_not_enforced`
- `test_rollout_symlink_inside_sessions_to_outside_file_is_blocked`
- `test_large_rollout_jsonl_body_is_not_opened_or_serialized`
- `test_large_session_index_payload_is_not_serialized`

新增测试文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`

## 哪些边界场景通过

已通过：

- SQLite 损坏文件：不崩，产生 `sqlite_open_failed:` 或 `sqlite_read_failed:` warning。
- 权限拒绝：临时假 global state chmod `000` 后，当前环境产生 `read_failed:<path>:PermissionError` warning。
- rollout symlink 越界：`sessions/` 内 symlink 指向外部文件，被 `is_relative_to(resolve)` 阻断。
- 大 rollout 正文：索引器只检查存在性，不打开 rollout 文件，正文 sentinel 未进入索引。
- 大 session_index 正文类字段：索引器只保留 id 统计，`first_user_message`、`preview`、`payload.content` sentinel 未进入索引。

## 哪些场景无法稳定模拟

本轮没有无法稳定模拟的场景。

说明：

- 权限拒绝在当前环境可稳定触发，所以测试通过。
- 测试中保留 skip 分支，因为不同系统或用户权限模型可能让 `chmod 000` 仍可读取文件。

## 验证命令

原 12 个测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 12 tests in 0.024s
OK
```

新增边界测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：

```text
Ran 5 tests in 0.012s
OK
```

联合测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：

```text
Ran 17 tests in 0.031s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：通过。

## 是否建议修改索引内核

本轮不建议立刻修改内核。

依据：

- 任务包要求的边界场景已经有降级、阻断或不输出正文的行为。
- 本轮允许写入不包含 `build_index.py`，所以没有动内核。

后续可考虑：

- 给 `parse_session_index()` 加行长上限或轻量字段抽取，降低超大 JSONL 解析成本。
- 扩展权限拒绝测试到 plugin manifest、skill、memory。
- 扩展 symlink 测试到目录 symlink 和 plugin skill symlink。

## 风险和下一步建议

风险：

- 大 session_index 测试只证明不把正文写入索引，不证明性能足够。
- 大 rollout 测试依赖 mock `Path.open()`，未来如果换别的读取 API，需要更新测试。
- 权限拒绝在其他机器可能被 skip，不应把它当跨平台强保证。

下一步：

- 回收线复跑 17 个测试。
- 接受后把边界测试加入索引内核固定验收清单。
- 性能问题另开任务，不要混进本轮边界异常补测。
