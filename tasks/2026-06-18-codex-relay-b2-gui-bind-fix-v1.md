# 修复任务包：甲·中转 relay ③b-2 GUI「点开 codex 对话不自动绑、发不出」bug v1

日期：2026-06-18
出处：咨询线（Claude）。用户 2026-06-18 真机实测：点开 codex 对话**绑不上、发不出**。前序：③b-2 GUI 直接发（`157738c`，审时只盯安全、漏了这个易用性 bug）。中转核心能力已验成（咨询线用 `codex exec` 真发 mariotest，codex 真收到回应，`evidence/2026-06-18-codex-relay-b1-first-real-relay-success-v1.md` 同类）。

## 0. 接手须知

- 这是 **bug 修复**（systematic-debugging）：**先在真机 app 复现确认根因，再改**；不盲改。
- 流水线：你复现 + 定位 + 修 + 真机验（Claude Preview / Tauri app 截图）+ mock/offline 测 → 独立复核 → 咨询线审 → 用户真机点一次确认。
- **不真跑 codex**（修的是绑定 UX，真发留用户在场）；不放宽 relay 安全闸 / 命根子（codex argv 沙箱限项目 + 拒审批绕过仍守）。

## 1. 现象 + 咨询线疑似根因（待你复现确认）

- **现象**：用户在 GUI 点开一个 codex 对话，撰写区**绑不上**（`relayDirectSendEnabled=false`）、Enter 发不出。
- **疑似根因（咨询线读代码定位，`AgentConversationShell.tsx`）**：
  1. `:200-201` `visibleSessions` 在 `conversationMode && selectedProjectRoot` 时**只显示 `session.project_root === selectedProjectRoot` 的会话** → 会话被「当前选中项目」过滤，用户可能选不到 / 选错目标 codex 会话。
  2. `:216-219` useEffect **仅当 `selectedSession?.project_root` 非空才 `setSelectedProjectRoot`** → 选中会话若 `project_root` 空 / 与当前 `selectedProjectRoot` 不一致，`selectedProjectRoot` 不跟上选中会话。
  3. `:205-207` `relayDirectSendEnabled` 卡 `selectedProjectRoot` → 上面两点导致它对不上选中会话 → `relayDirectSendBlockedReason`「未绑定项目」/绑不上。
- **先复现**：起前端（5173 已在跑）/真机 app，点一个 codex 会话，看 `relayDirectSendEnabled`、`selectedProjectRoot`、`selectedSession.project_root`、`softwareKeyOf` 实际值，钉死是上面哪条（或别的）。

## 2. 修复方向（= 用户要的「点开就自动绑」）

- **点开（选中）一个 codex 会话 = 自动把它的项目 + 会话绑成 relay target**：`selectedProjectRoot` **无条件跟随**选中会话（`selectedSession.project_root`），不被「当前项目过滤」卡死让用户选不到目标会话。
- `project_root` 空的兜底：用会话自己的 `target_cwd` / 让用户显式选项目，而不是直接 `relayDirectSendEnabled=false` 让人莫名发不出（至少把「为什么发不出」在 UI 上写清，别只是灰着）。
- 会话列表别因 `selectedProjectRoot` 过滤把目标 codex 会话隐藏掉（或给「看全部会话 / 切项目」入口）。

## 3. 安全硬约束（不放宽）

- **命根子仍守**：真发 codex argv `--sandbox workspace-write` + `--add-dir 项目根` + 拒 `--full-auto`/`dangerously-bypass`/`--approval*`（③b-2 已有，别动松）。
- **target 仍精确**：自动绑 ≠ 绑错——绑的必须是用户点开的那个会话的真实 project_root / canonical path，不靠猜、不 fallback 到错项目。
- 仍**手动一次一发**、不自动连环；不放宽旧闸（5 文件 diff 空）；Codex 本包不真跑 codex；mock/placeholder 仍 cfg(test)。

## 4. 验收

- 真机 app：点一个 codex 会话 → **自动绑**（target strip 亮、显示该会话项目/会话）→ Enter 能发（mock 验链路，真发留用户）。
- offline/交互测：选中 codex 会话→relayDirectSendEnabled=true；选中非 codex→「仅 Codex 会话可用」；project_root 空时 UI 写清原因。
- 旧闸 diff 空、命根子测试仍绿、`manual_relay` 测试绿、shape gate、build/typecheck。

## 5. 回交

- 复现记录 + 根因确认 + 修复 diff + 真机验证（Claude Preview/截图）+ 没真跑 codex 证明 + 命根子未放宽证明 → 独立复核 → 咨询线审 → 用户真机点一次确认绑上能发。

---

*本文是 GUI 绑定 bug 修复包，不授权真跑 codex（用户在场首发）、不放宽任何 relay 安全闸。*
