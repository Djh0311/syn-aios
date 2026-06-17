# 实现任务包：甲·中转 relay「真跑前必修 3 条」· 咨询线 → Codex v1

日期：2026-06-18

出自：咨询线（Claude）。性质：实现执行包。**这是 relay 计划 `docs/plans/2026-06-17-codex-relay-stepping-stone-plan-v1.md` 步骤 4（2026-06-18 补）的落地**。前序：mock 实现包 `tasks/2026-06-17-codex-relay-manual-implementation-v1.md`（Hegel `CLEAR_WITH_P2` + 咨询审 PASS `evidence/2026-06-18-codex-relay-manual-implementation-consulting-audit-v1.md`）。设计正本 `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`（尤其「Future implementation acceptance gates」+ §6 停/回滚）。

## 0. 接手须知

- 你是执行线。流水线：**你实现 + 测试（占位进程 / mock / env-gated，不真跑 Codex）→ 独立复核 → 咨询线（Claude）审实物（红队：必修 3 条真做实没？stop 真能掐死真进程？真 Codex 接入真锁死没？旧闸没放宽？没真跑 Codex？）→ 然后「第一次真 relay」才是单独一步、用户在场授权（③，不在本包）**。
- 先读：本文 + 设计正本 + 计划步骤 4 + `CURRENT.md` 首条 + `AGENTS.md` + `manual_relay.rs` 现状 + `codex_local_runner.rs:817` `run_real_codex_process`。**全程中文、术语标中文注释。子线不 `git add` / `git commit`。**
- **关键安全（本包死线）**：本包让 relay **具备**"真启动子进程 + 外部 stop 定向掐死 + 路径严校 + 确认原子"的能力,但 **spawn 的只能是无害占位进程(如 `sleep`)、绝不是真 Codex**;真 Codex 接入(program=codex + 真 prompt + 真 cwd)只写**env-gated `#[ignore]` runner**、**本包不跑**;**不真跑 Codex、不写真 `/Users/yoyi/.codex`、不放宽任何旧闸**;第一次真 Codex relay = 本包之后③、用户在场授权。

## 1. 拍板摘要

- **要做的事**：把 mock relay 跨到"**具备真跑能力 + 三道真护栏**"——补必修 3 条（路径精确 / 一次一发原子 / stop 真能掐死真进程），让 relay 离"真发 Codex"只差③用户在场解锁那一下。
- **代价**：一轮实现 + 测试;其中必修3 让 relay **第一次真 spawn 子进程**(占位、无害)、建进程句柄管理 + 外部 kill。
- **不做的后果**：mock relay 停在"逻辑成立"，真跑前的三个隐患（发错地方 / 一句跑两遍 / 停不掉）没兑现，③真 relay 不能安全开。
- **关键澄清**：本包**不真跑 Codex、不解锁真实执行、不放宽任何旧闸**;占位进程只为验证"能掐死真进程",真 Codex 接入 env-gated 锁死、留给③。

## 一句话判据

判某改动在不在本包内——问：**「是不是在补必修 3 条（路径严校 / 确认原子 / stop 真掐死占位进程）、且真 Codex 接入仍 env-gated 锁死、没真跑 Codex、没放宽任何旧闸？」** 是 → 做；否（尤其要解锁/改 `codex_local_runner` 真 Codex 启动闸、或要真跑 Codex）→ **停、回咨询线**。

## 2. 建什么（必修 3 条）

### 必修 1 · 路径精确（对应设计本分二「target 不靠 fallback 推断」）

- 现状：`manual_relay.rs::normalize_path_text`（行 758）`std::fs::canonicalize` 失败时走词法 `clean_path` 兜底。mock 路径不存在无害,但真跑时 symlink / 别名 / 不存在路径可能让 `target_hash` 失真、发错地方。
- 改：**run 真模式下,target 的 `project_root` / `target_cwd` / `allowed_write_roots` 必须 `canonicalize` 成功才放行**;失败(不存在 / 无权限)→ 阻断,**不用词法兜底当 target 指纹**。`preview` 层可保留词法显示,但须标 `path_verified=false`,且 `run` 在 `path_verified=false` 时拒绝。
- 测试：run 真模式遇 canonicalize 失败路径 → 拒;canonical 成功 → 放行;(可行则)symlink 与其目标归一为同一 `target_hash`。

### 必修 2 · 一次一发原子（对应设计本分三「一个 confirmation、terminal 后重新确认」）

- **前端(第一道、本就该有)**：relay 发送(confirm→run)触发后**立即清空输入框 + 发送/确认按钮 disabled 或 loading**,直到回执 terminal(完成 / 失败 / 停止)才解锁。offline 测试断言"发送后输入空 + 按钮禁用 + terminal 后恢复"。
- **后端(兜底)**：`consumed_confirmations` 现在"查(`contains_key`,行 327-333)+ 写(`insert`,行 400-403)"分两个 lock 窗口(TOCTOU)。改为**一把锁内原子 reserve/consume**(如 `entry` API:不存在才占住、占住即标记),使并发/重入双提交**只成功一次**。
- 测试：并发或重入用同一 `confirmation_id` 双提交,只一个成功、另一个被拒(`confirmation_already_consumed` 或 `reserved`)。

### 必修 3 · stop 真能掐死真进程(对应设计 §6「无法可点击 stop 就不许声称能停」)——本包最重

