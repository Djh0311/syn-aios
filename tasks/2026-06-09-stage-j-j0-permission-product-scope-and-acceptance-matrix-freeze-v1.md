# Stage J / J0 Permission, Product Scope, And Acceptance Matrix Freeze v1

日期：2026-06-09

状态：已完成。只读复核线结论为通过，无 P0/P1/P2 必须修补项；允许进入 J1 任务包准备。J0 不授权真实执行，J1 的任何真实执行点仍必须在 J1 任务包中重新逐项授权。

全局主管任务。本文是 Stage J 的 J0 任务包，用于冻结“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”的产品范围、安全边界、测试项目、UI 信息层级、分线职责和 J1-J6 验收矩阵。J0 是文档 / 任务包冻结，不改产品代码，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- 中间版本不是从零开始。C1-C6 已有工作流闭环，M1-M13 已有记忆层闭环，G1/G2 已有 runtime log / diagnostics，H/I 已有 `codex-local` runner 和多 agent / 多模型中立协议抽象，PCR0-PCR10 已完成统一 Product Command Routing checkpoint。
- 现阶段真正缺口不是“有没有 guard / preview / stub”，而是把 `codex-local` 自由操控、项目工作流自动编排、runtime/audit/readback、记忆 observation/candidate 串成用户可用的产品闭环。
- 如果直接开放裸控制台，会绕过项目、权限、任务包、任务记忆包、runtime log、audit、readback、用户确认和记忆状态机。
- 如果继续只做只读面板或健康探针，工作台仍不能完成用户目标。

Stage J 阶段目标冻结为：

```text
自由操控 Codex
+ 自动化工作流编排
+ 记忆层记录 / 分析 / 候选化
```

J0 假设：

- Stage J 只产品化 `codex-local`，不把 Claude Code / OpenClaw / OpenCode / OpenCode-like planned adapters 变成真实执行能力。
- 所有真实执行都必须归口统一 Product Command，不允许从旧 H5、legacy dispatch、direct CLI、MCP canvas run 或测试 probe 直接变成普通产品入口。
- 记忆层只能先记录 observation / candidate / audit/runtime refs，不允许绕过 M2/M9/M12 的确认链路直接写正式记忆。
- 入口文档只在 checkpoint 同步。J0 任务包创建本身不要求同步 `CURRENT.md` / `tasks/README.md`，除非 J0 收口时全局主管明确作为 checkpoint 更新。

## 1. 权威依据

J0 必须服从：

- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `tasks/2026-06-09-unified-product-command-routing-pcr10-final-review-and-checkpoint-closure-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`

只读参考的已完成事实：

- C1-C6 task / evidence / handoff。
- M1-M13 task / evidence / handoff。
- G1-G5 task / evidence / handoff。
- H0-H7 task / evidence / handoff。
- I0-I6 task / evidence / handoff。
- PCR0-PCR10 task / evidence / handoff。

## 2. J0 接受范围

J0 可接受为：

- Stage J 交付目标和不做项已冻结。
- J1-J6 的任务顺序、真实执行授权条件和验收矩阵已冻结。
- `mario test` 与隔离测试项目的使用原则已冻结。
- `codex-local` 自由操控入口必须绑定项目 / workflow / run unit / temporary run 的原则已冻结。
- 允许写入根、禁止路径、secret deny list、prompt body / transcript 存储策略已冻结。
- 记忆捕获策略已冻结：哪些进入 observation、哪些进入 candidate、哪些只能保留 audit/runtime ref。
- UI 信息层级已冻结：普通用户视图、详情视图、设置 / 开发者区。
- 多会话协作分线职责和复核回交要求已冻结。

J0 不接受为：

- J1 / J2 / J3 / J4 / J5 / J6 已完成。
- 通用自由 Codex 控制台已实现。
- 新 session 真实执行已成功。
- 自动化工作流真实闭环已跑通。
- 任意项目自由执行已开放。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 可无确认执行。
- 记忆层可自动写正式记忆。
- 真实 Tauri 产品验收完成。

## 3. Stage J 总边界

Stage J 必须做：

