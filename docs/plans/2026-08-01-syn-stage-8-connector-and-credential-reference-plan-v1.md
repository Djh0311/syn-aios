# Syn Stage 8：Connector 与 CredentialRef 计划 v1

日期：2026-08-01<br>
阶段：`M8`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M8。<br>
分层前置：`CON-000` design-only 仍需当前用户明确指令或 matching active package；“design-only”只缩小写面，不豁免激活规则。framework 实现需 M1/M2 exit；RoleSession / memory / Secretary 集成再等 M3/M7/M4 合同；真实 provider 另需独立授权。<br>
当前路线状态：M1–M4 已完成各自具名范围，M5–M8 未激活。Harness 动态 stage / leaf 另看 `../harness/plan.md`；本计划不授权网络、DSH、真实账号、凭据、付费服务提供方、外部动作、桌面应用或产品代码。

权威顺序：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 活动阶段（stage）/ 唯一活动叶（leaf）→ `../harness/authorization.json` → `../product/syn-product-canon-v1.md` 与 `../product/knowledge-infrastructure-canon-v1.md` → `../current-state.md` → 2026-08-01 修订 → 总计划 → 对应分层前置回执 → 本计划。提前只做设计不得建立生产连接器或凭据真源。

## 0. 当前事实与未知

### 可提取的现有 adapter 素材

- Codex local adapter 已有能力描述；其他 adapter 多为 planned / read-model-only。
- Supervisor MCP 有固定、fail-closed capability registry；`tools/list` 与 `tools/call` 共用授权，可提取为 CapabilityGateway。
- 现有知识文件库与工作区的读取、搜索、打开和引用能力有固定路径与合同边界，可提取为“外部知识来源适配器”的素材；所有智能体共用的知识登记、检索路由和上下文装配仍属于 M7 核心知识层，不在 M8 重新包装成外部适配器。
- Harness、Skill、Plugin 页面能显示索引 / metadata；它们是 inventory，不是运行能力。
- 现有 `CredentialRequirementDescriptor` 只描述 provider / status / read policy，并明确不读取 secret，可保留为诊断 metadata。

### 尚未成立

- 生产源码没有 `ConnectorDefinition`、`ConnectionAccount`、`CredentialRef`、`CapabilityGrant`、`SyncCursor`、`InboundItem` 等 M8 领域对象；
- 没有统一 connector lifecycle、provider data contract、授权 / 撤权、sync cursor、断线或错误状态；
- 没有明确的 credential vault port / backend；未发现可作为现状真源的 Keychain / encrypted vault 实现；
- 现有 ActionRequest / Result 属于项目 / supervisor 语义，不是 connector action contract；
- 某些外部知识来源写入当前可能先落文件、后写审计，存在可靠长事务缺口；
- 没有真实外部 connector、真实凭据、断开 / 撤权、App 显示证据。

### HOLD / 需冻结决定

- 首个 external connector 和 provider 的数据合同；
- 真源、正文 / 引用、同步方向、冲突、删除、撤权、外发、保留期、分页 / cursor；
- CredentialRef backend（Keychain、加密文件或其他）、加密、轮换、撤销、恢复、备份；
- provider 网络、账号、付费、rate limit、数据驻留和隐私要求；
- read / index / sync / action 的实际 capability 边界和确认方式；
- 多 provider、OpenClaw / Claude Code / OpenCode 等真实接入；
- AgentRuntime、Agent / Model、Tool、Harness、KnowledgeSource 与 Connector 的精确端口边界；
- DSH 插件 / MCP Tool 如何只提交 CapabilityRequest / ActionIntent，而不直接取得凭据和 connector write；
- 文件系统 sandbox 之外的网络、进程、同 UID 凭据、插件供应链和外部副作用隔离；
- 真实 App、真实 network 和真实 secret 的验收。

## 1. 阶段目标

