# Stage J / J2-B B2 Supervisor Acceptance Review v1

日期：2026-06-09

状态：B2 workspace-write 真实 `new_session` 探针经长期只读复核线复核，结论为 `accepted_with_deferred_items`；允许进入 J3 memory capture bus。J2-B 仍不接受为 Stage J 完成。

## 复核对象

- 任务包：`tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`
- B2 real evidence：`evidence/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1.md`
- B2 real handoff：`handoffs/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1-result.md`
- Run artifacts：`tmp/j2-b-real-workflow-automation/runs/j2-b-b2-run-1781008761311238000/`
- Allowed write file：`tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`

## 复核线结论

长期只读复核线结论：带 P2 通过，无 P0/P1。

主管线接受该结论，并将 B2 real probe 收口为 `accepted_with_deferred_items`。

接受范围：

- 指定 Stage J 隔离项目 / 指定 J2 developer run unit 的 workspace-write 真实 `new_session` 探针完成。
- 执行路径走 J2-B B2 bridge：`J2 run unit -> codex_control -> real_execution_product_command -> Phase A -> new-session Phase B`。
- Product sidecar 中 `command_family = real_execution_product_command`，`operation_id = new_session`，`target_session_id = null`。
- `allowed_write_roots` 保持窄根：`.workbench/stage-j/j2-b`。
- Workflow audit event 同步窄写根，并回链 run unit、Product Command、Phase A / B attempt、runtime log、task package 和 memory packet ref。
- Phase B flags 显示 `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=true`。
- Readback 成功，`result_count=1`，last message 包含 `J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09`。
- 隔离项目 manifest 只包含 `README.md`、`project-notes.md` 和 allowed write file。
- `README.md` / `project-notes.md` baseline hash 保持冻结值。
- Prompt body 正文未持久化到 product command sidecar、continuation sidecar、runtime log、workflow state 或隔离项目文件；marker 命中只在 readback plan、allowed file 和 last message 等预期位置。
- 普通前端没有 B2 真实执行 wrapper / 按钮入口。

## P2 后置项

- runner stderr summary 仍含 remote plugin auth / MCP termination 噪声。它被截断保存在 warning summary 中，不影响 B2 acceptance；后续 J3/J4/J5 面向用户展示时应继续归类为 runner 噪声，不作为主线产品证据展示。

说明：复核线不再将 `target_cwd` 为隔离项目 root 视为 P2 阻断；理由是 allowed write root 已收窄到 `.workbench/stage-j/j2-b`，且全项目 manifest 已证明除 allowed write file 外无新增或修改。

## 主管线复核判断

B2 的关键风险已经被控制：

- 没有用 H5 / legacy / direct CLI / MCP canvas run / test helper 冒充。
- 没有普通 UI 按钮接入 B2 真实执行。
- 没有读取 `/Users/yoyi/.codex`、full transcript、rollout 或 secrets 的证据。
- `.codex` 写入只属于本轮授权真实 `new_session` 的 Codex CLI 原生状态写入。
- Allowed write path 的写入结果可作为 J3 的 memory capture 输入，但还没有完成 C5 / observation / candidate 回收闭环。

## Fresh Verify

主管线在复核等待期间重新跑了低风险测试，未触发 ignored real harness：

- `cargo test --lib project_workflow_automation`：11 passed / 2 ignored。
- `cargo test --lib real_execution_command`：33 passed / 3 ignored。
- `cargo test --lib session_continuation`：17 passed / 4 ignored。
- `cargo test --lib runtime_log`：6 passed。

保留既有 warning：`JsonRpcError::invalid_params` dead code warning，和 B2 acceptance 无关。

## 不能声明

- 不能声明 J2-B 已完成完整 C5 / observation / candidate 回收闭环。
- 不能声明 J3 memory capture bus 完成。
- 不能声明任意项目无限制自由执行完成。
- 不能声明 planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 完成。
- 不能声明 Stage J 完成。

## 下一步

进入 J3 memory capture bus。J3 必须消费 J1 / J2 / B2 的 runtime / audit / readback / worker report refs，生成 observation / candidate，并继续保持 FormalMemory 用户确认链路。
