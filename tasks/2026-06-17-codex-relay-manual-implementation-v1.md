# 实现任务包：Codex 中转（甲）manual relay · 咨询线 → Codex v1

日期：2026-06-17

出自：咨询线（Claude）。性质：实现执行包。**设计正本 = `docs/plans/2026-06-17-codex-relay-stepping-stone-design-draft-v1.md`（用户 2026-06-17 拍板）**；计划 = `docs/plans/2026-06-17-codex-relay-stepping-stone-plan-v1.md`。二者冲突以设计正本 + 本文硬约束为准。

## 0. 接手须知

- 你是执行线。流水线：**你实现 + 测试（用 mock / fixtures、不真跑 Codex）→ 独立复核 → 咨询线（Claude）审（红队：合设计？旧闸没放宽？测试钉死？没真跑？stop 真能停？）→ 然后「第一次真 relay」是单独一步、用户在场授权**。
- 先读：设计正本 + 计划 + `CURRENT.md` 首条 + `AGENTS.md` + 设计里点到的源码。**全程中文、术语标中文注释。子线不 `git add` / `git commit`。**
- **关键安全（本包死线）**：本包 = **建机制 + 测试**，用 **mock / fixtures runner**；**不真跑 Codex、不真发模型、不写真 `/Users/yoyi/.codex`**。**第一次真 relay（真启动 Codex）是本包之后的单独一步、用户在场授权，不在本包内。**

## 1. 拍板摘要

- **要做的事**：实现设计里那条「老实 manual relay」窄入口——后端 `manual_relay` contract + 4 个窄 Tauri 命令 + 前端 relay 模式 + 回执，全部按设计的安全约束；用 mock 测、不真跑。
- **代价**：一轮实现 + 一批钉死安全的测试。
- **不做的后果**：设计停在纸面，Syn 仍"看得见、动不了"。
- **关键澄清**：本包**不真跑 Codex、不解锁执行、不放宽任何旧闸**；它只是把设计的机制建出来 + 用测试钉死，为"用户在场的第一次真 relay"备好。

## 一句话判据

判某改动在不在本包内——问：**「是不是在实现设计正本里那条 manual relay（手动一次一发 / 原话可见 / 指定 target / 不拆旧闸 / 留角色口），且用 mock 测、没真跑 Codex、没放宽任何现有真实执行闸？」** 是 → 做；否 → 停、回咨询线。

## 2. 建什么（build scope，依设计 §4/§5/§6）

- **后端 `manual_relay.rs`**：`ManualRelayEnvelope`（含 `payload_layers[]` 空 + `future_hooks` 预留，护栏 2）/ `ManualRelayGuard` / `ManualRelayAttempt` / `ManualRelayReceipt`。复用 `codex_local_runner` 的安全构件（结构化 command plan / stdin prompt / last-message path / timeout kill / target canonicalization / secret deny-list / readback），但走**新的 manual_relay contract**——**不伪装成 H2 continuation 或 K3-B1 recovery**。
- **4 个窄 Tauri 命令**（名仅建议）：`preview_manual_codex_relay`、`confirm_manual_codex_relay_once`、`run_manual_codex_relay_once`、`stop_manual_codex_relay_attempt`。`preview`/`confirm` 不调 runner；`run` 只在 `confirmation_id` + prompt hash + target hash + sandbox + write roots **全匹配**时启动一次。
- **前端**：新增 `send_mode="manual_relay"` 的显式按钮 / 二段确认（**旧 decision-only 发送原样保留**，不改其语义）；发送前展示 **exact payload + target（项目名 / canonical path / 会话 / sandbox / 写入根）**；pending message 用 relay 专属元数据；回流复用 last-message。新增窄 handler `handleSubmitManualRelayOnce()`，**不改** `handleSubmitConversationDraft()`。

## 3. 安全硬约束（必须成立，依设计）

