# 咨询线审计：甲·中转 relay ③b-2 GUI bind 修复 v1

日期：2026-06-18
审计线：咨询线（Claude）
审计对象：Codex GUI bind 修复（提交前工作树）；任务包 `tasks/2026-06-18-codex-relay-b2-gui-bind-fix-v1.md`。
前序复核：Erdos，`STATUS: CLEAR`，无 P0/P1/P2。
背景：用户真机实测「点开 codex 对话绑不上、发不出」；咨询线代码定位根因（会话被项目过滤 + selectedProjectRoot 没跟随选中会话），交 Codex 修。

## 结论

**STATUS: PASS（CLEAR）**。修对了——target 从选中会话真实 project_root 绑、跨项目 codex 会话不隐藏、缺项目禁用不猜；后端 relay 安全门 / 命根子未动。**残留：用户真机「点发」未验**（macOS 拒自动点击，Codex 未替点）——需用户重启 app 点一次确认能发。

## 实物核验

- **绑定改对**（`AgentConversationShell.tsx`）：`targetProjectRoot = (selectedSession.project_root ?? "").trim()`——**从选中会话真实 project_root 绑**，不吃旧 `selectedProjectRoot` 下拉状态；缺 project_root / 非 codex → **disabled（禁用），不猜测 fallback**（Erdos 同核）。
- **去项目过滤**：`visibleSessions` / `conversationSessionOptions` 改用全部 sessions、不再按当前项目过滤 → 跨项目 codex 会话可选（不被隐藏）。
- **发送一致**：`target_project_root` / `target_cwd` / `allowed_write_roots` 统一用 `relayTargetProjectRoot`（绑定项目），composer 收同值，避免旧 stale 项目。
- **后端 / 旧闸未动**：`manual_relay.rs` / `command_registry.rs` / `commands.rs` / 旧闸 5 文件 `git diff` 全空 → **命根子（codex argv 沙箱限项目 + 拒审批绕过）原样守住**；命令数 105 不变（纯前端）。
- 重跑 `cargo test --lib manual_relay` 16 passed / 2 ignored（咨询线亲跑，无回归）；offline 回归测覆盖（绑定 / 缺项目阻断 / 非 codex 阻断 / 跨项目不隐藏）；真机截图 `tauri-after-bind-fix.png`。
- Codex **未真发 codex**（未设 env、未点发送、未跑 ignored）。

## 边界 / 残留

未真发 codex、未碰后端安全门、未放宽旧闸。**真机「点 codex 会话 → 自动绑 → Enter 真发」需用户重启 app 亲点一次确认**——这是甲·中转 GUI 真用的最终验证。提交注意：`.playwright-cli/`、`tmp/b2-gui-real-send-browser/` 临时目录不进 git。不得据此声称：用户已真机验过能发 / 通用真实执行已解锁。
