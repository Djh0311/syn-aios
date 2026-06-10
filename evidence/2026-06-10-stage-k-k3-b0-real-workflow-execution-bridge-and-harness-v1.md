# Stage K / K3-B0 Real Workflow Execution Bridge And Harness Evidence v1

日期：2026-06-10

状态：主管线 fresh verify 已通过，K3-B0 接受为 `accepted_with_p2`。

主管线结论：

- K3-B0 可接受为 K3-B1 / K3-B2 真实执行前置 bridge / harness 已准备好。
- K3-B0 不接受为 K3-B1 / K3-B2 已执行，不接受为 K3-Level-B 真实执行完成，也不接受为 K3 或 Stage K 完成。
- K3-B0 产品代码路径和验证命令没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。
- 过程偏差：主管线收尾扫描误把 Markdown 反引号放进 shell 双引号，触发了 `codex exec` / `codex exec resume` 命令替换。输出显示 `Reading prompt from stdin... No prompt provided via stdin.`，并且访问 `/Users/yoyi/.codex/state_5.sqlite` 时因 readonly database 初始化失败；这不是 K3-B0 产品代码路径，也不能作为 K3-B1 / K3-B2 执行证据，但本轮不能再严格声称“完全没有执行 Codex 命令 / 完全没有触碰 `.codex`”。
- P2 保留：K3-B2 真实执行前建议加强 allowed file 内容 / marker / hash 断言；当前 ignored real entry 已打印 allowed path hash，并验证外部 manifest 不变和 allowed path 存在，但任务完成标准还应在 B2 执行任务里把 allowed file 内容证明写得更硬。

## 范围

本 evidence 覆盖：

- K3-B 专用 bridge / harness。
- B1 / B2 frozen refs 接入 Product Command Phase B request。
- fake / no-op 默认测试。
- ignored + env-gated 真实执行入口。
- Tauri command wrapper 对 runtime prompt 的二次 guard。

本 evidence 不覆盖：

- K3-B1 真实 `resume` 执行。
- K3-B2 真实 `new_session` 执行。
- 真实 prompt 发送。
- `/Users/yoyi/.codex` 读写。
- 前端 UI 接入。
- K4 / K5 / K6。

## 改动摘要

主要改动文件：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

关键实现：

- `project_workflow_automation.rs` 新增 K3-B B1 / B2 frozen config，覆盖 `execution_point_id`、`project_root`、`workflow_id`、`node_id`、`run_unit_id`、`work_item_id`、`task_memory_packet_ref`、`permission_envelope_ref`、`readback_marker`、prompt ref/hash、sandbox 和 allowed write path。
- `run_project_workflow_automation_k3_b_with_runner` 从 K3 workflow state / frozen run unit 构造 `codex_control`，再走统一 `real_execution_product_command` preview / prepare / user decision / Phase A / Phase B。
- K3-B1 路径使用 `resume` Phase B authorization；K3-B2 路径使用 `new_session` Phase B authorization。
- 默认无 runtime prompt 时，Phase B 进入 `phase_b_blocked`，`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`，`readback_summary.result_count=null`。
- `commands.rs` 的 Tauri wrapper 增加 `ensure_k3_b_tauri_no_real_harness_request`；只要 `runtime_prompt_body` 非空就返回 `k3_b_real_execution_requires_dedicated_level_b_authorization`，避免 UI / invoke 直接触发真实 harness。
- `lib.rs` 增加 Tauri guard 测试，确认非空 runtime prompt 被 wrapper 阻断，空 prompt 可进入 no-real harness。

关键代码参考：

- `commands.rs:269`：`run_project_workflow_automation_k3_b` Tauri wrapper。
- `commands.rs:282`：`ensure_k3_b_tauri_no_real_harness_request`。
- `project_workflow_automation.rs:989`：`run_project_workflow_automation_k3_b_at`。
- `project_workflow_automation.rs:1015`：`run_project_workflow_automation_k3_b_with_runner`。
- `project_workflow_automation.rs:4212`：K3-B1 no-op bridge 测试。
- `project_workflow_automation.rs:4286`：K3-B2 no-op manifest guard 测试。
- `project_workflow_automation.rs:4484`：K3-B1 ignored env-gated real execution entry。
- `project_workflow_automation.rs:4603`：K3-B2 ignored env-gated real execution entry。

## K3-B Bridge 绑定关系

K3-B bridge 绑定路径：

```text
K3 workflow state / frozen developer run unit
-> K3-B frozen execution point config
-> codex_control intent
-> real_execution_product_command preview / prepare / decision
-> Phase A trace
-> Phase B no-op / env-gated real entry
-> runtime log refs / audit refs / readback ref
-> K3 plan run unit output refs
```

K3-B0 明确不把 J2-B / H5 / K2 / legacy 路径作为 K3 完成证据。J2-B 相关代码仍保留为历史 / fixture / compatibility，不作为 K3-B 主路径。

## B1 / B2 No-Op 测试覆盖

B1 no-op 覆盖：

- frozen `execution_point_id`
- `run_unit_id`
- `workflow_id`
- `work_item_id`
- `task_memory_packet_ref`
- `permission_envelope_ref`
- `readback_marker`
- Product Command family
- prompt hash
- empty allowed write roots
- Phase A completed
- Phase B blocked by missing runtime prompt
- all real execution flags false
- `readback_summary.result_count == None`
- read-only core hash manifest requirement

B2 no-op 覆盖：

