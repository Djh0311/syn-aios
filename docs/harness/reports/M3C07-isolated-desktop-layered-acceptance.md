# M3C07 隔离桌面分层验收报告

日期：2026-08-10

结论：代码、仓储/transport/Handoff、固定 host command、前端离线交互、启动器静态契约、生产前端构建和真实隔离桌面 Computer Use 观察均通过。五个独立 M3 profile 完成 Agent/交办、CREATE/START/STOP/terminal 强退重启、Handoff、审计回滚和对象导航 fail-closed；另一个不带 M3 mode 的普通隔离 profile 证明验收控件默认隐藏。P0/P1 均为 0；本报告没有把 R4/M2 或静态证据写成 M3 桌面观察证据。

## 实现范围

- `SYN_R4_ACCEPTANCE_PROFILE` 仅提供全应用文件系统隔离基座。M3 runtime 另要求 debug build、有效 profile 与 `SYN_M3C07_ISOLATED_ACCEPTANCE=1` 三重 gate；普通启动、release、缺 profile 或缺 mode 都保持 `M3_BINDING_UNAVAILABLE`。
- M3C07 固定 Agent/交办 host command 只接收 action 与 nonce；renderer、cache、thread、selector、sandbox、write-root 和 profile hint 都不是 authority。M3C07 runtime 安装后，Tauri 全局 `invoke_handler` 仅放行四个 fixed acceptance IPC 及审计证明必需的纯读 IPC；所有 legacy Agent/交办 transport、manual relay、automation/workspace-write sibling IPC 都在命令 wrapper、binding、provider 与 effect 之前以 `m3c07_isolated_acceptance_legacy_ipc_blocked` 拒绝。普通模式仍使用完整 registry。
- M2 reference-slice 与 M3C07 的 mode argument 已互斥；同时，M3 runtime 在 direct child 携带任一 M2 marker family 时以 `m3c07_m2_reference_slice_mode_conflict` fail-closed，避免同一 child 混用两类证据边界。launcher 还会在创建 root、scrub inherited env、build 或 child spawn 之前归一化父环境的完整 M2 marker family：M3 CLI + 任一 M2 marker 以该固定码非零退出，M2 CLI + inherited M3 marker 以 `m2_reference_slice_m3c07_mode_conflict` 非零退出；两 CLI flag 仍保留既有 `mode_argument` 语义，普通 R4、纯 M2、纯 M3 不受影响。该前置冲突路径没有 fixture、build、child 或 receipt side effect。
- fake provider 使用隔离 profile 下的持久 SQLite ledger；真实 provider、真实 Codex 消息、账号、凭据、connector 和网络路径未接入。运行时 action receipt 与 launcher receipt 都显式给出 `real_provider_attempts: 0`，但 launcher 的该数值仅描述启动器自身，不能代替运行时 ledger。
- New/Continue/Stop 调用 M3C03 repository 与 M3C04 `M3RepositoryBackedConversationTransport`；Handoff 调用 M3C05 的 Create/Accept/RequestReturn/RecordReturnResult/SourceApplication 真状态机，不用平行 ledger 状态机冒充业务真源。
- `stage_create_pending`、`stage_start_pending`、`stage_stop_pending` 是固定 acceptance-only durable checkpoints。重新打开同一 repository/ledger 后只走 restart inventory/recovery readback，绝不重新 dispatch。
- Handoff 验证完成 `CREATED → ACCEPTED → RETURN_PENDING → RETURNED → APPLIED`，并在 reopen 后重放 source-application command，核对 receipt/fingerprint/hash/state digest、transition integrity 与 8 张 Handoff 因果表（含 source-command fence、source-validation proof）行数不变。
- host receipt readback 是服务端固定 host 的全表倒序读取；不会先做全局 `LIMIT 32` 再过滤，因而一个 host 的 32+ receipt 不会挤掉另一 host 的最近 receipt。
- 审计 late-failure 使用 test-only SQLite trigger 让 audit insert 失败，验证 claim rollback 后 effect 仍为 `REGISTERED`、无 provider attempt，随后清理 trigger、检查 FK 并 reopen。
- Agent 与交办均挂载同一隔离验收 panel。普通启动的初始未知态及 `M3_BINDING_UNAVAILABLE` 都返回空，不闪现验收控件；对象导航仍为 `OBJECT_NAVIGATION_ABSENT` 的可见 fail-closed 证据，未伪造成功。

