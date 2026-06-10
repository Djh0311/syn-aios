# Evidence: Workflow C3 Global Boundary Review And Authorization Activation v1

日期：2026-06-04

## 结论

C3 已完成：工作台新增全局主管方案边界复核 wrapper、checklist / findings / proposal-authority 回链校验、approved 授权生效路径、needs_changes / blocked 暂停路径、guard 验证摘要，以及项目工作流侧栏“全局边界复核”卡片和确认弹层。

接受为：

- C2 confirmed `ProjectConsultationProposal` 到 C1 `PlanAuthorization` 的回链一致性校验已落地。
- `PlanAuthorizationGlobalBoundaryReview` 兼容追加 `source_proposal_id`、`checklist`、`findings` 和 `reviewed_scope_fingerprint`。
- 新增 `record_global_boundary_review` Tauri command；内部复用 `plan-authorizations.v1.json`，不新增事实源。
- `approved` 要求 proposal 已用户确认、authorization 有用户确认、回链匹配、checklist 全部通过、无 blocking finding；成功后 authorization 进入 `active`。
- `needs_changes` / `blocked` 会让 authorization 进入 `paused`，C1 guard 继续阻断自动推进。
- active 后 C1 guard 对匹配输入返回 `authorized`，越界输入仍返回 `blocked`。
- 项目工作流侧栏显示“全局边界复核”卡片、复核状态、active authorization id、guard 验证摘要、最多 3 条 blocked reason / finding，以及 `批准并生效`、`要求修改`、`阻断方案` 动作。

不接受为：

- 项目主管已经拆任务。
- 项目主管已经自动派发 worker。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- 自动化工作流产品化闭环完成。
- 全局主管最终结果复核完成。
- 全局主管逐条确认 worker 日常汇报。

## 关键实现

- 更新后端 store：`prototypes/productized-desktop-shell/src-tauri/src/plan_authorization_store.rs`。
- 更新后端类型：`prototypes/productized-desktop-shell/src-tauri/src/types.rs`。
- 更新 Tauri command：`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`、`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`。
- 更新前端类型和 wrapper：`prototypes/productized-desktop-shell/src/lib/types.ts`、`prototypes/productized-desktop-shell/src/lib/tauri.ts`。
- 更新前端读模型 helper：`prototypes/productized-desktop-shell/src/lib/planAuthorization.ts`。
- 更新项目工作流 UI：`prototypes/productized-desktop-shell/src/App.tsx`、`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`、`prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`。
- 更新离线 UI 测试：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留既有 chunk size warning。
- `cargo test --lib global_boundary_review`
  - 5 passed。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib project_consultation_proposal`
  - 5 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib`
  - 187 passed, 1 ignored。
- `rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/control_core.rs src/commands.rs src/types.rs`
- Vite 本地 HTTP smoke：
  - 沙箱内启动 Vite 监听 127.0.0.1:5174 因 `EPERM` 失败；按环境规则经用户审批提权启动。
  - `curl -sS http://127.0.0.1:5174/` 返回 Vite HTML。
  - 验收后已停止 dev server，5174 端口已释放。

说明：

- 真实窗口 / 浏览器截图验收未完成：本轮工具发现没有暴露可用 in-app browser / screenshot 工具，项目依赖也没有 Playwright。已做 Vite 本地 HTTP smoke。
- `cargo test --lib` 保留既有 warning：`JsonRpcError::invalid_params` 未使用。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未迁移数据库。
- 未新增 `workflow-state.v0.json` 顶层数组。
- 未修改 workflow / work item / node 既有状态枚举。
- 未在 C3 自动创建 task package、prepared dispatch 或 workflow machine run。
- 未把 proposal、authorization 或 review 写成正式记忆。
- UI 未显示“worker 已执行 / 自动派发已开始 / Codex 已收到任务”。
- approved 路径会让 C1 authorization 进入 `active`；该能力已通过临时测试 sidecar 验证，不等于 worker 已启动。

## 后续

下一步建议进入 C4：项目主管拆任务和授权范围内 prepared auto dispatch。C4 必须基于 C3 active authorization 和 C1 guard，不能绕过授权范围检查直接真实派发 worker。
