# Evidence: Workflow C4 Project Director Task Decomposition And Authorized Prepared Auto Dispatch v1

日期：2026-06-04

## 结论

C4 已完成：工作台新增项目主管拆任务预览、C3 active authorization + C2 proposal 回链校验、C1 guard 范围校验、任务包 / worker work item / worker node / artifact 准备、M6 task memory packet snapshot 注入，以及受控 `prepared` dispatch 落账。

接受为：

- 项目主管可以基于 C3 active `PlanAuthorization` 生成 deterministic planned tasks。
- C4 会校验 C2 confirmed proposal 与 C3 authorization 回链，缺 active 授权或回链不匹配会拒绝。
- planned task 会经过 C1 guard 校验；越界读写范围、工具、检查、role / agent / task package kind 会被阻断。
- in-scope planned task 可以生成 work item、worker node、task package artifact 和 task memory packet frozen snapshot。
- 已有 active binding 且 guard authorized 时会创建 `state: "prepared"` dispatch，并保持幂等，不重复创建 dispatch。
- 缺 active binding 时返回 `needs_binding`，不创建可执行 prepared dispatch。
- prepared dispatch 保持 `started_at` / `ended_at` / `exit_code` / transcript readback 为空。
- 项目工作流侧栏显示“项目主管拆任务”摘要、授权对象、planned / prepared / blocked / needs_binding 数量、记忆快照摘要和最多 3 个 planned task。
- 确认弹层明确“只创建准备记录，不启动 worker”。

不接受为：

- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` / `codex exec resume` 已执行。
- worker 已结构化汇报。
- 项目主管已确认过程事实。
- 失败 / readback / 权限可见化已经完成。
- 全局主管最终结果复核完成。
- 自动化工作流产品化闭环完成。

## 关键实现

- 更新后端类型和命令：`prototypes/productized-desktop-shell/src-tauri/src/types.rs`、`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`、`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`。
- 复用记忆包注入：`prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`。
- 更新前端类型和 Tauri wrapper：`prototypes/productized-desktop-shell/src/lib/types.ts`、`prototypes/productized-desktop-shell/src/lib/tauri.ts`。
- 新增前端摘要 helper：`prototypes/productized-desktop-shell/src/lib/projectDirectorTaskPlan.ts`。
- 更新项目工作流 UI 和确认弹层：`prototypes/productized-desktop-shell/src/App.tsx`、`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`、`prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`。
- 更新离线 UI 测试：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`。

## 验收结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
  - 输出：`offline interaction tests passed: 9`
- `npm run build`
  - Vite build 通过；保留 chunk size warning。
- `cargo test --lib project_director_task_plan`
  - 3 passed。
- `cargo test --lib authorized_prepared_dispatch`
  - 2 passed。
- `cargo test --lib task_memory_injection`
  - 5 passed。
- `cargo test --lib plan_authorization`
  - 8 passed。
- `cargo test --lib workflow_authorization`
  - 1 passed。
- `cargo test --lib`
  - 192 passed, 1 ignored。
- `rustfmt --check src/plan_authorization_store.rs src/project_consultation_proposal_store.rs src/task_memory_injection.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`
- 前端禁止文案搜索：
  - `rg -n -F "worker 已执行" prototypes/productized-desktop-shell/src` 无命中。
  - `rg -n -F "自动派发已开始" prototypes/productized-desktop-shell/src` 无命中。
  - `rg -n -F "Codex 已收到任务" prototypes/productized-desktop-shell/src` 无命中。
  - `rg -n -F "worker 执行中" prototypes/productized-desktop-shell/src` 无命中。

说明：

- `cargo test --lib` 保留既有 warning：`JsonRpcError::invalid_params` 未使用。
- `npm run build` 保留 Vite chunk size warning，不影响构建通过。
- UI smoke 截图已存在：`evidence/2026-06-04-workflow-c4-ui-smoke.png`。本轮复核该截图为普通浏览器静态壳，显示普通浏览器没有 Tauri 数据桥；该截图只证明静态壳可渲染，不证明真实 Tauri 数据桥或真实项目数据验收完成。

## 边界确认

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec`。
- 未执行 `codex exec resume`。
- 未创建新的 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未迁移数据库。
- 未新增 `workflow-state.v0.json` 顶层数组。
- 未修改 workflow / work item / node / dispatch 既有状态枚举。
- 已创建的是 prepared dispatch 准备记录，不是执行态 dispatch。
- 未启动任何 worker。
- 未写 execution attempt started / running / completed。
- 未做 dispatch readback。
- 未把任务包内容、prepared prompt 或 worker 计划写成正式记忆。
- 未显示未实现的“一键真实执行 worker”按钮。

## 后续

下一步建议进入 C5：worker 结构化汇报、项目主管过程事实确认，以及失败 / readback / 权限可见化。C5 仍不能把 worker 汇报直接写正式事实，必须先经过项目主管确认和既有记忆 / observation 边界。
