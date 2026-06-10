# Stage H / H5-Level-B Project Workflow Real Dispatch Authorization And Fixture Freeze v1

日期：2026-06-08

状态：已完成授权与 fixture freeze；等待后续 H5-Level-B1 执行点授权。本任务包本身不执行真实 Codex。

用途：把 H5 Level A 已完成的 `prepared dispatch -> permission envelope -> CodexLocalExecutionRequest -> runtime/audit/readback preview -> worker report candidate/process fact handoff` 非真实链路，推进到 H5-Level-B 真实项目工作流派发前的授权包和 fixture freeze。此任务包只冻结执行对象、权限、fixture、prompt/ref/hash、readback、runtime log、audit、evidence、rollback 和停止条件；不授权立即执行真实 `codex exec` / `codex exec resume`。

## 0. 先说薄弱点

- H5 Level A 已通过主管复核，但它仍然只是非真实预览 / guard / handoff 链路；没有真实 worker / Codex 执行。
- H2 Phase B 和 E5 Level B 都证明 `mario test` 指定 session 的 `resume` 健康探针可行，但不能直接证明项目工作流真实派发产品化完成。
- H3-B 新会话真实 probe 已执行一次但失败分类完成，产品路径已补 `--skip-git-repo-check`；它仍不等于真实新 worker session 创建成功。
- 用户已允许在我方测试项目和 `/Users/yoyi/Documents/mario test` 内给权限，但这不是裸授权。每次真实 Codex 执行仍必须有任务包级执行点、prompt/ref/hash、`.codex` 最小范围、runtime log、audit、readback 和回滚记录。
- H5-Level-B 第一条推荐路径应先走既有 worker session 的 `resume`，验证项目工作流真实派发闭环；不要把这次 `resume` 成功包装成 H3-B new-session 成功。
- 历史 `mario test` 四角色 demo 曾跑通过，但不能复用历史结果当作本轮证据。本轮必须产生新的 attempt / readback / evidence / handoff。

## 1. 权威依据

必须服从：

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
- `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md`
- `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`
- `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`
- `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md`
- `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`
- `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`
- `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`
- `evidence/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-08-stage-h-h4-supervisor-acceptance-review-v1-result.md`
- `evidence/2026-05-30-mario-test-four-role-workflow-state-bindings-v1.md`
- `handoffs/2026-05-30-mario-test-four-role-workflow-state-bindings-v1-result.md`

## 2. 当前事实

已知：

- H5 Level A 已完成并通过主管复核；当前产品代码可以生成 / 预览 / 校验 H5 项目工作流派发链路，但不调用真实 runner。
- H4 Level A 已完成并通过主管复核；unknown-result 状态必须保持 `result_count=null`，duplicate blocked 必须写 attempt / audit / runtime log 且不调用 runner。
- H2 Phase B 已对 `/Users/yoyi/Documents/mario test` 总指导 session 完成一次真实 `codex exec resume` 探针，写入 `/Users/yoyi/.codex`，readback 返回固定标记。
- H3-B 已执行一次真实 `codex exec` new-session fixture probe，但因当时 command plan 缺少 `--skip-git-repo-check` 等原因失败分类；产品路径已修补，未二次执行。
- `/Users/yoyi/Documents/mario test` 已存在四角色工作流绑定：
  - project id：`project:users-yoyi-documents-mario-test`
  - workflow id：`workflow:users-yoyi-documents-mario-test:default`
  - 总指导 node：`workflow:users-yoyi-documents-mario-test:default:node:director`
  - 开发线 node：`workflow:users-yoyi-documents-mario-test:default:node:codex-dev`
  - 验证线 node：`workflow:users-yoyi-documents-mario-test:default:node:validation`
  - 回收线 node：`workflow:users-yoyi-documents-mario-test:default:node:review`
  - 总指导 native thread：`019e798a-6ce5-76c3-b8ee-33bd0fda841f`
  - 开发线 native thread：`019e798a-ac37-7771-b982-e38084fcd22e`
  - 验证线 native thread：`019e798a-f9d4-7922-8da8-1b6a8dbd6769`
  - 回收线 native thread：`019e798b-2ee7-7f90-beb2-9031f6ad3e05`

未知：

