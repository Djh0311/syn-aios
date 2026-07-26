# S1B-H2-R4D 缺失 submit_proposal 调用归因诊断 v1

- 日期：2026-07-22（+0800）
- 任务包：`tasks/2026-07-22-s1b-h2-r4d-absent-submit-proposal-attribution-diagnosis-package-v1.md`
- 结论：**E / `NEEDS_SAFE_INTERNAL_TOOL_INVOCATION_DIAGNOSTIC`。** R4C 第二句的自然对话和 resident turn 可唯一关联，但现有留存不足以把最早缺口裁为 A、B、C 或 D。

## 结论边界

本诊断没有把“零 Pending 卡”归因为模型未调用、产品批准包缺失、MCP transport、handler 或数据库。已实证的是：第二句完成 canonical `recorded → injected → natural reply`，同一 resident turn 已 `prepared → exited`，而 durable `submit_proposal` audit/outcome/card 都没有增量。

同回合的私有 trace 有时间窗内的结构化记录，但没有可归因的 JSON-RPC `tools/list` / `tools/call` method、MCP namespace 或 message id。一个未命名 `tools` 数组和 generic call shape 不能安全充当能力列表或 MCP wire receipt。因此，不能把 `submit_proposal`“未见”压缩为“主管没调用”，也不能把 audit=0 压缩为“handler 没到”。

## Gate 0：现场、工作树与冻结

