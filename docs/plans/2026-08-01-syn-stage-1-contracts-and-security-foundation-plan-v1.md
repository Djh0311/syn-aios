# Syn Stage 1：合同与安全 / 作用域基础计划 v1

日期：2026-08-01<br>
阶段：`M1`<br>
状态：**CLOSED / ACCEPTED 2026-08-03（用户拍板，见 `decisions/2026-08-03-syn-m1-closure-acceptance-v1.md`）；M2 后续已完成，M3 仍为 PLANNED / NOT_ACTIVE。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M1。<br>
当前权限：本计划已经关闭，只作 M1 阶段合同与验收范围记录；它不再授权任何产品代码，也不激活 M3 或后续阶段。

`SYN-FND-002`—`SYN-FND-005` 会改安全闸、作用域或授权判断，属于 `AGENTS.md` 高危清单；即使写在本阶段里，也必须每包由用户单独明确授权，不能因 Stage 1 已是 current 就自动连跑。

## 0. 权威、现状与入口冻结

当前权威链：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 活动阶段（stage）/ 唯一活动叶（leaf）→ `../harness/authorization.json` → `../current-state.md` → 两份 2026-08-01 正式修订 → 上位 master。M1 关闭事实再回看本计划和验收记录；旧 conversation-first、Jiaoban rebuild、R7 和知识专题计划只作历史证据，不能从中恢复执行权。

下表依据的是 M1 启动时的脏工作树静态盘点，只保存当时输入；当前事实改看 `../current-state.md`、当前源码与 M1/M2 主线验收，不把下表重新解释成今天的状态：

| 维度 | 当前事实 | 本阶段必须处理 |
|---|---|---|
| command / policy | `command_registry.rs` 主要是命令注册清单；不存在覆盖 Tauri、MCP、runner、connector、job 的统一 identity / policy gateway | 建立全入口 inventory 和 `migrated / guarded-legacy / blocked / not-in-scope` 状态 |
| conversation scope | Agent Conversation 固定使用 workspace-write，后端没有稳定 role / station / channel / permission snapshot；existing agent thread 缺少与 supervisor 同等级的 owner 校验 | 服务器侧解析身份与 scope，在 spawn 前拒绝伪造、跨项目和 Station 3b 写入 |
| Canvas path | legacy MCP storage 把 canvas / run / node / template ID 直接用于路径拼接，`canvas_load` 还会把广义 load error 当成新建条件 | 统一 ID、realpath、symlink 和 bootstrap-not-found 守卫 |
| workflow owner | workflow 列表仍有 slug `contains` 归属兼容；node query 丢弃 project root；更新入口没有统一 owner guard | workflow record 的 `project_id` 作为 owner 真源；无法精确映射即隔离 |
| report / execution | WorkerReport 没有完整绑定 project / dispatch / attempt / authenticated actor；部分执行入口仍消费 caller 形状或布尔授权矩阵 | 拆分 owner、report、ExecutionGrant 三个高风险包，不在一个大文件中混改 |
| audit / secret | Audit Ledger 产品 DTO 仍可携带多数来源的 raw JSON；序列化层没有统一 scrubber | 产品 DTO 只出摘要 / ref / hash；敏感内容机械拒绝或脱敏 |
| storage | SQLite 是局部 DB-primary bridge，多个 JSON sidecar 仍是 live source；通用 event / audit / outbox / receipt 表尚不存在 | M1 只冻结合同与迁移输入，不提前创建第二套临时事件真源 |

当前 HOLD / UNKNOWN：dirty tree 上 DB-primary 与 JSON 降级实况、真实 App / store / Codex 表现、现存运行数据是否含敏感原文、所有注册入口是否已覆盖、unknown / corrupt sidecar 的最终处置。它们必须在相应 task package 中直接核验，不能由本计划推断。

进入任何代码切片前必须冻结：当前 HEAD / status、M0 交接、十份合同版本、全入口 inventory、store / table / sidecar owner 与 join key、目标文件 opening hash、允许写面和单写者。写面撞上未归属 WIP 即停止。

