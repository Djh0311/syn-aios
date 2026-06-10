# Stage K / K3 Project Workflow Automation Level A Evidence v1

日期：2026-06-10

状态：主管线 fresh verify 已通过，K3-Level-A 接受为 `accepted`。

主管线复核时间：2026-06-10。

主管线结论：

- K3-Level-A 可接受为产品闭环和非真实验证完成。
- K3 整体不能声明完成，因为 K3-Level-B `mario test` 和隔离项目真实执行点未执行。
- 本轮主管线没有执行新的真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。

## 范围

- 本 evidence 只覆盖 K3-Level-A：产品闭环和非真实验证。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未做 K3-Level-B `mario test` 或隔离项目真实执行点。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## 改动摘要

- `project_workflow_automation.rs`
  - 在既有 Project Workflow Automation Phase A 链路上收敛为 K3-Level-A 产品口径。
  - 五类 run units 继续覆盖 `director_plan` / `developer_execution` / `verifier_check` / `collector_summary` / `director_final_review`。
  - 成功闭环后通过 `memory_capture_bus::capture_event` 写入 `audit_only` capture event，并把 capture event ref 回填到 run units。
  - 继续只走 Product Command preview / prepare / user decision / Phase A no-op。
  - Level A flags 保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
  - J2-B B1/B2 fake-runner / ignored real probe 兼容保留，未作为 K3-Level-A 证据。
- `types.rs` / `src/lib/types.ts`
  - `ProjectWorkflowRunUnit` 增加 `capture_event_refs`。
  - `ProjectWorkflowAutomationReadModel` 增加 `capture_event_count`。
- `runQueue.ts`
  - 运行队列合并 run unit 自带 capture refs 与 memory capture store 反查 refs，且去重。
- `ProjectsView.tsx` / `RunningWorkflowsView.tsx` / `AgentView.tsx`
  - 普通层显示 K3 Level A 自动编排状态、捕获来源、worker report、observation、读回未知和下一步。
  - `completed` 类普通显示收敛为“已记录 / 记录待复核”，避免非真实 Phase A 被理解为真实完成。
- `PermissionDialog.tsx` / `App.tsx`
  - K3 自动编排动作说明改为产品化措辞，继续强调不发送 prompt、不执行真实 Codex、不写 `.codex` 或项目文件。
- `secretaryReadModel.ts`
  - 秘书只读摘要增加捕获来源数量，不新增执行类建议。
- `offline-permission-dialog.test.tsx`
  - 离线 fixture / 断言同步 K3 Level A、capture refs 和 capture count。

## 链路证据

- 用户目标进入 `run_project_workflow_automation_phase_a_at`。
- 后端确保已有 workflow / node / work item。
- 生成五类 run units，绑定 project / workflow / node / work item / task package / memory packet。
- developer run unit 生成 Product Command preview / prepare，写入用户 decision，再运行 Phase A no-op。
- Phase A 输出 readback unavailable，`result_count=null`。
- worker report fixture 写入 C5 既有 worker structured report。
- project director process fact 写入 observation，但不生成 FormalMemory。
- K3 capture source 写入 memory capture sidecar，`candidate_policy=audit_only`，不生成 observation/candidate/FormalMemory。
- run units 回填 product command / runtime / audit / readback / worker report / capture / observation refs。

## 验证结果

主管线 fresh verify 追加结果：

- `cargo test --lib project_workflow_automation`：通过，11 passed / 2 ignored。
- `cargo test --lib memory_capture`：通过，7 passed。
- `cargo test --lib real_execution_command`：通过，36 passed / 7 ignored。
- `cargo test --lib worker_protocol`：通过，8 passed。
- `cargo test --lib`：通过，325 passed / 14 ignored；保留既有 warning：`mcp/protocol.rs invalid_params is never used`。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 scenarios passed。
- `npm run build`：通过；保留既有 Vite chunk size warning。

开发线原始验证记录：

