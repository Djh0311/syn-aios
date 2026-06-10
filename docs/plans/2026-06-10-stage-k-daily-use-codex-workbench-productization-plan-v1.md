# Stage K Daily-Use Codex Workbench Productization Plan v1

日期：2026-06-10

状态：Stage K 已收口为 `accepted_with_deferred_items`。本文用于把已完成的 Stage J checkpoint 推进到“日常可用工作台”：自由操控 Codex、自动化工作流编排、记忆层记录 / 分析 / 候选化。K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0 bridge / harness、K3-B1.0 prompt freeze repair、K3-B1.1 Codex state permission / retry gate、Stage K architecture calibration v2/v3 and gate、K4 非真实记忆产品化切片、K5 非真实运行 / 待办 / 失败恢复和操作控制切片、K6 真实 Tauri dogfood 核心入口验收均已完成。K6 final 记录见 `../../evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。Stage K 不接受为严格无缺口完成：K3-B1 已执行但失败分类，K3-B1 retry 申请再次被安全审查拒绝；如恢复 K3-B1/K3-B2 真实执行线，只能由用户手动执行 exact command 并回交，或用户明确重新批准风险后主管线再次申请；不得直接进入 K3-B2。本文不是执行任务包，不授权直接执行新的真实 `codex exec` / `codex exec resume`，不授权直接读写 `/Users/yoyi/.codex`，不替代 `CURRENT.md`、`STAGE_PLAN.md` 或 task / evidence / handoff 的完成事实。

补充 UI 参考登记：用户已提供 Xuanji UI 研究资料作为 Stage K 后续 UI 信息层级参考。使用范围仅限学习信息架构、布局模式和内容层级；不采用 Xuanji 的视觉风格，不复制其源码、命名、图标、品牌资产或具体实现。参考入口为 `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/research/xuanji-ui-design-extraction-report.md` 和 `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/research/xuanji-ui-source-snapshot-2026-06-10/README.md`。合适落点为 K5/K6 或单独 K-UI 信息层级 checkpoint。

## 0. 先说薄弱点

- 现在不是“没有底座”。C1-C6、M1-M13、E/F/G、H/I、PCR10 和 Stage J 已经给出了受控工作流、记忆层、真实执行归口和 Product Command 链路。
- 现在也不是“最终可用”。普通用户仍会在界面里看到过多开发者边界、运行细节和内部术语；智能体页还没有完全成为顺手的对话工作区。
- 真实 Codex 执行已经被证明可行，但当前仍偏“授权探针 / checkpoint 证明”，不是稳定日常入口。
- 自动化工作流已经能编排和记录，但还需要把“用户目标 -> Codex worker 执行 -> readback -> 过程事实 -> 记忆候选”做成可重复产品链路。
- 记忆层已经完整到当前 checkpoint，但用户体验还偏治理后台，不够像“工作台自动帮我沉淀上下文”。
- 如果下一阶段继续拆很多小 guard / stub / 只读面板，会继续消耗文档维护成本，产品体验不会明显变好。
- 如果下一阶段直接开放裸 Codex 控制台，会绕过项目、权限、运行日志、审计、readback 和记忆层，破坏工作台存在的意义。

## 1. 阶段目标

Stage K 的目标固定为：

```text
把 Stage J 的受控产品化 checkpoint 推进为日常可用工作台。
```

更具体地说，Stage K 完成后应当做到：

- 用户能在工作台中选择项目、选择或创建 Codex 对话，直接输入任务。
- 发送前能看懂影响范围：项目、目录、写入权限、记忆注入、readback 预期、审计影响。
- 用户确认后，工作台能走统一 Product Command 真实调用 `codex-local`。
- Codex 执行结果能进入运行队列、运行日志、审计、readback 和用户可读摘要。
- 项目工作流能从用户目标生成 run units，并按授权范围派发给 `codex-local`。
- 工作流执行过程能生成 observation 和 memory candidate，用户可以确认、拒绝、编辑或推迟。
- 普通 UI 以任务和对话为中心，开发者信息默认后撤到详情或设置。

## 2. 当前能力基线

Stage K 复用以下已完成能力，不重新造底座：

- `C1-C6`：方案授权、用户确认、全局边界复核、项目主管拆任务、prepared dispatch、worker 汇报、过程事实确认、最终结果复核。
- `M1-M13`：正式记忆、候选记忆、observation、任务记忆包、冲突 / lint、生命周期、实体关系、维护任务、成熟模式、跨项目记忆和最终验收。
- `E1-E7`：adapter descriptor、会话操作边界、provider availability、continuation preview、runtime attention、阶段 E 验收。
- `F1-F5`：项目工作流画布、节点详情、编辑 / 布局边界、画布边界硬化和阶段 F 验收。
- `G1-G5`：runtime log、diagnostics、真实 Tauri 部分截图证据、端到端回放和中间版本 deferred freeze。
- `H/I`：`codex-local` runner、真实 resume / new session 边界、多 agent / 多模型中立协议抽象。
- `PCR10`：统一 Product Command Routing，真实执行必须归口 Product Command，旧入口 legacy / guarded。
- `Stage J`：Codex Control Plane、项目工作流 run units、真实执行点、记忆捕获、运行队列 / 用户确认 / 失败控制、UI 信息层级 checkpoint。

## 3. 阶段边界

Stage K 做：

- 把智能体页改成真正的 Codex 对话工作区。
- 把通用 `resume` 和 `new session` 变成稳定产品入口。
- 把项目工作流真实派发做成可重复闭环。
- 把运行队列、待确认、失败恢复和 readback 状态做成用户能理解的操作面。
- 把工作台操作和 Codex 结果接入 observation / memory candidate。
- 把开发者 / 内部信息移到设置或默认折叠详情。
- 做真实 Tauri dogfood 验收，至少覆盖 `mario test`、工作台自身项目、一个隔离测试项目。

Stage K 不做：

- 不接 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实执行。
- 不做 provider credential store 或 model verification。
- 不开放任意目录无限制读写。
- 不让 agent 自治批准权限。
- 不让任何链路自动绕过用户确认写正式记忆。
- 不默认自动 retry / stop / restart。可以设计入口和确认流，但真实操作必须单独验收。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential 或完整 rollout 写入记忆。
- 不把普通浏览器 smoke 当作真实 Tauri 验收。

## 4. 产品形态原则

Stage K 的产品形态是“桌面工作台”，不是“任务包管理器”，也不是“开发者控制中心”。

普通用户主入口：

- `项目`：从项目目标开始，查看工作流和结果。
- `智能体`：像 Codex 一样对话，选择项目、选择对话、输入任务、确认后执行。
- `运行中工作流`：看正在做什么、哪里卡住、是否需要确认。
- `记忆层`：看候选、正式记忆、来源、影响和确认事项。
- `知识库`：看资料、笔记、引用和候选生成入口。
- `想法箱`：收纳尚未进入项目或任务的想法。
- `Skill / Harness`：保留为能力和运行器入口，但首屏必须对象化表达，内部字段进入开发者详情。
- `设置`：开发者内容、diagnostics、旧入口、raw refs、adapter / provider 细节。

普通 UI 不应默认展示：

- `Product Command`
- `runtime log`
- `audit refs`
- `readback enum`
- `sidecar`
- `store revision`
- `raw ids`
- `H/J/PCR` 阶段术语
- `planned adapter boundary` 长文案

这些内容可以存在，但默认放进“开发者详情”或“设置 > 开发者”。

## 5. 目标链路

### 5.1 单次 Codex 操作链路

```text
选择项目
-> 选择 / 新建 Codex 对话
-> 输入任务
-> 生成发送预览
-> 展示权限、写入范围、记忆注入、readback 预期
-> 用户确认
-> Product Command
-> codex-local resume / new session
-> runtime log / audit / readback
-> 用户可读结果摘要
-> observation / memory candidate
```

### 5.2 自动化工作流链路

```text
用户目标
-> 项目主管生成计划
-> run units
-> 开发线执行
-> 验证线检查
-> 回收线总结
-> 项目主管过程事实确认
-> 全局主管最终复核
-> 用户结果决定
-> 记忆候选 / 正式记忆
```

### 5.3 记忆链路

```text
用户操作 / Codex 执行 / worker report / readback / final review
-> capture event
-> observation
-> memory candidate
-> 用户或授权角色确认
-> formal memory
-> task memory packet recall
```

## 6. 开发工作方式

Stage K 不再按非常细的小任务滚动推进。默认拆成 7 个 checkpoint，每个 checkpoint 内可以有开发线并行，但入口文档只在 checkpoint 完成、阻断或阶段边界变化时同步。

全局主管职责：

- 冻结每个 checkpoint 的目标、边界、权限和验收口径。
- 分配开发线，避免写集冲突。
- 审查是否绕过 Product Command、runtime log、audit、readback、memory candidate。
- 复核真实执行授权是否明确。
- 合并回交后做最终验收，不用每个小补丁都同步所有权威入口。

开发线建议：

- `UI 线`：智能体页、运行中工作流、记忆层、设置开发者区的信息层级和交互。
- `Execution 线`：`codex-local` resume / new session、permission envelope、readback、failure classification。
- `Workflow 线`：项目目标、run units、worker handoff、项目主管过程确认、最终回收。
- `Memory 线`：capture event、observation、candidate、formal memory UX、task memory packet 注入。
- `Validation 线`：离线测试、Rust 测试、真实 Tauri 截图、dogfood 矩阵、越界扫描。

多会话协作规则：

- 同一条开发线尽量复用同一个对话，不要每个小问题新建线程。
- 并行只按写集拆分，不按过细概念拆分。
- 开发线回交必须包含改动文件、验证、边界确认、未完成项。
- 复核线不直接改代码，除非主管明确要求修补；默认只交报告。

## 7. Checkpoint 拆分

### K0：Stage K 范围、权限和验收矩阵冻结

类型：文档 / 只读复核。

目标：

- 冻结 Stage K 的交付目标。
- 冻结测试项目和真实执行授权规则。
- 冻结 UI 信息层级规则。
- 冻结 checkpoint 同步规则。
- 冻结哪些能力算完成，哪些不能冒领。

必须产出：

- K0 任务包。
- Stage K acceptance matrix。
- 测试项目矩阵：`mario test`、工作台自身项目、隔离测试项目。
- 真实执行授权矩阵：read-only、workspace-write、new session、resume、retry。
- UI 信息层级矩阵：普通层、详情层、设置开发者层。

验收：

- 不改产品代码。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 权威入口只登记 Stage K 计划和 K0 状态，不宣称能力完成。

### K1：智能体对话页日常可用重构

类型：前端产品化。

目标：

- 智能体页变成真正的对话界面，而不是控制中心。
- 主界面只保留项目、对话、消息、输入框、发送预览和必要状态。
- 开发者信息后撤到详情或设置。
- 页面不允许上下滚动失控，主聊天区域可用面积要接近 Codex 布局。

范围：

- `AgentView.tsx`
- 会话选择 / 对话列表 / 聊天区 / 输入框。
- 智能体页样式。
- 离线 UI 测试。
- 设置页开发者入口承接内部信息。

普通层必须保留：

- 项目选择。
- 对话选择 / 新建对话入口。
- 消息列表。
- 输入框。
- 发送前确认说明。
- 当前状态：可发送、等待确认、执行中、读回中、失败、需要用户处理。

普通层必须移走：

- Product Command 细节。
- adapter / provider 长边界。
- runtime log / audit raw refs。
- readback enum 细节。
- rollout path。
- sidecar / store revision。

验收：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- 普通可见文本不包含旧阶段术语和开发者边界长文案。
- 真实 Tauri 截图至少覆盖智能体页主界面。

非目标：

- 不改后端执行语义。
- 不新增真实 Codex 执行授权。

### K2：通用 Codex resume / new session 产品入口

类型：后端 + 前端 + 权限 + 真实执行 checkpoint。

目标：

- 把 J1/J2 已有 Product Command 能力做成普通用户可用入口。
- 支持已有对话 `resume`。
- 支持受控 `new session`。
- 发送前必须展示影响范围和权限。
- 执行后必须稳定进入 runtime log、audit、readback 和运行队列。

范围：

- Product Command input/output。
- `codex-local` runner bridge。
- Permission dialog。
- Prompt summary/ref/hash。
- allowed write roots / denied paths。
- readback plan。
- Runtime log / audit refs。
- UI 状态。

验收矩阵：

- `resume/read-only`：指定项目、指定 session，真实执行成功，项目文件 hash 不变。
- `resume/workspace-write`：只写 allowed write path，baseline 文件 hash 保持。
- `new-session/read-only`：隔离项目创建真实 session，readback 成功。
- `new-session/workspace-write`：隔离项目只写 allowed path，readback 成功。
- `blocked`：缺项目、缺授权、越界路径、secret 风险、重复 active attempt 都被阻断。
- `failed/readback unavailable`：不显示为 0 条结果，进入失败状态和待办。

强制边界：

- 每个真实执行点必须有任务包或 UI 确认。
- 不允许前端直接拼 CLI。
- 不允许裸 `codex exec` 绕过 Product Command。
- 不允许 prompt body 进入普通 sidecar / runtime log / memory。
- 不允许读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript。

验收：

- Rust 聚焦测试通过。
- `cargo test --lib` 通过或记录既有非相关失败。
- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build` 通过。
- 至少一次真实 `resume` 和一次真实 `new session` 通过。
- evidence / handoff 记录真实副作用、hash、readback、audit、runtime log。