## 1. 阶段目标

在不改产品外观、不迁移业务真源、不启动真实消息 / 外部连接的前提下，先冻结后续所有模块共同依赖的合同，并修掉会让新架构建立在错误作用域或不可靠权限上的早期风险。

阶段结束时必须得到：

- 一个无 IO 的 identity / scope / role / channel / object ref 合同；
- 一份覆盖所有入口的盘点，以及已迁移 / 高风险入口共用的 policy / authorization gateway；未迁入口必须精确标为 `guarded-legacy / blocked / not-in-scope`；
- command / event / audit / outbox 和敏感信息边界；
- conversation / handoff / open-loop / decision / workflow / memory / connector 的最小 schema；
- 当前静态发现的跨项目、Station 3b、路径 ID、workflow owner、worker report、execution grant 和 audit secret 风险的 fail-closed 修复；
- M2 可直接使用的 migration / parity / rollback 清单。

## 2. 本阶段不做

- 不重构首页、项目页视觉或项目页布局；
- 不建立完整 Secretary、Global Supervisor 或 Project Supervisor 产品闭环；
- 不切换 DB-primary，不迁移正式业务数据；
- 不接真实邮件、日历、OpenConnector 或其他外部 provider；
- 不读取或迁移真实凭据；
- 不运行真实项目写入、自动连环或发布；
- 不删除旧 command、store、JSON、会话或工作流路径；
- 不把合同文件、单测或 fake runner 当成真实桌面验收。

## 3. 冻结的合同

Stage 1 必须在 `docs/contracts/` 建立或等价地冻结以下版本化合同：

| 合同 | 最小字段 / 决定 | 下游使用者 |
|---|---|---|
| `identity-scope-v1` | ProjectId/Root、ScopeRef、RoleRef、CurrentObjectRef、ExecutionChannel、PermissionProfile | 全部 command / UI / adapter |
| `command-v1` | command_id、actor、scope、object、channel、expected revision、idempotency | application / policy |
| `event-audit-outbox-v1` | envelope、payload limits、sensitivity、correlation、receipt、retry | M2 事务底座 |
| `role-session-v1` | role、scope、object、channel、provider handle、permission snapshot、resume rule | M3 会话 |
| `handoff-v1` | from/to、scope、requested outcome、refs、risk、permission request、receipt | M3 / M4 / M6 |
| `attention-decision-v1` | OpenLoop / DecisionRequest、source、owner、reason、priority basis、dismiss / close | M4 秘书 |
| `project-orchestration-v1` | Proposal、Authorization、Run、WorkItem、PreparedAttempt、ExecutionGrant、Dispatch、Report、Review、Decision | M5 项目执行 |
| `memory-personal-model-v1` | observation/candidate/formal、fact/inference、policy result、conflict、version | M7 记忆 |
| `connector-capability-v1` | view/index/sync/action/secret、grant、credential ref、inbound/action result | M8 连接器 |
| `object-ref-navigation-v1` | type、scope、id、source ref、deep-link resolution | 所有前端读模型 |

每个合同必须写：唯一 owner、真源、合法状态、跨 scope 规则、事件、审计、敏感字段、幂等、失败、回滚、版本兼容和明确不做。

### 3.1 owner 与身份裁决

| 对象 | 唯一 owner / 真源 | 不得作为 owner 的对象 |
|---|---|---|
| Identity / Scope 解析 | `identity_scope` kernel；project root 由 project index 解析为稳定 `ProjectId` | 前端自报 role、station、permission |
| Workflow owner | workflow record 的 `project_id` | workflow id 中的 slug、未经核验的 root 字符串 |
| Policy decision | 纯 `policy` kernel，由统一 gateway 强制执行 | UI 隐藏、adapter 自行放行 |
| Authorization | Proposal → PlanAuthorization store | caller 传入的布尔矩阵 |
| ExecutionGrant | 服务端依据 authorization revision 生成的 immutable grant | runner 或模型自报范围 |
| WorkerReport | worker claim ledger；与 exact attempt / actor 绑定 | report 本身直接冒充 verified fact |
| Domain state | 各 aggregate | event、audit、snapshot 或兼容 JSON |
| Event / Audit / Outbox | M2 公共机制 | 全局业务事实池 |
| Product audit view | scrubbed read model | raw store JSON |

