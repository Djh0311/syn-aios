# Stage H / H5-Level-B2 Mario Test Project Workflow Write Probe v1

日期：2026-06-08

状态：已完成，并已通过全局主管恢复复核。

用途：在 H5-Level-B1 已完成 `mario test` read-only 真实 `resume` probe 并通过全局主管复核后，执行第一条受控写入型项目工作流真实派发 probe。B2 只允许在 `/Users/yoyi/Documents/mario test` 的工作台专用探针目录写一个可核验文件，用来证明真实 worker 能在授权写入范围内完成最小项目变更、回传 readback、生成 worker report candidate，并进入项目主管 process fact handoff。B2 不接受为 H5 通用产品化完成或阶段 H 完成。

## 0. 先说薄弱点

- B1 证明了既有开发线 worker session 可以通过产品 runner 做 read-only `resume`，但没有证明授权写入、项目文件 diff / hash 回收、worker report 到 process fact handoff 的写入型闭环。
- B2 会触发真实 Codex，并允许测试项目内的最小写入，风险高于 B1。
- 用户已在当前全局主管线授权：测试项目内的权限可以给，自己建立的测试项目和 `mario test` 都可以给。但这不是裸授权；开发线仍必须按本任务包冻结执行点、写入范围、prompt/ref/hash、runtime log、audit、readback、evidence 和 rollback 记录执行。
- B2 只验证既有 worker session 的 `resume + workspace-write` 路径，不验证 `new_session`。如果开发线发现必须新建 session，必须停止，回到 H3-B retry。
- B2 只允许写 `.workbench/h5-b2/real-dispatch-write-probe.md` 或同目录下的本轮运行记录；`index.html`、`styles.css`、`game.js`、`README.md` 必须 hash 前后一致。
- 如果产品路径无法触发真实 runner，开发线可以回交阻断；未经全局主管再次确认，不允许用 direct CLI diagnostic 冒充产品路径完成。

## 1. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`
- `tasks/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`
- `tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`
- `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`

## 2. 执行点授权

本任务包获得的执行点授权范围：

- 授权来源：用户在当前全局主管线明确表示“测试项目内的任何权限都可以给，自己建立的测试项目和 `mario test` 都可以给”。
- 授权项目：`/Users/yoyi/Documents/mario test`。
- 授权 operation：一次 `codex-local` `resume`。
- 授权 target session：`019e798a-ac37-7771-b982-e38084fcd22e`。
- 授权 target role：开发线 worker。
- 授权 sandbox：`workspace-write`，但只能用于本任务包允许的写入范围。
- 授权项目写入：只允许新增或更新 `/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md`，以及同目录下必要的本轮探针元数据文件。不得修改游戏核心文件。
- 授权工作台写入：允许在 `/Users/yoyi/workspace/product-line` 下写产品代码、测试、workflow sidecar、continuation、runtime log、audit、readback、evidence、handoff 和必要 fixture。
- 授权 Codex home 副作用：真实 Codex CLI 执行必然写入 `/Users/yoyi/.codex` 的最小原生会话状态；不授权读取用户完整会话数据或 secret。

不授权：

- 不授权读取 auth/token/secret/`.env`/keychain/OAuth/provider credential。
- 不授权读取完整 transcript 或 rollout。
- 不授权 `new_session`。
- 不授权自动重试。
- 不授权 stop / kill / restart。
- 不授权修改 `index.html`、`styles.css`、`game.js`、`README.md`。
- 不授权修改 `.git`。
- 不授权 planned adapters 真实接入。
- 不授权把 worker report 直接写正式事实或正式记忆。

