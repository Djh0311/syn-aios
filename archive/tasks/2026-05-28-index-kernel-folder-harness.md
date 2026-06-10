# 任务包：索引内核补文件夹式 Codex harness 扫描

## 任务名

为索引内核补充文件夹式 Codex harness 扫描和输出字段。

## 所属开发线

索引内核线。

这是现有索引内核线任务，不新增常设开发线。

## 背景

用户补充：当前自制 harness 基本针对 Codex，且以文件夹形式存在。当前索引的 `harness_candidates` 更偏文件级候选，不能完整表达文件夹式 harness。

阶段 3 最小工作流模型已经回收：

- Harness 应建模为 `ProjectCapability(capability_type=harness)` 和 `HarnessResource`。
- 文件夹根应作为 `HarnessResource.root_path`。
- 内部脚本、配置、README、manifest 应作为 `entrypoints` 或 `Artifact`。
- 与 Codex 的关系通过 `agent_type=codex`、`adapter_id=codex-local`、`capabilities` 表达。

依据：

- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/handoffs/2026-05-28-codex-workflow-min-model-review.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/prototypes/index-kernel/build_index.py`

## 目标

- 在索引内核中新增文件夹式 harness 扫描。
- 保留现有 `harness_candidates` 文件级候选，不破坏桌面应用现有读取。
- 新增 folder-level 输出结构，建议命名为 `harness_resources` 或等价字段。
- 每个文件夹式 harness 至少输出：
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
  - `warnings`
- 当前 Codex-only 默认：
  - `agent_type=codex`
  - `adapter_id=codex-local`
  - `source_kind=project_file` 或 `derived`
- 对缺 manifest、缺 README、无入口、目录过深、权限失败、疑似非 harness 目录给出 warning。
- 更新 `codex-index.json`。
- 增加测试覆盖文件夹式 harness。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/decisions/2026-05-28-extensible-first-development-rule.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/handoffs/2026-05-28-codex-workflow-min-model-review.md`
- `product-line/prototypes/index-kernel/`
- `product-line/prototypes/index-kernel/codex-index.json`

## 允许写入

- `product-line/prototypes/index-kernel/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不判断 harness 是否真的可用。
- 不把缺 manifest 的目录写成完整支持。
- 不把文件夹式 harness 写成 Codex-only 硬编码 schema；必须保留 `agent_type`、`adapter_id`、`source_kind`、`capabilities` 这类扩展字段。
- 不接入非 Codex agent。
- 不做桌面 UI 改动。
- 不做知识库、向量搜索、模型调度。

## 建议扫描口径

候选目录来源可以包括但不限于：

- 项目根下名字含 `harness`、`validation`、`verify`、`codex` 的目录。
- `scripts/`、`tools/`、`.codex/`、`harness/`、`tests/` 下含 manifest / README / 常见入口的目录。
- 已有 `harness_candidates` 文件路径的父目录。

必须保守：

- 默认浅层扫描。
- 跳过 `node_modules`、`.git`、`dist`、`build`、缓存目录。
- 不读取大文件正文。
- README / manifest 只读元数据或轻量结构字段，不展示长正文。
- 不跨出项目根。

## 验收标准

- 有 evidence 和 handoff。
- `build_index.py --check` 通过。
- 原有测试通过。
- 新增文件夹式 harness 测试通过。
- `codex-index.json` 仍包含原 `harness_candidates`，并新增文件夹级 harness 输出。
- 输出结构包含高扩展字段：`agent_type`、`adapter_id`、`source_kind`、`capabilities`。
- 缺 manifest、缺入口、权限失败等场景有 warning。
- 不读取或展示敏感内容。
- 不写 `/Users/yoyi/.codex`。
- 不自动运行 harness。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增或修改了哪些测试
4. 新增的 folder-level harness 字段是什么
5. 如何识别文件夹式 Codex harness
6. 如何避免误判普通目录
7. 哪些字段来自文件元数据，哪些仍是缺口
8. 是否触碰任何禁止事项
9. 验证命令和结果
10. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否正确补了文件夹式 harness 扫描。
- 是否保留旧 `harness_candidates` 兼容性。
- 是否满足高扩展开发规则。
- 是否没有把缺字段写成已完成。
- 是否没有读取正文或敏感内容。
- 是否没有自动运行 harness。
