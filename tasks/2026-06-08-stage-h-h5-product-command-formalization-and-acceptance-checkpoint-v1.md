# Stage H / H5 Product Command Formalization And Acceptance Checkpoint v1

日期：2026-06-08

状态：已完成并通过全局主管复核。

用途：把 H5 从 Level A 非真实产品路径、B1 read-only 真实 probe、B2 workspace-write 真实 probe，收敛成一个可开发、可复核、可进入阶段 H 后续的产品化 checkpoint。这个任务包不再继续拆 B3/B4 小探针；入口文档只在本任务完成或阻断时做 checkpoint 同步。

## 0. 全局主管理解

已知事实：

- H5 Level A 已完成：prepared dispatch 到 CodexLocal request / permission / runtime / audit / readback / worker report handoff 的非真实产品路径已通过全局主管复核。
- H5-Level-B1 已完成：`/Users/yoyi/Documents/mario test` 既有开发线 worker session 完成一次 read-only 真实 `resume` probe。
- H5-Level-B2 已完成：同一 `mario test` worker session 完成一次 workspace-write 真实 `resume` probe，只写 `.workbench/h5-b2/real-dispatch-write-probe.md`，核心项目文件 hash 前后一致。
- B1/B2 证明了受控单项目真实派发可行，但还不等于 H5 通用项目工作流真实派发产品化完成。
- 用户要求后续不要把任务拆得太细，入口文档只在 checkpoint 同步，减少上下文维护成本。

未知项：

- H5 通用产品 command 是否已有完整 Tauri command 暴露，还是仍停留在内部 bridge / preview。
- 前端是否需要新增 UI 入口，还是只补 existing permission / project workflow sidebar 的状态展示。
- 是否需要在本任务末尾再做一次真实 Codex 执行验收；默认不需要，除非全局主管在执行点重新授权。

本任务假设：

- 本轮以产品代码正式化、测试矩阵、证据矩阵和 H5 checkpoint 复核为主。
- 默认不执行新的真实 `codex exec` / `codex exec resume`，不新增真实 probe。
- 如开发线判断必须追加真实执行，只能在本任务内提交执行点授权清单，由全局主管确认后再执行。

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
- `tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md`
- `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`
- `evidence/2026-06-08-stage-h-h5-level-b2-supervisor-acceptance-review-v1.md`

## 2. 目标

本任务作为 H5 合并 checkpoint，目标是：

1. 正式化 H5 产品 command / 后端服务边界：从 C4 prepared dispatch 进入 H1/H2 runner 契约，前端不拼 CLI。
2. 明确并实现 command 的两类路径：
   - `preview / readiness / permission envelope`：可安全反复调用，不执行真实 Codex。
   - `execute after explicit approval`：只在执行点授权后触发真实 runner，并写 continuation / runtime log / audit / readback。
3. 把 B1/B2 真实 probe 结果纳入 H5 acceptance matrix，证明产品化基础来自真实证据，而不是只来自 fake runner。
4. 统一 H5 attempt / readback / worker report candidate / process fact handoff / diagnostics 的状态口径。
5. 补齐必要测试：成功、guard blocked、diagnostics blocked、duplicate blocked、memory stale、readback unavailable / failed / timed out、user rejected。
6. 如涉及 UI，按既有 UI 方案接入现有项目工作流 / 权限弹层 / 管理入口，不新增任务包中心，不混淆通知 / 待办 / 运行中。
7. 新增 H5 checkpoint evidence / handoff，并只在任务收口时同步权威入口文档。

## 3. 非目标

本任务默认不做：

- 不新增 B3/B4 小探针。
- 不默认执行新的真实 `codex exec`。
- 不默认执行新的真实 `codex exec resume`。
- 不创建真实 Codex new session；H3-B retry 仍需单独授权。
- 不执行 H4-Level-B 真实失败 / 超时探针。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不接 planned adapters 真实执行。
- 不做 provider credential store / model verification。
- 不做自动重试、自动恢复、stop / kill / restart 产品化。
- 不把 worker report、readback、tool output、observation 或 candidate 直接写正式事实 / 正式记忆。
- 不把 H5 checkpoint 说成阶段 H 完成；阶段 H 是否完成要看 H6/H7 或后续 H acceptance 计划。

## 4. 工作线职责

建议复用既有对话线程，不随手新建一次性线程。

开发线职责：

- 阅读本任务和 H5/B1/B2 证据。
- 实现或修补 H5 product command / bridge / runner boundary。
- 补齐 Rust / TS 类型和 Tauri command wrapper；只有 UI 确实需要时才改前端。
- 不执行新的真实 Codex，除非收到全局主管执行点授权。

验证线职责：

- 复核 command contract、状态机、sidecar 写入、runtime log、audit、readback 和 diagnostics 边界。
- 复跑相关 Rust / 前端测试。
- 扫描误导文案和敏感路径。
- 验证 B1/B2 evidence matrix 与新产品 command 口径一致。