## 3. 冻结的执行对象

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
target_node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
adapter_id: codex-local
operation: resume
sandbox: workspace-write
allowed_project_write_path: /Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md
readback_marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
```

## 4. 目标

H5-Level-B2 必须完成：

1. 冻结本轮 project / workflow / node / work item / prepared dispatch / task package artifact / task memory packet fingerprint。
2. 通过 H5 bridge 或等价后端产品路径构造 `CodexLocalExecutionRequest`，而不是前端或手工命令直接拼 CLI。
3. 运行 H1 guard、H4 duplicate guard、G2 diagnostics preflight、M6 memory packet stale / lint 检查。
4. 获得本任务包执行点授权后，触发一次真实 `codex-local resume`。
5. 写入 continuation / attempt / runtime log / audit / readback refs。
6. 让 worker 在允许路径写入最小探针文件。
7. readback 成功时生成 worker report candidate 或等价结构化结果摘要。
8. process fact 只进入 C5 handoff / decision 状态，不自动写正式事实，不自动写正式记忆。
9. 记录执行前后核心项目文件 hash，证明 `index.html` / `styles.css` / `game.js` / `README.md` 未变。
10. 记录执行后探针文件 hash 和内容摘要。
11. 新增 H5-Level-B2 evidence / handoff。
12. 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 H-I plan。

## 5. 非目标

本任务不做：

- 不创建新 Codex session。
- 不派发到验证线、回收线或总指导线。
- 不修改游戏核心文件。
- 不跑四角色完整 workflow machine。
- 不做自动重试、自动恢复、取消 / kill 产品化。
- 不做 H4-Level-B 真实失败 / 超时探针。
- 不做产品 command 正式化。
- 不接 planned adapters。
- 不做 provider credential store / model verification。
- 不做 H6 UI / Tauri 验收。
- 不把 B2 成功说成 H5 通用产品化或阶段 H 完成。

## 6. Prompt 合同

执行线应使用受控 prompt source，完整 prompt 不进入 shell argv、runtime log、audit、普通 evidence 或 UI。执行线必须记录 prompt summary / ref / sha256。

prompt summary：

```text
H5 Level B2 project-workflow-bound workspace-write dispatch probe for mario test codex-dev worker.
```

prompt ref：

```text
workbench-managed:h5-level-b2:mario-test:codex-dev:write-probe:v1
```

readback marker：

```text
H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
```

prompt 必须要求 worker：

- 只写入 `/Users/yoyi/Documents/mario test/.workbench/h5-b2/real-dispatch-write-probe.md`。
- 文件内容必须包含 marker、时间、scope、changed_files、process_fact_candidate。
- 不修改 `index.html`、`styles.css`、`game.js`、`README.md`。
- 不运行测试、不启动服务、不读取 secret、不读取完整 transcript / rollout。
- 回复固定 marker 和最小结构化 worker report candidate。

建议探针文件内容字段：

```text
marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
scope: h5_level_b2_workspace_write_project_workflow_dispatch_probe
changed_files:
- .workbench/h5-b2/real-dispatch-write-probe.md
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and wrote the authorized probe file only.
```

建议 worker report candidate 最小字段：

```text
status: completed
marker: H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08
scope: workspace_write_project_workflow_dispatch_probe
changed_files:
- .workbench/h5-b2/real-dispatch-write-probe.md
unchanged_core_files:
- index.html
- styles.css
- game.js
- README.md
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and wrote the authorized probe file only.
```

## 7. 产品路径优先级

优先级 1：工作台产品路径。

- 使用后端应用服务 / Tauri command / Rust runner / existing H5 bridge 触发真实 runner。
- 该路径必须写 continuation / runtime log / audit / readback。
- 若缺少执行 command 或产品路径只停留在 preview，开发线应先补最小产品路径，再执行。

优先级 2：回交阻断。

- 如果产品路径不能触发真实 runner，开发线应回交阻断，说明缺少哪个产品 command / guard / runtime log / audit / readback。
- direct CLI diagnostic 必须由全局主管另行确认，不能默认使用。

## 8. 执行前检查

执行线必须在真实执行前完成：

1. 读取本任务包、H5-Level-B 授权包、B1 evidence / handoff / supervisor review。
2. 复核 target session / cwd / sandbox / marker。
3. 确认 no duplicate queued/running dispatch。
4. 确认 diagnostics 无 blocking degraded state。
5. 确认 task memory packet 未 stale，lint 无 blocking；若无可用 packet，按任务包记录阻断或补产品路径，不静默执行。
6. 记录核心项目文件执行前 hash：
   - `/Users/yoyi/Documents/mario test/index.html`
   - `/Users/yoyi/Documents/mario test/styles.css`
   - `/Users/yoyi/Documents/mario test/game.js`
   - `/Users/yoyi/Documents/mario test/README.md`
7. 记录探针文件执行前状态：不存在 / 已存在及 hash。
8. 计算 prompt sha256。
9. 记录 expected workflow revision。
10. 记录 runtime log / audit / readback refs preview。
11. 确认需要触碰 `/Users/yoyi/.codex` 的最小副作用。

## 9. 成功验收

成功必须同时满足：

- 真实 `codex-local resume` 被触发。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- sandbox 为 `workspace-write`，且写入范围只落在 `.workbench/h5-b2/`。
- readback / last message 包含 `H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08`。
- continuation / attempt / runtime log / audit / readback refs 可追溯。
- worker report candidate 或等价结构化摘要可追溯。
- process fact handoff 状态明确，未自动写正式事实 / 正式记忆。
- 探针文件存在，内容包含 marker，hash 已记录。
- `mario test` 四个核心项目文件 hash 前后一致。
- 未读取完整 transcript / rollout / secret。
- evidence / handoff 写清可接受范围和不接受范围。

## 10. 失败验收

以下任一情况必须失败或阻断：

- 产品路径无法触发真实 runner。
- guard blocked。
- duplicate dispatch blocked。
- diagnostics blocking degraded。
- memory packet stale / lint blocking。
- exit code nonzero。
- timeout。
- readback failed / unavailable / timed_out。
- last message 缺 marker。
- 探针文件未写入或内容缺 marker。
- 核心项目文件 hash 变化。
- 写入 `.workbench/h5-b2/` 之外的项目文件。
- 需要读取完整 transcript / rollout / secret。
- runtime log / audit / readback 无法写入。

失败时必须：

- `result_count=null`。
- 不自动重试。
- 不自动 kill / stop / restart。
- 不自动回滚，除非用户另行授权。
- 写 failure evidence / handoff。

## 11. 验证命令

最小验证：

```text
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib
rustfmt --check src/session_continuation_store.rs src/h5_project_dispatch_bridge.rs src/codex_local_runner.rs src/types.rs src/commands.rs
```

如果本轮补了前端状态展示或权限弹层，必须追加：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

本任务包默认不要求真实 Tauri 截图；若开发线改 UI，必须按 UI 显示边界规则补真实 Tauri 或明确降级截图证据。

## 12. Evidence / Handoff

执行线必须新增：

- `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`

开发线回交后，全局主管必须另写主管复核：

- `evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1-result.md`

## 13. 回收结论边界

B2 若成功，最多接受为：

- H5-Level-B2 单项目 workspace-write 真实派发 probe 完成。
- `mario test` 开发线 worker session 能通过产品 runner 在授权写入范围内写入探针文件。
- continuation / runtime log / audit / readback / worker report candidate / process fact handoff 可追溯。

B2 即使成功，也不接受为：

- H5 通用项目工作流真实派发产品化完成。
- H5 product command 正式化完成。
- H3-B new-session 成功。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试、stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- worker report 写成正式事实或正式记忆。
- 阶段 H 完成。