- 工作台内 `codex-local` 自由操控入口：选择项目、选择 session、输入任务、预览影响、用户确认、真实执行。
- 项目级执行绑定：project、workflow、node、task package、task memory packet、run unit、session、runtime log、audit、readback。
- 自动化工作流编排：主管线、开发线、验证线、回收线等 run units 的生成、授权、执行、回交、复核。
- 记忆层记录和分析：用户操作、ProductCommand、runtime log、readback、worker report、process fact decision、final review 进入 observation / candidate 管道。
- UI 产品化表达：普通用户能看到“要做什么、正在做什么、需要我确认什么、结果是什么、哪些会进入记忆”。

Stage J 禁止做：

- 不接入 planned adapters 的真实执行。
- 不做 provider credential store、真实 token 读取或 model verification。
- 不开放无限制任意目录读写。
- 不让 agent 自治批准高风险权限。
- 不自动写正式记忆。
- 不无确认自动 retry / stop / restart。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential、rollout 写入普通 sidecar、runtime log、audit、memory observation、memory candidate 或 UI。
- 不把普通浏览器 smoke 当作真实 Tauri 验收。

## 4. 任务顺序和授权矩阵

| 任务 | 类型 | 是否允许真实 Codex | 是否允许读写 `/Users/yoyi/.codex` | 是否允许写项目文件 | 是否同步权威入口 | 冻结边界 |
| --- | --- | --- | --- | --- | --- | --- |
| J0 权限 / 范围 / 验收矩阵冻结 | 文档 / 只读复核 | 不允许 | 不允许 | 不允许 | 默认不；收口 checkpoint 可同步 | 冻结 Stage J 边界，不开发 |
| J1 Codex Control Plane 自由操控入口 | 后端 + 前端 + 测试 | 允许，但仅在 J1 任务包执行点逐项授权后 | 允许，但仅限目标 session / new-session 必要最小范围 | 允许，但仅限测试项目授权根 | 建议 J1 checkpoint 同步 | 必须走统一 Product Command；无确认不发送 prompt |
| J2 项目工作流自动编排闭环 | 后端 + 项目 / 运行中 UI + 测试 | 允许，但只能通过 J1/J2 product command 调度 | 只能继承 J1/J2 明确授权范围 | 仅限授权测试项目和 run unit 范围 | 建议 J2 checkpoint 同步 | run units 必须可追溯；失败不能包装成成功 |
| J3 记忆层捕获总线和候选生成 | 后端 + 记忆 UI + 测试 | 默认不新增真实执行；可消费 J1/J2 证据 | 默认不读写；只引用已有 runtime/audit/readback refs | 默认不写项目文件 | 建议 J3 checkpoint 同步 | 不自动写正式记忆；敏感材料不得进入记忆 |
| J4 运行队列、失败控制和用户确认队列 | 后端 + UI + 测试 | 可在已授权 product command 上做受控 retry/stop preview；真实 retry/stop 必须单独确认 | 仅限已授权 run 的必要范围 | 仅限已授权 run 的必要范围 | 建议 J4 checkpoint 同步 | 不做无确认自动 retry / stop / restart |
| J5 UI 信息层级和真实 Tauri 产品验收 | 前端 + 真实 Tauri 验收 | 不新增真实执行能力；只验收 J1-J4 已有路径 | 不新增 `.codex` 访问范围 | 不新增项目写入范围 | 建议 J5 checkpoint 同步 | 必须真实 Tauri；普通 UI 不铺开发者 raw 状态 |
| J6 最终验收和后续路线冻结 | 全局主管复核 + 文档 | 不允许新增真实执行 | 不允许新增 `.codex` 读写 | 不允许新增项目写入 | 必须同步 checkpoint | 冻结 accepted / deferred / blocked |

任何 J1-J4 的真实执行点在执行前必须重新列明：

- 操作类型：`resume`、`new_session`、`retry`、`stop`、`restart` 或其他。
- adapter：当前只能是 `codex-local`。
- 项目、project root、cwd、workflow、run unit、task package、memory packet。
- 目标 session 或新 session 创建规则。
- sandbox、timeout、allowed write roots、denied paths。
- `/Users/yoyi/.codex` 最小读取 / 写入范围。
- prompt summary、prompt hash、prompt body 运行时输入策略。
- readback plan、runtime log、audit、evidence / handoff 路径。
- failure、timeout、duplicate、cancel、degraded、rollback / recovery 策略。
- 用户确认方式，且高影响真实执行必须 `confirmed_by: "user"`。

