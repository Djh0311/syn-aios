# Stage K / K2 General Codex Resume And New Session Product Entry v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_deferred_items`。Level A 契约、发送预览、prepare、用户确认、Phase A 非真实预检、Phase B 普通入口接线、R1/R2/N1/N2 env-gated 真实执行测试 harness 和四个真实执行点均已完成。R1 `resume/read-only`、R2 `resume/workspace-write`、N1 `new_session/read-only`、N2 `new_session/workspace-write` 均通过受控 Product Command / `codex-local` runner 验收。本文是 Stage K 的 K2 任务包，用于把既有受控 Product Command / `codex-local` runner 能力推进为普通用户可用的通用 `resume` / `new session` 产品入口。不得裸调用 CLI，不得绕过 Product Command，不得把旧 H/J/PCR probe 结果当作 K2 已完成证据。

## 0. 全局主管理解

已知事实：

- K0 已冻结 Stage K 范围、权限、测试项目和真实执行点字段工作表。
- K1 正在把智能体页普通层收敛为对话工作区，但不改后端执行语义。
- Stage J / J1-J2、H2/H3/H5、PCR10 已证明 `resume`、workspace-write probe、new-session probe 和统一 Product Command 方向可行。
- 当前缺口是“日常产品入口”：用户在工作台里选择项目 / 对话 / 任务，确认影响范围后，稳定进入 Product Command、runtime log、audit、readback、运行队列和记忆捕获链路。

风险：

- 如果 K2 直接前端拼 `codex exec`，会绕过工作台边界。
- 如果 K2 复用历史 probe 授权，会把一次性探针误升级为通用产品权限。
- 如果保存 prompt body、完整 transcript 或敏感路径，会破坏记忆层边界。
- 如果 new-session 失败被包装成成功，会误导 K3 工作流闭环。

K2 的开发原则：

- 所有真实执行必须归口 `real_execution_product_command`。
- 所有执行前必须有用户可读 permission envelope。
- prompt body 只作为运行时输入，不进入普通 sidecar、runtime log、audit 或 memory。
- readback unavailable / failed / timed_out 的 `result_count` 必须保持 `null`，不能显示为 0。
- 真实执行后只记录脱敏 summary、refs、hash、marker、status 和必要短结果。

## 1. 权威依据

必须服从：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/memory-layer-design-v1.md`

必须参考的实现：

- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`

## 2. 接受范围

K2 可接受为：

- 工作台有通用 `codex-local` `resume` / `new session` 产品入口。
- 前端只提交结构化 command/intention，不拼 CLI。
- 后端统一 Product Command 可承载 `resume` 和 `new_session`。
- 执行前展示项目、cwd、sandbox、allowed write roots、denied paths、prompt summary/ref/hash、记忆包、readback plan、runtime/audit 影响。
- 用户确认后可执行真实 `resume/read-only`、`resume/workspace-write`、`new-session/read-only`、`new-session/workspace-write` 四类验收点。
- 执行结果写入 Product Command attempt、continuation attempt、runtime log、audit refs、readback summary 和运行队列可读状态。
- 失败、阻断、duplicate、readback unavailable / failed / timed_out 都有明确分类。

K2 不接受为：

- 任意目录无限制 Codex 控制台。
- planned adapters 真实接入。
- provider credential / model verification。
- 自动 retry / stop / restart。
- K3 项目工作流真实派发完成。
- K4 记忆捕获体验完成。
- FormalMemory 自动写入。
- Stage K 完成。

## 3. 开发范围

允许修改：

- Rust Product Command / runner bridge / commands / types。
- 前端 TS 类型和 Tauri wrapper。
- `AgentView.tsx` 的发送预览 / 权限确认入口。
- `PermissionDialog.tsx` 的 K2 用户确认文案。
- Runtime/readback/diagnostic read model 的必要衔接。
- 离线前端测试和 Rust 聚焦测试。

谨慎修改：

