# 索引内核文件夹式 harness 扫描交接

## 状态

任务完成，可进入回收。

## 做了什么

- 在项目对象上新增 `harness_resources` 文件夹级 harness 候选。
- 保留原有 `harness_candidates` 文件级候选。
- 给 folder-level harness 增加扩展字段：`agent_type`、`adapter_id`、`source_kind`、`capabilities`、`permission_level`。
- 新增文件夹式 harness 测试。
- 重新生成 `codex-index.json`。

## 改了哪些文件

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/2026-05-28-index-kernel-folder-harness.md`

## 新增或修改了哪些测试

新增测试文件：

- `product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py`

新增 5 个测试：

- manifest / README / entrypoint 完整资源。
- 缺 manifest 的资源 warning。
- 无入口资源 warning。
- 普通目录不误判为 resource。
- 旧文件级 `harness_candidates` 兼容保留。

当前总测试数：26。

## 新增的 folder-level harness 字段

项目对象新增：

- `harness_resources`

resource 字段：

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

## 如何识别文件夹式 Codex harness

当前只做候选识别：

- 项目根下一层目录名含 `harness`、`validation`、`verify`、`codex`。
- `scripts/`、`tools/`、`.codex/`、`harness/`、`tests/` 下的一层子目录。
- `harness/` 和 `.codex/` 目录本身。
- 文件级 `harness_candidates` 的父目录，前提是父目录有 harness 命名、manifest 或 README 等更强信号。

当前 Codex-only 默认：

- `agent_type=codex`
- `adapter_id=codex-local`
- `permission_level=read_only`

## 如何避免误判普通目录

- 不全项目递归。
- 跳过 `.git`、`node_modules`、`dist`、`build` 和缓存目录。
- 普通 `scripts/verify.py` 只保留为文件级候选，不把 `scripts/` 自动升格为 `harness_resources`。
- 缺 manifest、README、entrypoints、version 时写 warning。

## 哪些字段来自文件元数据，哪些仍是缺口

来自元数据：

- `root_path`
- `manifest_path`
- `readme_path`
- `entrypoints[].path`
- `size_bytes`
- `updated_at_ms`

来自轻量 manifest JSON：

- `display_name`
- `version`
- `harness_kind`
- `capabilities`
- `agent_type`
- `adapter_id`

仍是缺口：

- TOML manifest 内容未解析。
- harness manifest 规范未定。
- 未验证 harness 是否可运行。
- 未记录最近验证状态。
- 未判断 harness 是否真的适合当前项目。

## 是否触碰禁止事项

没有。

- 没有写 `/Users/yoyi/.codex`。
- 没有改真实 Codex 状态库。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有自动运行 harness。
- 没有接入非 Codex agent。
- 没有做桌面 UI 改动。

## 验证命令和结果

完整测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：26 tests OK。

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：通过。

真实索引生成：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --pretty
```

结果：线程 310，项目 32，rollout 310/310。

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：`validation_ok`。

warning 汇总：

```json
{"entrypoints_truncated": 4, "handoff_candidates_truncated": 1, "harness_candidates_truncated": 1, "missing_entrypoints": 6, "missing_manifest": 12, "missing_readme": 12, "missing_version": 14, "project_root_missing": 2, "title_truncated": 76, "weak_harness_signal": 1}
```

## 当前索引统计

- 线程数：310
- 项目数：32
- skills 数：51
- plugins 数：11
- 文件级 harness 候选数：132
- 文件夹级 harness resources：14
- harness resource warnings：49

## 风险和下一步建议

- 风险：真实索引中 derived resource warning 很多，桌面应用线必须展示 warning，不能把它们当完整支持。
- 风险：TOML manifest 未解析，manifest 规范也未定。
- 风险：`/Users/yoyi` 这类宽项目根会产生弱候选，后续可能需要项目根过滤或人工隐藏。
- 下一步建议：先由总指导回收；若接受，桌面应用线可以读取 `harness_resources`，但只能显示为候选资源，不允许运行。
