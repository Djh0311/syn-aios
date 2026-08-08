# Syn Stage 3：角色会话与显式交接计划 v1

日期：2026-08-01<br>
阶段：`M3`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M3。<br>
硬前置：M1 identity / scope / policy 合同和后端守卫通过；M2 UoW / event / audit / receipt ports 可用。<br>
当前 active node / package：`NONE`；本计划不授权消息、Codex、App、store 或产品代码操作。

权威顺序：当前用户指令 → `../../AGENTS.md` → Harness Lite `../harness/plan.md` / current stage / current leaf / `../harness/authorization.json` → 2026-08-01 修订与当前 inventory → master → M1/M2 exit receipts → 本计划。下文的目标对象均是计划，不是当前实现事实。

## 0. 当前事实与未知

### 已有局部基座

- shared conversation transport 已有两种固定 profile：agent workspace-write 与 supervisor read-only，并支持 new / existing、poll、stop。
- supervisor 路已有 durable per-turn binding material 和严格 project / workflow 对照，可作为可信 binding 素材；它不是 durable RoleSession。
- Codex SQLite / rollout 可提供外部会话索引；旧 session continuation store 可提供执行型 resume 语义参考。
- 现有离线测试覆盖 transport controller 与部分 DOM 行为。

### 尚未成立

- 没有统一、持久的 `RoleSession`、`Turn`、`ConversationContext`、`CurrentObjectRef`、`ExecutionChannel`、`PermissionSnapshot`；
- agent existing 路缺少与 supervisor 同等级的 thread → project owner 后端校验；
- in-flight attempt 依赖 Rust 进程内 map，重启恢复没有正式入口；
- Jiaoban 与 Agent Center 各自用 React / module `Map` 持有选择、transcript 和发送态，App 重启丢失；
- supervisor turn binding 只有短期受信 turn，不是通用 RoleSession；
- offline role handoff 是人工写 workflow state 的旧路，不是目标 Handoff；
- 当前没有真实桌面 new / continue / stop / restart、跨项目拒绝或真实 Codex 成功证据。

### HOLD / 需冻结决定

- provider thread 的 natural key、重复索引、thread collision 和 orphan recovery；
- raw transcript 是只读引用、摘要还是缓存，以及保留 / 删除策略；
- in-flight 子进程重启后是 resume、orphan、failed 还是 user decision；
- Handoff 超时、拒绝、取消、重试与结果回源语义；
- 各角色真实 provider、模型、成本和消息权限；
- Station 3b、项目写、真实 Codex 消息的实际验收。

## 1. 阶段目标

1. 一个持久会话固定绑定角色、scope、当前对象、执行通道和 permission snapshot；
2. provider thread / conversation id 只作为外部 handle，不拥有 Syn 的会话身份；
3. App 重启后从 Syn 真源恢复正确会话，不依赖前端 module cache；
4. transport 退化成 start / poll / stop / resume adapter，不能自行决定 scope 或推进业务状态；
5. 建立显式 Handoff、接单、拒绝、取消、回执和结果回源；
6. 普通聊天保持普通聊天，不静默生成 workflow、task、formal memory 或 authorization；
7. 为 M4 Secretary、M5 Project Supervisor、M6 Global Supervisor 提供同一角色会话底座。

## 2. 本阶段不做

- 不实现完整 Secretary attention、项目执行、全局跨项目分析、记忆治理或 connector；
- 不让 Handoff 变成自动授权或自动派活；
- 不复制 Codex 原始 transcript 为第二事实源；
- 不让前端传入 role、scope、station 或 permission 后由后端直接信任；
- 不把 stop request 冒充进程已停止，也不把 thread id 存在冒充会话可恢复；
- 不删除旧 frontend cache / continuation / manual relay，直到新 primary parity 与回切通过；
- 不以离线 controller / DOM 测试代替真实 Tauri / Codex 证据。

## 3. 核心对象与 owner

| 对象 | owner / 真源 | 最小约束 |
|---|---|---|
| `RoleSession` | Conversation domain | role、scope、object、channel、permission revision、status、created/resumed timestamps |
| `Turn` | RoleSession aggregate | actor、input ref/hash、provider attempt、terminal status、receipt；raw transcript 不进 event |
| `ProviderHandle` | Conversation / RoleSession repository | provider、conversation/thread id、owner fingerprint、last verified；adapter 只校验 / 使用 handle，不拥有映射，也不把 handle 当授权 |
| `ConversationContext` | application projection | source refs、summary、current object、可选 external-context refs；可重建。memory packet 生命周期仍归 M5/M7 |
| `Handoff` | Handoff aggregate | from/to、scope、outcome、refs、risk、permission request、status、receipt |
| transcript | provider / Codex 真源 | Syn 只持引用、受控摘要或经合同允许的 cache |

稳定 join：`RoleSessionId + RoleRef + ScopeRef + CurrentObjectRef + ExecutionChannel`；provider handle 必须反查 owner fingerprint。线程冲突或无法归属时 quarantine，不猜项目。

## 4. 任务切片

### SYN-SES-001 — RoleSession / Turn / Handoff 合同冻结

只写合同与 migration matrix。冻结 natural key、状态机、permission drift、thread collision、orphan、transcript、timeout/cancel、idempotency、source ref 和 rollback。

### SYN-SES-002 — 后端 owner / scope 止血

依赖 M1 安全包。为 Agent existing 增加 thread → project / role / scope 校验；role、station、channel、profile 均由服务器 resolver 决定；跨项目与 Station 3b 写在 spawn 前拒绝。