### K2.5：架构校准 gate

类型：架构校准 + 安全边界 + 验收扫描。

状态：已完成，结论为 `accepted`。记录见 `../../evidence/2026-06-10-stage-k-k2-5-architecture-calibration-command-surface-product-path-and-memory-consistency-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k2-5-architecture-calibration-command-surface-product-path-and-memory-consistency-v1-result.md`。

依据：

- `docs/plans/2026-06-10-stage-k-architecture-calibration-plan-v1.md`
- `tasks/2026-06-10-stage-k-k2-5-architecture-calibration-command-surface-product-path-and-memory-consistency-v1.md`

目标：

- 在 K3 开始前收敛 command surface、Product Command 主路径、workflow dispatch 边界、MCP canvas sealed boundary、memory consistency、workspace-write proof 和 Stage K architecture acceptance gate。
- 保持 Stage K 原目标不变，不把架构校准变成新阶段目标漂移。
- 确保 K3 不继续继承 K2/J2 probe、legacy wrapper 或 MCP canvas runner 旁路风险。

范围：

- Tauri command surface 分类：`product_current`、`legacy_blocked`、`developer_settings_only`、`read_viewer_only`、`test_probe_only`、`sealed_experiment`。
- K2 / J2 固定执行点和 canonical prompt 的 fixture-only 边界。
- `RunUnit -> ProductCommand -> WorkerReport -> ProcessFact -> MemoryCapture` 主路径。
- memory capture / observation / candidate / formal memory 的跨 sidecar consistency finding。
- read-only / workspace-write 的文件 hash / manifest proof。