## 5. 测试项目矩阵

| 项目 | 路径 | 用途 | 默认权限 | 备注 |
| --- | --- | --- | --- | --- |
| `mario test` | `/Users/yoyi/Documents/mario test` | 历史真实 resume / 项目工作流 probe 参考；J1/J2 可作为受控验收项目 | J0 不授权；J1/J2 执行点必须重新授权 | 不能因为 E/H/PCR 旧 probe 成功就默认继承权限 |
| Stage J 隔离测试项目 | `/Users/yoyi/workspace/product-line/test-fixtures/stage-j-isolated-project` | 默认优先的新隔离项目；用于自由操控、自动编排、记忆捕获闭环 | J0 不创建；J1 或 J2 可在任务包中创建并授权 | 必须可安全删除或回滚，不包含 secret / `.env` |
| 真实业务项目 | 由用户单独指定 | 非默认；仅在用户明确指定且任务包列出备份 / 回滚时使用 | 默认禁止 | 不得在 Stage J 默认验收中使用 |

测试项目必须满足：

- allowed write roots 明确且足够窄。
- denied paths 明确。
- 不包含 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 可通过 file hash、runtime log、audit、readback、worker report、memory candidate refs 交叉验证。
- 如果是写入型任务，必须记录预期写入文件和回滚方式。

## 6. 路径、数据和敏感信息边界

默认允许的写入：

- `product-line/tasks/**`、`product-line/evidence/**`、`product-line/handoffs/**` 的 Stage J 文档记录。
- 工作台自有受控 sidecar / store：ProductCommand、continuation、runtime log、audit、workflow state、memory observation / candidate，且必须由产品代码路径写入。
- J1/J2 任务包逐项授权的隔离测试项目路径或 `mario test` 子路径。

默认禁止：

- `/Users/yoyi/.codex`，除非 J1/J2/J4 的真实执行任务包逐项授权。
- auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 完整 transcript / rollout。
- 用户未授权的真实业务项目。
- 任意 provider credential store。
- 任意 shell 临时脚本绕过 Product Command / permission envelope / audit 的执行路径。
- 使用 `--dangerously-bypass-approvals-and-sandbox`。

`/Users/yoyi/.codex` 最小授权原则：

- 真实 `resume` / `new_session` 可由 runner 必要写入 Codex 自身状态。
- readback 只能读取任务包列明的目标 session / run 所需最小结果，不得读取完整历史 transcript 作为普通证据。
- 任何 `.codex` 读取都不得进入 memory observation / candidate 的原文，只能进入脱敏 summary、ref、hash 或边界状态。

## 7. Prompt Body 和 Transcript 策略

Prompt body 策略：

- prompt body 只作为运行时输入发送给 `codex-local`。
- 普通 sidecar、runtime log、audit、memory observation、memory candidate 默认只存 `prompt_summary`、`prompt_ref`、`prompt_hash`、任务包引用和用户确认引用。
- 如未来需要保存 prompt body，必须单独任务包、单独 UI 提示、单独用户确认，并说明保留期限和删除方式。

Transcript 策略：

- 完整 transcript 默认禁止读取、展示、存储或入记忆。
- readback 只能保存用户可理解的结果摘要、状态、错误类别、必要输出片段和 refs。
- `readback_unavailable`、`readback_failed`、`timed_out`、`blocked_by_guard` 必须保持真实状态，`result_count` 不得伪装为 0。

## 8. 记忆捕获策略

Stage J 记忆捕获必须使用分层策略：

