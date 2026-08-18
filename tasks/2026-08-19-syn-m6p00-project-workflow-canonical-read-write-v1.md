# Grok 窄包：M6P00 项目工作流 canonical ProjectId 正式读写入口

执行记录：本包由 Grok 完成首轮实现，Codex 按用户 2026-08-19 的“Grok 优先、Codex 保底”口径独立复核并在同一写域内返修；任一时刻只有一个源码写者。

本包只把 M1 exact-alias 解析出的 canonical `ProjectId` 接入既有项目工作流正式入口；不改执行合同语义，不扩到 M6 域层。工作副本可能已有本包未提交增量，先审现状再做最小修复，不丢弃已有改动。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`

不要修改其他源码，不要读或触碰受保护的 6 个未跟踪 `m6_*.rs` 与 `gen/schemas/linux-schema.json`，不要格式化整个 crate，不要暂存或提交。

## 正式入口

下列入口必须先从 `AppState` 的 M1 read port 对 `project_root` 做 exact-alias 解析，再进行任何业务读写；不得接受调用方 `project_id` 作为真源，也不得回退到 `project_id(project_root)`：

1. `resolve_supervisor_conversation_context`
2. `record_plan_authorization_user_confirmation`
3. `run_project_workflow_automation_phase_a`
4. `list_project_workflows`
5. `submit_project_workflow_draft`
6. `get_project_workflow_nodes`

允许把既有 task-package 专用 M1 resolver 提升为同文件通用 helper，但不得改变 task-package 行为。M1 authority 未安装、alias malformed/unknown 时必须在业务写前 fail-closed。

## 行为要求

- 列表与节点读取只返回 `workflow.project_id == canonical ProjectId` 的记录；同 root 的 path-derived owner 与 foreign owner 都不得被选中。
- 新建 workflow 必须持久化 canonical ProjectId；更新现有 workflow 时先核验 owner，path-derived/foreign owner 在任何 nodes、edges、binding、audit 或备份写入前拒绝。
- Phase-A core 接受显式 canonical ProjectId。若输入带 `project_id` claim，只有与 canonical 完全相等才允许继续；不带 claim 也不得自行派生。plan、work item、observation、memory capture 等本次路径上的项目字段统一使用 canonical 值。
- plan authorization 的 memory capture 使用 canonical ProjectId；M1 解析失败必须发生在 authorization sidecar 写前。
- 不改 ExecutionGrant、WorkerReport、receipt、audit、quarantine 或 runner `new-grant / guarded-legacy / blocked` 分类。
- 仅测试 helper 可保留 path-derived fixture，必须带 legacy 原因与明确失效条件，不得进入生产调用图。

## 直接测试

统一前缀 `m6p00_project_workflow_`，至少覆盖：

- canonical、path-derived、foreign 三种 owner 同时存在时，list/get 只选 canonical；
- 新建写 canonical，更新 foreign/path-derived owner 零部分写；
- supervisor context 使用真实 ordinary M1 exact alias，并拒绝调用方 path-derived claim 与 path-derived workflow owner；
- M1 未安装、alias unknown/malformed 在业务写前拒绝；
- Phase-A claim mismatch 零写，且成功用例的 canonical id 必须刻意不同于 `project_id(root)`，证明 plan/capture 持久化的不是路径派生值；
- 生产函数 span 不含本次被替代的 path-derived fallback。

## 交付验证

- `CARGO_TARGET_DIR=/tmp/syn-m6p00-project-workflow-target cargo test --lib --offline m6p00_project_workflow_ -- --test-threads=1`
- `CARGO_TARGET_DIR=/tmp/syn-m6p00-project-workflow-target cargo test --lib --offline project_workflow_ -- --test-threads=1`
- `CARGO_TARGET_DIR=/tmp/syn-m6p00-project-workflow-target cargo test --lib --offline conversation_transport_ -- --test-threads=1`
- 仓库根：`git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/commands.rs prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`

若第三条命中与本包无关的既有 stop/recovery 失败，必须在同一 `HEAD` 的干净 disposable checkout 运行相同唯一测试作对照；不得为通过本包擅自修 execution/stop 语义。逐条报告实际退出码、测试计数和对照结果，不得把基线失败说成通过。
