# 实现任务包：甲·中转 relay ③b-1「真发入口（env-gated 真 spawn codex；Codex 不真跑、真发留用户在场）」v1

日期：2026-06-18
出处：咨询线（Claude）。前序：③a 接通真 codex 配置层（Dirac `CLEAR` + 咨询审 PASS，`2e90792`）。排布：`docs/plans/2026-06-18-master-roadmap-phased-v1.md` 阶段 0 · 步 6 · ③b-1。

## 0. 接手须知

- 流水线：你写真发入口 + 结构/mock 测（**你绝不真跑 codex**）→ 独立复核 → 咨询线审实物 → 然后**用户在场跑那一次真发**（③b-1 真发本身不在本包）。
- 先读：本文 + `manual_relay.rs`（尤其 `real_codex_env_gated` 分支、`ensure_real_codex_env_authorized`、现有 ignored test `manual_relay_real_codex_requires_env_authorization`——它**只验 config、未真 run/spawn**，本包要补上"真 run”入口）+ `CURRENT.md` + `AGENTS.md`。**全程中文。子线不 commit。**
- **关键安全死线**：本包**写"能真发"的入口 + 结构测**；**Codex 自己不设 `MANUAL_RELAY_REAL_CODEX_CONFIRM`、不真 spawn codex、不真跑**。第一次真 spawn codex = 用户在场、单独步。

## 1. 拍板摘要

- **做什么**：现有 ③a 只到"真 codex 配置层"（ignored test 只验拿到 `program=codex` 的 config、没真 run/spawn）。本包补一个**真发入口**——env-gated ignored runner/test：设环境变量 → 真 `run_manual_relay_once(real_codex_env_gated)` → 真 spawn codex（stdin 送 prompt）→ readback last-message → 回执 + stop 能掐。
- **代价**：一个真发入口 + 结构测；做完"**就差用户在场跑那一次**"。
- **不批**：本包不真跑 codex、不解锁、不放宽旧闸、不接前端（那是 ③b-2）。

## 一句话判据

判改动在不在本包内——问：**「是不是在补 env-gated 真发入口（真 run→真 spawn codex 的路径 + 结构测），且 Codex 自己没设 env、没真跑 codex、没放宽旧闸、没接前端？」** 是 → 做；否（要真跑 / 发真项目 / 接前端 / 碰旧闸）→ 停、回咨询线。

## 2. 建什么

- **真发入口**（升级现有 ignored test 或新写一个 env-gated `#[ignore]` runner）：设 env → 真 `run_manual_relay_once` 走 `real_codex_env_gated` → 真 `spawn` codex（`program=codex`、stdin 送 prompt、`--output-last-message` 走 workbench-managed path）→ readback 只读 last-message → 回执（`real_codex_executed=true`、exit、readback、stop 能掐）。
- **第一次真发的最小安全设置**（写进入口默认 + runbook，用户可改）：
  - **target = 临时 fixture 项目**（temp dir + `git init`，**不是 mariotest**——第一次最小风险）；
  - **prompt = 最小无害**（如"在当前目录创建 `hello.txt`、写一行 hi"——可观察、可回滚、不动任何真项目）；
  - sandbox = workspace-write 限于该 temp 项目；timeout 合理（如 60s）。
- **结构/mock 测**（Codex 跑、**不真 spawn codex**）：mock-codex 桩验入口逻辑（已有）；env-gated 真 codex 入口仍 `#[ignore]`、Codex 不设 env、不跑。

## 3. 安全硬约束

- 本包 Codex **不真跑 codex**（不设 env、不跑真发入口）；真 spawn = 用户在场。
- 真发默认打 **temp fixture 项目 + 最小无害 prompt**（不碰 mariotest / 真项目）。
- **不放宽旧闸**（5 文件 diff 空）；**不新增 Tauri 命令**；不解锁通用真实执行；不接前端。
- `.codex`：读按新口径（读放开 / 写不碰）；真发时 codex CLI 自己正常写它的 `.codex`（用户机器），Syn 不额外读写。
- 回执 `real_codex_executed` 只在真 spawn 时 true；readback 只 last-message、不读 transcript/rollout。

## 4. TDD 验收门

- 没 env → 不 spawn（沿用 ③a）；有 env 的真发入口 `#[ignore]`、本包未跑；mock 桩验入口路径（真 run 形态、回执字段、stop 掐）；旧闸 5 文件 diff 空；命令净增 0。

## 5. 本包不做

- 不真跑 codex（用户在场）；不发 mariotest / 真项目（先 temp）；不接前端（③b-2）；不解锁 / 不放宽旧闸。

## 6. 用户在场第一次真发（③b-1 真发，本包之后）

用户在场、设 `MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY`、跑真发入口，看 **codex 真启动、真在 temp 项目创建 `hello.txt`、回执 `real_codex_executed=true`**——**Syn 第一次真启动 Codex 干活的验证**。验通后再考虑发 mariotest / 接 GUI（③b-2）。

## 7. 验证 + 回交

- 结构/mock 测 + **没真跑 codex 的证明** + 真发入口存在但锁死证据（`#[ignore]` + env-gate）+ 旧闸 diff 空 → 独立复核 → 咨询线审。

## 8. 不接受为

- 已真跑 codex / 已解锁 / 放宽旧闸 / 发了真项目 / 接了前端 / 回执假报 real_codex_executed。

---

*本文是真发入口实现包，不授权真跑 codex（③b-1 真发 = 用户在场、单独步）。*
