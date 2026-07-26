# 任务包：L3 Syn 原生知识工作区开发 v2

- 日期：2026-07-23
- 状态：**原 N0-N6 合同；N0-N5 离线实现与验证已收口。07-25 UI 路线已修订，本包不自动授权 N2R**
- 负责人：现有 Codex 开发线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 决策：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- 计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 取代：`tasks/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-development-package-v1.md`

## 0. Kickoff

- 任务：完成 N0-N6，让 Syn 原生知识工作区在未安装 Obsidian时也能真实日用。
- 负责人：现有 `gpt-5.6-terra / ultra` 开发线；允许它在文件写面互斥且证据清楚时拆并行开发对话。
- 交付物：原生 vault/索引、编辑工作区、图谱、JSON Canvas、附件/恢复、AI/MCP 只读接线、离线 evidence、真实 App evidence、CURRENT/AUTHORITY 回写。
- 验收标准：计划 N6 十二项真实 App 验收完成，离线验证通过，历史 shape/warnings 单列，staged 为空。

## 1. 对齐块

```yaml
authority_chain:
  - AGENTS.md
  - CURRENT.md
  - decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md
  - docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md
  - tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md
plan_anchor: docs/plans/2026-07-14-post-m5-stage-plan-v2.md#1-轨道表-L3-知识库
existing_before_new:
  - 复用固定 Syn vault、Markdown 浏览/编辑、wikilink、冲突拒绝和 Batch 2 audit
  - 复用 KnowledgeBaseView、knowledgeVault.ts、knowledgeBase.ts 与 PendingAction
  - 复用 @xyflow/react，不新增第二图或 Canvas 框架
  - 复用 capability_registry、可信 conversation binding 与 tools/list/tools/call 双闸
  - 保留 v1 typed Obsidian bridge 为可选兼容层
capabilities_touched:
  - validated vault relative paths and rebuildable indexes
  - native markdown workspace, backlinks, search and layout
  - knowledge graph and JSON Canvas
  - bounded attachments, refresh, conflict and recovery
  - supervisor knowledge_search/read/open/cite
forbidden_alternatives:
  - Obsidian 真嵌入、伴随窗口、Accessibility 窗口贴合或 Electron 迁移
  - Obsidian 商标、受限品牌资产、插件 API 或私有代码复制；核心桌面 UI 高保真另按 N2R 新授权执行
  - 第二知识真相源或不可重建索引
  - 任意 filesystem/shell/CLI/eval/CDP
  - 放宽主管 sandbox、binding、allowlist 或新增知识写工具
```

## 2. N0 必做：先收口 v1 WIP

1. 记录 v1 已修改文件和测试结果。
2. 保留 `obsidian_integration.rs`、`obsidianIntegration.ts` 中可选 open/read/search bridge；删除或隔离未完成 companion 入口时只改本线新 WIP。
3. `KnowledgeBaseView` 的主层级改为 Syn 原生工作区；Obsidian 状态只可作为收起的兼容入口。
4. 保留 `knowledge_capabilities.rs` 和 registry/binding 的只读实现。
5. 明确未执行真实 Obsidian 安装、CLI 注册、辅助功能授权和其他 vault 访问。

完成 N0 后再扩文件写面。

## 3. 初始写入白名单

以下是 N0-N2 与 N6 的初始上限；N3-N5 的新增模块在各阶段 kickoff 前由执行线列出精确文件，指导线可在不改变决策的情况下补白名单。

### 后端

- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_index.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/obsidian_integration.rs`（v1 新 WIP，兼容层收口）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/capability_registry.rs`（已有未跟踪实现，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/knowledge_capabilities.rs`（v1 新 WIP）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs`（已有未跟踪实现，merge-only）
- 同目录新增的精确 knowledge index 测试文件。

### 前端与测试

- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/knowledge/`（可新增原生知识工作区组件）
- `prototypes/productized-desktop-shell/src/lib/knowledgeVault.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
- `prototypes/productized-desktop-shell/src/lib/obsidianIntegration.ts`（兼容层收口）
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`（只把旧“未执行 Obsidian 原生同步”边界改为 v2 中立知识候选边界）
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/knowledge-vault-notes.test.tsx`
- `prototypes/productized-desktop-shell/tests/obsidian-integration.test.tsx`（兼容层回归）
- `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx`（可新增）
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`（只改上述 v2 边界断言）
- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryKnowledgeTextFixtures.ts`（只改知识库相关 fixture）
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`

### 文档

- 本 decision/plan/task；
- `evidence/2026-07-23-l3-syn-native-knowledge-workspace-offline-verification-v2.md`（新增）
- `evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v2.md`（新增）
- `CURRENT.md`、`AUTHORITY.md`
- `docs/plans/2026-07-14-post-m5-stage-plan-v2.md`
- `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
- `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`
- `docs/harness-catch-log.md`（只有真实 catch 时追加）

禁止修改 workflow DB schema/bridge/CAS、正式记忆模型、旧 resident/private-home 路线、真实业务项目和 `.codex` 凭据。

## 4. 阶段执行方式

- 严格按 N0→N1→N2；N3/N4 可在路径和写入合同冻结后并行；N5 合并附件/恢复；N6 总验收。
- 每阶段先写可失败场景和边界测试，再写最小实现。
- 不为“以后可能兼容插件”新增抽象层。
- 不把 UI 组件状态当文件真相；索引、反链图和工作区状态均可重建或安全丢弃。
- 新依赖必须能明确替代大量自制基础设施；`@xyflow/react` 已存在，图谱与 Canvas 先复用。

## 5. 当前必须保住的安全回归

- 固定 vault 根和路径穿越/符号链接拒绝；
- stale revision/mtime/hash 冲突零覆盖；
- AI 拒绝零写、允许单写并产生 `knowledge_vault_audit`；
- `knowledge_search/read/open/cite` 对缺 binding、错项目、未知工具、大小写变体和额外字段 fail closed；
- 工具失败不吞自然回复；
- 不安装 Obsidian时原生知识工作区完整可用；
- 可选兼容桥仍不得接受任意 binary、vault、argv、command 或 shell。

## 6. 验证与回交

每阶段回交一次简短证据；最终回交必须包括：

1. 实际改动文件与白名单核对；
2. 原生文件真相、可重建索引和路径闭锁证据；
3. 编辑、反链、搜索、图谱、Canvas、附件、工作区恢复证据；
4. AI/MCP 权限与零越权写证据；
5. Rust 定向、`cargo check --lib`、typecheck、离线 runner、目标 fmt、shape、`git diff --check`；
6. N6 十二项真实 App 结果与截图/日志路径；
7. 历史 warnings/shape、未完成项和实际 catch 单列；
8. staged 为空，未 commit/push，未访问其他 vault/真实项目。

开发线自报完成后，指导线仍需核 diff、关键测试和真实 App 实物；收到回传不等于验收通过。
