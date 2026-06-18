# 交接：甲·中转 relay — 当前状态 + 坑 + 重启建议（供新对话接手）v1

日期：2026-06-18
出处：咨询线（Claude），本对话。
触发：用户 2026-06-18 对现有 GUI relay 真机体验极度不满（原话"一坨狗屎"），叫停修复，要开新对话重新干。本文交接真实状态，供新对话 + 用户基于实物决定怎么重启，**不粉饰**。

## TL;DR

relay 甲·中转的**后端安全机制 + 命令行中转**做扎实了、且验成了（`codex exec` 把用户原话真发 mariotest、codex 真收到真回）；但 **GUI 真机真用体验是痛点**——用户实测点 codex 会话反复"绑不上 / 发不出"，分 6 步的实现被批"绕远路"。用户要新对话**重新干**（很可能是重做/简化"中转 UI"，而非继续打补丁）。

## 1. 当前真实状态

**已做 + 已提交**（main，今日 commit 链）：
- `b99f16c` ① mock 实现（manual_relay contract + 4 Tauri 命令 + 前端 relay 模式）
- `a65e6d7` ② 必修 3 条（路径严校 / 一次一发原子 / stop 真杀占位进程）
- `2e90792` ③a 接通真 codex（env-gated 双锁、未真跑）
- `197fd98` ③b-1 真发入口（ignored + env-gate + temp 无害 prompt）
- `9b7360a` 🎯 第一次真发 codex 成功（temp 真建 hello.txt，实物铁证）
- `157738c` ③b-2 GUI 直接发（绑 codex 会话 Enter 直发 + codex argv 沙箱限项目 + 拒审批绕过）
- `e53f32a` GUI bind bug 修复（点 codex 会话从真实 project_root 自动绑、跨项目不隐藏）

**能用 / 已验成**：
- **命令行中转**：`codex exec --sandbox read-only` 在 mariotest（`/Users/yoyi/codex-workflow-mario-test`）把用户原话真发，codex 真收到真回（session `019ed9f7`、gpt-5.5）。→ "话 → codex 真到达"**成立**。
- **后端 `manual_relay.rs` 安全机制（扎实、建议保留别推翻）**：三本分（原话逐字 / target 精确 / 手动一次一发）、命根子（真发 codex argv `--sandbox workspace-write` + `--add-dir 项目根` + 拒 `--full-auto`/`dangerously-bypass`/`--approval`）、stop 真杀、回执、placeholder/mock `cfg(test)` only。每步独立复核 + 咨询审 PASS。

**卡 / 没验通**：
- **GUI 真机"点发"用户实测失败 / 体验糟**。bind fix 后理论上"点 codex 会话→自动绑→Enter 发"，但**用户真机点发未验通**（macOS 拒 computer-use 自动点击、Codex 没替点；用户自己点反复挫败）。
- 用户对整套 GUI relay 流程极度不满，叫停。

## 2. 坑 / 为什么卡（诚实，含咨询线 mistake）

- **GUI 易用性反复出问题**：③b-2 绑定要"选中 codex 会话 + 有 project_root + software==codex"，"点开对话"不直觉地自动绑，反复"绑不上"。审 ③b-2 时咨询线**只盯安全、漏了易用 bug**（mistake）。
- **relay 分步过重，被批"绕远路"**：六步每步独立复核+审+提交。而"把一句话中转给 codex"本质 = 一行 `codex exec`。教训：**应对「用户要的简单事」先用最直方式做成，再谈安全分步**。
- **真机 GUI 验证难**：computer-use 要用户在 Claude app 授系统权限（辅助 + 录屏）；macOS 拒自动点击；Claude Preview 连 Vite 5173 但 Tauri `invoke` 不可用 → GUI 真用一直没被 agent 自己验通，全靠用户手点（反复挫败）。

## 3. 资产（新对话直接复用，别重造）

- 后端 `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`：relay 全套安全机制。**好的，别推翻**。
- `codex exec` 直发已验证（命令行中转通）。
- 前端 `src/views/agent/AgentConversationShell.tsx` / `AgentChatComposer.tsx`：GUI direct relay（绑定 + target strip + Enter 直发）——**做了但真机体验是痛点**。
- Tauri app dev 跑法：`prototypes/tauri-capability-probe/.tauri-cli/bin/cargo-tauri dev --config '{"build":{"beforeDevCommand":""}}' --no-dev-server-wait`（复用现有 5173 Vite）。

## 4. 重启可能方向（新对话 + 用户定，别替定）

- **A. 只修 GUI 真机体验**：用户在场真机点（或解决 computer-use 权限），把"点 codex 会话→自动绑→发"真机验通。最小改动。
- **B. 重做"中转 UI"**：弃现有绑定流程，做一个最直接的"打字→发给 codex"UI（更贴用户心智）。
- **C. 简化 relay**：评估六步分法是否过度，能否合并/砍。
- 注：后端安全机制（§3）建议保留，重做主要在前端 UX。

## 5. 正本 / 关键文件指针

- 当前事实：`CURRENT.md`「当前结论」line5 + checkpoints。
- 分阶段总图：`docs/plans/2026-06-18-master-roadmap-phased-v1.md`。
- relay 计划/设计：`docs/plans/2026-06-17-codex-relay-stepping-stone-plan-v1.md`、`...-design-draft-v1.md`。
- 各步任务包 + evidence：`tasks/2026-06-1[78]-codex-relay-*`、`evidence/2026-06-18-codex-relay-*`。
- 前端：`AgentConversationShell.tsx`（绑定逻辑）、`AgentChatComposer.tsx`（target strip + 直发）。后端：`manual_relay.rs`。

## 6. 给新对话 Claude 的协作提醒（重要）

- 用户是 vibe coder，要**直接、快、能用**；**极度反感"绕远路" + 被"教做事 / 建议节奏"**（记忆 `feedback-dont-coach-user`、`feedback-plain-language`、`feedback-discussion-mode`）。
- relay 定位 = **等于直接用 codex**，别过度包装；用户要的简单事**先用最直方式做成**（如 `codex exec`），别一上来六步安全分装。
- **GUI 改动早做真机验证**（别只 offline 测就说做好，用户一点就穿帮）。
- `.codex` 读取已放宽（记忆 `feedback-codex-home-read-allowed`）。
- 安全硬线仍在：真跑 codex 不可逆要用户授权、命根子（codex 审批/沙箱）不放松、不碰旧闸；`git commit` 问一次。

---

*本文不粉饰：后端扎实、命令行中转验成；GUI 真用体验是痛点，用户叫停要重启。新对话基于实物 + §4，与用户定怎么重做。*
