# Evidence: Workflow C1 Plan Authorization And Controlled Auto Dispatch Foundation v1

日期：2026-06-04

## 结论

C1 已完成：工作台新增方案授权 sidecar、授权对象读写骨架、授权范围 guard、自动推进 inspect / prepare 前置检查，以及项目工作流侧栏只读“方案授权摘要”。

接受为：

- `plan-authorizations.v1.json` sidecar 骨架已落地，包含 revision、authorizations、audit_events、lock、备份、原子写和损坏 JSON 拒绝覆盖。
- 后端已有 `PlanAuthorization` / `AuthorizedExecutionScope` / `AutoDispatchGuardInput` / `AutoDispatchGuardResult` / `PlanAuthorizationReadModel` 等类型。
- 控制核心能 deterministic 检查角色、agent、读写范围、工具、检查、任务包类型、停止条件和授权状态。
- `inspect_task_package_dispatch_readiness_at`、`prepare_workflow_node_dispatch_at`、`prepare_offline_role_dispatch_at` 已接入授权检查摘要。
- 项目工作流详情侧栏只读展示“方案授权摘要”、范围计数、最近 guard 结果和最多 3 条 blocked reason。

不接受为：

- 阶段 C 完成。
- 自动化工作流产品化闭环完成。
- 项目咨询、用户确认入口、全局主管真实方案复核或真实自动派发完成。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- 真实 Tauri 窗口 / 截图验收完成。

## 关键实现

- 新增后端模块：`prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`。
- 更新后端类型：`prototypes/productized-desktop-shell/src-tauri/src/types.rs`。
- 更新控制核心：`prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`。
- 更新 Tauri command：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`、`src-tauri/src/lib.rs`。
- 更新前端类型和 wrapper：`src/lib/types.ts`、`src/lib/tauri.ts`。
- 新增前端读模型 helper：`src/lib/planAuthorization.ts`。
- 更新项目工作流 UI：`src/App.tsx`、`src/views/ProjectsView.tsx`。
- 更新离线 UI 测试：`tests/offline-permission-dialog.test.tsx`。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留既有 chunk size warning。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib`
  - 177 passed, 1 ignored。
- `rustfmt --check src/plan_authorization_store.rs src/control_core.rs src/commands.rs src/types.rs`

说明：

- `cargo test --lib` 曾在并行运行中出现一次旧 stub last-message 相关偶发失败，单测复跑通过，最终完整 `cargo test --lib` 已通过。
- 本轮未启动真实 Tauri 窗口，未产出截图；项目工作流 UI 只做 SSR 离线测试覆盖和生产 build 验证。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未创建新的 Codex session。
- 未迁移数据库。
- 未新增 `workflow-state.v0.json` 顶层数组。
- 未修改 workflow / work item / node 既有状态枚举。
- 未显示“一键自动执行真实 worker”或未实现 adapter 的真实执行按钮。

## 后续

下一步建议进入 C2：项目咨询方案生成和用户确认入口。C2 仍必须基于 C1 的授权对象和 guard，不应绕过方案授权启动真实 worker。