- 当前工作台 state 中是否已经存在可直接用于本轮 H5-Level-B 的 fresh prepared dispatch。
- 当前 `mario test` 开发线 worker session 是否仍可稳定 resume。
- 当前产品路径是否已足够从 H5 bridge 直接进入 real runner；若不能，执行线必须先实现最小受控执行路径，不得退回裸 CLI 当作产品化验收。
- readback 是否能稳定取得本轮 H5 固定 marker 和结构化 worker report candidate。

本任务包采用的假设：

- H5-Level-B 第一条真实派发优先验证 `resume` 到既有 `mario test` 开发线 worker session，而不是新建 session。
- 本轮推荐使用只读 / 无项目文件修改的最小 dispatch probe，先证明项目工作流派发链路本身真实可追溯；需要真实写项目文件的 B2 probe 另行授权。
- 如果执行线发现必须走 `new_session`，必须停止并回到 H3-B retry，不得在 H5-Level-B 中偷换前置。

## 3. 接受范围

本任务包完成后最多接受为：

- H5-Level-B 真实项目工作流派发的授权包和 fixture freeze 已创建。
- 推荐 fixture、目标 worker session、operation、allowed write roots、`.codex` 最小副作用、prompt summary/ref/hash、readback、runtime log、audit、evidence、rollback 和停止条件已冻结。
- 后续开发线可以按本任务包准备 H5-Level-B1 执行任务，但仍必须在执行点再次确认真实 Codex 执行。

本任务包完成后不接受为：

- H5-Level-B 已执行。
- H5 已完成。
- 真实项目工作流派发已发生。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- worker 已执行。
- H3-B retry 已授权或成功。
- H4-Level-B 真实失败 / 超时探针完成。
- 阶段 H 完成。

## 4. 推荐 H5-Level-B1 执行包

推荐第一条真实执行包如下。所有阻断项必须在 H5-Level-B1 执行任务中再次确认。

| 授权项 | 推荐值 | 是否阻断真实执行 | 说明 |
| --- | --- | --- | --- |
| project label | `mario test` | 是 | 明确指 `/Users/yoyi/Documents/mario test`，不是 `/Users/yoyi/codex-workflow-mario-test`。 |
| project root / cwd | `/Users/yoyi/Documents/mario test` | 是 | 必须在执行前复核存在。 |
| project id | `project:users-yoyi-documents-mario-test` | 是 | 来自历史四角色绑定。 |
| workflow id | `workflow:users-yoyi-documents-mario-test:default` | 是 | 本轮必须绑定到该 workflow 或新建隔离 H5 workflow；不能裸跑。 |
| target node | `workflow:users-yoyi-documents-mario-test:default:node:codex-dev` | 是 | H5 是项目主管派 worker，第一条建议派开发线 worker，不派总指导当 worker。 |
| target session | `019e798a-ac37-7771-b982-e38084fcd22e` | 是 | 既有开发线 native thread。 |
| operation | `resume` | 是 | 不依赖 H3-B new-session 成功。 |
| adapter | `codex-local` | 是 | planned adapters 仍不可执行。 |
| sandbox | `read-only` for B1 | 是 | B1 不授权项目文件修改；B2 写入 probe 另拆。 |
| allowed write roots | `/Users/yoyi/workspace/product-line` evidence / sidecars；Codex CLI 必需 `.codex` 最小副作用 | 是 | B1 不授权修改 `/Users/yoyi/Documents/mario test` 项目文件。 |
| denied paths | auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout | 是 | 任何需要读取这些内容的路径都必须停止。 |
| prepared dispatch | 待 H5-Level-B1 执行线创建或冻结 | 是 | 不能复用历史 run 当本轮 evidence。 |
| work item | 建议 `h5-level-b-real-dispatch-probe-v1` 或等价新 work item | 是 | 必须和 prepared dispatch 一一对应。 |
| task memory packet | 使用 M6 frozen snapshot 或明确记录本轮无可用 snapshot 而阻断 | 是 | stale / lint blocking 必须阻断。 |
| prompt summary | `H5 Level B project workflow real dispatch read-only probe` | 是 | 普通 evidence 只保存 summary/ref/hash，不保存完整 prompt。 |
| prompt ref | `workbench-managed:h5-level-b:mario-test:codex-dev:read-only-probe:v1` | 是 | 执行前必须产生 prompt hash。 |
| readback marker | `H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08` | 是 | last message 必须包含该 marker。 |
| readback plan | last message + workbench attempt/runtime/audit refs | 是 | readback failed/unavailable/timed_out => `result_count=null`。 |
| runtime log | 必须写脱敏 dispatch attempt / readback / failure 分类 | 是 | 不保存完整 prompt、raw transcript、secret。 |
| audit | 必须写用户/主管授权、guard result、dispatch started/completed/failed、readback status | 是 | audit 不替代 runtime log。 |
| evidence path | `evidence/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-run-v1.md` | 是 | 真实执行结果单独记录。 |
| handoff path | `handoffs/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-run-v1-result.md` | 是 | 不写入本授权包 handoff 冒充执行结果。 |

