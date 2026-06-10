# Handoff: Workflow C3 Global Boundary Review And Authorization Activation v1

日期：2026-06-04

## 当前状态

C3 已完成并通过代码验收。当前入口应更新为“C3 已完成，下一步 C4”。

## 已做内容

- 新增严格 C3 wrapper：`record_global_boundary_review`。
- 复用 `plan-authorizations.v1.json`，不新增新的事实源。
- `PlanAuthorizationGlobalBoundaryReview` 兼容追加 `source_proposal_id`、`checklist`、`findings`、`reviewed_scope_fingerprint`。
- C3 approved 前会校验：
  - proposal 存在且 status 为 `user_confirmed`。
  - proposal `plan_authorization_id` 匹配 authorization。
  - authorization `source_proposal_id` 匹配 proposal。
  - project_id / workflow_id 匹配。
  - authorization 有 user confirmation。
  - checklist 全部为 true。
  - findings 不包含 blocking。
- approved 后 authorization 进入 `active`。
- needs_changes / blocked 后 authorization 进入 `paused`，C1 guard 继续阻断。
- active 后 guard 对匹配输入返回 `authorized`，对越界写入仍返回 `blocked`。
- 项目工作流侧栏新增“全局边界复核”卡片。
- 确认弹层展示复核结论、摘要、授权对象、方案标题、目标、读写范围、工具 / 检查、停止条件和 finding。
- UI 明确“授权有效；仍未派发 worker”，不显示自动派发已开始。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib global_boundary_review`
- `cargo test --lib plan_authorization`
- `cargo test --lib project_consultation_proposal`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`
- `rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/control_core.rs src/commands.rs src/types.rs`
- Vite HTTP smoke：`npm run dev -- --port 5174` + `curl -sS http://127.0.0.1:5174/`

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未新增 workflow state 顶层结构。
- 未改 workflow / work item / node 状态枚举。
- 未创建 task package、prepared dispatch 或 workflow machine run。
- 未做真实窗口 / 浏览器截图验收；本轮没有暴露可用 browser / screenshot 工具，项目未安装 Playwright。

## 接手建议

下一步是 C4：项目主管拆任务和授权范围内 prepared auto dispatch。

C4 注意：

- 必须基于 C3 active authorization。
- 必须继续调用 C1 guard；不能把 active 解释成可以任意派发。
- C4 可以做 prepared dispatch，但真实 worker / Codex 执行仍需要新的任务包和用户明确授权。
- 不能把 C3 的授权有效解释为 worker 已启动或自动化工作流闭环完成。
