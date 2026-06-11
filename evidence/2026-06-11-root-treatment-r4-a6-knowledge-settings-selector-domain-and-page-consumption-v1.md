# Root Treatment / R4-A6 Knowledge Settings Selector Domain And Page Consumption v1 Evidence

日期：2026-06-11

状态：已完成并通过复核线 `STATUS: CLEAR`。

任务包：`tasks/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1.md`

Planning baseline commit：`c248f9bb390458ba64f2a809ec6876c543b5ff91`

Implementation commit：`9a175ff22e3177511e5b7749b7bf0c79eb47db98`

Review result：`STATUS: CLEAR`，复核线 `019eb51c-61fe-7fc3-8973-b22a4ce58911`。

## 1. Scope

R4-A6 只实现 Knowledge Base / Settings 两个页面的首批前端纯 selector 分域和页面最小消费。

本轮接受范围：

- `pageSelectors.ts` 新增 `KnowledgeBasePageReadModel` / `SettingsPageReadModel` 和 split-input selector。
- `KnowledgeBaseView.tsx` 使用 Knowledge selector 输出承接页头、首屏统计和 Obsidian-compatible boundary。
- `SettingsView.tsx` 使用 Settings selector 输出承接页头、常规统计、内部边界摘要和页面合同数量。
- `r4-page-selectors.test.ts` 覆盖 Knowledge / Settings selector source boundary、只读边界、候选和正式记忆分离、设置页不读凭据且不触发执行。

本轮不接受范围：

- 不接受为 R4 完成。
- 不接受为页面真实数据来源迁移。
- 不接受为 `query_workbench_page_read_model` 被页面真实消费。
- 不接受为 `WorkbenchSnapshot` 废弃。
- 不接受为 UI 重做、布局重做或视觉验收。
- 不接受为真实 Tauri / 截图验收完成。
- 不接受为 R3 Level B、真实执行、多 agent 并行真实执行解锁。

## 2. Changed Files

- `prototypes/productized-desktop-shell/src/lib/pageSelectors.ts`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/SettingsView.tsx`
- `prototypes/productized-desktop-shell/tests/r4-page-selectors.test.ts`
- `tasks/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1.md`
- `evidence/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1.md`
- `handoffs/2026-06-11-root-treatment-r4-a6-knowledge-settings-selector-domain-and-page-consumption-v1-result.md`

## 3. Implementation Notes

`pageSelectors.ts` 新增：

- `deriveKnowledgeBasePageReadModel`
- `deriveKnowledgeBasePageReadModelFromParts`
- `deriveSettingsPageReadModel`
- `deriveSettingsPageReadModelFromParts`

新增 selector 均使用既有 `selectorSourceBoundary()`，保持：

- `workbench_snapshot_active=true`
- `page_ui_migrated=false`
- `tauri_command_consumed=false`
- `writes_stores=false`

Knowledge selector 保留边界：

- `knowledge_hit_and_candidate_are_not_formal_memory`
- formal memory link 和 candidate link 分离计数。
- Obsidian-compatible 仍是占位和边界说明，不声明原生同步或 vault 扫描完成。

Settings selector 保留边界：

- `credential_display_allowed=false`
- `execution_from_settings_allowed=false`
- `settings_page_must_not_read_credentials_or_trigger_execution`
- 开发者内容仍在 Settings 内部边界摘要，不扩成普通页面首屏 raw materials。

## 4. Verification

已运行并通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - `offline interaction tests passed: 14`
  - `r4 page selectors test passed`
- `npm run build`
  - 通过，保留既有 Vite chunk size warning。
- `node scripts/harness/workbench-shape-gate.js --mode check`
  - `Status: pass`
  - `Errors: 0`
  - `Warnings: 1`
  - 继承警告：`tauri_command_total_increased 97/96`
- `git diff --check`

一次 shape gate 曾在 `prototypes/productized-desktop-shell` 子目录以错误相对路径执行，返回 `MODULE_NOT_FOUND`；随后已在 `product-line` 根目录按正确路径重跑并通过。

未运行 Rust：

- 本切片未改 Rust、Tauri command、sidecar、DB migration 或 workflow state schema。

## 5. Boundary Confirmation

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / 截图工具。
- 新增 Tauri command、sidecar、DB migration、store kind 或 workflow state 顶层结构。
- 改 `App.tsx` 数据加载路径。
- 把 `query_workbench_page_read_model` 接成页面真实数据源。
- 解冻 Stage L / Stage K / backlog 功能。

## 6. Review Request

复核线请只读审查：

- Knowledge / Settings 页面是否真的消费 selector 输出。
- selector 是否保持前端纯函数和 source boundary。
- 是否避免新增 Tauri command、sidecar、DB migration、真实执行路径。
- 是否避免视觉 / CSS / 布局变更。
- 是否避免把 knowledge hit / candidate 说成正式记忆。
- 是否避免把 Settings 开发者边界摘要扩成普通用户首屏 raw materials。
