# M3C08 主线集成与验收报告

日期：2026-08-10<br>
状态：`COMPLETED / MAINLINE / STAGE-05 CLOSED`<br>
已关闭叶：`stage-05 / M3C08-integration-regression-closeout`

本报告已经整理 M3C01–M3C07 的可核对事实、退出矩阵、迁移 / 回切边界、隔离桌面证据和下游交接。主线程已提供 M1 frozen input、Rust 聚焦与完整主机权限 `--lib`、script disambiguation、resident-session 精确重跑、前端 / build 和 launcher 的最终回归证据；M3C08 内容提交为 `fa8e392`，阶段结论为 `COMPLETED / MAINLINE / STAGE-05 CLOSED`。与本次状态回写同批的终态控制提交执行并归档 M3C08 `done` 与 stage-05 `close-stage`；本报告不猜测该控制提交 hash，也不声称 push、merge 或 release。

## 1. 范围、来源与结论口径

| 项目 | 记录 |
| --- | --- |
| 审计 HEAD | `0a6507c8ce2dddee6a0f99b2d806c6e2bdf0b4a1`（M3C07 提交） |
| M3C08 内容提交 | `fa8e392`（`fix(m3): close integration regression [catch:none]`） |
| 本次读取范围 | `AGENTS.md`、stage-05、M3C08 leaf、授权、M3 总 / 阶段计划、current-state、task queue、M3C01–M3C07 archive leaf、M3C07 report、receipt 与提交记录 |
| 当前授权记录 | `USER-SYN-M3-AUTONOMOUS-STAGE-05-20260809`；真实 provider / 消息、真实项目 / 账号、凭据、connector、远端、部署、发布、merge、push 和 M4–M10 实现均不在范围 |
| 本报告的终态边界 | 回归命令与 P0/P1/P2 已记录；终态控制提交执行并归档 M3C08 `done` 与 stage-05 `close-stage`，不猜测该控制提交 hash，不声称 push、merge 或 release |

## 2. M3C01–M3C07 提交清单

| 叶 | 提交 | 已进入的具名范围 |
| --- | --- | --- |
| M3C01 | `29085cc` | 冻结 M3 RoleSession / Turn / Handoff 实施补充合同与迁移矩阵 |
| M3C02 | `17933ea` | existing conversation 的 server owner / scope spawn 前守卫 |
| M3C03 | `0769dc5` | RoleSession repository、schema 与 shadow import |
| M3C04 | `089d36e` | ConversationTransportPort 与 fake provider 重启语义 |
| M3C05 | `b953939` | 显式 Handoff 状态机与结果回源 |
| M3C06 | `3472262` | server-owned RoleSession read model 与前端 cache 退位 |
| M3C07 | `0a6507c` | 隔离桌面分层验收与 Agent / Jiaoban synthetic host |

该清单只记录已提交的具名范围，不把提交存在本身提升为新的主线回归、真实 provider、真实 Codex、发布或 M4/M5 实现结论。

## 3. 冻结输入与状态型计划回写

2026-08-10 的静态 SHA-256 核对结果如下；四项均与 M3C01 frozen inputs 相同：

| M1 冻结合同 | SHA-256 |
| --- | --- |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |

M3C01 同时冻结了 M3 阶段计划的历史输入快照 `9403851ece470c32bac5071e2613495a6f0e525214dbd6990a1cd2d28d1ce013`。它不是上述四份 M1 合同。现行 M3 计划在本次文档回写前已为 `d584cc19592095cbeb521b483319ab77b61ecc3276351220ef5b94c0c9dae25c`，并会因本次状态型回写再次变化；这不改写 M3C01 合同，也不得表述为“全部 frozen hash 一致”。

## 4. M3 退出矩阵

| 退出项 | 直接来源 | 当前结论 | 边界 |
| --- | --- | --- | --- |
| M1 冻结合同不变 | 本报告 §3 的静态 SHA-256 | 四项匹配 | 不将 M3 计划状态文件计入四项 |
| RoleSession / Turn / ProviderHandle / ConversationContext | M3C01–M3C03 提交、archive leaf 与 §5 回归 | 已实现主线范围 | 不扩大为真实 provider 结论 |
| owner / scope / Station 3b spawn 前拒绝 | M3C02 archive leaf | 已实现主线范围 | 不代表真实 provider 已调用 |
| transport、fake provider、重启幂等 | M3C04 archive leaf | 已实现主线范围 | 仅 fake provider 结论 |
| Handoff 状态、receipt、source-owner 回写 | M3C05 archive leaf | 已实现主线范围 | 不产生授权 |
| server read model 与 cache 退位 | M3C06 archive leaf | 已实现主线范围 | cache 只作 display fallback |
| Agent / Jiaoban 隔离桌面验收 | M3C07 report 与 6 份 launcher receipt | 已归档 synthetic evidence | 仅 acceptance-only host；截图不在仓库 |
| legacy 回切 | M3C01 rollback、M3C06 / M3C07 archive | 旧 UI / read fallback 保留 | M3C07 隔离 global invoke allowlist 会拒绝 legacy transport |
| 完整主线合同、Rust、schema、fake provider、前端和 non-test build 回归 | 主线程最终输出 | `COMPLETED / MAINLINE`；命令与初次红灯见 §5 | 不将 host-environment 结果升级为真实 provider 结论 |
| M3C08 内容提交 | `fa8e392` | 已提交（`fix(m3): close integration regression [catch:none]`） | 不推测终态控制提交 hash |
| 真实 provider、真实 Codex、真实项目 / 账号、connector、发布 | 授权与 M3C07 evidence boundary | `NOT_ENTERED` | 不可由 synthetic 结果升级 |
| M4 / M5 / M6+ 实现 | plan、stage、当前用户指令 | `PLANNED / NOT_ACTIVE` | 本报告不授予实现权 |
| stage-05 / M3C08 关闭 | 与本次状态回写同批的终态控制提交 | `COMPLETED / MAINLINE / STAGE-05 CLOSED`；执行并归档 `done` / `close-stage` | 本报告不执行 Git 或 Harness 控制操作 |

