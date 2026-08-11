# M4R05 持续 Secretary 对话验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R05`

## 1. 结论

M4R05 已通过。普通 Secretary Home/Board composer 已接到 registered Tauri command、固定的 Secretary RoleSession/PersonalScope authority、既有 M3 Turn/ConversationTransport lifecycle 和 provider-owned transcript。renderer 只提交经过 trim 与 UTF-8 长度校验的 `message` 和 server-shape `client_message_ref`，不提交 session、role、scope、channel、permission、provider handle 或 transcript selector作为权威。

全新隔离 profile 中，受控 launcher 的第一个 product child process 完成两轮成功对话和一次 exact replay，再由 launcher 对该 child PID 执行并确认 `SIGKILL`；第二个 child process 从同一 profile 恢复原 RoleSession 与两轮历史，继续完成第三轮成功和第四轮 durable provider failure。最终 composite 为 `PASS`、证据等级为 `ISOLATED_PRODUCT_APP`。该证据证明的是普通产品调用链在首次进程已经完成消息后的强退恢复与继续，不外推到“首次消息前先重启再首次发送”，也不等于真实 provider、可携带 binary identity 或 OS 级网络审计。

## 2. 生产调用链与权限边界

```text
Secretary Home / Board 的普通 composer
  -> tauri.ts loadSecretaryConversation() / sendSecretaryMessage({ message, client_message_ref })
  -> registered Tauri commands
  -> AppState.m4_secretary_conversation_runtime
  -> fixed Secretary RoleSession / role / PersonalScope / channel authority
  -> current permission + current verified provider binding checks
  -> mechanical M4 deterministic brief context（included_skill_refs=[]）
  -> M3 start_role_turn + registered provider effect
  -> M3 repository-backed ConversationTransport claim / dispatch / readback
  -> provider-owned persistent transcript ledger
  -> M3 lifecycle 与 provider raw message 的 exact authorized join
  -> server-returned complete conversation snapshot
  -> App whole-snapshot replace -> Board history / terminal failure
```

M3 RoleSession、Turn、effect、receipt 与 readback 是 lifecycle 真源；provider ledger 只持有 provider-owned raw user/assistant content 与调用账本。读取任意历史前都重新校验 RoleSession=`ACTIVE`、permission=`CURRENT`、current binding=`VERIFIED`，再逐 turn 校验 client message、turn/input identity、hash、provider attempt、message ref 与 terminal time ordering。空历史也先过 session/permission authority，但直接返回空 snapshot，不读取 provider。

发送端先以 server-owned identity 幂等准备 raw input，再提交 M3 Turn/effect；claim、provider write、readback 与 apply 之间的 crash seam 都走 readback-only convergence，不重新 fresh-dispatch。相同 `client_message_ref` 与相同正文只回放同一 turn/command receipt；相同 ref 与不同正文固定拒绝；相同正文配新 ref 才形成新 turn。UI 在 pending 时不追加 optimistic user bubble，也不从 receipt 本地拼 assistant transcript；成功、terminal failure 或重载后都以服务端完整 snapshot 替换。

公开 DTO 不返回 provider handle/attempt、binding revision、permission ref、文件路径或 raw database identity。内部 provider/SQLite 错误在 command 边界映射为稳定公开 code。第四轮 synthetic provider failure 作为正常 terminal `FAILED` turn 持久化，assistant message 为 null，UI 显示固定 `M4_SECRETARY_PROVIDER_FAILURE`，而不是把失败伪造成 command transport exception。

## 3. 受控隔离产品进程行为

可携带 composite receipt 保存于：

- `docs/harness/reports/M4R05-persistent-secretary-conversation-behavior-receipt.json`
- SHA-256：`7f69faa4c1e5ca23933a671d90cdbb41c4f6ccd24988a1f85eec27db131857cd`

fresh profile：`syn-r4-acceptance-83xdXs`。profile 内 composite 与可携带 receipt 逐字相同；两份 phase receipt SHA-256 分别为：

- two_rounds_arm：`f22ef7a8707b5a064aae03d631639111cb0ca093726a45efe7712f17be4da6a0`
- restart_continue_failure：`662506c7e4ea8d0ba23ceebdb5947ec9e698144d678bc317d0f964b956e03291`

