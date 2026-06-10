# Stage J / J2-B Controlled Real Workflow Automation Execution Point Freeze v1

日期：2026-06-09

状态：冻结包已通过长期只读复核线审查，结论为带 P2 通过；B1 execution bridge 已完成并通过主管线 fresh verify；B1 read-only 真实 `resume` 探针已执行，并经长期只读复核线复核后由主管线收口为 `accepted_with_deferred_items`；B2 execution bridge Level A 已完成；B2 workspace-write 真实 `new_session` 探针已执行成功，并经长期只读复核线复核后由主管线收口为 `accepted_with_deferred_items`。允许进入 J3 memory capture bus。本任务包本体仍是执行点冻结文件，不等于 Stage J 全部完成。

全局主管任务。本文接在 J2-A `accepted_with_deferred_items` 之后，用于把项目工作流自动编排 run units 从 Phase A no-op 推进到受控真实执行前的执行点冻结。J2-B 的目标不是裸控制台，而是证明“用户目标 -> run unit -> 统一 Product Command Phase B -> runtime / audit / readback -> worker report -> process fact observation”的产品链路可以在受控测试项目内真实跑通。

## 0. 先说薄弱点

- J2-A 已经补齐项目页产品入口和离线编排记录，但没有真实 Codex 执行。
- J1-B / PCR9 / H5-Level-B 证明过指定 session 的真实 `resume` 能跑，但它们不是 J2 run unit 自动编排闭环证据。
- 如果 J2-B 直接跑 `codex exec resume`，会绕过 J2 run unit、Product Command、runtime log、audit、readback 和 C5 回收。
- 如果 J2-B 一口气追求完整多角色自动闭环，容易把失败、readback unavailable 或人工手工操作包装成成功。
- 本任务包先冻结执行点；只有复核通过且主管线明确启动执行点后，才能执行真实 Codex。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`
- `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `evidence/2026-06-09-stage-j-j2-supervisor-acceptance-review-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

## 2. 用户授权和主管边界

用户已说明测试项目内的任何权限都可以给，包括全局主管自己建立的测试项目和 `mario test`。本任务包将该授权解释为“可以冻结并准备测试项目执行点”，不是裸授权。

真实执行仍必须满足：

- 任务包冻结对象、session / new-session strategy、sandbox、allowed write roots、denied paths、prompt summary/ref/hash、readback marker、baseline / rollback / cleanup。
- 长期只读复核线先审查本任务包，无 P0/P1 后才能进入执行点启动。
- 主管线明确启动 B1 或 B2 执行点。
- 执行线不得使用 legacy / H5 / direct CLI / MCP canvas run 冒充 J2-B 统一 product command 路径。

## 3. J2-B 总体目标

J2-B 最少分两段：

1. B1：`mario test` read-only run unit 真实 `resume` 探针，证明 J2 run unit 可以通过统一 Product Command Phase B 真实发送并读回。
2. B2：Stage J 隔离测试项目 workspace-write run unit 探针，证明自动编排链路能把允许路径内的写入结果回收到 workflow / C5 / observation。

J2-B 不做：

- 不开放任意目录自由执行。
- 不做 planned adapters 真实执行。
- 不做 provider credential store 或 model verification。
- 不做无确认自动 retry / stop / restart。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript 或 rollout。
- 不把 worker report、observation 或 candidate 自动写正式记忆。
- 不把 J2-B 完成说成 J3 / J4 / J5 / J6 或 Stage J 完成。

## 4. B1 Read-Only Run Unit Freeze

### 4.1 冻结对象

```text
project_label: mario test
project_root: /Users/yoyi/Documents/mario test
project_id: project:users-yoyi-documents-mario-test
workflow_id: workflow:users-yoyi-documents-mario-test:default
workflow_node_id: workflow:users-yoyi-documents-mario-test:default:node:codex-dev
run_unit_role: developer_execution
target_session_id: 019e798a-ac37-7771-b982-e38084fcd22e
adapter_id: codex-local
operation: resume
sandbox: read-only
readback_marker: J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09
```

B1 必须使用 J2 run unit / Product Command 证据链。历史 H5-Level-B1、PCR9 或 J1-B 的完成证据只能作为可行性参考，不能冒充本轮 J2-B evidence。

### 4.2 允许和禁止

允许：

- 对冻结的 `mario test` 开发线 worker session 执行一次真实 `resume`。
- 发送符合本任务包 prompt 合同的真实 prompt。
- 由 Codex CLI 对 `/Users/yoyi/.codex` 写入本次执行所需的最小原生状态。
- 在 `product-line` 下写本轮 product command attempt、continuation attempt、runtime log、audit/readback refs、evidence / handoff。
- 读取 `mario test` 四个核心文件 hash 用于 before / after 对比。

禁止：

- 修改 `/Users/yoyi/Documents/mario test` 项目文件。
- 读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 创建新 Codex session。
- 改用 H5 / legacy / direct CLI / MCP canvas run 作为完成证据。
- 自动 retry / stop / restart。
- 把 worker report 直接写成正式事实或正式记忆。

### 4.3 B1 Prompt 合同

prompt summary：

```text
J2-B mario test developer run unit read-only real resume probe.
```

prompt ref：

```text
workbench-managed:j2-b:mario-test:developer-run-unit:read-only:v1
```

prompt sha256：

```text
31c8ceb071804168e46a1d5b3d3accbded1539037472479649766d676672caa0
```

hash 口径：按下方 canonical prompt source 代码块内文本计算，包含最后一行后的单个换行，不包含代码块后的额外空行。

canonical prompt source：

```text
You are the codex-local developer run unit for Stage J / J2-B project workflow automation read-only closed-loop probe.

