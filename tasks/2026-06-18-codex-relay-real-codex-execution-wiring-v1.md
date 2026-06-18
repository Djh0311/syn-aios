# 实现任务包：甲·中转 relay③a「接通真 codex 执行路径（仍 env-gated 锁死）」· 咨询线 → Codex v1

日期：2026-06-18

出自：咨询线（Claude）。性质：实现执行包，**relay 链条最敏感一环**。前序：必修 3 条实现包 `tasks/2026-06-18-codex-relay-pre-real-relay-must-fix-implementation-v1.md`（Darwin `CLEAR_WITH_NOTE` + 咨询审 PASS `evidence/2026-06-18-codex-relay-pre-real-relay-must-fix-consulting-audit-v1.md`）。计划步骤见 `docs/plans/2026-06-17-codex-relay-stepping-stone-plan-v1.md` 步骤 4/5。

## 0. 接手须知

- 你是执行线。流水线：**你实现 + 测试（假 codex 测试桩 / env-gated ignored，绝不跑真 codex）→ 独立复核 → 咨询线（Claude）审实物 → 然后③b「第一次真 codex relay」才是单独一步、用户在场授权**。
- 先读：本文 + `manual_relay.rs` 现状（尤其 `real_codex_env_gated` 分支 `:332`、`spawn_placeholder_process` `:839`、`stop_manual_relay_attempt` `:455`、`build_codex_local_request` 的 `readback_plan`）+ `CURRENT.md` 首条 + `AGENTS.md`。**全程中文、术语标中文注释。子线不 `git add` / `git commit`。**
- **关键安全（本包死线）**：③a **只接通"能真启动 codex"的代码 + 环境变量锁死**；**③a 绝不真跑 codex**——用**假 codex 测试桩**（一个写 last-message 后立即退出的本地脚本，冒充 codex）验证接通逻辑；真 codex（`program=codex`）只走 `#[ignore]` + env-gated、③a 不运行。**不读 full transcript / rollout / `.codex` 任何文件（含浏览器插件 skill，见 M-0004）；不放宽任何旧闸。第一次真 codex relay = ③b 用户在场授权，不在本包。**
- **M-0004 边界**：实现 / 验证（尤其任何浏览器或真机尝试）**不得读 `/Users/yoyi/.codex` 下任何文件**，包括浏览器插件 skill。需要浏览器验证就停、回咨询线。

## 1. 拍板摘要

- **要做的事**：把 manual_relay 的真 codex 执行路径从"直接报错占位"**接通为真能启动 codex**——但**用环境变量锁死**，平时 / CI 仍不跑；用假 codex 测试桩验证"启动→读回最后输出→停→回执"整条逻辑。
- **代价**：一轮实现 + 测试；**做完后 Syn 就具备"真启动 codex"的代码能力（虽锁着）——这是质变**：从"全程零真跑"到"差用户在场解锁那一下"。
- **不做的后果**：真 codex 路径停在"直接报错"，③b 没有可解锁的真实路径，relay 永远到不了"真发"。
- **关键澄清**：③a **不真跑 codex、不解锁、不放宽旧闸**；它把真实路径搭好 + 锁死 + 用测试桩验逻辑。解锁真跑 = ③b、用户在场、显式授权。

## 一句话判据

判某改动在不在③a 内——问：**「是不是在接通 manual_relay 自己的 env-gated 真 codex 执行路径、用假 codex 测试桩验逻辑、真 codex 仍 `#[ignore]`+环境变量锁死、没真跑 codex、没读 transcript/rollout/`.codex`、没放宽任何旧闸？」** 是 → 做；否（尤其要真跑 codex、要碰 `codex_local_runner` 的 real-resume 授权矩阵 / H2/PCR 闸、要读 `.codex`）→ **停、回咨询线**。

## 2. 建什么（接通真 codex 执行路径）

- **接通 `real_codex_env_gated` 模式**：现状是直接 `Err("manual_relay_real_codex_env_gated_not_enabled_in_this_package")`（`manual_relay.rs:332`）。改为：在**环境变量授权**（`MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`，沿用②已定的名）**且**通过既有全部护栏（路径严校 / hash / confirmation 一次性 / duplicate / secret-deny / payload exact）时，**真启动 codex**：用 guard 已生成的 codex command_plan（`program=codex`、stdin 写 prompt、`--output-last-message` 走 workbench-managed path），**复用②已建的 child 句柄登记 / `stop` 定向 kill / 回执机制**；没授权 → 仍 `Err`、不跑。
- **优先在 `manual_relay` 内自包含接通**（把②的占位 spawn 机制扩为"真 program + stdin prompt + readback last-message"）。**只在确需复用 `codex_local_runner::run_real_codex_process` 的纯进程逻辑、且完全不牵动其 real-resume 授权矩阵 / H2 / PCR 闸时**才复用；**一旦发现接通要改动或借道任何旧闸 → 停、回咨询线**。
- **readback 边界**：真跑后只读 workbench-managed last-message（沿用 `readback_plan` 的 `last_message_only_no_full_transcript_read`）；**不读 full transcript / rollout 正文 / `.codex`**。回执 `real_codex_executed=true`（真跑时）、`exit_code` / `readback_status` / `last_message_hash`，`syn_read_codex_home=false` / `syn_wrote_codex_home=false`。
- **假 codex 测试桩**（③a 验证用，不跑真 codex）：新增一个"mock codex"模式——`program` 指向一个本地测试脚本（写一行到 last-message path 后立即退出，**绝不是 codex**），跑通"启动 → 读回 last-message → 回执"整条逻辑 + `stop` 能掐死它。以此证明接通逻辑正确，**而不触碰真 codex**。

