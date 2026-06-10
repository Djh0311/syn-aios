# memory-layer M1.1 formal memory context binding guard evidence v1

日期：2026-06-03

## 先说薄弱点

- 本轮只补 `create_formal_memory_record` 的上下文绑定 guard，不做 M2 候选采纳。
- 正式记忆仍写入 M1 的 JSON sidecar，不迁移数据库。
- workflow state 项目存在性采用保守拒绝：state 不存在、`projects[]` 缺失、为空，或不含 `project_root`，都会拒绝创建正式记忆。
- 没有改 UI；本轮没有新增真实浏览器 / Tauri 窗口截图。

## 已实现

后端：

- `create_formal_memory_record` 命令入口改为先走 `create_formal_memory_record_at(...)`。
- 新增 `validate_formal_memory_context_binding(...)`。
- 后端由 `project_root` 推导：
  - `expected_project_id = project_id(project_root)`
  - `expected_workflow_id = default_workflow_id(project_root)`
- 创建正式记忆前校验：
  - `request.project_id`
  - `request.workflow_id`
  - `scope.project_id`
  - `scope.workflow_id`
- `project` / `workflow` / `session` scope 必须带 `scope.project_id`。
- `workflow` / `session` scope 必须带 `scope.workflow_id`。
- `project_director` 仍只能写 `project` / `workflow` / `session` scope。
- 只读检查 `workflow-state.v0.json`：`projects[]` 必须包含当前 `project_root`。兼容现有 workflow state 的 `root_path` 字段，也兼容测试/旧数据里的 `project_root` 字段。
- guard 通过后才调用 M1 的 `formal_memory_store::create_record(...)`，原有 record / version / audit 写入逻辑不变。

测试：

- 新增命令入口级测试：
  - `formal_memory_context_accepts_matching_project_and_workflow`
  - `formal_memory_context_rejects_mismatched_project_id`
  - `formal_memory_context_rejects_mismatched_workflow_id`
  - `formal_memory_context_rejects_project_director_cross_project`
  - `formal_memory_context_rejects_missing_project_in_workflow_state`
  - `formal_memory_context_keeps_existing_m1_guards`

## 红灯记录

先加测试后运行：

```text
cargo test --lib formal_memory_context -- --nocapture
失败：E0425 cannot find function `create_formal_memory_record_at`
```

红灯说明：当时正式记忆创建还没有命令入口级上下文绑定 helper，新增测试无法编译。

## 验证

已通过：

```text
cargo test --lib formal_memory_context -- --nocapture
通过：6 passed

cargo test --lib formal_memory -- --nocapture
通过：13 passed

cargo test --lib formal_memory_store -- --nocapture
通过：6 passed

rustfmt --check src/formal_memory_store.rs src/control_core.rs src/lib.rs src/commands.rs
通过

npm run typecheck
通过

npm run test:offline-interaction
通过：offline interaction tests passed: 9

npm run build
通过；仍有既有 Vite chunk > 500 kB warning

cargo test --lib
通过：128 passed，1 ignored
备注：仍有既有 warning：src/mcp/protocol.rs JsonRpcError::invalid_params unused
```

## 边界自检

- 未实现 M2 候选采纳。
- 未新增“采纳候选为正式记忆”命令。
- 未改 `workflow-state.v0.json` 结构。
- 未迁移数据库。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未接 Obsidian / 知识库。
- 未接向量库 / 图数据库。
- 未把秘书、worker 或黑板候选变成正式记忆写入者。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`

## 结论

接受为：M1.1 正式记忆上下文绑定 guard 完成。正式记忆创建不再只靠请求字段内部一致性判断项目归属，必须与 `project_root` 的后端推导项目 / workflow 一致，并且 workflow state 当前项目列表必须包含该 `project_root`。

不接受为：M2 候选采纳完成、正式记忆生命周期完成、任务包召回或注入完成、中间版本记忆层完成、Obsidian / 知识库 / 向量库 / 图数据库集成完成。