## 5. M3C08 主线命令与结果回填表

下表记录最终主线回归，并保留完整 `--lib` 受限 sandbox 首跑的红灯、修复和主机权限重跑，避免只记录最终绿灯。

| 需要覆盖的层 | 主线程实际命令 | 结果 / 退出码 | 证据位置 |
| --- | --- | --- | --- |
| M1 / M3 合同与 frozen input 校验 | `sha256sum docs/contracts/role-session-v1.md docs/contracts/handoff-v1.md docs/contracts/identity-scope-v1.md docs/contracts/event-audit-outbox-v1.md`<br>`git diff --exit-code 29085cc -- docs/contracts/role-session-v1.md docs/contracts/handoff-v1.md docs/contracts/identity-scope-v1.md docs/contracts/event-audit-outbox-v1.md` | 两条命令均 exit 0；四项 exact | 本报告 §3 |
| Rust 聚焦 M3：M3C07 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_` | exit 0；11 passed / 0 failed；Cargo 4.92s | 初次 runner |
| Rust 聚焦 M3：保留语义 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c0` | exit 0；123 passed / 0 failed；Cargo 40.29s | 初次 runner |
| Rust 完整库初次受限 sandbox | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib` | exit 101；1520 passed / 4 failed / 45 ignored | §5.1 保留红灯与原因 |
| scoped launcher 修复 | `node --check prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs`<br>`cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib acceptance_runtime_profile_launcher_`<br>`cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_` | 三条命令均 exit 0；focused 5/5、`m3c07_` 11/11 | 仅 `scripts/run-r4-isolated-app-preflight.mjs` |
| resident-session exact host rerun | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib supervisor_session_launcher::resident_session_tests::s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config -- --exact` | exit 0；1/1；3.27s | PID `lstart` sandbox 差异分类 |
| Rust 完整库最终主机权限环境 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib` | exit 0；1524 passed / 0 failed / 45 ignored；72.83s；141 条既有 warning | 最终 Rust 全库结论 |
| M3C07 compile boundary | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_ --no-run` | exit 0 | 主线程回归 |
| TypeScript typecheck（启动器纠偏后直接复跑） | `npm --prefix prototypes/productized-desktop-shell run typecheck` | exit 0 | 使用既有 ignored `node_modules` |
| 离线前端交互（启动器纠偏后直接复跑） | `npm --prefix prototypes/productized-desktop-shell run test:offline-interaction` | exit 0；runner 遍历 39 entrypoint；摘要 `offline interaction tests passed: 15` | 使用既有 ignored `node_modules` |
| 前端生产构建（启动器纠偏后直接复跑） | `npm --prefix prototypes/productized-desktop-shell run build` | exit 0；306 modules；built in 955ms；仅既有 Vite `>500k` chunk warning | 使用既有 ignored `node_modules` |
| isolated launcher syntax（启动器纠偏后直接复跑） | `node --check prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs` | exit 0 | 不包含 `npm ci --offline` |
| M3C08 内容提交与终态控制 | `fa8e392`；终态控制提交 | 内容提交已完成；终态控制提交执行并归档 `done` / `close-stage` | 不推测终态控制提交 hash；不声称 push、merge 或 release |

### 5.1 完整 `--lib` 初次红灯、修复与最终绿灯

- 3 个失败是 M3C07 launcher source-string 与既有 `acceptance_runtime_profile` helper 的 collision，不是 runtime 或 sandbox 行为失败。current leaf 按 stage 总范围精确扩充唯一脚本 `prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs`，以运行时等价源码消歧修复这 3 个静态冲突；`node --check`、launcher focused 5/5 与 `m3c07_` 11/11 均通过。可见 Terra 任务 `019fe81d-03df-7f83-b647-7aba1b3524e3` 是该修复的执行记录。
- 第 4 个失败是 exact resident-session test 在受限 sandbox 读取 PID `lstart` 的 EPERM 环境差异。主线程在主机权限环境重跑同一个精确测试，结果为 exit 0、1/1 通过、3.27s，符合既有 M2 evidence 分类。
- 修复与环境分类后，完整 `--lib` 在主机权限环境为 exit 0、1524 passed / 0 failed / 45 ignored、72.83s，保留 141 条既有 `unused` / `dead_code` / `private_interfaces` warning；无新增失败。初次 1520 / 4 / 45 仍是本报告的透明历史证据。