## 3. 安全硬约束（本包死线，必须成立）

- **③a 不真跑 codex**：自动测试只用假 codex 测试桩 + 负向（无授权不跑）；真 codex（`program=codex`）路径走 `#[ignore]` + 环境变量，**③a 不运行它**。
- **真 codex 双锁**：环境变量未设 → `Err`、不跑；`#[ignore]` 测试默认不跑。两把锁缺一不可。
- **不读敏感**：不读 auth / token / secret / `.env` / keychain / OAuth / credential / full transcript / rollout 正文 / `.codex` 任何文件（含浏览器插件 skill，M-0004）。
- **不写 `.codex`**：真 codex 由 codex CLI 自己跑时正常写（③b），Syn 不额外读写。
- **不放宽 / 不借道任何旧闸**：`run_real_resume_phase_b_with_runner()` 授权矩阵、K3-B1/B2、H5/PCR、`inspect_codex_local_execution_guard()` 的必填项**都不动**；relay 真 codex 走自己的 env-gate，不伪装成 H2/real-resume。旧闸 5 文件 diff 应空（若复用 `run_real_codex_process` 纯逻辑导致 `codex_local_runner.rs` 有 diff，必须是**不改其授权/闸**的纯调用，且在 evidence 里逐行说明 + 停一下让咨询线先看）。
- **三本分维持**：原话逐字（`payload_layers` 空、`effective_prompt==original`）、target 精确、手动一次一发，全不变。
- **碰线就停**：要真跑 codex / 要改旧闸 / 要读 `.codex` / 要浏览器真机验证 → **停、回咨询线**，不自己越线。

## 4. TDD 验收门（测试钉死）

- 无环境变量授权 → `real_codex_env_gated` 仍 `Err`、不 spawn 任何进程。
- 有环境变量授权（在 `#[ignore]` 测试里）→ 走到真启动路径；但该测试 `#[ignore]`、③a 默认不跑（证明真 codex 路径存在但锁死）。
- 假 codex 测试桩：启动 → 读回 last-message → 回执 `real_codex_executed`/`readback_status`/`last_message_hash` 正确；`stop` 能掐死测试桩、`real_process_killed=true`。
- readback 只读 last-message：测试证明不读 full transcript / rollout / `.codex`。
- 既有护栏在真 codex 模式下仍拦：路径未验证 / hash 不匹配 / confirmation 复用 / duplicate / secret-deny 仍 `Err`。
- regression：旧闸 5 文件测试证明未放宽；旧闸 diff 空（或纯调用 + 逐行说明）。
- 全量：`cargo test --lib` / `cargo test --lib manual_relay` / `npm run typecheck` / `test:offline-interaction` / `build` / `cargo fmt -- --check` / shape gate / `git diff --check`。

## 5. 本包不做（deferred）

- **不真跑 codex**（= ③b，用户在场）。
- 不解锁 K3-B1/B2 / real-resume / 乙·工作流连环 / 多 agent 并行 / 通用真实执行授权。
- 不上线角色 / 任务包 / 记忆注入（`payload_layers` 仍空）。
- 不做自动 retry / 自动 rollback / 自动 stop / 自动连环 / memory formalization。
- 不做浏览器真机验证（碰 `.codex` 风险，见 M-0004）。

## 6. ③b 第一次真 codex relay（单独步、本包之后、用户在场）

③a **接通 + 测试桩验通 + 复核 + 咨询审**后，③b 是**单独一步**：**用户在场**，看 exact payload + target，**显式授权语句**，设 `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY` / GUI 真按，**第一次真启动一次 codex**。这一步是接通逻辑的**第一次正向验证**（可能暴露接通 bug，当场看）。**③a 不含真跑。**

## 7. 验证 + 回交

- 跑 §4 各门；回交：实现 diff + evidence（测试桩"启动→readback→stop→回执"证据 + **没真跑 codex 的证明** + 真 codex 路径 env-gated 锁死证据 + readback 只 last-message 证据 + 旧闸 diff 空或纯调用逐行说明 + 没读 `.codex` 证明）→ 独立复核 → 咨询线审。

## 8. 不接受为

- 不接受为已真跑 codex / 已解锁真实执行 / 弱化或借道了任何旧闸 / 真 codex 未双锁 / 读了 transcript·rollout·`.codex` / 做了浏览器真机验证碰 `.codex` / payload 加料 / 能自动连环。

---

*本文是实现执行包，不授权真跑 codex（③b 单独步、用户在场）。需真跑 / 碰旧闸 / 读 `.codex` / 浏览器真机验证，先回咨询线。*
