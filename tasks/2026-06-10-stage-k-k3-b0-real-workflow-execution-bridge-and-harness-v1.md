# Stage K / K3-B0 Real Workflow Execution Bridge And Harness v1

日期：2026-06-10

状态：已完成，结论为 `accepted_with_p2`。本文是 K3-Level-B 正式字段冻结后的前置开发任务，不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。本文目标是补齐 K3-B1 / K3-B2 真实执行前必须存在的 K3 专用 bridge / harness，避免直接复用 J2-B、K2、H5 或 legacy 路径。

前置：

- K2.5 architecture calibration 已完成，结论 `accepted`。
- K3-Level-A 已完成，结论 `accepted`。
- K3-Level-B 字段冻结已完成，结论 `accepted_with_pre_execution_blocker`：`2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md`。

## 1. 目标

补齐 K3-B 专用真实执行前置路径：

```text
K3 run unit
-> K3-B bridge / harness
-> Product Command Phase B request builder
-> permission envelope / duplicate guard / diagnostics / memory packet check
-> fake/no-op validation path
-> ignored + env-gated real execution entry for later B1/B2
```

本任务只接受为“真实执行 bridge / harness 已准备好”。不接受为 B1/B2 已执行，不接受为 K3-Level-B 完成。

## 2. 必须实现

- 新增或收敛 K3-B 专用 bridge，来源必须是 K3 run unit / workflow automation read model。
- B1 / B2 必须使用 K3 字段冻结里的 `execution_point_id`、`run_unit_id`、`workflow_id`、`work_item_id`、`task_memory_packet_ref`、`permission_envelope_ref`、`readback_marker`。
- 生成 Product Command Phase B request 时必须带上 K3 refs，不能使用 J2-B hard-coded path 作为 K3 事实源。
- 增加 fake/no-op 测试，证明 K3-B bridge 能构造 B1/B2 请求、执行 gate、duplicate guard、permission envelope、hash / manifest input 和 readback plan，但不调用真实 runner。
- 增加 ignored + env-gated 真实执行测试入口，供后续 B1 / B2 任务单独授权时使用；默认测试不得运行真实 Codex。
- B2 `new_session` 若当前路径无法稳定回链新 session / readback / run unit refs，必须在代码或 handoff 中明确阻断，不得降级伪造成功。
- workspace-write 证明必须依赖执行前后 manifest / hash diff，不得只靠 `sandbox=workspace-write` 推断 `writes_project_files=true`。
- readback failed / unavailable / timed_out 必须保持 `result_count=null`。
- worker report / process fact / capture source 可回链，但不得自动写 FormalMemory。

## 3. 禁止

- 禁止执行真实 `codex exec` / `codex exec resume`。
- 禁止发送 prompt。
- 禁止读写 `/Users/yoyi/.codex`。
- 禁止读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 禁止新增裸 `Command::new("codex")` 产品路径。
- 禁止前端拼 CLI 或绕过 Product Command。
- 禁止复用历史 K2 / H5 / J2-B 授权、prompt、marker、execution point id 或完成证据。
- 禁止把 fake/no-op 结果显示或记录成真实执行成功。

## 4. 建议落点

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

前端默认不新增 UI。若必须同步类型：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

## 5. 验收

必须通过：

```text
cargo test --lib project_workflow_automation
cargo test --lib real_execution_command
cargo test --lib memory_capture
cargo test --lib runtime_log
cargo test --lib worker_protocol
cargo test --lib
cargo fmt -- --check
```

如改前端类型或测试，追加：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

必须扫描：

```text
rg -n "Command::new\\(\"codex\"\\)|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner" prototypes/productized-desktop-shell/src-tauri/src
rg -n "executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|canvasStartRun|canvasTickRun" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
rg -n "prompt_body|full transcript|rollout|secret|token|\\.env|provider credential|keychain|OAuth" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
rg -n "result_count.*0|readback_unavailable|readback_failed|readback_timed_out|candidate.*formal|observation.*formal" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

## 6. 回交要求

开发线回交必须说明：

- 改了哪些文件。
- K3-B bridge / harness 如何绑定 K3 run unit 和 Product Command。
- B1 / B2 fake/no-op 测试覆盖哪些字段。
- ignored + env-gated 真实执行入口名称。
- 是否发现 B2 `new_session` 不能稳定支撑，如不能，给出阻断原因。
- 验证命令结果。
- 是否触碰真实 Codex、prompt、`/Users/yoyi/.codex` 或敏感材料。
- 不可声称事项。

## 7. 回收结论

主管线收口：

- K3-B0 接受为 `accepted_with_p2`。
- K3-B 专用 bridge / harness 已准备好。
- B1 / B2 frozen refs 已绑定 K3 run unit 和统一 Product Command Phase B。
- B1 / B2 fake/no-op 测试、hash / manifest guard、permission envelope、duplicate guard、readback `result_count=null` 和 ignored + env-gated real entries 已覆盖。
- Tauri wrapper 已阻断非空 `runtime_prompt_body`，普通 invoke 不能直接触发真实 K3-B harness。

验证记录：

- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored。
- `cargo test --lib real_execution_command`：36 passed / 7 ignored。
- `cargo test --lib memory_capture`：7 passed。
- `cargo test --lib runtime_log`：6 passed。
- `cargo test --lib worker_protocol`：8 passed。
- `cargo test --lib`：331 passed / 16 ignored。
- `cargo fmt -- --check`：通过。

记录：

- `evidence/2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1.md`
- `handoffs/2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1-result.md`

P2：

- K3-B2 真实执行前建议加强 allowed write path 内容 / marker / hash 断言；当前 ignored real entry 已打印 allowed path hash，并验证外部 manifest 不变和 allowed path 存在，但 B2 执行任务包应把 allowed file 内容证明写入验收。

不可声称：

- 不接受为 K3-B1 / K3-B2 已执行。
- 不接受为 K3-Level-B 真实执行完成。
- 不接受为 K3 或 Stage K 完成。
- K3-B0 产品代码路径没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`。
- 主管线收尾扫描发生一次 shell 反引号过程偏差，误触发 `codex exec` / `codex exec resume` 命令替换；输出显示无 stdin prompt，`.codex` state db 写入失败，不作为产品路径执行证据。
