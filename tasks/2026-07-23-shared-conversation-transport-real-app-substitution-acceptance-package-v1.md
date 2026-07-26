# 任务包：共享 Conversation Transport 真实 App 替代性验收 v1

- 日期：2026-07-23
- 状态：**已执行并按首句停止条件收口；验收未通过**
- 后续重验入口：`tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md`
- 前置实现：`tasks/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-implementation-package-v1.md`
- 前置证据：`evidence/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-verification-v1.md`
- 类型：真实 App 高风险验收；不含代码、配置、schema 或真实数据修复

## 0. 授权与唯一目标

用户在 2026-07-23 于离线收口后明确表示“现在就可以开”，授权从全新 Gate 0 开启共享 Conversation Transport 的真实 App 替代性验收。

唯一目标：用当前源码新构建并冻结的裸 debug binary，在固定测试项目交办页完成一次新会话的三句受控验收，证明共享 transport、可信 binding、MCP `submit_proposal` 与分层 receipt 在真实 Codex client 中可替代旧 resident 主路线。

## 1. 永久边界

允许：

- 只读 Gate 0/结束对账；构建并启动当前源码对应的一枚裸 debug binary；
- 三句各一次的真实产品交互；正常写入对话 canonical、工具审计及至多一张 `PendingUserConfirmation` 卡；
- 一次 UI refresh；正常 Quit；写脱敏 evidence，并最小同步 `CURRENT.md`、`AUTHORITY.md`；
- 若正常 Quit 后只残留本轮已冻结、身份精确匹配且零 holder 的单一裸 binary，可在用户本次现场授权内向该 PID 发送一次 `TERM`。

禁止：

- 不修改 Rust/TypeScript/test/config/schema/依赖，不直接写、恢复、reseed、migrate、reconcile 或回滚真实 store；
- 不放宽主管 `read-only + 空写根`、MCP 精确 allowlist、approval、path-lock 或进程清理；
- 不重发、不补卡、不批准卡，不启动 chain/worker，不修改固定测试项目；
- 不读取或输出凭据、用户正文、完整 identity、raw arguments、stderr、argv、环境或私有路径；
- 不 stage、commit、push、reset、clean 或 stash。

## 2. Gate 0：全新现场

1. 确认 Workbench/Tauri/dev/Vite/Codex/MCP scoped process、registry、lock、workflow state 与 DB/WAL/SHM holder 全空；registry entries 为 0。任一非空即停止，不启动。
2. 冻结 HEAD、staged、porcelain、共享 transport 承重源码 hash 与脏改归属；不覆盖既有脏文件。
3. 普通只读检查 SQLite integrity、DB/JSON 安全投影与 storage mode；冻结 proposal/Pending/chain/worker、共享 canonical 事件、binding/tool outcome 基线。
4. 冻结固定测试项目 HEAD、porcelain 与全文件 manifest。历史 resident 事件不得计入本轮。

## 3. Gate 1：当前源码新 binary

在 `prototypes/productized-desktop-shell` 运行：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

要求 exit 0，承重源码 build 前后 hash 不变；冻结 `src-tauri/target/debug/codex-governance-workbench` 的 SHA-256、size、mtime。只启动这一枚 binary。

## 4. 真实 App 三句验收

### 4.1 首句：新会话与首次工具发现

用户在固定测试项目交办页只发送一次：

`我想给这个游戏里的标题改成小马里奥`

要求：自然回复成功；真实 `thread.started` 被宿主观察并形成 Active binding；同一 turn 的首次 MCP `tools/list` 能看到且只看到 `submit_proposal`，或有等价的可信服务端事实。若只返回空列表、binding 尚未 Active、无回复或身份不可对账，立即停止，不发第二句。

### 4.2 第二句：单工具与单卡

仅首句全绿后只发送一次：

`按这个出方案`

要求：同一 thread；`submit_proposal` handler 到达并成功；自然回复保留；proposal/Pending 严格 `+1/+1`，chain/worker `+0/+0`，目标卡匹配“小马里奥”。看到一张卡立即停止，不点卡。

### 4.3 第三句：续聊

仅第二句全绿且未点卡后只发送一次普通追问：

`先别执行，告诉我这个方案准备改哪些地方。`

要求：仍在同一 thread，自然回复成功；proposal/Pending/chain/worker 不再增长。不得再次调用或补发 `submit_proposal`。

## 5. receipt 与失败裁决

- tool、audit、projection 或 canonical 任一失败时，已经成立的 assistant reply 和业务结果必须保持；失败只落在对应层。
- 若 audit 失败但 Pending 已成立，只记录 audit/canonical 失败，不重发、不补卡。
- 任一失败按最早可证边界停止；本包不现场修码，不从“没卡”猜单一根因。

## 6. 关闭与完成标准

成功候选只允许 refresh 一次，所有本轮计数不得重复增长。随后正常 Quit，并核对：

- scoped process、holder、registry、lock 全空；
- DB/JSON/integrity 可只读对账，无新增不可解释 degradation；
- 固定测试项目 manifest 不变；
- 三句同一 thread，恰一张 Pending，未批准，chain/worker 零增量；
- staged 仍为空，未 commit。

只有以上全部成立才可称“共享 Conversation Transport 真实 App 替代性验收通过”。

## 7. 交付物

- `evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md`
- 最小同步 `CURRENT.md`、`AUTHORITY.md`
- 若出现新的实际拦截，追加 `docs/harness-catch-log.md`；否则明确零 catch。

## 8. 实际执行结论（2026-07-23）

- Gate 0 与当前源码 debug build 均通过；只启动了冻结 SHA-256 为 `31d87f1da5af7d37d3ea08db1dd87f34c30d8d6c6539153298ac1efe90c7bdc3` 的裸 binary。
- 固定测试项目首句仅发送一次。界面新增该用户消息，随后显示主管未形成回复；按本包停止合同，没有发送第二句或第三句，没有 refresh、点卡、补卡或重试。
- 最早可证失败面是主管 conversation turn binding 未持久化：发送后、退出前与退出后，JSON/SQLite 均保持 `sessions=25`、`conversation_turn_binding=0`；`recorded +1`，但 `injected/reply/proposal/Pending/chain/execution_attempt/node_dispatch` 全部零增量。
- 当前证据不能继续把失败压成 binding 构造、DB-primary delta、JSON projection 或 `thread.started` 时序中的某一单一子因；第二、第三句因此不具备执行前提。
- 窗口关闭和 App 菜单 Quit 均未结束本轮裸 binary；最终只向本轮启动终端发送一次中断。退出后精确 binary 进程、store holder、registry entries 均为 0，SQLite 副本 integrity=`ok`。
- 固定测试项目全文件 manifest 与 Gate 0 相同；承重源码 hash 不变；未修改代码、测试、配置、schema 或固定测试项目，未 stage、commit、push。

完整脱敏证据见 `evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md`。

本包第 4.1 节“只看到 `submit_proposal`”保留为当时快照的历史验收谓词。知识能力接入后的后续验收采用
`decisions/2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md`
冻结的精确五工具面，不回写本次历史结果。
