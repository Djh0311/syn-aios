# L3 Syn R4 isolated runtime profile preflight verification v1

- 日期：2026-07-24
- 任务包：`tasks/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-package-v1.md`
- 状态：**PENDING_AUTHORIZED_I5_HOME_ONLY_UI_DISCOVERY**
- 更正：撤回历史 `BLOCKED_I5_MACOS_LOCKED_UI_INSPECTION`。旧运行只证明 UI state read 未在 Home assertion 前完成；工具返回的 “Mac is locked” 不能证明宿主 macOS 锁屏。指导线随后只读实测 Finder accessibility tree 与 screenshot 均可用，故不再沿用该归因。
- 已修根因：旧 launcher 在最终 Syn 启动前把 `ui-inspection.json` 写入隔离根；Rust `validate_root_layout` 只允许 `profile.json`、`fixture`、`workflow-state`、`app-data`、`codex-db`、`logs`，固定拒绝为 `acceptance_runtime_profile_reused`；`run()` 早退后又被 launcher 的 exit-0 分支误表述为 normal exit。现 root allowlist 未放宽：launcher 只将外部 observation 投影到空 `logs/ui-inspection.json`，且不预创建；profile/AppState 初始化失败固定映射为非零 78/79，exit 0 也只有在完成外部 UI observation 后才可称为正常结束。
- 停点：修复后获准的一次 Home-only isolated Syn 启动完成 build；首个且唯一 UI 操作为 `sky.list_apps()`，它返回的 Syn bundle 为非运行态，故没有可安全读取的本次 UI target。未 attach、未读 Home、未截图、未写 observation；本线只终止了该次 launcher 明示的自建 PID。未发送主管首句、未启动 Codex CLI/MCP server、未调用 `tools/list` 或任何工具，也未进入十二项。
- 本次指导验收返工仅加固 prelaunch 根目录的跨语言等价合同，不改变上述运行结论：旧测试只逐项检查 launcher 源码是否 `contains` 六个名称，因此内存中把第七项 `unexpected-extra` 加入 `PRELAUNCH_ROOT_ENTRY_NAMES` 后仍会通过。现 Rust `validate_root_layout` 实际消费单一 `PREPARED_ROOT_ENTRY_NAMES` 六项常量；测试严格解析 Node 的 `PRELAUNCH_ROOT_ENTRY_NAMES` 与 Node 自身 `PROFILE_FILE_NAME` 字面量，拒绝重复、数量不等或集合不等。内存变异覆盖第七项、Node 六项逐个删除/改名，以及 Rust 六项逐个删除/改名；均必须被拒绝。
- 新的 discovery-only 专项静态发现旧路径使用 `--no-bundle` 后直接启动裸 Mach-O，而可被 Sky 按 bundle identity 枚举的 `.app` 可能是旧产物。最小修复改为用临时 CLI config 启用 `--bundles app`，验证 bundle 内 executable 为本次 build 产物后直接启动它；普通 build 不注入 profile，最终进程仍只注入 fresh isolated profile。新增合同拒绝 `--no-bundle`、裸 binary、bundle 名/identifier 漂移与缺少 ready identity，root/logs、78/79、exit-0 语义均未改。
- 唯一 discovery-only 启动已消费：fresh bundle build 与 ready envelope 成功，但在任何 `sky.list_apps()` 调用前，receipt 报 final Syn 子进程 `SIGKILL`、`ui_observation_missing` 与 `failure_stage=ui_inspection`。这只能证明 target 未保持到可列举时刻；信号来源未知，不能把它归为 launcher 自杀、macOS 锁屏或 Sky 发现失败。未调用 Computer Use、未 attach、未读取 Home、未截图、未写 observation、未重试；已知自建 PID 事后 `kill -0` 确认不存在。本轮原件已从保留的隔离根恢复为 [launcher-receipt-v4.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v4.json)，并与原件逐字节一致，SHA-256 为 `a56bc4879b5f124ba5c3e8cf7599a666cd3110fddcaae326daa846a86a161e6b`；它不含绝对路径、完整命令、环境值、PID 或 raw stdout/stderr。
- 最新一次、独立授权的 pre-list 诊断启动已消费：新增 launcher lifecycle ledger 后，fresh isolated build 成功、ready envelope 出现后 final child 仍在首次 UI 枚举前以 `SIGKILL` 结束。新的 [launcher-receipt-v5.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v5.json) 逐字节对应本次保留隔离根 receipt（SHA-256 `10ba897b7c452f8739b829fb9a10561cb93dbac040871cdc9e94461133bb3092`）：launcher 没有尝试 kill child，launcher 也未 self-reraise，parent 未收到 `SIGTERM`/`SIGINT`/`SIGHUP`，child 的 `exit` 与 `close` 都观察到 `SIGKILL`；PGID/session 投影为 `unavailable`，receipt 没有记录其是 child 提前消失、`ps` 失败还是其他 query 失败。仅对该 child 与 receipt mtime 前后 70 秒做 process-scoped macOS fault/error 元数据投影，得到 `matching_record_present`，见 [process-scoped-termination-metadata-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/process-scoped-termination-metadata-v1.json)（SHA-256 `0ba1b0cc32ecdc579e32444885b53af8bb3582dea8ee58d8c8e6b818c9163340`）；原始日志未保存，匹配记录也不能识别信号发送者。child 已不存在，故未调用 `sky.list_apps()` 或任何 Computer Use；停点不变。
- 用户手动运行证明普通 `cargo-tauri dev` 链可保持 Syn 存活后，另行授权一次只增加 isolated profile 的对照。fresh synthetic root 启动前严格为六项且 `logs/` 为空；同一 `.cargo-target` dev binary 在 profile 注入后进入运行态，storage-mode 缺失路径指向本轮 isolated root，Syn、cargo-tauri 与 Vite 连续存活至少 60 秒。随后仅以 Ctrl-C 结束本线，会后这三类进程及 5173 监听均不存在。未调用 Computer Use、未读 UI、未截图、未进入 Gate 0。脱敏原件为 [manual-dev-isolated-profile-control-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/manual-dev-isolated-profile-control-v1.json)（SHA-256 `1c04367637620e1c8257f6d530b8fc38c4f2bf0d353daf7d05b26cce5b16d869`）。这排除了 isolated profile 是 SIGKILL 的充分条件，但还不能在 binary、bundle、direct spawn 与 cargo-tauri process context 之间唯一归因。
- 后续离线 bundle-integrity 返工先以 0/1 红合同抓到 launcher 未建立 resource seal，再只改 launcher 与其合同测试：fresh `.app` 在最终 Syn 启动前必须经固定 `/usr/bin/codesign --force --deep --sign -` ad-hoc seal，并通过 `/usr/bin/codesign --verify --deep --strict`；禁止 `--ignore-resources`，seal/verify 的启动失败、非零退出或信号均 fail closed 为 `failure_stage=bundle_integrity`。修复后聚焦合同 1/0、完整 profile 套件 19/0。离线 fresh build 先复现原 strict verify 失败，执行与 launcher 相同的 seal 后，`.app` 与真实 executable 均 strict verify 通过且 `Contents/_CodeSignature/CodeResources` 存在。该离线返工当时未启动 App，故当时没有宣称旧 `SIGKILL` 已修复。
- 用户新授权的第一轮 fresh-bundle pre-list 重验已消费：sealed fresh bundle 产生 ready envelope 后，本轮 Syn 连续存活至少 88 秒，超过旧失败观察窗口且未观察到 `SIGKILL`。按授权边界没有调用 Computer Use 或 UI；随后只向精确自建 PID 发 SIGTERM。新的 [launcher-receipt-v6.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v6.json) 与隔离根原件逐字节一致（SHA-256 `2f3f3096e9ecc279ef053dd3a90400a4b8692ffb548fbb3303e4aaa7940a9fd7`），记录 build 0、child exit/close SIGTERM、launcher 未 kill child、parent 未收到 `SIGTERM`/`SIGINT`/`SIGHUP`，以及父子/process-group/session 关系均确认。脱敏观察投影为 [fresh-bundle-pre-list-revalidation-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/fresh-bundle-pre-list-revalidation-v1.json)（SHA-256 `bde4bd8f13cb7e188b8f800c5a8504a329ff524397c868c5c433d189eefcf610`）。本线 Syn/cargo-tauri/Vite/5173 均无残留。该单次结果只证明旧失败本轮未复现，不证明所有运行均已修复。
- 上述状态在本指导对话执行前已写入 CURRENT，但本对话只复核 launcher/进程而未先重读 CURRENT，因而重复消费一次同范围 pre-list 重验。第二轮 sealed fresh bundle ready 后连续存活至少 90 秒，同样没有观察到 `SIGKILL`；全程仍未调用 UI，随后以 Ctrl-C 结束。新的 [launcher-receipt-v7.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v7.json) 与本轮隔离根原件逐字节一致（SHA-256 `efab175b4a946427109b42cbfad29c86d78bbf47607bd27eb08cfd8cc62223ed`），记录 build 0、child exit/close SIGINT、launcher 未主动 kill、parent 收到并在 receipt 后重发 SIGINT，以及父子/process-group/session 关系均确认。脱敏观察投影为 [fresh-bundle-pre-list-revalidation-v2.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/fresh-bundle-pre-list-revalidation-v2.json)（SHA-256 `03e7112247b6cb748ce87df69b05eb3184ecc24377e11b5adcbad7808a7602ef`）。收尾无本线进程/5173 残留；该重复运行不扩大结论强度，当前停点不变。

