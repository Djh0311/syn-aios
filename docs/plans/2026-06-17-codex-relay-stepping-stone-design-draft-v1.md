# 设计草案：Codex 中转（甲）老实 relay 跳板 v1

日期：2026-06-17

性质：设计草案，供咨询线审 + 用户拍板。计划正本为 `docs/plans/2026-06-17-codex-relay-stepping-stone-plan-v1.md`；kickoff 为 `handoffs/2026-06-17-codex-relay-stepping-stone-design-claude-to-codex-kickoff-v1.md`。

## 拍板摘要

- **批准什么**：批准后续另开实现包，做一条“老实中转 / manual relay”窄入口：用户在 Syn 里手动写一句，明确选项目 / Codex 会话，确认后由工作台把这句原样交给 Codex，最后把回执和最后输出回到会话面。
- **代价是什么**：第一次让 Syn 真实启动 Codex；它的风险级别不是“安全模拟”，而是接近用户自己在终端直接用 Codex。必须接受 Codex 会向外部模型服务发送这句 prompt，并由 Codex CLI 正常使用 / 写入自己的本地状态。
- **不批准的后果**：会话引擎继续只记录发送意图，Syn 仍然“看得见、动不了”，不能作为走向蓝图角色编排的第一跳。
- **关键澄清**：这不是产品全局真实执行解锁，不是 K3-B1/K3-B2 恢复，不是 H2/H3/H5/PCR 执行门放宽，不是自动连环编排，也不是把工作台终局改成聊天客户端。

## 一句话判据

判断后续实现是否仍在本设计内：**它是否只把用户明示的一句话、一次性、发到用户明示的项目 / Codex 会话，并让用户看见实际 payload、回执、停止 / 回滚边界，同时不拆不弱化任何现有真实执行闸？** 是，则属于甲·中转；否则停下回咨询线。

## 本轮状态

本轮只产出设计文档；未实现、未运行真实 Codex、未新增 Tauri command、未调用 runner、未读写 `/Users/yoyi/.codex`、未碰 K3-B1 / K3-B2 / real-resume / product-command 闸。

## 已核源码事实

- 当前会话撰写区仍是 decision-only：`AgentChatComposer` 标记 `data-send-mode="decision-only"`，并写明“本按钮不真跑 Codex、不解锁 K3-B1 / K3-B2”（`prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx:24`、`:64`）。
- 当前发送处理只把用户文字加入 pending transcript，不调用 Tauri 或 runner（`prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx:219`-`:229`）。
- pending user message 明确记录 `conversation_engine_send_mode: "decision_only"` 与 `real_codex_executed: false`（`prototypes/productized-desktop-shell/src/lib/conversationEngine.ts:20`-`:25`）。
- 底层 `codex_local_runner` 的 guard 已有可复用安全构件：adapter 必须是 `codex-local`，operation 仅限 `new_session` / `send_message` / `resume`，必须有用户确认、授权范围、audit ref、路径 / prompt / readback 检查，命令计划必须是 program + argv + stdin prompt，不允许 shell 拼接（`prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs:150`-`:236`）。
- 真实 Codex 进程由 `Command::new(program)` 启动，prompt 走 stdin，stdout 丢弃，last message 写到 workbench-managed path，超时会 kill child，runner 声明不持久化 prompt body / raw stdout / stderr（`prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs:817`-`:980`）。
- 现有 H2 real resume 门明确只接受 `resume`，不处理 `send_message`；预检和 Phase B 都有这道检查（`prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs:399`-`:401`、`:938`-`:940`）。
- K3-B1 recovery 现有产品路径把“手动 exact command + 回交 + 不自动成功”作为安全恢复参照，并明确 L1 不执行 Codex、不存 prompt body / `.codex` 内容（`prototypes/productized-desktop-shell/src-tauri/src/k3_b1_recovery.rs`）。

## 设计结论

