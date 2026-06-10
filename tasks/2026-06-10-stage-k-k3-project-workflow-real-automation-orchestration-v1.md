# Stage K / K3 Project Workflow Real Automation Orchestration v1

日期：2026-06-10

状态：随 Stage K final freeze 收口为 `accepted_with_deferred_items`。Level A 已完成并通过主管线 fresh verify；Level B 字段冻结、K3-B0 bridge / harness、K3-B1.0 prompt freeze repair、K3-B1 真实执行失败分类和 K3-B1.1 retry gate 均已完成；K3-B1 retry 申请再次被安全审查拒绝，K3-B2 仍不得启动。本文是 Stage K 的 K3 任务包，用于把 K2 已完成的通用 `codex-local` `resume` / `new_session` 产品入口，和 Stage J 已完成的项目工作流 run unit、memory capture、run queue / confirmation 能力，收敛成日常可用的“项目工作流真实自动化编排闭环”。K3 当前只接受为 Stage K 带 deferred 的项目工作流产品化 checkpoint，不接受为 K3-Level-B 完整真实闭环完成、K3-B1 retry 成功或 K3-B2 可开始。

Level A 记录见：

- `../evidence/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1.md`
- `../handoffs/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1-result.md`

本文不是 Stage K 严格无缺口完成声明，不授权 planned adapters，不授权 provider credential / model verification，不授权无确认 retry / stop / restart，不授权任意目录无限制执行。

## 0. 全局主管理解

已知事实：

- K0 已冻结 Stage K 范围、权限、测试项目和验收矩阵。
- K1 已把智能体页普通层收敛为日常对话工作区，但不代表工作流真实编排完成。
- K2 已完成通用 `codex-local` `resume` / `new_session` 产品入口、Phase A 非真实预检、Phase B 普通入口接线，以及 R1/R2/N1/N2 四个受控真实执行点；K2 结论为 `accepted_with_deferred_items`。
- Stage J / J2 已有项目工作流自动编排 run unit 和 J2-B B1/B2 真实执行探针；J3 已有 memory capture bus；J4 已有 run queue / confirmation / failure control；这些是 K3 的复用基础，不是 K3 自动完成证据。
- 用户目标是“自由操控 Codex + 自动化工作流 + 记忆层记录”，K3 负责其中“自动化工作流真实派发闭环”的产品化部分。

当前缺口：

- J2/J3/J4 仍偏 checkpoint / 探针语义；K3 要把它们变成 Stage K 的日常产品链路。
- 项目工作流需要从用户目标生成可执行 run units，并把 run unit 的真实执行结果回收到 worker report、process fact、run queue 和用户可读总结。
- K2 能执行单次 Codex 操作，但还没有把多个 run units 组织成项目级闭环。

关键风险：

- 如果 K3 直接调 CLI，会绕过 Product Command、runtime log、audit、readback 和记忆层。
- 如果 K3 把 `planned` / `prepared` / `waiting_user` 显示为“正在执行”或“已完成”，会误导用户。
- 如果 K3 为了自动化而让 agent 自治批准权限，会破坏工作台治理边界。
- 如果 K3 将 worker report、observation 或 candidate 直接写成 FormalMemory，会绕过记忆层确认权。
- 如果 K3 保存 prompt body、完整 transcript、raw stdout / stderr 或 secret-like 内容，会破坏 K2/K4 的安全边界。

## 1. 权威依据

