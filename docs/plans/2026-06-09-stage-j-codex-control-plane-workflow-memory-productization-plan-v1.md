# Stage J Codex Control Plane, Workflow Automation, And Memory Capture Productization Plan v1

日期：2026-06-09

状态：阶段 J 开发计划，已由 J6 最终验收冻结为 `accepted_with_deferred_items`。本文用于把中间版本已经完成的工作流、记忆层、runtime log / diagnostics、统一 Product Command Routing 组合成真正可用的产品能力。本文不是执行任务包，不授权直接执行真实 Codex，不授权直接读写 `/Users/yoyi/.codex`；当前完成事实以 J6 task / evidence / handoff、`CURRENT.md` 和 `tasks/README.md` 为准。

## 0. 先说薄弱点

- 现在不是“没有工作流”或“没有记忆层”。C1-C6、M1-M13、G1/G2、PCR0-PCR10 都已经给了可复用底座。
- 现在的核心缺口是产品化集成：用户还不能在工作台里自然地自由操控 `codex-local`，也不能把自由操作稳定绑定到项目工作流、运行日志、审计和记忆候选。
- 如果继续只做 guard、preview、stub 或只读面板，工作台会继续像“能解释风险但不能干活”的工具。
- 如果直接开放裸控制台，工作台会绕过项目、权限、任务包、记忆包、runtime log、audit、readback 和用户确认，变成危险的 Codex 外壳。
- Stage J 必须交付产品闭环，不是再证明一次真实执行可以跑。

## 1. 阶段目标

Stage J 的唯一交付目标：

```text
自由操控 Codex
+ 自动化工作流编排
+ 记忆层记录 / 分析 / 候选化
```

阶段完成后，中间版本应当支持：

- 用户在工作台内选择项目、会话、运行模式，输入任务并触发 `codex-local`。
- 用户可以选择 resume 既有 session，也可以在受控边界内创建新 session。
- 每次真实操作都必须走统一 Product Command，而不是旧 H5、legacy dispatch、direct CLI 或测试 probe。
- 每次操作都能绑定到项目、工作流、任务、角色或临时 run。
- 项目主管可以把用户目标拆成开发线 / 验证线 / 回收线等工作单元，并按授权范围自动编排。
- runtime log、audit、readback、worker report、project director process fact decision 能串成可追溯链路。
- 用户操作、prompt summary、执行结果、失败原因、worker report、用户确认和项目主管确认进入 observation / memory candidate 管道。
- 记忆层可以读取、分析和生成候选，但不能绕过确认权直接写正式记忆。

## 2. 已完成能力基线

Stage J 复用以下已完成能力，不重新发明：

- C1-C6：方案授权、用户确认、全局边界复核、项目主管拆任务、prepared dispatch、worker 汇报、过程事实确认、最终复核和用户结果决定。
- M1-M13：ObservationStore、MemoryCandidate、FormalMemory、TaskMemoryPacket、lint / conflict、lifecycle、知识库入口、实体关系、维护任务、成熟模式和最终权威验收。
- E1-E7：adapter descriptor、会话操作边界、provider availability、continuation preview、stub、runtime attention、阶段 E acceptance。
- F1-F5：项目工作流画布、节点详情、编辑 / 布局边界、画布边界硬化和阶段 F acceptance。
- G1-G5：runtime log、diagnostics、真实 Tauri 部分截图证据、端到端回放和中间版本 deferred freeze。
- H / I：`codex-local` runner、真实 resume / new-session 边界、多 agent / 多模型中立协议抽象。
- PCR0-PCR10：统一 Product Command Routing 已收口，真实执行必须归口统一 product command。

## 3. 阶段边界

Stage J 做：

- `codex-local` 的工作台内自由操控产品入口。
- 项目级真实执行绑定：项目、workflow、node、task package、memory packet、session。
- 自动化工作流编排：主管线 / 开发线 / 验证线 / 回收线的工作单元和回交流程。
- 操作和结果进入 memory observation / candidate 管道。
- 真实执行状态、失败、readback、权限、审计在 UI 中用用户可理解语言展示。
- 在 `mario test` 和一个隔离测试项目上做真实闭环验收。

Stage J 不做：

