# 实现任务包：甲·中转 relay ③b-2「GUI 直接发（会话即 target · 打字回车直发 · target 常驻可见）」v1

日期：2026-06-18
出处：咨询线（Claude）+ 用户 2026-06-18 拍板「最后要做成可以直接发的」。前序：③b-1 第一次真发成功（`evidence/2026-06-18-codex-relay-b1-first-real-relay-success-v1.md`）。排布：`docs/plans/2026-06-18-master-roadmap-phased-v1.md` 阶段 0 · ③b-2。

## 0. 接手须知

- 流水线：你实现 GUI 直接发链路 + mock 结构测（**你不真跑 codex**）→ 独立复核 → 咨询线审实物 → **用户在场 GUI 首次真发（先 temp）**。
- 先读：本文 + `manual_relay.rs`（真 codex 路径 / command_plan / 回执 / stop）+ **`codex_local_runner.rs` 真启动的 codex argv（审批/sandbox 模式）** + 前端 `AgentChatComposer.tsx`/`AgentConversationShell.tsx` + `CURRENT.md` + `AGENTS.md`。**全程中文。子线不 commit。**
- **关键安全死线**：本包实现 GUI 直接发链路 + mock 测；**Codex 自己不真跑 codex**；首次 GUI 真发 = 用户在场、先 temp。

## 1. 拍板摘要

- **做什么**：终态做成**直接发**——在绑定了 codex 会话的撰写区里**打字、回车，直接真发给那个 codex**，像直接用 codex 一样顺。**无"真发模式开关"**。
- **代价**：一轮前后端实现 + mock 测。**安全级跃迁**：产品 GUI 直接真发 codex。仍**手动一次一发、非自动连环**。
- **关键安全模型（务必理解）**：去掉 Syn 的二次确认后，**防"codex 乱改"靠 codex 自己的审批 / sandbox（= 等于用户直接用 codex）**；Syn 守的只剩「**发对 target（常驻可见）+ 原话逐字 + 一次一发**」。**前提（命根子）：relay 真发给 codex 的审批/sandbox 不得比"直接用 codex"更松**。
- **不批**：本包 Codex 不真跑 codex；不解锁自动连环 / 多 agent；不放宽旧闸；不放松 codex 审批；首次真发用户在场。

## 一句话判据

判改动在不在本包内——问：**「是不是在实现『会话即 target、打字回车直接真发、target 常驻可见』的 GUI 直发链路，守『发对 target + 原话逐字 + 一次一发』，且 codex 审批/sandbox 不放松、Codex 自己没真跑 codex、没解锁自动连环、没碰旧闸？」** 是 → 做；否 → 停、回咨询线。

## 2. 建什么

- **前端（直接发）**：
  - 绑定 codex 会话/项目的撰写区，**打字 → 回车 / 发送，直接真发给该会话的 codex**（无模式开关、无二次确认弹窗）。
  - **target 常驻可见**：撰写区 / 会话头常驻显示「↔ [codex 项目名 / canonical path / 会话]」——用户始终一眼知道"在真发、发给谁"（防发错项目）。
  - 回执回流 GUI（codex 结果 / changed files / `real_codex_executed` / exit / readback）；**stop 按钮**停正在跑的真发。
  - 区分"真发会话"与"非真发（草稿/秘书）"靠**会话绑定本身**（绑了 codex 的会话才真发），不靠开关。
- **后端**：GUI 直发命令走 `real_codex_env_gated` 真 spawn（复用 ③a/③b-1 已审路径），**去环境变量门**；靠 target canonical + hash 校验、confirmation 一次一发、duplicate 阻断、原话逐字守。命令面 +1（说明）。
- **codex 审批/sandbox（命根子，必须定死 + 写进 evidence）**：relay 真发构造的 codex argv **不得用 `--full-auto` / `--dangerously-bypass-approvals` 等绕过 codex 审批**；sandbox 至少限于 target 项目（workspace-write 限项目根 / 或更严）。**即：relay 真发的"改文件风险"由 codex 自身审批/sandbox 兜底，等于或严于用户直接用 codex。** 实现时核实并在 evidence 写明实际 argv + sandbox。
- **实现期不真跑**：GUI 直发逻辑用 mock-codex 桩 + offline 交互测；Codex 不真启动 codex；首次真 codex 经 GUI = 用户在场。

## 3. 安全硬约束

- **三本分**：① 原话逐字（`effective_prompt==original`、`payload_layers` 空）② **target 精确**（canonical + hash、**常驻可见**、对不上即拒、不靠猜）③ **手动一次一发**（一条一发、`auto_chain=false`、同 target running 阻断）。
- **codex 审批不放松**（§2 命根子）：不绕审批、sandbox 限项目；evidence 写明实际 codex argv。
- **第一次 GUI 真发先连 temp / 受控项目**；**mariotest / 真项目要用户显式选 / 绑定**（不默认、不隐式推断）。
- 不放宽旧闸（5 文件 diff 空）；mock/placeholder 仍 cfg(test)；不解锁自动连环 / 多 agent / K3-B1/B2。
- 去 env 门仅限"产品 GUI 直发"运行时；Codex 实现期 mock 测、不真跑。
- `.codex`：读按新口径、写不碰。

## 4. TDD 验收门

- 绑 codex 会话的撰写区：回车 → 走真发链路（mock 桩验：target 校验 / 一次一发 / 回执 / stop）；非绑定会话不真发。
- target 不匹配 / 未绑 → 拒；duplicate running → 阻断；`auto_chain` 永远 false。
- **codex argv 审批不放松**：测试 / evidence 证明真发 argv 不含 `--full-auto` 类绕审批、sandbox 限项目。
- offline 交互测：常驻 target 显示、回车直发（无弹窗）、回执 / stop。
- 旧闸 5 文件 diff 空；mock/placeholder 仍 cfg(test)；Codex 实现期未真跑 codex（证明）。
- 全量：`cargo test --lib` / `manual_relay` / `typecheck` / `test:offline-interaction` / `build` / `fmt` / shape gate（命令 +1 说明）/ `git diff --check`。

## 5. 本包不做

- Codex 不真跑 codex（首次 = 用户在场）；不默认发真项目（先 temp / 显式绑）；不解锁自动连环 / 多 agent；不动旧闸；不放松 codex 审批；不上线角色 / 任务包 / 记忆注入。

## 6. 用户在场第一次 GUI 真发（本包之后）

用户在场：在绑 temp 受控项目的会话撰写区，看常驻"↔ [temp 项目]"，回车真发一句 → 看 codex 真跑、回执、changed files、**codex 审批层是否如预期兜底**。验通后放开绑定 mariotest / 真项目，日常直接发。

## 7. 验证 + 回交

- 跑 §4 各门；回交：前后端 diff + evidence（mock 测直发链路 + **Codex 没真跑 codex 的证明** + **真发 codex argv 审批/sandbox 实录** + 三本分守住 + 旧闸 diff 空 + 命令 +1 说明）→ 独立复核 → 咨询线审。

## 8. 不接受为

- 已真跑 codex / 默认发了真项目 / **codex 审批被放松（--full-auto 类）** / target 靠猜或不可见 / 能自动连环 / 放宽旧闸。

---

*本文是 GUI 直接发实现包，不授权真跑 codex（首次 = 用户在场、先 temp）。终态"直接发"= 等于直接用 codex；防误改靠 codex 自身审批，Syn 守发对 target + 原话 + 一次一发。*