- `memory_capture_bus.rs`：只允许消费 K2 脱敏 execution summary 生成 observation / candidate 来源，不允许 prompt body / full transcript 入库。
- `runQueue.ts` / `RunningWorkflowsView.tsx`：只允许显示 K2 运行状态，不做 retry/stop/restart 真实操作。

默认不修改：

- planned adapter 执行路径。
- provider credential store。
- workflow state 顶层 schema，除非有迁移说明和测试。
- FormalMemory adoption / lifecycle 规则。

## 4. 产品链路

```text
Agent 页输入任务
-> preview K2 user command
-> permission envelope
-> 用户确认
-> Product Command prepare / decision
-> Phase A audit/no-op trace
-> Phase B codex-local real run
-> continuation attempt
-> runtime log / audit
-> readback summary
-> run queue item / user-facing summary
-> memory capture observation/candidate source
```

## 5. UI 显示边界确认

本任务改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端可见 UI。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改后端 Product Command 执行语义。

普通层允许显示：

- 运行模式：继续已有对话 / 新建对话。
- 项目、目标对话、任务摘要、权限影响、记忆注入预览、readback 预期。
- 用户确认按钮。
- 执行中 / 读回中 / 成功 / 失败 / 需要处理。

普通层禁止默认显示：

- raw CLI。
- raw prompt body 持久化说明。
- full transcript / rollout path。
- secret / token / credential 原文。
- Product Command 内部 revision / sidecar path / raw audit ids。
- H/J/PCR 阶段术语。

开发者详情允许显示：

- Product Command id / store revision / audit refs / runtime refs / readback refs。
- command plan summary。
- failure classification detail。

## 6. 真实执行点字段工作表

### K2-R1：`resume/read-only` / `mario test`

| 字段 | 值 |
| --- | --- |
| `execution_point_id` | `stage-k-k2-r1-mario-test-resume-read-only` |
| `operation` | `resume` |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/Documents/mario test` |
| `project_id` | `project:users-yoyi-documents-mario-test` |
| `workflow_id` / `run_unit_id` / `node_id` | `workflow:stage-k:k2:mario-test` / `run-unit:stage-k:k2:r1` / `node:stage-k:k2:r1` |
| `target_session_id` | 历史参考总指导 session `019e798a-6ce5-76c3-b8ee-33bd0fda841f`；执行前必须确认仍可用 |
| `sandbox` | `read-only` |
| `allowed_write_roots` | 空数组 |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、未授权项目文件 |
| `prompt_summary` | `Stage K K2 resume/read-only health probe for mario test.` |
| `prompt_ref` | `prompt:stage-k:k2:r1:mario-test-resume-read-only` |
| `prompt_hash` | `2dc6d059fe5373ba547da91bd2b28296ab0ec15450cb7264f26243a3bff86e1d` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k2:r1:mario-test`，无正式注入时说明 no included memories |
| `permission_envelope_ref` | `permission:stage-k:k2:r1` |
| `readback_plan` | 期待 marker `K2_R1_MARIO_TEST_RESUME_READ_ONLY_OK_2026_06_10`；失败分类保持 `result_count=null` |
| `runtime_log_policy` | 写 K2 runtime log summary，不写 prompt body |
| `audit_policy` | 写 preview/decision/attempt/readback audit refs |
| `baseline_hashes` | 执行前记录 `index.html`、`styles.css`、`game.js`、`README.md` hash |
| `.codex_scope` | 允许 runner 必要写 Codex 自身状态；readback 只读目标 session last message / marker |
| `dirty_worktree_policy` | 不回退用户改动；只核对 baseline 文件 hash |
| `rollback_policy` | read-only 无项目写入；如项目文件变化即阻断验收 |
| `user_confirmation` | 必须 `confirmed_by: "user"` |

### K2-R2：`resume/workspace-write` / `mario test`