推荐做**新增并列 manual relay 窄入口**，不要把现有 H2/H3/K3/H5/PCR product-command 链改轻，也不要把会话发送按钮直接接到既有 Phase B。

原因：

- 现有 product-command / real-resume 链是“工作流 / 任务包 / 授权矩阵 / 审计”的重门，职责是乙·编排和高风险执行，不适合为了“一句话手动中转”而放宽。
- `codex_local_runner` 的结构化命令、安全检查、stdin prompt、last-message readback、超时 kill 等部件值得复用，但应由新的 `manual_relay` contract 调用，不能伪装成 H2 continuation 或 K3-B1 recovery。
- 会话引擎应复用发送框、目标会话选择、pending/receipt 回流、transcript timeline，但必须新增明确 relay mode / confirmation path，不能悄悄改变现有 `decision_only` 语义。

## 目标用户流

1. 用户在会话页选择项目根与目标 Codex 会话；若是新会话，必须显式选择“新会话”，不得从空目标隐式推断。
2. 用户输入一句话；UI 在发送前展示“将真实发送的 payload”和目标绑定。
3. 用户点击“手动 relay 发送一次”；该点击只授权这一条，不继承为后续自动发送。
4. 后端创建 relay attempt，启动一次 Codex CLI，prompt 通过 stdin 发送，last message 写到 workbench-managed path。
5. UI 把用户原话、运行中状态、最终回执和 Codex 最后一条输出回流到同一会话 timeline。
6. 运行结束后自动停止；不会根据 Codex 输出继续发下一条，也不会触发 K3-B2 / workflow automation。

## 回答 kickoff 六点

### 1. relay 路径如何复用会话引擎

复用三层，不复用现有 decision-only 行为本身：

- **输入层**：继续使用会话页的项目 / 会话选择和 composer 位置，但新增 `send_mode="manual_relay"` 的显式按钮或二段确认；原 `decision_only` 发送保留。
- **timeline 层**：复用 `appendPendingUserMessage` 的显示思路，但 pending message 元数据必须改为 relay 专属，例如 `conversation_engine_send_mode="manual_relay_confirmed_once"`、`real_codex_executed=true|false`、`relay_attempt_id`、`target_project_root`、`target_session_id`。
- **回流层**：runner 的 `--output-last-message <workbench-managed-last-message>` 作为最小输出源；Syn 不需要读完整 rollout / transcript 正文即可显示本次最后回复。

实现期不得把现有 `handleSubmitConversationDraft()` 直接改成 runner 调用；应新增窄 handler，例如 `handleSubmitManualRelayOnce()`，并让旧 handler 继续保持 decision-only。

### 2. 三本分怎么落

**本分一：原话转发可见。**

- v1 的 `effective_prompt` 必须逐字等于用户输入的 `original_user_text`，不得隐藏附加 system wrapper、任务包、记忆或角色指令。
- 发送前 UI 展示 exact payload；运行记录保存 prompt hash / length / visible preview，不把 prompt body 塞进 runtime log / audit log。
- 若未来加入角色 / 记忆 / 任务包注入，必须变成可见 `payload_layers[]`，逐层展示；届时不能再声称“只发原话”。

**本分二：转到指定项目 / 会话。**

- relay request 必须含 `target_project_root_canonical`、`target_cwd_canonical`、`target_session_id` 或显式 `new_session=true`。
- UI 发送前展示项目名、canonical path、会话标题 / session id、sandbox、允许写入根。
- 后端校验用户确认时看到的 target hash 与执行时 canonical target 完全一致；不一致则阻断。
- 对 existing session：只能向用户选中的 session 做 `codex exec resume <session_id>`；不能从标题搜索、最近会话或 project fallback 推断。
- 对 new session：必须在 UI 上独立选择“新会话”，并显示它会在该项目下创建新的 Codex 会话；不得把“没有选会话”解释成新会话。

**本分三：手动、一次一发。**