## 1. I0 冻结与实际启动数据流

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；开工暂存区为空，既有大量 porcelain 脏改保持原样。
- 开工 porcelain 指纹：`afe87dd98fd36199c7f7fc229bd134ae1a344adaf48b46c1678325ddd48918d4`。
- 冻结承重 SHA（开工时）：`lib.rs=828667f3…a9abf3ff`、`index_host_app_entrypoints.rs=5ae1f355…68182a8e`、`codex_db.rs=12332191…b8b5d09d`、`knowledge_vault.rs=95d524cf…742230dc`、`mcp/storage.rs=7024aad7…ce0710b3`、`package.json=ac0211ea…62c8ca49`。
- `run()` 现先执行 `acceptance_runtime_profile::initialize_from_env()`，再 `AppState::try_new()`；两者均先于 migration、两类 reaper、SQLite startup mode 和 `tauri::Builder`。profile 失败会在这些动作和开窗前返回。
- profile 激活时，`AppState` 的 index/tasks/workflow、snapshot session mode、Codex DB、app-data/vault/recovery、Canvas 和 debug logs 均由同一 immutable `RuntimePaths` 派生；session 固定为 `IndexOnly`。正常环境变量缺席时仍走既有默认路径。
- isolated profile 下不启动 knowledge-open relay listener，因此本包没有 supervisor/MCP 触发面。