- 不接 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实执行。
- 不做 provider credential store 或 model verification。
- 不开放无限制任意目录读写。
- 不允许 agent 自治批准权限、自动写正式记忆或自动绕过用户确认。
- 不做无确认自动 retry / stop / restart。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential 或 rollout 放入记忆。
- 不把普通浏览器 smoke 当作真实 Tauri 验收。

## 4. 产品形态

Stage J 的用户可见产品形态应是“工作台主控台”，不是开发者调试面板。

主要入口：

- `项目`：项目内发起目标、查看工作流、查看运行结果和记忆候选。
- `智能体`：查看和选择 Codex session、resume / new session、执行状态和 readback。
- `运行中工作流`：查看当前执行、阻断、等待确认、失败和回收状态。
- `记忆`：查看本轮操作生成的 observation / candidate / formal memory 关系。
- `秘书`：解释下一步、风险、待确认项，不直接替用户批准高风险执行。
- `设置 / 开发者`：旧入口、legacy 状态、diagnostics、raw refs 和内部边界。

普通用户视图不能铺 raw enum、sidecar 字段、internal id、完整日志和开发者边界长文案；这些只进详情或设置开发者区。

## 5. 目标链路

```text
用户输入目标
-> 选择项目 / session / 运行模式
-> 生成 ProductCommand preview
-> 展示权限、记忆包、写入范围、readback 预期
-> 用户确认
-> codex-local 执行 resume 或 new session
-> runtime log / audit / readback
-> worker report candidate
-> 项目主管过程事实确认
-> Observation
-> MemoryCandidate
-> 用户或授权角色确认
-> FormalMemory / lifecycle
```

自动化工作流链路：

```text
用户目标
-> 项目主管生成计划
-> 开发线执行
-> 验证线检查
-> 回收线总结
-> 项目主管过程确认
-> 全局主管最终复核
-> 用户结果决定
-> 记忆候选 / 正式记忆
```

## 6. 核心对象

Stage J 不应新增一堆孤立对象，而应把已有对象连接起来。

### 6.1 CodexControlCommand

产品层的自由操控请求。必须包含：

- `control_command_id`
- `project_id`
- `project_root`
- `workflow_id`
- `workflow_node_id`
- `task_package_ref`
- `memory_packet_ref`
- `target_session_id`
- `session_mode`: `resume | new_session`
- `adapter_id`: 当前只允许 `codex-local`
- `prompt_summary`
- `prompt_ref`
- `prompt_hash`
- `allowed_write_roots`
- `denied_paths`
- `readback_plan`
- `requested_by`
- `created_at`

它应映射到已有 `RealExecutionProductCommand*`，不能绕开 PCR 链路。

### 6.2 WorkbenchRunUnit

用于把一次自由操作或自动派发绑定到工作流。

必须包含：

- `run_unit_id`
- `workflow_id`
- `role`: `director | developer | verifier | collector | secretary | user`
- `command_ref`
- `status`
- `runtime_log_refs`
- `audit_refs`
- `readback_ref`
- `worker_report_ref`
- `observation_refs`
- `memory_candidate_refs`

### 6.3 MemoryCaptureEvent

用于把工作台操作进入记忆管道。

必须包含：

- `capture_event_id`
- `source_type`: `user_action | product_command | runtime_log | readback | worker_report | process_fact_decision | final_review`
- `source_ref`
- `project_id`
- `workflow_id`
- `summary`
- `sensitivity`
- `candidate_policy`
- `created_at`

它不能直接写正式记忆，只能进入 observation 或 candidate。

## 7. 开发任务拆分

为避免碎片化，Stage J 只拆成六个大任务包。

### J0：阶段 J 权限、产品范围和验收矩阵冻结

类型：文档 / 只读复核。

目标：

- 冻结 Stage J 的交付目标和不做项。
- 冻结测试项目：至少 `mario test` 和一个隔离测试项目。
- 冻结真实执行授权方式、允许写入路径、禁止读取范围、失败降级和回滚规则。
- 冻结 UI 信息层级：普通用户、详情、开发者区的显示边界。
- 冻结验收矩阵：自由操控、自动编排、记忆记录三条链路必须同时验收。

验收：