## 4. 任务切片和顺序

### SYN-FND-001 — 合同与迁移基线（文档切片）

依赖：无。<br>
建议写面：`docs/contracts/**`、本 stage plan 的验收附件、draft task package；不改 product source。

交付：

- §3 十份合同；
- 旧对象 / command → 新对象 / port / retire 的逐项矩阵；
- 当前 store、表、sidecar、projection 的 owner / join key / migration 盘点；
- M1 测试矩阵、M2 shadow-write / parity / rollback 输入；
- 所有 HOLD 写成显式 open design item，不在代码里临时决定。
- 冻结 OpenLoop 与既有 Todo 的语义和物理关系；在合同决定前，不新建第二套含义重叠的待办真源。
- 列出所有 Tauri / MCP / runner / background job 入口，给出 owner、scope、policy、当前 bypass 状态和迁移目标；不能只清点“新或被触及”的命令。
- 冻结 WorkerReport 的 acceptable attempt state、manual / offline report kind、authenticated actor / thread binding；对 M2 command receipt、outbox lease、projection checkpoint 和 unknown quarantine 只冻结外部接口、不变量和禁止字段。持久化 / 运行时状态机由 M2 单一 owner 设计，语义变化必须回到 M1 合同版本评审。

验收：合同互相引用无环形 owner；每个正式动作都能回答 command、policy、state、event、audit、outbox；secret / raw transcript / tool output 禁止字段可机械测试。

### SYN-FND-002 — 路径和对象 ID 守卫

依赖：FND-001 的 identity / path 合同。<br>
建议写面：Canvas storage / path validator、新的纯 guard 模块及聚焦测试。

交付：

- 所有进入 `PathBuf::join` 的 canvas/template/run/node ID 统一验证；
- 拒绝空值、`.`、`..`、路径分隔符、绝对路径、percent / unicode 编码变体；
- symlink 后 realpath 仍必须留在允许根；
- 删除、写入、读取使用同一 validator；
- 旧不合法 ID 只报告 / 隔离，不在本包自动删改。
- `canvas_load` 只有在“ID 已验证且目标明确不存在”时允许 bootstrap；解析、越界、权限和其他 IO 错误必须 fail closed。
- legacy MCP tool 与 worker Markdown outbox 的内部路径入口也受同一 guard，不因 start / tick 已封存而遗漏 abort / status / load / delete。

验收：表驱动 + property 测试；temp root 的 traversal / symlink 故障注入；拒绝发生在文件系统 mutation 前。

### SYN-FND-003 — Identity / Scope / Policy Kernel

依赖：FND-001。<br>
建议写面：新的纯 Rust kernel 模块、DTO conversion、command gateway、聚焦测试。

交付：

- 类型化 ScopeRef、RoleRef、ObjectRef、ExecutionChannel、PermissionProfile；
- 所有新 / 被触及 command 先 resolve identity / scope，再 policy 判定；
- Agent Conversation 接收服务器生成的 role / scope / station，不信任前端自报；
- Station 3b read-only / zero-write 在后端 enforcement；
- 顶层秘书使用个人 / 全局安全工作目录或无项目根 profile，不再借固定项目目录获得读取面；
- permission profile 在会话续接时不可漂移。

验收：跨项目、缺 scope、伪造 project id、scope/channel 切换、Station 3b write、permission escalation 全部拒绝；fake runner / spy repository 证明零 spawn、零业务状态 / projection / outbox mutation。

### SYN-FND-004A — Workflow 归属止血

依赖：FND-003。<br>
建议写面：workflow query guard、ownership resolver 及聚焦测试；不触碰 WorkerReport 或 execution authorization 入口。

交付：

