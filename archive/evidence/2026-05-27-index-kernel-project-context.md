# 索引内核项目上下文补齐证据

## 结论先说

薄弱点：

- 本轮新增的是“候选元数据”，不是事实判定。`harness_candidates` 只说明发现了入口名或配置文件，不说明这些 harness 能跑、应该跑、适合当前项目。
- 当前真实索引仍有项目上下文 warning。依据：`--warning-summary` 输出 `project_root_missing=2`、`harness_candidates_truncated=1`。
- 当前真实 SQLite 线程数继续变化，本轮生成时是 296。依据：真实生成命令输出 `thread_count=296`。

可用结果：

- 项目对象已新增 `authority_files`、`handoff_files`、`evidence_files`、`harness_candidates`、`context_warnings`。
- 顶层 `source_stats.project_context` 已新增项目上下文扫描统计。
- 已新增离线项目上下文夹具测试，不依赖真实 `/Users/yoyi/.codex`。
- 真实 `codex-index.json` 已重新生成。

## 本轮读取范围

按任务包读取：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md`
- `product-line/handoffs/2026-05-27-index-kernel-edge-validation-review.md`
- `product-line/STAGE_PLAN.md`

真实索引生成时，只读检查了索引中项目根路径下的候选文件存在性和元数据。没有读取 README、AGENTS、handoff、evidence 正文进索引。

本轮没有读取或打印 `auth.json`、`.env`、授权文件、密钥、令牌。没有写真实 `/Users/yoyi/.codex`。

## 本轮写入

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_project_context.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-27-index-kernel-project-context.md`
- `product-line/handoffs/2026-05-27-index-kernel-project-context-result.md`

说明：`test_build_index_failures.py` 和 `test_build_index_edge_cases.py` 只修了 sqlite 测试连接关闭方式，避免 `ResourceWarning` 污染验证输出。

## 新增字段

项目对象新增：

- `authority_files`：README、AGENTS、CLAUDE、阶段计划、任务队列等入口候选，只含路径、类型、大小、更新时间、warning。
- `handoff_files`：handoff 候选，只含路径、类型、大小、更新时间、warning。
- `evidence_files`：evidence 候选，只含路径、类型、大小、更新时间、warning。
- `harness_candidates`：harness 入口候选，只含入口类型、来源、路径、名称、大小、更新时间、warning。
- `context_warnings`：项目上下文扫描 warning。

`source_stats` 新增：

- `project_context.role`
- `project_context.projects_scanned`
- `project_context.projects_missing`
- `project_context.authority_file_count`
- `project_context.handoff_file_count`
- `project_context.evidence_file_count`
- `project_context.harness_candidate_count`
- `project_context.context_warning_count`

## 扫描规则

稳定读取的候选：

- 项目根、`docs/`、`product-line/` 下的入口文件名：`README.md`、`AGENTS.md`、`CLAUDE.md`、`STAGE_PLAN.md`、`TASK_QUEUE.md`、`task-queue.md`、`current-state.md`、`decisions.md`、`open-questions.md`。
- 项目根、`docs/`、`product-line/` 下的 `handoff(s)` 目录内 `.md` 文件。
- 项目根、`docs/`、`product-line/` 下的 `evidence(s)` 目录内 `.md` 文件。
- `package.json` 的 script 名；不记录 script 命令正文。
- `Makefile` target 名；不记录 target 命令正文。
- `scripts/`、`tools/` 下常见脚本路径：`.js`、`.mjs`、`.cjs`、`.ts`、`.mts`、`.cts`、`.py`、`.sh`。
- 根目录配置入口：`vite.config.*`、`project.godot`、`pyproject.toml`、`package.json`。

默认跳过：

- `.git`
- `node_modules`
- `dist`
- `build`
- `coverage`
- `.next`
- `.nuxt`
- `.svelte-kit`
- `.turbo`
- `.venv`
- `__pycache__`
- `target`
- `vendor`

## 新增测试

新增文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_project_context.py`

新增 4 个项目上下文测试：

- 扫描 README / AGENTS / docs / handoff / evidence / package scripts / Makefile / scripts / Vite 配置，只输出元数据和入口名，不输出正文或脚本命令。
- 项目根目录不存在时，项目保留并记录 `project_root_missing`。
- authority 候选是越界 symlink 时，候选被阻断并记录 `symlink_outside_project:<path>`。
- handoff 候选超过上限时截断并记录 `handoff_candidates_truncated`。

原 17 个测试仍通过；新增后总数为 21。

## 验证命令和结果

完整测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py
```

结果：

```text
Ran 21 tests in 0.052s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py
```

结果：通过。

真实索引生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果摘要：

```json
{"memory_count": 11, "plugin_count": 11, "project_count": 30, "rollout_checked": 296, "rollout_existing": 296, "skill_count": 50, "thread_count": 296, "warning_count": 0}
```

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
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
{"harness_candidates_truncated": 1, "project_root_missing": 2, "title_truncated": 65}
```

## 当前真实索引统计

当前交付索引：

- `product-line/prototypes/index-kernel/codex-index.json`

当前统计：

- 生成时间：`2026-05-27T09:48:51Z`
- 线程数：296
- 项目数：30
- skills 数：50
- plugins 数：11
- memories 元数据入口数：11
- rollout 存在率：296/296
- authority 候选数：28
- handoff 候选数：12
- evidence 候选数：12
- harness 候选数：132
- 项目上下文 warning 数：3
- 缺失项目根：2

样例：

- `/Users/yoyi/workspace`：authority 2，handoff 12，evidence 6，harness 0，context warning 0。
- `/Users/yoyi/Desktop/kt-erp`：authority 6，evidence 5，harness 10，context warning 0。
- `/Users/yoyi/gamework`：authority 1，harness 40，context warning 包含 `harness_candidates_truncated`。

## 不确定候选

不能当事实的字段：

- `harness_candidates` 不能证明命令能运行或应该运行。
- `authority_files` 不能证明文件内容是当前权威，只能证明路径和类型是候选。
- `handoff_files`、`evidence_files` 不能证明这些文件仍有效，只能证明它们存在并符合候选路径规则。
- `project_root_missing` 不能证明项目已删除，只能证明当前扫描时路径不是目录。

## 风险

- 候选扫描是浅层规则，不会发现任意深层 handoff/evidence。
- package script 只保留 script 名，桌面应用线如果要展示命令正文，必须另开权限和脱敏设计。
- Makefile target 解析是保守正则，复杂 Makefile 可能漏报。
- 权限拒绝类项目目录在真实环境中未专门制造；现有降级主要来自 `OSError` 捕获和离线夹具。
