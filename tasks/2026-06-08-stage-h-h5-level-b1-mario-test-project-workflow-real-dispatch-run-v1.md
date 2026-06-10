# Stage H / H5-Level-B1 Mario Test Project Workflow Real Dispatch Run v1

日期：2026-06-08

状态：已完成，并已通过全局主管回收复核。开发记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`；主管复核见 `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`。

用途：在 H5-Level-A 非真实产品路径集成和 H5-Level-B 授权与 fixture freeze 完成后，执行第一条真实项目工作流派发 probe：从工作台项目工作流 / prepared dispatch / task package / memory packet / permission envelope 出发，真实派发到 `mario test` 的既有开发线 worker session，完成一次 read-only `codex-local resume`，并回收 continuation / runtime log / audit / readback / worker report candidate / process fact handoff 证据。

## 0. 先说薄弱点

- 这是 H5-Level-B1 执行任务，会触发真实 Codex。它比 H5-Level-A 和 H5-Level-B 授权包风险更高。
- 用户已在当前主管线明确授权：测试项目内的权限可以给，自己建立的测试项目和 `mario test` 都可以给。但这不是裸授权；开发线仍必须按本任务包冻结执行点、记录副作用、回收 evidence / handoff。
- B1 只验证既有 worker session 的 `resume` 路径，不验证 `new_session`。如果开发线发现必须新建 session，必须停止，回到 H3-B retry。
- B1 默认 read-only，不允许修改 `/Users/yoyi/Documents/mario test` 项目文件。真实写入项目文件的 B2 必须另拆任务包。
- 历史 `mario test` 成功不能复用为本轮证据；本轮必须产生新的 attempt / readback / evidence / handoff。
- 如果产品路径无法触发真实 runner，开发线可以回交阻断或做经主管授权的 direct CLI diagnostic；direct CLI diagnostic 不能单独接受为 H5 产品路径完成。

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
- `evidence/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1-result.md`
- `evidence/2026-05-30-mario-test-four-role-workflow-state-bindings-v1.md`
- `handoffs/2026-05-30-mario-test-four-role-workflow-state-bindings-v1-result.md`

## 2. 执行点授权

本任务包获得的执行点授权范围：

- 授权来源：用户在 2026-06-08 当前全局主管线明确表示“测试项目内的任何权限都可以给，自己建立的测试项目和 `mario test` 都可以给”。
- 授权项目：`/Users/yoyi/Documents/mario test`。
- 授权 operation：一次 `codex-local` `resume`。
- 授权 target session：`019e798a-ac37-7771-b982-e38084fcd22e`。
- 授权 target role：开发线 worker。
- 授权副作用：Codex CLI 执行必然写入 `/Users/yoyi/.codex` 的最小原生会话状态。
- 授权工作台写入：允许在 `/Users/yoyi/workspace/product-line` 下写产品代码、测试、workflow sidecar、continuation、runtime log、audit、readback、evidence、handoff 和必要 fixture。
- 授权项目写入：B1 不授权修改 `/Users/yoyi/Documents/mario test` 项目文件；只允许读取 hash / 文件元信息证明未变更。

不授权：

- 不授权读取 auth/token/secret/`.env`/keychain/OAuth/provider credential。
- 不授权读取完整 transcript 或 rollout。
- 不授权 `new_session`。
- 不授权自动重试。
- 不授权 stop / kill / restart。
- 不授权修改 `mario test` 项目文件。
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
sandbox: read-only
readback_marker: H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
```

## 4. 目标

H5-Level-B1 必须完成：

