# Stage L Post-K Deferred Closure And Daily-Use Hardening Plan v1

日期：2026-06-10

状态：计划已创建，L0 已完成，L1 任务包已创建但治理期暂停执行。根据 `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`，Root Treatment / Stage R 治理阶段插队执行；Stage L / L1-L6 暂挂为 `deferred_during_root_treatment`。本文承接 Stage K / K6 final freeze，仍是治理收口后恢复 Stage L / Stage K 的后续计划，不是当前治理期执行入口。Stage K 已冻结为 `accepted_with_deferred_items`，不是严格无缺口完成；Stage L 的目标仍是把 Stage K 留下的关键 deferred 项收敛成可执行、可验收、可回收的产品化硬化阶段。

本文不是执行任务包，不授权新的真实 `codex exec` / `codex exec resume`，不授权直接读写 `/Users/yoyi/.codex`，不授权启动 K3-B1 retry、K3-B2、真实 retry / stop / restart / resume 或 planned adapters 真实接入。所有真实执行仍必须进入独立任务包、权限 envelope、用户确认、runtime log、audit、readback 和 evidence / handoff。

## 0. 全局主管理解

已知事实：

- Stage K 已完成 K0-K6 并冻结为 `accepted_with_deferred_items`。
- K3-B1 已真实执行但失败分类为 `failed_classified_codex_state_readonly`。
- K3-B1 retry 申请再次被安全审查拒绝；不能 workaround、间接执行或绕过审查。
- K3-B2 依赖 K3-B1 成功和主管复核，当前不得启动。
- K6 已完成真实 Tauri 核心入口 window-only 截图：首页、智能体、运行中工作流、项目、记忆层、知识库、设置、想法箱、Skill、Harness。
- 深层项目 workflow 节点详情、任务记忆包详情、权限弹层、操作控制详情和真实恢复策略仍未完成真实 Tauri 硬化。
- 记忆层已经具备 observation / candidate / FormalMemory 状态机，但“工作台所有操作被记忆层读取、分析、候选化”的日常产品体验仍需要继续打通。

Stage L 目标冻结为：

```text
关闭 Stage K 的关键 deferred 项，让工作台从“日常可用 checkpoint”推进到“可恢复、可解释、可验收的日常硬化版本”。
```

## 1. Stage L 做什么

Stage L 必须做：

- 把 K3-B1 blocked 状态变成用户可理解、可恢复、可审计的产品路径。
- 明确 K3-B1 的安全替代方案：用户手动 exact command 回交、重新风险批准，或更窄的本地执行桥。
- 在 K3-B1 成功或等价替代路径通过后，准备并执行 K3-B2 isolated workspace-write 条件。
- 产品化真实操作控制：retry、stop、restart、resume 的确认、状态、失败、审计、readback 和记忆捕获边界。
- 补真实 Tauri 深层子视图验收：项目 workflow 节点详情、任务记忆包、权限弹层、操作控制详情、失败恢复详情、记忆候选确认。
- 强化记忆闭环：工作台操作进入 capture / observation / candidate，用户确认后才能进入 FormalMemory。
- 继续保持桌面端 Tauri 工作台边界，不做手机端 UI。

Stage L 不做：

- 不做 planned adapters 真实接入。
- 不做 provider credential store、真实 token 读取或 model verification。
- 不开放任意目录无限制执行。
- 不绕过用户确认执行高风险操作。
- 不让 agent 自治批准真实执行、retry、stop、restart 或 FormalMemory 写入。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential 或 rollout 写入普通 sidecar、runtime log、audit、memory observation、memory candidate 或 UI。
- 不把普通浏览器 smoke 当真实 Tauri 验收。

## 2. 推荐 checkpoint

