# Stage J / J2-B B2 Real Isolated Project Workflow New-Session Write Probe Evidence v1

日期：2026-06-09

状态：B2 workspace-write 真实 `new_session` 探针已执行成功，并经长期只读复核线复核后由主管线收口为 `accepted_with_deferred_items`。J3 记忆捕获总线尚未完成，Stage J 尚未完成。

## 本轮目标

在 J2-B execution point freeze、B1 read-only 真实 `resume` 探针、B2 execution bridge Level A 均完成后，对 Stage J 隔离测试项目执行一次受控 workspace-write 真实 `codex-local new_session` 探针。

本轮必须证明：

- 真实执行由 J2 run unit 绑定的统一 Product Command Phase B 触发。
- 入口不是 H5 / legacy / direct CLI / MCP canvas run。
- Codex 只写 allowed project write path。
- baseline 文件 hash 保持不变。
- runtime log / audit / readback refs 可追溯。
- prompt body 不持久化到 product sidecar、continuation sidecar、runtime log 或 workflow state。

## 冻结对象

- Project root：`/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project`
- Project id：`project:stage-j-j2-b-isolated-project`
- Workflow id：`workflow:stage-j-j2-b-isolated-project:default`
- Node id：`workflow:stage-j-j2-b-isolated-project:default:node:codex-dev`
- Run unit：`developer_execution`
- Adapter：`codex-local`
- Operation：`new_session`
- Sandbox：`workspace-write`
- Allowed write root：`/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b`
- Allowed write path：`/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`
- Prompt ref：`workbench-managed:j2-b:isolated-project:developer-run-unit:workspace-write:v1`
- Prompt hash：`a1e3eb2285a75b30d0104f5bd032e3b4fdfc51111ff52949597ce78de5878bb0`
- Readback marker：`J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09`

## 执行方式

B2 真实探针通过 ignored / env-gated Rust harness 触发，非 direct CLI：

```text
J2_B_B2_REAL_EXECUTION_AUTHORIZED=1 \
J2_B_B2_PROJECT_ROOT=/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project \
J2_B_B2_ALLOWED_WRITE_PATH=/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md \
J2_B_B2_EXPECTED_MARKER=J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09 \
J2_B_B2_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs \
cargo test --lib project_workflow_automation::tests::j2_b_b2_real_isolated_project_workflow_new_session_probe_requires_env_authorization -- --ignored --nocapture
```

结果：`1 passed; 0 failed`。

本命令在用户授权范围内执行真实 Codex，会让 Codex CLI 写入 `/Users/yoyi/.codex` 原生状态。本轮记录只引用工作台自有 run artifacts；未读取 `/Users/yoyi/.codex`，未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

## 运行产物

Run root：

```text
/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000
```

产物路径：

- `workflow-state.v0.json`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- `runtime-logs.v1.json`
- `j2-b-b2-last-message-2026-06-09t12-00-03z.json`

关键 id：

