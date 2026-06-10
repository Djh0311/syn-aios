# Stage K / K2.5 Architecture Calibration: Command Surface, Product Path, And Memory Consistency v1

日期：2026-06-10

状态：已完成，结论为 `accepted`。本文是 Stage K 在 K2 已完成、K3 开始前插入的架构校准任务包。原 Stage K 目标不变，仍是“自由操控 Codex + 自动化工作流 + 记忆层记录”。本任务用于收敛命令面、产品主路径、工作流派发边界、记忆一致性和架构验收 gate。

记录见：

- `../evidence/2026-06-10-stage-k-k2-5-architecture-calibration-command-surface-product-path-and-memory-consistency-v1.md`
- `../handoffs/2026-06-10-stage-k-k2-5-architecture-calibration-command-surface-product-path-and-memory-consistency-v1-result.md`

本文不授权新的真实 `codex exec` / `codex exec resume`，不授权发送 prompt，不授权读写 `/Users/yoyi/.codex`，不授权 planned adapters 真实接入，不授权 provider credential / model verification。

## 0. 全局主管理解

已知事实：

- K0、K1、K2 已完成。
- K2 已接受为通用 `codex-local` `resume` / `new_session` 产品入口、Product Command 归口、权限确认、Phase A 非真实预检、Phase B 普通入口接线，以及 R1/R2/N1/N2 四个受控真实执行点完成。
- 当前下一步原为 K3 项目工作流真实自动化编排产品化。
- 复核发现底层仍存在 Product Command 主路径、K2/J2 probe 常量、legacy blocked wrapper、MCP canvas runner、transcript viewer、memory capture 跨 sidecar 半事务等并存状态。

本任务判断：

- 不能推翻 Stage K 原计划。
- 不能直接跳进 K3 继续叠功能。
- 必须先完成 K2.5 架构校准，再进入 K3。

## 1. 权威依据

