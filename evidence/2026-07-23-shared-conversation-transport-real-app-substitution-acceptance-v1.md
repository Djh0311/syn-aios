# 共享 Conversation Transport 真实 App 替代性验收证据 v1

- 日期：2026-07-23
- 执行合同：`tasks/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-package-v1.md`
- 结论：**FAIL / 按首句停止条件安全收口**
- 范围：固定测试项目、当前源码新建 debug binary、交办页新会话首句一次

## 1. 结论摘要

当前离线实现尚未通过真实 App 替代性验收。

首句只发送一次。页面出现该用户消息，随后显示：`消息已送到主管，但主管这次没回上来——可以再发一次。` 本轮没有自然回复，也没有可信 binding、MCP 工具发现、方案卡或执行增量。按照合同立即停止，没有发送第二句和第三句，没有重试、refresh、补卡、点卡、批准卡或启动 chain/worker。

最早可证失败面是：**主管 conversation turn binding 没有形成可持久化事实。** 发送后、退出前与退出后，JSON 和 SQLite 都保持原有 `sessions=25`，`conversation_turn_binding=0`。源码顺序表明 host 应先建立 binding，再启动 supervisor transport；但公开命令会把 binding 构造/持久化内部错误折叠，前端又把 start rejection 收敛为通用失败，因此本证据不能诚实地继续归因为某一个 DB、校验、projection 或 `thread.started` 子分支。

## 2. Gate 0 与冻结物

- product-line HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：0；既有大脏树按原归属冻结，未 reset/clean/stash，未覆盖非本包脏改。
- scoped Workbench/Tauri/dev/Vite/Codex/MCP process：本包目标进程为 0；workflow/registry/SQLite holder 为 0；registry revision/entries=`1140/0`。
- SQLite 普通 immutable 只读检查：integrity=`ok`。
- storage initialized/degraded=`42/11`。
- 固定测试项目：HEAD=`caa02ded684d9e1d92d00c367949fab6f83430d1`，staged=0，porcelain=14，全文件 manifest=`f9c8867116851f688ee1311869c8703fd1f7f4f833cecd482eb42bb9115ad9a4`。

### 2.1 Gate 0 业务基线

| 事实 | Gate 0 |
|---|---:|
| workflow revision / audit | 303 / 1800 |
| recorded / injected / supervisor reply / delivery diagnostic | 14 / 5 / 5 / 1 |
| proposal / Pending / decision / proposal audit | 74 / 17 / 57 / 131 |
| supervisor session / audit / conversation binding | 25 / 263 / 0 |
| chain run / execution attempt / node dispatch | 40 / 164 / 404 |

### 2.2 承重源码冻结 hash

| 文件 | SHA-256 |
|---|---|
| `manual_relay/conversation_transport.rs` | `6a611416a15e2f37d6722ff902364651fa00a8a843d944d65873ff25c13f4bed` |
| `commands.rs` | `b5a9dd8aebeb6f95e7783288313e804a0db40d963db8ab2b44ec2d7281f45078` |
| `mcp/capability_registry.rs` | `7bca0d04c980be25685f03bd55957917d75ab3b55c73a3812c8bcd410b5d4f3b` |
| `mcp/supervisor_conversation_binding.rs` | `6d326503d1d9195ddca29e162fd4d3374da8c517b33695187ac718df26ef7175` |
| `mcp/supervisor_orchestrator.rs` | `33aa18716465aa01e6c01c39f9d6044f6d270c232a935c507816ca7bfbff136c` |
| `conversationTransport.ts` | `a2c2bbce597338ae7c07ca39cf0be40d3b5b9536f2b331d81a51fbcdfed75dbc` |
| `ProjectJiaobanPanel.tsx` | `e21a71f58d5cb5e2afbab8bdf6c6aad97b991b0565ee3436b49475dfab750b74` |
| `useJiaobanConversationState.ts` | `b86a1dff8b75e8dcb72c746cb3876473ed09a1ad0f551ee472e2a433d33ca071` |
| `JiaobanConversation.tsx` | `45873256bd5160339185d8d69a5c7707a396a53f9188474f425ca641d4d3c4eb` |

## 3. Gate 1：当前源码新 binary

运行：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

- exit 0；前端 build 与 Rust build 均成功；Rust 598 warnings。
- binary：`prototypes/productized-desktop-shell/src-tauri/target/debug/codex-governance-workbench`
- SHA-256：`31d87f1da5af7d37d3ea08db1dd87f34c30d8d6c6539153298ac1efe90c7bdc3`
- size：`67597080`
- mtime：`2026-07-23T01:45:57+0800`
- build 前后九个承重源码 hash 完全相同。

## 4. 首句真实 App 结果

在固定测试项目交办页选择新交办，只发送一次合同首句。观察到：