- **手动一次一发**：每次只认一个 `confirmation_id`；`auto_chain=false` 永远；attempt terminal 后须重新确认；同一 target session 有 running attempt → 默认**阻断**。
- **原话可见**：`effective_prompt == original_user_text` 逐字；无隐藏 wrapper / 记忆 / 角色 / 任务包；prompt body **不入** runtime log / audit log。
- **target 精确**：canonical path + target hash 对上才发；**不靠标题 / 最近 / fallback 猜**；new session 必须显式选。
- **安全级 = 直接用 Codex**：不读 auth / token / secret / full transcript / rollout body；不写 `.codex`（`.codex` 只由 Codex CLI 自己跑时正常写）；外发只有 Codex 调模型那一下；**secret / auth / `.env` / keychain / OAuth / credential / 完整 transcript / `.codex` 内容的读取请求 → 阻断或转单独高风险**。
- **不拆 / 不弱化任何旧闸**：`run_real_resume_phase_b_with_runner()` 授权矩阵、K3-B1 recovery / K3-B2 gate、H5/PCR product command、`inspect_codex_local_execution_guard()` **都不动**；要支持 relay 的更轻字段 → **新增 relay guard 或新增严格分支，不删既有必填项**。不新增自动连环 / 后台 worker / 多 agent 并行 / 通用真实执行授权。
- **停**：`stop` 只 kill 本 attempt 的 child（维护 attempt → pid 句柄）；不碰其它 Codex / session / worker；receipt 记 `killed_by_user`。**做不出可点击 stop（只能靠 timeout）→ 不许声称"能停"，退回咨询线重定范围。**
- **回滚保守**：跑前记 target repo HEAD / dirty / changed-files hash；**脏工作树默认不自动 `git reset`/`checkout`**，只给变更清单 + 手动恢复建议；clean tree 可生成 rollback suggestion，真恢复另批；非 git 只给清单 / 备份建议。
- **回执**：依设计 §6 字段（attempt/confirmation id、target、prompt sha256/length/是否 exact、command redacted preview（program+argv+stdin、无 shell）、起止/exit/timed_out/killed_by_user/readback、`real_codex_executed`/`syn_wrote_codex_home=false`/`syn_read_codex_home=false`、changed files、git before/after）。

## 4. TDD 验收门（用测试钉死，依设计「Future implementation acceptance gates」）

- UI：旧 decision-only 发送**仍不真跑**；manual relay 必须显 payload + target + one-shot warning。
- 后端：prompt hash / target hash / confirmation id 不匹配 → 阻断。
- 后端：running duplicate attempt 被阻断；`auto_chain` 永远 false。
- 后端：secret / token / `.env` / keychain / OAuth / credential / 完整 transcript / rollout / `.codex` 内容读取请求 → 阻断或进单独高风险路径。
- runner：command plan 无 shell、prompt 走 stdin、last message 在 workbench-managed run dir。
- stop：只 kill 当前 attempt；receipt 记 stop 结果。
- rollback：dirty tree 不自动 destructive revert；clean tree 可生成 rollback suggestion、真恢复另批。
- regression：K3-B1 / K3-B2 / H2 real resume / H3 new session / H5 product command 测试证明**旧门未放宽**。

## 5. 本包不做（deferred）

- **不真跑 Codex**（本包用 mock / fixtures runner 测；真跑见 §6）。
- 不上线角色 / 任务包 / 记忆注入（`payload_layers` 留空）。
- 不解锁 K3-B1 / B2 / 乙·工作流连环 / 多 agent 并行 / 通用真实执行授权。
- 不自动 stop / retry / rollback / memory formalization。

## 6. 第一次真 relay（单独一步、本包之后、用户在场）

本包**实现 + mock 测通过 + 复核 + 咨询线审**后，第一次真 relay 是**单独一步**：**用户在场**，看 exact payload + target，**显式授权语句**，才真启动一次 Codex（可用 env-gated ignored 真 runner 或 GUI，用户在场）。**本包不含真跑。**

## 7. 验证 + 回交

- `cargo test --lib`（含 §4 各门）/ `npm run typecheck` / `test:offline-interaction` / `build` / `cargo fmt -- --check` / shape gate / `git diff --check`。
- 回交：实现 diff + evidence（测试输出 + 各 guard 钉死证据 + **没真跑 Codex 的证明** + stop 是真可点还是 degraded）→ 独立复核 → 咨询线审。

## 8. 不接受为

- 不接受为已真跑 Codex / 已解锁真实执行 / 弱化了任何旧闸 / payload 加了料（非 exact original）/ 能自动连环 / stop 做不出却声称能停 / 写了 `.codex`。

---

*本文是实现执行包，不授权真跑 Codex（§6 单独步、用户在场）。需扩范围先回咨询线。*