profile fingerprint 为 `4a6a46fab771523c2da94d53ff0534c94187f8a85266749c04042adc33ad7def`；第二份 receipt 的 previous SHA 精确等于第一份 receipt SHA。两个 App process hash 与两个 nonce hash 均互异。portable composite 记录 build 成功及两次受控 child launch 的行为结果，但未携带 executable SHA-256、mtime、CDHash、bundle path 或 codesign verification result。因此 `ISOLATED_PRODUCT_APP` 在本包中表示普通产品调用链的行为证据等级，不构成可携带、可重放的二进制身份或签名状态证明。当前 launcher 源码包含 fresh-build 与 strict codesign preflight；该源码控制流审查不能追溯性地把某一 executable SHA 或签名结果绑定到已保存 receipt。

关键直接事实：

- phase 1 打开 Board conversation 一次，`initial_turn_count=0`，且空白 message 的 submit disabled；随后两次真实 DOM submit 得到 2 个 `SUCCEEDED` turn、2 个 user message node 和 2 个 assistant message node。再以同一 `client_message_ref` 和正文调用普通 send wrapper，返回同一 turn，M3/provider dispatch 增量为 0。
- phase 1 baseline 只有同一个 active RoleSession 和 1 条 `REGISTERED` CREATE effect，provider session/transcript/call count 全为 0；终态为 verified handle/current binding/context 各 1，2 个 Turn、2 个 START effect/readback/receipt，provider `start_session=1`、`continue_turn=2`、`poll=3`。
- phase 1 PASS receipt 发布后，launcher 对直接 spawn 的真实 App PID 执行强退并确认 `signal=SIGKILL`、`timed_out=false`；不是只杀 waiter。phase 2 使用同一 profile、不同 PID/nonce 启动，初始历史 exact 恢复为前两轮，restart load 的 provider dispatch 增量为 0。
- phase 2 再经两次真实 DOM submit：第三轮为 `SUCCEEDED`，第四轮为 `FAILED/M4_SECRETARY_PROVIDER_FAILURE`。最终同一 RoleSession 下共 4 个 Turn，其中 3 succeeded、1 failed；UI 有 4 个 user node、3 个 assistant node，失败 turn 没有伪 assistant 内容。phase 2 正常 `exit_code=0`、`timed_out=false`、`signal=null`。
- 最终 RoleSession hash 为 `5d9b28674b9e5e0659e203f5abd0390bb402eaa03a04ed388a21fe8db9d2cbad`，history hash 为 `e931f28237d49bf66fc2fdaafe0d9ac4f1f5c5de379f2dcb003ff1c8c6b48624`，conversation hash 为 `5a0642621c4cf667f7d5127b430cfdd4a0ced1a513cae0bf513eedda3bd218d4`。receipt 不携带 raw message、raw client ref、raw PID、provider handle 或 session id，只携带 hash、计数与固定 code。

## 4. 持久化、零副作用与反例

driver 在每个 phase 的 renderer 行为前后分别以 `READ_ONLY` + `PRAGMA query_only=ON` 打开 M3、provider 与 M4 SQLite，共 6 个只读连接。独立审计在副本上确认三库 `integrity_check=ok`、foreign-key violation=0，并逐项复核 final DB 与 receipt：

- M3 最终 1 个 active RoleSession、1 个 verified handle/current binding/context、4 个 Turn（3 succeeded/1 failed）、4 个 START effect/readback/receipt；全部 handoff 写面为 0。
- provider 最终 1 个 session、4 个 transcript row（3 succeeded/1 failed），`start_session=1`、`continue_turn=4`、`poll=5`、`read_transcript=12`、`resume_readback=0`、`stop=0`；receipt 记录并由 driver 校验 M3/provider ordered turn/client/binding hashes，portable evidence 不另存 raw DTO。
- M4 model invocation、source-owner writeback request/receipt 与 coordination row 均为 0；17 个正式对象表的 2 条 canonical record fingerprint 在两个 phase 的 baseline/final 以及跨进程边界都不变，SHA-256=`d538f7fcf9428b9e5c751bbe8488c1ec3e9434bbc2fe1c7436660bf12719e5ca`。
- fresh ordinary profile 的 Workbench product root（`app-data/CodexGovernanceWorkbench`）下没有 `runtime-artifacts/workbench.sqlite`、`workflow-state/workflow-state.v0.json` 或 `runtime-artifacts/storage-mode.v1.json`；这不是伪造 `integrity=ok`。profile 外层用于 acceptance fixture 的 Workflow JSON 不属于这个 Workbench product root。driver 直接要求 product root 内三项持续 absent，并递归要求该 root 只有目录 `index-kernel`、`tasks` 及常规单链接文件 `index-kernel/codex-index.json`、`tasks/README.md`，label+bytes digest 在所有快照保持 `cd98bf5ee1245f0462dfed9e5e68b0b9c3541543980b491a64b483036d614274`。额外根目录、嵌套空目录或任一禁止 artifact 都会 fail closed。