| 项目 | 脱敏结果 |
| --- | --- |
| scoped Workbench / dev / Vite / Codex-MCP | `0 / 0 / 0 / 0` |
| 通用 Tauri 宿主 | `4`，但关联父链=`0`、相关 open-FD process=`0`；不构成 Workbench 现场 holder |
| workflow-state / JSON / registry holder | `0` |
| DB / WAL / SHM holder | `0`；WAL/SHM 均不存在 |
| registry / lock | entries=`0`；lock=`0` |
| Git | HEAD=`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged=`0`；诊断写前 porcelain=`42` |
| R4 源码 | R4C 的 8 个 SHA-256 `8/8` 相同；新增冻结 `mcp/supervisor_orchestrator.rs`=`87f648a710404e0cc731651ea01954ff4c66927a1e6604c7edab1ef56dff5241` |
| 相关既有脏项 | `4` 个相关路径均与 R4C 冻结内容一致；无无法归属的相关漂移 |
| 固定测试项目 | HEAD=`caa02ded684d9e1d92d00c367949fab6f83430d1`；porcelain=`14`；历史/当前 16 个 non-`.git` 文件 hash+相对路径集合差=`0` |

真实 store 冻结（仅 hash）：

| 对象 | SHA-256 |
| --- | --- |
| workflow state | `6b6f1d3a3098a5ccb6826c230bcc89587caef07e719d62e9350cd0942ace5d02` |
| project proposals | `3d7d965e02fb12761d5f7e9d85218fd154050131edf77e92951f90540238f631` |
| supervisor orchestrator | `e63079fcaad521a823e33a2c4cc1bce9ecb2536f4e24e9f2407646a914f7140b` |
| process registry | `13e6b91e623c52784670e97c72903809bd83ebe22cd40f71a5801007db689d66` |
| production SQLite | `7eeba975b698365c0c87648aaa3a70d48ac6a98c16ac50d23a7987b938151b8f` |

普通 SQLite read-only 只返回受控失败码 `14`，未转为可写恢复；在 holder=0 且 WAL/SHM 不存在的前提下，immutable read-only `integrity_check=ok`。以下均为**当前 global count-level projection**，不冒充全语义 reconcile：

| projection | JSON | DB |
| --- | ---:| ---:|
| canonical `R/I/S/D` | `13/5/5/0` | `13/5/5/0` |
| proposal / Pending / chain | `74/17/40` | `74/17/40` |
| dispatch / binding / attempt / control | `404/76/164/164` | `404/76/164/164` |
| initialized / degraded-json-only | `39/11` | `39/11` |
| orchestrator sessions / audits / worker entries | `25/263/13` | `25/263/13` |
| `supervisor_tool_call` / `submit_proposal` audit | `14/0` | `14/0` |

## 第二句身份与 resident turn 锁定

| 关联面 | 结果 |
| --- | --- |
| R4C 首句 / 第二句 digest16 | `9c724595e7ca0d29` / `231eb41321f7c7f7`，各命中 canonical record 一次 |
| 第二句 canonical | `recorded/injected/replied/diagnostic = 1/1/1/0`；在 13 条 recorded 中 time-order=`13` |
| resident session | active-message match=`1`；run digest16=`18ad059764393529`；thread digest16=`c4cdd7e81ff8e498`；generation=`6` |
| lifecycle | 同 run/message 的 `prepared=1`、`exited=1`，顺序完整；最终 launch=`resident_exited` |
| proposal outcome | active outcome=`not_requested`；同 turn outcome audit=`0` |
| runner anchor | 既有 resume artifact 与该 `prepared → exited` 时间窗相符；未读取 stderr 或最后消息 |

canonical 与 resident lifecycle/run/session 是两类独立来源，故第二句自身不是 R2/R3 或首句的 identity ambiguity。

## 证据矩阵

| 边界 | 可安全观察到的事实 | 不能推出什么 |
| --- | --- | --- |
| canonical | 第二句 `1/1/1/0`，自然 reply outcome=`not_requested` | reply/user 正文，或工具调用与否 |
| resident turn | binding、generation、thread、`prepared → exited` 均可关联 | private runner 输出内容 |
| launch capability | 同时间窗私有 trace 有 17 个结构记录；其中未命名 `tools` 数组=`1`，entries=`2`，带 name entries=`0` | 它不是可证明的 `tools/list`，不能据此断言 submit 不可见或批准包缺失 |
| invocation wire | formal `tools/list`=`not_recoverable`；formal `tools/call`=`not_recoverable`；generic call shape=`3`，named call shape=`1`，`other_tool_observed`=`true`，MCP namespace metadata=`0` | generic call shape 不是可归因的 MCP call；不得输出其他工具名、arguments 或原始 transport 内容 |
| server / audit | durable turn-window `supervisor_tool_call=0`、submit audit=`0`、proposal-outcome audit=`0`；active outcome=`not_requested` | audit=0 不等于 handler 未到或无 wire call |
| materialization | JSON+DB proposal/Pending/chain 均无 R4C 第二句后增量；chain 未动 | count-level projection 不是 full semantic reconcile，也不能替代 handler receipt |

静态源码还说明为何不能把 audit 缺失当调用缺失：`call_tool_with_invoker` 先进入既有 handler，再以 `append_audit(...)?` 收口；audit write 失败会改变可见结果而不产生 durable tool audit（`mcp/supervisor_orchestrator.rs:773`、`:793`、`:2149`）。这只是**不相容替代解释**，不是现场根因推断。resident `prepared`/`exited` 与 active message 的 durable 绑定位置见 `mcp/supervisor_orchestrator_resident_session.rs:132`、`:347`。

## A–E 裁决

| 候选 | 结论 | 原因 |
| --- | --- | --- |
| A capability/approval envelope absent | 未证 | 没有可归因的 formal `tools/list` 或唯一预批准留存；未命名数组不能替代它。 |
| B supervisor did not invoke available tool | 未证 | formal tools/call 与 MCP namespace 未留存；不能把“未见 submit”写成“未调用”。 |
| C wire reached transport but no durable handler attribution | 未证 | generic call shape 不是可验证的 MCP wire receipt。 |
| D handler reached but rejected/persistence boundary failed | 未证 | 没有 durable handler receipt/outcome；audit 缺失仍有不相容解释。 |
| **E safe internal tool invocation diagnostic required** | **成立** | 第二句身份已唯一，但 capability/wire/handler 之间缺 message-scoped、脱敏、可持久关联的事实。 |

最早**可观察**缺口是 per-message tool capability/wire attribution 留存，而不是单一代码根因。两类独立证据为：(1) complete canonical/resident lifecycle 与零 durable handler/card 事实；(2) 同窗私有 trace 缺 formal JSON-RPC/MCP namespace，且静态 audit 边界允许 audit=0 的替代解释。

## 未执行与唯一后续

- 未启动或构建 App；未发送、重发、刷新、运行 Codex CLI/MCP server/sidecar；未读取 stderr、prompt、reply、arguments、auth/token 或 private path。
- 未写真实 JSON/DB/WAL/SHM、未改代码/配置/审批/安全闸、未点卡、未启动 chain/worker、未 stage/commit/reset/clean/stash。
- 本次发现一个新的留存拦截：R4C private trace 不能以安全字段把 capability/wire/handler 归因到 message。已按 catch 规则追加一条脱敏账。
- 唯一下一步：`tasks/2026-07-22-s1b-h2-r4e-safe-message-scoped-tool-wire-diagnostic-package-v1.md`。它只实现和离线验证安全的 message-scoped diagnostic；真实 App R4F 必须再另包、另授权。

## 收尾复核

- 结束时 scoped Workbench/Vite/Codex-MCP=`0/0/0`，workflow-state 与 DB/WAL/SHM holder=`0`，registry entries/lock=`0`；四个通用 Tauri 宿主仍无相关父链或 open FD。
- 9 个 R4/R4D 冻结源码 hash=`9/9`，5 个冻结 store hash=`5/5`；固定测试项目 HEAD/porcelain 仍为 `caa02ded684d9e1d92d00c367949fab6f83430d1` / `14`，16-file hash+relative-path set difference=`0`。
- `git diff --check` exit=`0`；staged=`0`；结束 porcelain=`44`，其中本轮新增未跟踪的 R4D evidence 与 R4E task=`2`。未运行代码测试：本包没有代码改动，且合同禁止 build/App/CLI。
