# Stage K / K0 Scope, Permission, And Acceptance Matrix Freeze v1

日期：2026-06-10

状态：已完成。复核线只读审查无 P0/P1；P2 已补“真实执行点字段工作表”和“候选测试项目登记表”。本文是 Stage K 的 K0 任务包，用于冻结“日常可用 Codex 工作台产品化”的范围、权限、安全边界、测试项目、UI 信息层级、分线职责和 K1-K6 验收矩阵。K0 是文档 / 只读复核任务，不改产品代码，不授权真实 `codex exec` / `codex exec resume`，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- Stage J / J6 已完成，结论为 `accepted_with_deferred_items`。J6 接受为“自由操控 Codex + 自动化工作流编排 + 记忆层记录 / 分析 / 候选化”的当前产品化 checkpoint 完成。
- 中间版本不是从零开始。C1-C6 已有受控自动化工作流闭环，M1-M13 已有记忆系统闭环，E/F/G 已有会话 / 画布 / runtime / diagnostics / 部分 Tauri 验收，H/I 已有 `codex-local` runner 和中立多 agent 抽象，PCR10 已把真实执行归口统一 Product Command。
- 当前缺口不是“缺 guard / 缺 preview”，而是日常可用性：用户还不能非常自然地在工作台里选择项目、选择对话、输入任务、确认影响范围、触发 Codex、看到结果、沉淀记忆。
- 如果继续拆很多小型只读面板，产品体验不会明显变好。
- 如果直接开放裸 Codex 控制台，会绕过项目、权限、runtime log、audit、readback 和记忆层。

Stage K 阶段目标冻结为：

```text
把 Stage J 的受控产品化 checkpoint 推进为日常可用工作台。
```

K0 假设：

- Stage K 先把 `codex-local` 做到真正日常可用，不接 planned adapters 的真实执行。
- 所有真实执行继续归口统一 Product Command，不允许从旧入口、legacy dispatch、直接 CLI 或测试 probe 变成普通产品路径。
- 记忆层继续使用 observation / candidate / FormalMemory 状态机，不允许绕过用户确认自动正式化。
- 权威入口只在 checkpoint 完成、阻断或阶段边界变化时同步；小修补不滚动更新所有入口。

## 1. 权威依据

K0 必须服从：