## 启动器模式

`scripts/run-r4-isolated-app-preflight.mjs --m3c07-isolated-acceptance`：

- 构建产物一次，生成一个合成 R4 profile/root/reentry capability，并只在 M3 child 环境中注入 `SYN_M3C07_ISOLATED_ACCEPTANCE=1`。
- 每次 child 启动输出 `syn_m3c07_ui_inspection_ready.v1`，含 `run_hash`、`syn_pid`、`launch_index`、`ui_inspection_path`、M3 receipt 路径和 profile SHA-256。
- 非最终 `SIGKILL`/`SIGTERM` 后，最多 8 次以同一 profile、reentry capability 和 build 产物 relaunch；每次 launch disposition 写入独立 `m3c07-isolated-readiness-receipt.json`。
- 仅当外部 Computer Use 在最终观察后写入已完成的 `ui-inspection.json`，loop 才结束；receipt 记录 `same_profile`/`same_profile_reused`、每次 disposition 和 `real_provider_attempts: 0`。M2 reference slice 与普通 R4 receipt 分支保持独立且互斥。

## 已验证证据

| 层 | 命令 | 结果 |
| --- | --- | --- |
| M3C07 单元/集成 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_` | 11 passed / 0 failed |
| M3 保留语义 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml m3c0` | 123 passed / 0 failed |
| M3C07 编译边界 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml m3c07_ --no-run` | exit 0 / 编译错误 0 |
| 启动器静态 | `node --check scripts/run-r4-isolated-app-preflight.mjs`；`m3c07_launcher_keeps_an_explicit_mode_same_profile_relaunch_and_separate_receipt`；离线 launcher mode-policy test | 均 exit 0；纯 policy 覆盖完整 M2 marker family、双向 inherited mode conflict、纯模式允许及 root/scrub/build/spawn 前短路 |
| TypeScript | `npm run typecheck` | exit 0 |
| 离线前端交互 | `npm run test:offline-interaction` | exit 0；39 个 bundled test entrypoints 全部执行，M3 acceptance suite 15 checks |
| 前端生产构建 | `npm run build` | exit 0；Vite 306 modules transformed |

Rust 命令保留仓库既有 warning（M3C07 相关 test-only import 已限定到 `#[cfg(test)]`）；没有编译错误。此前一次 `npm ci --offline` 因本机离线缓存缺少 `zustand` 失败，未联网；随后使用主线程保全的既有、被忽略 baseline `node_modules` 完成上述 TypeScript/前端验证，未改 package 或 lock。

## 真实隔离桌面观察（已完成）

所有窗口操作均由 Computer Use 锁定到本仓库构建的完整 App 路径；同 bundle identifier 的旧工作区 App 未作为目标。每次只强退 readiness 给出的 synthetic child PID，launcher 保持运行并以原 profile/reentry/build 自动拉起下一进程。窗口截图定位为 Codex 主任务 `019fe53e-c4c2-7ab0-a965-0e231075df57` 的线程内 Computer Use 证据；脱敏 launcher JSON receipt 则是仓库持久证据，位于 `docs/harness/reports/M3C07-isolated-desktop-evidence/`。

