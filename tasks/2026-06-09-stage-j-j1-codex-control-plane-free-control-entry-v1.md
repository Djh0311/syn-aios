# Stage J / J1 Codex Control Plane Free Control Entry v1

日期：2026-06-09

状态：J1-A 已完成并通过长期只读复核线复核，结论为 `accepted_with_deferred_items`；后续 J1-B 已作为独立真实执行点完成并收口为 `accepted_with_deferred_items`。

全局主管任务。本文是 Stage J 的 J1 任务包，用于把 `codex-local` 从“只读边界 / 历史探针 / H5 派生入口”推进到工作台内可用的自由操控入口：用户选择项目、选择 session、输入任务、生成 Product Command preview、查看权限和记忆影响、确认后进入受控执行链路。J1 必须继续服从 J0，不授权裸 CLI / 裸控制台，不绕过统一 Product Command，不自动写正式记忆。

## 0. 先说薄弱点

- 现有统一 Product Command 已收口到 PCR10，但普通 UI 仍没有完整的“选择项目 / session / 输入任务 / 预览 / 确认 / 执行 / 读回”产品入口。
- 现有 `PreviewRealExecutionProductCommandInput` / `PrepareRealExecutionProductCommandInput` 主要从 H5 project workflow dispatch preview 派生；这不足以表达用户自由输入任务。
- 如果 J1 只是把 H5 旧入口换个名字，会继续带着历史 workflow dispatch 语义，不能交付“自由操控 Codex”。
- 如果 J1 直接调 `codex exec` / `codex exec resume`，会绕过 PCR0-PCR10、J0、权限弹层、runtime log、audit、readback 和记忆捕获边界。

## 1. 权威依据

必须服从：

- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

## 2. J1 目标

J1 要交付：

1. 工作台内 `codex-local` 自由操控入口。
2. 用户可选择项目、目标 session、运行模式和 sandbox。
3. 用户可输入任务正文；普通 sidecar / runtime log / audit / memory 只保存 `prompt_summary`、`prompt_ref`、`prompt_hash`，不保存 prompt body。
4. 后端支持 `codex_control` source，不再只依赖 H5 dispatch preview 派生命令。
5. preview / prepare / decision / Phase A / Phase B 继续走统一 Product Command。
6. UI 展示权限、写入范围、readback 预期、runtime/audit 影响和记忆捕获预期。
7. Level A 完成非真实产品入口和 Phase A 接线。
8. Level B 在明确执行点授权后，对指定项目 / 指定 session 做一次真实 `resume`。

## 3. J1 非目标

J1 不做：

- 不做 J2 项目主管自动拆任务 / 多 run unit 自动编排。
- 不做 J3 memory capture bus 完整落地；J1 只预留 capture summary 和 refs。
- 不做 `new_session` 真实执行成功验收；`new_session` 在 J1 只做到 readiness / preview / blocked-or-deferred。
- 不做 planned adapters 真实接入。
- 不做 provider credential store / model verification。
- 不做自动 retry / stop / restart。
- 不做真实 Tauri 全量验收；J1 可做浏览器/离线辅助，真实 Tauri 关键路径归 J5。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential、rollout 或 prompt body 持久化。

## 4. 分阶段执行

### J1-A：产品入口、后端契约和非真实链路

J1-A 允许改产品代码，但不允许执行真实 Codex，不允许发送 prompt，不允许读写 `/Users/yoyi/.codex`。

必须完成：

- 后端新增或扩展 Product Command 输入 source：`codex_control`。
- 新增 `CodexControlCommand` 或等价 input，最小字段：
  - `project_id`
  - `project_root`
  - `workflow_id`
  - `node_id`
  - `work_item_id`
  - `task_package_ref`
  - `memory_packet_ref`
  - `adapter_id`
  - `session_mode`
  - `target_session_id`
  - `sandbox`
  - `prompt_summary`
  - `prompt_ref`
  - `prompt_hash`
  - `allowed_write_roots`
  - `denied_paths`
  - `readback_plan`
  - `timeout_ms`
  - `requested_by`
