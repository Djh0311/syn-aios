# Stage J / J2-B B2 Real Isolated Project Workflow New-Session Write Probe Result v1

日期：2026-06-09

状态：B2 workspace-write 真实 `new_session` 探针已通过，并经长期只读复核线复核后由主管线收口为 `accepted_with_deferred_items`。J3 尚未完成，Stage J 尚未完成。

## 结果摘要

- 指定 Stage J 隔离测试项目的 J2 developer run unit workspace-write 真实 `new_session` 已通过。
- 入口是 `run_project_workflow_automation_j2_b_b2_at` 的 env-gated real harness，非 H5 / legacy / direct CLI / MCP canvas run。
- 产品链路是 `J2 run unit -> codex_control -> real_execution_product_command -> Phase A -> new-session Phase B`。
- Readback marker 已读回：`J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09`。
- Codex 只写 allowed write path：`.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`。
- Baseline `README.md` 和 `project-notes.md` hash 保持冻结值。

## 关键产物

- Evidence：`evidence/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1.md`
- Run root：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000`
- Product sidecar：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/real-execution-product-commands.v1.json`
- Continuation sidecar：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/session-continuations.v1.json`
- Runtime log：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/runtime-logs.v1.json`
- Workflow state：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/workflow-state.v0.json`
- Last message：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/j2-b-b2-last-message-2026-06-09t12-00-03z.json`
- Allowed write file：`tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`

## 关键 id

- Product command id：`real-exec-command:codex-control:b7c65a1f0ee6`
- Product attempt id：`real-exec-command-attempt:phase-b-new-session:real-exec-command:codex-control:b7c65a1f0ee6:4`
- Continuation id：`session-continuation:v1:ab246dba122a4afe7f76475af041a54393737f77151abdfecbc4ea5be43d5f2c`
- Continuation attempt id：`session-continuation-attempt:h3-b:2026-06-09T12:00:03Z:2c5e1500737685e9`
- Runtime log ref：`runtime-log:dispatch-attempt:session-continuation-attempt:h3-b:2026-06-09T12:00:03Z:2c5e1500737685e9`

## 验证

- B2 ignored real harness：`1 passed`。
- `cargo test --lib project_workflow_automation`：11 passed / 2 ignored。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo test --lib runtime_log`：6 passed。
- 文件 hash 复核：
  - `README.md`：`b21eda72c5261bb74eb8f6f8a5fed04036c7e2571cd13bb72353c9471208e908`
  - `project-notes.md`：`c6c8fb4c0e688663a87b8cedf519ef5dc3ce7c3f3455f2add94a1f2642ca7c4d`
  - allowed write file：`4483182dbfd619331105b86b0ee165c227ce51449bf2ceb8f588da8e5bff1e8e`
- `rg --hidden --no-ignore` 扫描确认 canonical prompt body 正文未持久化；marker 只在预期摘要、allowed file 和 last message 中出现。

## 边界

- 本轮真实执行会写 `/Users/yoyi/.codex`，这是 B2 授权范围内的 Codex CLI 原生状态写入。
- 本轮没有读取 `/Users/yoyi/.codex`。
- 本轮没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 本轮没有修改隔离项目 baseline 文件。
- 本轮没有接普通 UI 执行按钮。
- 本轮没有启动真实 Tauri / Browser / Chrome / Vite dev / screenshot。

## 需要复核线重点看

1. B2 是否真正通过 J2 run unit / `codex_control` / 统一 `real_execution_product_command` / new-session Phase B 触发。
2. Allowed write root 是否仍然收窄到 `.workbench/stage-j/j2-b`，而不是整个隔离项目根。
3. 全项目 manifest 是否只新增 allowed write file。
4. Prompt body 是否未持久化到 sidecar / runtime / workflow state。
5. runner stderr summary 噪声是否只应归类为 P2。
6. 是否存在普通前端 UI 误触发 B2 真实执行入口。

## 主管复核

长期只读复核线结论为带 P2 通过，无 P0/P1。主管线接受 B2 为 `accepted_with_deferred_items`。

主管收口记录见：

- `evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`

## 下一步

1. 进入 J3 memory capture bus。
2. J3 消费本轮 B2 的 runtime / audit / readback / worker report refs，生成 observation / candidate。
3. 正式记忆仍必须走既有确认链路。

## 不能声明

- 不能声明 J2-B 已完成完整 C5 / observation / candidate 回收闭环。
- 不能声明 J3 / J4 / J5 / J6 完成。
- 不能声明任意项目无限制自由执行或 Stage J 完成。