- `cargo test --lib project_workflow_automation`：通过，11 passed / 0 failed / 2 ignored。
- `cargo test --lib real_execution_command`：通过，36 passed / 0 failed / 7 ignored。
- `cargo test --lib session_continuation`：通过，17 passed / 0 failed / 4 ignored。
- `cargo test --lib runtime_log`：通过，6 passed / 0 failed。
- `cargo test --lib memory_capture`：通过，7 passed / 0 failed。
- `cargo test --lib worker_protocol`：通过，8 passed / 0 failed。
- `cargo test --lib workflow_authorization`：通过，1 passed / 0 failed。
- `cargo test --lib formal_memory`：通过，29 passed / 0 failed。
- `cargo test --lib memory_candidate`：通过，9 passed / 0 failed。
- `cargo test --lib`：通过，323 passed / 0 failed / 14 ignored。
- `cargo fmt -- --check`：通过。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，14 scenarios passed。
- `npm run build`：通过；Vite 仍有既有 chunk size warning。

## 扫描分类

### 误导文案

命令：

`rg -n '自动执行已完成|Codex 已收到任务|worker 正在执行|已正式记忆|自动记住|readback.*0 条' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

分类：

- 测试 forbiddenText / fixture 命中：`offline-permission-dialog.test.tsx` 中多处用于断言 UI 不显示误导文案。
- 边界常量命中：`canvasSurfaceBoundaries.ts` 登记禁止文案。
- 正向边界说明命中：`RunningWorkflowsView.tsx`、`projectCanvas.ts` 的“不能显示成 0 条结果”说明。
- 未发现本轮 K3 普通 UI 新增“自动执行已完成 / worker 正在执行 / 已正式记忆 / 自动记住”正向展示。

### 真实执行绕路

命令：

`rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|run_workflow_machine|execute_workflow_node_dispatch' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

分类：

- 历史真实 runner / legacy 命中：`lib.rs`、`mcp/codex_runner.rs`、`commands.rs` 中旧入口和 CLI runner 仍存在，但本轮未接 K3-Level-A。
- Product Command / session continuation preview 和授权说明命中：用于展示或测试“未授权不执行 / Level B 才执行”。
- UI 权限文案命中：说明高风险动作需要确认，或说明离线路径不执行。
- 本轮 K3-Level-A 未新增 `Command::new("codex")`，未调用 `run_workflow_machine` / `execute_workflow_node_dispatch` 作为完成证据。

### 敏感材料

命令：

`rg -n 'secret|token|\.env|keychain|OAuth|provider credential|full transcript|rollout|prompt_body' prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

分类：

- 禁止路径 / deny-list / redaction policy 命中：用于阻断 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 既有真实执行点和 ignored tests 命中：J2-B / PCR9 / K2 等 prompt hash、prompt body runtime-only、readback 边界测试。
- 运行时输入类型命中：`prompt_body` 类型字段存在于受控 Phase B 路径，本轮 K3-Level-A 未调用。
- UI / fixture 命中：用于显示“不读取 / 不持久化”边界。
- 本轮新增 K3 capture event 使用 summary / refs，不保存 prompt body、raw stdout/stderr、secret-like 内容或 full transcript。

## 边界确认

- K3-Level-A 成功路径只写工作台自有 sidecars / workflow state / capture sidecar。
- Product Command Phase A flags 均为 false：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- `readback_unavailable` 的 `result_count` 保持 `null`，UI 显示未知 / 不可用。
- capture event 为 `audit_only`，不是 observation/candidate/FormalMemory。
- FormalMemory 未自动写入。

## 剩余风险 / P2

- K3-Level-B 真实执行点仍未执行，K3 整体不能声明完成。
- 项目页仍保留历史 legacy / H5 / workflow machine 区块的可见说明；本轮只收敛 K3 自动编排相关普通口径。
- 没有做真实 Tauri 窗口截图验收；按任务边界 deferred。
