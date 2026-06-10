# 索引内核文件夹式 harness 扫描证据

## 结论先说

薄弱点：

- 新增的 `harness_resources` 仍是候选，不是“可用 harness”事实。依据：任务明确禁止自动运行 harness，也禁止判断 harness 是否真的可用。
- 真实索引里缺 manifest / README / version 的 resource 很多。依据：`--warning-summary` 输出 `missing_manifest=12`、`missing_readme=12`、`missing_version=14`。
- `/Users/yoyi` 这类宽项目根会带来弱信号目录候选。依据：真实索引中 `/Users/yoyi` 产生 4 个 derived harness resource，均缺 manifest、README、entrypoints、version。

可用结果：

- 保留原 `harness_candidates` 文件级候选。
- 项目对象新增 `harness_resources` 文件夹级候选。
- `harness_resources` 包含扩展字段：`agent_type`、`adapter_id`、`source_kind`、`capabilities`、`permission_level`。
- `source_stats.project_context` 新增 `harness_resource_count` 和 `harness_resource_warning_count`。
- 已新增文件夹式 harness 离线测试。

## 本轮读取范围

按任务包读取：

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/handoffs/2026-05-28-codex-workflow-min-model-review.md`
- `product-line/prototypes/index-kernel/`
- `product-line/prototypes/index-kernel/codex-index.json`

真实索引生成时，只读检查了项目根内浅层文件夹、manifest / README / entrypoint 的存在性和元数据。没有运行 harness，没有读取命令输出，没有展示敏感文件内容。

本轮没有写 `/Users/yoyi/.codex`，没有改真实 Codex 状态库。

## 本轮写入

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-28-index-kernel-folder-harness.md`
- `product-line/handoffs/2026-05-28-index-kernel-folder-harness-result.md`

Python 运行测试时生成过 `__pycache__`，已清理；交付目录只保留脚本、索引和测试文件。

## 新增输出字段

项目对象新增：

- `harness_resources`

每个 `harness_resources[]` 对象字段：

- `root_path`
- `display_name`
- `harness_kind`
- `source_kind`
- `agent_type`
- `adapter_id`
- `capabilities`
- `entrypoints`
- `manifest_path`
- `readme_path`
- `version`
- `size_bytes`
- `updated_at_ms`
- `permission_level`
- `warnings`

当前 Codex-only 填值：

- `agent_type=codex`
- `adapter_id=codex-local`
- `permission_level=read_only`

但 schema 没写成 Codex-only；未来可以换 agent / adapter / source。

## 识别规则

候选目录来源：

- 项目根下一层，目录名含 `harness`、`validation`、`verify`、`codex`。
- `scripts/`、`tools/`、`.codex/`、`harness/`、`tests/` 下的一层子目录。
- `harness/` 和 `.codex/` 目录本身。
- 旧 `harness_candidates` 文件路径的父目录，但只有父目录带更强 resource 信号时才升格。

强信号：

- 目录名含 `harness`、`validation`、`verify`、`codex`。
- 或目录内有 manifest / README。

entrypoints：

- manifest：`harness.json`、`harness.toml`、`codex-harness.json`、`codex-harness.toml`、`manifest.json`、`manifest.toml`、`package.json`
- README：`README.md`、`README`
- 常见脚本：`.js`、`.mjs`、`.cjs`、`.ts`、`.mts`、`.cts`、`.py`、`.sh`

只读取：

- manifest JSON 的轻量结构字段：`name`、`display_name`、`version`、`harness_kind`、`kind`、`type`、`capabilities`、`agent_type`、`adapter_id`
- 文件元数据：路径、大小、更新时间

不输出：

- README 正文
- 脚本正文
- manifest 中的命令正文
- 任何 harness 执行结果

## 降级 warning

resource 级 warning：

- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`
- `weak_harness_signal`
- `entrypoints_truncated`

项目级 warning：

- `harness_resources_truncated`
- `harness_resource_outside_project:<path>`
- 复用已有 symlink / 权限 / 目录读取 warning

## 新增测试

新增文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py`

新增 5 个测试：

- 有 manifest / README / entrypoint 的文件夹式 harness 输出完整 resource，且不输出 README / 脚本 / manifest 命令正文。
- 缺 manifest 的文件夹式 harness 仍保留为候选，并给 `missing_manifest` / `missing_version`。
- 无入口的文件夹式 harness 给 `missing_entrypoints`。
- 普通目录没有 harness 信号时不升格为 resource。
- 旧 `harness_candidates` 文件级候选仍保留，`scripts/verify.py` 不会把普通 `scripts/` 目录误判为 folder harness。

## 验证命令和结果

完整测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：

```text
Ran 26 tests in 0.061s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：通过。

真实索引生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果摘要：

```json
{"memory_count": 11, "plugin_count": 11, "project_count": 32, "rollout_checked": 310, "rollout_existing": 310, "skill_count": 51, "thread_count": 310, "warning_count": 0}
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
{"entrypoints_truncated": 4, "handoff_candidates_truncated": 1, "harness_candidates_truncated": 1, "missing_entrypoints": 6, "missing_manifest": 12, "missing_readme": 12, "missing_version": 14, "project_root_missing": 2, "title_truncated": 76, "weak_harness_signal": 1}
```

## 当前真实索引统计

当前交付索引：

- `product-line/prototypes/index-kernel/codex-index.json`

统计：

- 生成时间：`2026-05-28T12:41:43Z`
- 线程数：310
- 项目数：32
- skills 数：51
- plugins 数：11
- rollout 存在率：310/310
- 文件级 harness 候选数：132
- 文件夹级 harness resource 数：14
- harness resource warning 数：49

## 避免误判普通目录的措施

- 只做浅层扫描，不全项目递归。
- 跳过 `.git`、`node_modules`、`dist`、`build`、缓存目录等。
- 普通 `scripts/` 目录不会因为存在 `verify.py` 自动升格为 `harness_resources`。
- 旧文件级候选的父目录只有在目录名含 harness / validation / verify / codex 或有 manifest / README 时才升格。
- 缺 manifest / README / entrypoints / version 时明确 warning，不写成完整支持。

## 风险

- 部分 derived resource 可能是弱候选，需要桌面应用展示 warning，不能当已支持 harness。
- 当前只解析 JSON manifest 的轻量字段；TOML manifest 只记录路径，不解析版本和能力。
- 文件夹式 harness manifest 规范仍未定，后续需要单独决策。
- 真实线程数和项目数继续变化，310 / 32 只代表本轮生成时状态。