| 场景 | run / profile SHA-256 | 窗口操作与运行时证据 | launcher 结论 |
| --- | --- | --- | --- |
| Agent 生命周期、Handoff、回滚、terminal | `aa8b21a3…` / `3611a3eb…` | New/Continue/Stop 为 `1/1 → 2/2 → 3/3`；audit rollback 保持 `3/3` 且 `M3_AUDIT_WRITE_FAILURE_ROLLBACK_VERIFIED`；Handoff 为 `4/4` 且 `M3_HANDOFF_CREATED_ACCEPTED_RETURNED_APPLIED_EXACT_REPLAY_VERIFIED`；对象导航为 `OBJECT_NAVIGATION_ABSENT`；terminal 重启 readback 保持 `4/4` | 2 launches；第 0 次 `SIGKILL`，第 1 次完成观察后 `SIGTERM`；`same_profile_reused=true`，exit 0 |
| 修复后交办固定 host | `6e9c99f9…` / `316ea4e7…` | synthetic 非测试项目的早退分支实际显示“隔离桌面验收 · 交办”；Jiaoban New/Continue/Stop 为 `1/1 → 2/2 → 3/3`；terminal 重启 readback 保持 `3/3` | 3 launches；前两次 `SIGKILL`，最终 `SIGTERM`；三次 profile hash 相同，exit 0 |
| CREATE pending | `19c72f6b…` / `070868c1…` | 强退前 `stage_create_pending` 为 dispatch/readback `1/0`；重启后 `M3_RESTART_CREATE_READBACK_APPLIED`，计数 `1/1`，无重复 dispatch | 2 launches；`same_profile_reused=true`，exit 0 |
| START pending | `da7ceab8…` / `0ce3fe8b…` | New 后落 START：强退前 `2/1`、回合 `STARTING`；重启后 `M3_RESTART_TURN_READBACK_APPLIED`、`2/2`、回合 `SUCCEEDED` | 2 launches；`same_profile_reused=true`，exit 0 |
| STOP pending | `d21f6ff9…` / `7e55c3f4…` | New/Continue 后落 STOP：强退前 `3/2`、回合 `ACTIVE`；重启后 `M3_RESTART_TURN_READBACK_APPLIED`、`3/3`、回合 `CANCELLED` | 2 launches；`same_profile_reused=true`，exit 0 |
| 普通隔离模式负例 | `6271709f…` / 非 M3 receipt | Home 首帧、Agent 页和交办页均没有 `M3C07` 或“隔离桌面验收”控件 | 普通 R4 receipt；完成观察后 `SIGTERM`，exit 0 |

上述五份 M3 receipt 均为 `syn_m3c07_isolated_desktop_launcher_receipt.v1`，均记录 `real_provider_attempts=0`、`ui_inspection_completed=true`、`environment_unchanged=true` 和 parent/child/process-group/session 关系成立。launcher 的 `runtime_action_evidence=not_observed_by_launcher` 是刻意的证据分层：launcher 只证明 gate/profile/process；本节的 Computer Use 窗口读取和运行时 panel receipt 证明 action/ledger，二者不互相冒充。

持久 receipt SHA-256：

- `agent-terminal-launcher-receipt.json`：`8e42340b8af9113dcd749c1b08b950cc12d0d3d1cf74b8f0bb543daa962752a5`
- `jiaoban-launcher-receipt.json`：`6ff0fcfc68319a3704dd6d3e0f93a4490c1ea9764ce2f0a383d98cb894fa15bb`
- `create-pending-launcher-receipt.json`：`8c10eb2e94bc9fbbf275bf34cf36d1b39a3c8bb654e33d86dd98273cbcbf2649`
- `start-pending-launcher-receipt.json`：`fdd44336765cbc3e6556c581e993c48696ac538c5ff2f5dfde928137a2aaf53b`
- `stop-pending-launcher-receipt.json`：`280d9b5649637fe478c0eddf9ea0a42d7506a4cb33bc1182ef08053cbbb82efa`
- `normal-mode-launcher-receipt.json`：`c30867b9fa7043f495a5436b22b4d4a4ef99904324e59838ec3e1002c57121f8`

## 未关闭项

- P0：0。
- P1：0。真实隔离桌面窗口、三处 pending、terminal、Agent/交办和最终 UI inspection 均已执行。
- 覆盖范围仅为 Agent/交办的 acceptance-only panel；不把其他产品页面或 M2/R4 观察归为 M3C07 desktop evidence。
- 已知能力边界：object navigation 仍是 `OBJECT_NAVIGATION_ABSENT`，panel 明确 fail-closed 展示，未声称导航成功。
- acceptance-only P2：对不满足前置条件的 stage/restart 操作，panel 暂时显示错误文本；点击“刷新 readback”可恢复完整状态。该行为未进入普通生产 UI，不影响后端 fail-closed 或持久证据。