必须服从：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `tasks/2026-06-10-stage-k-k0-scope-permission-and-acceptance-matrix-freeze-v1.md`
- `tasks/2026-06-10-stage-k-k1-agent-conversation-workspace-daily-use-refactor-v1.md`
- `tasks/2026-06-10-stage-k-k2-general-codex-resume-new-session-product-entry-v1.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`
- `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`
- `tasks/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`
- `tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

必须优先复用：

- K2：通用 Product Command `resume` / `new_session` preview、prepare、decision、Phase A、Phase B、permission envelope、readback。
- J2：`ProjectWorkflowAutomationPlan` / `ProjectWorkflowRunUnit` / J2-A closed loop / J2-B B1/B2 bridge。
- J3：`MemoryCaptureEvent`、observation、MemoryCandidate source refs。
- J4：run queue、user confirmation queue、failure control summary。
- C5/C6：worker structured report、project director process fact decision、global final review、user result decision。
- M2/M6/M9/M12：candidate 到 FormalMemory 的确认、任务记忆包、生命周期和 mature pattern gate。
- PCR10：真实执行必须归口统一 Product Command，旧 H5 / legacy / direct CLI 不可作为 K3 完成证据。

## 2. K3 目标

K3 要交付：

1. 用户在项目页输入项目目标后，工作台能生成项目工作流 run units。
2. run units 至少覆盖：`director_plan`、`developer_execution`、`verifier_check`、`collector_summary`、`director_final_review`。
3. 每个 run unit 都有项目、workflow、node、work item、task package、memory packet、Product Command、runtime log、audit、readback、worker report、capture / observation / candidate refs。
4. 开发 run unit 能通过 K2 Product Command 执行真实 `codex-local`。
5. 验证 run unit 至少能以 read-only 或 fixture 方式形成可审查结果；如执行真实 Codex，也必须走 K3 真实执行点授权。
6. 回收 run unit 能生成用户可读摘要，不把 worker report 当正式事实或正式记忆。
7. 项目主管能确认 process fact；该确认可以形成 observation / candidate 来源，但不能自动写 FormalMemory。
8. 项目页和运行中工作流页能显示每个 run unit 的用户可读状态。
9. 失败 / readback unavailable / duplicate / permission denied 进入待办或失败状态，不伪装成功。
10. K3 完成后，允许 K4 消费 K3 事件来源推进记忆捕获体验。

## 3. K3 非目标

K3 不做：

- 不接 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实执行。
- 不做 provider credential store、真实 token 读取或 model verification。
- 不开放任意目录无限制执行。
- 不允许 agent 自治批准权限。
- 不做无确认自动 retry / stop / restart。
- 不 kill Codex 进程。
- 不自动写 FormalMemory。
- 不保存完整 prompt body、完整 transcript、raw stdout、raw stderr、rollout 或 secret-like 内容。
- 不把普通浏览器 smoke 当作真实 Tauri 验收。
- 不把 K3 完成说成 K4/K5/K6 或 Stage K 完成。

## 4. 分阶段执行

K3 采用合并型 checkpoint，内部按 Level A / Level B / Level C 回收，避免拆成过多维护成本高的小任务。

### K3-Level-A：产品闭环和非真实验证

Level A 允许改产品代码，但不允许执行真实 Codex，不允许发送 prompt，不允许读写 `/Users/yoyi/.codex`。

必须完成：

- 将 Stage J `project_workflow_automation` 的 J2 特定实现收敛为 Stage K 可复用产品路径，或新增等价 K3 application service。
- 支持用户目标生成五类 run units，并生成 deterministic IDs。
- 每个 run unit 至少绑定：
  - `project_id`
  - `project_root`
  - `workflow_id`
  - `workflow_node_id`
  - `work_item_id`
  - `run_unit_kind`
  - `role`
  - `task_package_ref`
  - `memory_packet_ref`
  - `product_command_ref` 或 `product_command_preview_ref`
  - `runtime_log_refs`
  - `audit_refs`
  - `readback_ref`
  - `worker_report_ref`
  - `capture_event_refs`
  - `observation_refs`
  - `memory_candidate_refs`
- 开发 run unit 能生成 K2 Product Command preview / prepare / user decision / Phase A no-op。
- Level A flags 必须保持：
  - `prompt_sent=false`
  - `real_codex_executed=false`
  - `writes_codex_home=false`
  - `writes_project_files=false`
- worker report fixture 可回收到 C5。
- process fact decision 可形成 observation source，但不能自动写 FormalMemory。
- 运行中工作流页、项目页、智能体页只显示用户可理解的 run unit 状态摘要；raw refs 进入开发者详情。
- `readback_unavailable` / `readback_failed` / `timed_out` / `blocked_by_guard` 的 `result_count` 保持 `null`，显示为“未知 / 不可用”，不能显示为 0。

### K3-Level-B：受控真实执行闭环

Level B 必须等 K2.5 architecture gate 已通过、Level A fresh verify 通过、主管线明确启动后才能执行。

Level B 建议只做两个真实执行点：

1. `K3-B1 mario test read-only workflow loop`
   - 目标：证明项目工作流开发 run unit 能对 `mario test` 走真实 `resume/read-only`，并回收到 run unit / runtime / audit / readback / worker report。
   - 项目：`/Users/yoyi/Documents/mario test`
   - 写入：不允许写项目文件。
   - baseline：执行前后记录 `index.html`、`styles.css`、`game.js`、`README.md` hash。
   - 成功标准：readback marker 成功，核心文件 hash 不变，run unit 状态进入 `completed_needs_review` 或等价待主管确认状态。

2. `K3-B2 isolated project workspace-write workflow loop`
   - 目标：证明项目工作流开发 run unit 能对 Stage K 隔离项目走真实 `new_session` 或指定 `resume/workspace-write`，只写 allowed path，并回收到 worker report / process fact / capture source。
   - 项目：`/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project`
   - 写入：只允许写任务包冻结的 `.workbench/stage-k/k3/` 子路径。
   - baseline：执行前后记录隔离项目 manifest 和核心文件 hash。
   - 成功标准：readback marker 成功，除 allowed path 外无新增 / 修改，run unit refs 可追溯。

Level B 禁止：

- 使用 H5 / legacy / direct CLI / MCP canvas / test helper 冒充 Product Command Phase B。
- 复用 K2 的真实执行授权直接跑 K3；每个 K3 执行点必须单独列字段工作表。
- 自动 retry。
- 因 readback 失败而手动改 evidence 包装成成功。

### K3-Level-C：主管验收和 checkpoint 回收

Level C 不新增真实执行。

必须完成：

- 汇总 Level A / Level B evidence 和 handoff。
- 复核 run unit refs、runtime log、audit、readback、worker report、process fact、capture source 的闭环。
- 长期复核线只读复核无 P0/P1 后，K3 可收口为 `accepted` 或 `accepted_with_deferred_items`。
- K3 收口时同步 `CURRENT.md`、`tasks/README.md`、`README.md`、`STAGE_PLAN.md`、`AUTHORITY.md` 和 Stage K plan。

## 5. 真实执行点字段工作表

K3 的任何真实执行点执行前必须填完整下表。缺一项即阻断。

| 字段 | 要求 |
| --- | --- |
| `execution_point_id` | 全局唯一，例如 `stage-k-k3-b1-mario-test-workflow-read-only` |
| `operation` | `resume` 或 `new_session` |
| `adapter_id` | Stage K 只能是 `codex-local` |
| `project_root` / `project_id` | 绝对路径和稳定 project id |
| `workflow_id` / `run_unit_id` / `node_id` | 必须填写，且能在 run model 中回链 |
| `target_session_id` | `resume` 必填；`new_session` 必须写明记录新 session id 的方式 |
| `sandbox` | `read-only` 或 `workspace-write` |
| `allowed_write_roots` | 必须绝对路径且尽量窄；read-only 为空数组 |
| `allowed_write_path` | workspace-write 必须列唯一文件或最小目录 |
| `denied_paths` | 必须包含 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript、rollout 和未授权项目路径 |
| `prompt_summary` / `prompt_ref` / `prompt_hash` | 必须填写；prompt body 只作为运行时输入，不持久化 |
| `task_memory_packet_ref` | 必须说明 included / excluded / review materials；无 included memories 也要说明 |
| `permission_envelope_ref` | 必须有用户可读影响范围和确认记录 |
| `readback_plan` | 必须说明 expected marker、last message、失败分类和 `result_count=null` 规则 |
| `runtime_log_policy` | 必须写 runtime log summary，不写 prompt body/raw output |
| `audit_policy` | 必须写 preview/decision/attempt/readback/process fact audit refs |
| `baseline_hashes` | 必须记录执行前后 hash 或 manifest |
| `.codex_scope` | 只允许 runner 必要的最小 Codex session state；不得读取 secret/full transcript/rollout |
| `dirty_worktree_policy` | 不回退用户改动；只核对冻结文件和 allowed path |
| `rollback_policy` | workspace-write 必须说明 allowed file 保留或清理策略 |
| `user_confirmation` | 高影响真实执行必须 `confirmed_by: "user"` |

## 6. UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增普通用户主导航入口。

普通 UI 应显示：

- 用户目标。
- 自动编排阶段：计划、开发、验证、回收、复核。
- 每个 run unit 当前状态。
- 是否等待用户确认。
- 是否真实执行、是否读回、是否失败。
- worker report 摘要。
- process fact / observation / candidate 摘要。
- 下一步建议。

普通 UI 禁止默认显示：

- 裸 CLI 命令。
- raw Product Command JSON。
- sidecar 绝对路径。
- internal id 长列表。
- prompt body。
- full transcript / rollout。
- raw stdout / stderr。
- secret / credential。
- “自动执行已完成”“Codex 已收到任务”“worker 正在执行”等无真实 attempt 支撑的文案。
- “已正式记忆”“自动记住”之类绕过确认的文案。

显示位置：

- `项目`：项目工作流侧栏 / 节点详情显示 K3 run units、状态和结果摘要。
- `运行中工作流`：显示 run units、等待确认、阻断、失败和 readback unknown。
- `智能体`：显示与当前项目 / session 相关的工作流执行状态，不重复铺完整管理后台。
- `记忆层`：K3 只显示 capture / observation / candidate 来源摘要；正式化体验归 K4。
- `设置 > 开发者`：raw refs、diagnostics、sidecar 路径、Product Command ids。

本任务不做手机端 UI，不新增 mobile responsive 规则。

Xuanji UI 参考只用于后续信息架构判断：可学习其运行态“聊天 + 任务 + 执行流 + 文件上下文 + 记忆 / 权限”的信息组织方式；不得采用其视觉风格，不复制源码、命名、图标、品牌资产或具体实现。

## 7. 后端改动范围

允许改：

- `prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_capture_bus.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/worker_protocol.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