Scope:
- Project: /Users/yoyi/Documents/mario test
- Workflow: workflow:users-yoyi-documents-mario-test:default
- Run unit: developer_execution
- Operation: resume only
- Sandbox: read-only
- Marker: J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09

Rules:
- Do not modify files.
- Do not run commands.
- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.
- Reply only with the marker and a minimal structured worker report candidate.
```

## 5. B2 Isolated Workspace-Write Run Unit Freeze

### 5.1 冻结对象

```text
project_label: stage-j-j2-b-isolated-project
project_root: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project
project_id: project:stage-j-j2-b-isolated-project
workflow_id: workflow:stage-j-j2-b-isolated-project:default
workflow_node_id: workflow:stage-j-j2-b-isolated-project:default:node:codex-dev
run_unit_role: developer_execution
adapter_id: codex-local
sandbox: workspace-write
allowed_project_write_path: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md
readback_marker: J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09
```

B2 session strategy：

- 默认先冻结隔离项目和写入边界，不在本任务包内直接启动。
- 执行前必须在 B2 addendum 或执行任务包里重新确认 `target_session_id` 或 `new_session` strategy。
- 如果使用 `new_session`，必须复用已修补的 product runner command plan，并确认 `--skip-git-repo-check`、sandbox、cwd、prompt stdin、runtime log 和 readback 仍符合 H3.1/H3-B 边界。
- 如果无法冻结 target session 或 new-session readiness，B2 必须阻断，不能用 direct CLI 手工结果冒充。

### 5.2 隔离项目 baseline

本任务包创建并冻结最小隔离项目 fixture：

- `tmp/stage-j-j2-b-isolated-project/README.md`
  - sha256：`b21eda72c5261bb74eb8f6f8a5fed04036c7e2571cd13bb72353c9471208e908`
- `tmp/stage-j-j2-b-isolated-project/project-notes.md`
  - sha256：`c6c8fb4c0e688663a87b8cedf519ef5dc3ce7c3f3455f2add94a1f2642ca7c4d`

B2 成功后，上述 baseline 文件 hash 必须保持不变。唯一允许 worker 写入的项目文件是 `allowed_project_write_path`。

### 5.3 B2 Prompt 合同

prompt summary：

```text
J2-B isolated project developer run unit workspace-write real probe.
```

prompt ref：

```text
workbench-managed:j2-b:isolated-project:developer-run-unit:workspace-write:v1
```

prompt sha256：

```text
a1e3eb2285a75b30d0104f5bd032e3b4fdfc51111ff52949597ce78de5878bb0
```

hash 口径：按下方 canonical prompt source 代码块内文本计算，包含最后一行后的单个换行，不包含代码块后的额外空行。

canonical prompt source：

```text
You are the codex-local developer run unit for Stage J / J2-B project workflow automation workspace-write closed-loop probe.