## 2. I1–I3 红绿合同与 fixture

- 历史 profile 定向 Rust 合同曾为 **15 passed / 0 failed**。它覆盖 canonical temp 直接子根、0700/owner/symlink/hardlink、profile-only/reused fixture、严格 schema/额外字段、expiry、固定路径、IndexOnly、normal/release 行为、getter 未初始化 fail-closed、启动顺序与全部路径 consumer 接线，并把 receipt 语义限定为 Home 初始视图配置已固定、fixture 声明路径投影及 UI 观察状态，禁止表述为 UI 或 runtime 已验。prelaunch 集合合同将其增至 16；本轮 fresh-bundle discovery 合同再增至 §3 的 17。两者都不是 token/禁词表，而是对实际 Rust allowlist 或实际 Tauri 配置与 launcher 声明的精确比较。
- receipt 红测先以 **14 passed / 1 failed** 失败，原因是旧 launcher 仍含 `home_initial_view_verified`。随后改为 v2 receipt，绿测为 **15 passed / 0 failed**；旧 `resolved_root_containment` 也已替换为带 provenance 的 `declared_fixture_path_containment`。
- dotenv 风险的红合同先按预期失败：launcher 未在 Vite build 前 pin Home；修复后 `normalBuildEnvironment.VITE_STAGE_K_INITIAL_VIEW = "home"`，静态合同转绿。`VITE_STAGE_K_INITIAL_VIEW=knowledge node scripts/run-r4-isolated-app-preflight.mjs` 以 exit 1 在 fixture/build/App 前停止且无输出。
- fixture 只生成一个 root 内 synthetic project 和 workflow，ID 从 canonical synthetic project root 使用既有 stable-ID 语义生成；index 只有该 project，threads/sessions 为空，tasks 为空，workflow state 仅有该 project/workflow，app-data/vault/canvas 初始为空。没有从真实 index/tasks/Codex DB/vault 复制内容。

## 3. I4 离线门（本轮精确返工实际读数）