- 每次 relay attempt 只消费一个 `manual_relay_confirmation_id`。
- attempt 完成 / 失败 / 超时后，状态必须 terminal；后续输入必须重新点击确认。
- 禁止根据 Codex 输出自动触发下一次 relay；禁止后台队列、连环 worker、auto retry、K3-B2 解锁。
- duplicate guard：同一 target session 若已有 running relay attempt，下一条必须阻断或排队等待用户重新确认；默认推荐阻断而非排队。

### 3. “安全级 = 直接用 Codex”怎么论证

这个论证只在以下条件全部成立时有效：

- Syn 不隐藏加料：`effective_prompt == original_user_text`，或所有注入层都可见且需另行拍板。
- Syn 不读取 Codex auth / token / secret / full transcript / rollout body；回流只读本次 workbench-managed last message。
- Syn 不写 `/Users/yoyi/.codex`；`.codex` 的正常写入只来自 Codex CLI 自己运行，等价于用户直接用 Codex CLI。
- 外发只有 Codex CLI 调模型那一下；Syn 不额外把 prompt 发给第二个服务。
- target、sandbox、write roots 在发送前可见且后端逐项校验。

因此它不是“低风险模拟”，而是“把用户直接用 Codex 的动作搬到 Syn 里，并加上 target / payload / receipt 审计”。如果 prompt 要求读取 secret、auth、`.env`、keychain、OAuth、provider credential、完整 transcript 或 `.codex` 内容，v1 应阻断或至少转入单独高风险确认；默认 direct-use 级不覆盖这些内容读取。

### 4. 怎么在现有 guard 里轻接通、不拆不弱化闸

推荐未来实现的后端结构：

- 新增 `manual_relay.rs`，拥有自己的 `ManualRelayRequest` / `ManualRelayGuard` / `ManualRelayAttempt` / `ManualRelayReceipt`。
- 复用或镜像 `codex_local_runner` 的安全构件：结构化 command plan、stdin prompt、last-message path、timeout kill、duplicate guard、target canonicalization、secret deny list、readback status。
- 新增窄 Tauri commands（名称仅为建议）：`preview_manual_codex_relay`、`confirm_manual_codex_relay_once`、`run_manual_codex_relay_once`、`stop_manual_codex_relay_attempt`。
- `preview` 和 `confirm` 不调用 runner；`run` 只在 `confirmation_id` 与 prompt hash / target hash / sandbox / write roots 全部匹配时启动一次。

必须保持不变的旧门：

- 不改 `run_real_resume_phase_b_with_runner()` 的 authorization matrix，不让 H2 resume 接受 relay 的轻确认。
- 不改 K3-B1 recovery 的 `manual_recovery_needs_review` / K3-B2 gate。
- 不改 real_execution_product_command 的 PCR/H5 workflow product command 链，不把 relay attempt 伪装成 workflow run unit。
- 不把 `codex_local_runner::inspect_codex_local_execution_guard()` 改松；如果要支持 relay 的更轻字段，应新增 relay guard 或新增严格分支，而不是删除既有必填项。
- 不新增自动连环、后台 worker dispatch、多 agent 并行执行或通用真实执行授权。

“不走乙的重审批”的含义仅是：一句话中转不要求先生成主管任务包 / workflow node / memory packet / supervisor matrix；它仍要求用户在 UI 上确认本次 payload 与 target，且后端必须有本次 relay 专属 guard。

### 5. 留好以后插角色 / 任务包 / 记忆注入 / 审计的口

relay v1 的 request envelope 不应是一个裸字符串，而应预留分层结构：

```text
ManualRelayEnvelope
- relay_id
- target_binding
  - project_id
  - project_root_canonical
  - target_cwd_canonical
  - target_session_id | new_session
  - sandbox
  - allowed_write_roots
- payload
  - original_user_text
  - effective_prompt
  - payload_layers[]   # v1 为空；未来角色/任务包/记忆注入必须显式进这里
  - prompt_sha256
- policy
  - manual_once=true
  - auto_chain=false
  - duplicate_scope
  - denied_material_policy
- future_hooks
  - role_id?
  - task_package_ref?
  - memory_packet_ref?
  - supervisor_review_ref?
  - post_run_memory_capture_policy?
- audit_refs
- receipt_refs
```

