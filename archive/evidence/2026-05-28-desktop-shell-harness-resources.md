# 桌面壳接入 harness_resources 证据

## 结论

薄弱点先说：

- `harness_resources` 仍是候选资源，不是可运行事实。依据：索引内核回收意见明确不接受为 harness 可用性验证完成。
- 真实索引里缺口很多。依据：索引内核 evidence 记录 `missing_manifest=12`、`missing_readme=12`、`missing_entrypoints=6`、`missing_version=14`。
- `entrypoints` 真实形状是对象数组，不是字符串数组。依据：本轮抽查 `codex-index.json`，entrypoint 对象包含 `entry_type`、`name`、`path`、`size_bytes`、`source_kind`、`updated_at_ms`、`warnings`。
- 本轮只做展示和 warning，不新增运行按钮、不自动运行 harness、不写验证状态。

可接受点：

- 桌面壳类型和 Rust 映射已读取 `projects[].harness_resources[]`。
- Harness 管理页区分文件夹级 `harness_resources` 和文件级 `harness_candidates`。
- Harness 管理页显示任务要求字段：`display_name`、`root_path`、`harness_kind`、`agent_type`、`adapter_id`、`source_kind`、`capabilities`、`manifest_path`、`readme_path`、`version`、`entrypoints`、`permission_level`、`warnings`。
- 缺 manifest / README / version / entrypoints 的 warning 会直接展示。
- 项目详情显示文件夹级 harness resource 数量和 warning。
- 旧 `harness_candidates` 显示保留。

## 本轮读取依据

- `product-line/tasks/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-index-kernel-folder-harness-review.md`
- `product-line/evidence/2026-05-28-index-kernel-folder-harness.md`
- `product-line/decisions/2026-05-28-codex-workflow-min-model.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/`

没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文、工具输出、命令输出、输入历史、记忆正文

## 修改文件

- `product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/views/HarnessBoardView.tsx`
- `product-line/prototypes/productized-desktop-shell/src/styles.css`
- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`

## 如何读取 harness_resources

前端类型新增：

- `HarnessEntrypoint`
- `HarnessResource`
- `ProjectRecord.harness_resources`

Rust 映射新增：

- `HarnessEntrypoint`
- `HarnessResource`
- `parse_harness_resources`
- `parse_harness_entrypoints`

读取来源：

- `projects[].harness_resources[]`

字段映射：

- `root_path`
- `display_name`
- `harness_kind`
- `agent_type`
- `adapter_id`
- `source_kind`
- `capabilities`
- `manifest_path`
- `readme_path`
- `version`
- `entrypoints[]`
- `permission_level`
- `size_bytes`
- `updated_at_ms`
- `warnings`

## 如何区分 resource 和 candidate

Harness 管理页分两块：

- 文件夹级 harness resources：来自 `projects[].harness_resources[]`，标为“候选资源，未验证”。
- 文件级 harness candidates：来自 `projects[].harness_candidates[]`，标为“文件候选”，兼容保留。

项目适配列也分别显示：

- `resources`
- `candidates`

## warning 展示

Resource 卡片直接显示 `warnings` 数组。

缺字段也在字段行上显式显示：

- `manifest_path` 缺失显示“缺失”。
- `readme_path` 缺失显示“缺失”。
- `version` 缺失显示“缺失”。
- `entrypoints` 为空显示“缺失”。

看板新增“缺失 warning”列，解释：

- `missing_manifest`
- `missing_readme`
- `missing_version`
- `missing_entrypoints`
- `weak_harness_signal`

## 测试

更新：

- `product-line/prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

新增覆盖：

- 项目详情显示文件夹级 harness resources。
- 项目详情显示 resource warning。
- Harness 看板显示文件夹级 resources 和文件级 candidates 的区别。
- Harness 看板显示 `display_name`、`root_path`、`harness_kind`、`agent_type`、`adapter_id`、`source_kind`、`capabilities`、`manifest_path`、`readme_path`、`version`、`entrypoints`、`permission_level`。
- Harness 看板显示 `missing_manifest`、`missing_readme`、`missing_version`、`missing_entrypoints`。
- Harness 看板显示“不新增运行按钮”“不自动运行 harness”“不代表可运行或已验证”。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline`

端口检查：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 禁止事项检查

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未新增“运行 harness”按钮。
- 未把 `harness_resources` 标成已可用、已验证或已支持。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、模型调度。
- 未做 release 打包。

## 风险

- 真实索引里部分 resource 是弱候选，桌面壳只能展示 warning，不能判断真假。
- 当前只展示 entrypoint 元数据，不读取 entrypoint 文件内容，不解析命令语义。
- 如果后续索引 schema 继续变化，桌面壳需要同步更新类型。