| 检查 | 实际结果 |
| --- | --- |
| `cargo test acceptance_runtime_profile_prelaunch_layout_and_exit_contract_fail_closed --lib --quiet` | 红灯依次为 0 passed / 1 failed：缺少 Rust 单一 allowlist 常量；第七项 Node 变异报 `launcher=7 Rust=6`；Node `PROFILE_FILE_NAME` 改名曾被旧解析器漏过。修复后最终为 1 passed / 0 failed，并在一个测试内机械拒绝第七项、Node 六项逐个删除/改名及 Rust 六项逐个删除/改名 |
| `cargo test acceptance_runtime_profile_launcher_builds_a_fresh_app_bundle_for_sky_target_discovery --lib --quiet` | 红灯为 0 passed / 1 failed（旧 launcher 缺少 fresh bundle 声明）；修复后 1 passed / 0 failed，五个内存变异均被拒绝 |
| `cargo test acceptance_runtime_profile_launcher_records_pre_list_sigkill_diagnostics_fail_closed --lib --quiet` | 新红合同先为 0 passed / 1 failed（旧 launcher 缺少固定诊断 schema）；实现后为 1 passed / 0 failed。内存变异拒绝任一 parent signal 漂移、额外 signal、缺少 `close`、缺少 `sess` 投影、主动 `child.kill` 与 raw PID 字段 |
| `cargo test acceptance_runtime_profile_launcher_seals_and_strictly_verifies_fresh_bundle_before_launch --lib --quiet` | 红灯为 0 passed / 1 failed（launcher 缺少固定 `CODESIGN_PATH`）；修复后最终为 1 passed / 0 failed。内存变异拒绝 `--ignore-resources`、绕过 seal/verify、PATH 解析 codesign，以及 seal/verify 非零仍放行 |
| `cargo test acceptance_runtime_profile --lib --quiet` | 历史 receipt 红绿为 14/1 → 15/0；prelaunch 合同后为 16/0；fresh bundle 后为 17/0；诊断 wrapper 后为 18/0；bundle integrity 合同后最终为 **19 passed / 0 failed / 1226 filtered** |
| `cargo check --lib` | exit 0；工作树既有 598 warnings |
| `npm run typecheck` | exit 0 |
| launcher syntax | `node --check` exit 0 |
| fresh `.app` 离线 bundle integrity | fresh build 后先按预期 strict verify 失败；执行与 launcher 相同的 ad-hoc seal 后，`.app` deep/strict verify、真实 executable strict verify 与 `CodeResources` 存在性检查均 exit 0 |
| `git diff --check` | exit 0 |
| staged | `git diff --cached --name-only` 无输出 |

目标 rustfmt 检查按任务包完整执行。`acceptance_runtime_profile.rs`、`acceptance_runtime_profile_tests.rs`、`lib.rs` 和 `index_host_app_entrypoints.rs` 为绿；完整目标集合仍在既有脏改的 `codex_db.rs`、`knowledge_vault.rs`、`mcp/storage.rs` 报格式差异，差异均不在本包 profile 接线 hunk。为保护共享脏树，未对这些历史 hunk 作格式化。该历史格式债按任务包 §7 单列，不被表述为本轮全绿。

shape baseline 为 **17 errors / 5 warnings / 5 info**；check 同为 **17/5/5** 且以历史债退出 1。没有声称绝对零净增。本次只重跑用户指定的 profile/production/typecheck/formatter/diff/staged 门，不把未重跑的其余 15 组离线 runner 表述为本次精确返工的绿灯。

## 4. I5 单次 isolated smoke（修复后实际结果）

1. 离线门后只执行 `npm run r4:isolated-preflight` 一次；launcher 先以未注入 profile 的普通环境 build，随后只向最终 Syn 进程注入 profile。该一次性启动已消费，未重试。
2. final ready envelope 后，首个且唯一 Computer Use 调用为 `sky.list_apps()`。它返回一个精确 Syn bundle 标识，但状态为 `isRunning=false`；这不是 macOS 锁屏结论，也不满足“本次已启动 UI target 可读”的条件。为避免以该标识触发默认 profile 的后台启动，没有调用 `get_app_state`、attach、click、输入、页面切换或任何其他 UI 操作。
3. 本线用 launcher 明示的单一自建 PID 发送 SIGTERM，随后得到 [launcher-receipt-v3.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v3.json)：build `exit_code=0`，Syn `signal=SIGTERM`；因为没有完成外部 UI observation，receipt 如实为 `ui_inspection_attempted=false`、`ui_inspection_completed=false`、`synthetic_home_verified=false`、`screenshot_saved=false`，并以 `failure_stage=ui_inspection` fail closed。`syn_exit_disposition=unexpected_exit` 只是“未完成 observation 后被按合同终止”的受控结果，绝不再称 normal exit。
4. observation 路径在启动前保持不存在，`logs/` 也经 prelaunch 合同验证为空；故没有违反“外部观察器仅于 profile 初始化、发现 target 后创建”的限制。没有截图、没有 synthetic-only Home 断言、没有 UI 级 fixture manifest 主张。
5. receipt 的十项 `declared_fixture_path_containment=true` 仍仅是 launcher 对 fixture/declared paths 的投影，provenance 固定为 `launcher_declared_fixture_path_projection`；它不能证明 App runtime 或 Home UI。`fixture_synthetic_identity_hash`、`profile_declared_session_source=IndexOnly` 与 `home_initial_view_config_pinned=true` 同样只表示启动配置/fixture 已固定。

