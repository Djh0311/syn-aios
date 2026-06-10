# Evidence: Stage H / H5 Project Workflow Real Dispatch Integration Level A v1

日期：2026-06-08

## 1. 结论

H5-Level-A 非真实产品路径集成已完成，结论为：

```text
accepted_as_h5_level_a_non_real_product_path_integration
```

接受为：

- 新增 H5 项目工作流派发 bridge 的后端预览 / 校验路径。
- 可以从 C4 `prepared` dispatch、M6 frozen task memory packet、H1 `CodexLocalExecutionRequest` / guard、H4 duplicate / readback unknown-result 规则、G1 runtime log ref、G2 diagnostic minimal input、C5 worker report / process fact handoff 和 C6 final review handoff status 生成 Level A 预览。
- 预览会输出 `permission_envelope`、`codex_local_request`、`codex_local_guard`、`runtime_audit_preview`、`readback_boundary`、`worker_report_candidate` 和 `process_fact_handoff`。
- Level A 预览固定 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`、`writes_workbench_state=false`。
- `readback_boundary.result_count = null`；`not_attempted`、readback failed / unavailable / timed out 等 unknown-result 不能被显示为真实 0 条。
- stale memory packet、duplicate active attempt、diagnostics blocking degraded、non-prepared dispatch、missing prompt ref/hash、resume target missing、new_session 缺 H3-B / Level-B 授权都会阻断 Level B。
- TS 类型和 Tauri wrapper 已补齐，但未接可见 UI。

不接受为：

- H5 已完成。
- H5-Level-B 真实项目工作流派发已授权或已执行。
- 真实 worker 已执行。
- 真实 Codex 已执行。
- `codex exec` / `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权、已执行或已成功。
- H4-Level-B 真实失败 / 超时探针已完成。
- worker report 已成为正式事实。
- observation / candidate 已成为正式记忆。
- planned adapters 已接入。
- provider credential / model 已验证。
- 阶段 H 已完成。

## 2. 实现落点

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/h5_project_dispatch_bridge.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`

## 3. 关键链路

H5 bridge 输入：

- `project_root`
- `project_id`
- `workflow_id`
- `dispatch_id`
- `actor_id`
- operation / session / cwd / sandbox preview fields
- prompt summary / ref / sha256
- optional H5 minimal diagnostic summary
- optional expected workflow revision

H5 bridge 读取 / 校验：

- `workflow-state.v0.json` 中 C4 `workflow_node_dispatches[]` 的 `state = prepared` dispatch。
- 绑定的 work item / node / target session / plan authorization id。
- 绑定 task package artifact 中的 M6 `memory_packet_snapshot` 和 fingerprint。
- M6 current store revisions，用于判断 memory packet stale。
- 工作台自有 `session-continuations.v1.json`，只用于 duplicate active attempt scope 检查。

H5 bridge 输出：

- permission envelope preview：adapter、operation、target session、cwd、allowed write roots、denied paths、prompt summary/ref/hash、memory fingerprint、readback boundary、`.codex` Level-B 最小授权说明。
- H1 `CodexLocalExecutionRequest`：结构化 request，可由 H1 guard inspection 生成 command plan preview，但 Level A 不调用 runner。
- runtime / audit preview refs：只作为 preview，不写 runtime log sidecar 或 audit event。
- readback boundary preview：`not_attempted` 且 `result_count=null`。
- worker report candidate：只作为 C5 结构候选，不是正式事实。
- process fact handoff：默认 `request_rework` candidate，不能确认 process fact。
- C6 final review handoff status：明确仍需要真实 worker report 和项目主管 process fact decision。

## 4. UI 边界

本轮没有改可见 UI，没有新增一级入口、右侧入口、项目页按钮或执行按钮。

只新增 TS 类型和 Tauri wrapper：

- `H5ProjectWorkflowDispatchPreviewInput`
- `H5ProjectWorkflowDispatchPreview`
- `previewH5ProjectWorkflowDispatch(...)`

因此没有显示：

- `Codex 已收到任务`
- `worker 执行中`
- `真实派发已开始`
- `worker 已执行`
- `系统已记住`

## 5. 验证记录

已通过：

```text
cargo test --lib h5_project_dispatch_bridge -- --nocapture
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/types.rs src/commands.rs src/lib.rs
npm run typecheck
npm run test:offline-interaction
npm run build
rg -n "H5 project workflow real dispatch integration 任务包草案|H5 任务包草案|已创建草案|当前只允许 Level A 协议|当前只接受为 H5-Level-A 协议|本任务包创建和 Level A 不做|不修改产品代码|执行 H5-Level-A 协议|H5-Level-A 协议 / 产品路径集成设计" CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md
```

结果摘要：

- H5 定向 Rust：3 passed。
- `cargo test --lib`：257 passed / 3 ignored。
- `rustfmt --check`：passed。
- `npm run typecheck`：passed。
- `npm run test:offline-interaction`：offline interaction tests passed: 12。
- `npm run build`：passed；保留既有 Vite chunk size warning。
- H5 stale wording scan：无匹配。
- Rust 保留既有 warning：`JsonRpcError::invalid_params` dead code。

## 6. 边界确认

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送 prompt。
- 创建真实 Codex session。
- 创建真实项目派发 run。
- 读取或写入 `/Users/yoyi/.codex`。
- 读取 auth / token / secret / `.env` / keychain / OAuth / provider credential / full transcript / rollout。
- 接 planned adapters 真实执行。
- 做 provider credential store / model verification。
- 做自动 retry / kill / stop 产品化。
- 把 worker report、observation、candidate 或 knowledge hit 写成正式事实 / 正式记忆。

## 7. H5-Level-B 仍需授权

进入 H5-Level-B 前仍必须逐项确认：

- 是否允许真实项目工作流派发。
- fixture project、workflow、node、work item、prepared dispatch。
- 使用 `resume` 还是 `new_session`；如使用 `new_session`，H3-B retry 是否已授权并回收清楚。
- 是否授权真实 `codex exec` / `codex exec resume`。
- 是否授权触碰 `/Users/yoyi/.codex` 的最小必要范围。
- allowed write roots、denied paths、rollback / cleanup。
- prompt summary/ref/hash，完整 prompt 仍不得进入 argv、shell、普通 evidence、runtime log 或 audit。
- task memory packet fingerprint / stale / lint。
- readback plan、runtime log、audit、diagnostics、evidence、handoff。
- duplicate / diagnostics / memory stale / prompt hash / readback failed / timed out / user rejected 时是否停止，不得自动 retry。