| 来源 | 默认去向 | 是否可生成 candidate | 是否可直接正式化 | 说明 |
| --- | --- | --- | --- | --- |
| 用户操作 / 用户确认 | observation + audit ref | 可 | 不可 | 记录用户决定、范围和原因，不记录敏感原文 |
| ProductCommand preview / decision / attempt | runtime/audit + observation summary | 可 | 不可 | 记录做了什么、为什么、结果是什么 |
| runtime log | runtime ref + diagnostic summary | 视情况 | 不可 | 失败类别和状态可入候选，raw log 不入记忆 |
| readback | readback summary + observation | 可 | 不可 | 不读取完整 transcript，不把 unavailable 写成成功 |
| worker report | observation | 可 | 不可 | 必须标明来源是 worker report，不等于正式事实 |
| project director process fact decision | observation | 可 | 不可 | 可作为候选强来源，但仍需 formal memory 确认 |
| global final review / user result decision | observation + candidate refs | 可 | 不可 | 可推动正式化提案，但不得绕过用户确认 |
| secret / token / `.env` / credential / full transcript / rollout | 禁止 | 不可 | 不可 | 只允许记录“已排除 / 被阻断”的边界摘要 |

正式记忆规则：

- 任何 MemoryCandidate 到 FormalMemory 必须继续走 M2 / M9 / M12 既有状态机。
- 高影响正式化、生命周期操作或跨项目记忆仍必须 `confirmed_by: "user"`。
- 项目主管确认不能替代用户确认。
- candidate / observation / knowledge hit 不能在 UI 中写成“已记住”或“正式记忆”。

## 9. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

Stage J 后续 UI 显示边界：

| 层级 | 应显示 | 禁止显示 |
| --- | --- | --- |
| 普通用户首屏 | 项目、智能体、Skill、Harness、运行中工作流；“我要做什么 / 现在在做什么 / 需要确认什么 / 结果是什么” | raw enum、sidecar 字段、internal id、完整日志、developer boundary 长文案 |
| 项目详情 | 项目目标、工作流、run units、任务包摘要、记忆包摘要、权限和结果 | 完整 prompt body、完整 transcript、secret、provider credential |
| 智能体 | `codex-local` session 选择、resume / new-session readiness、执行状态、readback 摘要 | planned adapter 可执行按钮、凭据已验证误导文案、裸 CLI 控制台 |
| 运行中工作流 | running / waiting_user / blocked / failed / readback unavailable / duplicate blocked | 将失败包装成成功、将 unknown 显示为 0 |
| 记忆 | observation / candidate / formal memory 的来源、状态、确认入口 | candidate 写成正式记忆、敏感原文、完整 transcript |
| 设置 / 开发者 | legacy 入口、diagnostics、raw refs、内部边界、开发者检查 | 不能作为普通用户完成任务的唯一入口 |

平台边界：

- 只做桌面 Tauri 工作台。
- 不做手机端 UI。
- 不做 mobile-first responsive 设计。
- 普通浏览器可作为辅助 smoke，但不能替代真实 Tauri 验收。

## 10. J1-J6 验收矩阵

| 阶段 | 必须验收 | 不得声称 |
| --- | --- | --- |
| J1 | 用户在工作台内选择项目 / session，输入任务，生成 preview，确认后真实 `codex-local resume`；运行中和智能体页可读回 | 任意项目自由执行、新会话真实成功、planned adapters 可执行 |
| J2 | 用户目标生成 run units，开发 / 验证 / 回收 / 主管复核链路可追溯；至少一个测试项目完成闭环 | agent 自治批准、高风险自动派发、失败自动包装成功 |
| J3 | 一次真实操作后可在记忆中心看到 observation / candidate / source refs；正式记忆仍需确认 | 操作自动变正式记忆、secret/full transcript 入记忆 |
| J4 | 用户确认队列、retry / stop / restart preview、duplicate guard、failure/degraded 状态可用 | 无确认自动 retry / stop / restart |
| J5 | 真实 Tauri 覆盖 J1-J4 关键路径；普通 UI 信息层级清楚 | 普通浏览器 smoke 冒充 Tauri、开发者 raw 状态铺首屏 |
| J6 | Stage J 三条主链路 evidence / handoff / acceptance matrix 冻结；后续路线明确 | 最终蓝图完成、planned adapters 接入、provider/model 验证完成 |

Stage J 最低通过线：

- `mario test` 至少一条受控真实 `resume` 产品路径证据。
- 隔离测试项目至少一条自由操控或自动工作流闭环证据。
- 至少一条真实操作进入 observation / candidate 管道的证据。
- 运行状态、失败状态、readback 状态、用户确认状态可追溯。
- 真实 Tauri 验收覆盖核心路径或明确列为 deferred，不得冒领。