| 字段 | 值 |
| --- | --- |
| `execution_point_id` | `stage-k-k2-r2-mario-test-resume-workspace-write` |
| `operation` | `resume` |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/Documents/mario test` |
| `project_id` | `project:users-yoyi-documents-mario-test` |
| `workflow_id` / `run_unit_id` / `node_id` | `workflow:stage-k:k2:mario-test` / `run-unit:stage-k:k2:r2` / `node:stage-k:k2:r2` |
| `target_session_id` | 历史参考开发线 session `019e798a-ac37-7771-b982-e38084fcd22e`；执行前必须确认仍可用 |
| `sandbox` | `workspace-write` |
| `allowed_write_roots` | `/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/` |
| `allowed_write_path` | `/Users/yoyi/Documents/mario test/.workbench/stage-k/k2/resume-workspace-write-probe.md` |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、除 allowed path 外的项目文件 |
| `prompt_summary` | `Stage K K2 resume/workspace-write probe writing only the allowed marker file.` |
| `prompt_ref` | `prompt:stage-k:k2:r2:mario-test-resume-write` |
| `prompt_hash` | `03091a7bfc9e8a9b86bcc79f421f8b0ab982cd513cca7e1b8346afc709205c49` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k2:r2:mario-test` |
| `permission_envelope_ref` | `permission:stage-k:k2:r2` |
| `readback_plan` | 期待 marker `K2_R2_MARIO_TEST_RESUME_WRITE_OK_2026_06_10`；失败分类保持 `result_count=null` |
| `runtime_log_policy` | 写 K2 runtime log summary，不写 prompt body |
| `audit_policy` | 写 preview/decision/attempt/readback audit refs |
| `baseline_hashes` | 执行前后记录核心项目文件 hash；记录 allowed write file hash |
| `.codex_scope` | 允许 runner 必要写 Codex 自身状态；readback 只读目标 session last message / marker |
| `dirty_worktree_policy` | 不回退用户改动；除 allowed path 外任何核心文件变化均阻断 |
| `rollback_policy` | allowed file 可删除或保留为 evidence，任务包必须记录选择 |
| `user_confirmation` | 必须 `confirmed_by: "user"` |

### K2-N1：`new-session/read-only` / Stage K 隔离项目

| 字段 | 值 |
| --- | --- |
| `execution_point_id` | `stage-k-k2-n1-isolated-new-session-read-only` |
| `operation` | `new_session` |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project` |
| `project_id` | `project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project` |
| `workflow_id` / `run_unit_id` / `node_id` | `workflow:stage-k:k2:isolated` / `run-unit:stage-k:k2:n1` / `node:stage-k:k2:n1` |
| `target_session_id` | 新 session 成功后由 Product Command / readback 记录 |
| `sandbox` | `read-only` |
| `allowed_write_roots` | 空数组 |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、`product-line` 非 fixture 路径 |
| `prompt_summary` | `Stage K K2 new-session/read-only health probe in isolated project.` |
| `prompt_ref` | `prompt:stage-k:k2:n1:isolated-new-session-read-only` |
| `prompt_hash` | `b19d41bf5e37cd41af5630cd71241f729e576ed4409574b10594c56e2d359833` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k2:n1:isolated` |
| `permission_envelope_ref` | `permission:stage-k:k2:n1` |
| `readback_plan` | 期待 marker `K2_N1_ISOLATED_NEW_SESSION_READ_ONLY_OK_2026_06_10`；失败分类保持 `result_count=null` |
| `runtime_log_policy` | 写 K2 runtime log summary，不写 prompt body |
| `audit_policy` | 写 preview/decision/attempt/readback audit refs |
| `baseline_hashes` | 执行前记录 fixture 目录状态；执行后确认无项目文件写入 |
| `.codex_scope` | 允许 runner 必要创建 Codex session；readback 只读新 session 最小 last message / marker |
| `dirty_worktree_policy` | fixture 目录必须隔离；不修改产品源码 |
| `rollback_policy` | read-only 无项目写入；可清理 fixture 空目录 |
| `user_confirmation` | 必须 `confirmed_by: "user"` |