没有访问或呈现真实 store/vault/project/workflow 内容，也不对全系统进程零残留作主张；只确认本线已知 PID 收到 SIGTERM 且 launcher 会话已退出。

## 4.1 I5 discovery-only 专项（本轮实际结果）

1. 只在上述离线门全绿后执行一次隔离 launcher。新 launcher 使用本次 build 生成的 configured `.app` 内 executable，普通 build 环境不含 profile，最终子进程才获得 fresh isolated profile。
2. ready envelope 已产生，但 launcher 返回前 final Syn 子进程被报告为 `SIGKILL`。恢复的 [launcher-receipt-v4.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v4.json) 逐字节对应本次保留隔离根，记录 build `exit_code=0`、Syn `launched=true/signal=SIGKILL`、`syn_exit_disposition=unexpected_exit`、`ui_observation_missing` 与 `failure_stage=ui_inspection`。之后的 `ui_inspection` 是结果分类，不是信号来源证明。
3. 因目标不再运行，未调用 `sky.list_apps()`；调用它只会得出事后非运行态，不能回答“本次正在运行 target 是否可发现”。没有任何其他 Computer Use、UI、CLI/MCP、工具、Gate 0 或十二项动作。
4. launcher 会话退出；仅对其 ready envelope 所报的自建 PID 做 `kill -0`，确认进程已不存在。没有声称全系统零残留，也没有保存 raw stdout/stderr、命令、环境、真实路径或窗口内容。
5. 既有 terminal executor transcript 的可证事实是：两次各 30 秒的轮询仍返回活跃会话，下一次轮询在约 5.5 秒后返回 launcher `exit_code=1`；没有记录任何 executor `timeout` 标志或外层终止原因。当前 Codex app 也没有附着的 terminal session 可供补取。因此只能确认 launcher 命令以非零结束；无法从现有材料判断 executor 是否曾超时、谁终止了 child，或该外层会话与 `SIGKILL` 的因果关系。

## 4.2 I5 pre-list SIGKILL 一次诊断（最新实际结果）

1. 运行前冻结 HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、porcelain 指纹 `cba82024a806126817e692f56d07bdc1daed09b2bce07299220eee62062a6387`、暂存区为空；最终 launcher SHA-256 为 `f07bd2e3cda26d7c71a537ecc231e41254637e2784df7dacb9c9fe91c5df5c0e`，profile/entrypoint/Tauri config 均未因本诊断改变。
2. 红合同先以 0/1 失败：旧 launcher 没有固定 pre-list diagnostic schema。最小修复只改 launcher 与 `acceptance_runtime_profile_tests.rs`：最终 child 的专用 ledger 在 post-exit receipt schema v3 中记录 child-kill 尝试、三种 parent signal、`exit`/`close` 与无原始 PID 的 PGID/session 布尔投影；`/bin/ps` 原始输出仅留内存。为不改变根目录/空 `logs/` 合同，诊断不在启动前创建任何 sidecar。若 launcher 收到可捕获 signal 且 child 已关闭，receipt 后才移除监听并 self-reraise；该动作与 child kill 分字段记录。
3. 离线实数：聚焦诊断合同 1/0；`cargo test acceptance_runtime_profile --lib --quiet` 18/0；`cargo check --lib` exit 0（既有 598 warnings）；`npm run typecheck`、`node --check`、目标 rustfmt、`git diff --check` 均 exit 0，staged 为空。没有重跑不在本诊断包内的历史 runner；shape/既有格式债继续单列。
4. 唯一允许的 `npm run r4:isolated-preflight` 启动：build 成功，ready envelope 之后 receipt 即报告 final child `SIGKILL`。由 [launcher-receipt-v5.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v5.json) 可证：child `exit` 与 `close` 都被 launcher 观察到，launcher 对 child 的 kill 尝试为 false，launcher 没有 self-reraise，parent 三个可捕获 signal 都为 false；PGID/session 投影为 unavailable，无法由该字段单独判断采样失败原因。其后仅以 ready envelope 中的精确自建 PID 做一次 `kill -0`，返回不存在；没有触碰其他进程。
5. 因 child 已退出，按合同没有调用 `sky.list_apps()`，更没有 attach、Home 读取、截图、点击、输入、UI observation、Gate 0、主管首句、CLI/MCP、`tools/list` 或任何 knowledge tool。没有重试。
6. 只读 macOS 元数据检查限定为该 child 与 receipt mtime 前后 70 秒、仅 `fault`/`error` 的 process-scoped 投影；结果是 `matching_record_present`，见 [process-scoped-termination-metadata-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/process-scoped-termination-metadata-v1.json)。没有保存 raw record、PID、命令、环境、路径、stdout 或 stderr。此事实既不能证明 macOS 发送了 `SIGKILL`，也不能排除 terminal executor 或其他外部主体；该 terminal session 的最终 timeout/终止原因仍不可恢复。