## 5. B1 Prompt 合同

H5-Level-B1 的 prompt 必须满足：

- 只要求 worker 返回固定 marker 和最小结构化 worker report candidate。
- 不要求读取、列出或修改项目文件。
- 不要求运行命令。
- 不包含 secret、token、auth、`.env`、credential、完整 transcript 或 rollout 请求。
- 不通过 shell argv 传入；必须通过 stdin 或工作台受控 prompt source 传入。
- 完整 prompt 不进入普通 evidence、runtime log、audit 或 UI；执行线只记录 prompt summary、prompt ref、prompt sha256 和 marker。

本任务包冻结的 readback marker：

```text
H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
```

建议 worker report candidate 最小字段：

```text
status: completed
marker: H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
scope: read_only_project_workflow_dispatch_probe
changed_files: []
process_fact_candidate: codex-local worker received a project-workflow-bound dispatch and returned a readback marker.
```

## 6. 执行前必须冻结的工作台对象

H5-Level-B1 执行线必须在真实执行前冻结：

1. `project_id`
2. `workflow_id`
3. `workflow_node_id`
4. `work_item_id`
5. `prepared dispatch id`
6. `task package artifact id`
7. `task memory packet fingerprint`
8. `prompt ref`
9. `prompt sha256`
10. `allowed write roots`
11. `sandbox`
12. `target session id`
13. `expected workflow revision`
14. `diagnostics summary`
15. `duplicate guard scope`
16. `runtime log refs preview`
17. `audit refs preview`
18. `readback plan`

如果当前产品代码无法冻结这些对象，执行线必须先补非真实产品路径或回交阻断，不能用手工命令绕过。

## 7. 真实执行路径要求

优先路径：

- 通过工作台后端应用服务 / Tauri command / Rust runner 路径执行 H5 dispatch。
- 该路径必须复用 H1 `CodexLocalExecutionRequest`、H4 duplicate/readback/failure 边界、G1 runtime log、G2 diagnostics 和 H5 bridge。
- 前端不得直接拼 `codex` 命令。

允许的诊断 fallback：

- 如果产品路径不能触发真实 runner，可由全局主管另行批准一次 direct CLI diagnostic。
- direct CLI diagnostic 只能证明 Codex CLI 可用，不能接受为 H5 产品路径完成。

建议 direct CLI 语义仅用于诊断参考，不是默认执行方式：

```text
codex exec -C "/Users/yoyi/Documents/mario test" --sandbox read-only resume --skip-git-repo-check --json --output-last-message <last-message-path> 019e798a-ac37-7771-b982-e38084fcd22e
```

## 8. 明确授权范围

后续 H5-Level-B1 如获执行点授权，允许：

- 对 `/Users/yoyi/Documents/mario test` 的开发线 worker session 执行一次真实 `codex exec resume` 或等价产品 runner。
- 发送符合本任务包 prompt 合同的真实 prompt。
- 由 Codex CLI 必然写入 `/Users/yoyi/.codex` 的最小原生会话状态。
- 在 `/Users/yoyi/workspace/product-line` 下写本轮 evidence、handoff、runtime log、audit、continuation、readback 和必要的工作台 sidecar。
- 读取 `/Users/yoyi/Documents/mario test` 四个项目文件 hash，用于证明 B1 不修改项目文件。

后续 H5-Level-B1 不允许：

- 修改 `/Users/yoyi/Documents/mario test` 项目文件。
- 读取 auth/token/secret/`.env`/keychain/OAuth/provider credential。
- 读取完整 transcript 或 rollout。
- 创建新的 Codex session。
- 派发到验证线、回收线或总指导线，除非另拆任务包。
- 自动重试、kill、stop、restart 或静默恢复。
- 把 worker report 直接写成正式事实或正式记忆。
- 把 observation / candidate / knowledge hit 当正式记忆。
- 把 readback failed/unavailable/timed_out 显示为真实 0 条结果。

