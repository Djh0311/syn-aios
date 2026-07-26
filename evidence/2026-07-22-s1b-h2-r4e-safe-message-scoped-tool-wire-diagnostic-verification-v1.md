# S1B-H2-R4E 安全 message-scoped 工具线诊断验证 v1

日期：2026-07-22（+0800）  
任务包：`tasks/2026-07-22-s1b-h2-r4e-safe-message-scoped-tool-wire-diagnostic-package-v1.md`  
前置：`evidence/2026-07-22-s1b-h2-r4d-absent-submit-proposal-attribution-diagnosis-v1.md`

状态：**代码与离线验证完成；未启动或构建真实 App，未运行 Codex CLI/MCP server，未读取或写入真实 store；R4F 现场归因尚未出包或获授权。**

## 结论

R4E 已在既有 resident session、`supervisor_orchestrator` 与 Batch 2 canonical 写路之间补齐一条安全、message-scoped 的工具线事实族：`tools/list`、`tools/call`、`submit_proposal` handler 到 audit 边界分别留下可关联的固定枚举/布尔事实。

每次追加都要求一个正在运行、已绑定的 resident turn：已有 canonical `message_id`、generation、thread 与 run 的短摘要必须同时可验证；否则安静不记，不跨回合归因。幂等键至少覆盖 `(message_id, stage, invocation)`，因此同一 resident turn 的技术重复不会增加事实、卡或 chain。

新增 canonical event 的精确字段白名单为：`event_id`、`event_type`、`target_ref`、`message_id`、`stage`、三个 capability 布尔/`not_observed`、`invocation`、`handler_status`、`audit_status`、`generation`、`run_digest`、`thread_digest`。其中 run/thread 只保留既有 short digest；没有正文、arguments、实际其他工具名、错误细节、stderr、argv、环境、token/auth、完整 identity 或私有路径。

诊断是严格 best-effort：它仍只经 `write_m5b_batch2_workflow_state` 写一次；失败被调用点吞掉，不改变既有 recorded/injected/reply、用户人话、工具结果、proposal/Pending/chain，也不触发 retry、rebase、降级或新 DB 写路。

## Gate 0 与范围

- 开工 HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 集为空；porcelain 为既有 44 项。
- Gate 0 相关源码 SHA-256：`supervisor_resident_oneshot_session.rs=552531eab5ae…e3f509e8`、`supervisor_resident_oneshot_tests.rs=6c8ba3dbc0c3…7ee0da14`、`mcp/supervisor_orchestrator.rs=87f648a71040…6dff5241`、`mcp/supervisor_orchestrator_submit_proposal.rs=6130ee77e3b6…d351962b`、`mcp/supervisor_orchestrator_s1_tests.rs=f58799ee4b24…22845742`。
- R4E 相关初始脏项仅为已归属的 resident session 与其测试文件；其余 R4E 接点起始 clean。没有 reset、clean、stash、覆盖并行改动，未触发 `BLOCKED_DIRTY_OVERLAP`。
- 仅触及 resident diagnostic 接点、MCP tools/list/call/submit 的观测调用点及离线测试。没有新 Tauri command、sidecar、MCP server、transport、read-model 或 DB 写路；未改 H2 单工具预批准、allowlist、read-only、approval、sandbox、watchdog、invalid-resume 单次轮转、进程清理或 M5 逻辑。

## 先红后绿与离线验收

| 检查 | 结果 |
| --- | --- |
| 先红 | 新 R4E 测试先引用尚不存在的 audit-failure 注入 seam，编译如预期失败；随后最小实现前的 fixture 因既有 Batch 2 schema 校验拒绝而不产生事实。只把该离线 fixture 补为既有合法 workflow-state 形状后继续。 |
| `cargo test --lib s1b_h2_r4e -- --nocapture` | **5 passed, 0 failed**：绑定 tools/list、submit/other tools/call、handler accepted/denied、audit 写失败、重复投递、Batch 2 写失败保持完成对话与稳定人话。 |
| `cargo test --lib s1b_h2 -- --nocapture` | **21 passed, 0 failed**。隔离环境对受控 fake child 的 `ps lstart` 可见性有限，按既有离线测试权限复跑；没有启动真实 App、Codex CLI 或 MCP server。 |
| `cargo test --lib m5b -- --nocapture` | **10 passed, 0 failed**。 |
| `cargo test --lib m5f1 -- --nocapture` | **3 passed, 0 failed**。 |
| `cargo check --lib` | **通过**；595 个既有 unused/dead-code warnings，无编译错误。 |
| 脱敏检查 | 动态测试对 event 作精确字段白名单和受控 argument/identity 标记排除；event literal 静态扫描确认禁止 key 为零、绑定字段存在。 |
| `git diff --check` | **通过**。 |

新测试还明确证明：重复 submit 不增加诊断、Pending 卡仍恰一张且 chain 不变；audit 写失败仍能区分 handler 已 accepted/denied，不能伪装成 handler 未到；Batch 2 诊断写失败不留 partial fact、不重跑 resident turn。

## 变更、shape 与隐私边界

- R4E 实现位于 `supervisor_resident_oneshot_session.rs`、`mcp/supervisor_orchestrator.rs`、`mcp/supervisor_orchestrator_submit_proposal.rs` 及其离线测试；两份 resident 文件含先前已归属的 R3B 脏 hunk，本包只作最小语义合并。
- 历史 UI/harness shape 债为 `13 error / 5 warning / 5 info`；R4E 没有新增 UI shape gate 或把历史非零说成全绿。本包净增仅为一个安全 canonical 事实族和五项离线验证。
- `docs/harness-catch-log.md` 本次**零 catch**：预期红灯、失败注入、既有 warnings 和历史 fmt 债均不是新拦截，不追加账本。
- 所有验证使用临时离线 fixtures；没有复制、展示或入仓真实 store、runner 产物、私有 home 或认证资料。

## 未验证项与下一步

本包不证明真实 App 中的 tools/list 可见性、模型是否调用工具、handler 是否到达、Pending 卡是否新增，或同 thread 的现场结论。它没有发送 H2 任一句、刷新 UI、点击卡、批准卡、启动 chain/worker，亦未重建真实 binary。

下一步只能是**另行出包、用户在场、从全新 Gate 0 开始的 R4F**：一次新 client/message 的两句验收，仅在首句 canonical recorded→injected→自然回复完整后发送第二句；R4F 只读取同一 message 的本脱敏事实来裁决 A–D，不得依赖 R4D 私有 trace 猜测。若任一 Gate、绑定或一张 Pending 卡约束不满足，立即止损；不得重发、补卡、批准卡或启动 chain。