默认不改：

- FormalMemory schema。
- provider / credential store。
- planned adapter 真实执行逻辑。
- legacy / H5 真实 runner 产品入口。
- `workflow-state.v0.json` 顶层结构，除非只向既有数组追加 refs 且有测试覆盖。

数据写入边界：

- Level A 允许写工作台自有 workflow state、product command sidecar、continuation sidecar、runtime log、audit refs、memory capture / observation / candidate sidecar。
- Level A 不允许写 `/Users/yoyi/.codex`，不允许发送 prompt，不允许写测试项目业务文件。
- Level B 允许真实 runner 必要写 `/Users/yoyi/.codex`，但必须按字段工作表说明范围；项目写入只限 allowed path。

## 8. 前端改动范围

允许改：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/runQueue.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/RunningWorkflowsView.tsx`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`

默认不改：

- 主导航结构。
- 设置 / 开发者归档规则。
- 记忆中心正式化流程。
- 手机端 / mobile responsive 规则。

## 9. 多会话分工

全局主管线：

- 冻结 K3 任务包。
- 派发工作流 / 执行 / UI / 验证线。
- 复核所有回交，防止旧 H5 / legacy / direct CLI 冒充 K3。
- 只在 K3 checkpoint 完成、阻断或边界变化时同步权威入口。