### K2-N2：`new-session/workspace-write` / Stage K 隔离项目

| 字段 | 值 |
| --- | --- |
| `execution_point_id` | `stage-k-k2-n2-isolated-new-session-workspace-write` |
| `operation` | `new_session` |
| `adapter_id` | `codex-local` |
| `project_root` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project` |
| `project_id` | `project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project` |
| `workflow_id` / `run_unit_id` / `node_id` | `workflow:stage-k:k2:isolated` / `run-unit:stage-k:k2:n2` / `node:stage-k:k2:n2` |
| `target_session_id` | 新 session 成功后由 Product Command / readback 记录 |
| `sandbox` | `workspace-write` |
| `allowed_write_roots` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/` |
| `allowed_write_path` | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/.workbench/stage-k/k2/new-session-write-probe.md` |
| `denied_paths` | secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout、`product-line` 非 fixture 路径、fixture 中除 allowed path 外文件 |
| `prompt_summary` | `Stage K K2 new-session/workspace-write probe writing only the allowed marker file.` |
| `prompt_ref` | `prompt:stage-k:k2:n2:isolated-new-session-write` |
| `prompt_hash` | `3ff79f634ab4eaaf341878e62e0d8542d39b4c1d4f9cee67d69ba823849ebead` |
| `task_memory_packet_ref` | `memory-packet:stage-k:k2:n2:isolated` |
| `permission_envelope_ref` | `permission:stage-k:k2:n2` |
| `readback_plan` | 期待 marker `K2_N2_ISOLATED_NEW_SESSION_WRITE_OK_2026_06_10`；失败分类保持 `result_count=null` |
| `runtime_log_policy` | 写 K2 runtime log summary，不写 prompt body |
| `audit_policy` | 写 preview/decision/attempt/readback audit refs |
| `baseline_hashes` | 执行前记录 fixture 目录状态；执行后记录 allowed write file hash |
| `.codex_scope` | 允许 runner 必要创建 Codex session；readback 只读新 session 最小 last message / marker |
| `dirty_worktree_policy` | fixture 目录必须隔离；不修改产品源码 |
| `rollback_policy` | allowed file 可删除或保留为 evidence，任务包必须记录选择 |
| `user_confirmation` | 必须 `confirmed_by: "user"` |

## 7. 实施步骤

1. 补后端 Product Command input/output，使 `resume` 和 `new_session` 都能通过统一产品命令表达。
2. 补 permission envelope 和 blocked reasons：缺项目、缺 session/new-session strategy、缺 prompt hash、越界 allowed roots、敏感 denied paths 缺失、duplicate active attempt 必须阻断。
3. 补 Phase B bridge：`resume` 复用既有真实 resume runner；`new_session` 复用 H3-B runner 能力，但必须包进 Product Command attempt / continuation / runtime / audit / readback。
4. 补前端 Tauri wrapper 和 TS 类型。
5. Agent 页从 K1 的输入区接入 K2 preview / permission / confirm / run 状态；普通层不展示 raw CLI。
6. 执行默认 fake / no-op / unit test，不触发真实 Codex。
7. 完成 R1/R2/N1/N2 执行前检查后，再逐项进入真实执行；每项执行后记录 evidence/handoff。
8. 完成 K2 总复核和权威入口 checkpoint 同步。

## 8. 测试清单

Rust：

- `cargo test --lib real_execution_command`
- `cargo test --lib session_continuation`
- `cargo test --lib codex_local_runner`
- `cargo test --lib runtime_log`
- `cargo test --lib memory_capture`
- `cargo test --lib project_workflow_automation`
- `cargo test --lib`
- `cargo fmt -- --check`

前端：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

扫描：

- 禁止前端直接拼 `codex exec`。
- 禁止新增裸 `Command::new("codex")` 非 runner 归口。
- 禁止 prompt body / full transcript / secret-like 内容写入普通 store。
- 禁止 readback unknown 显示为 0。
- 禁止 planned adapters 显示为可执行。

真实执行验收：

- R1 `resume/read-only` 成功，项目核心 hash 不变。
- R2 `resume/workspace-write` 成功，只写 allowed file，核心 hash 不变。
- N1 `new-session/read-only` 成功，fixture 无项目写入。
- N2 `new-session/workspace-write` 成功，只写 allowed file。
- 至少一个失败 / blocked / duplicate 场景进入待办或失败摘要，不伪装成功。

## 9. 回交要求

开发线每轮回交必须包含：

- 改动文件。
- 实现路径。
- 执行点编号。
- 是否真实执行。
- `.codex` 副作用。
- 项目写入副作用。
- readback status / result_count。
- runtime/audit refs。
- baseline hash。
- 不能声明事项。

复核线：

- 默认只读。
- 真实执行复核只读取任务包列明的 evidence/handoff 和脱敏结果；不得扩展读取 `.codex` 历史。

## 10. 当前状态

K2 任务包已写。

Level A 已完成：

- 后端新增 Tauri command：`run_real_execution_product_command_new_session_phase_b`。
- 后端新增薄 wrapper：`run_real_execution_product_command_new_session_phase_b_at`，复用既有 Product Command new-session Phase B bridge。
- 前端新增 TS 类型：`H3RealNewSessionAuthorizationMatrix`、`RunControlledSessionContinuationRealNewSessionH3BInput/Output`、`RunRealExecutionProductCommandNewSessionPhaseBInput`。
- 前端新增 Tauri wrapper：`runRealExecutionProductCommandNewSessionPhaseB`。
- Agent 普通输入区接入 K2 结构化发送预览：支持继续已有对话和新建对话预览。
- Agent 普通发送预览卡接入非真实产品流：`写入准备` -> `用户确认` -> `记录预检`，分别走 Product Command prepare、decision 和 Phase A no-op trace。
- Agent 普通发送预览卡新增 `确认执行 Codex`，只在 Phase A 无阻断后启用；`resume` 走 `runRealExecutionProductCommandPhaseB`，`new_session` 走 `runRealExecutionProductCommandNewSessionPhaseB`。
- Phase B 前端入口不拼 CLI，不绕过 Product Command；真实执行必须等待 R1/R2/N1/N2 执行点逐项验收和 evidence 回收。
- Rust 已新增 R1/R2/N1/N2 `#[ignore]` + 环境变量双确认的真实执行测试入口；默认 `cargo test --lib` 不会执行真实 Codex。

已完成总验收：

- K2 acceptance evidence：`evidence/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-acceptance-v1.md`
- K2 acceptance handoff：`handoffs/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-acceptance-v1-result.md`

R1 已完成：

- Evidence：`evidence/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1.md`
- Handoff：`handoffs/2026-06-10-stage-k-k2-r1-mario-test-resume-read-only-real-execution-v1-result.md`

R2 已完成：

- Evidence：`evidence/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1.md`
- Handoff：`handoffs/2026-06-10-stage-k-k2-r2-mario-test-resume-workspace-write-real-execution-v1-result.md`

N1 已完成：

- Evidence：`evidence/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1.md`
- Handoff：`handoffs/2026-06-10-stage-k-k2-n1-isolated-new-session-read-only-real-execution-v1-result.md`

N2 已完成：

- Evidence：`evidence/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1.md`
- Handoff：`handoffs/2026-06-10-stage-k-k2-n2-isolated-new-session-workspace-write-real-execution-v1-result.md`

后续仍未完成：

- K3 项目工作流真实自动化编排产品化。
- K4 记忆捕获、候选确认和任务记忆注入体验。
- K5 失败、重试、停止、恢复和控制硬化。
- K6 Stage K dogfood / 真实 Tauri / 最终验收。