- 任务包明确不授权直接真实执行。
- 复核线确认计划没有绕过 PCR0-PCR10。
- 权威入口只登记阶段计划，不声称 Stage J 已完成。

### J1：Codex Control Plane 自由操控入口

类型：后端 + 前端 + 离线测试。

状态：J1-A 已完成，结论为 `accepted_with_deferred_items`；J1-B 已完成指定 `mario test` / 指定 session 的 read-only 真实 `resume` 探针，结论为 `accepted_with_deferred_items`。J1-A 记录见 `tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`、`evidence/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1-result.md`；J1-B 记录见 `tasks/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`evidence/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1.md`、`handoffs/2026-06-09-stage-j-j1-b-mario-test-codex-control-real-resume-execution-point-v1-result.md`、`evidence/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j1-b-supervisor-acceptance-review-v1-result.md`。

目标：

- 在工作台提供真正可用的 `codex-local` 操作入口。
- 支持选择项目、选择既有 session、选择 `resume`。
- 支持输入任务，但存储只保留 prompt summary/ref/hash，prompt body 不进入普通 sidecar / runtime log。
- 支持生成 preview / permission envelope / memory packet summary / write roots / readback expectation。
- 支持用户确认后执行真实 resume。
- `new_session` 先接 readiness / preview；是否真实执行由 J2/J6 或单独 Level B 决定。

验收：

- J1-A 已完成：普通 UI 有可用入口，不再只是开发者边界面板；`codex_control` 作为 source 接入统一 Product Command preview / prepare / user confirmation / Phase A no-op trace；无确认不发送 prompt；prompt body 不持久化；普通 UI 不暴露裸 `codex exec -C`；Product Command 绑定项目 / workflow / work item / task package / memory packet refs。
- J1-B 已完成：`mario test` 上完成一轮 read-only 真实 resume 操作，readback 成功且 `result_count=1`；保留底层 continuation 历史命名债 P2。J1-B 不接受为 J2 自动编排、J3 记忆捕获或 Stage J 完成。

### J2：项目工作流自动编排闭环

类型：后端 + 项目页 / 运行中 UI + 真实测试。

状态：J2-A 已完成，结论为 `accepted_with_deferred_items`；J2-B 执行点冻结、B1 read-only 真实 `resume` 探针和 B2 workspace-write 真实 `new_session` 探针也已完成并经主管线收口，结论为 `accepted_with_deferred_items`。J2-A 任务包为 `tasks/2026-06-09-stage-j-j2-project-workflow-automation-run-units-and-controlled-closed-loop-v1.md`；J2-B 任务包为 `tasks/2026-06-09-stage-j-j2-b-controlled-real-workflow-automation-execution-point-freeze-v1.md`。J2-B 记录见 `evidence/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1.md`、`handoffs/2026-06-09-stage-j-j2-b-b1-supervisor-acceptance-review-v1-result.md`、`evidence/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-09-stage-j-j2-b-b2-supervisor-acceptance-review-v1-result.md`。J2-B 接受为指定 run unit 的受控真实执行探针完成；不接受为完整 C5 / observation / candidate 回收闭环、J3 记忆捕获总线、任意项目无限制自由执行或 Stage J 完成。

目标：

- 将用户目标绑定到项目工作流。
- 项目主管能生成开发线 / 验证线 / 回收线 run units。
- 每个 run unit 都能通过 J1 / Product Command 执行或进入等待授权状态。
- worker report 进入工作流账本。
- 项目主管过程事实确认后进入 observation。
- 全局主管复核和用户结果决定使用已有 C5/C6 链路。

验收：

- 至少一个测试项目跑完“用户目标 -> 开发线 -> 验证线 -> 回收线 -> 主管复核”的真实或受控混合闭环。
- 失败 / blocked / readback unavailable 不被包装成成功。
- 工作流画布和运行中入口能解释当前处于哪一步。

### J3：记忆层捕获总线和候选生成

类型：后端 + 记忆 UI + 离线测试。

状态：已完成，结论为 `accepted_with_deferred_items`。任务包见 `tasks/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md`，记录见 `evidence/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1.md` 与 `handoffs/2026-06-09-stage-j-j3-memory-capture-bus-and-candidate-generation-v1-result.md`。复核线初审发现 P1，主管线已修补；复审确认无 P0/P1。J3 不接受为 Stage J 完成。

目标：

- 把用户操作、ProductCommand、runtime log、readback、worker report、process fact decision、final review 接入 `MemoryCaptureEvent`。
- 明确哪些事件进入 ObservationStore，哪些只保留 audit / runtime log。
- 从明确来源生成 MemoryCandidate。
- 使用现有 memory lint / conflict / eligibility。
- 不自动写正式记忆；正式化仍走 M2 / M9 / M12 等确认链路。
- 左侧栏已核对 `想法箱 / 知识库 / 记忆层` 主入口和 inkwash 图标组；J3 未新增主导航入口。

验收：

- 等价 fixture 生成 capture event 后，记忆中心能看到对应 capture / observation / candidate / source refs；后续真实 Codex 操作回收仍需在 J4/J5 或单独执行点中复核。
- 任务记忆包能解释本轮使用了哪些正式记忆、排除了哪些候选或敏感材料。
- secret/full transcript/rollout 不进入记忆。

### J4：运行队列、失败控制和用户确认队列

类型：后端 + UI + 测试。

状态：已完成，结论为 `accepted_with_deferred_items`。任务包见 `tasks/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md`，记录见 `evidence/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1.md` 与 `handoffs/2026-06-09-stage-j-j4-run-queue-failure-control-and-user-confirmation-queue-v1-result.md`。复核线最终确认无 P0/P1/P2。J4 不接受为真实 retry / stop / restart、FormalMemory 自动写入、J5/J6 或 Stage J 完成。

目标：

- 统一展示 running / waiting_user / blocked_by_guard / failed / readback_unavailable / timed_out / duplicate_blocked。
- retry / stop / restart 只做受控产品动作，必须有用户确认和 audit。
- 不做无确认自动重试。
- 支持用户在待办中处理权限确认、重试确认、结果确认和记忆正式化确认。

验收：

- 已完成：同一个 workflow/node/session 的重复派发会被阻断或要求明确新 run。
- 已完成：readback unavailable 继续显示未知 / 不可用，不显示成 0。
- 已完成：stop/retry 不绕过 Product Command 和 audit，默认只进入确认事项和 read model。

### J5：UI 信息层级和真实 Tauri 产品验收

类型：前端 + 真实 Tauri 验收。

状态：已完成，结论为 `accepted_with_deferred_items`。任务包见 `tasks/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md`，记录见 `evidence/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1.md` 与 `handoffs/2026-06-09-stage-j-j5-ui-information-hierarchy-and-real-tauri-product-acceptance-v1-result.md`。真实 Tauri 关键截图探针见 `evidence/tauri-verification/2026-06-09-stage-j-j5/01-agent-workbench-tauri-window.png`。J5 接受为智能体页普通对话工作区、开发者内容默认折叠、左侧栏 inkwash 入口和关键截图探针完成；不接受为完整真实 Tauri UI 自动化验收、真实执行新增授权、自动 retry / stop / restart、FormalMemory 自动写入、planned adapters 真实接入或 Stage J 完成。

目标：

- 清理自由操控、运行中、工作流、记忆之间的信息层级。
- 普通用户看到“我要做什么、现在在做什么、需要我确认什么、结果是什么”。
- 开发者内容进入设置 / 开发者区。
- 真实 Tauri 下完成关键路径截图和手动验收。

验收：

- 左侧主入口路径清楚：项目 / 智能体 / 想法箱 / 知识库 / 记忆层 / Skill / Harness / 运行中工作流；设置固定在底部，开发者内容从设置进入。
- 左侧主入口图标沿用 inkwash 原型符号语言：想法箱 `✎`、知识库 `▢`、记忆层 `◐`、运行中工作流 `≋`。
- 真实 Tauri 覆盖 J1-J4 的关键路径。
- 没有把开发者 raw 状态铺到普通首屏。

### J6：Stage J 最终验收和后续路线冻结

类型：全局主管复核 + 文档。

状态：已完成，Stage J 最终结论冻结为 `accepted_with_deferred_items`。任务包见 `tasks/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md`，记录见 `evidence/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1.md` 与 `handoffs/2026-06-10-stage-j-j6-final-acceptance-and-roadmap-freeze-v1-result.md`。J6 接受为 Stage J 目标“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”的当前产品化 checkpoint 完成；不接受为最终蓝图完整工作台、任意目录无限制自由执行、planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart、所有操作自动写 FormalMemory 或完整真实 Tauri UI 自动化验收完成。

目标：

- 汇总 J0-J5。
- 冻结 acceptance matrix。
- 标记 accepted / deferred / blocked。
- 同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 决定下一阶段是否进入 planned adapters / provider credential / model verification。

验收：

- 自由操控 Codex、自动化工作流、记忆层记录三条链路都有证据。
- 不冒领最终蓝图。
- 不把 `codex-local` 能力泛化为所有 agent。

## 8. 并行分线

建议使用长期复用线程，不要每个小任务开新线。

### 主管线

- 维护阶段目标、任务边界、授权和 acceptance matrix。
- 决定何时允许真实执行。
- 接收开发线 / UI 线 / 记忆线 / 复核线回交。
- 不亲自抢开发线代码，除非是小型收口修补。

### 后端线

- J1 / J2 / J3 / J4 后端服务和 store / read model。
- Product Command 对接、run unit、memory capture event。
- Rust 测试和迁移边界。

### UI 线

- J1 / J2 / J4 / J5 用户界面。
- 项目、智能体、运行中、记忆、秘书的信息层级。
- 离线交互测试和真实 Tauri 验收准备。

### 记忆线

- J3 capture event 到 observation / candidate。
- 任务记忆包解释、lint、正式化确认链路复核。
- 防止自动写正式记忆。

### 复核线

- 只读审查 P0/P1/P2。
- 检查真实执行是否绕过 Product Command。
- 检查记忆是否绕过状态机。
- 检查 UI 是否冒领 planned adapter / provider / 自动重试。

## 9. 验收矩阵

| 能力 | 最低验收 |
| --- | --- |
| 自由操控 Codex | 用户在工作台内输入任务，选择项目和 session，确认后真实 `codex-local resume` |
| 新会话 | 至少完成受控 readiness / preview；真实 new-session 成功可作为 J6 deferred 或单独 Level B |
| 自动工作流 | 用户目标能生成 run units，并完成开发 / 验证 / 回收 / 主管复核链路 |
| 运行记录 | ProductCommand attempt、runtime log、audit、readback refs 可追溯 |
| 记忆记录 | 操作和结果进入 observation / candidate，不自动写正式记忆 |
| UI | 普通用户路径可用，开发者信息不铺首屏 |
| 安全 | 无确认不执行；secret/full transcript/rollout 不进入 prompt、日志或记忆 |
| 真实验收 | 至少 `mario test` + 隔离项目各一条闭环证据 |

## 10. 风险和缓解

| 风险 | 缓解 |
| --- | --- |
| 自由控制台绕过工作流 | 所有执行必须绑定 project/workflow/run unit 或显式 temporary run |
| prompt body 泄漏 | sidecar/runtime log 只存 summary/ref/hash，prompt body 只作为运行时输入 |
| 记忆层污染 | 操作先进入 observation/candidate，正式化必须确认 |
| 自动重试造成破坏 | J4 默认只做用户确认后的 retry，不做无确认自动重试 |
| planned adapter 被误解为可用 | UI 和测试继续保持 planned/unavailable/no credential/model unverified |
| 运行失败被包装成成功 | readback/failure/timed_out/blocked 状态必须保留，result_count unknown 用 null |
| 多线程上下文失控 | 长期分线，少开新线；checkpoint 才同步权威入口 |

## 11. 不得声称完成

Stage J 即使完成，也不得声称：

- Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。
- provider credential / model verification 已完成。
- 任意目录无限制自由执行。
- agent 可以自行批准权限。
- 所有操作自动变成正式记忆。
- 自动 retry / stop / restart 可以无确认执行。
- 最终蓝图完整工作台完成。

## 12. 开始 J1 前必须确认

- J0 任务包已完成。
- PCR10 结论仍为 `accepted_with_deferred_items` 且无 P0/P1。
- 测试项目和允许写入路径已冻结。
- 真实执行授权方式已冻结。
- 记忆 capture 的来源、敏感等级和候选策略已冻结。
- UI 普通用户 / 详情 / 开发者区显示边界已冻结。
