# Evidence：Memory Layer M6 Workflow Task Package Injection And End To End Loop v1

日期：2026-06-04

## 结论

M6 工作流任务包注入和第一条端到端记忆闭环已完成。

接受为：

- `generate_task_package_file_at` 会从 `TaskMemoryPacketBuilder` 构造冻结 `TaskPackageMemoryPacketSnapshot`。
- task package artifact 保存 `memory_packet_snapshot`、`memory_packet_fingerprint`、`memory_packet_generated_at`、`memory_packet_store_revisions`、`memory_packet_stale`、`memory_packet_warnings`。
- 生成的任务包 Markdown 包含“正式记忆上下文”，展示 included 正式记忆、来源摘要、入选理由、排除摘要、待审查材料摘要和边界说明。
- `prepare_workflow_node_dispatch_at` / `prepare_offline_role_dispatch_at` 使用 artifact 中同一份 frozen snapshot 渲染 prompt，不重新召回记忆。
- `inspect_task_package_dispatch_readiness_at` 能识别缺失 / stale 记忆快照，formal / candidate / observation / lint revision 变化会使快照 stale。
- M5 open blocking lint finding 命中的正式记忆不会进入任务包 snapshot included list。
- 项目工作流侧栏只读显示“任务包记忆注入摘要”，不新增一级入口、右侧入口或项目 tab。

不接受为：

- 中间版本记忆层完成。
- 完整正式记忆生命周期完成。
- 完整维护任务系统完成。
- 真实 worker 已执行。
- 自动化工作流产品化闭环完成。

## 关键实现

- 新增 `src-tauri/src/task_memory_injection.rs`：snapshot 构造、fingerprint、artifact 写入、stale revision 比对、markdown/prompt 记忆块渲染。
- 扩展 `src-tauri/src/types.rs`：`TaskPackageMemoryPacketSnapshot`、store revisions、injection summary、dispatch snapshot 引用。
- 扩展 `src-tauri/src/lib.rs`：任务包生成、readiness、workflow dispatch prepare、offline dispatch prepare、读模型派生和 Rust 测试。
- 扩展 `src-tauri/src/task_memory_packet_builder.rs`：输出 lint store revision；补中文连续文本 4 字窗口相关性，避免中文任务目标无法命中正式记忆。
- 扩展前端类型和只读摘要：`src/lib/types.ts`、`src/lib/candidateGovernance.ts`、`src/views/ProjectsView.tsx`、`src/App.tsx`。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，9 scenarios。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- `cargo test --lib task_memory_injection`：通过，5 passed。
- `cargo test --lib task_memory_packet`：通过，10 passed。
- `cargo test --lib memory_lint`：通过，9 passed。
- `cargo test --lib memory_candidate_adoption`：通过，7 passed。
- `cargo test --lib observation`：通过，10 passed。
- `cargo test --lib formal_memory`：通过，19 passed。
- `cargo test --lib`：通过，169 passed，1 ignored。
- `rustfmt --check src/task_memory_injection.rs src/task_memory_packet_builder.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs`：通过。

## 续验记录

2026-06-04 续跑完成：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，9 scenarios。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- `cargo test --lib task_memory_injection`：通过，5 passed。
- `cargo test --lib`：通过，169 passed，1 ignored；保留既有 `JsonRpcError::invalid_params` unused warning。
- `rustfmt --check src/task_memory_injection.rs src/task_memory_packet_builder.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`：通过。
- 沙箱内启动 Vite 绑定 `127.0.0.1:5173` 仍被 EPERM 拒绝；经授权非沙箱启动后，`curl -I http://127.0.0.1:5173/` 返回 `HTTP/1.1 200 OK`，HTML 入口可读取；结束前已关闭 5173 端口进程。
- 本轮工具发现未暴露 in-app Browser / 截图控制工具；未做真实浏览器截图或交互验收。
- 已同步 `AUTHORITY.md`、`STAGE_PLAN.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/plans/memory-layer-implementation-slice-v1.md` 中 M6 状态，消除旧的未完成状态残留。

## UI Smoke

- 沙箱内启动 Vite 绑定 `127.0.0.1:5173` 被 EPERM 拒绝；按权限规则以非沙箱启动 `npm run dev -- --host 127.0.0.1`。
- `curl -I http://127.0.0.1:5173/` 返回 `HTTP/1.1 200 OK`。
- `curl http://127.0.0.1:5173/` 返回 Vite HTML 入口。
- 本轮没有可用 in-app Browser 控制工具，项目也未安装 Playwright；未下载依赖。
- 真实窗口 / 截图验收未完成。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未新增 workflow-state 顶层数组。
- 未修改 workflow / work item / node 状态枚举。
- 未把 candidate、observation、knowledge hit 或 LLM summary 放进 included list。
- 未把任务包内容回灌为正式记忆。