- frozen `execution_point_id`
- `run_unit_id`
- `workflow_id`
- `work_item_id`
- `task_memory_packet_ref`
- `permission_envelope_ref`
- `readback_marker`
- `operation_id == "new_session"`
- target session is `None`
- allowed write root / allowed write path
- Phase B blocked by missing runtime prompt
- all real execution flags false
- `readback_summary.result_count == None`
- `only_allowed_write_path_may_change` manifest requirement
- `k3_b2_new_session_product_command_phase_b_path_available_env_gated` warning

拒绝场景覆盖：

- wrong prompt hash before sidecars
- non-user confirmation before sidecars
- duplicate Phase A / Product Command guard
- Tauri wrapper rejects non-empty runtime prompt

## Ignored Real Entries

K3-B0 新增或确认的 ignored + env-gated real entries：

- `k3_b1_real_mario_test_workflow_resume_requires_env_authorization`
- `k3_b2_real_isolated_workflow_new_session_requires_env_authorization`

这两个入口默认不会运行。后续 K3-B1 / K3-B2 必须单独任务包、单独授权、单独记录真实副作用。

## 验证结果

主管线 fresh verify：

- `cargo test --lib project_workflow_automation`：通过，15 passed / 4 ignored。
- `cargo test --lib real_execution_command`：通过，36 passed / 7 ignored。
- `cargo test --lib memory_capture`：通过，7 passed。
- `cargo test --lib runtime_log`：通过，6 passed。
- `cargo test --lib worker_protocol`：通过，8 passed。
- `cargo test --lib`：通过，331 passed / 16 ignored；保留既有 warning：`mcp/protocol.rs invalid_params is never used`。
- `cargo fmt -- --check`：通过。

未跑 `npm`：

- K3-B0 未改前端类型、前端 UI 或前端测试。
- 开发线原回交包含前端 wrapper 无接入结论；主管线 fresh scan 也确认普通前端视图没有 K3-B wrapper / button 命中。

## 扫描分类

### 真实 Codex spawn

命令：

```text
rg -n "Command::new\\(\"codex\"\\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner" product-line/prototypes/productized-desktop-shell/src-tauri/src
```

结果：

- 无命中。
- K3-B0 未新增裸 `Command::new("codex")` 产品路径。

### 前端调用入口

命令：

```text
rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun|runProjectWorkflowAutomationK3B|run_project_workflow_automation_k3_b" product-line/prototypes/productized-desktop-shell/src product-line/prototypes/productized-desktop-shell/tests
```

分类：

- 仅 `tauri.ts` 仍有既有 legacy / canvas wrappers。
- 普通 `App.tsx`、`views`、`components` 未命中 K3-B wrapper / button。
- K3-B0 未把真实 harness 接到前端普通入口。

### 敏感材料 / prompt

命令：

```text
rg -n "prompt_body|full transcript|rollout|secret|token|\\.env|provider credential|keychain|OAuth" product-line/prototypes/productized-desktop-shell/src-tauri/src product-line/prototypes/productized-desktop-shell/src
```

分类：

- 大量命中为既有 guard、deny-list、测试、边界文案和运行时类型。
- K3-B0 新增的 prompt body 使用只在 runtime input / ignored real entry 中，且测试断言不写入 product command sidecar、continuation sidecar 或 runtime log。
- Tauri wrapper 默认阻断非空 runtime prompt，避免普通 invoke 直接触发真实执行。

### Readback / memory boundary

命令：

```text
rg -n "result_count.*0|readback_unavailable|readback_failed|readback_timed_out|candidate.*formal|observation.*formal" product-line/prototypes/productized-desktop-shell/src-tauri/src product-line/prototypes/productized-desktop-shell/src
```

分类：

- 命中为既有 readback unknown boundary、UI 状态展示、测试、candidate / observation 非正式记忆边界、正式采纳命令和 memory consistency 检查。
- K3-B0 no-op 测试明确断言 readback unavailable / blocked 时 `result_count=None`。
- 未发现 K3-B0 把 candidate / observation 自动写成 FormalMemory。

## 边界确认

- K3-B0 产品代码路径未执行真实 `codex exec` / `codex exec resume`。
- K3-B0 产品代码路径未发送 prompt。
- K3-B0 产品代码路径未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 未新增前端真实执行按钮。
- 未启动 Tauri、Browser、Chrome、Vite preview 或截图工具。
- 未改 workflow state 顶层结构。
- 未自动写 FormalMemory。
- 主管线收尾扫描发生一次 shell 反引号过程偏差，误触发 Codex CLI 初始化；输出显示无 stdin prompt，`.codex` state db 写入失败，不作为产品路径执行证据。

## P2 后续修补建议

K3-B2 真实执行前建议补强：

- allowed write path 内容必须包含 frozen marker。
- allowed write path hash 必须进入 evidence / handoff。
- B2 manifest proof 除“外部 manifest 不变 + allowed path exists”外，应明确记录 allowed file before / after 状态和内容摘要。
- `approved_for_h3_b` 命名属于历史兼容债；不阻断 K3-B0，但后续可在 Product Command new-session Phase B 做语义命名清理。

## 下一步

下一步可以准备 K3-B1，但只能作为单独真实执行任务包：

- 指定 `/Users/yoyi/Documents/mario test`。
- 指定 session `019e798a-ac37-7771-b982-e38084fcd22e`。
- 指定 read-only。
- 指定 marker `K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10`。
- 执行前必须再次确认会写入 `/Users/yoyi/.codex`，并记录项目核心文件 hash。
