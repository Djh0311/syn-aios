# 任务包：S1B-H2-R4F 一次真实工具归因与 Pending 卡收口 v1

- 日期：2026-07-22
- 状态：已出包，未执行；须用户在场精确授权
- 类型：真实 App 单次高风险验收；不含代码、数据或配置修复
- 前置证据：`evidence/2026-07-22-s1b-h2-r4e-safe-message-scoped-tool-wire-diagnostic-verification-v1.md`
- 原始产品合同：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`
- 唯一 kickoff：`handoffs/2026-07-22-s1b-h2-r4f-one-pass-real-app-tool-attribution-and-pending-card-kickoff-v1.md`

## 0. 唯一目标与硬收敛

从全新 Gate 0 和当前源码的新 debug build 开始，只执行一次“两句自然对话 → 工具归因 → Pending 卡”现场流程。

- 成功：同一 resident thread 中两句各发送一次，第二句的 R4E message-scoped 事实证明 `submit_proposal` 可见、被调用、handler/audit 完成，并恰好新增一张目标 `PendingUserConfirmation` 卡；一次 refresh 不重复，chain/worker/固定测试项目不动。
- 失败：按本包矩阵直接裁决最早失败边界，并只生成**一个最小修复任务包及 kickoff**。不得再生成 R4A/R4D/R4E 式中间诊断包，不得现场修码或重发。

R4E 离线 green 不等于 live green。R4F 也不得为满足 A–D 而伪造结论；若 R4E 事实在有效 message binding 下仍完全缺失，将其作为明确的“诊断接线未在 live 生效”修复目标，而不是继续另开归因包。

## 1. 永久边界

允许：

- 只读 Gate 0/结束对账；构建并启动本轮新冻结的裸 debug binary；
- 两句各一次的真实产品交互；读取同 message 的 R4E 固定枚举/布尔诊断；
- 正常产品写入 canonical 对话、诊断、审计，以及成功路径中的至多一张 Pending 卡；
- 成功落卡后 refresh 一次；正常 Quit；按 kickoff 的窄授权清理本轮精确裸 binary 残留；
- 写 R4F evidence、最小 `CURRENT.md`，新拦截才向 catch log EOF 追加；失败时只出一个修复包和 kickoff。

禁止：

- 不改 Rust/TypeScript/test/config/schema/依赖，不直接写、恢复、reseed、迁移或 reconcile 真实 store；
- 不复用旧 baseline、binary、client/message identity，不重发、不发第三句；
- 不批准第二工具、wildcard/default/full-auto/bypass，不放宽 allowlist、read-only、sandbox、reviewer、path-lock、写根、watchdog、invalid-resume 或进程清理；
- 不点卡、不批准卡、不启动 chain/worker，不修改固定测试项目；
- 不读取 R4D 私有 trace，不输出正文、arguments、完整 identity、原始 error/stderr、argv、环境、token/auth 或私有路径；
- 不 stage、commit、push、reset、clean 或 stash。

## 2. Gate 0：一次新鲜冻结

1. 确认 Workbench/Tauri/dev/Vite/Codex/MCP、registry、lock、workflow state 与实际 DB/WAL/SHM holder 全空；registry entries 必为 0。任一非空即 `BLOCKED_LIVE_HOLDER`，不 build、不启动。
2. 冻结 HEAD、staged、porcelain 与 dirty ownership；冻结 R4E 实际触及的五个源码及 H2 UI 状态文件：
   - `src-tauri/src/supervisor_resident_oneshot_session.rs`
   - `src-tauri/src/supervisor_resident_oneshot_tests.rs`
   - `src-tauri/src/mcp/supervisor_orchestrator.rs`
   - `src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs`
   - `src-tauri/src/mcp/supervisor_orchestrator_s1_tests.rs`
   - `src/views/projects/jiaoban/useJiaobanConversationState.ts`
   任何无法归属的相关漂移为 `BLOCKED_DIRTY_OVERLAP`。
3. 普通只读核 SQLite integrity、storage mode 与 DB/JSON 安全投影；冻结真实 sidecar/DB/WAL/SHM hash、resident generation/thread/session、固定测试项目 HEAD/porcelain/full manifest。
4. 冻结基线 `R0/I0/S0/D0/B0/P0/C0`，以及 R4E 事件总数和各 stage/invocation/status 计数。历史 R4C/R4E 事实不得计入本轮。

## 3. Gate 1：当前源码新 binary

在 `prototypes/productized-desktop-shell` 执行既有 debug build：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

要求 exit 0；Gate 0 源码 hash build 前后相同；冻结 `src-tauri/target/debug/codex-governance-workbench` 的 SHA-256、size、mtime。只启动这一枚 binary。build 后若出现 holder/残留，停止。

## 4. 唯一真实回合

### 4.1 首句

启动健康且 composer 可用后，只发送一次：

`我想给这个游戏里的标题改成小马里奥`

只有同一新 identity 满足 `recorded=1 / injected=1 / natural reply=1 / diagnostic=0`，并关联本轮 resident generation/thread/run，proposal/Pending/chain 仍为基线，才进入第二句。否则立即止损，不重发。

### 4.2 第二句

只发送一次：

`按这个出方案`

等待该 message 的自然 reply、R4E 诊断和业务结果稳定；不得再次点击。只允许既有唯一预批准工具 `supervisor_orchestrator.submit_proposal`。出现批准弹窗、第二工具或安全闸扩张立即停止，不点击。

## 5. 当场裁决：不再拆中间诊断包

只读取第二句关联的 `supervisor_resident_tool_invocation_diagnostic_recorded`，按时间顺序核以下 stage：

`tools_list_served → tools_call_received → submit_handler_entered → submit_handler_finished → tool_audit_boundary`

| 结果 | 必要事实 | 裁决/唯一修复方向 |
| --- | --- | --- |
| PASS | submit visible=true、only submit preapproved=true、submit call、handler accepted、audit accepted，且 proposal/Pending `+1/+1`、chain `+0` | H2 live 通过，不出修复包 |
| A | `tools_list_served` 证明 submit 不可见，或唯一预批准不成立 | 工具能力/批准包映射修复 |
| B | submit 可见且唯一预批准；第二句自然 reply 完成；无 submit call，或仅 `other_tool` | prompt/tool affordance 或模型工具选择修复 |
| C | `tools_call_received=submit_proposal`，但无可关联 `submit_handler_entered` | MCP transport/dispatch 绑定修复 |
| D1 | handler entered，finished=`denied` | handler 拒绝条件修复 |
| D2 | handler accepted，audit=`audit_write_failed` | audit 持久化边界修复 |
| D3 | handler/audit accepted，但 proposal 或 Pending 未严格 `+1` | proposal/Pending materialization 修复 |
| LIVE-DIAG | 第二句 binding 完整，但五个 R4E stage 全部缺失或事实无法绑定 | R4E live 接线修复；这就是一个具体修复包，不再另开诊断包 |

不得仅凭“没卡”选择 B/D。每个裁决至少由 canonical/resident 与 R4E/业务投影两类证据交叉支持。

失败时只能输出一个最小修复任务包和 kickoff：写清唯一根因边界、最小文件面、先红后绿夹具、离线闸和最终一次 live 验收；不执行修复。

## 6. Pending、refresh 与关闭

仅 PASS 候选允许继续：必须严格为 `R/I/S/D = +2/+2/+2/+0`、proposal/Pending/chain=`+1/+1/+0`，且恰一张目标匹配的 Pending 卡。看到卡立即停止，不点卡。

refresh 一次；所有计数不得再次增加，目标卡仍恰一张。随后正常 Quit。

若正常 Quit 后只残留本轮已冻结裸 binary，且 PID、可执行文件、启动时间、cwd 均匹配本轮，registry/store holder 均为 0，按 kickoff 只可向该单一 PID 发送一次 `TERM`；禁止 `pkill`、进程组 kill、`KILL/-9` 或触碰其他进程。随后必须核到进程、holder、registry 全空。

## 7. 结束证据

最终回传只需八项：

1. Gate 0/1、binary digest；
2. dirty ownership 与源码/项目冻结；
3. 两句 identity short digest、canonical/resident delta；
4. 第二句五 stage 的固定枚举/布尔矩阵；
5. A/B/C/D/LIVE-DIAG 或 PASS 裁决及两类证据；
6. proposal/Pending/chain、refresh 与项目不变量；
7. 正常关闭及是否使用窄 TERM；
8. evidence/CURRENT/catch、staged/commit 和唯一后续。

成功后 H2 收口，下一阶段回到既定交办页 UI：把历史栏移到右侧方案卡左边，作为方案列表。失败后只进入本轮产出的一个最小修复包，修复后进行一次最终 live 验收。
