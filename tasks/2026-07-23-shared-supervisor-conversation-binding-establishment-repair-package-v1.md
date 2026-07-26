# 共享主管 Conversation Binding 建立链离线诊断与最小修复 v1

日期：2026-07-23  
状态：历史前序离线修复记录；其四阶段/失败收口结论已由 [阶段语义与失败收口返工 v1](2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-package-v1.md) 补正。真实 App 验收未授权、未执行。

## 后续纠正（当前口径）

本包原先把 `binding_construct → binding_persist_db → binding_project_json → transport_start` 写成完整的建立链阶段。该说法过度：它遗漏了 store 准备失败，曾把激活失败误标成 DB 持久化失败，也没有区分“终结写入失败，因而无法确认 binding 已终结”。

当前源码与验收口径以 [阶段语义与失败收口返工 v1](2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-package-v1.md) 及其 [验证证据](../evidence/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md) 为准。本文以下四阶段和私有副本回放只保留为当时的部分证据，不能再作为全链路失败分类或“终结已确认”的依据。

## 目标与结论

本包当时只诊断并补齐共享主管 Conversation Binding 的部分建立链可观测性：

`binding_construct → binding_persist_db → binding_project_json → transport_start`

已证明一份经允许的真实 store 私有临时副本在 `25` 个会话、`0` 个 binding 的形状下，可以完成 DB-primary 与 JSON projection 的 `25 → 26`、`0 → 1` Starting binding 写入。该观察只证明 Starting 写入，不证明激活、transport 失败收口或终结已确认。因没有运行真实 App、没有重放真实用户输入，也没有拿到当日失败调用的子错误事实，**不得把这项离线成功说成已修复真实 App 的根因**。

## 事实与未知项

已知：

- 07-23 的真实 App 替代性验收首句只留下 canonical recorded；JSON/SQLite 中没有 conversation binding，后两句按停止合同未发。
- 副本审计前 DB/JSON 的 supervisor session 计数均为 `25`，绑定数均为 `0`；SQLite integrity 检查通过。
- 源码原有建立顺序是 context/turn/binding 构造、建立 binding、可选 host 观察到的 thread 激活、再启动 transport；原先多个失败面会折叠为通用错误，前端 catch 也会丢失错误 family。

未知：

- 当日真实 App 调用究竟停在 context、binding 构造、DB-primary、JSON projection 还是 transport 前置条件；本包没有其 message-scoped 错误事实。
- 真实 App 三句新验收是否能通过；本包不授权也不执行它。

## 最小修复

- 历史修复曾将 binding 建立错误分为 `binding_construct`、`binding_persist_db`、`binding_project_json`，另将 transport 启动失败作为 `transport_start`；这不是当前完整阶段表，不能忽略后续补入的 `binding_store_prepare`、`binding_activate` 与 `binding_terminate`。
- DB-primary 写路把 store 准备、DB delta、JSON projection 分别分类；测试注入只存在于 `cfg(test)`。
- 历史 receipt 只覆盖当时四阶段；当前 receipt 的终结未确认语义和工具闭锁以返工包为准。两版都不得伪造 conversation/thread/attempt/reply/tool/proposal/chain/worker，也不得返回路径、argv、环境、stderr 或原始用户文本。
- 前端只保存固定阶段枚举；运行时未知阶段或未知 thrown error 均降为通用安全文案，不把原始错误串带到 UI 或状态。
- 新增 25-session DB-primary 夹具、DB 写失败、JSON projection 失败以及真实私有副本回放测试。副本回放测试默认忽略，只有显式传入已获准的私有临时副本才运行。

## 验收证据

详见 [历史离线验证证据](../evidence/2026-07-23-shared-supervisor-conversation-binding-establishment-offline-verification-v1.md)。当前失败收口验收应看返工证据；本包通过项只包括：

- 真实私有副本：启动对账后 `25/0 → 26/1`，新 binding 为 `Starting`，未发布工具、未启动 transport。
- DB-primary 夹具：25 会话新增 Starting binding；DB 失败保持 `25/0`；JSON projection 失败保留受控 DB-leading `26/1` 且不发布工具。
- 主管阶段安全 receipt、binding 合同测试、M5-B DB-primary 回归、前端 typecheck、15 组离线交互测试和 `cargo check --lib`。

## 明确不声称

- 不声称真实 App 根因已定位或已修复。
- 不声称真实用户消息、主管回复、proposal、Pending、chain 或 worker 已发生。
- 不声称全仓 shape/format 历史债务已清零；shape 仍是历史 `16 / 5 / 5`，全仓格式检查被白名单外既有差异阻断。

## 下一步（需新授权）

新开真实 App 验收包后，先冻结当前 build、进程和真实 store 状态；再按既有三句合同进行一次全新、无重试的验收。若第一句再次失败，只采集新增的固定 `binding_stage` 和已授权的安全计数；第二、第三句仍须遵守停止合同，不能由本包的离线绿灯自动放行。