聚焦反例还覆盖：same ref/different text conflict、same text/new ref 新 turn、同毫秒 turn 的 server-owned SQLite 插入顺序、2048 turn 容量前置拒绝、authority/session/binding drift、provider row/hash/ref/time tamper、PREPARED/REGISTERED/claimed/provider-terminal 多个 crash seam、provider session 交叉与 failure ordinal 隔离、0700/0600 权限及 symlink/hardlink 拒绝。所有恢复路径只做 authoritative readback，不把重启误当 fresh resend。

## 5. 验证

本包最终运行并通过：

- red baseline commit `7f9c6da717f0ec49c22fcd76327431fcfff0cb4e` 上的 conversation 探针为 RED；AppState runtime、M3 turn edge、transcript join、load/send commands、renderer clients、App send edge 与 enabled composer 等 marker 尚未齐备。
- `node scripts/run-m4-remediation-probes.mjs --only=conversation --expect=green`：13/13 static markers GREEN；六份冻结合同 SHA-256 exact。该探针只证明固定 marker，不替代行为证据。
- `RUSTFLAGS=-Awarnings cargo test --offline --lib m4r05_ --no-fail-fast`：21/21。
- `cargo test --offline --lib m3c04_ --no-fail-fast`：15/15。
- `RUSTFLAGS=-Awarnings cargo check --tests --offline`：exit 0。
- `pnpm run typecheck`：exit 0。
- `node scripts/run-offline-interaction-test.mjs`：exit 0，包含 M4R05 UI、driver 与 isolated-App runner 静态门。
- `node --check scripts/run-r4-isolated-app-preflight.mjs` 与 M4R05 runner static test：exit 0。
- `node scripts/run-r4-isolated-app-preflight.mjs --m4r05-ordinary-conversation`：runner target 的两次受控 child launch 为 `PASS`；phase 1 由 launcher 确认 SIGKILL，phase 2 正常退出。没有额外 GUI 重启。
- `git diff --check`：exit 0。

最终语义复审、absence-evidence 复审与 receipt/SQLite 独立审计均由 Terra Ultra 子代理执行。报告已把未封存 executable identity 明确降级为证据上限；产品行为、false-PASS 与最终报告复审为 0 blocker / 0 P1 / 0 P2。

## 6. 证据上限与下一入口

本包证明的是 synthetic fresh profile 中普通 product constructor、普通 command registry、真实 DOM composer、M3 lifecycle、persistent synthetic provider transcript、强退后的 terminal-history readback 与继续发送。M3/provider/M4 数据库直接支持 lifecycle、调用次数、正式对象零增量与 model/writeback 零值；launcher 记录 real model/provider/connector/network write/Codex message attempts 为 0，但这不是 OS 级抓包或系统全局网络审计。`environment_unchanged=true` 只表示 launcher 进程的环境边界前后未变。

本次严格序列是“首次进程先完成两轮消息，再 SIGKILL，随后同一 RoleSession 恢复并继续”。它不证明“首条消息发送前先退出/重启，随后仍能首次发送”。冻结 M3 的 fresh dispatch permit 只存在于创建进程；该前置重启反例会固定返回 `M4_SECRETARY_FRESH_SESSION_PERMIT_UNAVAILABLE`，并保持 provider/Turn 写入为 0。本包保留该 fail-closed 上限，没有持久化、重铸或绕过 fresh permit，也不把 post-message continuity 外推为所有 restart continuity。

actual root 没有保存被执行 App 的独立副本，composite 也没有记录 executable SHA/mtime/CDHash；因此 portable receipt 可以独立验证 phase/profile/PID/nonce/前序 receipt 与三库快照链，但不能脱离工作区单独证明执行 binary 的不可变身份。phase baseline 数据库也没有另存原始副本，baseline 事实以 phase receipt 中的只读快照及跨阶段 exact comparison 为证。两项都按证据上限保留，不升级成更强结论。

本包没有实现或验收 M4R06 五类旧读面或 M4R07 最终总验收，也没有进入真实资料、真实模型/provider、真实消息、账号/凭据/connector、远端、发布或 M5–M10。

下一唯一入口是 `M4R06`。本包完成后只激活该 leaf，不提前施工 M4R07。
