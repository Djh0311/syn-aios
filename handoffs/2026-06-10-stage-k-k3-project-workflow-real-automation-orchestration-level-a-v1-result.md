# Stage K / K3 Project Workflow Automation Level A Handoff v1

日期：2026-06-10

回交方：product-line 长期开发线

状态：K3-Level-A 已通过主管线 fresh verify，结论为 `accepted`。K3 整体未完成，下一步是 K3-Level-B 真实执行字段冻结和受控执行点。

## 完成的 Level

K3-Level-A：产品闭环和非真实验证。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `evidence/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1.md`
- `handoffs/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1-result.md`

未修改入口文档：`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## 闭环摘要

- 用户目标可生成五类 run units：主管计划、开发线、验证线、回收线、主管复核。
- 开发线 run unit 只走统一 Product Command preview / prepare / user decision / Phase A no-op。
- Phase A 成功路径回填 product command、runtime log、audit、readback、worker report、observation 和 capture refs。
- worker report fixture 回收到既有 C5 worker structured report。
- process fact 通过既有 C5 process fact 写 observation。
- 新增 memory capture source 为 `audit_only`，只作为捕获来源索引，不生成 candidate / FormalMemory。
- 运行队列、项目页、智能体页显示 Level A 闭环、捕获来源、读回未知、worker report、observation 和下一步。

## 边界

- 是否真实执行：否。
- 是否发送 prompt：否。
- 是否读写 `/Users/yoyi/.codex`：否。
- 是否写项目文件：否。
- 是否自动写 FormalMemory：否。
- 是否接 planned adapters / provider / model verification：否。
- 是否做 Level B：否。

## 验证

主管线 fresh verify 已追加：

- `cargo test --lib project_workflow_automation`：通过，11 passed / 2 ignored。
- `cargo test --lib memory_capture`：通过，7 passed。
- `cargo test --lib real_execution_command`：通过，36 passed / 7 ignored。
- `cargo test --lib worker_protocol`：通过，8 passed。
- `cargo test --lib`：通过，325 passed / 14 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 scenarios passed。
- `npm run build`：通过，保留既有 Vite chunk size warning。

开发线原始验证：

- `cargo test --lib project_workflow_automation`：通过。
- `cargo test --lib real_execution_command`：通过。
- `cargo test --lib session_continuation`：通过。
- `cargo test --lib runtime_log`：通过。
- `cargo test --lib memory_capture`：通过。
- `cargo test --lib worker_protocol`：通过。
- `cargo test --lib workflow_authorization`：通过。
- `cargo test --lib formal_memory`：通过。
- `cargo test --lib memory_candidate`：通过。
- `cargo test --lib`：通过。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，保留既有 Vite chunk size warning。

## 扫描

已执行误导文案、真实执行绕路、敏感材料三类扫描。命中主要为：

- 测试中的 forbiddenText / fixture。
- 既有 legacy / Level B / ignored real probe / Product Command 授权说明。
- deny-list / redaction policy / 不读取敏感材料说明。

未发现本轮 K3-Level-A 新增真实 Codex 调用、prompt 发送、`.codex` 读写或自动正式记忆口径。

## 不能声明完成的事项

- 不能声明 K3 整体完成，因为 Level B 真实 `mario test` 和隔离项目执行点未执行。
- 不能声明 K4 记忆捕获体验完成。
- 不能声明 K5 retry / stop / restart 或失败恢复完成。
- 不能声明 K6 dogfood / 真实 Tauri 验收完成。
- 不能声明 Stage K 完成。

## 建议主管线处理

- K3-Level-A 已接受。
- 若进入 K3-Level-B，需单独冻结真实执行字段工作表和用户确认，不复用本轮 Level A 作为真实执行授权。