- `get_project_workflow_nodes` 必须验证传入项目与 workflow owner；
- 删除 workflow id `contains(slug)` 的归属兼容判定；
- 兼容数据无法归属时 fail closed + 生成诊断，不自动猜 owner。

验收：伪造 workflow / project 矩阵；所有拒绝在状态变化之前；合法旧 fixture 通过明确 adapter 而非模糊 contains。

### SYN-FND-004B — WorkerReport 精确绑定

依赖：FND-003、FND-004A。<br>
建议写面：worker report validator / claim ledger、聚焦 adapter 与测试；不与 ExecutionGrant 接线共改承重入口。

交付：

- report 精确绑定 `project_id + workflow_id + work_item_id + node_id + dispatch_id + attempt_id + authenticated_actor + report_hash`；
- 执行型报告只接受合同允许的 attempt 状态；人工 / offline report 使用不同 kind，不冒充真实执行；
- transport 持久化失败必须 fail closed、返回明确错误且不发生业务状态提升；M1 只冻结 durable recovery 合同，持久恢复记录 / result command 由 M2 实现；
- report 仍是 claim，只有 readback / review / user decision 才能升级正式事实。

### SYN-FND-004C — ExecutionGrant 唯一执行授权

依赖：FND-003、FND-004A。<br>
建议写面：grant mint / load / revoke、authorization resolver、分批入口 adapter 与测试。

交付：

- grant 至少冻结 authorization revision、scope fingerprint、principal、PreparedAttempt / Attempt identity、object / command / channel、expiry / revocation 和 grant hash；
- 冻结 ExecutionGrant 是 authorization 的服务端派生快照还是独立持久记录，以及物理位置、revision / CAS、expiry / revoke 原子性和 migration owner；
- workflow、Phase B、runner 只接收 grant id，由服务端加载并复核；
- caller 布尔矩阵、测试路径 path-lock 和 helper 构造的 confirmed 值不得成为正式授权真源；
- 每次只迁一个执行入口；所有已知 caller-controlled execution 入口在 M1 退出前必须 `migrated` 或 `blocked`。`guarded-legacy` 只允许用于已机械证明授权真源在服务端、caller 值不参与放行的入口，并记录证明引用。

### SYN-FND-005 — Event / Audit 敏感边界与统一 Receipt

依赖：FND-001、FND-003。<br>
建议写面：event DTO / validator、secret scrubber、audit read model boundary、error / receipt types 及测试。

交付：

- event payload 只允许 summary / ref / hash；
- raw transcript、prompt、tool output、credential、token、OAuth、`.env` 和 auth 内容机械拒绝或脱敏；
- Audit Ledger 不把各 store 的 raw JSON 原样暴露给普通产品响应；
- allowed / denied / needs_confirmation / committed / external_pending / external_result / projection_degraded 统一 receipt；
- scrub 前后保留可审计的分类和 hash，不保留 secret 正文。

验收：敏感词、结构化 token、嵌套 JSON、错误堆栈、provider response、Unicode / 编码变体测试；产品 DTO 不含 raw secret 字段。

### SYN-FND-006 — Stage 1 集成与真实 App 安全验收

依赖：FND-002—005 全部通过。<br>
建议写面：集成 fixture、隔离 profile 启动脚本 / 验收记录；不改业务功能。

场景：

1. scratch project 的合法 read-only 会话允许；
2. Station 3b write 在后端拒绝；
3. Project A token / workflow 不能读写 Project B；
4. path traversal / symlink 请求零文件变更；
5. 伪造 Authorization / WorkerReport 零 spawn、零状态变化；
6. 现有 `run_secretary_explain` 或等价 one-shot Secretary profile 不读取任一项目原始根；本阶段不声称顶层 Secretary RoleSession 已实现；
7. audit / error UI 只有脱敏摘要；
8. 关闭并重启 App 后，守卫仍生效。

真实 App 只使用隔离 profile、scratch / fixture 数据和 fake runner。真实 Codex、真实项目写入和真实凭据不属于本包。