## 4.3 manual dev + isolated profile 单变量对照

1. 运行前只读确认 Syn、cargo-tauri、Vite 和 5173 均无残留，staged 为空。新 root 使用系统 temp API 创建为 0700；`profile.json`、fixture、workflow-state 与空 app-data/codex-db/logs 都是 synthetic，启动前根目录精确六项。
2. 对照复用用户刚才可运行的动态本地 dev config + `cargo-tauri dev` 链和同一个 `.cargo-target`，只增加 `SYN_R4_ACCEPTANCE_PROFILE`。输出进入 `Running .../.cargo-target/debug/codex-governance-workbench`，并打印 `storage mode=json_only`；其缺失配置路径在本轮 isolated root 内，未回落默认真实路径。
3. 启动约 60 秒后只读进程关系仍显示 cargo-tauri、Vite、Syn 全部存活；Syn 的 parent 是 cargo-tauri，且两者同 process group。此后用 Ctrl-C 正常停止；会后这三类进程均不存在，5173 无监听。
4. 全程没有 Computer Use、UI attach/read/click/screenshot、observation sidecar、主管首句、CLI/MCP、`tools/list`、knowledge tool 或十二项。收尾检查发现既有打开脚本以 `exec` 替换 shell 后，预先注册的 `EXIT` trap 没有运行，dynamic Tauri config 因而残留；只读确认该 0700 目录只含本轮单一 0644 config 后，已精确删除该文件与空目录。fresh synthetic root 按任务合同保留，仅新增 isolated log。
5. dev binary SHA-256 为 `5cb03670a4ca6c6228e76baca2955d8c5a0dd4640003b548d4ca9360625eb156`，此前 launcher fresh bundle executable 为 `00e87932a7d7d9f8f9e1390ada690c345e86a35ec6e34af65063ebcfaef75424`，两者均为 arm64 但并非同一字节。`codesign --verify --strict` 对 dev binary 为 valid；对 fresh `.app` 和其 executable 均报 `code has no resources but signature indicates they must be present`，使用 `--ignore-resources` 才为 valid。该静态失败是目前最强候选，但没有新的受控运行证明它就是 SIGKILL 原因，故不能把发送者写成 macOS/AMFI。

## 4.4 fresh bundle resource-seal 离线修复

1. 红合同 `acceptance_runtime_profile_launcher_seals_and_strictly_verifies_fresh_bundle_before_launch` 在实现前按预期为 0/1，失败点是 launcher 缺少固定 `CODESIGN_PATH`。合同要求绝对路径 `/usr/bin/codesign`、固定 ad-hoc seal 与 deep/strict verify 参数、禁止 `--ignore-resources`，并要求 seal/verify 的启动失败、非零退出或 signal 全部 fail closed。
2. 最小实现只在 fresh executable freshness 校验之后、`runDiagnosedChild` 之前执行 seal/verify；失败固定标为 `bundle_integrity`，不会启动最终 Syn。聚焦合同最终 1/0，完整 profile 套件最终 19/0。
3. 只离线运行 `cargo-tauri build --debug --bundles app`，没有执行 launcher 或 `.app`。新产物在 seal 前复现 `code has no resources but signature indicates they must be present`；执行与 launcher 相同的 `/usr/bin/codesign --force --deep --sign -` 后，`.app` 的 deep/strict verify、`Contents/MacOS/codex-governance-workbench` 的 strict verify 与 `Contents/_CodeSignature/CodeResources` 存在性检查均 exit 0。
4. `cargo check --lib` exit 0（598 条既有 warnings）；`npm run typecheck`、launcher syntax、目标测试文件 rustfmt、`git diff --check` 均 exit 0，staged 为空。
5. 这只关闭已确认的离线 resource-seal 缺口，不证明旧 `SIGKILL` 的发送者或运行时结果。当时下一步被限定为用户另行授权的一次 fresh-bundle pre-list 运行；仍不得自动进入 Computer Use、Gate 0、主管首句、CLI/MCP、工具或十二项。

## 4.5 sealed fresh bundle pre-list 单次重验