验收：

- 不执行新的真实 Codex。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 架构扫描能分类裸 `Command::new("codex")`、legacy command、MCP canvas runner、prompt persistence、readback null、candidate/formal memory 边界和跨 sidecar orphan / partial write。
- K2.5 通过后，K3 才能开始。

### K3：项目工作流真实派发闭环

类型：工作流 + 执行 + UI。

状态：Level A 已完成，结论为 `accepted`；Level B 字段冻结已完成，并经 K3-B1.0 prompt freeze repair 修补为 `accepted_with_prompt_freeze_repair`；K3-B0 专用 bridge / harness 已完成，结论为 `accepted_with_p2`；K3-B1.1 Codex state permission / retry gate 已完成，结论为 `accepted_product_classification_retry_gate`；K3-B1 retry 申请再次被安全审查拒绝；下一步只能是用户手动执行 exact command 并回交，或用户明确重新批准风险后主管线再次申请，不能直接跳到 B2 或声明 K3-Level-B 完成。Level A 记录见 `../../evidence/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-level-a-v1-result.md`；字段冻结记录见 `../../evidence/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1-result.md`；K3-B0 记录见 `../../evidence/2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k3-b0-real-workflow-execution-bridge-and-harness-v1-result.md`。

目标：

- 从项目目标生成工作流 run units。
- 每个 run unit 可以被派给 `codex-local`。
- 工作流状态能显示在项目页和运行中工作流。
- 开发线 / 验证线 / 回收线能回交结果。
- 项目主管能确认过程事实。

