# Stage L / L0 Post-K Deferred Closure Scope, Permission, And Acceptance Freeze v1

日期：2026-06-10

状态：已完成，结论为 `accepted`。本文是 Stage L 的 L0 任务包，用于冻结 post-K deferred closure / daily-use hardening 的目标、权限、安全边界、分线职责、测试项目、真实执行前置条件和 L1-L6 验收矩阵。L0 是文档 / 只读复核任务，不改产品代码，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。记录见 `../evidence/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md` 与 `../handoffs/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1-result.md`。

## 0. 全局主管理解

已知事实：

- Stage K / K6 final 已完成，结论为 `accepted_with_deferred_items`。
- Stage K 已接受为日常可用工作台 checkpoint，但不接受为严格无缺口完成。
- K3-B1 已执行但失败分类；retry 申请再次被安全审查拒绝。
- K3-B2 依赖 K3-B1 成功和复核，当前不得启动。
- K4 / K5 已完成非真实产品化切片；真实 retry / stop / restart / resume 未实现。
- K6 已完成真实 Tauri 核心入口截图，深层子视图仍未完整验收。

L0 目标：

- 冻结 Stage L 的交付目标和不做项。
- 冻结 L1-L6 顺序、分线、执行授权和验收矩阵。
- 冻结 K3-B1 recovery、K3-B2、operation control、deep Tauri、memory loop 的前置条件。
- 冻结入口文档同步规则：只在 checkpoint 完成、阻断或阶段边界变化时同步。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `tasks/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md`
- `evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md`
- `handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. L0 接受范围

L0 可接受为：

- Stage L 目标和不做项已冻结。
- L1-L6 checkpoint 顺序和前置关系已冻结。
- K3-B1 recovery 的合法路径已冻结：用户手动 exact command 回交、重新风险批准，或更窄的本地执行桥设计。
- K3-B2 前置条件已冻结：K3-B1 成功或等价替代路径通过、allowed path / marker / hash / rollback 工作表完整。
- retry / stop / restart / resume 的真实操作授权原则已冻结。
- 深层 Tauri 子视图截图范围已冻结。
- 记忆层 capture / observation / candidate / FormalMemory confirmation 边界已冻结。
- 分线职责、写集边界和回交要求已冻结。

L0 不接受为：

- L1-L6 已完成。
- K3-B1 retry 已执行或成功。
- K3-B2 可开始。
- 真实 retry / stop / restart / resume 已实现。
- 新的真实 Codex 执行已获授权。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动写 FormalMemory。
- 真实 Tauri 深层截图验收完成。

## 3. L1-L6 验收矩阵

| 任务 | 类型 | 是否允许真实 Codex | 是否允许读写 `/Users/yoyi/.codex` | 是否允许写项目文件 | 权威入口同步 |
| --- | --- | --- | --- | --- | --- |
| L0 范围 / 权限 / 验收矩阵冻结 | 文档 / 只读复核 | 不允许 | 不允许 | 不允许 | L0 完成时同步 |
| L1 K3-B1 recovery product path | 产品路径 / UI / 安全边界 | 默认不允许；retry 必须独立授权 | 默认不允许；retry 必须列最小范围 | 默认不允许 | 完成或阻断时同步 |
| L2 K3-B2 isolated workspace-write | 真实执行点 / 工作流 | 仅 L1 前置满足后允许 | 仅任务包列明最小范围 | 仅 allowed path | 完成或阻断时同步 |
| L3 Operation control hardening | 后端 / 前端 / 状态 | 每个真实操作单独授权 | 仅必要最小范围 | 仅任务包列明 | checkpoint 同步 |
| L4 Deep Tauri subview acceptance | 真实 Tauri / 截图 | 不新增真实执行 | 不新增 `.codex` 范围 | 不新增项目写入 | 完成时同步 |
| L5 Memory capture daily loop | 记忆层 / UI | 默认不新增真实执行 | 默认不新增 `.codex` 范围 | 默认不写项目文件 | 完成时同步 |
| L6 Stage L final acceptance freeze | 文档 / 验收 | 不允许新增真实执行 | 不允许新增 `.codex` 范围 | 不允许新增项目写入 | 必须同步 |

## 4. 分线职责

主管线：

- 维护目标、边界、任务包、入口和最终接受结论。
- 分发多会话协作任务，但不得催促开发线跳过复核。
- 收回每条线的结果后做 P0/P1/P2 和边界复核。

执行线：

- 负责 Product Command、CodexLocalRunner、permission envelope、runtime log、audit、readback、duplicate guard、timeout / failed / blocked 状态。
- 不直接改 UI 风格，不写正式记忆，不绕过 Product Command。

工作流线：

- 负责 run unit、workflow node、worker report、process fact、handoff、run queue。
- 不直接调用裸 CLI，不直接写 FormalMemory。

UI / Tauri 线：

- 负责用户可理解界面、权限弹层、运行中操作控制、深层 Tauri 截图。
- 不新增真实执行语义，不把开发者状态铺到普通主层。

记忆线：

- 负责 capture event、observation、candidate、FormalMemory confirmation、task memory packet。
- 不自动正式化记忆，不把候选说成正式记忆。

复核线：

- 只读审查架构、安全、UI、记忆、测试和证据链。
- 不替主管线做最终接受结论。

## 5. 真实执行前置条件

L1-L3 的任何真实执行点必须冻结：

- `execution_point_id`
- `operation`
- `adapter_id`
- `project_root` / `project_id`
- `workflow_id` / `run_unit_id` / `node_id`
- `target_session_id` 或 new session 创建规则
- `sandbox`
- `allowed_write_roots`
- `denied_paths`
- `prompt_summary` / `prompt_ref` / `prompt_hash`
- `task_memory_packet_ref`
- `permission_envelope_ref`
- `readback_plan`
- `runtime_log_policy`
- `audit_policy`
- `memory_capture_policy`
- `rollback_or_recovery_plan`
- `user_confirmation`

缺任一项不得执行。

## 6. L0 验证要求

L0 执行时至少完成：

- 扫描入口文档，确认 Stage L plan 和 L0 task 是当前下一步。
- 扫描 `K3-B2 可开始`、`K3-B1 retry 成功`、`真实 retry 已实现` 等误导口径，命中必须分类。
- 确认 L0 没有改产品代码。
- 确认 L0 没有执行真实 Codex、没有发送 prompt、没有读写 `/Users/yoyi/.codex`。
- 生成 L0 evidence / handoff。

## 7. L0 完成后下一步

L0 完成并通过复核后，进入 L1：K3-B1 blocked recovery product path。

L1 之前不得启动 K3-B1 retry、K3-B2 或新的真实操作控制。
