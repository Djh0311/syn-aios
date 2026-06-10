# Handoff: Workflow C5 Worker Structured Report, Process Fact Confirmation And Failure Visibility v1

日期：2026-06-04

## 当前状态

C5 已完成并通过验收。当前入口应更新为“C5 已完成，下一步拆 C6”。

## 已做内容

- 新增 worker 结构化汇报记录命令：`record_worker_structured_report`。
- 新增项目主管过程事实决定命令：`record_project_director_process_fact_decision`。
- worker report 写入现有 `audit_events[]`，事件类型为 `worker_structured_report_recorded`。
- 项目主管确认低风险本项目 process fact 后写 `observations.v1.json`，`observation_type = "process_fact"`。
- `request_rework` / `block_and_escalate` 只写 review / audit，不写 observation。
- `SubagentReport` 和 `ReviewResult` 读模型补齐 C5 字段。
- 项目工作流侧栏新增 C5 摘要区，展示 report / pending / confirmed、readback、permission、failure 和 issue / risk。
- 确认弹层新增 worker report 和 process fact decision 摘要，并明确不写正式记忆、不完成最终验收。
- 离线测试覆盖 C5 面板显示、action payload 和禁止文案边界。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib worker_structured_report`
- `cargo test --lib process_fact`
- `cargo test --lib dispatch_readback_stats`
- `cargo test --lib workflow_exception`
- `cargo test --lib observation`
- `cargo test --lib workflow_authorization`
- `cargo test --lib plan_authorization`
- `cargo test --lib`
- `rustfmt --check src/control_core.rs src/commands.rs src/types.rs src/observation_store.rs src/codex_transcript.rs src/codex_db.rs src/lib.rs`

Smoke：

- Vite dev server 已启动在 `http://127.0.0.1:4173/`。
- `curl -sS -I http://127.0.0.1:4173/` 返回 `HTTP/1.1 200 OK`。
- 本轮没有真实 Tauri 窗口 / 截图验收；没有可用 browser screenshot 工具，本地没有 Playwright / Puppeteer。

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未新增 `workflow-state.v0.json` 顶层结构。
- 未改 workflow / work item / node / dispatch 状态枚举。
- 未把 worker 汇报写成正式事实。
- 未把 observation / candidate 写成正式记忆。
- 未自动生成 MemoryCandidate。
- 未写正式记忆。
- 未完成全局主管最终结果复核。
- 未完成用户结果接受。
- 未完成完整自动重试系统。
- 未完成真实 Tauri 截图验收。

## 接手建议

下一步拆 C6：全局主管最终结果复核、用户结果查看和阶段 C 验收。

C6 注意：

- 继续基于 C1-C5 授权、prepared dispatch、worker report、process fact observation 和 review / audit 边界。
- 不要把 C5 observation 当正式事实或正式记忆。
- 不要让秘书确认 worker 汇报、过程事实或成果。
- 真实 worker / Codex 执行仍需要新的任务包和用户明确授权。