范围：

- 项目目标入口。
- run unit 生成和展示。
- worker handoff。
- readback 到 worker report。
- process fact decision。
- final review handoff。
- 运行中工作流页。

最小闭环：

```text
用户目标
-> 生成 3 个 run units：开发 / 验证 / 回收
-> 开发 run unit 真实执行
-> 验证 run unit 真实或 fixture 执行
-> 回收 run unit 生成摘要
-> 项目主管确认过程事实
-> 用户看到最终结果
```

验收：

- `mario test` 完成一次 read-only 工作流闭环。
- 隔离项目完成一次 workspace-write 工作流闭环。
- 运行中工作流页能显示每个 run unit 状态。
- 失败 run unit 能进入待办，不自动伪装成功。
- C5 / C6 链路仍可追溯。

非目标：

- 不做 planned adapters。
- 不做无限制多 agent 自治。
- 不做无确认自动重试。

### K4：记忆捕获、候选确认和任务记忆注入体验

类型：记忆层 + UI + 工作流接入。

状态：非真实产品化切片已完成，结论为 `accepted_non_real_productization_slice`。记录见 `../../evidence/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k4-memory-capture-candidate-and-task-memory-injection-ux-v1-result.md`。

当前接受范围：

- 记忆页 / 运行中工作流页能用普通用户可读层级展示捕获、观察、候选、待正式化、补证、任务记忆包入选 / 排除 / 待审材料和行动项。
- Observation / Candidate / Capture 仍明确不是 FormalMemory。
- TaskMemoryPacket included 仍只允许正式记忆；candidate / observation 只能作为待审材料。
- 本轮不依赖 K3-B1 retry 或 K3-B2，不执行真实 Codex，不发送 prompt，不读写 `/Users/yoyi/.codex`。

