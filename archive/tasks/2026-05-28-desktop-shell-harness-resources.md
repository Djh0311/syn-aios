# 任务包：桌面壳接入文件夹式 harness resources

## 任务名

在产品化桌面壳中接入索引内核新增的 `harness_resources`。

## 所属开发线

桌面应用线。

这是现有桌面应用线任务，不新增常设开发线。

## 背景

索引内核已新增 `projects[].harness_resources[]`，用于表达文件夹式 Codex harness 候选。

总指导回收结论：

- `harness_resources` 是候选资源，不是可运行事实。
- 真实索引里 `missing_manifest`、`missing_readme`、`missing_version` 等 warning 很多。
- 桌面应用线只能展示为候选资源，并必须展示 warning。

依据：

- `product-line/handoffs/2026-05-28-index-kernel-folder-harness-review.md`
- `product-line/evidence/2026-05-28-index-kernel-folder-harness.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`

## 目标

- 更新 `product-line/prototypes/productized-desktop-shell/` 的类型和数据映射，读取 `projects[].harness_resources[]`。
- Harness 管理页区分：
  - 文件夹级 harness resources。
  - 文件级 harness candidates。
- 项目详情工作流和右侧详情面板可以显示文件夹级 harness resources 数量和 warning。
- Harness 看板必须显示：
  - `display_name`
  - `root_path`
  - `harness_kind`
  - `agent_type`
  - `adapter_id`
  - `source_kind`
  - `capabilities`
  - `manifest_path`
  - `readme_path`
  - `version`
  - `entrypoints`
  - `permission_level`
  - `warnings`
- 缺 manifest / README / version / entrypoints 必须显式显示 warning。
- 保留旧 `harness_candidates` 显示，不破坏现有页面。
- 不新增运行按钮。
- 不把资源显示为“可用”或“已验证”。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-28-index-kernel-folder-harness-review.md`
- `product-line/evidence/2026-05-28-index-kernel-folder-harness.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不新增“运行 harness”按钮。
- 不把 `harness_resources` 标成已可用、已验证或已支持。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，或更新测试后通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- Harness 管理页能看到文件夹级 resources 和文件级 candidates 的区别。
- 缺 manifest / README / version / entrypoints 的 warning 能显示。
- 项目详情能显示 harness resource 数量和 warning。
- 不展示敏感内容。
- 不写 Codex 状态库。
- 不运行 harness。
- 验证后 5173 无监听残留。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增或修改了哪些测试
4. 如何读取 `harness_resources`
5. 如何区分 folder resource 和 file candidate
6. warning 如何展示
7. 是否新增任何运行能力
8. 是否触碰禁止事项
9. 验证命令和结果
10. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否正确接入 `harness_resources`。
- 是否保留旧 `harness_candidates`。
- 是否展示 warning。
- 是否没有把候选资源写成可用事实。
- 是否没有运行 harness。
- 是否没有展示正文或敏感内容。