1. 运行前冻结 HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、porcelain 指纹 `cba82024a806126817e692f56d07bdc1daed09b2bce07299220eee62062a6387`、launcher SHA-256 `8ac98cec6ad931d121487ff84573dc1ba6f606984a7862fc5dad2adfe105b6a3`、合同测试 SHA-256 `fde75f802018f000172fbf7a4deb1a131c60517b18a29e47e43604dfa3f6b77c`，staged 为空。本线 Syn/cargo-tauri/5173 无残留；只看到另一工作区既有 5182 Vite，未触碰。
2. 唯一一次 `npm run r4:isolated-preflight` 先完成 fresh bundle build、ad-hoc seal 与 deep/strict verify，再输出 ready envelope。ready 后只读 `ps` 先确认 child 已存活 45 秒；继续等待后再次确认存活至少 88 秒。旧 v4/v5 的 pre-list `SIGKILL` 在本轮没有复现。
3. 全程未调用 `sky.list_apps()`、Computer Use 或任何 UI；`logs/ui-inspection.json` 在停止前仍不存在。超过历史退出窗口后，仅向 ready envelope 明示的精确自建 PID 发 SIGTERM。
4. launcher 随后按预期以 exit 1 停在 `failure_stage=ui_inspection`，因为本轮有意没有 UI observation；这不是启动失败。v6 receipt 记录 build `exit_code=0`、Syn `signal=SIGTERM`、child exit/close 均观察到 SIGTERM、launcher child-kill=false、parent 三种 signal=false、父子/process-group/session 均为 true、environment unchanged。receipt 不含绝对路径、PID、命令、环境值或 raw stdout/stderr。
5. v6 receipt 已与隔离根原件 `cmp` 一致，SHA-256 为 `2f3f3096e9ecc279ef053dd3a90400a4b8692ffb548fbb3303e4aaa7940a9fd7`；脱敏时长/边界投影 SHA-256 为 `bde4bd8f13cb7e188b8f800c5a8504a329ff524397c868c5c433d189eefcf610`。终止后精确 PID 不存在，本线 Syn/cargo-tauri/Vite 与 5173 均无残留。
6. 结论强度限定为“sealed fresh bundle 的旧 pre-list `SIGKILL` 在这一次没有复现”。没有读取 Home、没有 synthetic-only UI 结论、没有截图，也没有进入 Gate 0、主管首句、CLI/MCP、`tools/list`、knowledge tool 或十二项。

## 4.6 重复 pre-list 重验与正本刷新 catch

1. 本指导对话收到用户“可以”后，只复核了 launcher 合同、进程/5173 无残留和 staged 为空，没有先重读已在本次开工前更新的 CURRENT；因此没有看到 §4.5 已完成，重复执行了一次相同范围的 `npm run r4:isolated-preflight`。两轮不重叠，没有并发持有 App/build/runtime 资源。
2. 第二轮 fresh bundle 完成 build、seal/strict verify 并输出 ready；只读 `ps` 最终确认 Syn 连续存活至少 90 秒，旧 `SIGKILL` 再次没有复现。没有调用 `sky.list_apps()`、Computer Use、Home 读取、截图或 observation。
3. 通过 PTY Ctrl-C 受控结束后，v7 receipt 记录 child exit/close SIGINT、launcher child-kill=false、parent 收到 SIGINT 并在写 receipt 后 self-reraise、父子/process-group/session 均为 true。`failure_stage=ui_inspection` 仍是刻意没有 UI observation 的预期闭锁，不是启动失败。
4. 结束后本线 Syn/cargo-tauri/Vite 与 5173 均无残留；v7 与隔离根原件 `cmp` 一致。此前正本引用但 raw 目录缺失的 v6 也已从其保留 isolated root 恢复，v6/v7 分名保存，没有覆盖任何原件。
5. 该 catch 的结论是：每次真实/隔离 App 动作前不仅要扫进程和 staged，还必须重读 CURRENT/任务包的即时状态。第二轮结果只重复支持“不再复现旧 pre-list SIGKILL”，不得把证据数量升级为 I5 UI 验收。

## 5. 范围与未执行项

- 未读取或修改默认真实 Codex session DB、默认 vault、其他 vault、真实 index/tasks 或认证材料；未修改 `HOME` 或 `CODEX_HOME`。
- 未发送主管首句，未启动 Codex CLI/MCP server，未调用 `tools/list`、`submit_proposal` 或四项 knowledge tools，未进入 Gate 0 或十二项。
- 未进入 Knowledge/Agents。Home-only 限制仍必要：这些现有 UI 路由的 browser localStorage 不在本包隔离范围。
- 未 stage、commit、push、reset、clean、stash；没有删除任何 root。

## 6. 实际写入与最终 SHA

