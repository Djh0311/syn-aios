# 索引内核边界异常补测回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-index-kernel-edge-validation.md`
- 开发线：验证线
- 新增测试：`product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- 回传 evidence：`product-line/evidence/2026-05-27-index-kernel-edge-validation.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-index-kernel-edge-validation-result.md`

## 结论

接受为索引内核边界异常补测结果。

这个结论只表示：当前索引内核在本轮 5 个新增边界场景和原有 12 个异常夹具场景下通过复核。它不表示索引内核已经覆盖阶段 1 的所有数据范围。

## 先说薄弱点

- 本轮没有修改 `build_index.py`。依据：验证线回传和 evidence 都说明允许写入不包含内核脚本。
- 大 `session_index.jsonl` 测试只证明正文类字段没有进入索引输出，不证明逐行 `json.loads()` 的性能足够。依据：`parse_session_index()` 仍按行解析 JSON。
- 权限拒绝只覆盖临时假 `.codex-global-state.json`，没有覆盖 plugin、skill、memory 路径。依据：新增测试文件只有一个权限拒绝用例，目标是 `global_state_path`。
- 权限拒绝测试保留 skip 分支，说明它不是跨系统强保证。依据：测试里在 `chmod 000` 未触发 `PermissionError` 时调用 `skipTest()`。
- 当前索引 JSON 没有 `harness` 字段，也没有项目内 README / AGENTS / handoff / evidence 候选字段。依据：`codex-index.json` 顶层字段为 `generated_at`、`warnings`、`threads`、`projects`、`skills`、`plugins`、`memories`、`source_stats`。

## 复核结果

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py` 通过，12 个测试全部通过。
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py` 通过，5 个测试全部通过。
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py` 通过，17 个测试全部通过。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py` 通过。

## 已覆盖场景

- SQLite 损坏文件。
- 临时假 global state 权限拒绝。
- `rollout_path` 通过 symlink 指向允许目录外文件。
- 大 rollout JSONL 正文不被打开或序列化。
- 大 `session_index.jsonl` 正文类字段不进入索引输出。

## 当前生效结论

- 边界异常测试进入索引内核固定验收清单。
- `rollout_path` 越界判断必须继续在文件存在性检查前执行。
- `session_index.jsonl` 只能作为辅助来源，不能当线程权威来源。
- 不应因为本轮测试通过就进入完整桌面应用实现；阶段 1 仍缺项目上下文和 harness 候选扫描。

## 派生任务

- 新增索引内核线任务包：`product-line/tasks/2026-05-27-index-kernel-project-context.md`

## 状态

已回收，接受；进入索引内核项目上下文补齐。
