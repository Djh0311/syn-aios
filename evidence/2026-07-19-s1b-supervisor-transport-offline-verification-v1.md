# S1B 主管一次一发传输离线与副本店 live 验证 v1

日期：2026-07-19
权威任务包：`tasks/2026-07-18-s1b-supervisor-transport-oneshot-resume-package-v1.md`
H1 收口：`tasks/2026-07-19-s1b-h1-live-mcp-approval-harness-package-v1.md`；详证 `evidence/2026-07-19-s1b-h1-live-mcp-approval-verification-v1.md`

## 已执行

- `cargo check --offline`：通过（仓内既有 594 个 warning，未把 warning 说成已清零）。
- `cargo test --offline s1b_ -- --nocapture`：16 passed、0 failed、1 ignored。
- `cargo test --offline supervisor_exec_registration_is_fail_closed_and_reapable -- --nocapture`：1 passed。
- `cargo test --offline s1_ -- --nocapture`：11 passed、0 failed、1 ignored。
- `cargo test --offline -q m5b_`：9 passed；`cargo test --offline -q m5c_`：5 passed。
- `cargo test --offline --lib`：1009 passed、0 failed、44 ignored。
- `pnpm typecheck`、`pnpm test:offline-interaction`（15 个离线交互组）、`git diff --check` 与改动 Rust 文件的 `rustfmt --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode baseline --json`：13 errors、5 warnings、5 infos，baseline 模式通过；`--mode check` 以同一 13 项历史债失败，**本包零净增**。

## 2026-07-19 invalid-resume 打回修正

- 案发夹具逐字保留真实 CLI stderr：`Reading prompt from stdin...` 后接 `thread/resume failed: no rollout found ... (code -32600)`；夹具先模拟 resume 的 durable 旧 thread 预绑定，但不伪造 stdout `thread.started`。
- 分类是结构性条件：仅当**resume 回合非零退出且未收到真实 stdout `thread.started`**，才进入 invalid-resume；不依赖或扩充任何单一拒票字符串。命中后沿既有单次路径 archive home → generation 换代 → facts 注入 → 同回合 initial exec，且 replacement initial 不递归换代。
- 离线案发测试覆盖：旧 thread 首轮 → 真 CLI 拒票 → 新 generation initial 通过真实本地 `submit_proposal` handler 落 `PendingUserConfirmation` → 后两轮均 resume 同一新 thread；workflow chain 保持未启动。
- 原始 stderr 只写 owner-only stderr artifact 和 orchestrator audit 的 private `parameter_summary`；投影给 pilot read model 的 `result_summary` 与最终用户错误均为人话，换代 initial 也失败时固定返回“主管这句没接上——再发一次或换个说法。”

## 定向覆盖

- 三个用户消息各自一次一发；首轮 `exec`、后两轮 `resume`，同一 thread 与项目私有 `CODEX_HOME` 保留。
- 私有 home 的 `config.toml` 被解析并断言仅有 `supervisor_orchestrator` MCP；auth 是符号链接，普通回合不清家，generation、项目元数据或白名单不一致即拒绝复用。
- 首轮 `thread.started` 同步持久化 binding 后，由 mock 在同回合调用**真实本地** `submit_proposal` handler / proposal store；结果为既有 `PendingUserConfirmation`，workflow chain 未启动。另有并发 race：工具先到、binding 后到，工具只会有界等待该 binding，绝不借旧 thread。
- silent watchdog 的一次重试、第二次静默人话停；若首轮已收到 `thread.started`，重试会 `resume` 同一 thread 而非另开 thread。invalid resume 会 archive home、换新 generation 并注入事实；清理未能确认进程组退出时保留 PID 与 `resident_turn_cleanup_failed`，直至重扫对账；dead `resident_turn_running`、有 binding/无 binding 的 `resident_turn_starting` 都会对账。
- 本地 `/bin/sh` 夹具让父子进程忽略 TERM，验证一次一发清理会继续做进程组 KILL sweep，最终 group 不存在；registry 单测同时断言只登记已核验的 `codex exec` 组。

## H1 修前的副本店 live 复核（历史诊断）

- 真实 CLI 用副本 store 预置 generation 5 的旧 thread；首轮 resume 非零且没有真实 stdout `thread.started` 后，实测自动归档旧家、换到 generation 6、注入项目事实并以 initial exec 建新 thread。后两轮均续同一 generation 6 thread，事实标记可回引；invalid-resume 自愈链成立。
- 回合前后 `ps` 对账零残留；固定测试项目未被这次副本店验证改写。
- 完整 ignored 场景继续要求模型调用真实 `submit_proposal`。本次模型已生成合法工具调用，但 Codex 客户端在私有 MCP handler 前返回 `user cancelled MCP tool call`；proposal 计数 74→74、唯一 marker=0、workflow chain 未启动。因此**真实工具落卡分支没有通过**，不是产品 handler 的失败证据。
- 07-19 用户拍板：把上述客户端批准可达性单列 **S1B-H1 harness 欠账**；它不阻塞 S1C 纯前端布局包，但在修复并复跑前不得声称 S1B 完整 live 落卡已过。

## S1B-H1 收口后的完整副本店 live

- 测试专用 wrapper 只暴露并预批准 `supervisor_orchestrator.submit_proposal`。initial 覆盖进入 `exec` 外层；resume 覆盖放在字面 `resume` 后，进入该子命令自己的 `-c` 解析层。wrapper 带 `--strict-config`，不设置 server-wide default、`approval_policy`、reviewer 或 sandbox，不含 full-auto/bypass；产品 argv、私有家 `config.toml` 与 handler 均未改。
- `s1b_live_resume_tool_card_and_replacement_require_explicit_harness_authorization` 已真实执行：**1 passed / 0 failed**，71.89s。
- proposal revision 131→132；唯一 marker `S1B_LIVE_CARD_MARIO_20260719` 落一张 `pending_user_confirmation` 卡；`supervisor_tool_call` 审计为 accepted。workflow chain 保持 40 条，不批准、不起链。
- 落卡后两轮续 generation 6 的同一 thread `019f76c0-6639-74d3-a3f4-0688e58498ed` 并回引事实 marker。随后置入不存在的 thread，真 CLI 非零且无 stdout `thread.started`，命中 `resume_exit_without_thread_started`；generation 6 归档，active 换 generation 7，新 thread `019f771a-ac76-7bc1-b92e-a8204cf92f9f` 由持久事实重建成功。
- 副本进程登记最终为空；未见本次 scratchpad、wrapper 或新旧测试 thread 的残留进程。同机一条指向真实 App store 的既有 supervisor MCP 进程不属于本测试，未动。

## 仍不声称通过

- 用例仍保留 `#[ignore]` 和精确确认变量，普通离线全量不会隐式调用真实 Codex。
- 本次是真 CLI + 副本 store + 既有 handler 证据，不等于真实 Tauri 交办页已经目视显示卡片。
- Pending 卡未获用户批准，完整“聊→工具落卡→批→跑”仍是下一次用户在场真机首单，不能由 H1 授权自动推进。
- 修前 `user cancelled MCP tool call` 只作为 harness 历史诊断，不改写成产品 handler 回归。
- shape check 的 13 项是既有历史债，不能表述为全绿；本文件只证明本包没有把它增成第 14 项。
