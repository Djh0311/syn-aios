# 索引内核 hardening 回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-index-kernel-hardening.md`
- 开发线：索引内核线
- 修改脚本：`product-line/prototypes/index-kernel/build_index.py`
- 修改测试：`product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- 输出索引：`product-line/prototypes/index-kernel/codex-index.json`
- 回传 evidence：`product-line/evidence/2026-05-27-index-kernel-hardening.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-index-kernel-hardening-result.md`

## 结论

接受为索引内核 hardening 结果。

不把交付索引里的线程数当长期事实。Codex 真实线程数仍在变化：交付索引是 295 条；回收线显式 `--codex-home /Users/yoyi/.codex` 复核时已经是 296 条。

## 先说薄弱点

- 真实线程数变化快，不能把 295 写成稳定事实。
- 当前 `codex-index.json` 是生成时样例，不是实时数据库镜像。
- 对象级 warning 仍有 `title_truncated=64`，说明标题收紧仍在工作，也说明标题里可能有过长上下文。
- 仍未覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL。
- `threads.source` 仍未结构化解析。
- warning 语义校验只能检查 warning 有无，不能检查次数或复杂条件。

## 复核结果

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py` 通过，12 个测试全部通过。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py` 通过。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json` 输出 `validation_ok`。
- `--forbid-warning sqlite_thread_count_differs_from_expected` 输出 `validation_ok`。
- `--warning-summary` 输出 `{"title_truncated": 64}`。
- 显式 `--codex-home /Users/yoyi/.codex` 可用，回收线复核时输出线程数 296、项目数 30、skills 50、plugins 11、rollout 296/296。
- 测试文件已改为使用 `IndexSources.from_codex_home()`，没有继续覆盖模块全局路径常量。

## 当前生效结论

- `IndexSources` 是索引内核正式数据源注入入口。
- CLI 支持 `--codex-home <path>`。
- 线程数回归检查必须显式使用 `--expect-thread-count <n>`，不再默认硬套旧盘点数字。
- `--check` 支持 `--require-warning`、`--forbid-warning`、`--warning-summary`。
- 桌面应用线后续调用索引器时必须显式展示或记录实际读取的 Codex home。
- 桌面应用线可以使用 `codex-index.json` 作为静态样例，但运行时应重新生成或明确样例时间。

## 派生任务

- 新增验证线任务包：`product-line/tasks/2026-05-27-index-kernel-edge-validation.md`

## 状态

已回收，接受；进入边界异常补测。
