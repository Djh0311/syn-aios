# 索引内核项目上下文补齐回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-index-kernel-project-context.md`
- 开发线：索引内核线
- 修改脚本：`product-line/prototypes/index-kernel/build_index.py`
- 新增测试：`product-line/prototypes/index-kernel/tests/test_build_index_project_context.py`
- 回传 evidence：`product-line/evidence/2026-05-27-index-kernel-project-context.md`
- 回传 handoff：`product-line/handoffs/2026-05-27-index-kernel-project-context-result.md`

## 结论

接受为索引内核项目上下文补齐结果。

这个结论只表示：当前索引可以给桌面应用线提供项目上下文候选。它不表示 harness 能运行，也不表示 authority / handoff / evidence 文件就是当前权威。

## 先说薄弱点

- 新增字段都是候选元数据，不是事实判定。依据：回传 handoff 明确写明 `authority_files`、`handoff_files`、`evidence_files`、`harness_candidates` 都不能当已接受事实。
- 为了抽取 package script 名和 Makefile target 名，索引器会读取 `package.json` 和 `Makefile`，但不输出命令正文。依据：`scan_package_scripts()` 和 `scan_makefile_targets()` 读取文件；新增测试用 `echo should-not-be-indexed` 验证命令正文没有进入索引。
- 当前真实索引仍有对象级 warning：`harness_candidates_truncated=1`、`project_root_missing=2`、`title_truncated=65`。依据：回收线复跑 `--warning-summary`。
- 项目扫描规则偏浅，复杂 Makefile、深层目录、特殊项目结构可能漏报。依据：回传 evidence 的风险说明。
- 真实项目目录权限拒绝没有专门制造；本轮主要靠离线夹具和 `OSError` 分支。依据：回传 evidence 的风险说明。

## 复核结果

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py` 通过，21 个测试全部通过。
- `python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py` 通过。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json` 输出 `validation_ok`。
- `python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json --warning-summary` 输出 `{"harness_candidates_truncated": 1, "project_root_missing": 2, "title_truncated": 65}`。
- 回收线复跑真实索引生成：线程 296、项目 30、skills 50、plugins 11、rollout 296/296。

## 字段复核

项目对象已包含：

- `authority_files`
- `handoff_files`
- `evidence_files`
- `harness_candidates`
- `context_warnings`

`source_stats` 已包含：

- `project_context`

当前样例索引统计：

- authority 候选数：28
- handoff 候选数：12
- evidence 候选数：12
- harness 候选数：132
- 项目上下文 warning 数：3

## 当前生效结论

- `codex-index.json` 可以作为桌面应用线第一包的静态输入。
- 桌面应用线只能把 authority / handoff / evidence / harness 展示成候选。
- 桌面应用线必须展示 `context_warnings`，不能吞掉 `project_root_missing`、`harness_candidates_truncated`、`title_truncated`。
- 桌面应用线不能展示 README / AGENTS / handoff / evidence 正文。
- 桌面应用线不能自动运行 harness，不能自动判定项目类型，不能自动判定当前权威。

## 派生任务

- 新增桌面应用线任务包：`product-line/tasks/2026-05-27-desktop-app-static-index-shell.md`

## 状态

已回收，接受；阶段 1 索引内核收口完成到可供静态桌面壳接入的程度。下一步进入桌面应用线第一包。
