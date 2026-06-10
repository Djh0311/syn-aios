# Handoff: Workflow C1 Plan Authorization And Controlled Auto Dispatch Foundation v1

日期：2026-06-04

## 当前状态

C1 已完成并通过验收。当前可从“C1 待执行”更新为“C1 已完成，下一步 C2”。

## 已做内容

- 新增方案授权 sidecar：`plan-authorizations.v1.json`。
- 新增 `PlanAuthorization`、`AuthorizedExecutionScope`、`AutoDispatchGuard*`、`PlanAuthorizationReadModel` 等后端 / 前端类型。
- 新增授权 store：load / create / user confirmation / global boundary review / revoke / inspect。
- 新增控制核心 `inspect_auto_dispatch_scope`。
- 把授权检查接到任务包 readiness、节点 prepare、离线角色 prepare。
- 项目工作流侧栏新增只读“方案授权摘要”。
- 离线测试覆盖 UI 摘要和 blocked reason。
- Rust 测试覆盖无授权、待用户确认、待全局复核、active 通过、写范围越界、角色 / agent 越界、revoked / paused / expired、停止条件、inspect audit。

## 验收命令

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib plan_authorization`
- `cargo test --lib workflow_authorization`
- `cargo test --lib`
- `rustfmt --check src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs`

## 明确未做

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未接 Claude / OpenClaw / OpenCode。
- 未新增 workflow state 顶层结构。
- 未改 workflow / work item / node 状态枚举。
- 未做真实 Tauri 窗口或截图验收。

## 接手建议

下一步是 C2：项目咨询方案生成和用户确认入口。

C2 注意：

- 只能把方案草案和用户确认流接到 C1 的授权对象。
- 不能绕过 C1 guard 自动派发。
- 不能把 pending / needs_review 显示成“已自动执行”。
- 不要把全局主管复核做成逐条 worker 日报确认。
- 继续不读写 `/Users/yoyi/.codex`，除非新任务包和用户另行明确授权。