## 6. 已归档的 M3C07 分层命令结果（不是 M3C08 回填）

下列内容仅转录 `M3C07-isolated-desktop-layered-acceptance.md` 已报告的隔离结论，便于避免把旧证据误记为本次主线回归：

| 层 | 已归档命令 | 已归档结果 |
| --- | --- | --- |
| M3C07 单元 / 集成 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_` | 11 passed / 0 failed |
| M3 保留语义 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c0` | 123 passed / 0 failed |
| M3C07 编译边界 | `cargo test --locked --offline --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib m3c07_ --no-run` | exit 0 / 编译错误 0 |
| TypeScript | `npm run typecheck` | exit 0 |
| 离线前端交互 | `npm run test:offline-interaction` | exit 0；39 bundled test entrypoints；M3 acceptance suite 15 checks |
| 前端生产构建 | `npm run build` | exit 0；Vite 306 modules transformed |

该表不证明 M3C08 已重新执行这些命令，也不证明真实 provider / Codex / 项目 / 发布通过。

## 7. M3C07 receipt 清单与 SHA-256

仓库中只有以下 6 份脱敏 launcher JSON receipt；窗口截图只存在 Codex 主任务 `019fe53e-c4c2-7ab0-a965-0e231075df57` 的线程内。

| receipt | SHA-256 |
| --- | --- |
| `agent-terminal-launcher-receipt.json` | `8e42340b8af9113dcd749c1b08b950cc12d0d3d1cf74b8f0bb543daa962752a5` |
| `jiaoban-launcher-receipt.json` | `6ff0fcfc68319a3704dd6d3e0f93a4490c1ea9764ce2f0a383d98cb894fa15bb` |
| `create-pending-launcher-receipt.json` | `8c10eb2e94bc9fbbf275bf34cf36d1b39a3c8bb654e33d86dd98273cbcbf2649` |
| `start-pending-launcher-receipt.json` | `fdd44336765cbc3e6556c581e993c48696ac538c5ff2f5dfde928137a2aaf53b` |
| `stop-pending-launcher-receipt.json` | `280d9b5649637fe478c0eddf9ea0a42d7506a4cb33bc1182ef08053cbbb82efa` |
| `normal-mode-launcher-receipt.json` | `c30867b9fa7043f495a5436b22b4d4a4ef99904324e59838ec3e1002c57121f8` |

## 8. 迁移、回切与未进入边界

- M3 的迁移输入仅为 Codex SQLite / rollout index、durable supervisor binding 与 valid continuation 的受限 shadow 分类；frontend cache、raw transcript 和不能精确归属的 thread 不得升格为真源。
- server-owned read model 为恢复入口；Agent / Jiaoban cache 仅作兼容显示。回切只可切旧 UI / read fallback 或关闭新 projection，并保留 provenance、receipt、export / manifest。
- M1 thread-owner、scope 与 Station 3b guard 必须留存。回切不得重放 effect、恢复跨项目 bypass、把 cache 提升为 owner，或删除 unresolved orphan。
- M3C07 acceptance-only child 使用 global `invoke_handler` allowlist 拦截 legacy Agent / Jiaoban transport；该事实只说明隔离 child 的 fail-closed 范围，不是普通产品模式的物理退役。
- 未进入：真实 provider、真实 Codex 消息、真实项目和用户数据、真实账号、凭据、外部 connector、网络、部署、发布、merge、push、真实数据迁移和完整产品桌面覆盖。

## 9. P0 / P1 / P2

| 级别 | M3C08 主线结论 | 已归档 M3C07 结论 | 处理 |
| --- | --- | --- | --- |
| P0 | 0 | 0 | 无 |
| P1 | 0 | 0 | 无 |
| P2 | acceptance-only panel 在无效 stage / restart 命令后会暂时以错误文本遮住状态；刷新 readback 后恢复 | 同一已知 acceptance-only P2 | 不影响后端或普通产品模式 |

## 10. 主线程回填清单与停止条件

主线回归与 P0/P1/P2 已填实，M3C08 内容提交为 `fa8e392`。终态控制提交执行并归档 M3C08 `done`、stage-05 `close-stage`；该控制提交 hash 不在本报告猜测。本报告不执行 Git 或 Harness 控制操作，也不声称 push、merge 或 release。

M3 / stage-05 已收口，当前没有活动工程任务。M4/M5/M6+ 不激活；后续实现需要新的明确用户指令、匹配的新 stage、唯一 leaf 和授权。
