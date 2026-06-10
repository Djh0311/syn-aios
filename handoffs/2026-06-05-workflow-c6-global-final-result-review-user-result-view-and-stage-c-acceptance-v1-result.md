# Handoff: Workflow C6 Global Final Result Review, User Result View And Stage C Acceptance v1

日期：2026-06-05

## 当前状态

C6 已完成并通过验收。阶段 C 的 C1-C6 受控闭环可接受为阶段完成；中间版本整体和完整记忆系统仍未完成。

## 已做内容

- 新增全局主管最终结果复核命令：`record_global_final_result_review`。
- 新增用户结果决定命令：`record_user_result_decision`。
- 新增阶段 C 验收摘要命令：`generate_stage_c_acceptance_summary`。
- 全局最终复核写入既有 `reviews[]` / `audit_events[]`。
- 用户结果决定写入既有 `reviews[]` / `audit_events[]`。
- 阶段 C 验收摘要写入既有 `artifacts[]` / `audit_events[]`。
- 新增 `WorkflowResultSummaryReadModel`，项目工作流读模型可显示 final review、user decision、stage gate、open blockers、deferred items。
- 项目工作流侧栏新增“C6 结果 / 阶段验收”摘要区和三类确认动作。
- 确认弹层新增全局最终复核、用户结果决定和阶段 C 验收摘要边界说明。
- 离线测试覆盖 C6 面板显示、action payload、确认弹层和禁止文案边界。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib global_final_result_review`
- `cargo test --lib user_result_decision`
- `cargo test --lib stage_c_acceptance`
- `cargo test --lib process_fact`
- `cargo test --lib dispatch_readback_stats`
- `cargo test --lib workflow_authorization`
- `cargo test --lib plan_authorization`
- `cargo test --lib`
- `rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs src/lib.rs`

Smoke：

- Vite dev server 已启动在 `http://127.0.0.1:5176/`。
- `curl -sS -I http://127.0.0.1:5176/` 返回 `HTTP/1.1 200 OK`。
- 真实窗口 / 截图验收未完成：当前没有暴露 browser screenshot 工具，Node 环境没有 Playwright。

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未新增 `workflow-state.v0.json` 顶层结构。
- 未改 workflow / work item / node / dispatch 状态枚举。
- 未把 worker report 写成正式事实。
- 未把 `process_fact` observation 写成正式记忆。
- 未自动生成 MemoryCandidate。
- 未写正式记忆。
- 未完成中间版本整体。
- 未完成 M7-M13 完整记忆系统。
- 未完成完整自动重试系统、运行日志体系或运维诊断。
- 未完成真实 Tauri 全面截图验收。

## 接手建议

下一步进入阶段 D / M7-M13，先拆正式记忆生命周期相关任务包。阶段 C 后置项仍需另拆：

- 真实 worker / Codex 执行授权任务包。
- 真实 Tauri 全面截图验收。
- 完整自动重试、运行日志和运维诊断。

继续保持边界：

- `process_fact` observation 只能作为过程事实证据，不是正式记忆。
- 用户结果决定只适用于本次结果，不代表未来任务默认接受。
- 阶段 C 完成不等于中间版本整体完成。
