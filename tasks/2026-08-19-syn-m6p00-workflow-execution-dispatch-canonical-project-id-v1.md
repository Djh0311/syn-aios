# Grok 窄包：M6P00 workflow 执行与 dispatch 正式入口 canonical ProjectId

执行记录：本包优先派给 Grok；其超出轮次且未形成写入后，由 Codex 按用户 2026-08-19 的“Grok 优先、Codex 保底”口径接管。任一时刻只有一个源码写者。

本包只补当前 M6P00 已审计出的执行链正式入口。项目路径只作为 M1 exact alias；正式入口必须先解析 canonical `ProjectId`，再将该值沿既有调用链传给业务 helper。不得改变现有执行合同、安全闸或派发状态机。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs`

不要修改其他源码。不要读取或触碰受保护的 6 个未跟踪 `m6_*.rs`、`.bak` 与 `gen/schemas/linux-schema.json`；不要暂存、提交、reset、stash、clean 或全 crate 格式化。

## 入口与接线要求

1. 复用 `commands.rs` 中现有 `resolve_m1_canonical_project_id`。以下正式 Tauri command 必须在任何业务写、backup、authorization audit、dispatch reserve、consult/spawn 之前解析 `request.project_root` 的 canonical ID；M1 未安装、alias unknown/malformed 时零写 fail-closed：
   - `bind_workflow_node_codex_session`
   - `execute_workflow_node_dispatch`
   - `record_workflow_dispatch_director_review`
   - `prepare_offline_role_dispatch`
   - `record_offline_role_result_handoff`
   - `record_offline_director_review`
2. 将 canonical ID 显式传入对应 `_for_index_at` / `_at` / context helper，不允许 helper 重新调用 `project_id(project_root)`。
3. 新增或更新的 `workflow_node_session_bindings`、`WorkflowNodeDispatchContext`、`AutoDispatchGuardInput`、dispatch/review 记录中的 `project_id` 必须使用 canonical 值。
4. 读取或更新既有 workflow、binding、dispatch、review 前，凡目标记录已有 `project_id`，必须与 canonical 精确相等；path-derived/foreign owner 在任何 mutation 前拒绝。不得用仅 root/workflow_id 相等代替 owner 校验。
5. 只测试 helper 如必须保留 path-derived fixture，须标注 legacy fixture、保留理由与失效条件，并保证不进入 production command span。

## 绝对不改

- ExecutionGrant、WorkerReport、receipt、audit、quarantine 的合同语义；
- `m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类；
- path-lock、Station 3b/4、authorization completeness、resume runner、超时/重试、状态转换或任何真实执行解封；
- command 注册集合、前端、GUI、provider、账号、网络业务写。

## 直接反例

测试名统一以 `m6p00_workflow_execution_` 开头，至少覆盖：

- canonical 刻意不同于 `project_id(root)` 时，binding、dispatch context、authorization input 与 review/offline records 都持久化 canonical；
- canonical/path-derived/foreign 同键记录并存时只读取/更新 canonical；
- path-derived 或 foreign owner 的 binding/workflow/dispatch/review 在业务写前拒绝，状态文件与 sidecar/authorization store 精确零写；
- M1 port unavailable、alias unknown/malformed 的正式 command resolver 零写；
- 生产 command 到 helper 的 span 不含本包替代的 `project_id(&request.project_root)` 或等价 fallback。

测试必须离线、临时目录运行，不启动真实 Codex/provider，不使用真实项目或账号。

## 验证

从 `prototypes/productized-desktop-shell/src-tauri` 运行：

```bash
CARGO_TARGET_DIR=/tmp/syn-m6p00-workflow-execution-target cargo test --lib --offline m6p00_workflow_execution_ -- --test-threads=1
CARGO_TARGET_DIR=/tmp/syn-m6p00-workflow-execution-target cargo test --lib --offline workflow_node_dispatch_ -- --test-threads=1
CARGO_TARGET_DIR=/tmp/syn-m6p00-workflow-execution-target cargo test --lib --offline offline_role_ -- --test-threads=1
```

仓库根运行：

```bash
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/commands.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_run_dispatch_entrypoints.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_execution_entrypoints.rs
```

逐条报告真实退出码、测试计数、精确改动文件与仍保留的 legacy fixture；在所有验证结束前不要宣称完成。
