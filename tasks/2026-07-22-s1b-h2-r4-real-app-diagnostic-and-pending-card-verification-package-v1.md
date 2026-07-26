# 任务包：S1B-H2-R4 真实 App 可归因对话与单张 Pending 卡验收 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；等待用户在场发送唯一 kickoff**
- 类型：真实 App 高风险验收；不含代码修复
- 权威 kickoff：`handoffs/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-kickoff-v1.md`
- 前置离线证据：`evidence/2026-07-22-s1b-h2-r3b-safe-internal-diagnostic-verification-v1.md`
- 前次现场证据：`evidence/2026-07-22-s1b-h2-r2-real-app-pending-card-verification-v1.md`
- 原始产品合同：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`

## 0. 唯一目标

在用户在场、现场重新关闭并冻结当前源码/binary 后，只发送一次新鲜首句。若主管链失败，读取同一 `message_id` 的 R3B 安全 diagnostic 并立即止损；若主管自然回复成功，才发送一次明确出方案句，要求唯一 MCP handler 生成恰好一张 `PendingUserConfirmation` 卡，然后停止。

本包不允许通过重复发送探测状态。成功出口是“一次首句 + 一次出方案 + 一张 Pending 卡”；失败出口是“一次失败消息 + 一条同 identity 的稳定 diagnostic + 零后续重发”。两种出口都必须可核对。

## 1. 产品合同

1. 对话框只负责像普通聊天一样与主管自然交流。
2. 用户明确说“出方案”后，主管才可通过 MCP 把结构化方案投影为右侧方案卡。
3. Syn 负责 canonical、会话事实、MCP 权限、幂等、审计和用户真话在用户与 Codex/其他 agent 之间自然流转。
4. 产品仅允许预批准 `supervisor_orchestrator.submit_proposal`；不得提供第二工具、wildcard、默认工具集或全自动执行权。
5. Pending 卡只是等待用户确认，不是执行批准。看到目标卡后必须停止，不点卡、不起链。

## 2. 已知事实

1. R2 真实现场三次用户动作分别形成三条 canonical record，但均无 injected/reply；旧计数为 `recorded/injected/replied = 11/3/3`，proposal/Pending/chain 为 `74/17/40`。这些只是最后一次证据，不是 R4 当前基线。
2. R3 裁决为 D：旧证据无法在 prepared 前多个失败门中诚实归因。
3. R3B 已新增 message-scoped `supervisor_resident_delivery_diagnostic_recorded`：只含稳定 `stage/stable_error_family` 与安全 identity 字段，走既有 Batch 2 canonical 写路，不进入用户 read model。
4. diagnostic 是 best effort；写失败不覆盖已完成的 user-message record，不重读、不重试、不 rebase，也不再跑 consult。
5. R3B 定向 5/0，S1B/H2 `32/0/1 ignored`，S1 submit `3/0`，M5-B/C/F1 `10/0、5/0、3/0`，cargo check、typecheck、离线交互与 rustfmt 通过；shape 仍是历史 `13/5/5`，零净增。
6. R3B 未启动真实 App、未操作真实 store；所以新 diagnostic 尚无真实生产路径证据。

## 3. 未知项

只能在新 kickoff 后重新确认：

- App/dev/Codex/MCP、registry 与 store holder 是否全部清零；
- HEAD、dirty ownership 与本包冻结源码是否漂移；
- 真实 store 当前 revision、DB/JSON 投影健康及各计数；
- 当前 proposal/Pending/chain、delivery diagnostic、generation/thread/session 状态；
- 固定测试项目的 git 状态与业务文件 hash；
- 当前源码重新 build 后裸 binary 的 hash、大小与 mtime；
- 首句会自然完成，还是产生哪个稳定 diagnostic；
- 第二句是否真正到达唯一 handler 并只落一张 Pending 卡。

不得把未知项预写成通过。

## 4. 出包时冻结源码

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：空
- 相关源码 SHA-256：

```text
552531eab5ae6f9beae7c857c6a438b8794dd52f9347e56052318232e3f509e8  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs
6c8ba3dbc0c38ad43a132651f216a35715120a1e82ab3ebc3208bdf97ee0da14  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs
d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs
6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs
7f382cadf799f9dc6e4a34e86b22aca666d9bb8983dee717c235d85c2e03252e  prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model_entrypoints.rs
47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2  prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts
279a728ee6487f7e8afecf5e81ad4df1dccb06b2b68179fafc2444a5bce3cb92  prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs
d15bbdb16dee75dc415d1e4e050b275eb89e75a940f1a93c281e7845693519f0  prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs
```

其中 resident session/test 与 knowledge-vault/M5-B 文件含已归属、未提交改动。执行者只冻结，不得清理或覆盖。任一 hash 漂移或归属不明即 `BLOCKED_DIRTY_OVERLAP`。

## 5. 唯一现场授权

只有用户在后续新消息中发送本包 kickoff，才可进入 Gate 0。核心授权文字为：

> S1B-H2-R4 开工；授权重新 Gate 0、构建并冻结当前裸 debug binary，启动真实 App，在固定测试项目中首句只发送一次；只有首句收到主管自然回复后，才把“按这个出方案”发送一次。失败只读同 message diagnostic 并止损；到一张 Pending 卡即停，不批准卡、不启动 chain、不改测试项目。

本任务包、R3B 完成回传或任何旧“可以做”都不能替代这条新现场授权。

## 6. 允许与禁止

### 允许

- 只读检查仓库、源码、evidence、真实 store、resident session、registry、进程/holder 与固定测试项目；
- 运行本包 Gate 1 的 debug build，产生正常 `dist/target` 构建产物；
- 启动本轮新冻结的裸 debug binary；
- 在固定测试项目中按 Gate 2/3 各发送一次精确文本；
- 允许产品正常写入 canonical user/injected/reply、内部 diagnostic、审计和恰好一张 Pending 方案卡；
- 正常关闭 App，保存脱敏 evidence，最小更新 `CURRENT.md` / catch log。

### 禁止

- 不修改任何 Rust、TypeScript、测试、配置、schema、依赖或脚本；
- 不 reset、clean、stash、stage、commit 或 push；
- 不执行 apply、reseed、迁移、恢复或直接写真实 store；
- 不自行 kill 进程；发现 holder/残留时请用户关闭；
- 不发送第三条消息，不重复首句或第二句，不复用旧 `client_request_id`；
- 不点击卡片，不批准方案，不启动 chain/worker，不改固定测试项目；
- 不接受工具批准弹窗，不添加第二工具，不扩大 approval/sandbox/path-lock/write root；
- 不把用户正文、原始错误、stderr、token、auth、私有路径或 `CODEX_HOME` 正文写入 evidence/read model；
- live 暴露代码问题时不得现场修码，必须止损并另出包。

## 7. Gate 0：现场、脏基线与真实 store 基线

1. 用户先关闭所有 Workbench/dev 实例。
2. 只读确认 Workbench、Tauri/dev/probe、Vite、Codex/MCP 残留为空；workflow-state、DB、WAL/SHM、JSON、registry 无 holder；registry entries=0。
3. 记录 HEAD、porcelain、staged 集并重算第 4 节八个 hash。
4. 只读确认 SQLite integrity、DB/JSON 核心投影与 revision；若普通只读打开失败，不得自动改用可写恢复。只允许有证据的 immutable read-only 核验，并明确口径。
5. 冻结以下变量：

   ```text
   R0 = user_message_recorded
   I0 = user_message_injected
   S0 = supervisor_message_recorded
   D0 = delivery_diagnostic_recorded
   B0 = proposal total
   P0 = PendingUserConfirmation total
   C0 = chain total
   ```

6. 冻结 resident generation/thread/session、DB-primary initialized/degraded、dispatch/binding/attempt/control 等辅助计数。
7. 冻结固定测试项目 HEAD、porcelain 与全文件/业务文件 manifest。
8. 若 R3B 后已出现未知的新消息/diagnostic/卡片，必须先解释归属；不能与 R4 增量混算。
9. Gate 0 任一项不绿即停止，不读取后再写、不自行 kill。

## 8. Gate 1：重建并冻结当前裸 binary

在 `prototypes/productized-desktop-shell` 运行：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

要求：

- exit 0；
- 第 4 节八个源码 hash 在 build 前后不变；
- 冻结 `src-tauri/target/debug/codex-governance-workbench` 的绝对路径、SHA-256、大小和 mtime；
- 只启动这个裸 executable，不使用历史 `.app` 或旧 binary；
- build 后重新确认相关残留/holder 不影响现场。

build 失败即停止；不修代码，不拿 R3B 旧 build 代替。

## 9. Gate 2：首句只发送一次

1. 启动 Gate 1 新冻结的裸 binary，进入固定测试项目交办页。
2. 在发送前记录当前页面、计数和 composer 状态。明确告诉在场用户本轮只允许一次提交，避免双击或重复发送。
3. 以一个新的用户动作和新的 `client_request_id` 发送精确文本一次：

   `我想给这个游戏里的标题改成小马里奥`

4. 记录本次新 `message_id` / client tail / timestamp；不得复用 R2 三条 identity。
5. 等待命令完成、canonical refresh 与主管自然回复；不得因界面暂时等待而再次点击。
6. 只有同时满足以下条件，才可进入 Gate 3：
   - recorded 恰好 `R0+1`；
   - 同 message injected 恰好一条，`I0+1`；
   - 同 message supervisor reply 恰好一条，`S0+1`；
   - 没有同 message delivery diagnostic，`D=D0`；
   - UI 展示自然主管回复，没有原始内部错误；
   - proposal/Pending/chain 仍为 `B0/P0/C0`；
   - session/thread/generation 与 lifecycle 可关联。

7. 正常路径应续接同一有效 thread。若触发现有 invalid-resume 自愈，只允许一次 rotate→generation+1→facts→initial；必须记录旧/新 thread、generation、归档与事实注入。递归轮转、信息断裂或多次 initial 都失败。

## 10. Gate 2 失败裁决

首句没有完整自然回复时，不发送第二句，立即按同一 `message_id` 对账：

| 事实 | 裁决 | 动作 |
|---|---|---|
| 无新 recorded | `message_not_recorded` / canonical 失败 | 不期待 diagnostic；关 App、止损 |
| recorded +1、injected/reply +0、同 message diagnostic +1 | R3B 生产留痕成功 | 只记录 `stage/stable_error_family/generation/thread`，关 App、另出定向修复包 |
| recorded +1、injected/reply +0、diagnostic 0，且 lifecycle 无成功 turn | `BLOCKED_LIVE_DIAGNOSTIC_MISSING` | 不猜根因、不查用户面原文补结论，关 App、另出包 |
| recorded +1、lifecycle 显示 turn completed，但 injected 0 | `BLOCKED_CANONICAL_INJECTION_WRITE` | 记录确切写边界，关 App、另出包 |
| injected +1 但 reply 0 | `BLOCKED_CANONICAL_REPLY_WRITE` | 不重发，关 App、另出包 |
| diagnostic 多于一条或 identity 不匹配 | 诊断幂等/关联失败 | 不删除、不补写，关 App、另出包 |
| quota/provider 等 stable family | 外部条件已被安全分类 | 保留现场事实，关 App；不得改代码伪装恢复 |

只读取安全 diagnostic 字段；本包不要求把私有 stderr 写入 evidence。diagnostic append 若失败可能留下 0 条，此时按缺失事实处理，不能把 absence 当成某一具体 family。

## 11. Gate 3：明确出方案只发送一次

仅 Gate 2 完整通过后发送一次：

`按这个出方案`

要求：

1. 新 recorded/injected/reply 各恰好再 `+1`，因此相对 Gate 0 总增量为 `R/I/S = +2/+2/+2`。
2. 第二回合无 delivery diagnostic，最终 `D=D0`。
3. 只调用 `supervisor_orchestrator.submit_proposal`，无需人工工具批准；若出现批准弹窗，不点击并停止。
4. 有真实 handler acceptance、proposal receipt 和主管自然 final；工具结果与 final 关联同一 active message/thread。
5. proposal 恰好 `B0+1`，Pending 恰好 `P0+1`，chain 始终 `C0`。
6. 方案卡内容必须对应“小马里奥标题修改”目标，不能落无关卡。
7. 第一/第二回合默认续同一 thread。若第二回合发生一次合法 invalid-resume 轮转，只能在事实重建完整、自然上下文未断、工具/结果/final 均绑定替代 thread 时记录为恢复路径；不得发生第二次轮转或双卡。该情况必须单列，不能冒充普通 same-thread。

若 outcome 为 `message_sent_proposal_tool_failed`：说明对话注入/回复可能成功但卡未生成。不得重发“出方案”；记录 tool outcome、handler 前后证据和零卡事实后停止，另出工具路径修复包。

## 12. Gate 4：一次刷新幂等

只有 Gate 3 成功落一张卡才执行：

1. 刷新或离开再进入交办页一次；
2. recorded/injected/reply、diagnostic、proposal/Pending/chain 均不得再增加；
3. 仍只显示一张本轮目标卡；
4. 不重发任何文本，不点击卡片。

## 13. Gate 5：正常关闭与最终对账

1. 正常关闭 App；不自行 kill。
2. 只读确认 registry、相关进程与所有 store holder 清零。
3. 对账最终变量：

   成功路径：

   ```text
   R = R0 + 2
   I = I0 + 2
   S = S0 + 2
   D = D0
   B = B0 + 1
   P = P0 + 1
   C = C0
   ```

   失败路径：严格按 Gate 10 或 Gate 3 实际增量记录，不补发、不修正真实 store。

4. DB/JSON 核心投影、revision、integrity、initialized/degraded 与辅助计数必须解释一致；不得把正常审计增长误报为 degradation。
5. 固定测试项目 HEAD、porcelain、manifest 必须与 Gate 0 一致。
6. 如正常关闭后出现孤儿/holder，报告并等待用户处理；不把业务成功与进程卫生混成一个结论。

## 14. 证据与状态回写

建议交付：

- `evidence/raw/2026-07-22-s1b-h2-r4-real-app/`
- `evidence/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-v1.md`

evidence 至少记录：Gate 0/1、binary hash、基线变量、新 message identity、lifecycle/thread/generation、diagnostic 安全字段或成功链、handler/receipt、卡片 delta、refresh、项目 hash 与进程/holder 对账。

不得记录用户正文副本、原始 stderr、token/auth、私有路径正文。精确验收短句可引用任务合同，不必从现场 dump 复制。

验收后最小更新：

- `CURRENT.md`：真实成功、具体失败或 blocker；
- `docs/harness-catch-log.md`：只有发现新拦截才 EOF 追加；
- 若失败，另出以 stable family/明确边界命名的下一任务包与 kickoff。

仍不得 stage 或 commit。

## 15. 成功标准

只有以下全部成立才能宣称 S1B-H2 真实 App 验收通过：

1. 当前 R3B 源码对应的裸 debug binary 已重建、冻结并实际启动；
2. 首句只发送一次并得到自然主管回复；
3. 第二句只发送一次并到达唯一 `submit_proposal` handler；
4. user/injected/reply 总增量严格 `+2/+2/+2`，无 delivery diagnostic；
5. 只新增一张目标匹配 Pending 卡，refresh 不重复；
6. chain/worker/固定测试项目均未变化；
7. 无工具批准弹窗、第二工具、原始错误泄露或虚假成功文案；
8. session/thread/generation、store/DB、进程/holder 均有证据且可解释；
9. 结果已写入新 evidence 与 `CURRENT.md`；
10. 未 stage、未 commit。

若首句失败但 diagnostic 正确落账，只能宣称“R3B 真实诊断留痕通过、H2 对话仍失败”，不能宣称 H2 通过。

## 16. 十项回传

1. Gate 0/1 及 binary 冻结结果；
2. HEAD、staged、dirty ownership 与八个源码 hash；
3. `R0/I0/S0/D0/B0/P0/C0` 和最终 delta；
4. 首句唯一 message/client identity、lifecycle、thread/generation；
5. 首句成功回复，或同 message diagnostic 的安全 stage/family；
6. 第二句是否发送，以及唯一 handler/tool outcome/final；
7. Pending 卡唯一性与 refresh 幂等；
8. chain/worker、DB/JSON、项目 manifest 对账；
9. App 关闭后的 registry/holder/process；
10. evidence/CURRENT/catch/下一包路径，stage/commit 状态与所有未执行项。

