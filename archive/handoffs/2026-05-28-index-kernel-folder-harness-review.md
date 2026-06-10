# 索引内核文件夹式 harness 扫描总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-index-kernel-folder-harness.md`
- 开发线：索引内核线
- Evidence：`product-line/evidence/2026-05-28-index-kernel-folder-harness.md`
- Handoff：`product-line/handoffs/2026-05-28-index-kernel-folder-harness-result.md`
- 被回收产物：`product-line/prototypes/index-kernel/`

## 结论

接受为“文件夹式 Codex harness 候选扫描增强”。

不接受为“harness 可用性验证完成”，不接受为“harness 管理完成”，不接受为“自动运行 harness 能力完成”。

依据：

- 项目对象新增 `harness_resources` 文件夹级候选。
- 原有 `harness_candidates` 文件级候选保留。
- `harness_resources` 包含 `agent_type`、`adapter_id`、`source_kind`、`capabilities`、`permission_level` 等高扩展字段。
- 新增 5 个 folder harness 测试。
- 总指导线复跑 26 个测试通过。
- 总指导线复跑 `build_index.py --check` 返回 `validation_ok`。
- warning 汇总明确仍有 `missing_manifest=12`、`missing_readme=12`、`missing_version=14` 等缺口。

## 先说薄弱点

- `harness_resources` 仍是候选，不是可运行事实。依据：任务禁止自动运行 harness，也禁止判断 harness 是否真的可用。
- 真实索引里弱候选不少。依据：warning 汇总有 `missing_manifest=12`、`missing_readme=12`、`missing_entrypoints=6`、`missing_version=14`、`weak_harness_signal=1`。
- `/Users/yoyi` 这类宽项目根会产生弱候选。依据：evidence 记录 `/Users/yoyi` 产生 4 个 derived harness resource。
- TOML manifest 未解析。依据：handoff 明确 TOML manifest 只记录路径，不解析版本和能力。
- harness manifest 规范仍未定。依据：handoff 明确这是后续风险。

## 接受内容

接受新增输出：

- `projects[].harness_resources[]`

接受 resource 字段：

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

接受当前 Codex-only 默认：

- `agent_type=codex`
- `adapter_id=codex-local`
- `permission_level=read_only`

接受兼容性判断：

- 旧 `harness_candidates` 文件级候选仍保留。
- 普通 `scripts/verify.py` 不会把普通 `scripts/` 目录误判为 folder harness。

## 总指导线复跑验证

完整测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：

- 26 tests OK。

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py product-line/prototypes/index-kernel/tests/test_build_index_project_context.py product-line/prototypes/index-kernel/tests/test_build_index_folder_harness.py
```

结果：

- 通过。

结构校验：

```bash
python3 product-line/prototypes/index-kernel/build_index.py --check product-line/prototypes/index-kernel/codex-index.json
```

结果：

- `validation_ok`

warning 汇总：

```json
{"entrypoints_truncated": 4, "handoff_candidates_truncated": 1, "harness_candidates_truncated": 1, "missing_entrypoints": 6, "missing_manifest": 12, "missing_readme": 12, "missing_version": 14, "project_root_missing": 2, "title_truncated": 76, "weak_harness_signal": 1}
```

索引抽查：

- 生成时间：`2026-05-28T12:41:43Z`
- 线程数：310
- 项目数：32
- skills 数：51
- plugins 数：11
- 文件级 harness 候选数：132
- 文件夹级 harness resource 数：14

这些数量只代表本轮生成时状态，不作为长期固定事实。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有写 `/Users/yoyi/.codex`。
- 没有改真实 Codex 状态库。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有自动运行 harness。
- 没有接入非 Codex agent。
- 没有做桌面 UI 改动。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 索引内核已能输出文件夹级 harness 候选。
- 文件级 `harness_candidates` 兼容保留。
- 桌面应用线可以读取 `harness_resources`。

仍不能说：

- harness 已可运行。
- harness manifest 规范已定。
- TOML manifest 已解析。
- 桌面应用已展示新的 `harness_resources`。
- harness 管理完成。

## 下一步

下一步派给桌面应用线：接入 `harness_resources`，只做候选展示和 warning 展示。

约束：

- 不运行 harness。
- 不把弱候选显示成已支持。
- 必须展示 manifest / README / version / entrypoints 缺失 warning。
- 必须保留文件级候选和文件夹级资源的区别。
