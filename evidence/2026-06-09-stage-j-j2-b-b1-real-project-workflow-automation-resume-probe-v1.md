# Stage J / J2-B B1 Real Project Workflow Automation Resume Probe Evidence v1

日期：2026-06-09

状态：B1 read-only 真实 `resume` 探针已通过，并经长期只读复核线复核后由主管线收口为 `accepted_with_deferred_items`；J2-B 尚未整体完成。

## 本轮目标

在 J2-B execution point freeze 和 B1 execution bridge 均通过复核后，对指定 `mario test` / 指定开发线 session 执行一次真实 `codex-local resume`。本轮必须证明真实执行是由 J2 run unit 绑定的统一 Product Command Phase B 触发，不使用 H5 / legacy / direct CLI / MCP canvas run 冒充。

## 冻结对象

- Project root：`/Users/yoyi/Documents/mario test`
- Project id：`project:users-yoyi-documents-mario-test`
- Workflow id：`workflow:users-yoyi-documents-mario-test:default`
- Node id：`workflow:users-yoyi-documents-mario-test:default:node:codex-dev`
- Run unit：`developer_execution`
- Adapter：`codex-local`
- Operation：`resume`
- Target session：`019e798a-ac37-7771-b982-e38084fcd22e`
- Sandbox：`read-only`
- Allowed write roots：`[]`
- Prompt ref：`workbench-managed:j2-b:mario-test:developer-run-unit:read-only:v1`
- Prompt hash：`31c8ceb071804168e46a1d5b3d3accbded1539037472479649766d676672caa0`
- Readback marker：`J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09`

## 实现补充

为避免真实执行只能靠临时命令触发，本轮在 `project_workflow_automation.rs` 新增了一个默认 ignored / env-gated 的真实执行 harness：

- `j2_b_b1_real_mario_test_project_workflow_resume_probe_requires_env_authorization`
- 默认 `cargo test --lib` 不会触发真实 Codex。
- 只有显式设置 `J2_B_B1_REAL_EXECUTION_AUTHORIZED=1` 等 env 后才运行。
- harness 直接调用 `run_project_workflow_automation_j2_b_b1_at`，仍走 J2-B bridge 和统一 Product Command Phase B。

## 执行结果

执行命令：

```text
env J2_B_B1_REAL_EXECUTION_AUTHORIZED=1 J2_B_B1_PROJECT_ROOT=/Users/yoyi/Documents/mario\ test J2_B_B1_SESSION_ID=019e798a-ac37-7771-b982-e38084fcd22e J2_B_B1_EXPECTED_MARKER=J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09 J2_B_B1_WORKFLOW_STATE_PARENT=/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs cargo test --lib project_workflow_automation::tests::j2_b_b1_real_mario_test_project_workflow_resume_probe_requires_env_authorization -- --ignored --nocapture
```

结果：`1 passed; 0 failed`。

本命令经用户授权和主管线确认执行，会让 Codex CLI 写入 `/Users/yoyi/.codex` 的原生状态。本轮没有读取 `/Users/yoyi/.codex`，也没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。

## 运行产物

Run root：

```text
/Users/yoyi/workspace/product-line/tmp/j2-b-real-workflow-automation/runs/j2-b-b1-run-1781005213078398000
```

产物路径：

- `workflow-state.v0.json`
- `real-execution-product-commands.v1.json`
- `session-continuations.v1.json`
- `runtime-logs.v1.json`
- `j2-b-b1-last-message-2026-06-09t11-00-03z.json`

关键 id：

- Product command id：`real-exec-command:codex-control:c920e54866d8`
- Product attempt id：`real-exec-command-attempt:phase-b:real-exec-command:codex-control:c920e54866d8:4`
- Continuation id：`session-continuation:v1:c71d08bdb8a3f1b3f363052d5f512548b001166b18ccacf99b6005b0359f92aa`
- Continuation attempt id：`session-continuation-attempt:h2-phase-b:2026-06-09T11:00:03Z:2d8aed8735ec9c97`
- Runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h2-phase-b:2026-06-09T11:00:03Z:2d8aed8735ec9c97`

## 关键断言

- `real-execution-product-commands.v1.json` 中 `command_family = real_execution_product_command`。
- source 是 `codex_control`，不是 H5 / legacy / direct CLI。
- `allowed_write_roots = []`，`sandbox = read-only`。
- Phase B attempt status：`phase_b_real_resume_executed`。
- Phase B flags：
  - `runner_call_allowed = true`
  - `prompt_sent = true`
  - `real_codex_executed = true`
  - `writes_codex_home = true`
  - `writes_project_files = false`
- Readback：`status = succeeded`，`result_count = 1`。
- Last message 含 marker：`J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09`。
- Prompt body 未持久化到 product command sidecar / continuation sidecar / runtime log。

## Baseline Hash

执行前后 `/Users/yoyi/Documents/mario test` 四个核心文件 hash 一致：

- `index.html`：`f08427fd9510ff2037742c4062052c0b38a3f05bff876d17b3228ff49ce6f7bf`
- `styles.css`：`6864e759c7cf53698a6225c121d53b1e14dc06a4d70930efb546822fafe3ad1f`
- `game.js`：`814d4fc9e6ab7585c48c55b702f290d899aba05444607fd8f8c38843f47bc8cd`
- `README.md`：`02792d82ffd3b9cfc435da4389f0d064d2134a0b76a3f43056f0e23f0af573b5`

## Post-Run 扫描分类

- Marker 只在 workbench-managed last-message 和产品记录摘要中命中。
- `You are the codex-local developer run unit` 未在 run artifacts 中命中；prompt body 未被 sidecar / runtime log 持久化。
- `full transcript material` 命中是 denied path label，不是读取完整 transcript 的证据。
- `Command::new("codex")` 未由本轮新增；真实进程调用来自既有 `codex_local_runner` 并通过统一 Product Command gate。
- `run_project_workflow_automation_j2_b_b1` 在前端 `src` / tests 无普通 UI wrapper 或按钮调用；仅 Rust command / bridge / tests 命中。

## 验证矩阵

- B1 ignored real harness：通过，`1 passed`。
- `cargo fmt -- --check`：通过。
- `cargo test --lib project_workflow_automation`：8 passed / 1 ignored。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo test --lib runtime_log`：6 passed。
- `cargo test --lib codex_local_runner`：11 passed。
- `cargo test --lib`：310 passed / 9 ignored。
- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：13 passed。
- `npm run build`：通过；仅保留既有 Vite chunk size warning。

## 边界确认

- 本轮确实执行了真实 `codex-local resume`。
- 本轮确实发送了 B1 canonical prompt。
- 本轮确实允许 Codex CLI 写入 `/Users/yoyi/.codex` 原生状态。
- 本轮没有授权或检测到项目文件写入。
- 本轮没有读取 `/Users/yoyi/.codex`、secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 本轮没有接 UI 执行按钮。

## 主管复核

长期只读复核线结论为带 P2 通过，无 P0/P1。主管线接受 B1 为 `accepted_with_deferred_items`。

记录见：

- `evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`

## 不能声明

- 不能声明 J2-B 全部完成：B2 workspace-write 真实探针仍未执行。
- 不能声明 J3 记忆捕获总线完成。
- 不能声明 worker report candidate 已完整进入 C5 / observation 回收闭环。
- 不能声明任意项目自由执行完成。
- 不能声明 planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 完成。
- 不能声明 Stage J 完成。