1. 输入区在等待时显示主管处理中；
2. 页面新增一条本轮用户消息；
3. 数秒后输入解锁，没有主管自然回复；
4. 页面显示主管未回提示；
5. 立即停止，第二句和第三句均未发送。

发送后、退出前的安全对账：

- workflow revision/audit=`306/1803`；recorded=`15`，injected/reply/diagnostic=`5/5/1`；
- proposal/Pending=`74/17`；chain/execution attempt/node dispatch=`40/164/404`；
- supervisor JSON revision/session/audit=`300/25/263`，全 store conversation binding=`0`；
- registry revision/entries=`1142/0`。

因此首句只证明 canonical `recorded +1`；没有证据证明 supervisor transport、`thread.started`、Active binding 或首次 MCP `tools/list` 成立。

## 5. 退出与最终对账

窗口关闭与 App 菜单 Quit 都没有结束本轮冻结 binary。随后只对本轮启动终端发送一次中断；终端 exit=1。最终：

- 精确 binary 进程=0；workflow、registry、DB/WAL/SHM holder=0；registry entries=0；
- SQLite 主文件/WAL/SHM 在零 holder 后复制到 `/private/tmp`，仅对副本运行 integrity 与查询，integrity=`ok`；
- 固定测试项目 HEAD、porcelain、staged 与 manifest 均与 Gate 0 相同。

### 5.1 Gate 0 → 最终业务对账

| 事实 | Gate 0 | 最终 | 增量 |
|---|---:|---:|---:|
| workflow revision / audit | 303 / 1800 | 307 / 1804 | +4 / +4 |
| recorded | 14 | 15 | +1 |
| injected / supervisor reply / delivery diagnostic | 5 / 5 / 1 | 5 / 5 / 1 | 0 / 0 / 0 |
| storage initialized / degraded | 42 / 11 | 45 / 11 | +3 / 0 |
| proposal / Pending / decision / proposal audit | 74 / 17 / 57 / 131 | 74 / 17 / 57 / 131 | 0 / 0 / 0 / 0 |
| supervisor session / audit / binding | 25 / 263 / 0 | 25 / 263 / 0 | 0 / 0 / 0 |
| chain / execution attempt / node dispatch | 40 / 164 / 404 | 40 / 164 / 404 | 0 / 0 / 0 |
| registry revision / entries | 1140 / 0 | 1143 / 0 | +3 / 0 |

DB 副本也给出：supervisor sessions=`25`、含 conversation binding 的 session=`0`、project proposals/Pending=`74/17`、chain/execution attempt/node dispatch=`40/164/404`。

最终关键 store SHA-256：

- workflow：`75ee9f9fda819db5a6152a6d7f33d485f260d8cb05b01fd66941569ef2c3a41e`
- proposal：`3d7d965e02fb12761d5f7e9d85218fd154050131edf77e92951f90540238f631`（与 Gate 0 相同）
- supervisor：`e63079fcaad521a823e33a2c4cc1bce9ecb2536f4e24e9f2407646a914f7140b`（与 Gate 0 相同）
- registry：`a7f28b09d7ad72e6dc77cb2c0f0342682600a43fd7c18175affc91a3b747b854`
- SQLite 副本：`826474fd2d6a628d2f2dde90dd46daf880a47b8e9f8843b21fdb517465c8f52b`

## 6. 最早可证边界与未决子因

静态源码顺序为：

1. host 解析并冻结 project/workflow context；
2. 构造固定 `supervisor-read-only`、空写根、`submit_proposal` 单能力 binding；
3. `establish_supervisor_conversation_turn_binding` 通过现有 DB-primary + JSON projection 持久化；
4. 才启动真实 conversation transport；
5. 仅宿主观察到 `thread.started` 后把 binding 激活为 Active。

本轮第 3 步没有形成可读事实；第 5 步及 MCP 工具发现没有可验前提。当前命令把 binding 构造与持久化内部错误分别折叠为稳定通用 family，前端 start catch 又不保留该 family；没有 message-scoped、脱敏的 binding-stage receipt。因此本证据只裁决到“binding 持久化未成立”，不猜以下任一单因：

- binding 输入校验；
- DB-primary delta 写；
- JSON compatibility projection；
- 锁或同步边界；
- `thread.started` / 首次 `tools/list` 时序。

## 7. 验收判定与下一包入口

真实 App 替代性验收未通过；离线通过不能升级为产品通过。下一包应先做**共享主管 binding 建立链的离线可观测性与最小修复**：在不读取敏感值、不放宽 read-only/空写根/allowlist 的前提下，为 binding construct、DB-primary persist、JSON projection 建立 message-scoped 安全 receipt/测试，使用真实 store 的只读副本形状复现并修复最早失败点。修复包离线全绿后，再另开一次全新真实 App 三句验收。

本包没有修改 Rust、TypeScript、测试、配置、schema 或固定测试项目；没有 stage、commit、push。
