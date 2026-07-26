# 任务包：S1B-H2-R4C 全新 Gate 0 与真实 App Pending 卡复验 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；必须由用户在场另行精确授权**
- 类型：真实 App 高风险验收；不含代码、数据或配置修复
- 前置证据：`evidence/2026-07-22-s1b-h2-r4b-safe-offline-reconcile-probe-verification-v1.md`
- 原始产品合同：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`
- 历史现场合同：`tasks/2026-07-22-s1b-h2-r4-real-app-diagnostic-and-pending-card-verification-package-v1.md`

## 0. 唯一目标

在**新的** Gate 0、**新的**当前源码 debug build 与**新的**裸 binary 冻结后，用户在固定测试项目中只发送一次首句。只有同一 message 完成 canonical recorded、injected 与主管自然 reply 后，才发送一次“按这个出方案”。只允许 `supervisor_orchestrator.submit_proposal`，并在恰好新增一张目标匹配的 `PendingUserConfirmation` 卡后停止。

成功出口不是“消息看起来发出”，而是：首句一次、第二句一次、同 identity 的自然回复与 handler/receipt 可关联、恰一张 Pending 卡、一次 refresh 不重复、chain/worker/项目不动。失败出口是一次失败动作加可归因的安全事实，绝不重发或现场修码。

## 1. R4B 后可用事实与不可推断项

R4B 在一次已删除的 0700 最小离线副本中直接运行既有 Rust reconciler，实测 `project_proposals` 为 DB/JSON/matched=`74/74/74`，DB-leading/JSON-leading/hash-mismatch=`0/0/0`，stored hash shape 为 `74/74`。它排除了**该冻结副本、该目标表**的 shared-key canonical-hash/freshness 差异。

这不证明真实 App startup 已绿，不证明 R4-R2 的历史启动期 fail-closed 已被解释或消失，也不代替现场 Gate 0、当前 binary 冻结或 H2 自然对话验收。不能据此重 seed、写 store、放宽任何安全闸，或直接发送消息。

## 2. 唯一现场 kickoff

只有用户在现场发送以下等价精确授权时才可开始：

> S1B-H2-R4C 开工；用户在场，授权从全新 Gate 0 重新核空 holder、冻结当前 store/项目与源码，重新构建并冻结当前裸 debug binary。只启动该 binary；首句只发送一次，只有同 message injected 且主管自然 reply 后才第二句一次。只允许预批准 `supervisor_orchestrator.submit_proposal`，验到一张目标 Pending 卡即停，不点卡、不起 chain、不改项目。

R4B 的 green、旧 R4/R2 kickoff、旧 binary 或任何离线完成回报都不能替代这条授权。

## 3. 永久边界

### 允许

- Gate 0/结束时只读检查仓库、进程、holder、registry、真实 store、DB/WAL/SHM、固定测试项目及安全 read model；
- 构建本包当前源码对应的 debug binary，并正常启动这一枚冻结的裸 executable；
- 产品正常记录两条 canonical 对话、正常 supervisor reply、必要的安全 diagnostic/审计以及至多一张 Pending 卡；
- 正常关闭 App，写脱敏 evidence 与最小 `CURRENT.md` 更新；仅出现新 interceptor 时向 catch log EOF 追加。

### 禁止

- 不修改 Rust/TypeScript/test/config/schema/依赖/脚本，不直接写真实 JSON/DB/WAL/SHM，不 apply/reseed/migrate/recover/rollback；
- 不 reset/clean/stash/stage/commit/push，不自行 kill 进程；任一 holder/残留必须 `BLOCKED_LIVE_HOLDER` 并等待用户处理；
- 不复用 R2/R4 的 baseline、binary、client identity 或 message identity；不双击、不自动重发、不发送第三句或技术性重试；
- 不批准卡、不启动 chain/worker、不改固定测试项目；
- 不批准第二工具、wildcard/default approval、full-auto/bypass，不放宽 read-only、approval、sandbox、reviewer、path-lock、写根、watchdog、invalid-resume 单次轮转或进程组清理；
- evidence/read model 不得写用户正文、完整 identity、原始 stderr/error、record JSON、token/auth、私有 home 或绝对私有路径。

## 4. Gate 0：每次重新开始

1. 用户关闭 App 后，确认 Workbench/Tauri/dev/Vite/Codex/MCP、registry、workflow-state、DB/WAL/SHM holder 全空；registry entries 必为 0，任一非空即止损，不 kill。
2. 冻结 HEAD、staged、porcelain 和已归属的 dirty ownership；重算以下八个 R4 源码 hash。任一漂移或无法归属的相关变动为 `BLOCKED_DIRTY_OVERLAP`：

   ```text
   552531eab5ae6f9beae7c857c6a438b8794dd52f9347e56052318232e3f509e8  src-tauri/src/supervisor_resident_oneshot_session.rs
   6c8ba3dbc0c38ad43a132651f216a35715120a1e82ab3ebc3208bdf97ee0da14  src-tauri/src/supervisor_resident_oneshot_tests.rs
   d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845  src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs
   6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b  src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs
   7f382cadf799f9dc6e4a34e86b22aca666d9bb8983dee717c235d85c2e03252e  src-tauri/src/workflow_read_model_entrypoints.rs
   47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2  src/views/projects/jiaoban/useJiaobanConversationState.ts
   279a728ee6487f7e8afecf5e81ad4df1dccb06b2b68179fafc2444a5bce3cb92  src-tauri/src/knowledge_vault.rs
   d15bbdb16dee75dc415d1e4e050b275eb89e75a940f1a93c281e7845693519f0  src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs
   ```

3. 以普通只读口径复核 SQLite integrity、storage mode、DB/JSON 核心投影与 revision；冻结当前 source hashes、`project_proposals`/Pending/chain、DB-primary initialized/degraded 与 resident generation/thread/session。只读打开失败不得转可写恢复。
4. 冻结 `R0/I0/S0/D0/B0/P0/C0`，其中依次为 user recorded、same-message injected、supervisor natural reply、delivery diagnostic、proposal total、Pending total、chain total；同时冻结 dispatch/binding/attempt/control 的辅助计数。
5. 冻结固定测试项目 HEAD、porcelain、全文件与业务文件 manifest。R3B/R4 之后的任何既有消息、diagnostic 或卡必须先归属，不能算作本轮增量。

Gate 0 任一项不绿，不 build、不启动、不发送。

## 5. Gate 1：当前裸 binary

在 `prototypes/productized-desktop-shell` 使用既有 debug build 入口：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

要求 exit 0；八个 hash build 前后相同；冻结新 `src-tauri/target/debug/codex-governance-workbench` 的 SHA-256、大小、mtime 与构建时间。只可启动这枚新冻结的裸 executable，不能启动历史 `.app` 或旧 binary。build 后再次核无残留 holder；失败不修码、不降级为旧 binary。

## 6. Gate 2：启动健康与首句

启动 Gate 1 binary，进入固定测试项目交办页。若 startup 在 composer 可用前报新的 reconciliation/health 阻断，**首句不得发送**；正常关闭、只读记录安全的 stage/family/计数后另出包，不从私有 stderr 猜根因。

若启动健康，提醒在场用户只提交一次，生成新的 client/message identity，精确发送一次：

`我想给这个游戏里的标题改成小马里奥`

不得再次点击。只有同一 identity 同时满足 `R=R0+1`、`I=I0+1`、`S=S0+1`、`D=D0`，UI 有自然主管回复，proposal/Pending/chain 仍为 `B0/P0/C0`，且 lifecycle/thread/generation 可关联，才可进入 Gate 3。

若首句不完整：不发送第二句；只读取同一 message 的安全 diagnostic `stage/stable_error_family/generation/thread`。recorded 无增、diagnostic 缺失/重复/identity 不符、injected 无 reply 或 lifecycle 断裂，分别按其精确失败边界止损并另包；不重发、不补写、不从用户面或 private stderr 推断。

现有 invalid-resume 自愈若发生，只容许既有的一次 archive→generation+1→facts→initial；必须完整记录旧/新 thread 与事实注入，递归轮转或第二次 initial 均失败。

## 7. Gate 3：第二句、唯一工具与单卡

仅 Gate 2 全绿后，以另一条新的 identity 精确发送一次：

`按这个出方案`

要求：

1. 相对 Gate 0，`R/I/S` 均严格 `+2`，`D=D0`；两句自然 reply 与工具结果留在可关联 thread。
2. 只允许预批准 `supervisor_orchestrator.submit_proposal`。出现工具批准弹窗、第二工具、approval 扩张或 raw internal error 即停止，不点击。
3. 必须有 handler 到达、tool receipt 与主管自然 final；proposal=`B0+1`、Pending=`P0+1`、chain=`C0`，且卡目标匹配标题改为“小马里奥”。
4. 若自然对话成功但工具/落卡失败，绝不重发第二句；记录 handler 前后安全事实、零/非一张卡边界后另出工具路径包。
5. 看到一张目标 Pending 卡立即停止，不点卡、不批准、不起 chain/worker。

## 8. Gate 4/5：一次 refresh、正常关闭与结论

只有 Gate 3 成功落卡才 refresh 一次。refresh 后消息、diagnostic、proposal/Pending/chain 均不得增长，仍仅一张本轮目标卡。

正常关闭 App（不 kill），再核 registry、holder/process、DB/JSON/integrity、storage initialized/degraded、固定项目 manifest 与 source hash。成功路径必须严格为：

```text
R=R0+2, I=I0+2, S=S0+2, D=D0,
B=B0+1, P=P0+1, C=C0
```

任一数字不符、Pending 非一、chain/worker/项目有变化、holder 残留或状态无法解释，立即止损；不自动回滚、rebase、删除配置或再次启动。

## 9. 证据与十项回传

新 evidence 只能记录脱敏 Gate 0/1、binary digest、计数 delta、identity short tail、safe diagnostic stage/family、handler/receipt 事实、卡 delta/refresh、DB/JSON/项目与进程对账；不能存私有原文。最小更新 `CURRENT.md`；仅新 interceptor 才 catch log EOF。不得 stage/commit。

回传必须明确分开：

1. Gate 0/1 与 binary 冻结；
2. HEAD/staged/dirty ownership 与八 hash；
3. `R0/I0/S0/D0/B0/P0/C0` 与最终 delta；
4. 首句唯一 identity、lifecycle、thread/generation；
5. 首句自然回复或同 message 的安全 diagnostic；
6. 第二句是否发送、handler/tool/final；
7. Pending 唯一性与 refresh；
8. chain/worker、DB/JSON、项目 manifest；
9. 关闭后的 registry/holder/process；
10. evidence/CURRENT/catch/下一包、stage/commit 与未执行项。

只有全部成功条件成立才能称 H2 真实 App 通过；R4B 离线 green 本身不得被表述为 live 通过。