1. 统一内部与外部 adapter 的 capability envelope、输入输出 schema、风险、确认、审计、重试和失败合同；不把不同 owner 的端口合成万能 adapter；
2. 建立 ConnectorDefinition、ConnectionAccount metadata、CapabilityGrant、SyncCursor、InboundItem、ActionRequest / Result；
3. `view / index / sync / action / secret` 分开声明、授权、撤销和审计；
4. 建立 opaque `CredentialRef` 与 vault port；普通 DB / event / audit / memory / chat 只存 opaque ref、非敏感 status 和非 secret-derived content hash，不存 secret 或 secret 的直接 hash；
5. 分开抽取 AgentRuntime、Agent / Model、Tool、Harness、KnowledgeSource 与 Connector 端口；DeepSeek Harness 若接入，只属于 `AgentRuntimeAdapter`；
6. 先用 mock provider 走完整合同，再以一个低风险只读 connector 做首个真实样本；
7. 设置 / 管理面展示来源、授权范围、最近同步、错误、断开和撤权，不显示 secret 正文；
8. 写型 external action 永远与只读 connector 分包，并用 M2 outbox / effect id / result command。

## 2. 本阶段不做

- 不把 `credential_status`、环境变量或 provider 可用性冒充凭据仓；
- 不把 read 授权扩大为 sync / action / secret；
- 不把真实 secret 放入 SQLite record_json、event、audit、memory、chat、prompt、日志或截图；
- 不把 token / password / credential 原文直接 hash 后写入普通 store、event、audit、chat 或 memory；凭据完整性指纹只能由 vault 内部持有，或由 vault 生成 scope-bound keyed fingerprint 并按 policy 暴露；
- 不在第一个 read connector 包里顺带开放外部写；
- 不让 adapter 直接推进 domain state 或绕过 policy / grant；
- 不以 mock、index metadata 或 capability list 声称 provider 已接入；
- 不在 M8 物理删除旧内部命令 / adapter；
- 不把开发护栏候选、技能插件或知识索引自动视为可执行连接器。
- 不让 Harness Tool / Plugin 自己解引用 CredentialRef、绕过 ConnectorGateway 或直接写外部系统。
- 不把 MCP Tools bridge 等同于完整 MCP Resources / Prompts、GUI 控制或业务连接器已经存在。
- 不把 filesystem sandbox 标签等同于网络、进程、凭据和恶意插件的完整隔离。

## 3. 对象、owner 与 capability 边界

| 对象 | owner / 真源 | 不变量 |
|---|---|---|
| `ConnectorDefinition` | connector registry | provider/type/schema/capabilities/risk/version；不含账号 secret |
| `ConnectionAccount` | connector domain | provider account metadata、status、labels、credential_ref；不存 secret |
| `CredentialRef` | protected vault | opaque id、kind、status、rotation metadata；普通 store 不可解引用 |
| `CapabilityGrant` | Policy / Grant domain | subject、scope、capability、constraints、expiry/revocation、confirmation；Connector 只持 grant ref / projection |
| `SyncCursor` | connector sync repository | provider/account/dataset/version/cursor/watermark；幂等 |
| `InboundItem` | connector domain | source ref、external id/version、summary/ref/hash、sensitivity、dedupe |
| `ActionRequest/Result` | Action domain | user confirmation、effect id、payload ref/hash、external receipt、result command；Outbox 只拥有 delivery / effect 状态，不拥有 action 业务结论 |
| adapter | infrastructure | 只执行已授权 capability，不拥有 grant / domain transition |
| `AgentRuntimeAdapter` | M5 runtime port | 只执行 Workcell；不拥有 ConnectorDefinition、CredentialRef、外部事实或 ActionResult |
| `CapabilityRequest/ActionIntent` | Policy / Connector gateway input | 来自 runtime / tool 的请求；必须重新判 Grant、effect id、预算和 provider data contract |

每个 capability 合同至少固定：subject / role / scope、input / output schema、risk、confirmation、secret / egress、audit、idempotency、retry、rate / budget、failure / compensation、retention。

## 4. 任务切片

### SYN-CON-000 — Design-only 前置