1. 冻结本轮 project / workflow / node / work item / prepared dispatch / task package artifact / task memory packet fingerprint。
2. 通过 H5 bridge 或等价后端产品路径构造 `CodexLocalExecutionRequest`。
3. 运行 H1 guard、H4 duplicate guard、G2 diagnostics preflight、M6 memory packet stale / lint 检查。
4. 获得本任务包执行点授权后，触发一次真实 `codex-local resume`。
5. 写入 continuation / attempt / runtime log / audit / readback refs。
6. readback 成功时生成 worker report candidate 或等价结构化结果摘要。
7. process fact 只进入 C5 handoff / decision 状态，不自动写正式事实，不自动写正式记忆。
8. 记录执行前后 `/Users/yoyi/Documents/mario test` 四个项目文件 hash，证明 B1 没有修改项目文件。
9. 新增 H5-Level-B1 evidence / handoff。
10. 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 H-I plan。

## 5. 非目标

本任务不做：

- 不创建新 Codex session。
- 不派发到验证线、回收线或总指导线。
- 不修改 `mario test` 项目文件。
- 不跑四角色完整 workflow machine。
- 不做自动重试、自动恢复、取消 / kill 产品化。
- 不做 H4-Level-B 真实失败 / 超时探针。
- 不接 planned adapters。
- 不做 provider credential store / model verification。
- 不做 H6 UI / Tauri 验收。
- 不把 B1 成功说成阶段 H 完成。

## 6. Prompt 合同

执行线应使用受控 prompt source，完整 prompt 不进入 shell argv、runtime log、audit、普通 evidence 或 UI。执行线必须记录 prompt summary / ref / sha256。

prompt summary：

```text
H5 Level B1 project-workflow-bound read-only dispatch probe for mario test codex-dev worker.
```

prompt ref：

```text
workbench-managed:h5-level-b1:mario-test:codex-dev:read-only-probe:v1
```

readback marker：

```text
H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
```

prompt 必须要求 worker：

- 只回复固定 marker 和最小结构化 worker report candidate。
- 不读取、列出或修改任何项目文件。
- 不运行命令。
- 不创建计划。
- 不调用工具。
- 不访问 secret、token、auth、`.env`、credential、完整 transcript 或 rollout。

## 7. 产品路径优先级

优先级 1：工作台产品路径。

- 使用后端应用服务 / Tauri command / Rust runner / existing H5 bridge 触发真实 runner。
- 该路径必须写 continuation / runtime log / audit / readback。
- 若缺少执行 command 或产品路径只停留在 preview，开发线应先补最小产品路径，再执行。

优先级 2：direct CLI diagnostic，仅在产品路径阻断且全局主管同意时使用。

- 只能证明 Codex CLI 和目标 session 可用。
- 不能单独接受为 H5-Level-B1 产品路径完成。
- 仍必须记录 hash、exit code、last message、`.codex` 副作用和不接受范围。

参考 CLI 语义：

```text
codex exec -C "/Users/yoyi/Documents/mario test" --sandbox read-only resume --skip-git-repo-check --json --output-last-message <last-message-path> 019e798a-ac37-7771-b982-e38084fcd22e
```

prompt 必须通过 stdin 传入，不得拼进 shell 字符串。

## 8. 执行前检查

执行线必须在真实执行前完成：

1. 读取本任务包和 H5-Level-B 授权包。
2. 复核 target session / cwd / sandbox / marker。
3. 确认 no duplicate queued/running dispatch。
4. 确认 diagnostics 无 blocking degraded state。
5. 确认 task memory packet 未 stale，lint 无 blocking；若无可用 packet，按任务包记录阻断或补产品路径，不静默执行。
6. 记录项目文件执行前 hash：
   - `/Users/yoyi/Documents/mario test/index.html`
   - `/Users/yoyi/Documents/mario test/styles.css`
   - `/Users/yoyi/Documents/mario test/game.js`
   - `/Users/yoyi/Documents/mario test/README.md`
7. 计算 prompt sha256。
8. 记录 expected workflow revision。
9. 记录 runtime log / audit / readback refs preview。
10. 确认需要触碰 `/Users/yoyi/.codex` 的最小副作用。

## 9. 成功验收

成功必须同时满足：

