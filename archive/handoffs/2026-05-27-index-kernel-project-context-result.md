# 索引内核项目上下文补齐交接

## 状态

任务完成，可进入回收。

## 做了什么

- 在项目对象上补了只读上下文候选字段。
- 增加项目上下文扫描统计。
- 增加离线项目上下文夹具测试。
- 重新生成 `codex-index.json`。
- 清理了测试输出里的 sqlite `ResourceWarning`。

## 改了哪些文件

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_project_context.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-index-kernel-project-context.md`

## 新增字段和测试

项目对象新增字段：

- `authority_files`
- `handoff_files`
- `evidence_files`
- `harness_candidates`
- `context_warnings`

`source_stats` 新增：

- `project_context`

新增测试文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_project_context.py`

新增 4 个测试，总测试数从 17 增至 21。

## 哪些项目上下文候选能稳定读取

稳定读取的是元数据，不是正文：

- README / AGENTS / CLAUDE / STAGE_PLAN / task queue 等入口文件候选。
- handoff / evidence 目录下 `.md` 文件候选。
- package script 名。
- Makefile target 名。
- scripts / tools 下常见脚本路径。
- Vite / Godot / Python / Node 配置入口路径。

## 哪些字段只是候选，不能当事实

- `authority_files` 只是权威入口候选，不说明内容仍然有效。
- `handoff_files` 和 `evidence_files` 只是文件候选，不说明已经回收或接受。
- `harness_candidates` 只是入口候选，不说明命令可运行、适合运行或应该运行。
- `context_warnings` 是扫描时状态，不是长期事实。

## 验证结果

完整测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py
```

结果：21 tests OK。

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py
```

结果：通过。

真实索引生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果：线程 296，项目 30，rollout 296/296。

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：`validation_ok`。

warning 汇总：

```json
{"harness_candidates_truncated": 1, "project_root_missing": 2, "title_truncated": 65}
```

## 当前索引统计

- 线程数：296
- 项目数：30
- skills 数：50
- plugins 数：11
- authority 候选数：28
- handoff 候选数：12
- evidence 候选数：12
- harness 候选数：132
- 项目上下文 warning 数：3

## 是否建议桌面应用线开始接静态索引

建议可以开始接静态索引，但只能按候选展示，不要做自动判断。

可以接：

- 项目页的 authority / handoff / evidence 候选列表。
- harness 页的候选入口台账。
- 项目上下文 warning 展示。

不要接成：

- 自动运行 harness。
- 自动判定项目类型。
- 自动判定文件是当前权威。
- 展示 README / AGENTS / handoff / evidence 正文。

## 风险和下一步建议

- 风险：候选扫描规则偏浅，可能漏掉深层目录。
- 风险：Makefile target 解析保守，复杂 Makefile 可能漏报。
- 风险：package script 只保留名字，后续如果要看命令正文必须另开脱敏设计。
- 下一步：桌面应用线可以用当前 `codex-index.json` 做只读页面输入；验证线后续可补权限拒绝和超大项目目录夹具。
