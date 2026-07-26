# 任务包：S1B-H2-R4E message-scoped 安全工具线诊断 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；须由用户另行精确授权**
- 前置证据：`evidence/2026-07-22-s1b-h2-r4d-absent-submit-proposal-attribution-diagnosis-v1.md`
- 类型：最小离线可观测性补口；不是 H2 修复、不是现场复验

## 0. 唯一目标

为一个已绑定的 resident user message，补齐从**真实 `tools/list`**、`tools/call`、`submit_proposal` handler 到 audit/outcome 的无敏感、可幂等诊断事实。目标是让下一次单独授权的真实 App 回合能够把 A–D 分开；本包本身不得启动 App、发送消息或落卡。

R4D 已锁定第二句的 canonical/resident turn，但既有私有 trace 没有可归因的 JSON-RPC method/MCP namespace；`supervisor_tool_call=0` 因 audit 写失败仍不能等价于“未调用”。因此本包不能把 R4D 的 E 改写成模型、审批、handler 或 DB 的既定根因。

## 1. 唯一 kickoff

只有用户发送下列等价精确授权时才可开工：

> S1B-H2-R4E 开工；仅实现并离线验证 message-scoped 的安全 tools/list→tools/call→handler/outcome 诊断。不得启动或构建真实 App、不得读写真实 store、不得发送 H2 消息、不得改 H2 单工具审批或任何安全闸；代码和离线闸通过即停，真实 R4F 必须另包另授权。

R4D 的只读授权、任何旧 live/binary、或离线测试绿都不能替代这一授权。

## 2. 不可突破的边界

- 不新增 Tauri command、sidecar、MCP server、消息运输路、approval wildcard/default/full-auto/bypass；不改 `read-only`、sandbox、reviewer、path-lock、写根、watchdog、invalid-resume 单次轮转、进程组清理或 M5 DB-primary/CAS/fallback。
- 不改变 `supervisor_orchestrator.submit_proposal` 的 allowlist/预批准语义；不得批准卡、启动 chain/worker，或修改固定测试项目。
- 不启动 App、`cargo-tauri`、Codex CLI、MCP server/sidecar；不读取、复制、写入真实 Workbench JSON/DB/WAL/SHM、私有 home、runner output 或认证资料。
- 不把用户/主管正文、tool arguments、原始 error/stderr、argv、环境、token/auth、完整 private path 或完整 identity 写入 canonical/read model/evidence/test snapshot。
- 不 stage、commit、push、reset、clean 或 stash。

## 3. 唯一可新增的诊断事实

只经既有 Batch 2 canonical 写路追加一个既有 message 所属的安全事实族；建议事件名为 `supervisor_resident_tool_invocation_diagnostic_recorded`。每条事实必须能用现有 message/run/session 绑定，且只可包含：

| 字段 | 允许值 |
| --- | --- |
| `message_id` | 仅作既有 canonical join；read model/evidence 一律只显示 short digest |
| `stage` | `tools_list_served` / `tools_call_received` / `submit_handler_entered` / `submit_handler_finished` / `tool_audit_boundary` |
| capability | `submit_proposal_visible` boolean、`other_tool_visible` boolean、`only_submit_preapproved` boolean 或 `not_observed` |
| invocation | `submit_proposal` / `other_tool` / `none` 三值之一；绝不记录其他工具名或 arguments |
| handler/audit | 固定枚举 `entered` / `accepted` / `denied` / `audit_write_failed` / `not_observed`；不得带 detail |
| binding | 既有 generation 与 thread/run 的安全关联；read model/evidence 只显示 generation 与 short digest |

事实 idempotency key 必须至少覆盖 `(message_id, stage, invocation classification)`；同一 resident turn 的技术重试不能重复追加。任何诊断写失败都只保留已有业务行为和已有自然对话结果，不能触发重试、rebase、降级、落卡或用户面原错误。

## 4. 实施约束

1. 先冻结 HEAD、staged、porcelain、相关源码 hash；若相关脏项无法归属，`BLOCKED_DIRTY_OVERLAP`，不覆盖并行改动。
2. 只在现有 `supervisor_orchestrator` / resident-session / Batch 2 canonical 写路的最小接点落事实；禁止为方便新增任何 transport 或持久化面。
3. `tools/list` 只在存在有效 active message、已绑定 thread/generation 时记录；否则不跨回合归因。
4. `tools/call` 入口先将 name 分类为 `submit_proposal` / `other_tool`，再调用既有分派；不能序列化或摘要 arguments。
5. handler 事实仅由 handler 路径记录；audit 事实必须独立表达 audit 边界是否完成，不能把 audit 缺失伪造为 handler 未到。
6. 审计/诊断写失败的对外文案继续走既有人话边界，禁止泄露原始 detail；不得以新诊断写成功作为既有 proposal/Pending 成功的替代证据。

## 5. 离线验收（先红后绿）

至少覆盖：

1. 合法绑定 turn 的 `tools/list`：只记录 submit 可见/唯一预批准的固定事实，不记录配置正文。
2. `tools/call` 的 submit 与非 submit 分支：前者记录 submit，后者仅 `other_tool=true`，两者均不写 arguments 或实际其他工具名。
3. submit handler 到达、accepted/denied、audit-write failure：三层事实可区分；audit 失败不得把 handler 调用压成“未调用”。
4. 同 message/stage 的重复投递：诊断事实只一条，既有 proposal/Pending/chain 业务幂等不回归。
5. canonical diagnostic Batch 2 写失败：既有 `recorded/injected/reply` 与用户面稳定人话保持；无 retry/rebase/新 DB 写或降级副作用。
6. 脱敏扫描：新增 canonical/read-model/test evidence 不含正文、arguments、raw error、stderr、argv、环境、token/auth、完整 identity 或 private path。

跑最小定向 Rust 测试、相关 S1B/H2 与 M5 离线闸、`cargo check --lib`、`git diff --check`；如测试 helpers 触及 production 路径，必须以 non-test check 证明可编译。报告历史 shape 债与本包净增分开，不得把离线绿称为 R4F/live 绿。

## 6. 停止条件与下一步

代码和离线闸完成即停止。此包不允许运行真实 App 或再次发送两句；只有另一个、用户在场的 R4F 任务包，才能从新的 Gate 0 运行一次带该安全诊断的真实回合。R4F 的唯一职责是读取同 message 的脱敏事实来裁决 A–D，不得依靠 R4D 私有 trace 猜测。