| Checkpoint | 名称 | 类型 | 是否允许真实 Codex | 核心产物 |
| --- | --- | --- | --- | --- |
| L0 | 范围、权限、验收矩阵冻结 | 文档 / 只读复核 | 不允许 | Stage L 目标、分线、权限、验收矩阵冻结 |
| L1 | K3-B1 blocked recovery product path | 后端 / 前端 / 文档 | 默认不允许；如 retry 必须独立授权 | blocked 原因、恢复选项、用户回交路径、失败状态 UI |
| L2 | K3-B2 isolated workspace-write execution closure | 执行点 / 工作流 | 仅在 L1 成功或等价替代通过后允许 | isolated workspace-write run、hash、runtime / audit / readback |
| L3 | Operation control hardening | 后端 / 前端 / 状态机 | 每个真实操作独立授权 | retry / stop / restart / resume 的产品化控制面 |
| L4 | Deep Tauri subview acceptance | 真实 Tauri / 截图 | 不允许新增真实执行 | 深层子视图 window-only 截图和缺口矩阵 |
| L5 | Memory capture to candidate daily loop | 记忆层 / UI | 默认不新增真实执行 | 操作事件到 observation / candidate / FormalMemory confirmation |
| L6 | Stage L final acceptance freeze | 文档 / 验收 | 不允许新增真实执行 | accepted / deferred / blocked freeze 和 post-L 路线 |

## 3. 分线职责

全局主管线：

- 维护任务边界、授权条件、入口文档和最终验收。
- 复核每条开发线回交，不允许把 blocked / failed 冒充完成。
- 真实执行前必须重新确认执行点字段、风险、allowed roots、denied paths、prompt、readback 和 rollback。

执行 / runner 线：

- 只处理 Product Command、CodexLocalRunner、permission envelope、runtime log、audit、readback、duplicate guard、timeout、failed state。
- 不改 UI 风格，不写记忆正式化，不直接读 secret。

工作流线：

- 处理 workflow run unit、项目节点、worker report、process fact、handoff、run queue。
- 不直接调用裸 CLI，不绕过 Product Command。

UI / Tauri 线：

- 处理智能体对话、运行中工作流、项目 workflow 深层详情、权限弹层、操作控制详情和真实 Tauri 截图。
- 不新增真实执行语义，不把开发者内容铺到普通用户主层。

记忆线：

- 处理 capture event、observation、candidate、FormalMemory confirmation、task memory packet 和 lint。
- 不自动写正式记忆，不把 candidate / knowledge hit 说成正式记忆。

复核线：

- 只读审查 P0/P1/P2、架构边界、UI 信息层级、安全边界和验收证据。
- 不承担开发写入，不替主管线做最终接受结论。

## 4. 真实执行前置工作表

任何 L1-L3 真实执行点必须在任务包中列明：

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

缺任一项即阻断，不允许执行。

## 5. Stage L 接受口径

Stage L 可接受为：

- K3-B1 blocked recovery 产品路径完成，并且用户能理解阻断原因、恢复方式和风险。
- K3-B2 在满足前置后完成 isolated workspace-write 或被明确冻结为 blocked/deferred。
- retry / stop / restart / resume 至少形成可确认、可审计、可回收的产品化控制面。
- 真实 Tauri 深层子视图补齐主要证据，或明确缺口矩阵。
- 工作台操作能进入记忆层 capture / observation / candidate，并通过用户确认进入 FormalMemory。
- accepted / deferred / blocked 被最终冻结。

Stage L 不接受为：

- 最终蓝图完整工作台。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 任意目录无限制执行。
- 所有操作无确认自动执行。
- 所有记忆自动正式化。
- 完整自动 UI 测试完成。

## 6. 下一步

L0 已完成，记录见 `../../evidence/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1.md` 与 `../../handoffs/2026-06-10-stage-l-l0-post-k-deferred-closure-scope-permission-and-acceptance-freeze-v1-result.md`。

L1 任务包已创建但治理期暂停执行：`../../tasks/2026-06-10-stage-l-l1-k3-b1-blocked-recovery-product-path-v1.md`。治理收口并恢复 Stage L 前，不启动 K3-B1 retry，不启动 K3-B2，不新增真实操作控制，不读写 `/Users/yoyi/.codex`，不启动新的真实 Tauri 截图验收。