- `preview_real_execution_product_command` 和 `prepare_real_execution_product_command` 支持 `source_kind="codex_control"`。
- 生成的 Product Command request 必须继续使用统一 `command_family="real_execution_product_command"`，不得另起新的 Product Command family 分叉；`codex_control` 必须体现在 `source_kind="codex_control"`、`operation_id` 和 preview / warning / duplicate scope 等可追溯字段中。
- `operation_id` 当前只允许 `resume` 可进入 J1-A prepare / confirm / Phase A；`new_session` 只生成 readiness / blocked / deferred preview，不写可执行 Product Command，不冒领真实成功。
- `prepare` 可写 product command sidecar，但不得写 continuation / runtime log / attempt。
- `confirm` 只写用户 decision / audit ref，不调用 runner。
- `runRealExecutionProductCommandPhaseA` 可创建 continuation / no-real attempt / runtime log ref / readback unavailable，且 flags 必须保持：
  - `prompt_sent=false`
  - `real_codex_executed=false`
  - `writes_codex_home=false`
  - `writes_project_files=false`
- UI 新增普通用户可见入口，位置优先在 `智能体` 页面主区域；可从 `项目` 页面链接进入，但不能把开发者 raw 状态铺到首屏。
- UI 不显示裸 CLI、完整 prompt hash 细节、sidecar path、internal id；这些只能进详情或设置 / 开发者。
- 秘书只解释下一步和风险，不直接替用户批准真实执行。

### J1-B：真实 resume 执行点授权和一次受控真实运行

J1-B 只有在 J1-A 通过测试、只读复核线通过、并且用户 / 全局主管明确批准执行点后才允许执行。

J1-B 执行点默认建议：

- 项目：`/Users/yoyi/Documents/mario test`
- adapter：`codex-local`
- operation：`resume`
- session：必须由 J1-B 任务包或执行记录再次列明，不能默认继承旧 E/H/PCR session。
- sandbox：优先 `read-only`；如需写入，必须另列唯一写入文件。
- allowed write roots：只作为 sandbox / runner 授权解释；`read-only` 下不得解释为项目写授权。
- denied paths：必须包含 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- prompt body：只作为 runtime stdin 发送；不写入 product command sidecar / continuation sidecar / runtime log / audit / memory。
- prompt summary/ref/hash：必须在执行前冻结。
- readback：只读目标 session / 本次 attempt 的必要摘要；不得读取完整 transcript。

J1-B 成功必须满足：

- 真实执行来自 `run_real_execution_product_command_phase_b` / `run_real_execution_product_command_phase_b_at` 或其受控 wrapper。
- `prompt_sent=true`
- `real_codex_executed=true`
- `writes_codex_home=true`
- `writes_project_files` 与 sandbox / 授权一致。
- readback marker 或结果摘要成功，`result_count` 正确；失败 / unavailable 必须保留 `null`，不能伪装为 0。
- runtime log、audit、product command attempt、continuation attempt 可追溯。
- 项目核心文件 hash 前后一致，除非 J1-B 明确授权唯一写入文件。

## 5. UI 显示边界

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作。

J1 普通 UI 应显示：

- 项目选择。
- session 选择。
- 运行模式：`resume` 可用；`new_session` 预览 / 暂缓。
- sandbox / 写入范围摘要。
- prompt summary 输入和任务正文输入。
- prompt body 保存策略说明。
- readback 预期。
- 记忆影响预览：会产生 observation / candidate 候选来源，但不会自动正式化。
- 操作按钮状态：预览、准备、确认、Phase A；Phase B 只有执行点授权后才出现或可用。

J1 普通 UI 禁止显示：

- 裸 CLI 命令。
- raw enum 长串。
- sidecar 绝对路径。
- internal id 列表。
- 完整 transcript。
- secret / credential。
- planned adapter 可执行按钮。
- “已记住”“已正式记忆”之类误导文案。

## 6. 后端改动范围

允许改：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 必要时新增小型 Rust 模块，但优先复用 `real_execution_command.rs`。

默认不改：

- 旧 legacy dispatch runner 真实路径。
- `workflow-state.v0.json` 顶层结构。
- `session-continuations.v1.json` schema，除非只追加兼容字段且有迁移/默认值。
- runtime log schema，除非只追加兼容 ref/summary。
- FormalMemory schema。

## 7. 前端改动范围

允许改：

- `src/lib/types.ts`
- `src/lib/tauri.ts`
- `src/views/AgentView.tsx`
- 必要时 `src/views/ProjectsView.tsx` 添加轻量入口或链接。
- `src/lib/secretaryReadModel.ts`
- `tests/offline-permission-dialog.test.tsx`
- `src/styles.css` 中 J1 面板必要样式。

