# Handoff：Memory Layer M6 Workflow Task Package Injection And End To End Loop v1 Result

日期：2026-06-04

## 回收结论

接受为 M6 工作流任务包注入和第一条端到端记忆闭环完成。

不接受为中间版本记忆层完成；不接受为完整正式记忆生命周期完成；不接受为完整维护任务系统完成；不接受为真实 worker 已执行；不接受为自动化工作流产品化闭环完成。

## 当前可操作状态

- 任务包生成会保存冻结的 `TaskPackageMemoryPacketSnapshot` 到现有 `task_package` artifact。
- 任务包 Markdown 和 prepared dispatch prompt 使用同一份 snapshot，不重新构建记忆包。
- readiness 会检查 memory snapshot 缺失 / stale；formal、candidate、observation、lint store revision 变化会被识别。
- 如果 task package 声明需要记忆，但 snapshot 缺失或 included 正式记忆为空，会阻断派发准备。
- 项目工作流侧栏可只读显示“任务包记忆注入摘要”，包含 included / excluded / review materials / stale / warnings。

## 重要非目标

- M6 不执行 worker。
- M6 不执行真实 Codex 或 `codex exec resume`。
- M6 不读写 `/Users/yoyi/.codex`。
- M6 不自动把 worker 汇报写成正式记忆。
- M6 不提供完整记忆中心、生命周期 UI、维护任务系统或 finding 生命周期 UI。
- M6 不把 candidate / observation / knowledge hit / LLM summary 放入 included list。

## 验证记录

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
- `rustfmt --check ...`：通过。

Rust 测试仍有既有 warning：`JsonRpcError::invalid_params` unused。

续验记录（2026-06-04）：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，9 scenarios。
- `npm run build`：通过；保留既有 Vite chunk size warning。
- `cargo test --lib task_memory_injection`：通过，5 passed。
- `cargo test --lib`：通过，169 passed，1 ignored；保留既有 `JsonRpcError::invalid_params` unused warning。
- `rustfmt --check src/task_memory_injection.rs src/task_memory_packet_builder.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`：通过。
- 已同步 `AUTHORITY.md`、`STAGE_PLAN.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/plans/memory-layer-implementation-slice-v1.md`，把 M6 状态从残留“待执行”改为已完成，并指向 M7-M13 后续范围。

## UI 验收缺口

已做普通 Vite smoke：非沙箱启动 `npm run dev -- --host 127.0.0.1`，`curl -I http://127.0.0.1:5173/` 返回 `HTTP/1.1 200 OK`，HTML 入口可读取。

真实窗口 / 截图验收未完成。本轮没有可用 in-app Browser 控制工具，项目未安装 Playwright，未下载依赖。

## 下一步建议

下一步不要把 M6 解释成完整记忆系统。继续按 `docs/plans/memory-layer-implementation-slice-v1.md` 拆 M7-M13：正式记忆管理 UI、知识库边界、正式记忆生命周期、实体关系治理、维护任务和最终验收。