Scope:
- Project: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project
- Workflow: workflow:stage-j-j2-b-isolated-project:default
- Run unit: developer_execution
- Operation: resume or new session only after task package authorization
- Sandbox: workspace-write
- Marker: J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09
- Allowed write path: /Users/yoyi/workspace/product-line/tmp/stage-j-j2-b-isolated-project/.workbench/stage-j/j2-b/developer-run-unit-write-probe.md

Rules:
- Only create or update the allowed write path.
- Do not modify source, docs, task, evidence, handoff, or other project files.
- Do not read secrets, auth tokens, .env files, keychain data, OAuth credentials, provider credentials, rollout data, or full transcripts.
- Reply only with the marker and a minimal structured worker report candidate listing the allowed file.
```

## 6. 必须走的产品链路

B1/B2 成功路径必须满足：

1. 以 J2 run unit 为源头创建或引用统一 Product Command。
2. Product Command `command_family` 必须为 `real_execution_product_command`。
3. source / metadata 必须能回链到 J2 automation plan、run unit、workflow、node、work item、task package 和 memory packet。
4. 真实执行只能来自 `run_real_execution_product_command_phase_b` 或其受控 wrapper。
5. 不能用 H5 dispatch、legacy dispatch、direct CLI、测试 helper、MCP canvas run 冒充。
6. 执行前必须记录 expected store revision / record version；revision conflict 必须阻断。
7. readback 必须只读取本次 attempt 的必要摘要或 last message，不读取完整 transcript。
8. readback unavailable / failed / timed out 必须保持 `result_count=null`，不得伪装成 0。
9. worker report 必须进入 C5 回收路径；process fact observation 仍不是正式记忆。

## 7. 执行前检查

执行前必须完成：

- 重新确认 J2-A checkpoint 已完成，且当前入口不是 `pending_review`。
- 长期只读复核线确认本任务包无 P0/P1。
- 重新计算 B1/B2 prompt sha256。
- 重新计算 B1 `mario test` 核心文件 baseline hash：
  - `/Users/yoyi/Documents/mario test/index.html`
  - `/Users/yoyi/Documents/mario test/styles.css`
  - `/Users/yoyi/Documents/mario test/game.js`
  - `/Users/yoyi/Documents/mario test/README.md`
- 重新计算 B2 隔离项目 baseline hash。
- 确认没有 duplicate running attempt；如有 active attempt，必须按 H4 duplicate guard 阻断。
- 确认真实执行权限记录为 `confirmed_by: "user"`。
- 确认不需要读取 full transcript；如果需要，停止。
- 确认不需要读取 secret/token/`.env`/keychain/OAuth/provider credential/rollout。

## 8. 失败分类

以下情况必须失败或阻断：

- 未走统一 Product Command。
- 使用 legacy / H5 / direct CLI / test helper 冒充。
- prompt hash 与 canonical source 不一致。
- 缺用户确认或 `confirmed_by != "user"`。
- target session / new-session strategy 未冻结。
- sandbox 与冻结值不一致。
- B1 写入项目文件。
- B2 写入 allowed path 以外的项目文件。
- 试图读取 secret / full transcript / rollout。
- readback unavailable / failed / timed out。
- result marker 不匹配。
- product command sidecar / continuation sidecar / runtime log JSON 损坏。
- store revision / record version 冲突。

## 9. 验收标准

J2-B B1 可接受为：

- 指定 `mario test` / 指定开发线 session 的一次 J2 developer run unit read-only 真实 `resume` 探针完成。
- 真实 prompt 已通过 J2 run unit 绑定的统一 Product Command Phase B 发送。
- 工作台 product command attempt、continuation attempt、runtime log、audit/readback refs、run unit refs 可追溯。
- readback marker 成功读回。
- `mario test` 核心文件 hash 前后一致。
- worker report 和 process fact 可回收。

J2-B B2 可接受为：

- 指定隔离测试项目 / 指定 run unit 的 workspace-write 真实执行探针完成。
- 只写 allowed project write path。
- baseline 文件 hash 前后一致。
- 写入结果可回收到 workflow / C5 / observation。

J2-B 不接受为：

- 任意项目无限制自由执行。
- 所有自动工作流场景完成。
- J3 记忆捕获总线完成。
- 记忆正式化自动完成。
- 自动 retry / stop / restart 完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- Stage J 完成。

## 10. 验证矩阵

执行点启动后必须记录：

- B1/B2 prompt hash 复算。
- B1/B2 baseline hash before / after。
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib project_workflow_automation`
- `cargo test --lib real_execution_command`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib codex_local_runner`
- `cargo test --lib`
- `cargo fmt -- --check`
- 真实执行 / 敏感路径扫描分类。

如果本轮只创建任务包、未执行真实 Codex，则不得把上述执行后验证写成已通过。

## 11. 分线职责

主管线：

- 维护本任务包、执行边界和 acceptance matrix。
- 派发长期只读复核线审查。
- 复核通过后决定是否启动 B1 执行点。
- B1 通过后再决定是否启动 B2 执行点。

执行线：

- 只有在主管线明确启动对应执行点后才运行。
- 必须走产品链路，不得直接 CLI。
- 必须记录所有成功 / 失败证据。

记忆线：

- 复核 process fact observation 边界。
- 确认 J2-B 不绕过 J3 memory capture bus 和 FormalMemory 确认链路。

复核线：

- 只读审查 P0/P1/P2。
- 检查真实执行是否绕过 Product Command。
- 检查 prompt / readback / baseline / denied paths 是否自洽。

## 12. 回交产物

本任务包：

- `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`

任务包创建记录：

- `evidence/2026-06-09-stage-j-j2-b-execution-point-freeze-task-package-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-execution-point-freeze-task-package-v1-result.md`

若执行并回收，应新增：

- `evidence/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-v1-result.md`

主管复核如单独记录，可新增：

- `evidence/2026-06-09-stage-j-j2-b-supervisor-acceptance-review-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-supervisor-acceptance-review-v1-result.md`

## 13. 主管复核结论

长期只读复核线已回交 J2-B execution point freeze 复核结论：带 P2 通过，无 P0/P1。

接受范围：

- B1 可以作为下一步优先启动对象，但必须先确认代码路径能把 J2 run unit 绑定到统一 Product Command Phase B。
- B1 仍不能使用 J1-B / PCR9 / H5-Level-B 历史结果冒充，必须产生新的 J2-B evidence。
- B2 当前只冻结隔离项目和写入边界；执行前仍必须补 addendum 或执行任务包，重新冻结 target session / new-session strategy。

主管线补充判断：

- 现有 J2-A `codex_control_for_unit` 使用 `sha256(run_unit_id:user_goal)` 作为 prompt hash，和 B1 canonical prompt hash 不同。
- 因此 B1 不应直接复用任意 J2-A command；需要最小 J2-B B1 execution bridge 将本任务包冻结的 prompt summary/ref/hash、session、sandbox 和 run unit refs 写入统一 Product Command，再进入既有 Phase A no-op + Phase B。

## 14. B1 Execution Bridge 完成记录

B1 execution bridge 已完成，记录见：

- `evidence/2026-06-09-stage-j-j2-b-b1-execution-bridge-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b1-execution-bridge-v1-result.md`

接受范围：

- 新增 Tauri command `run_project_workflow_automation_j2_b_b1`。
- Bridge 严格校验 B1 冻结字段，按 `preview -> prepare -> user decision -> Phase A -> Phase B` 串联统一 Product Command。
- 默认测试只走 fake runner，不执行真实 Codex，不发送真实 prompt，不读写 `/Users/yoyi/.codex`。
- `read-only` sandbox 允许 `allowed_write_roots=[]`；非 `read-only` sandbox 仍要求显式写根。

后续状态：

- B1 已执行并读回 marker，记录见 `evidence/2026-06-09-stage-j-j2-b-b1-real-project-workflow-automation-resume-probe-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b1-real-project-workflow-automation-resume-probe-v1-result.md`；长期只读复核线已审查通过，主管线收口记录见 `evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`。
- B2 execution bridge Level A 已完成，B2 workspace-write 真实 `new_session` 探针也已执行成功；记录见第 16 / 17 节。
- worker report candidate / C5 / observation 完整真实回收尚未完成。

## 15. B1 Real Resume Probe 完成记录

B1 read-only 真实 `resume` 探针已通过，记录见：

- `evidence/2026-06-09-stage-j-j2-b-b1-real-project-workflow-automation-resume-probe-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b1-real-project-workflow-automation-resume-probe-v1-result.md`

接受范围：

- 指定 `mario test` / 指定 session `019e798a-ac37-7771-b982-e38084fcd22e` 的 J2 developer run unit 真实 `resume` 探针完成。
- 真实 prompt 已通过 J2 run unit 绑定的统一 Product Command Phase B 发送。
- Readback marker `J2_B_MARIO_TEST_DEVELOPER_RUN_UNIT_READ_ONLY_OK_2026_06_09` 已读回。
- `mario test` 四个核心文件 hash 前后一致。
- `allowed_write_roots=[]`，`writes_project_files=false`。

主管复核：

- 长期只读复核线结论为带 P2 通过，无 P0/P1。
- 主管线接受 B1 为 `accepted_with_deferred_items`。
- 主管收口记录见 `evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`。

后续状态：

- B2 workspace-write 真实 `new_session` 探针已执行成功，并已由主管线收口为 `accepted_with_deferred_items`；记录见第 17 节。
- J3 memory capture bus 尚未完成。
- J2-B 不得冒领为 Stage J 完成。

## 16. B2 Execution Bridge Level A 完成记录

B2 execution bridge Level A 已完成，记录见：

- `evidence/2026-06-09-stage-j-j2-b-b2-execution-bridge-level-a-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b2-execution-bridge-level-a-v1-result.md`

接受范围：

- 新增 / 确认 Tauri command `run_project_workflow_automation_j2_b_b2`。
- Bridge 串联 J2 run unit、`codex_control` source、统一 `real_execution_product_command`、Phase A 和 new-session Phase B。
- Bridge 和 workflow audit event 将 `allowed_write_roots` 收窄为 `.workbench/stage-j/j2-b`；真实 harness 会预创建该目录但不创建 allowed file，并校验全项目文件 manifest before / after。
- 默认 fake-runner 测试验证 product command attempt、continuation attempt、runtime log、audit/readback refs 和 run unit refs。
- B2 真实 probe 仅保留 ignored / env-gated harness，默认测试不会执行真实 Codex。

## 17. B2 Real New-Session Write Probe 完成记录

B2 workspace-write 真实 `new_session` 探针已执行成功，记录见：

- `evidence/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1.md`
- `handoffs/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1-result.md`

接受范围：

- 指定 Stage J 隔离项目 / 指定 run unit 的 workspace-write 真实 `new_session` 探针已通过。
- 入口为 J2-B B2 bridge / env-gated real harness，非 H5 / legacy / direct CLI / MCP canvas run。
- 产品链路为 `J2 run unit -> codex_control -> real_execution_product_command -> Phase A -> new-session Phase B`。
- Phase B flags 显示 `prompt_sent=true`、`real_codex_executed=true`、`writes_codex_home=true`、`writes_project_files=true`。
- Readback marker `J2_B_ISOLATED_PROJECT_DEVELOPER_RUN_UNIT_WRITE_OK_2026_06_09` 已读回，`result_count=1`。
- 只写 allowed write path：`.workbench/stage-j/j2-b/developer-run-unit-write-probe.md`。
- `README.md` 和 `project-notes.md` baseline hash 保持冻结值。
- Prompt body 未持久化到 product command sidecar / continuation sidecar / runtime log / workflow state。

主管复核：

- 长期只读复核线结论为带 P2 通过，无 P0/P1。
- 主管线接受 B2 为 `accepted_with_deferred_items`。
- 主管收口记录见 `evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`。

仍未完成：

- worker report candidate / C5 / process fact observation / memory candidate 完整真实回收尚未完成。
- J3 memory capture bus 尚未完成。
- J2-B 不得冒领为任意项目无限制自由执行或 Stage J 完成。