- 现状：`manual_relay.rs::run_manual_relay_once` 调 `fixture_receipt`、**完全不 spawn**;`stop` 只从内存 registry remove + 标 `killed_by_user`(行 408)。`codex_local_runner::run_real_codex_process`(行 817)虽有 spawn + 超时 kill,但 **child 句柄不外露、同步阻塞、无"外部 stop 定向 kill"通道**。
- 建(新机制)：
  1. relay run 增加**真进程模式**:真 `spawn` 一个子进程,**spawn 后立即把 child 句柄 / pid 登记进 `active_attempts`**(现在只存 `receipt`,要能定向 kill),**不能同步阻塞到进程结束**(否则 stop 插不进)——用后台线程 / 异步等待 + 句柄登记。
  2. `stop_manual_relay_attempt` 从 registry 取该 attempt 的 child / pid,**真 kill**,wait 回收,断言进程真终止;回执记 `killed_by_user=true` + `real_process_killed=true`。
  3. **验证只用无害占位进程**:program = `sleep`(或一个立即退出 / 可控时长的测试脚本),**绝不是 codex**。测试:spawn 占位进程(长命)→ 登记 → stop → 进程真的没了(pid 不可 wait / 已回收)。
  4. duplicate / 一次一发 / 路径 / secret-deny 等既有护栏在真进程模式下仍成立。
- **真 Codex 接入(锁死、本包不跑)**：把 program 设为 codex + 真 prompt(stdin)+ 真 cwd + 真 sandbox 的路径,**只写成 env-gated `#[ignore]` runner**(沿用项目既有 env-gated ignored 模式),平时 / CI 不跑;解锁 = ③用户在场设环境变量 / GUI 真按。relay 真 Codex 走**自己的 env-gate**,**不借道 `run_real_resume_phase_b_with_runner` 等旧闸、不改 `codex_local_runner` 的真 Codex 启动闸**。

## 3. 安全硬约束（本包死线，必须成立）

- **不真跑 Codex**：本包只 spawn 无害占位进程验 stop;真 Codex 路径 env-gated `#[ignore]`、本包不跑。
- **不放宽任何旧闸**：`run_real_resume_phase_b_with_runner()` 授权矩阵、K3-B1 recovery / K3-B2 gate、H5/PCR product command、`inspect_codex_local_execution_guard()`、real-resume 门**都不动**;diff 必须为空。relay 真 Codex 接入走新 env-gate,不借道旧闸。
- **不写 `.codex`**：占位进程不碰 `.codex`;真 Codex 由 Codex CLI 自己跑时正常写(③),Syn 不额外读写 `.codex` 正文 / auth / token / rollout。
- **三本分维持**：原话逐字(`effective_prompt==original`、`payload_layers` 空)不变;必修1 强化本分二、必修2 强化本分三。
- **碰锁就停**：若实现必修3 发现"必须解锁 / 改 `codex_local_runner` 真 Codex 启动闸"或"必须真跑 Codex 才能验证"→ **停、回咨询线重定范围**,不自己解闸、不自己真跑。
- **stop 做不出真掐死**：若占位进程也无法实现可点击定向 kill、只能靠 timeout → **不许声称能停**,退回咨询线(沿用设计 §6)。

## 4. TDD 验收门（测试钉死）

- 必修1：run 真模式遇 canonicalize 失败路径被拒;canonical 成功放行;`preview` 词法路径标 `path_verified=false` 且 run 拒之。
- 必修2：前端发送后输入清空 + 按钮禁用、terminal 后恢复(offline);后端同一 confirmation 并发/重入双提交只成功一次。
- 必修3：spawn 占位进程(长命)→ stop → 进程真终止(断言不可再 wait / 已回收);回执 `real_process_killed=true`、`real_codex_executed=false`;真 Codex runner 为 `#[ignore]` env-gated、本包未运行(证明其存在但锁死)。
- regression：K3-B1 / K3-B2 / H2 real-resume / H3 / H5 product command + `codex_local_runner` real-resume 门测试证明**旧门未放宽**;旧闸 5 文件 diff 空。
- 全量：`cargo test --lib` / `cargo test --lib manual_relay` / `npm run typecheck` / `test:offline-interaction` / `build` / `cargo fmt -- --check` / shape gate / `git diff --check`。

## 5. 本包不做（deferred）

- **不真跑 Codex**(占位进程验 stop;真 Codex env-gated 不跑)。
- 不解锁 K3-B1 / B2 / real-resume / 乙·工作流连环 / 多 agent 并行 / 通用真实执行授权。
- 不上线角色 / 任务包 / 记忆注入(`payload_layers` 仍空)。
- 不做自动 retry / 自动 rollback / 自动 stop / memory formalization。

## 6. 第一次真 relay（③，单独步、本包之后、用户在场）

本包**实现 + 测试通过 + 独立复核 + 咨询线审**后,③第一次真 relay 是**单独一步**:**用户在场**,看 exact payload + target,**显式授权语句**,设环境变量 / GUI 真按,才把 program 换成真 codex、第一次真启动一次 Codex。**本包不含真跑。**

## 7. 验证 + 回交

- 跑 §4 各门;回交：实现 diff + evidence（测试输出 + 必修3 占位进程"真启动→真掐死→进程真没"的证据 + **没真跑 Codex 的证明** + 真 Codex runner env-gated 锁死的证据 + 旧闸 diff 空）→ 独立复核 → 咨询线审。

## 8. 不接受为

- 不接受为已真跑 Codex / 已解锁真实执行 / 弱化了任何旧闸 / 真 Codex 接入未锁死(非 env-gated)/ stop 做不出真掐死却声称能停 / 必修任意一条只改了字段没真做实 / 写了 `.codex`。

---

*本文是实现执行包,不授权真跑 Codex（③单独步、用户在场）。需扩范围或要碰旧闸先回咨询线。*