## 9. 执行前检查

H5-Level-B1 执行前必须完成：

1. 复核本任务包仍是当前入口允许项。
2. 复核没有其他 queued / running duplicate dispatch。
3. 复核 G2 diagnostics 没有 blocking degraded state。
4. 复核 task memory packet fingerprint 未 stale，lint 没有 blocking。
5. 复核 target session 是 `019e798a-ac37-7771-b982-e38084fcd22e`，cwd 是 `/Users/yoyi/Documents/mario test`。
6. 记录执行前 hash：
   - `/Users/yoyi/Documents/mario test/index.html`
   - `/Users/yoyi/Documents/mario test/styles.css`
   - `/Users/yoyi/Documents/mario test/game.js`
   - `/Users/yoyi/Documents/mario test/README.md`
7. 计算 prompt sha256。
8. 记录 runtime log / audit / readback refs preview。
9. 明确 `.codex` 最小副作用会发生。
10. 由全局主管给出本次执行点确认。

## 10. 成功验收

H5-Level-B1 真实执行成功必须同时满足：

- 产品路径或经主管批准的执行路径确实发起一次真实 `codex-local` resume。
- `prompt_sent=true`。
- `real_codex_executed=true`。
- `writes_codex_home=true`，并如实记录该副作用。
- last message / readback 包含：

```text
H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08
```

- continuation / attempt / runtime log / audit / readback refs 均可追溯。
- worker report candidate 或等价结构化结果摘要已产生。
- process fact handoff 状态明确：只允许进入 C5 决策链路，不直接写正式事实。
- `/Users/yoyi/Documents/mario test` 四个项目文件 hash 前后一致。
- 没有读取完整 transcript、rollout 或敏感凭据。
- evidence / handoff 明确接受范围和不接受范围。

## 11. 失败验收

以下情况必须按失败或阻断记录，不得包装成通过：

- guard blocked。
- diagnostics blocking degraded。
- duplicate dispatch blocked。
- task memory packet stale。
- lint blocking 未处理。
- target session 不匹配。
- exit code nonzero。
- timeout。
- readback failed / unavailable / timed_out。
- last message 缺 marker。
- 项目文件 hash 变化。
- 需要读取 full transcript / rollout / secret。
- 产品路径无法写 runtime log / audit / readback refs。

失败时：

- `result_count` 必须保持 `null`，不能显示 0。
- 不自动重试。
- 不自动 kill Codex。
- 不自动回滚用户文件；只报告 diff / hash 和回滚建议，除非用户另行授权。
- 必须新增 failure evidence / handoff。

## 12. H5-Level-B2 后置写入探针

B1 成功后，如需要证明真实项目文件写入能力，应另拆 H5-Level-B2：

- 仅允许写 `/Users/yoyi/Documents/mario test/.workbench/h5-level-b/` 或单个明确 probe 文件。
- 必须有新的 prepared dispatch、prompt hash、allowed write root、diff/rollback、runtime/audit/readback。
- 不得复用 B1 授权。
- B2 仍不接受为复杂业务自动编排、自动重试、new-session 产品化或阶段 H 完成。

## UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取并纳入约束：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

本任务包不新增 UI。后续 H5-Level-B1 若改权限弹层、项目工作流状态、运行中 / 通知 / 待办 / 管理入口或智能体页 readback，需要补真实 Tauri 截图验收或明确记录未完成，普通浏览器 smoke 不能冒充真实 Tauri。

## 13. 验收本任务包

本任务包是文档 / 授权冻结任务，验收为：

- 新增本 H5-Level-B 授权与 fixture freeze 任务包。
- 新增 evidence。
- 新增 handoff。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 H-I plan。
- 扫描确认没有把 H5-Level-B 写成已执行、已完成或阶段 H 完成。

不要求运行 `npm` / `cargo`，因为本任务包不改产品代码。

## 14. 下一步

本任务包完成后，下一步不是直接进入 H6。

可选下一步：

- H5-Level-B1 执行任务：按本任务包执行 `mario test` 开发线 worker 的 read-only real dispatch probe。
- H3-B retry：如全局主管决定先补新会话成功证据，则回到 H3-B retry，不进入 H5-Level-B1。
- H4-Level-B：如需要真实失败 / 超时探针，则另拆 H4-Level-B。

除非 H5-Level-B1 成功回收，否则不得把 H5 写成完成。
