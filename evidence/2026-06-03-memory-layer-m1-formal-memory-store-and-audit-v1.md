# memory-layer M1 formal memory store and audit evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮只完成 M1：正式记忆受控存储和审计骨架。
- 这不是候选采纳流程，不是任务包召回，不是任务包注入，也不是中间版本记忆层完成。
- 正式记忆第一版仍用 JSON sidecar，不是 SQLite 事务库。
- UI 只做只读摘要；没有完整记忆管理页面。
- 没有真实浏览器 / Tauri 窗口截图证据。原因：当前可用工具没有 in-app browser 控制工具，项目也没有 Playwright 依赖；本轮未新增浏览器依赖。

## 已实现

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`。
- 新增 sidecar：`<workflow_state_dir>/formal-memories.v1.json`。
- 新增 store 类型：`FormalMemoryStoreV1`。
- 新增正式记忆版本类型：`MemoryVersion`。
- 新增正式记忆审计事件类型：`MemoryAuditEvent`。
- 新增命令：
  - `load_formal_memory_store`
  - `create_formal_memory_record`
- `create_formal_memory_record` 显式创建正式记忆时同步写：
  - `MemoryRecord`
  - `MemoryVersion` 第一版
  - `MemoryAuditEvent(memory_record_created)`

控制核心：

- 新增 `validate_formal_memory_create(...)`。
- 新增 `validate_formal_memory_status(...)`。
- 校验 claim、body、source_refs、scope、model_export_policy、memory_type、actor_role。
- 正式记忆初始状态只能是 `memory_active`。
- `secret` 来源或简单标记的敏感内容必须 `model_export_policy = blocked`。
- `project_director` 只允许创建本项目 / workflow / session 作用域的 `project_memory`、`workflow_summary`、`session_summary`。
- `global_director` 在 M1 保守拒绝创建正式全局记忆，避免提前打开全局写入边界。

前端：

- 新增 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`、`FormalMemoryStoreV1`、创建输入输出类型。
- 新增 Tauri 包装：`loadFormalMemoryStore`、`createFormalMemoryRecord`。
- 新增只读摘要 `summarizeFormalMemoryStore(...)`。
- 项目工作流侧栏候选治理卡展示：
  - `formal-memories.v1.json`
  - revision
  - record / active / version / audit 数量
  - 最近一条 audit event
  - “创建时写入 version 和 audit”
  - “M1 不包含候选采纳和任务包注入”
- 记忆入口占位也显示正式记忆 sidecar、数量和 revision。

## 数据完整性

- 原子写入：tmp 文件写入、sync、rename。
- 写入前备份旧 `formal-memories.v1.json` 到 `backups/`。
- lock 文件防并发覆盖：`.formal-memories.v1.lock`。
- revision 冲突拒绝写入：`formal_memory_store_conflict`。
- 损坏 JSON 读取时拒绝覆盖。
- 不写入 `workflow-state.v0.json`。

## 测试证据

红灯阶段：

- `cargo test --lib formal_memory_store -- --nocapture` 先因缺 `CreateFormalMemoryRecordInput`、`formal_memory_store`、`validate_formal_memory_status` 失败。
- `npm run test:offline-interaction` 先因缺 `summarizeFormalMemoryStore` export 失败。

通过的 Rust 测试：

- `formal_memory_store_creates_record_version_and_audit`
- `formal_memory_store_rejects_missing_source_refs`
- `formal_memory_store_rejects_candidate_status`
- `formal_memory_store_keeps_candidate_store_separate`
- `formal_memory_store_damaged_json_is_not_overwritten`
- `formal_memory_store_revision_conflict_is_rejected`

最新验证命令：

```text
npm run typecheck
通过：tsc --noEmit

npm run test:offline-interaction
通过：offline interaction tests passed: 9

npm run build
通过：tsc --noEmit && vite build
备注：仍有 Vite chunk > 500 kB warning，属于既有构建体积提醒。

cargo test --lib
通过：122 passed，1 ignored
备注：仍有既有 warning：src/mcp/protocol.rs JsonRpcError::invalid_params unused。

rustfmt --check src/formal_memory_store.rs
通过

curl -sS -I http://127.0.0.1:5174/
通过：HTTP/1.1 200 OK
```

## 边界自检

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未改 `workflow-state.v0.json` 结构。
- 未迁移数据库。
- 未接 Obsidian / 知识库。
- 未接向量库 / 图数据库。
- 未新增候选采纳命令。
- 未把 `candidate_confirmed` 自动创建为正式记忆。
- 未把正式记忆注入 worker 任务包。

## 未做范围

- M2：候选到正式记忆的受控采纳。
- M4：任务记忆包生成器和预览。
- M6：工作流任务包注入和端到端闭环。
- M9：正式记忆编辑、废弃、冻结、合并、拆分等生命周期操作。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`

## 结论

接受为：正式记忆受控 store M1 完成，显式正式记忆创建能同步生成 record、version、audit；无来源创建、候选状态、损坏 JSON 和 revision 冲突会被拒绝；候选 store 和正式记忆 store 分离；读模型能显示正式记忆骨架状态。

不接受为：中间版本记忆层完成、候选采纳完成、任务包召回完成、任务包注入完成、正式记忆生命周期完成、Obsidian / 知识库集成完成、向量库 / 图数据库完成。
