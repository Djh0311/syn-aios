# Handoff: Workflow C4 Project Director Task Decomposition And Authorized Prepared Auto Dispatch v1

日期：2026-06-04

## 当前状态

C4 已完成并通过验收。当前入口应更新为“C4 已完成，下一步 C5”。

## 已做内容

- 新增 `preview_project_director_task_plan` 和 `prepare_authorized_auto_dispatch`。
- C4 准备链路要求 C3 active authorization，并校验 C2 confirmed proposal 与 C1 authorization 回链。
- planned task 继续复用 C1 guard；越界 scope 会返回 blocked，不会静默丢弃。
- in-scope planned task 会准备 work item、worker node、task package artifact 和 M6 task memory packet frozen snapshot。
- 已有 active binding 时创建 `state: "prepared"` dispatch；缺 binding 时返回 `needs_binding`，不创建可执行 prepared dispatch。
- prepared dispatch 保持 `started_at`、`ended_at`、`exit_code` 和 transcript readback 为空。
- 重复 prepare 保持幂等，不重复创建 dispatch。
- 项目工作流侧栏新增“项目主管拆任务”摘要卡，确认弹层明确只创建准备记录，不启动 worker。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib project_director_task_plan`
- `cargo test --lib authorized_prepared_dispatch`
- `cargo test --lib task_memory_injection`
- `cargo test --lib plan_authorization`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`
- `rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/task_memory_injection.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`
- 禁止文案搜索：`worker 已执行`、`自动派发已开始`、`Codex 已收到任务`、`worker 执行中` 在 `prototypes/productized-desktop-shell/src` 无命中。

截图证据：

- `evidence/2026-06-04-workflow-c4-ui-smoke.png`
- 该截图是普通浏览器静态壳 smoke，不是真实 Tauri 数据桥验收。

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未新增 `workflow-state.v0.json` 顶层结构。
- 未改 workflow / work item / node / dispatch 状态枚举。
- 未启动 worker。
- 未写 execution attempt。
- 未做 dispatch readback。
- 未完成 worker 结构化汇报。
- 未完成项目主管过程事实确认。
- 未完成失败 / readback / 权限可见化。
- 未完成自动化工作流产品化闭环。

## 接手建议

下一步是 C5：worker 结构化汇报、项目主管过程事实确认和失败 / readback / 权限可见化。

C5 注意：

- worker 汇报不能直接变成正式事实。
- 过程事实必须由项目主管确认后进入 observation / candidate / formal memory 边界。
- readback 失败不能伪装成真实 0 条读回。
- 真实 Codex / worker 执行仍需要新的任务包和用户明确授权。
