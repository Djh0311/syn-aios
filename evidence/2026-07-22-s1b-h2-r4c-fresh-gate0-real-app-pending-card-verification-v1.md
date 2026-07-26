# S1B-H2-R4C 全新 Gate 0 真实 App Pending 卡复验 v1

- 日期：2026-07-22（+0800，用户在场）
- 任务包：`tasks/2026-07-22-s1b-h2-r4c-fresh-gate0-real-app-pending-card-verification-package-v1.md`
- 结论：**Gate 0/1/2 通过；Gate 3 按失败矩阵止损；S1B-H2 真实 App 两句→Pending 卡验收未通过。**

## 结论与不外推项

本轮从新的 Gate 0、当前源码 build 和新冻结的裸 binary 开始。两条由用户各发送一次的对话都完成了同一 canonical 的 `recorded → injected → supervisor natural reply` 链，且没有 delivery diagnostic；因此首句的自然对话通路已实证恢复。

第二句也完成了该对话链，但没有可持久化关联的 `submit_proposal` tool call、handler receipt、resident proposal outcome 或方案卡：proposal/Pending/chain 均保持基线。按合同不重发第二句、不补卡、不刷新、不点卡、不启动 chain/worker，也没有现场修码。

这只证明**“第二句后没有可归因的持久化工具/落卡事实”**。它不能区分工具没有被本回合暴露、已暴露但主管未调用、transport/client 在 server audit 前中断，或 audit 留痕本身失败；因此不把零卡归因为单一代码或模型根因。

## Gate 0 / Gate 1

| 项目 | 结果 |
| --- | --- |
| Gate 0 holder / registry | Workbench、Tauri/dev/Vite、scoped Codex/MCP、store 与实际 DB/WAL/SHM holder 均为 0；registry `entries=0` |
| 仓库 | HEAD=`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged=0；porcelain=40（既有已归属脏形，不清理） |
| R4 源码冻结 | 任务包列出的 8 个文件为 `8/8` SHA-256 match；build 前后与结束复核均未漂移 |
| 固定测试项目 | HEAD=`caa02ded684d9e1d92d00c367949fab6f83430d1`；porcelain=14；non-`.git` full-file manifest=`f9c8867116851f688ee1311869c8703fd1f7f4f833cecd482eb42bb9115ad9a4`，结束时相同 |
| DB-primary 起点 | storage mode=`db_primary_json_projection`；普通只读 integrity=`ok`；核心 JSON/DB count-level 投影对平 |
| 当前 debug build | `cargo-tauri build --debug` exit 0；只启动 `src-tauri/target/debug/codex-governance-workbench` |
| 裸 binary 冻结 | SHA-256=`2980c45e8a61b713eb029f32f71a51f693e6c7aae5756fd413ad570218d532b2`；size=`66548968`；mtime epoch=`1784702858` |

本次确实重跑 build；binary 内容恰与较早冻结值相同，不将“字节相同”伪报为复用旧 binary。

## 两句 canonical 与 resident 关联

所有 identity 和 thread 都只以 SHA-256 短 digest 记录；未写用户正文、完整 ID、reply 或私有 runner 内容。

| 回合 | identity digest16 | recorded | injected | natural reply | diagnostic | thread digest16 | generation | reply outcome |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | --- |
| 首句 | `9c724595e7ca0d29` | 1 | 1 | 1 | 0 | `c4cdd7e81ff8e498` | 6 | `not_requested` |
| 第二句 | `231eb41321f7c7f7` | 1 | 1 | 1 | 0 | `c4cdd7e81ff8e498` | 6 | `not_requested` |

两句关联到同一 resident thread 和 generation 6；没有 invalid-resume 换代、递归 initial 或重复 message。第二句在 Gate 2 全绿后才由用户发送一次，符合 Gate 3 的唯一发送前提。

## 计数、工具与卡片边界

| 指标 | Gate 0 | 结束 | 差值 | 结论 |
| --- | ---: | ---: | ---: | --- |
| `R/I/S/D` | `11/3/3/0` | `13/5/5/0` | `+2/+2/+2/+0` | 两句均完成自然回复，无 delivery diagnostic |
| proposal / Pending / chain | `74/17/40` | `74/17/40` | `0/0/0` | 未生成本轮 Pending 卡，chain 未动 |
| dispatch / binding / attempt / control | `404/76/164/164` | `404/76/164/164` | `0/0/0/0` | 无执行侧增量 |
| DB-primary initialized / degraded | `38/11` | `39/11` | `+1/0` | App startup 正常记录一次 initialized；无新增 degraded |
| `supervisor_tool_call` | `14` | `14` | `0` | 无本轮持久化工具调用 |
| `submit_proposal` tool audit | `0` | `0` | `0` | 无 handler/receipt 的肯定证据 |
| active proposal outcome | — | `not_requested` | — | 未形成 `materialized` 或 `tool_failed` 结果 |

第二句后的 card 成功条件 `B=B0+1`、`P=P0+1` 不成立，故 Gate 4 的唯一 refresh **未执行**。这不是遗漏，也不能用 refresh 或重发弥补。

## 结束对账与关闭

- 用户先正常关闭 App；窗口关闭后发现一枚本轮冻结的裸 Workbench executable 仍残留。经用户随后明确授权，仅向该已核验的单一 PID 发送 `TERM`，没有碰父进程、store、项目或其他进程；复核 Workbench/Tauri/dev/MCP 均为 0。
- 结束时 store holder=0、实际 DB/WAL/SHM holder=0、lock=0、registry=`entries=0`（revision=1137）。无本轮孤儿。
- JSON 与 DB 的安全 count-level 对账一致：`R/I/S/D=13/5/5/0`、`B/P/C=74/17/40`、dispatch/binding/attempt/control=`404/76/164/164`、worker count=`13`、supervisor sessions/audits/actions=`25/263/34`、initialized/degraded=`39/11`。
- 关闭后普通 `sqlite3 -readonly` 打开没有成功；在 holder=0、WAL/SHM 不存在的前提下，改用 `immutable=1` 的**只读**口径核到 `integrity_check=ok` 并完成上述计数对平。没有转可写恢复、replay、reconcile、apply 或导出。此 count-level 结果不冒充 full semantic reconcile。
- `project-proposals`、plan authorizations、supervisor action sidecar 保持 Gate 0 hash；workflow state、orchestrator sidecar 和 SQLite DB 仅出现本轮正常对话/启动带来的预期变化。固定测试项目 manifest 与 8 源码 hash 均不变。

## 未执行、文档与下一步

- 未批准卡、未启动 chain 或 worker、未改固定测试项目、未直接写真实 JSON/DB/WAL/SHM、未改代码/审批/sandbox/read-only/watchdog/invalid-resume/清理规则。
- 本轮未跑新增代码或离线测试；这是现场验收与只读收尾，代码/离线既有 green 不被表述为本轮 live 通过。
- 未 stage、commit、push、reset、clean 或 stash。
- 本轮发现的“自然 reply 已完成但缺少持久化 tool/receipt/card 事实”不同于历史已记录的“明确 client cancellation”；已向 catch log EOF 追加一条脱敏拦截记录。
- 下一步只可由新的、用户另行精确授权的只读 R4D 包执行：`tasks/2026-07-22-s1b-h2-r4d-absent-submit-proposal-attribution-diagnosis-package-v1.md`。该包先建立工具可见→tools/call→handler→audit/outcome→materialization 的最早可观察边界；不启动 App、不重发、不改代码或真实 store。