仍未完成：

- 单次真实 Codex 操作后自动生成 observation / candidate 的真实执行验收。
- 工作流真实闭环后自动生成候选的真实执行验收。
- 用户确认候选后写 FormalMemory 的新能力。
- K4 全量完成。

目标：

- 工作台操作和 Codex 执行结果进入记忆层分析链路。
- 用户能看懂“这次产生了哪些候选记忆”。
- 下一次任务能看懂“将注入哪些记忆”。
- 正式记忆仍必须经过确认权和审计。

范围：

- MemoryCaptureEvent 展示和过滤。
- Observation 到 candidate 的体验。
- Candidate 确认 / 拒绝 / 编辑 / 延后。
- TaskMemoryPacket preview 在发送前展示。
- 执行结果和 worker report 生成候选。
- 记忆中心信息层级整理。

验收：

- 单次 Codex 操作后生成 observation / candidate。
- 工作流闭环后生成候选记忆列表。
- 用户确认后写 FormalMemory record / version / audit。
- 被拒绝 / 隔离 / 需要确认的候选不会进入正式记忆。
- 任务发送前能展示 included / excluded / review materials。
- 不把 candidate / knowledge hit / observation 当正式记忆。

非目标：

- 不做自动技能化。
- 不接向量库 / 图数据库 / GraphRAG。
- 不做 Obsidian 原生同步。

### K5：运行中、待办、失败恢复和操作控制

类型：运行状态 + UX + 安全边界。

状态：非真实产品化切片已完成，结论为 `accepted_non_real_productization_slice`。记录见 `../../evidence/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k5-running-todo-failure-recovery-and-operation-control-ux-v1-result.md`。

目标：

- 用户能看懂当前正在做什么、哪里卡住、下一步要做什么。
- 失败、超时、读回不可用、重复派发、权限拒绝都有明确状态。
- retry / stop / restart 可以先做受控提案和确认流，再按授权推进真实操作。

范围：

