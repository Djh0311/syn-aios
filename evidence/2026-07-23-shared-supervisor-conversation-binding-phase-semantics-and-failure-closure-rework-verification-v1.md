# 共享主管 Conversation Binding 阶段语义与失败收口返工验证 v1

日期：2026-07-23  
范围：仅本地源码、测试夹具和系统临时目录；未启动真实 App，未读取或修改真实 store，未 stage/commit

## 基线与边界

- 基线 HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；暂存区保持为空。本工作树原本已脏，未 reset、clean、stash 或接管无关差异。
- 所有 DB-primary 验证由 `std::env::temp_dir()` 下的新建 workflow JSON、sidecar 和 SQLite fixture 完成；fixture `Drop` 删除其自身临时根目录。
- 没有执行前序的真实私有副本 replay ignored test，也没有设置其环境变量。

## 修正后的可观察状态机

前序文档只列出 construct / DB persist / JSON projection / transport start，无法如实表达 store 准备、activation 和终结未确认。本次后端和前端共享如下固定阶段：

`binding_construct → binding_store_prepare → binding_persist_db → binding_project_json → binding_activate → transport_start → binding_terminate`

这不是成功路径顺序图；它是 receipt 的互斥失败分类。`binding_terminate` 优先于原启动失败阶段，因为没有 durable 证据时不得声称 binding 已终结。

## 四类注入验证

| 注入位置 | 真实执行的失败钩子 | 阶段与 lifecycle 证据 | tools/list 与 tools/call |
| --- | --- | --- | --- |
| store 准备 | DB-primary lock 后、load 前的 `StorePrepare` | establishment 返回 `BindingStorePrepare`；JSON 仍 25 session，DB 仍 `25/0` | list 为空，call 拒绝 |
| activation | `Activate` 钩子 | command receipt=`binding_activate`；临时 JSON 和 SQLite record lifecycle 均为 `failed` | list 为空，call 拒绝 |
| transport start | production error-wrapper 注入 `Err` | command receipt=`transport_start`；临时 JSON 和 SQLite record lifecycle 均为 `failed` | list 为空，call 拒绝 |
| termination | `Finish` 钩子 | receipt=`binding_terminate` 且人话为“终结未确认”；JSON 和 SQLite 都仍为 `active`，因此没有伪称 Failed | list 为空，call 拒绝，依靠进程内 fail-closed guard |

`binding_terminate` 不是补写 Failed 的替身：它故意保留“未确认”事实，并以独立闭锁阻止潜在 Active binding 再发布工具。

## 前端边界

- `SupervisorConversationBindingStage` 和 runtime `safeSupervisorBindingStage` 同步允许 `binding_store_prepare`、`binding_activate`、`binding_terminate`。
- 离线交互测试将这三个 receipt 阶段逐个穿过 controller；未知阶段仍被剔除，终结未确认的 UI 文案不含“已终结”，激活失败的 UI 文案也不臆测“运输没有启动”。
- catch 路径只把已知稳定 code 映射到阶段；raw error 不进入 UI/controller state。

## 定向命令结果

| 命令 | 结果 |
| --- | --- |
| `cargo test --lib 'mcp::supervisor_conversation_binding::tests' -- --nocapture` | 5 passed |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::shared_supervisor_' -- --nocapture` | 12 passed，1 ignored（需要显式私有副本，未运行） |
| `cargo test --lib 'injected_supervisor_' -- --nocapture` | 3 passed（activation、transport、termination receipt 注入） |
| `npm run typecheck` | passed |
| `node scripts/run-offline-interaction-test.mjs` | 15 passed |
| `cargo check --lib` | passed；既有 warnings 598 条 |
| `git diff --check` | passed |

第一次从仓库根目录调用离线交互脚本时，路径不存在并立即返回 `MODULE_NOT_FOUND`；没有执行测试或修改文件。随后在 `prototypes/productized-desktop-shell` 正确工作目录重跑，上表的 15 组全部通过。

## Shape 历史债务（单列，不作为绿灯）

`node scripts/harness/workbench-shape-gate.js --mode check` 的结果为 `Status: fail`，`Errors/Warnings/Info=16/5/5`。这是工作树既有的大范围 ratchet、超限文件和未知 sidecar debt；本包没有调高 baseline，也没有试图把它写成绿。`commands.rs` 与 `mcp/supervisor_orchestrator.rs` 本身也在既有超限清单中，因此本次只交付必要的最小修复，不把定向测试通过解释为 shape gate 通过。

## 裁决与不声称

离线证明只覆盖：源码中的失败分类、临时 DB/JSON fixture lifecycle、前端枚举过滤，以及工具面 fail-closed 行为。它不证明真实 App 首句会进入这条链，也不证明真实 source 的任何 lifecycle 或数据已经变化。

前序 [建立链离线验证](2026-07-23-shared-supervisor-conversation-binding-establishment-offline-verification-v1.md) 中“四阶段已可完整区分”及“transport failure 已终结”的过度表述已由本文纠正；其历史副本 Starting 写入观察不被否定，但不再构成完整失败收口结论。