必须服从：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`

必须参考的实现：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/codex_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/orchestrator.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

## 2. 接受范围

K2.5 可接受为：

- 命令面分类清楚。
- Product Command 主路径和 fixture / probe / legacy / viewer / sealed experiment 路径边界清楚。
- MCP canvas 真实 runner 不再构成未分类产品旁路。
- K3 项目工作流派发主路径明确走 `RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture`。
- workspace-write 的项目写入证明有明确实现计划或最小实现。
- memory capture / observation / candidate / formal memory 的跨 sidecar 一致性有 scanner / finding taxonomy 或最小实现。
- Stage K 后续验收有架构扫描 gate。

K2.5 不接受为：

- K3 已完成。
- K4 已完成。
- K5 已完成。
- K6 已完成。
- 新的真实 Codex 执行完成。
- 任意项目无限制自由执行。
- 自动 retry / stop / restart。
- planned adapters 真实接入。
- 自动写 FormalMemory。

## 3. 开发范围

允许修改：

- 架构计划和任务包。
- 后端命令分类 / sealed boundary / scan helper。
- Product Command request / intent builder 的低风险抽象。
- workflow automation bridge 的主路径标记和 fixture-only 边界。
- memory consistency finding / scanner 的低风险只读实现。
- write proof manifest / hash diff 的低风险只读或 runner 后置摘要实现。
- 测试和扫描脚本。

谨慎修改：

- `real_execution_command.rs`：不得破坏 K2 已通过真实执行点。
- `codex_local_runner.rs`：不得新增真实执行授权条件漏洞。
- `project_workflow_automation.rs`：不得把 J2-B bridge 改成普通 K3 主路径。
- `memory_candidate_store.rs` / `formal_memory_store.rs`：不得自动修正式记忆。

默认不修改：

- planned adapters。
- provider credential store。
- mobile / responsive UI。
- workflow state 顶层 schema，除非任务内另写迁移说明。

## 4. 分线任务

Execution 线：

- 分类所有真实执行相关 command。
- 确认 Product Command 是唯一真实执行主路径。
- 处理 K2/J2 hardcoded probe 的 fixture-only 边界。
- 处理 MCP canvas runner sealed / migration 决策。
- 处理 workspace-write proof。

Workflow 线：

- 定义 K3 主路径需要的 generic run unit dispatch contract。
- 确保 J2-B bridge 不再被表述为 K3 普通产品主路径。
- 衔接 worker report、process fact、final review 和 memory capture refs。

Memory 线：

- 定义跨 sidecar consistency finding taxonomy。
- 检查 capture event、observation、candidate、formal memory adoption link、runtime log、audit refs 的缺链路情况。
- 只生成 finding / proposal，不自动写 FormalMemory。

Validation 线：

- 做 command surface 扫描。
- 做裸 `Command::new("codex")` 分类扫描。
- 做 legacy frontend wrapper 扫描。
- 做 prompt persistence / secret / full transcript / readback null / candidate-formal confusion 扫描。

主管线：

- 合并各线结果。
- 决定哪些是 K2.5 阻断项，哪些可 defer 到 K3/K4/K5。
- 只在 checkpoint 完成、阻断或阶段边界变化时同步入口文档。

## 5. 验收步骤

### A. Command Surface Inventory

- 扫描 Tauri command 注册表。
- 给每个执行相关 command 分类。
- 记录普通 UI 是否仍调用 legacy wrapper。
- 记录 MCP canvas blocked command 和内部 runner 的差异。

### B. Product Path Calibration

- 检查 K2 / J2 hardcoded execution points。
- 区分 product generic path 和 fixture/probe path。
- 输出 K3 可复用的 generic dispatch contract。

### C. Memory Consistency Calibration

- 检查 memory capture 跨 sidecar 写入顺序。
- 定义 orphan / partial / missing link finding。
- 若实现 scanner，只读输出 finding，不自动修复。

### D. Write Proof Calibration

- 检查 `writes_project_files` 来源。
- 定义 read-only hash proof 和 workspace-write allowed path proof。
- 若实现最小代码，必须保持 prompt body 不持久化。

### E. Architecture Gate

- 形成 K3 前置 gate。
- 通过后才允许进入 K3。

## 6. 必跑扫描

- `rg -n "Command::new\\(\"codex\"\\)|codex exec|run_real_execution_product_command|execute_workflow_node_dispatch|run_workflow_machine|canvas_start_run|canvas_tick_run" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src`
- `rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun" prototypes/productized-desktop-shell/src`
- `rg -n "prompt_body|full transcript|rollout|secret|token|\\.env|provider credential|keychain|OAuth" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src`
- `rg -n "result_count.*0|readback_unavailable|readback_failed|readback_timed_out|candidate.*formal|observation.*formal" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src`

## 7. 验证要求

如只改文档 / 任务包：

- 扫描确认 K2.5 完成口径已同步。
- 扫描确认下一步已回到 K3，同时没有冒领 K3 已完成。

如改 TypeScript：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

如改 Rust：

- 相关 `cargo test --lib <filter>`
- `cargo test --lib`
- `cargo fmt -- --check` 或 `rustfmt --check` 相关文件

## 8. 禁止项

- 禁止执行新的真实 `codex exec` / `codex exec resume`。
- 禁止发送 prompt。
- 禁止读写 `/Users/yoyi/.codex`。
- 禁止读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、完整 rollout。
- 禁止绕过 Product Command。
- 禁止把 J2-B bridge 说成 K3 主路径。
- 禁止把 memory candidate / observation / knowledge hit 说成 FormalMemory。
- 禁止把 readback unavailable / failed / timed_out 显示成真实 0 条结果。

## 9. 完成产物

必须新增：

- K2.5 evidence。
- K2.5 handoff。

必须同步：

- `CURRENT.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `AUTHORITY.md`
- `docs/plans/README.md`
- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`

同步原则：

- K2.5 开始时只同步“下一步从 K3 调整为 K2.5”。
- K2.5 完成后同步结论和下一步 K3。