- 运行中工作流页。
- 右侧运行中 / 待办 / 通知。
- Failure classification。
- Retry proposal。
- Stop / restart readiness。
- Duplicate guard。
- Stale active attempt cleanup。

验收：

- `readback_unavailable` / `readback_failed` / `timed_out` 仍保持 result_count 为未知，不显示成 0。
- 用户能从待办进入确认 / 失败详情。
- 真实 retry 不默认执行，必须再次确认。
- stop / restart 不冒充已实现，除非另有真实验收。
- runtime log 和 audit 可追溯。

完成说明：

- 新增 `operation_control_summary` 前端只读摘要。
- 运行中工作流页新增“操作控制 / 恢复建议”普通层。
- retry / stop / restart / resume 仍只是需确认、后续任务或只读 readiness，不是真实操作能力。
- readback unavailable / failed / timed_out / null result count 继续显示为未知 / 不可用，不显示为 0。
- 复核线最终结论为通过，无 P0/P1/P2。

### K6：真实 Tauri dogfood 和验收收口

类型：真实桌面验收 + 文档收口。

状态：已完成，结论为 `accepted_with_deferred_items`。K6 原始结论为 `blocked_by_tauri_webview_blank_window`，记录见 `../../evidence/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k6-real-tauri-dogfood-and-stage-acceptance-freeze-v1-result.md`。K6.1 已进一步分类为 `blocked_by_window_capture_webview_layer_after_app_mount`；K6.2 已恢复 ScreenCaptureKit window-only 可见截图链路。最终回到 K6 主任务，补齐首页、智能体、运行中工作流、项目、记忆层、知识库、设置、想法箱、Skill、Harness 的真实 Tauri 核心入口截图，记录见 `../../evidence/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1.md` 与 `../../handoffs/2026-06-10-stage-k-k6-final-tauri-dogfood-core-path-screenshot-acceptance-v1-result.md`。

目标：

- 不再只靠离线测试和浏览器 smoke。
- 用真实 Tauri 验收核心路径。
- 冻结 Stage K 完成项和 deferred 项。

验收项目：

- `mario test`。
- 工作台自身项目。
- 新隔离测试项目。

截图 / 手动检查清单：

- 首页入口。
- 智能体对话页。
- 发送前权限弹层。
- 运行中工作流。
- 项目工作流。
- 记忆候选确认。
- 记忆正式化。
- 设置开发者区。
- 失败 / readback unavailable 状态。

测试命令：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- Rust 聚焦测试。
- `cargo test --lib`
- `rustfmt --check` 相关文件。
- 误导文案扫描。
- 敏感路径 / 真实执行扫描。

验收结论：

- `accepted`：仅当核心链路在真实 Tauri 和测试项目中全部通过。
- `accepted_with_deferred_items`：允许 UI 自动化、planned adapters、provider credential、自动 retry 等后置。
- `blocked`：真实执行链路或记忆链路无法形成最小闭环。

## 8. 关键数据和对象

Stage K 不应新增大量孤立状态。优先复用现有 Product Command、RunUnit、RuntimeLog、Audit、Readback、Observation、MemoryCandidate。

如需新增对象，必须说明为什么现有对象不能承载。

建议统一产品对象：

### 8.1 WorkbenchUserCommand

用户从 UI 发起的普通操作意图。

字段建议：

- `user_command_id`
- `project_id`
- `project_root`
- `session_mode`
- `target_session_id`
- `user_visible_prompt_summary`
- `prompt_ref`
- `prompt_hash`
- `requested_capability`
- `permission_preview_ref`
- `task_memory_packet_ref`
- `created_by`
- `created_at`

### 8.2 CodexExecutionIntent

可被 Product Command 消费的执行意图。

字段建议：

- `intent_id`
- `source_kind`
- `source_ref`
- `adapter_id`
- `execution_mode`: `resume | new_session`
- `sandbox_mode`
- `cwd`
- `allowed_write_roots`
- `denied_paths`
- `readback_plan`
- `runtime_log_policy`
- `audit_policy`

### 8.3 WorkbenchRunUnit

工作流执行单元。

字段建议：

