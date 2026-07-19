# S1B-H2 真实 App“消息已到、方案未落、UI 误报未送达”失败预登记 v1

日期：2026-07-19
状态：**案发事实已冻结；待 H2 修后对照**
对应任务包：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`

## 一、结论

这不是“用户消息没送到主管”，也不是 proposal handler 拒绝了参数。真实 App 已记录用户消息、完成 invalid-resume 换代、主管完成只读勘察并生成合法 `submit_proposal`；Codex 客户端在 handler 前取消了该 MCP 调用。主管随后仍产出 final answer，但产品把回合判失败，未把主管答复落 canonical，前端因此显示了错误的“没送到主管”。

H1 的结论必须缩窄为：**测试 wrapper 和复制店证明 handler / 传输夹具可达；没有证明真实 App 产品配置可达。**

## 二、案发输入与时间线

项目：`/Users/yoyi/codex-workflow-mario-test`
真实 App store 根：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench`

1. `2026-07-19T03:06:32.262Z`（北京时间约 11:06:32）：workflow store 追加：
   - event：`supervisor_resident_user_message_recorded`
   - message id：`user:1784430392262794000`
   - text：`我想给这个游戏里的标题改成小马里奥`
2. 旧 generation 5 resume 无效后，S1B 自愈创建 generation 6 thread：
   - `019f7857-0630-7d50-910d-855fa3e0d87a`
3. 主管只读检查 `README.md` / `index.html`，识别浏览器标题与首屏标题，并说明会最小替换、保护既有脏改。
4. `2026-07-19T03:09:04.978Z`：主管发起字段完整的 `mcp__supervisor_orchestrator__submit_proposal`。
5. `2026-07-19T03:09:05.009Z`：rollout 记录：
   - server：`supervisor_orchestrator`
   - tool：`submit_proposal`
   - result：`Err: user cancelled MCP tool call`
   - duration：0；说明调用停在客户端批准层，未进入 handler。
6. `2026-07-19T03:09:12.864Z`：同一 turn 仍产出 final answer，明确“当前方案卡提交被取消，因此尚未创建方案卡，也没有修改文件”。
7. App UI 最终显示：`这句没送到主管——稍后再试一次。`

## 三、直接证据

### 1. 用户消息确实已落 canonical

文件：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

证据字段：

- `event_id=supervisor-resident-message:user:1784430392262796000`
- `event_type=supervisor_resident_user_message_recorded`
- `message_id=user:1784430392262794000`
- `message_text=我想给这个游戏里的标题改成小马里奥`

因此“没送到主管”与 canonical 事实冲突。

### 2. 工具调用在 handler 前被取消

rollout：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/supervisor-resident-homes/18ad059764393529/active/sessions/2026/07/19/rollout-2026-07-19T11-06-37-019f7857-0630-7d50-910d-855fa3e0d87a.jsonl`

关键记录：

- 合法 `submit_proposal` 调用参数完整出现；
- `mcp_tool_call_end.result.Err=user cancelled MCP tool call`；
- 之后仍有主管 final answer 与 `task_complete`。

handler 没有被调用的旁证：

- proposal store 无新增；
- resident orchestrator 无本轮 proposal materialized 记录；
- 调用耗时为 0。

### 3. final message 已存在

文件：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/supervisor/e44505db8730affd/step-0.last-message.txt`

内容明确说明两处标题、建议改法、卡片提交被取消、文件未改。它证明主管回合已经形成可给用户的自然答复。

### 4. 真实产品私有配置缺少 H1 覆盖

文件：

`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/supervisor-resident-homes/18ad059764393529/active/config.toml`

当前只有：

- `mcp_servers.supervisor_orchestrator.command`
- `mcp_servers.supervisor_orchestrator.args`

没有：

- `enabled_tools=["submit_proposal"]`
- `tools.submit_proposal.approval_mode="approve"`

代码对照：

- H1 覆盖只出现在 `supervisor_resident_oneshot_tests.rs` 的 ignored-live wrapper；
- 产品配置由 `supervisor_session_launcher.rs::supervisor_mcp_config_toml` 生成，目前只序列化 command / args。

### 5. 卡、链、项目与进程结果

案发后只读快照：

- `workflow-state/project-proposals.v1.json`：revision **131**，proposal **74**；最新 proposal 早于本轮，无“小马里奥”Pending 新卡。
- `workflow-state/workflow-state.v0.json`：workflow chain **40**，本轮未新增。
- 主管 final answer 明确“没有修改文件”；rollout 只执行只读 `rg` / `git status` / `git diff`。
- `workflow-state/exec-process-registry.v1.json`：revision **1074**，`entries=[]`。

## 四、代码静态证据

1. `ProjectJiaobanPanel.tsx` 的 fallback 是：
   - `humanizeProviderUnavailable(error) ?? "这句没送到主管——稍后再试一次。"`
2. `useJiaobanConversationState.ts`：
   - 只有命令成功才刷新 canonical workflow / proposal；
   - 外层 catch 只写 generic error，不刷新，因此已记录的用户消息可能暂时不上脸。
3. `supervisor_resident_oneshot_session.rs`：
   - `"turn.failed" | "error"` 共写 `terminal_error`；
   - `"turn.completed"` 只写 boolean；
   - 进程结束后先返回 `terminal_error`，再检查退出码 / completion / final message。

## 五、事实与推断分界

### 已直接证明

- 用户消息已记录。
- invalid-resume 换代成功，主管实际读了项目。
- 主管生成合法工具调用。
- Codex 客户端返回 `user cancelled MCP tool call`。
- proposal handler 未物化新卡，chain 未启动。
- 同一主管 turn 最终形成了自然语言 final answer。
- UI 给了与 canonical 冲突的“没送到”文案。
- 真实产品配置没有 H1 wrapper 的单工具覆盖。

### 高置信推断，尚待 H2 测试闭合

- “产品配置缺少单工具批准”是 handler 不可达的首要原因。
- runner 将早期 / 可恢复 `error` 粘成 terminal error，是“已有 final answer 却命令失败”的首要候选；案发 stderr 的 websocket reset 与最终 task_complete 同时存在，但没有独立保存原始 stdout 顺序，所以不能把这一点写成已直接证明。

## 六、修后对照必须回答

1. 真实 App initial 与 resume 是否都实际读到**仅 submit_proposal**的可用 / 批准配置？
2. 同样输入是否保留自然对话，并只新增一张 Pending 卡？
3. 卡片失败时，主管 final answer 是否仍落 canonical？
4. UI 是否只在 canonical 未记录时说“没送到”？
5. chain、项目文件、沙箱、全局审批、其他工具是否保持不变？
6. 同一 turn 的技术重试是否保持 at-most-once 落卡？

只有以上对照在真实 Tauri App 成立，H2 才能收口；复制店 / ignored harness 不能代替。