在当前用户明确指令或 matching active package 下，只允许写 connector / provider-data / secret-boundary 设计与 mock fixture 规格；禁止 production schema、repository、adapter、网络、CredentialRef backend、App 和真实数据。它不改变 M8 状态，也不预批准 CON-001 以后实现。

### SYN-CON-001 — Adapter 与 Provider Data Contract 基线

冻结共享 capability envelope，以及 AgentRuntime、Agent / Model、Tool、Harness、KnowledgeSource、Connector 各自 interface / owner；再冻结 provider data contract 模板、secret boundary、error / receipt、revoke / delete / retention、mock fixtures。至少设计两种内部标识和调用形状不同的伪服务提供方，证明 Syn 的角色和会话身份不依赖某一家线程编号。只写合同。

### SYN-CON-002 — 内部 Adapter 抽取

把 Codex（代码智能体）、Agent Runtime、外部知识来源、主管能力协议和开发护栏包装为各自端口，共享最小 capability envelope；外部来源可使用模型上下文协议（MCP）或其他受控协议。DSH 只能包装为 `AgentRuntimeAdapter`，其 Tool / Plugin 输出重新进入 Syn CapabilityGateway。用两种伪适配器通过同一角色、会话、权限、结果和错误合同；不改变现有权限，不引入凭据引用，不把只读索引变成执行面，也不复制 M7 的知识来源登记与上下文装配职责。

### SYN-CON-003 — CredentialRef 与 vault port

只建立 opaque reference、resolve boundary、status / rotate / revoke contract、fake vault；静态 schema / deny-list 只能证明字段边界。必须用 sentinel 覆盖 fake-vault resolve、provider success / error / timeout / retry，并检查 event / audit / log / DTO / diagnostics / UI snapshot 落盘；真实 backend 获批后再做隔离 App sentinel。真实 backend 保持 HOLD。

### SYN-CON-004 — Connector registry 与 mock provider

实现 ConnectorDefinition、ConnectionAccount metadata、CapabilityGrant refs、SyncCursor、InboundItem schema / repository；从一开始只使用 CON-003 opaque CredentialRef / fake vault，不自造临时 credential 字段。用 mock provider 覆盖授权、同步、分页、重复项、断开、撤权、错误、重试。

### SYN-CON-005 — 一个低风险真实只读 connector

先单独冻结选定 provider data contract，再申请网络、账号、CredentialRef backend 和真实数据授权。范围仅 view 或受限 sync；每次只一个 provider / account / dataset。

### SYN-CON-006 — 设置与管理面

显示 provider、account label、capability grants、last sync、cursor / watermark、error、disconnect / revoke；secret 只显示存在 / 状态，不可显示、复制或进入普通诊断。

### SYN-CON-007 — 写型 action 独立候选包

本阶段只冻结，不默认激活。runtime / tool 只能提交 `ActionIntent`；ConnectorGateway 要求 explicit confirmation、effect id、outbox claim、external result command、partial failure / compensation、source readback 与 per-action grant；绝不复用只读 grant。checkpoint 后结果缺失进入 `OUTCOME_UNKNOWN`，不得由 runtime 盲重试。

### SYN-CON-008 — 隔离 / 真实 App 分层验收

先 mock connector + fake vault 做 App 管理和故障；真实只读 connector 只验获批 provider / account / dataset 的授权、sync、断开、撤权和错误。

## 5. 顺序、并行与写域

```text
CON-000（design-only）→ CON-001
CON-001 → CON-002
   └────→ CON-003 → CON-004 → CON-005 → CON-006 → CON-008
                              └────────→ CON-007（仅独立候选）
```

- connector framework 与 concrete provider 分 owner；真实 provider 一次只激活一个；
- CredentialRef port / backend 分包，backend owner 不与普通 connector repository 共享 secret 写面；
- M2 owns outbox / event / receipt，M8 只消费；M7 owns memory policy，InboundItem 不直接成为 FormalMemory；
- M4 只消费经过 provider-data contract、policy 和 sensitivity scrub 的 event summary + opaque source ref，不直接读取 InboundItem repository / 正文；M8 不直接写 Attention owner；
- 内部适配器抽取不得扩大主管能力协议、开发护栏或外部知识来源的现有能力；
- command registry、AppState、settings shell、SQLite schema、secret boundary 都必须唯一 writer 与 opening hash。

