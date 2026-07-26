# 任务包：S1B-H2-R4D 缺失 submit_proposal 调用归因诊断 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；须由用户另行精确授权**
- 前置证据：`evidence/2026-07-22-s1b-h2-r4c-fresh-gate0-real-app-pending-card-verification-v1.md`
- 类型：关闭现场上的最小只读内部归因；不含代码、真实 store、App、对话或工具调用

## 0. 唯一目标

只定位 R4C 第二句在“本回合工具能力可见 → tools/call → server handler → audit/outcome → Pending materialization”中的**最早可观察缺口**。不能把“没有 Pending 卡”直接归因于模型、MCP、批准、handler 或数据库。

已知而且仅已知的事实是：两句都完成 canonical `recorded/injected/natural reply`，第二句与同一 resident thread/generation 可关联；但 `submit_proposal` 的持久化 audit/call/outcome、proposal/Pending 增量均为 0，chain 与项目未动。首句未调用工具不是异常。

## 1. 新授权必须明确包含的范围

用户的 kickoff 必须逐项授权：

1. App、Workbench/Tauri/dev/Vite/Codex/MCP、registry、workflow state 与实际 DB/WAL/SHM holder 全空；任一非空即 `BLOCKED_LIVE_HOLDER`，不 kill。
2. 仅对 R4C 第二句关联的既有私有 runner/transport 产物做**本机、流式、结构字段**只读检查；不得复制、导出、入仓、截图或显示其正文、prompt/reply、arguments、raw stderr/error、argv、环境、auth/token、`CODEX_HOME` 或绝对私有路径。
3. 不启动 App、不 build、不发送消息、不刷新 UI、不运行新的 CLI/MCP server/sidecar；不写真实 JSON/DB/WAL/SHM，也不创建离线副本。
4. 永久写面仅限 R4D 脱敏 evidence、`CURRENT.md` 最小语义合并，以及确有新 interceptor 时 catch log EOF；不得改 Rust/TS/test/config/schema、审批/沙箱/read-only、watchdog、invalid-resume、进程清理、项目、`.codex`，不得 stage/commit/reset/clean/stash。

## 2. Gate 0：新鲜冻结与身份锁定

1. 重核 holder/registry/lock；registry entries 必为 0。
2. 冻结 HEAD、staged、porcelain、已归属 dirty ownership；重新核 R4C 的 8 个 hash，并额外冻结 `src-tauri/src/mcp/supervisor_orchestrator.rs`。任何相关漂移或无法归属改动=`BLOCKED_DIRTY_OVERLAP`。
3. 冻结 R4C 终态 `R/I/S/D/B/P/C`、按 tool name/status 的 audit 计数、resident active proposal outcome、DB/JSON 安全 projection、worker/chain 与固定项目 manifest。
4. 以两种安全来源锁定第二句（canonical + resident lifecycle/run/thread/generation）；只记录 short digest/time-order，不能与首句或历史 R2/R4 消息混淆。无法唯一锁定=`BLOCKED_EVENT_IDENTITY_AMBIGUOUS`。

任何 DB/JSON projection 无法可信读取为 `BLOCKED_STORE_PROJECTION_NOT_GREEN`；不得以 R4B 的旧离线结论代替当前读数。

## 3. 证据矩阵（每项至少两类独立证据）

| 边界 | 允许的脱敏结果 | 不得推断 |
| --- | --- | --- |
| canonical | second `recorded/injected/reply/diagnostic` 计数与 reply outcome | reply 正文或用户正文 |
| resident turn | prepared/running/exited、active-message 匹配、thread/generation/run short digest | 私有 runner 输出内容 |
| launch capability | 本 run 是否可证明加载 `supervisor_orchestrator`、`submit_proposal` 是否在 tools/list、是否唯一预批准 | 只从源码推断为现场事实 |
| invocation wire | `tools_list_present/absent/not_recoverable`、`submit_proposal_tools_call_observed/absent/not_recoverable`、`other_tool_observed/none/not_recoverable` | 参数、approval 原文或任何 raw error |
| server boundary | handler arrival、tool name/status、active proposal outcome 的 `observed/absent/not_recoverable` | 仅凭 audit=0 断言没有 call |
| materialization | proposal/Pending/chain 的 JSON+DB 增量与 card read model boolean | 用 count-level 对账冒充 full semantic reconcile |

只有在该 run 的时间窗和关联完整时，`absent` 才是事实。若 audit-write failure、artifact retention 或身份关联缺失仍可解释证据，必须输出 `not_recoverable`，不能把它压成“未调用”。

## 4. 裁决与唯一下一步

必须给出两类相互独立的证据与不相容替代解释；只能选择一类：

| 裁决 | 严格条件 | 后续方向 |
| --- | --- | --- |
| A `TOOL_CAPABILITY_OR_APPROVAL_ENVELOPE_ABSENT` | 同 run 证明 tool 未暴露或不是唯一预批准，且 canonical/turn 完整、无 call/handler 交叉印证 | 离线配置/注册映射诊断包，不直接修复或放宽批准 |
| B `SUPERVISOR_DID_NOT_INVOKE_AVAILABLE_TOOL` | 同 run 证明 tool 可见且唯一预批准，完整 trace 证明无 call，handler/audit/outcome 也无 | 最小 prompt/tool-affordance 离线诊断包，不 live 重发 |
| C `TOOLS_CALL_REACHED_TRANSPORT_BUT_NOT_DURABLY_ATTRIBUTED_TO_HANDLER` | wire call 存在，handler/audit/outcome 缺失或不能绑 identity | message/run-scoped 安全内部 tool-invocation diagnostic 设计包 |
| D `HANDLER_REACHED_BUT_TOOL_REJECTED_OR_PERSISTENCE_RECEIPT_BOUNDARY_FAILED` | durable handler/audit/outcome 存在；再区分拒绝与 accepted/no-card | handler-failure 离线诊断或安全离线 store probe |
| E `NEEDS_SAFE_INTERNAL_TOOL_INVOCATION_DIAGNOSTIC` | 保留/身份/audit 缺口使最早边界不可证 | 明列缺失事实，另出唯一安全诊断包 |

所有 identity/store/holder blocker 保持 blocker，不可降格为 E。

## 5. 硬性禁止与结束条件

- 不启动或构建 App；不发送、重发或刷新；不点卡、不批准、不启动 chain/worker；不 kill。
- 不改任何审批、工具、sandbox/read-only、path-lock、watchdog、invalid-resume、M5 DB-primary/CAS/fallback 或消息运输路。
- 不读取/输出 private runner 的原文或复制其文件；只读失败不得转可写，也不得新建副本。
- evidence 只含计数、枚举、short digest、stable stage/family 与裁决；不能含完整 identity、用户/主管正文、record JSON、参数、原始错误、凭据或私有路径。

成功出口不是 H2 live 通过或修码，而是：完成第二句的一行证据矩阵、确定 A–E 的最早可观察边界、写脱敏 evidence/最小 CURRENT，并给出唯一可执行后续包。结束时运行 `git diff --check`；不 stage/commit。