- 新增：`src-tauri/src/acceptance_runtime_profile.rs`（未改，`bcb0a8735757cf28642ff39a9e214fa1342a1ce056fd8a89f82e70a1b5e4c440`）、`src-tauri/src/acceptance_runtime_profile_tests.rs`（bundle-integrity 加固后 `fde75f802018f000172fbf7a4deb1a131c60517b18a29e47e43604dfa3f6b77c`）、`scripts/run-r4-isolated-app-preflight.mjs`（bundle-integrity 加固后 `8ac98cec6ad931d121487ff84573dc1ba6f606984a7862fc5dad2adfe105b6a3`）；本轮另在 merge-only `index_host_app_entrypoints.rs` 以固定启动失败出口收口（`f89a99ac41bc3c4627399781ad01f035cd2442766afdd8a2b165c0312ebabee7`）。
- 历史 raw receipt 为 `launcher-receipt.json`、`launcher-receipt-v2.json`、`launcher-receipt-v3.json`；其中 v3（`67b9e6486769a3f29efd62b53244065ef0a81cb47d233cf1f2bc8d15b58d6dad`）记录的是上一轮 `SIGTERM`，不能证明任一 `SIGKILL` 运行。`launcher-receipt-v4.json` 保留为上次 discovery-only `SIGKILL` 原件（SHA-256 `a56bc4879b5f124ba5c3e8cf7599a666cd3110fddcaae326daa846a86a161e6b`）；[launcher-receipt-v5.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v5.json) 是诊断运行原件（SHA-256 `10ba897b7c452f8739b829fb9a10561cb93dbac040871cdc9e94461133bb3092`）；[launcher-receipt-v6.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v6.json) 是第一轮 sealed fresh bundle pre-list 重验原件（SHA-256 `2f3f3096e9ecc279ef053dd3a90400a4b8692ffb548fbb3303e4aaa7940a9fd7`）；[launcher-receipt-v7.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/launcher-receipt-v7.json) 是重复重验原件（SHA-256 `efab175b4a946427109b42cbfad29c86d78bbf47607bd27eb08cfd8cc62223ed`），均与各自保留隔离根逐字节一致。配套 process-scoped metadata projection 的 SHA-256 为 `0ba1b0cc32ecdc579e32444885b53af8bb3582dea8ee58d8c8e6b818c9163340`；[manual-dev-isolated-profile-control-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/manual-dev-isolated-profile-control-v1.json) SHA-256 为 `1c04367637620e1c8257f6d530b8fc38c4f2bf0d353daf7d05b26cce5b16d869`；[fresh-bundle-pre-list-revalidation-v1.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/fresh-bundle-pre-list-revalidation-v1.json) SHA-256 为 `bde4bd8f13cb7e188b8f800c5a8504a329ff524397c868c5c433d189eefcf610`；[fresh-bundle-pre-list-revalidation-v2.json](raw/2026-07-24-l3-syn-r4-isolated-preflight/fresh-bundle-pre-list-revalidation-v2.json) SHA-256 为 `03e7112247b6cb748ce87df69b05eb3184ecc24377e11b5adcbad7808a7602ef`。这些原件/投影均不含绝对路径、PID、命令、环境值或 raw stdout/stderr，并已做 JSON/schema 与敏感字段静态检查。
- 此前 I2–I4 的其他 merge-only 接线为：`src-tauri/src/lib.rs`（`b0a88dcc9357…8787f333c`）、`codex_db.rs`（`8887abd8f1a0…877f5cc84`）、`knowledge_vault.rs`（`0df5240073e1…15ca7ef7c`）、`mcp/storage.rs`（`683fbc1c0831…878d3c86`）、`package.json`（`08a3abc466e2…6086d87e65`；仅 script registration）。
- 本 evidence、v1–v7 raw receipt、两版 pre-list 脱敏投影、process-scoped metadata projection、任务包、CURRENT、AUTHORITY 和真实 catch-log 为本包允许的文档写入。指导复核已在 catch-log 记录归因边界与本次未刷新 CURRENT 导致重复运行的真实拦截。

## 7. 当前停点

**PENDING_AUTHORIZED_I5_HOME_ONLY_UI_DISCOVERY**。sealed fresh bundle 的旧 pre-list `SIGKILL` 在两次不重叠的重验中均未复现，Syn ready 后分别存活至少 88 秒与 90 秒并受控退出；第二次是未先刷新 CURRENT 导致的冗余运行，不扩大结论强度。两次都没有调用 UI，所以 target 是否能被安全发现、Home 是否只呈现 synthetic 身份、截图/observation 是否能完成仍未知，不能宣称 I5 通过。下一步只可在用户新的明确授权下执行一次 Home-only UI target discovery；Gate 0、主管首句、Codex CLI/MCP、`tools/list`、任何工具调用和十二项继续锁定。