- 真实 `codex-local resume` 被触发。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`。
- readback / last message 包含 `H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08`。
- continuation / attempt / runtime log / audit / readback refs 可追溯。
- worker report candidate 或等价结构化摘要可追溯。
- process fact handoff 状态明确，未自动写正式事实 / 正式记忆。
- `mario test` 四个项目文件 hash 前后一致。
- 未读取完整 transcript / rollout / secret。
- evidence / handoff 写清可接受范围和不接受范围。

## 10. 失败验收

以下任一情况必须失败或阻断：

- 产品路径无法触发真实 runner，且没有主管批准 direct CLI diagnostic。
- guard blocked。
- duplicate dispatch blocked。
- diagnostics blocking degraded。
- memory packet stale / lint blocking。
- exit code nonzero。
- timeout。
- readback failed / unavailable / timed_out。
- last message 缺 marker。
- 项目文件 hash 变化。
- 需要读取完整 transcript / rollout / secret。
- runtime log / audit / readback 无法写入。

失败时必须：

- `result_count=null`。
- 不自动重试。
- 不自动 kill / stop / restart。
- 不自动回滚，除非用户另行授权。
- 写 failure evidence / handoff。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 可能改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 可能改读模型摘要或状态显示。
- [x] 可能改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

若执行线改可见 UI，必须遵守：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

允许：

- 现有项目工作流节点详情显示 H5-Level-B1 dispatch status、permission、runtime log、audit、readback、worker report candidate refs。
- 运行中 / 通知 / 待办分开显示状态。
- 管理入口显示脱敏 runtime log / diagnostics / audit refs。

禁止：

- 新增一级入口或任务包管理器。
- 自由聊天式裸 Codex 控制台。
- 未执行前显示“Codex 已收到任务”“worker 执行中”。
- readback 失败显示为 0 条。
- worker report 显示为正式事实或系统已记住。

## 11. 验证要求

如改 Rust：

```text
cargo test --lib h5_project_dispatch_bridge
cargo test --lib codex_local_runner
cargo test --lib h4_execution_boundary
cargo test --lib runtime_log_store
cargo test --lib session_continuation
cargo test --lib
rustfmt --check ...
```

如改前端 / TS：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

真实执行验收必须额外记录：

- 执行前后 hash。
- command argv 摘要。
- prompt summary / ref / sha256。
- exit code / timeout。
- last message marker。
- continuation / runtime log / audit / readback refs。
- 是否写 `/Users/yoyi/.codex`。
- 是否修改 `mario test`。

## 12. Evidence / Handoff

执行线必须新增：

- `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`
- `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`

如需要保存 raw 摘要文件，放入：

- `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1/`

不得保存：

- 完整 prompt。
- 完整 transcript / rollout。
- secret / token / auth / credential。
- raw stdout/stderr 中的敏感内容。

## 13. 回交要求

开发线完成后中文回交：

- 是否真实执行了 `codex exec` / `codex exec resume`。
- 是否通过产品路径执行；如果用了 direct CLI diagnostic，说明为什么不能接受为产品路径完成。
- 是否发送 prompt。
- 是否写 `/Users/yoyi/.codex`。
- target project / workflow / node / work item / dispatch / session。
- task memory packet fingerprint 和 stale / lint 状态。
- runtime log / audit / readback / worker report / process fact refs。
- 文件 hash / diff / rollback 状态。
- 验证命令结果。
- 接受范围和不接受范围。

## 14. 完成后口径

如果成功，最多接受为：

- H5-Level-B1 `mario test` 既有开发线 worker session 的 read-only real dispatch probe 完成。
- 工作台 H5 项目工作流真实派发链路第一次可追溯真实执行完成。

仍不接受为：

- H5 全部完成。
- `new_session` 产品化完成。
- 四角色工作流完整重跑完成。
- 真实写入项目文件能力完成。
- 自动重试 / 自动恢复完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- H6 UI / Tauri 验收完成。
- 阶段 H 完成。
