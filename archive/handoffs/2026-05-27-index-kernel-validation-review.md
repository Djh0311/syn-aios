# 索引内核异常夹具验证回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-index-kernel-validation.md`
- 开发线：验证线
- 测试文件：`product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- 回传 evidence：`product-line/evidence/2026-05-27-index-kernel-validation.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-index-kernel-validation-result.md`

## 结论

接受为异常夹具验证结果。

不把它解释成“索引内核已产品化”。它证明的是：当前原型在 9 个离线异常场景下能降级或给出 warning。

## 先说薄弱点

- 测试没有改 `build_index.py`，所以暴露的问题还没有修复。
- 测试通过导入模块后覆盖全局路径常量来注入假 Codex home，这不是正式产品入口。
- `validate_index()` 仍主要检查结构，不检查 warning 语义。
- `/Users/yoyi/.codex` 目录 mtime 在回收线复查时会变化，不能作为“测试未写真实 Codex 目录”的强证据。主证据应是测试代码把所有 Codex 路径指向临时目录。
- 本轮没有覆盖 SQLite 损坏文件、权限拒绝、符号链接绕过、超大 JSONL。

## 复核结果

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py` 通过，结果为 9 个测试全部通过。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py` 通过。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json` 输出 `validation_ok`。
- 测试文件使用 `tempfile.TemporaryDirectory()` 创建假 Codex home。
- 测试文件将 `CODEX_HOME`、`SQLITE_PATH`、`SESSION_INDEX_PATH`、`GLOBAL_STATE_PATH`、`SESSIONS_DIR`、`ARCHIVED_SESSIONS_DIR`、`SKILLS_DIR`、`PLUGIN_CACHE_DIR`、`MEMORIES_DIR` 指向临时目录。

## 已覆盖场景

- SQLite 文件不存在。
- SQLite 存在但没有 `threads` 表。
- `threads` 表缺少非关键字段。
- `threads` 表缺少 `id` 字段。
- `rollout_path` 指向不存在文件。
- `rollout_path` 指向允许目录外文件。
- `session_index.jsonl` 含坏 JSON 行。
- plugin manifest JSON 损坏。
- skill 文件编码异常。

## 当前生效结论

- 异常夹具测试可以进入索引内核固定验收清单。
- 索引内核下一步应进入修复线，补正式数据源注入和 warning 语义校验。
- 在修复完成前，桌面应用线可以使用 `codex-index.json` 作为只读样例，但不应直接依赖 `build_index.py` 的全局路径常量设计。

## 派生任务

- 新增修复线任务包：`product-line/tasks/2026-05-27-index-kernel-hardening.md`

## 状态

已回收，接受；派生修复线任务。
