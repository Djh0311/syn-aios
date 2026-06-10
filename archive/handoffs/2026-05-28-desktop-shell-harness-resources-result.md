# 桌面壳接入 harness_resources 交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-desktop-shell-harness-resources.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`

## 结论

建议接受为“桌面壳已展示文件夹式 harness resources”。

不建议接受为“harness 可运行、已验证或已支持”。依据：本轮没有运行 harness，没有新增运行按钮，也没有写验证状态。

## 做了什么

- 更新前端类型，新增 `HarnessResource` 和 `HarnessEntrypoint`。
- 更新 Rust 数据映射，读取 `projects[].harness_resources[]`。
- Harness 管理页拆分文件夹级 resources 和文件级 candidates。
- Harness 管理页显示 resource 的完整元数据字段和 warning。
- 项目详情显示文件夹级 harness resource 数量和 warning。
- 保留旧文件级 `harness_candidates` 展示。
- 更新离线交互测试覆盖 resource / candidate 区分和 warning 展示。

## 改了哪些文件

- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`

## 新增或修改了哪些测试

修改：

- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

覆盖：

- 项目详情显示 `文件夹 harness resources`。
- 项目详情显示 `Resource warning`。
- Harness 管理页显示 `文件夹级 harness resources` 和 `文件级 harness candidates`。
- Harness 管理页显示 `display_name`、`root_path`、`harness_kind`、`agent_type`、`adapter_id`、`source_kind`、`capabilities`、`manifest_path`、`readme_path`、`version`、`entrypoints`、`permission_level`。
- Harness 管理页显示 `missing_manifest`、`missing_readme`、`missing_entrypoints`、`missing_version`。
- Harness 管理页显示“不新增运行按钮”“不自动运行 harness”“不代表可运行或已验证”。

## 如何读取 harness_resources

Rust 侧新增：

- `HarnessResource`
- `HarnessEntrypoint`
- `parse_harness_resources`
- `parse_harness_entrypoints`

前端侧新增：

- `HarnessResource`
- `HarnessEntrypoint`
- `ProjectRecord.harness_resources`

读取字段：

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
- 以及展示辅助字段 `size_bytes`、`updated_at_ms`

注意：真实索引里的 `entrypoints` 是对象数组，不是字符串数组。本轮按对象读取 `entry_type`、`name`、`path`、`source_kind`、`size_bytes`、`updated_at_ms`、`warnings`。

## 如何区分 folder resource 和 file candidate

Harness 管理页：

- `文件夹级 harness resources`：来自 `projects[].harness_resources[]`，显示为候选资源，未验证。
- `文件级 harness candidates`：来自 `projects[].harness_candidates[]`，显示为文件候选，兼容保留。

项目详情：

- `文件夹 harness resources` 显示 resource 数量和 warning 数量。
- `文件 harness candidates` 显示文件级候选数量。

## warning 如何展示

- Resource 卡片直接渲染 `warnings`。
- 缺失字段行显示“缺失”。
- 看板单独有“缺失 warning”列，解释 `missing_manifest`、`missing_readme`、`missing_version`、`missing_entrypoints`、`weak_harness_signal`。
- 文件级 candidates 也保留自己的 warning 展示。

## 是否新增任何运行能力

没有。

未新增：

- 运行 harness 按钮
- 自动运行 harness
- 验证状态写入
- 可用 / 已验证 / 已支持 状态

## 是否触碰禁止事项

未触碰。

- 未写 `/Users/yoyi/.codex`。
- 未改 Codex 状态库。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史、记忆正文。
- 未自动运行 harness。
- 未新增运行按钮。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、模型调度。
- 未做 release 打包。

## 验证命令和结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

端口：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 风险和下一步建议

- 真实 resource 里 warning 很多，后续不要直接按 resource 数量判断可用能力。
- TOML manifest 目前上游只记录路径，不解析版本和能力；桌面壳只能展示缺口。
- 后续如果要做“可运行”，需要先定义 manifest 规范、权限确认流程、验证结果存储和审计记录。
- 项目工作流未来应把 harness resource 建成 capability 节点，把 entrypoint 建成 validation 节点；这需要本地工作台事实层，不应只靠前端推断。