- Product command id：`real-exec-command:codex-control:b7c65a1f0ee6`
- Product attempt id：`real-exec-command-attempt:phase-b-new-session:real-exec-command:codex-control:b7c65a1f0ee6:4`
- Continuation id：`session-continuation:v1:ab246dba122a4afe7f76475af041a54393737f77151abdfecbc4ea5be43d5f2c`
- Continuation attempt id：`session-continuation-attempt:h3-b:2026-06-09T12:00:03Z:2c5e1500737685e9`
- Runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h3-b:2026-06-09T12:00:03Z:2c5e1500737685e9`

## 关键断言

- `real-execution-product-commands.v1.json` 中 `command_family = real_execution_product_command`。
- Product command source 是 `codex_control`，不是 H5 / legacy / direct CLI。
- `operation_id = new_session`，`sandbox = workspace-write`。
- `allowed_write_roots = ["/Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b"]`。
- Phase B attempt status：`phase_b_real_new_session_executed`。
- Phase B flags：
  - `runner_call_allowed = true`
  - `prompt_sent = true`
  - `real_codex_executed = true`
  - `writes_codex_home = true`
  - `writes_project_files = true`
- Readback：`status = succeeded`，`result_count = 1`。
- Last message 含 marker：`J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09`。
- Runtime log 中 dispatch / readback / workflow_run 三类 entry 均为 `succeeded`，且 redacted summary 不持久化 prompt body 或 raw runner output。
- Workflow audit event `project_workflow_automation_j2_b_b2_phase_b_recorded` 回链 run unit、workflow、node、Product Command、Phase A / Phase B attempt、runtime log ref、task package ref 和 memory packet ref。

## Baseline Hash 和文件范围

执行后重新计算：

- `README.md`：`b21eda72c5261bb74eb8f6f8a5fed04036c7e2571cd13bb72353c9471208e908`
- `project-notes.md`：`c6c8fb4c0e688663a87b8cedf519ef5dc3ce7c3f3455f2add94a1f2642ca7c4d`
- Allowed write file：`4483182dbfd619331105b86b0ee165c227ce51449bf2ceb8f588da8e5bff1e8e`
- Last message file：`8adbfbb0a7e33237aa448f2c8e81f4bb84af0277ddead5287ea96ca3b7a4704e`

执行后隔离项目文件 manifest 仅包含：

- `README.md`
- `project-notes.md`
- `.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`

允许写入文件内容：

```text
J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09

worker_report_candidate:
- allowed_file: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md
```

## Prompt Body 持久化扫描

扫描命令覆盖 run root 和隔离项目，并使用 `--hidden --no-ignore` 避免漏扫 `.workbench`：

```text
rg --hidden --no-ignore -n "You are the codex-local developer run unit for Stage J / J2-B project workflow automation workspace-write closed-loop probe|Only create or update the allowed write path|Reply only with the marker and a minimal structured worker report candidate listing the allowed file|J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09" tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000 tmp/stage-j-j2-b-isolated-project
```

分类：

- Canonical prompt body 的正文句子未命中。
- Marker 命中在 readback plan 摘要、allowed write file 和 workbench-managed last message 中，属于本轮预期结果。
- Prompt body 未持久化到 product command sidecar / continuation sidecar / runtime log / workflow state。

## Post-Run 验证矩阵

真实探针后已完成：

- B2 ignored real harness：通过，`1 passed`。
- `cargo test --lib project_workflow_automation`：11 passed / 2 ignored。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo test --lib runtime_log`：6 passed。

B2 Level A 前置已完成并在对应 evidence 中记录：

- `cargo fmt -- --check`：通过。
- `cargo test --lib`：313 passed / 10 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过；仅保留既有 Vite chunk size warning。

## 边界确认

- 本轮确实执行了真实 `codex-local new_session`。
- 本轮确实发送了 B2 canonical prompt。
- 本轮确实允许 Codex CLI 写入 `/Users/yoyi/.codex` 原生状态。
- 本轮确实写入了隔离项目 allowed write path。
- 本轮没有修改隔离项目 baseline 文件。
- 本轮没有读取 `/Users/yoyi/.codex`、secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 本轮没有接普通 UI 执行按钮。

## 已知噪声

runner stderr summary 包含 remote plugin auth / MCP process group termination 警告。这类内容由 Codex CLI 真实进程输出，被产品路径截断为 summary warning；当前不影响探针通过，但后续 J4/J5 应继续把它作为 P2 runner 噪声分类，不把它展示成用户主线证据。

## 主管复核

长期只读复核线结论为带 P2 通过，无 P0/P1。主管线接受 B2 为 `accepted_with_deferred_items`。

主管收口记录见：

- `evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`

## 不能声明

- 不能声明 J2-B 已完整完成 C5 / observation / candidate 回收闭环。
- 不能声明 J3 记忆捕获总线完成。
- 不能声明 worker report candidate 已完整进入 C5 / observation / candidate 回收闭环。
- 不能声明任意项目无限制自由执行完成。
- 不能声明 planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 完成。
- 不能声明 Stage J 完成。

## 下一步

1. 进入 J3 memory capture bus。
2. J3 必须消费本轮 B2 的 runtime / audit / readback / worker report refs，生成 observation / candidate。
3. J3 不得直接写 FormalMemory；正式记忆仍走既有确认链路。