### SYN-SES-003 — RoleSession repository 与 shadow import

在 M2 UoW 上实现 RoleSession / Turn / ProviderHandle repository。迁移真源只限 Codex SQLite / rollout、durable supervisor binding 和有效 continuation records；只迁 binding / refs / hashes，不复制 raw transcript。两套前端 cache 不是迁移真源，只可在同一进程内做 parity telemetry。

### SYN-SES-004 — ConversationTransportPort

提取 start / continue / poll / stop / resume adapter；adapter 只消费冻结 context 和 grant。先 fake provider 验证 terminal state、timeout、cancel、restart，再按角色逐项申请真实 Codex 场景。

### SYN-SES-005 — 显式 Handoff 状态机

实现无歧义状态图：`created → accepted → returned`；或 `created → rejected | cancelled | expired`。另定义 accepted 后 return failure、receipt lost、超时、重复接单、原对象不存在和结果回源失败。Handoff 只能请求能力 / 权限，不能自行生成授权。

### SYN-SES-006 — 会话读模型与前端收口

在 DTO 冻结后接角色入口、固定上下文标签、历史会话目录和 source link。旧 Jiaoban / Agent Center cache 先 compatibility read-only；M9 前不物理删除。

### SYN-SES-007 — 隔离与真实 App 分层验收

先用 fake provider + isolated profile 覆盖每种角色 new / continue / stop / restart；真实 Codex 消息、真实项目 root 和 provider 进程分别单独授权。

## 5. 顺序、并行和写面

```text
SES-001 → SES-002 → SES-003 → SES-004 → SES-005
                              └──────────→ SES-006 → SES-007
```

- RoleSession repository / transport / provider binding 由会话线单写；
- M1 的 scope / policy、M2 的 UoW / schema、公共 `commands.rs` / AppState 仍由各自 owner 接线；M3 不越权改公共机制；
- Rust domain 与 React read-model consumer 可在 DTO 冻结后并行；
- SES-004 合同稳定后，M4/M5 只可做契约设计、只读 adapter 或隔离 fixture；涉及状态 / 真源的实现必须等待 SES-003、SES-005 和本阶段 exit 全部通过；
- `commands.rs`、command registry、App assembly、shared transport 与 frontend shell 都必须记录唯一 writer；
- 现有 dirty WIP 与目标文件 hash 不符即停，不覆盖会话修复 WIP。

## 6. 迁移与回滚

- 新 RoleSession 先 shadow 观察现有会话选择，不反写 provider 真源；
- 无法精确绑定的 thread 标为 `orphaned / ambiguous`，保留来源，不自动分配项目；
- permission snapshot 续接时重新核当前 policy；只允许保持或收窄，升级需新确认 / grant；
- in-flight restart 必须从 durable attempt receipt 恢复；没有恢复证据则进入 user-visible orphan / failed，不静默重启副作用；
- 旧 frontend cache 在观察期只作为显示 fallback，不是 owner；
- rollback 只允许切旧 UI / read path；M1 thread-owner、scope 和 Station 3b 后端 guard 始终保留。旧 adapter 无法通过这些 guard 时关闭发送，不恢复跨项目 bypass；
- 旧路 export / manifest 保留到 M9 独立退役。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / state fixtures | 状态、owner、collision、handoff 语义一致 | persistence 已实现 |
| Unit / property | role/scope 绑定、permission drift、幂等、跨项目拒绝 | provider 可用 |
| Temp repository + fake provider | new/continue/poll/stop/restart/orphan/handoff recovery | 真实 Codex / App 通过 |
| Non-test build | production path 可构建 | UI / 进程行为正确 |
| Isolated Tauri | 持久会话、重启、固定标签、拒绝可见，并保留真实桌面窗口截图 / 可见交互 / deep-link 点击证据 | 真实消息成功 |
| 经授权真实 Codex | 指定 role/profile/root 的单一消息场景 | 所有角色、项目或发布通过 |

真实 App 必须直接观察：每种角色新建 / 续接 / stop；在 start / pending / terminal 三个点分别强退与重启；同一 turn / idempotency key 不重复 provider send 或其他外部副作用；角色、项目、对象、通道精确恢复；跨项目与 Station 3b 在 spawn 前拒绝；Handoff 重放不重复结果。`thread_id` 或 binding 存在本身不是成功。

## 8. 独立授权与停止条件

scope / security 接线、local schema / store migration、App 启动 / 强制退出、真实 Codex 消息、真实项目 root、provider 进程、旧会话写路关闭分别建包。M3 不授权真实项目修改、runner、外部 connector、凭据、Git 或发布。

立即停止：thread owner 不唯一；restart 只能猜；permission 静默升级；前端隐藏替代后端拒绝；Handoff 自动生效为授权；需要复制 raw transcript；adapter 自行改业务状态；写面撞未归属 WIP；离线测试被表述成真实消息。

## 9. 阶段退出与下游交接

全部满足才允许 M4 / M5 进入实现：

- RoleSession / Turn / ProviderHandle / Handoff 合同与 repository 版本冻结；
- 每种角色的 owner、scope、channel、permission 与 provider handle 可持久恢复；
- 跨项目、伪造 thread、Station 3b、permission drift 在 spawn 前 fail closed；
- Handoff 可接单 / 拒绝 / 取消 / 回源，重试幂等；
- 两套 frontend cache 不再是事实 owner，旧路有 manifest / fallback；
- 隔离 App 证据通过；真实 Codex 结论按实际授权单独记录；
- CURRENT 回写完成 / HOLD / 下一步；未显式激活不得自动进入 M4 或 M5。