## 5. 串并行规则

- FND-001 必须先完成并冻结。
- FND-002 可与 FND-003 并行，前提是不共同修改 command registry / AppState 承重文件。
- FND-004A / B / C 依次冻结共同 ID 合同，但允许在写面不重叠后分包；不得由一个“大治理重构”同时改 owner、report、grant。
- FND-005 可在 FND-003 DTO 稳定后并行。
- FND-006 只能在其余切片全部收口后执行。
- 同一 Rust 承重文件单写者；必要时先抽 module，再让后续任务只写新 module。
- `commands.rs`、`command_registry.rs`、`types.rs`、`c4_c6_workflow_governance_entrypoints.rs`、AppState 装配均视为公共承重面；任务包必须给出 opening hash 与唯一 writer。

## 6. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract lint / fixtures | 字段、状态和禁止项一致 | 代码已实现 |
| Rust unit / property | pure guard / policy 在样本空间 fail closed | Tauri command 已全部接入 |
| Temp integration | command 在 fake adapter / temp store 前拒绝；业务状态、projection、outbox 和外部副作用为零 | 真实 provider / App 可用 |
| Non-test build | production code 可构建 | 真实桌面行为正确 |
| Tauri isolated profile | 本机隔离场景允许 / 拒绝真实可见 | 真实项目 / provider / 发布通过 |

含 Rust production 路径的任务至少运行聚焦测试和 non-test build。每个安全修复还要证明被拒请求没有 spawn、业务状态 / projection / outbox mutation、文件写或外部调用。允许向专用、append-only、已脱敏的安全审计 sink 记录一次拒绝；它必须与业务 store 明确分离，避免“零写入”和“拒绝可审计”自相矛盾。

## 7. 迁移与回滚

- 本阶段不删除旧数据，不切真源。
- 新 kernel 先包住被触及入口；未迁入口列入 migration matrix，不假称全覆盖。
- 旧不合法对象只隔离、报告和导出；修复或删除另开迁移任务。
- 每个 guard 支持按精确入口回退代码，但不能通过 feature flag 放宽高风险边界。
- 若合法历史数据被新合同拒绝，先停在兼容 adapter 设计，不得恢复模糊 owner / path 推断。
- M1 的 event DTO / scrubber 只冻结边界；event ledger、UoW、outbox 持久化属于 M2，M1 不建立临时并行真源。

## 8. 阶段退出门

全部满足才进入 M2：

- §3 合同冻结并通过引用 / schema / fixture 检查；
- FND-002、FND-003、FND-004A/B/C、FND-005 聚焦测试和 non-test build 通过；
- FND-006 隔离 Tauri 场景通过并有 before / after 证据；
- 所有已知入口有 `migrated / guarded-legacy / blocked / not-in-scope` 状态；其中 caller-controlled execution 入口全部 `migrated / blocked`，`guarded-legacy` 满足 §4 的服务端授权证明条件；
- workflow、report、grant 均有精确 owner / join identity；无法映射的历史对象进入隔离清单而非模糊匹配；
- 没有以 UI 隐藏代替后端拒绝；
- 没有真实 secret、真实外部动作或真实项目写入；
- dirty WIP 被保留，diff 只包含激活任务允许写面；
- `CURRENT` 回写实际完成、验证、未知和下一任务；
- 用户 / 当前权威明确激活 M2 前，Stage 1 不自行续跑。
- 所有 active node、CURRENT 和 task package 必须使用 `SYN-FND-*` / `SYN-DAT-*` 完整 ID，不得使用裸 `M1` / `M2` 复活历史同名路线。

## 9. 第一个可激活任务

首个任务只能是 `SYN-FND-001` 文档 / 合同包。它不修改产品代码，完成后由指导线核对合同是否覆盖当前源码和两份 2026-08-01 修订，再决定是否激活 FND-002 / FND-003。

建议 active package 必须逐项列出：允许写入的合同文件、禁止产品代码、现状 inventory / source anchor、验收脚本、Git 未授权和完成后的 handoff。