全局主管职责：

- 冻结任务范围，不继续拆小 probe。
- 审核开发线 handoff 和验证线回交。
- 决定是否需要执行点授权。
- 任务完成或阻断时，同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 H-I plan。

## 5. 产品 command 要求

产品 command 必须做到：

- 只接受工作台内部 prepared dispatch / work item / workflow node / task package artifact / memory packet snapshot 引用。
- 校验 C1/C3 authorization、dispatch state、duplicate guard、diagnostics、memory packet stale / lint、allowed write roots、target session 或 new session strategy。
- 构造 `CodexLocalExecutionRequest`，记录 prompt summary / ref / hash，不把完整 prompt 放入 UI、普通 evidence、audit 或 runtime log。
- 执行前必须输出 permission envelope / readiness decision。
- 真实执行只能由后端 runner 触发，且必须在 explicit approval 后。
- 执行结果必须写 continuation / attempt / runtime log / audit / readback refs。
- readback unknown 状态必须保持 `result_count = null` 或等价未知，不写成真实 0 条。
- worker report candidate 只进入 C5 handoff / process fact decision，不自动写正式事实或正式记忆。

## 6. UI 显示边界

若本任务触及 UI：

- 必须先复核 `docs/workbench-frontend-display-boundary-v1.md` 和 `docs/plans/task-package-ui-display-boundary-rule-v1.md`。
- 权限弹层只显示人话摘要、影响范围、风险、allowed write roots、prompt summary / ref / hash、memory packet 摘要、readback plan、runtime / audit preview。
- 项目工作流页只显示状态、readiness、attempt 摘要、readback 状态、worker report candidate 和 process fact handoff，不铺 raw transcript / raw stdout / stderr。
- 管理入口显示 runtime log / audit / diagnostics；通知、待办、运行中继续分离。
- 禁止显示“Codex 已收到任务”“worker 执行中”“真实派发已开始”，除非真实执行已经发生并有 attempt/runtime 证据。
- 无真实窗口或浏览器截图工具时，不得声称 UI 验收完成；可记录为 deferred。

## 7. 执行点授权规则

本任务包创建本身不授权新的真实执行。

如开发线认为必须追加真实执行验收，必须先提交执行点授权清单：

- project root、workflow、node、work item、dispatch id。
- operation：`resume` 或 `new_session`。
- target session id 或 new session strategy。
- sandbox、cwd、allowed write roots、denied paths。
- prompt summary / ref / sha256。
- expected readback marker。
- `.codex` 最小副作用说明。
- rollback / cleanup / hash-diff plan。
- runtime log / audit / evidence / handoff refs。
- stop condition。

全局主管未确认前，不能执行真实 `codex exec` / `codex exec resume`。

## 8. 验收标准

产品验收至少满足：

- H5 product command / bridge 已实现或明确证明已有实现并补齐缺口。
- Preview / readiness / permission envelope 可在不执行真实 Codex 的情况下生成。
- Explicit approval 后的 execute 路径在代码层可追溯到 H1/H2 runner 契约。
- B1/B2 evidence matrix 已纳入 acceptance，不再把单项目 probe 误写为 H5 通用完成。
- Runtime log、audit、readback、diagnostics、worker report candidate、process fact handoff 边界清楚。
- Unknown readback 不显示为真实 0 条。
- 禁止读取 secret / transcript / rollout 的边界有测试或扫描证据。
- 若改 UI，完成 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`，并按工具可用性记录截图 / smoke。
- Rust 相关测试通过，至少覆盖 H5 bridge、session continuation、codex local runner、runtime log、diagnostics、workflow authorization。
- 新增 evidence / handoff，明确接受范围和不接受范围。

## 9. 推荐验证命令

按实际改动裁剪，不需要为了文档任务跑无关命令。

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib h5_project_dispatch_bridge
cargo test --lib session_continuation
cargo test --lib codex_local_runner
cargo test --lib runtime_log
cargo test --lib diagnostics
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/h5_project_dispatch_bridge.rs src/session_continuation_store.rs src/codex_local_runner.rs src/runtime_log_store.rs src/diagnostics_store.rs src/types.rs src/commands.rs
```

如没有改 UI，可以不跑 frontend build，但 evidence 必须说明原因。

## 10. 收口要求

任务完成时必须新增：

- `evidence/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`
- `handoffs/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1-result.md`

checkpoint 同步范围：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

不要在中间小修时反复同步入口文档；只在完成、阻断或范围改变时同步。

## 11. 不接受口径

即使本任务完成，也不接受为：

- H3-B retry 成功。
- `new_session` 产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- 自动重试 / 自动恢复 / stop / kill / restart 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 完整多 agent / 多模型协作抽象完成。
- 阶段 H 整体完成。
