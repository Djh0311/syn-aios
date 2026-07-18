# S1 主管自由对话与方案工具：实现/验证交接 v1

日期：2026-07-18  
任务包：`tasks/2026-07-18-s1-freeform-supervisor-and-proposal-tool-package-v1.md`  
上位决定：`decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md`  
P1-B 退役断言预登记：`evidence/2026-07-18-s1-p1-b-retired-test-preregistration-v1.md`

本文件区分已验证的离线实现、历史形状债和未做的真实模型/真机验收；不把三者混为“已绿”。

## 1. 交付设计与范围

主管中栏现在采用“自由聊天走会话，结构化方案只能走工具，批准仍只在右侧卡”的模型。普通用户消息不会创建 proposal、不会批准、不会推进 workflow chain；`submit_proposal` 成功只写既有 proposal store，状态为 `PendingUserConfirmation`。

P2-A 的方案 schema/lint/落卡函数本体没有改动。S1 新增的是 private MCP 工具边界的严格参数解析和对既有写入函数的调用，不增 sidecar。

## 2. 勘察补遗的发送路边界

S1 只泛化既有路② `submit_supervisor_resident_answer`，复用路④ resident session 的同 thread `codex-reply` 器官；没有新增第五个用户消息 sender。

- `manual_relay` 仍是智能体页任意会话、preview+confirm 的手动指挥通道。
- S1 是固定项目主管 thread 的 canonical 用户消息入口；两者边界不同，不在本包互相改造。
- `readonly_codex_consult`、`AgentConversationShell`、`TranscriptViews`、`AgentChatComposer` 均未触碰。

## 3. A1 与 P1-B 退役

主管回文不再要求 `supervisor_resident_turn.v1` 的二选一 JSON，也不再因自由文本产出 `protocol_invalid` 保守停。历史审计记录及读模型兼容映射保留。

六项退役断言、理由和 P1-A 替代测试已预登记在 P1-B 清单中；历史 `supervisor_resident_session_tests.rs` 保留为证据但不再注册。新注册入口为 `supervisor_resident_freeform_tests.rs`。

## 4. A2：`submit_proposal` 私有工具

- pilot MCP 工具面仍为既有三个只读工具；带 `supervisor-resident:` run namespace 的 resident 工具面才增加 `submit_proposal`。
- tools/call 还须通过 durable resident session 的 project/root/workflow/thread 绑定，且从 canonical 最新用户消息取得 requirement snapshot；模型不能覆盖这些 server-owned 字段。
- 参数逐字段严格拒绝未知字段和错型字段；毒任务图在 proposal store 前返回人话 lint 错误。
- 成功只产生一张 `PendingUserConfirmation` 卡和既有 `project_consultation_proposal_created` 审计，链保持空、未启动。

实现为 `mcp/supervisor_orchestrator_submit_proposal.rs`，将主 MCP 文件维持在 2997 行，未新增形状门债。

## 5. A3：canonical 对话与读模型

用户消息先持久化，再通过 private resident host 注入；公开 transport 不能绕过该持久化，也不冒充用户。新增 canonical 事件为：

- `supervisor_resident_user_message_recorded`
- `supervisor_resident_user_message_injected`
- `supervisor_resident_supervisor_message_recorded`

读模型词表同步扩展为 `user_message`、`user_message_injected`、`supervisor_message`，并显式保持为不推进 workflow 的黑板条目。P3-A 过程消息派生未动。

## 6. A4：中栏单通道与右卡隔离

测试项目的 composer 无论 phase 都只走 `submitSupervisorResidentAnswer`。普通消息请求在飞或报错只影响中栏的 thinking/alert，不禁用或污染右侧 proposal 卡的“允许并开始”；卡上的“按我说的改”复用同一 composer，不会直出 proposal。

保留的单条请求 in-flight 抑制仅防重复点击，不是 workflow running/done/blocked 的状态锁。

## 7. P1-A 与高危面保留

同 thread 续接、host 失活 rebuild、thread-invalid 当回合换代、项目槽隔离、私有 MCP 白名单与换代事实注入均由新的 S1 测试覆盖。`submit_proposal` 是白名单唯一新增项。S1 三支、写域锁定、终标/复核面、worker 面和 P3-A 过程面未改。

## 8. 已执行验证

| 命令 | 结果 |
| --- | --- |
| `npm run typecheck` | 通过 |
| `npm run test:offline-interaction` | 15 组通过；含 `jiaoban-conversation-center` |
| `cargo check --offline -q` | 通过（仓内既有 warnings 仍在） |
| `cargo test --offline -q s1_freeform_` | 8 passed，1 ignored |
| `cargo test --offline -q s1_resident_submit_proposal` | 3 passed |
| `cargo test --offline -q m5b_` | 9 passed |
| `cargo test --offline -q m5c_` | 5 passed |
| `cargo test --offline --lib` | 1000 passed，44 ignored |
| `rustfmt --edition 2021 --check --config skip_children=true`（新增 MCP 子模块及测试） | 通过 |
| `git diff --check` | 通过 |

## 9. 四闸 / M5 / 形状门口径

M5 的 DB-primary、投影、重放与 JSON-only 定向回归如上均通过。

`node scripts/harness/workbench-shape-gate.js --mode baseline --json` 为 pass，`13 errors / 5 warnings / 5 infos`；`--mode check` 仍 exit 1，且是相同的 13/5/5 历史债。此前本包把 `mcp/supervisor_orchestrator.rs` 推至 3454 行，额外制造了第 14 个 error；已拆分后主文件为 2997 行，额外 error 消失。故本包没有用调高 baseline 掩盖形状问题，但历史债未被宣告为绿。

## 10. 未验收项、已知遗留与下一步

1. **A5 真跑/真机未执行。**尚无真实 Codex 三轮自由聊天→工具落卡→批准→链跑 transcript，也没有两个真实模型反例或用户自然聊一单的三数复测。执行前需要用户授权真实模型调用，并核验目标 executable/固定项目环境。
2. **P1-E 遗留入口需单独处理。**工作流侧栏还有两个可见的旧 `run-project-consultation` 按钮；为避免它成为第二个公共写入/出卡入口，底层命令已显式退役并返回错误。它们不应直接改接为另一条用户消息 sender；后续应在独立 P1-E 清理中决定隐藏、跳转主交办中栏，或给出迁移说明。
3. **作用域例外已披露。**任务补遗要求 consult 零碰，但上位决定要求 `submit_supervisor_resident_answer` 是唯一用户消息入口。为消除旧直出方案 route，`run_project_consultation` 被改为 retirement tombstone；这不是“consult 面完全零改”的声明。
4. resident run-id namespace 是工具展示层的可信本地门；实际写入仍检查 durable binding。若未来该 MCP 服务暴露给不受信任 caller，应把“private home provenance”升级为不可伪造的 host capability，而非仅依赖 run-id 前缀。
5. 首轮 `codex` 返回前才拿到并持久化 thread binding；工具面已展示 `submit_proposal`，但首轮若抢跑调用会被 durable-binding 闸拒绝。A5 规定的三轮聊天路径不受影响；若要支持首轮即出卡，需单独设计不破 P1-A 的预绑定能力。