默认不改：

- 导航主入口结构。
- 设置 / 开发者归档规则。
- 手机端 / mobile responsive 规则。
- 真实 Tauri 截图验收逻辑。

## 8. 测试矩阵

J1-A 必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib real_execution_command`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib codex_local_runner`
- `cargo test --lib`
- `cargo fmt -- --check`

J1-A 必须新增 / 覆盖测试：

- `codex_control` source preview / prepare 不依赖 H5 dispatch preview。
- `new_session` 在 J1-A 不真实执行。
- 无用户确认不允许 Phase B。
- 非 `confirmed_by: "user"` 的高影响确认被阻断。
- prompt body 不进入 sidecar / runtime log / memory。
- Phase A flags 全 false：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- UI 不出现裸 CLI / planned adapter 可执行 / 自动正式记忆文案。
- J1-A 普通 UI / `AgentView` / `App` / `components` 不得调用或暴露 `runRealExecutionProductCommandPhaseB` / `run_real_execution_product_command_phase_b`。
- J1-A 普通 UI 不得复用 `executeLegacyWorkflowNodeDispatch` / `runLegacyWorkflowMachine` / H5 dispatch wrapper 作为自由操控入口；`h5_project_workflow_dispatch` 只能保留为历史 / workflow dispatch source。

J1-B 真实执行前必须追加：

- 执行点授权任务包或 J1 任务包内执行点章节。
- 项目文件 hash baseline。
- prompt summary/ref/hash。
- target session 确认。
- `.codex` 最小读写范围说明。
- rollback / stop / timeout 策略。

## 9. 分线职责

### 主管线

- 维护本任务包。
- 派发只读复核线审查 J1。
- 决定 J1-A 是否可开发。
- 决定 J1-B 是否进入真实执行点授权。
- 控制 checkpoint 入口同步。

### 后端线

- 实现 `codex_control` source。
- 保持所有执行归口 Product Command。
- 补 Rust 测试。
- 不执行真实 Codex，除非 J1-B 获授权。

### UI 线

- 实现智能体页自由操控入口。
- 保持普通用户信息层级。
- 补离线交互测试。
- 不把开发者 raw 状态放回普通首屏。

### 复核线

- 只读检查任务包和后续代码。
- 查是否绕过 Product Command。
- 查是否持久化 prompt body / transcript / secret。
- 查是否冒领 `new_session`、planned adapters、自动 retry 或正式记忆。

## 10. 只读复核要求

复核线必须检查：

- J1 是否足够支撑开发。
- J1 是否明确新增 `codex_control` source，而不是复用 H5 旧入口冒充自由操控。
- J1-A / J1-B 边界是否清楚。
- 是否仍服从 J0、PCR10 和 Product Command Routing。
- 真实执行、`.codex`、prompt body、transcript、secret、rollout 边界是否清楚。
- UI 是否符合桌面 Tauri、普通用户 / 详情 / 开发者分层。
- 是否可以开始 J1-A 开发。

复核线不得改文件、不得执行真实 Codex、不得读写 `/Users/yoyi/.codex`、不得启动 GUI / Tauri / Browser。

## 11. 验收和不得声明

J1-A 完成后可接受为：

- Codex Control Plane 自由操控入口的非真实产品链路完成。
- `codex_control` source 已支持 preview / prepare / decision / Phase A。
- 普通 UI 可以完成预览、准备、确认和 Phase A 非真实链路。
- prompt body 未被持久化。

J1-B 完成后可接受为：

- 指定项目 / 指定 session / 指定授权范围内的一次真实 `codex-local resume` 产品路径完成。

J1 完成后仍不得声明：

- Stage J 已完成。
- J2 自动编排完成。
- J3 记忆捕获总线完成。
- `new_session` 真实成功。
- 任意项目自由执行。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 完成。
- 真实 Tauri 全量验收完成。

## 12. 回交产物

J1 任务包：

- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`

J1-A 已使用本任务包主文件名回交：

- `evidence/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `handoffs/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1-result.md`

J1-B 如获授权并执行，应新增：

- `evidence/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-level-b-real-resume-v1.md`
- `handoffs/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-level-b-real-resume-v1-result.md`