v1 只允许 `payload_layers=[]` 或只含用户可见的边界说明；未来若插入角色 / 任务包 / 记忆包，需要新的用户拍板和复核，且 UI 必须展示“最终发给 Codex 的完整 payload”。

### 6. 回执 / 停 / 回滚

**回执必须包含：**

- `relay_attempt_id`、`confirmation_id`、target project / session / sandbox / write roots。
- `effective_prompt_sha256`、prompt length、是否 exact-original。
- command redacted preview，必须仍是 program + argv + stdin prompt，不出现 shell 字符串拼接。
- start / end timestamp、exit code、timed_out、killed_by_user、readback status、last-message hash / size。
- `prompt_sent`、`real_codex_executed`、`writes_codex_home_by_codex_cli`、`syn_read_codex_home=false`、`syn_wrote_codex_home=false`。
- changed files summary（若 sandbox 允许写）、git HEAD/status before/after（若 target 是 git repo）、warnings。

**停必须是本 attempt 范围内的 stop。**

- run command 必须维护 active child handle / pid / attempt id；`stop_manual_codex_relay_attempt` 只能 kill 对应 relay child。
- 不得 kill 其它 Codex 进程、其它 session、其它 workflow worker。
- stop 后 receipt 写 `killed_by_user=true`、exit / signal、last-message 是否可用；不自动 retry。
- 如果实现期无法提供可点击 stop，只靠 timeout，不应宣称“能停”；只能作为 degraded v0 回咨询线重定范围。

**回滚必须默认保守。**

- relay 运行前记录 target repo 的 HEAD、dirty status、tracked changed files hash；运行后记录 changed files summary。
- 若工作树原本干净，可生成本 attempt 的 rollback suggestion / reverse patch；执行真实回滚仍需要单独确认。
- 若工作树原本 dirty，默认只给“本次可能变更清单 + 手动恢复建议”，不得自动 `git reset` / `git checkout`，避免误删用户已有改动。
- 非 git 项目只能提供文件清单和备份 / diff 建议；不能声称完整可回滚。

## Future implementation acceptance gates

实现包至少需要钉住这些测试 / gate：

- UI：旧 decision-only 发送仍不真跑；manual relay 必须显示 payload + target + one-shot warning。
- 后端：prompt hash / target hash / confirmation id 不匹配即阻断。
- 后端：running duplicate attempt 被阻断；auto-chain 字段永远 false。
- 后端：secret / token / `.env` / keychain / OAuth / credential / full transcript / rollout / `.codex` 内容读取请求被阻断或进入单独高风险路径。
- runner：command plan 无 shell、prompt via stdin、last message path 在 workbench-managed run dir。
- stop：只 kill 当前 attempt；receipt 记录 stop 结果。
- rollback：dirty tree 不自动 destructive revert；clean tree 可生成 rollback suggestion 但真恢复另批。
- regression：K3-B1 / K3-B2 / H2 real resume / H3 new session / H5 product command tests 证明旧门未放宽。

## Deferred / not in this design approval

- 真实实现 manual relay。
- 运行任何 `codex exec` / `codex exec resume`。
- 写入或读取 `/Users/yoyi/.codex` 的正文 / auth / token / rollout / prompt body。
- 产品全局真实执行授权、K3-B1 retry、K3-B2、乙·工作流连环、多 agent 并行真实执行。
- 角色 / 任务包 / 记忆注入真正上线。
- 自动 stop / retry / rollback / memory formalization。

## 推荐拍板口径

若咨询线和用户接受本草案，下一步不是直接实现，而是另写“manual relay implementation”窄任务包：明确新增文件、命令、测试、真机验收、第一次真实 Codex relay 用户在场授权语句，以及“实现期仍不得放宽旧闸”的复核清单。