## 11. 分线职责

### 主管线

- 维护 Stage J 目标、任务边界、授权矩阵和 checkpoint。
- 决定何时允许真实执行。
- 派发并复用长期开发线 / UI 线 / 记忆线 / 复核线。
- 接收回交，判断 P0/P1/P2，必要时要求修补。
- 控制权威入口同步时机。

### 后端线

- J1/J2/J3/J4 的 Rust 服务、store、read model、ProductCommand 接线、run unit、memory capture event。
- 不直接改 UI 信息架构。
- 不执行真实 Codex，除非任务包执行点明确授权。

### UI 线

- J1/J2/J4/J5 的普通用户路径和信息层级。
- 保持桌面 Tauri 工作台，不做手机端。
- 开发者 / 内部边界信息进入设置 / 开发者。
- UI 不新增绕过 ProductCommand 的执行按钮。

### 记忆线

- J3 memory capture event 到 observation / candidate。
- 任务记忆包解释、lint、conflict、formalization 确认链路。
- 防止 raw prompt、完整 transcript、secret、rollout 入记忆。

### 复核线

- 只读审查任务包、代码和证据。
- 检查真实执行是否绕过 ProductCommand。
- 检查记忆是否绕过状态机。
- 检查 UI 是否冒领 planned adapters、provider credential、自动 retry、正式记忆。
- 回交 P0/P1/P2，不直接改文件。

### 真实探针线

- 只在 J1/J2/J4 的执行点授权后运行。
- 只使用任务包列明的项目、session、cwd、allowed write roots、timeout。
- 真实执行前后必须记录 file hash / runtime log / audit / readback / evidence。
- 不继承旧 E/H/PCR probe 授权。

## 12. J0 只读复核要求

复核线必须检查：

- J0 是否足够支撑 J1-J6 开发。
- J0 是否把“自由操控 Codex”误写成裸 CLI / 裸控制台。
- J0 是否绕过 PCR0-PCR10。
- J0 是否给真实执行、`.codex` 读写、项目写入留下未定义口子。
- J0 是否允许 prompt body、完整 transcript、secret、rollout 入日志或记忆。
- J0 是否明确 candidate / observation 不等于 formal memory。
- J0 是否明确 planned adapters 不可执行。
- J0 是否明确 UI 分层和桌面 Tauri 验收。
- J0 是否符合“少拆任务、checkpoint 同步入口文档”的工作方式。

复核线不得：

- 改文件。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret、token、`.env`、keychain、OAuth、provider credential、full transcript、rollout。
- 启动 Tauri / Browser / Chrome / 截图工具。

## 13. J0 验收 / 扫描

J0 默认不跑：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test`

原因：J0 不改产品代码。

J0 收口前建议扫描：

```text
rg -n "Stage J|J0|自由操控 Codex|自动化工作流编排|记忆层记录|不授权真实|/Users/yoyi/.codex|checkpoint" tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md
rg -n "J1 已完成|J2 已完成|Stage J 已完成|通用自由 Codex 控制台已实现|planned adapters 真实接入" CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md
```

如果 J0 收口时同步入口文档，则必须追加扫描入口旧口径和过度声明。

## 14. 回交产物

J0 任务包：

- `tasks/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`

J0 收口时应新增：

- `evidence/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1.md`
- `handoffs/2026-06-09-stage-j-j0-permission-product-scope-and-acceptance-matrix-freeze-v1-result.md`

J0 收口结论必须包含：

- 复核线 P0/P1/P2。
- 是否允许进入 J1。
- 是否需要同步 `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `README.md`。
- J1 是否允许真实执行，以及若允许，执行点授权必须另写在 J1 任务包中。

## 15. 不得声明

J0 完成后仍不得声明：

- Stage J 已完成。
- 自由操控 Codex 已实现。
- 自动化工作流真实编排已完成。
- 记忆层已经记录本轮真实操作。
- 任何新的真实 `codex exec` / `codex exec resume` 已执行。
- `/Users/yoyi/.codex` 已被授权读写。
- `mario test` 或隔离测试项目已被授权写入。
- planned adapters 已真实接入。
- provider credential / model verification 已完成。
- 真实 Tauri 验收已完成。