- `docs/plans/2026-06-10-stage-k-daily-use-codex-workbench-productization-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-09-stage-j-codex-control-plane-workflow-memory-productization-plan-v1.md`
- `docs/plans/2026-06-09-unified-product-command-routing-development-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

只读参考的已完成事实：

- Stage J / J6 task / evidence / handoff。
- J1-J5 task / evidence / handoff。
- C1-C6 task / evidence / handoff。
- M1-M13 task / evidence / handoff。
- E/F/G task / evidence / handoff。
- H/I task / evidence / handoff。
- PCR0-PCR10 task / evidence / handoff。

## 2. K0 接受范围

K0 可接受为：

- Stage K 交付目标和不做项已冻结。
- K1-K6 的任务顺序、真实执行授权条件和验收矩阵已冻结。
- 测试项目矩阵已冻结：`mario test`、工作台自身项目、隔离测试项目。
- `codex-local` 日常入口必须绑定项目、session、Product Command、runtime log、audit、readback 和记忆捕获的原则已冻结。
- `resume`、`new session`、workspace-write、retry、stop、restart 的授权边界已冻结。
- prompt body、transcript、rollout、secret、token、`.env`、keychain、OAuth、provider credential 的禁止存储 / 禁止入记忆策略已冻结。
- UI 信息层级已冻结：普通用户层、详情层、设置 / 开发者层。
- 多会话协作分线职责、写集边界和回交要求已冻结。
- checkpoint 文档同步规则已冻结。

K0 不接受为：

- K1-K6 已完成。
- Stage K 已完成。
- 真实 Codex 执行已获新授权。
- 通用自由 Codex 控制台已开放。
- 任意目录无限制执行已开放。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 自动 retry / stop / restart 已实现并可无确认执行。
- 记忆层可自动写正式记忆。
- 完整真实 Tauri 产品验收完成。

## 3. Stage K 总边界

Stage K 必须做：

- 智能体页日常可用：选择项目、选择 / 新建对话、输入任务、发送前确认、执行状态、读回结果。
- 通用 Codex 执行入口：`resume` 和 `new session` 都必须经过 Product Command、permission envelope、runtime log、audit、readback。
- 项目工作流真实派发：用户目标生成 run units，至少开发 / 验证 / 回收链路可追溯。
- 运行中和待办产品化：执行中、阻断、失败、等待确认、读回不可用必须清楚显示。
- 记忆层体验产品化：执行结果和工作流事实进入 observation / candidate，用户可确认 / 拒绝 / 编辑 / 延后。
- 真实 Tauri dogfood：核心路径必须有真实桌面验收证据或明确缺口。

Stage K 禁止做：

- 不接入 Claude Code / OpenClaw / OpenCode / OpenCode-like 的真实执行。
- 不做 provider credential store、真实 token 读取或 model verification。
- 不开放无限制任意目录读写。
- 不让 agent 自治批准高风险权限。
- 不自动写正式记忆。
- 不无确认自动 retry / stop / restart。
- 不把完整 transcript、secret、token、`.env`、keychain、OAuth、provider credential、rollout 写入普通 sidecar、runtime log、audit、memory observation、memory candidate 或 UI。
- 不把普通浏览器 smoke 当作真实 Tauri 验收。

## 4. K1-K6 任务顺序和授权矩阵

| 任务 | 类型 | 是否允许真实 Codex | 是否允许读写 `/Users/yoyi/.codex` | 是否允许写项目文件 | 是否同步权威入口 | 冻结边界 |
| --- | --- | --- | --- | --- | --- | --- |
| K0 范围 / 权限 / 验收矩阵冻结 | 文档 / 只读复核 | 不允许 | 不允许 | 不允许 | K0 完成时同步 checkpoint | 冻结 Stage K 边界，不开发 |
| K1 智能体对话页日常可用重构 | 前端 + UI 测试 | 不允许新增真实执行 | 不允许新增 `.codex` 访问 | 不允许写项目文件 | 默认不同步；K1 完成可写 evidence/handoff | 只改 UI 信息层级和交互，不改执行语义 |
| K2 通用 Codex resume / new session 产品入口 | 后端 + 前端 + 真实执行 checkpoint | 允许，但仅在 K2 执行点逐项授权后 | 允许，但仅限目标 session / new-session 必要最小范围 | 允许，但仅限授权测试项目和 allowed roots | K2 完成必须同步 checkpoint | 必须走 Product Command；无确认不发送 prompt |
| K3 项目工作流真实派发闭环 | 工作流 + 执行 + UI | 允许，但只能通过 K2/K3 product command 调度 | 只能继承 K2/K3 明确授权范围 | 仅限授权测试项目和 run unit 范围 | K3 完成建议同步 checkpoint | run units 必须可追溯；失败不能包装成成功 |
| K4 记忆捕获 / 候选确认 / 任务记忆注入体验 | 记忆层 + UI + 测试 | 默认不新增真实执行；消费 K2/K3 事件来源 | 默认不新增 `.codex` 访问 | 默认不写项目文件 | K4 完成必须同步 checkpoint | 不自动写正式记忆；敏感材料不得进入记忆 |
| K5 运行中 / 待办 / 失败恢复 / 操作控制 | 状态 UX + 安全边界 | 真实 retry / stop / restart 必须单独授权；默认只做 proposal / confirmation flow | 仅限已授权 run 必要范围 | 仅限已授权 run 必要范围 | 默认不同步；若开放真实操作必须同步 | 不做无确认自动 retry / stop / restart |
| K6 真实 Tauri dogfood 和验收收口 | 验收 + 文档 | 不允许新增真实执行；只验收 K1-K5 已授权路径 | 不允许新增 `.codex` 范围 | 不允许新增项目写入范围 | 必须同步 checkpoint | 冻结 accepted / deferred / blocked |

任何 K2-K5 的真实执行点在执行前必须重新列明：

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
| `mario test` | `/Users/yoyi/Documents/mario test` | 历史真实 resume / 项目工作流 probe 参考；K2/K3 可作为受控验收项目 | K0 不授权；K2/K3 执行点必须重新授权 | 不能因为 E/H/PCR/J 旧 probe 成功就默认继承权限 |
| 工作台自身项目 | `/Users/yoyi/workspace/product-line` | dogfood；验证工作台能服务自身项目 | K0 不授权真实执行；K2/K3 若使用必须列 allowed roots | 禁止修改权威入口以外文件，除非任务包明确列出 |
| Stage K 隔离测试项目 | `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project` | 默认优先的新隔离项目；用于 new session、workspace-write、记忆闭环 | K0 不创建；K2 或 K3 可在任务包中创建并授权 | 必须可安全删除或回滚，不包含 secret / `.env` |
| 真实业务项目 | 用户单独指定 | 非默认；仅在用户明确指定且任务包列出备份 / 回滚时使用 | 默认禁止 | 不得在 Stage K 默认验收中使用 |

测试项目必须满足：

- allowed write roots 明确且足够窄。
- denied paths 明确。
- 不包含 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 可通过 file hash、runtime log、audit、readback、worker report、memory candidate refs 交叉验证。
- 写入型任务必须记录预期写入文件和回滚方式。

### 5.1 候选测试项目登记表

下表只冻结 K2/K3/K6 可引用的登记字段，不授权真实执行，不创建新项目，不继承旧 probe 权限。

| 项目 | project_id | 已知 / 候选 session | baseline 要求 | allowed write roots | denied paths | readback marker 策略 |
| --- | --- | --- | --- | --- | --- | --- |
| `mario test` | `project:users-yoyi-documents-mario-test` | 历史总指导 session `019e798a-6ce5-76c3-b8ee-33bd0fda841f`；历史开发线 session `019e798a-ac37-7771-b982-e38084fcd22e`；均仅为历史参考，K0 不授权复用 | K2/K3 真实执行前必须记录 `index.html`、`styles.css`、`game.js`、`README.md` 或任务包指定文件 hash | read-only 默认空；workspace-write 只能写任务包列明的 `.workbench/stage-k/**` 或更窄路径 | secret、`.env`、auth/token、未列入 allowed roots 的项目文件、用户未授权路径 | 每个真实 probe 必须有唯一 marker，例如 `K2_RESUME_*`、`K2_NEW_SESSION_*`、`K3_RUN_UNIT_*`，或明确失败分类 |
| 工作台自身项目 | `project:users-yoyi-workspace-product-line` | K0 不指定 session；K2/K3 若 dogfood 必须单独选择 / 创建 session | 真实执行前必须记录待写文件 hash；默认不允许改权威入口以外文件 | 默认不授权；如授权只能写任务包列出的 docs/tasks/evidence/handoff 或隔离 fixture | 源码、secret、`.env`、`.git` 写入、未列明权威入口、用户未授权文件 | 必须能回链 Product Command、runtime log、audit、readback 和 handoff |
| Stage K 隔离测试项目 | `project:users-yoyi-workspace-product-line-test-fixtures-stage-k-isolated-project` | K0 不创建 session；K2 可创建 new session | 创建前记录目录不存在或空目录状态；执行后记录新增文件 hash | 只能写 `/Users/yoyi/workspace/product-line/test-fixtures/stage-k-isolated-project/**` | `product-line` 其他路径、`.codex` 外显数据、secret、`.env`、provider credential | new session 成功必须有 last message marker；失败必须保留 readback failed / unavailable 分类 |
| 真实业务项目 | 待用户指定 | 待用户指定 | 必须单独备份 / hash / rollback | 默认禁止 | 默认全部禁止 | 不进入 Stage K 默认验收 |

### 5.2 真实执行点字段工作表

K2/K3/K5 的任何真实执行点必须在任务包中填完整以下字段，缺一项即阻断：

| 字段 | 要求 |
| --- | --- |
| `execution_point_id` | 全局唯一，包含阶段、checkpoint、项目和操作类型 |
| `operation` | `resume`、`new_session`、`retry`、`stop`、`restart` 或其他明确操作 |
| `adapter_id` | Stage K 默认只能是 `codex-local` |
| `project_root` / `project_id` | 必须为绝对路径和稳定 project id |
| `workflow_id` / `run_unit_id` / `node_id` | 工作流派发必须填写；自由对话可使用 temporary run，但必须可追溯 |
| `target_session_id` | `resume` 必填；`new_session` 必须写明创建后如何记录 session id |
| `sandbox` | `read-only` 或 `workspace-write`；workspace-write 必须列 allowed roots |
| `allowed_write_roots` | 必须是绝对路径，且尽量窄 |
| `denied_paths` | 必须包含 secret、`.env`、auth/token、provider credential、未授权项目路径 |
| `prompt_summary` / `prompt_ref` / `prompt_hash` | 必须填写；prompt body 默认只作为运行时输入，不持久化 |
| `task_memory_packet_ref` | 必须说明 included / excluded / review materials；无记忆包也要说明原因 |
| `permission_envelope_ref` | 必须有用户可读影响范围和确认记录 |
| `readback_plan` | 必须说明 expected marker、last message、失败分类和 `result_count=null` 规则 |
| `runtime_log_policy` | 必须写入工作台 runtime log 或说明阻断原因 |
| `audit_policy` | 必须写 audit refs 或说明阻断原因 |
| `baseline_hashes` | 写入型或重要 read-only probe 必须记录执行前后 hash |
| `.codex_scope` | 必须说明是否允许 Codex 自身状态写入、是否读取目标 session 最小结果 |
| `dirty_worktree_policy` | 若目标项目有非本轮改动，必须记录并不得回退用户改动 |
| `rollback_policy` | 写入型任务必须写回滚或清理方式 |
| `user_confirmation` | 高影响真实执行必须 `confirmed_by: "user"` |

K5 的 `retry` / `stop` / `restart` 特别规则：

- 默认只能做 proposal / readiness / confirmation flow。
- 真实 retry / stop / restart 必须作为独立执行点填写上表。
- 不能把 stop / restart 按钮显示成已实现真实能力，除非已有对应真实验收。

## 6. 路径、数据和敏感信息边界

默认允许的写入：

- `product-line/tasks/**`、`product-line/evidence/**`、`product-line/handoffs/**` 的 Stage K 文档记录。
- 工作台自有受控 sidecar / store：ProductCommand、continuation、runtime log、audit、workflow state、memory observation / candidate，且必须由产品代码路径写入。
- K2/K3/K5 任务包逐项授权的隔离测试项目路径或 `mario test` 子路径。

默认禁止：

- `/Users/yoyi/.codex`，除非 K2/K3/K5 的真实执行任务包逐项授权。
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

Stage K 记忆捕获必须使用分层策略：

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

Stage K 后续 UI 显示边界：

| 层级 | 应显示 | 禁止显示 |
| --- | --- | --- |
| 普通用户层 | 项目、对话、输入、发送前确认、执行状态、结果摘要、待确认事项、记忆候选摘要 | raw Product Command、runtime log refs、audit refs、sidecar path、store revision、readback enum、阶段编号 |
| 详情层 | 写入范围、记忆包 included/excluded/review、权限原因、readback 人话解释、worker report 摘要、evidence / handoff 链接 | secret、token、完整 transcript、完整 rollout、provider credential |
| 设置 / 开发者层 | diagnostics、legacy 状态、adapter / provider raw boundary、raw refs、旧入口、内部枚举 | 真实凭据、完整 secret、绕过确认的执行按钮 |

K1/K5/K6 涉及 UI 时必须额外验证：

- 普通入口不出现 `Product Command`、`runtime log`、`audit refs`、`readback enum`、`sidecar`、`store revision`、`H/J/PCR` 阶段术语。
- 开发者内容默认折叠或进入 `设置 > 开发者`。
- 真实 Tauri 验收不能用普通浏览器 smoke 冒充。

## 10. 多会话协作分工

Stage K 默认使用多会话协作，但不生成大量一次性线程。

角色规划：

- `主管线`：本线程。负责目标冻结、任务包、权限边界、分线协调、最终复核和入口同步。
- `复核线`：复用现有审查线程。负责只读审查计划、代码架构、越界风险和验收报告；默认不改代码。
- `UI 线`：K1/K5/K6 前端信息层级和真实 Tauri 体验；写集以 `src/views/**`、`src/styles.css`、前端测试为主。
- `Execution 线`：K2 后端执行链路；写集以 Rust runner / commands / types / TS wrapper 为主。
- `Workflow 线`：K3 项目工作流 run units / handoff / readback / process fact；写集以工作流读模型和项目页为主。
- `Memory 线`：K4 memory capture / observation / candidate / FormalMemory UX；写集以记忆 store / UI / tests 为主。
- `Validation 线`：测试矩阵、Tauri 截图、扫描、evidence 汇总；默认不改产品代码。

协作规则：

- 不同开发线必须有明确写集，避免同时改同一文件。
- 同一主题尽量复用同一对话线，不要每个小补丁新建线程。
- 开发线回交必须写清：改动文件、验证结果、真实执行 / `.codex` / secret 边界、未完成项。
- 复核线输出 P0/P1/P2，不直接替开发线扩范围。
- 主管线不催促正在工作的开发线，但必须在 checkpoint 收口前做独立复核。

## 11. K1-K6 验收矩阵

| Checkpoint | 必须通过 | 不可冒领 |
| --- | --- | --- |
| K1 | 智能体页像对话工作区；普通层只见项目 / 对话 / 消息 / 输入 / 状态；开发者内容后撤；前端验证通过；至少一张真实 Tauri 截图或明确缺口 | 不代表真实执行入口完成 |
| K2 | 至少一次真实 `resume` 和一次真实 `new session` 在授权测试项目通过；runtime log / audit / readback 完整；失败分类正确 | 不代表项目工作流自动派发完成，不代表任意目录执行 |
| K3 | 至少一次项目工作流 run unit 真实派发成功；运行中工作流可见；失败进入待办；C5/C6 可追溯 | 不代表多 provider / planned adapters 完成 |
| K4 | 至少一个执行结果生成 observation / candidate；用户确认后可写 FormalMemory record/version/audit；下一次任务能展示 task memory packet | 不代表所有操作自动写正式记忆 |
| K5 | 运行中 / 待办 / 失败恢复信息清晰；retry / stop / restart 默认只做 proposal / confirmation；unknown result 不显示为 0 | 不代表无确认自动重试完成 |
| K6 | `mario test`、工作台自身项目、隔离项目至少覆盖核心路径；真实 Tauri 核心截图；Stage K accepted/deferred/blocking 冻结 | 不代表最终蓝图完整工作台 |

## 12. 验证要求

每个涉及代码的 checkpoint 默认运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 相关 Rust 聚焦测试
- `cargo test --lib` 或记录无法运行 / 既有失败
- `rustfmt --check` 相关文件
- 误导文案扫描
- 敏感路径 / 真实执行扫描

K0 只读 / 文档验证：

- 新计划和 K0 任务包存在。
- K0 不写成真实执行授权。
- K0 不写成 Stage K 已完成。
- K1-K6 前置关系清楚。
- UI 信息层级、测试项目、真实执行授权和 checkpoint 同步规则完整。
- 复核线无 P0/P1 阻断。

涉及真实执行的 checkpoint 额外记录：

- 执行前 hash / baseline。
- 执行后 hash / baseline。
- allowed write path 证明。
- prompt summary/ref/hash。
- readback last message 或失败分类。
- runtime log refs。
- audit refs。
- `.codex` 副作用范围。
- 回滚 / 清理说明。

## 13. 文档同步规则

必须同步入口的时机：

- K0 完成。
- K2 完成。
- K4 完成。
- K6 完成。
- 出现阻断、权限事故、真实执行失败且影响边界。
- 阶段目标或非目标变化。

不必同步入口的时机：

- 单个 UI 文案修补。
- 单个 CSS / 信息层级小改。
- 测试断言调整。
- 开发者详情位置移动。
- 不改变产品能力的小型重构。

K0 完成时建议同步：

- `CURRENT.md`
- `tasks/README.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- K0 evidence / handoff

## 14. K0 验收清单

- [ ] K0 任务包创建。
- [ ] K0 evidence 创建。
- [ ] K0 handoff 创建。
- [ ] 复核线只读审查无 P0/P1。
- [ ] 扫描无“Stage K 已完成”误口径。
- [ ] 扫描无“授权直接真实执行”误口径。
- [ ] 权威入口在 K0 收口后同步到“Stage K / K0 已完成，下一步 K1/K2”。
- [ ] 明确下一步分线：K1 UI 线和 K2 Execution 线可并行，但 K2 真实执行必须另行授权。

## 15. 下一步

K0 收口后，推荐进入两个并行准备线：

1. K1 UI 线：智能体对话页日常可用重构。默认不授权真实执行，不改后端执行语义。
2. K2 Execution 线：通用 Codex resume / new session 产品入口。先写 K2 任务包和执行授权矩阵，再开发；真实执行点必须单独确认。

K3/K4/K5 等 K1/K2 的接口和事件来源稳定后再推进。