## 6. 迁移、撤权与回滚

- 内部 adapter 先 wrapper / shadow，不改旧行为；旧 command 到 M9 才 unregister；
- mock provider 的数据不可混入真实 store；profile / account / credential namespace 隔离；
- SyncCursor 只在 item + event / audit commit 后推进；receipt 丢失重试按 external id / version 幂等；
- revoke 先阻断 adapter，再标 grant / account，清理缓存按 provider data contract；不得因撤权删除原 owner 事实；
- CredentialRef rotation / revoke 不复制旧 secret；backup / restore 必须保持 reference integrity；
- rollback 可禁用 connector / 切回旧 internal adapter，不恢复 revoked grant；
- 真实数据删除、provider-side action 或 secret backend 更换分别另批。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / static | capability、secret、provider data、revoke 边界一致 | adapter 已接入 |
| Unit / property | 未授权在 adapter 前拒绝、cursor 幂等、secret exclusion | 网络 / provider 可用 |
| Mock integration | 两种不同形状的伪适配器、auth/sync/disconnect/revoke/error/retry、fake vault | 真实 connector 通过 |
| Non-test build | production path 可构建 | 桌面 / network 行为正确 |
| Isolated Tauri | 管理面、错误、撤权、secret 不可见 | 真实账号通过 |
| 经授权真实 read connector | 指定 provider/account/dataset 的真实证据 | write action、其他 provider 或发布通过 |

机械验收：unsupported / ungranted capability 在 adapter 前拒绝；runtime / plugin 不能直接解引用 CredentialRef 或执行 connector action；静态 secret scan 与 sentinel 动态 success/error/timeout/retry/diagnostics/UI 检查分别通过；session / trace / memory /普通日志不出现 secret；cursor / dedupe 重试不重复 InboundItem；断开 / 撤权后零新调用；错误在 App 可见且不泄露 provider response 原文。静态 scan 或 filesystem sandbox 单独不构成运行时 secret-exclusion / isolation 结论。

## 8. 独立授权与停止条件

真实 vault backend、每个 provider、每个真实 account / dataset、网络、付费、真实数据、真实 sync、每种 external action、provider-side delete 分别建包。M8 不授权无人值守 action、生产发布、Git 或旧路删除。

立即停止：provider data contract 未冻结；read grant 被复用为 action；secret 进入普通 store / DTO / log；adapter 绕过 policy；cursor 可能丢 / 重复数据且无恢复；撤权不生效；mock 被表述成 real；真实网络 / 账号缺独立授权；WIP 写面冲突。

## 9. 阶段退出与 M9 输入

全部满足才完成 M8：

- internal adapters 在统一 capability contract 下工作且未扩权，两种不同形状的伪适配器证明角色 / 会话身份不绑定 Codex 或某一家线程编号；
- Connector / Grant / Cursor / Inbound / CredentialRef 合同和 repository 冻结；
- mock connector 全生命周期、secret exclusion、failure / recovery 通过；
- 一个低风险真实只读 connector 只有在获批后才记录真实 App 证据；若未获批则 M8 整体保持 `PARTIAL / HOLD`，不得标 `COMPLETE`。framework 可事实收口，但 master 的真实 Connector / App 退出门仍未通过；
- write action 仍关闭或有独立未激活包；
- 旧 adapter / command 有 manifest / rollback，不物理删除；
- 向 M9 交已通过的 framework / read-model contracts 与 retirement candidates；真实 connector 处于 HOLD 时，M9 不得假设 runtime 成立或退役其替代链；
- `../current-state.md` 回写实际完成、暂缓和下一入口；M9 未激活不得续跑。