- `run_unit_id`
- `workflow_id`
- `node_id`
- `role`
- `status`
- `command_ref`
- `attempt_refs`
- `runtime_log_refs`
- `audit_refs`
- `readback_ref`
- `worker_report_ref`
- `observation_refs`
- `memory_candidate_refs`

### 8.4 MemoryCaptureEvent

进入记忆层的捕获事件。

字段建议：

- `capture_event_id`
- `source_type`
- `source_ref`
- `project_id`
- `workflow_id`
- `summary`
- `sensitivity`
- `candidate_policy`
- `blocked_reason`
- `created_at`

## 9. UI 信息层级规则

Stage K UI 必须按人类操作方式组织，而不是按后端对象堆叠。

### 9.1 普通用户层

普通用户层只回答：

- 我在哪个项目？
- 我在和哪个 Codex 对话？
- 我要让它做什么？
- 发送前会影响哪里？
- 现在执行到哪一步？
- 需要我确认什么？
- 结果是什么？
- 有什么值得记住？

### 9.2 详情层

详情层可以展示：

- 记忆包 included / excluded / review materials。
- 写入范围。
- 风险和权限原因。
- readback 用户解释。
- worker report 摘要。
- evidence / handoff 链接。

### 9.3 开发者层

开发者层才展示：

- Product Command ids。
- runtime log refs。
- audit refs。
- sidecar path。
- store revision。
- readback enum。
- adapter / provider raw boundary。
- diagnostic raw detail。
- legacy command status。

开发者层默认位置：

- `设置 > 开发者`
- 或普通页面底部默认收起的 `开发者详情`

## 10. 权限和安全规则

真实 Codex 执行必须满足：

- 有项目。
- 有 cwd。
- 有 allowed write roots。
- 有 denied paths。
- 有 prompt summary/ref/hash。
- 有 permission envelope。
- 有 user confirmation。
- 有 runtime log。
- 有 audit。
- 有 readback plan。
- 有 failure classification。

禁止：

- 前端直接执行 CLI。
- 使用 shell 字符串拼接 Codex 命令。
- 绕过 Product Command。
- 无确认写项目文件。
- 无确认读写 `/Users/yoyi/.codex`。
- 读取 secret、token、`.env`、keychain、OAuth、provider credential。
- 把完整 transcript 或 rollout 放进记忆。
- 把 readback unavailable 解释为 0 条结果。

## 11. 测试和验收矩阵

每个 checkpoint 至少包含：

- TypeScript typecheck。
- 离线交互测试。
- Vite build。
- 相关 Rust 聚焦测试。
- 误导文案扫描。
- 敏感路径扫描。
- 产品边界说明。

涉及真实执行的 checkpoint 额外包含：

- 真实执行前 hash / baseline。
- 真实执行后 hash / baseline。
- allowed write path 证明。
- readback last message 或失败分类。
- runtime log refs。
- audit refs。
- `.codex` 副作用范围说明。
- 回滚 / 清理说明。

涉及 UI 的 checkpoint 额外包含：

- UI 显示边界确认。
- 普通层 / 详情层 / 开发者层清单。
- 真实 Tauri 截图或明确未完成说明。

## 12. 文档同步规则

为降低维护成本，Stage K 默认不在每次小修补后同步所有权威入口。

必须同步入口的时机：

- K0 完成。
- K2 完成，因为涉及真实执行产品入口。
- K4 完成，因为涉及记忆层产品闭环。
- K6 完成，因为涉及阶段验收。
- 出现阻断、权限事故、真实执行失败且影响边界。
- 阶段目标或非目标变化。

不必同步入口的时机：

- 单个 UI 文案修补。
- 单个 CSS / 信息层级小改。
- 测试断言调整。
- 开发者详情位置移动。
- 不改变产品能力的小型重构。

每个 checkpoint 最少产出：

- task package。
- evidence。
- handoff。
- 验证清单。
- 边界确认。

## 13. 风险清单

### P0 风险

- 绕过 Product Command 执行真实 Codex。
- 无确认读写 `/Users/yoyi/.codex`。
- 无确认写项目文件。
- secret / token / `.env` / keychain / OAuth / provider credential 被读取或写入记忆。
- readback failed 被写成成功。
- candidate / observation 被当成正式记忆。