工作流线：

- 负责 run unit 生成、状态机、worker report、process fact、final review 回链。
- 默认不执行真实 Codex。
- 不改智能体页大布局。

执行线：

- 负责 K3 run unit 到 K2 Product Command 的 bridge、permission envelope、Phase A / Phase B、readback、runtime/audit。
- 真实执行只在 Level B 字段工作表和主管授权后进行。
- 不改 UI 信息架构。

UI 线：

- 负责项目页、运行中工作流、智能体页的 K3 状态展示。
- 普通层只展示用户可理解状态；开发者信息后撤。
- 不改后端 runner 或真实执行语义。

记忆线：

- 负责 K3 执行结果进入 capture / observation / candidate source refs。
- 不做 FormalMemory 自动写入；正式化体验归 K4。

验证线：

- 只读复核任务包、代码、evidence/handoff、误导文案、敏感路径和真实执行边界。
- 不改文件，不执行真实 Codex，除非主管另行明确授权它作为执行验证线。

## 10. 测试矩阵

Level A 必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib project_workflow_automation`
- `cargo test --lib real_execution_command`
- `cargo test --lib session_continuation`
- `cargo test --lib runtime_log`
- `cargo test --lib memory_capture`
- `cargo test --lib worker_protocol`
- `cargo test --lib workflow_authorization`
- `cargo test --lib formal_memory`
- `cargo test --lib memory_candidate`
- `cargo test --lib`
- `cargo fmt -- --check`

Level A 必须新增或覆盖测试：

- 用户目标生成五类 run units。
- run unit 绑定 project / workflow / node / work item / task package / memory packet。
- run unit Product Command preview / prepare / Phase A 不发送 prompt、不执行真实 Codex。
- duplicate run 或 duplicate active attempt 被阻断或要求新 run。
- blocked / failed / readback unavailable / timed out 保持 `result_count=null`。
- worker report fixture 可回收为 C5 report。
- process fact 可写 observation source，但不生成 FormalMemory。
- 普通 UI 不显示 raw refs、sidecar path、prompt body、full transcript、自动正式记忆文案。

Level B 必须通过：

- 精确 ignored / env-gated 真实执行测试，不允许 broad `--ignored`。
- B1 `mario test` read-only 文件 hash 前后一致。
- B2 隔离项目 workspace-write 只写 allowed path。
- Product Command attempt、continuation attempt、runtime log、audit、readback、run unit refs 可交叉验证。
- readback marker 成功；失败则按 failed / unavailable 回收，不能包装成功。
- prompt body 不落 Product Command sidecar、continuation sidecar、runtime log、memory capture、evidence/handoff 正文。

建议扫描：

- 误导文案：`自动执行已完成|Codex 已收到任务|worker 正在执行|已正式记忆|自动记住|readback.*0 条`
- 真实执行绕路：`Command::new\\(\"codex\"\\)|codex exec|codex exec resume|run_workflow_machine|execute_workflow_node_dispatch`
- 敏感材料：`secret|token|\\.env|keychain|OAuth|provider credential|full transcript|rollout|prompt_body`

命中必须分类，不能用“无命中”替代分类说明。

## 11. Evidence / Handoff 要求

Level A evidence 必须记录：

- 改动文件。
- run unit 生成和 refs。
- Product Command Phase A no-op flags。
- worker report / process fact / observation source。
- UI 信息层级。
- 验证命令。
- 未执行真实 Codex、未读写 `/Users/yoyi/.codex` 的边界。

Level B evidence 必须记录：

- 每个真实执行点字段工作表。
- 用户确认记录。
- 执行前后 hash / manifest。
- Product Command id / attempt id。
- continuation id。
- runtime log refs。
- audit refs。
- readback status / marker / result_count。
- worker report / process fact refs。
- allowed file hash。
- prompt non-persistence 扫描。
- `.codex` 副作用范围说明。
- 失败 / deferred / cleanup。

K3 总 handoff 必须说明：

- K3 接受为什么。
- K3 不接受为什么。
- 是否允许进入 K4。
- 哪些 P2 延后到 K5/K6 或 K-UI checkpoint。

## 12. 回交格式

开发线回交必须包含：

- 改动文件。
- 完成的 Level。
- 是否真实执行。
- 是否发送 prompt。
- 是否写 `/Users/yoyi/.codex`。
- 是否写项目文件，写了哪些 allowed path。
- Product Command / runtime / audit / readback / worker report / memory capture refs。
- 验证命令和结果。
- 不可声称事项。
- 需要主管决策的问题。

复核线回交必须包含：

- P0/P1/P2。
- 关键证据行号。
- 是否允许进入下一 Level。
- 是否允许 K3 收口。
- 边界确认。

## 13. 完成定义

K3 原始最低完成定义：

- Level A 产品闭环和非真实验证完成。
- 至少一个 `mario test` read-only 项目工作流真实 run unit 通过。
- 至少一个隔离项目 workspace-write 项目工作流真实 run unit 通过，且只写 allowed path。
- 项目页和运行中工作流页能显示 run unit 状态。
- worker report、process fact、runtime log、audit、readback、capture source refs 可追溯。
- 失败或 readback unknown 不显示为成功或 0 条结果。
- 复核线无 P0/P1。

当前实际收口：

- Level A、字段冻结、K3-B0、K3-B1.0 和 K3-B1.1 已完成。
- K3-B1 已真实执行但失败分类为 `failed_classified_codex_state_readonly`；retry 申请再次被安全审查拒绝。
- K3-B2 依赖 K3-B1 成功和复核，当前不得启动。
- K3 随 K6 final / Stage K acceptance freeze 收口为 `accepted_with_deferred_items`，不冒领为原始最低完成定义全部满足。

K3 可接受为 `accepted_with_deferred_items` 的后置项：

- K3-B1 retry 成功。
- K3-B2 isolated workspace-write 项目工作流真实 run unit 通过。
- 多个真实项目的完整多 run unit 真实并行执行。
- 完整真实 Tauri 截图验收。
- 自动 retry / stop / restart。
- planned adapters。
- provider credential / model verification。
- FormalMemory 正式化体验。

K3 不可接受为：

- K3-Level-B 完整真实闭环完成。
- K3-B1 retry 成功。
- K3-B2 可开始。
- K4 记忆捕获体验完成。
- K5 失败恢复 / 操作控制完成。
- K6 dogfood 完成。
- Stage K 严格无缺口完成。