### P1 风险

- UI 又变回控制中心。
- 运行队列和待办合并导致用户不知道要处理什么。
- 自动工作流绕过项目主管 / 全局主管确认。
- new session 成功率不稳定但被包装成已完成。
- retry / stop / restart 被提前显示为可用真实能力。

### P2 风险

- Skill / Harness 仍偏开发者视角。
- 记忆层页面信息密度过高。
- 真实 Tauri 截图覆盖不足。
- evidence / handoff 过多导致维护成本升高。

## 14. 推荐推进顺序

推荐顺序：

1. K0：冻结范围和验收矩阵。
2. K1：智能体对话页可用性。
3. K2：通用 Codex 执行入口。
4. K2.5：架构校准 gate。
5. K3：项目工作流真实派发闭环。
6. K4：记忆捕获和候选确认体验。
7. K5：运行中 / 待办 / 失败恢复。
8. K6：真实 Tauri dogfood 和阶段冻结。

可以并行：

- K1 UI 线可以和 K2 后端执行线并行，但不能抢同一个 `AgentView.tsx` 写集。
- K3 工作流线在 K2.5 架构 gate 通过后开始。
- K4 记忆线可以先做 UI / read model，但真实 capture 必须等 K2/K3 的事件来源稳定。
- K5 可以在 K2/K3 后半段并行。
- K6 必须等 K1-K5 至少一个完整链路可跑后执行。

## 15. Stage K 完成定义

Stage K 最低完成定义：

- 用户能在智能体页选择项目和对话，输入任务并发起受控 Codex 执行。
- 至少一次真实 `resume` 和一次真实 `new session` 在授权测试项目中通过。
- 至少一次项目工作流 run unit 真实派发成功。
- 执行结果进入 runtime log、audit、readback、运行队列。
- 至少一个执行结果生成 observation / memory candidate。
- 用户能确认一个候选进入正式记忆，并产生 version / audit。
- 下一次任务能看到将注入的任务记忆包。
- 失败、读回不可用、权限拒绝和重复派发都能正确显示，不伪装成功。
- 真实 Tauri 至少完成核心路径截图验收。

Stage K 仍可 deferred：

- planned adapters 真实接入。
- provider credential / model verification。
- 完整自动 retry / stop / restart。
- 完整 UI 自动化测试。
- GraphRAG / 向量库 / 图数据库。
- Obsidian 原生同步。

## 16. 下一步建议

K0、K1、K2、K2.5、K3-Level-A、K3-Level-B 字段冻结、K3-B0 bridge / harness、K3-B1.0 prompt freeze repair、K3-B1.1 Codex state permission / retry gate、Stage K architecture calibration v2/v3 and gate、K4 非真实记忆产品化切片、K5 非真实运行 / 待办 / 失败恢复和操作控制切片、K6 真实 Tauri dogfood 核心入口验收均已完成。Stage K 当前冻结为 `accepted_with_deferred_items`。下一步不应继续在 K6 内追加小补丁；应进入 post-K deferred closure：K3-B1 已执行但失败分类，K3-B1 retry 申请再次被安全审查拒绝。如恢复 K3-B1/K3-B2 真实执行线，只能由用户手动执行 exact command 并回交，或用户明确重新批准风险后主管线再次申请，而不是直接跳到 K3-B2。

K3-B1 任务包应冻结：

- `/Users/yoyi/Documents/mario test` 项目路径。
- 目标 session：`019e798a-ac37-7771-b982-e38084fcd22e`。
- operation：`resume`。
- sandbox：`read-only`。
- marker：`K3_B1_MARIO_TEST_WORKFLOW_READ_ONLY_OK_2026_06_10`。
- prompt ref / prompt hash。
- `.codex` 必要副作用范围。
- 项目核心文件 baseline hash。
- Product Command / continuation / runtime log / audit / readback refs。
- 停止条件和失败分类。

K3-B1 完成并经主管线复核后，才能准备 K3-B2 workspace-write / new-session 任务包。K3-B2 前应先处理 K3-B0 P2：allowed write path 的内容 / marker / hash 断言必须写进验收。
